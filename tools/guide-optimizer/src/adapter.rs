//! The GEPA adapter: a candidate is one guide text; an evaluation asks the
//! model for packs under that guide and judges them; a proposal reflects on
//! the judged replies and must pass the verification rules to be tried.

use crate::model::Metered;
use crate::task::{Language, Verdict, judge, pack_text};
use gepa::Reflective;
use gepa::{
    Candidate, EvalBatch, GepaAdapter, ReflectiveSample, extract_new_instruction, render_prompt,
};
use lesscode_core::specimen_runner::sha256_hex_bytes;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

pub const COMPONENT: &str = "guide";
const AUTHOR_MAX_TOKENS: u64 = 4_000;
const REFLECT_MAX_TOKENS: u64 = 8_000;
const SECTIONS: [&str; 4] = ["## Fields", "## Operators", "## Rules", "## Output"];

/// GEPA's reflection prompt with the limits a new guide must respect.
const REFLECTION_TEMPLATE: &str = r#"I provided an assistant with the following instructions to perform a task for me:
```
<curr_param>
```

The following are examples of different task inputs provided to the assistant along with the assistant's response for each of them, and some feedback on how the assistant's response could be better:
```
<side_info>
```

Your task is to write a new instruction for the assistant.

Read all the assistant responses and the corresponding feedback. Work out what the instruction failed to make clear and say it in general terms, so that it helps for any programming language.

Limits the new instruction must respect, or it is discarded unread:
- Keep the first paragraph, which states the purpose, exactly as it is.
- Keep the sections "Fields", "Operators", "Rules" and "Output".
- It applies to every language: do not name a programming language and do not write syntax node names that belong to one grammar. Describe constructs in words.
- Do not copy names of checks from the feedback.
- At most <max_chars> characters.

Provide the new instructions within ``` blocks."#;

pub fn guide_id(guide: &str) -> String {
    sha256_hex_bytes(guide.as_bytes())[..12].to_owned()
}

/// What a candidate guide must satisfy before it is tried.
pub struct Rules {
    pub purpose: String,
    pub max_chars: usize,
    pub banned: Vec<String>,
}

impl Rules {
    pub fn new(baseline: &str, banned: Vec<String>) -> Self {
        let purpose = baseline
            .split("\n\n")
            .nth(1)
            .unwrap_or_default()
            .trim()
            .to_owned();
        let known: Vec<&str> = words(baseline).collect();
        let banned = banned
            .into_iter()
            .filter(|b| b.contains('_') && !known.contains(&b.as_str()))
            .collect();
        Rules {
            purpose,
            max_chars: baseline.chars().count() * 3 / 2,
            banned,
        }
    }

    pub fn check(&self, guide: &str) -> Result<(), String> {
        if self.purpose.is_empty() || !guide.contains(&self.purpose) {
            return Err("the statement of purpose was changed".into());
        }
        if let Some(missing) = SECTIONS.iter().find(|s| !guide.contains(**s)) {
            return Err(format!("section {missing} is missing"));
        }
        let chars = guide.chars().count();
        if chars > self.max_chars {
            return Err(format!("{chars} characters, more than {}", self.max_chars));
        }
        match words(guide).find(|w| self.banned.iter().any(|b| b == w)) {
            Some(word) => Err(format!(
                "it names {word}, which belongs to one grammar or to a check"
            )),
            None => Ok(()),
        }
    }
}

fn words(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|w| !w.is_empty())
}

/// One authoring call and its verdict, as logged.
#[derive(Clone)]
pub struct Trial {
    pub phase: String,
    pub guide: String,
    pub language: String,
    pub verdict_score: f64,
    pub record: Value,
    pub reply: String,
    pub feedback: Vec<String>,
}

pub struct Authoring {
    pub model: Arc<Metered>,
    pub languages: Arc<BTreeMap<String, Language>>,
    pub train: Vec<String>,
    pub val: Vec<String>,
    pub rules: Arc<Rules>,
    pub log: Arc<Mutex<Vec<Trial>>>,
    pub rejected: Arc<Mutex<Vec<Value>>>,
    last: Vec<Trial>,
}

impl Authoring {
    pub fn new(
        model: Arc<Metered>,
        languages: Arc<BTreeMap<String, Language>>,
        train: Vec<String>,
        val: Vec<String>,
        rules: Arc<Rules>,
    ) -> Self {
        Authoring {
            model,
            languages,
            train,
            val,
            rules,
            log: Arc::default(),
            rejected: Arc::default(),
            last: Vec::new(),
        }
    }

    fn one(&self, phase: &str, guide: &str, language: &str) -> Trial {
        let task = &self.languages[language];
        let (reply, verdict, tokens, stop) =
            match self
                .model
                .chat("author", guide, &task.request, AUTHOR_MAX_TOKENS)
            {
                Err(e) => (
                    String::new(),
                    failed(format!("no reply: {e}")),
                    (0, 0),
                    "error".to_owned(),
                ),
                Ok(reply) if reply.stop_reason == "refusal" => (
                    reply.text,
                    failed("the request was refused".into()),
                    (reply.input_tokens, reply.output_tokens),
                    "refusal".to_owned(),
                ),
                Ok(reply) => {
                    let mut verdict = judge(task, &pack_text(&reply.text));
                    if matches!(reply.stop_reason.as_str(), "max_tokens" | "length") {
                        verdict =
                            failed("the reply was cut off before the pack was complete".into());
                    }
                    (
                        reply.text,
                        verdict,
                        (reply.input_tokens, reply.output_tokens),
                        reply.stop_reason,
                    )
                }
            };
        let record = json!({
            "phase": phase, "guide": guide_id(guide), "language": language, "score": verdict.score,
            "admitted": verdict.admitted, "same": verdict.same, "behaviour": verdict.behaviour,
            "input_tokens": tokens.0, "output_tokens": tokens.1, "stop_reason": stop, "feedback": verdict.feedback,
        });
        Trial {
            phase: phase.to_owned(),
            guide: guide.to_owned(),
            language: language.to_owned(),
            verdict_score: verdict.score,
            record,
            reply,
            feedback: verdict.feedback,
        }
    }

    /// Fresh authoring calls for the languages given, at most four at a time.
    pub fn run(&self, phase: &str, guide: &str, languages: &[String]) -> Vec<Trial> {
        let mut trials = Vec::new();
        for chunk in languages.chunks(4) {
            let done: Vec<Trial> = std::thread::scope(|scope| {
                let handles: Vec<_> = chunk
                    .iter()
                    .map(|language| scope.spawn(move || self.one(phase, guide, language)))
                    .collect();
                handles
                    .into_iter()
                    .map(|h| h.join().expect("a trial does not panic"))
                    .collect()
            });
            trials.extend(done);
        }
        self.log.lock().unwrap().extend(trials.iter().cloned());
        trials
    }

    fn picked(&self, set: &[String], ids: &[usize]) -> Vec<String> {
        ids.iter().map(|i| set[*i].clone()).collect()
    }
}

fn failed(reason: String) -> Verdict {
    Verdict {
        score: 0.0,
        admitted: false,
        same: false,
        feedback: vec![reason],
        behaviour: String::new(),
    }
}

fn scores(trials: &[Trial]) -> Vec<f64> {
    trials.iter().map(|t| t.verdict_score).collect()
}

fn guide_of(candidate: &Candidate) -> &str {
    candidate
        .get(COMPONENT)
        .map(String::as_str)
        .unwrap_or_default()
}

impl GepaAdapter for Authoring {
    type Output = ();

    async fn evaluate_minibatch(
        &mut self,
        ids: &[usize],
        candidate: &Candidate,
        capture_traces: bool,
    ) -> EvalBatch<()> {
        let trials = self.run(
            "minibatch",
            guide_of(candidate),
            &self.picked(&self.train, ids),
        );
        let result = scores(&trials);
        if capture_traces {
            self.last = trials;
            EvalBatch::traced(result)
        } else {
            EvalBatch::scored(result)
        }
    }

    async fn evaluate_valset(&mut self, candidate: &Candidate) -> EvalBatch<()> {
        EvalBatch::scored(scores(&self.run("valset", guide_of(candidate), &self.val)))
    }

    async fn evaluate_valset_ids(&mut self, ids: &[usize], candidate: &Candidate) -> EvalBatch<()> {
        EvalBatch::scored(scores(&self.run(
            "valset",
            guide_of(candidate),
            &self.picked(&self.val, ids),
        )))
    }

    async fn propose_new_texts(
        &mut self,
        candidate: &Candidate,
        _components: &[String],
        _captured: &EvalBatch<()>,
    ) -> Result<Candidate, String> {
        if self.last.is_empty() {
            return Err("No valid predictions found for any module.".into());
        }
        let dataset: Vec<ReflectiveSample> = self
            .last
            .iter()
            .map(|trial| {
                let feedback = if trial.feedback.is_empty() {
                    "Accepted: it passed every check and behaves like the reference.".to_owned()
                } else if trial.verdict_score > 0.0 {
                    format!(
                        "It passed the checks but behaves differently from the reference: {}",
                        trial.feedback.join("; ")
                    )
                } else {
                    format!("Rejected by the checks: {}", trial.feedback.join("; "))
                };
                vec![
                    (
                        "Inputs".to_owned(),
                        Reflective::Text(format!("Write the pack for: {}", trial.language)),
                    ),
                    (
                        "Generated Outputs".to_owned(),
                        Reflective::Text(pack_text(&trial.reply)),
                    ),
                    ("Feedback".to_owned(), Reflective::Text(feedback)),
                ]
            })
            .collect();
        let template =
            REFLECTION_TEMPLATE.replace("<max_chars>", &self.rules.max_chars.to_string());
        let prompt = render_prompt(guide_of(candidate), &dataset, Some(&template));
        let reply = self
            .model
            .chat("reflect", "", &prompt, REFLECT_MAX_TOKENS)?;
        if reply.stop_reason == "refusal" {
            return Err("the reflection request was refused".into());
        }
        let proposed = extract_new_instruction(&reply.text);
        if let Err(reason) = self.rules.check(&proposed) {
            self.rejected
                .lock()
                .unwrap()
                .push(json!({"reason": reason, "guide": guide_id(&proposed), "text": proposed}));
            return Err(format!("the proposed guide was discarded: {reason}"));
        }
        Ok(Candidate::from([(COMPONENT.to_owned(), proposed)]))
    }
}
