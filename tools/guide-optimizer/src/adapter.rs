//! The GEPA adapter: a candidate is one guide text; an evaluation asks the
//! model for packs under that guide and judges them; a proposal reflects on
//! the judged replies and must pass the verification rules to be tried.

use crate::model::{Metered, NOT_RUN};
use crate::task::{Language, Verdict, judge, pack_text};
use gepa::Reflective;
use gepa::{
    Candidate, EvalBatch, GepaAdapter, ReflectiveSample, extract_new_instruction, render_prompt,
};
use lesscode_core::specimen_runner::sha256_hex_bytes;
use lesscode_model::repair::{self, Outcome, with_repairs};
use lesscode_model::{CallError, CallRequest, Stop};
use serde_json::{Value, json};
use std::cell::Cell;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

pub const COMPONENT: &str = "guide";
pub const DEFAULT_MAX_TOKENS: (u32, u32) = (4_000, 8_000);
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
    /// Output limits of an authoring call and of a reflection call.
    pub max_tokens: (u32, u32),
    /// Repair calls allowed after a rejected first reply.
    pub repairs: usize,
    /// Whether the request carries the grammar's node kind names.
    pub node_kinds: bool,
    last: Vec<Trial>,
}

/// The word a record uses for how a reply stopped.
fn stop_word(stop: &Stop) -> &str {
    match stop {
        Stop::Complete => "complete",
        Stop::CutOff => "cut_off",
        Stop::Refused => "refusal",
        Stop::Other(word) => word,
    }
}

/// One call in the record of a trial.
fn entry(admitted: bool, stop: &str, tokens: (u64, u64), feedback: &[String]) -> Value {
    json!({
        "admitted": admitted, "stop_reason": stop, "input_tokens": tokens.0,
        "output_tokens": tokens.1, "feedback": feedback,
    })
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
            max_tokens: DEFAULT_MAX_TOKENS,
            repairs: 0,
            node_kinds: false,
            last: Vec::new(),
        }
    }

    /// A first call and, while the pack is rejected, at most `repairs` more,
    /// made by the seam's repair loop. A reply that was cut off or refused is
    /// rejected unjudged; a call that fails ends the trial.
    fn one(&self, phase: &str, guide: &str, language: &str) -> Trial {
        let task = &self.languages[language];
        let request = CallRequest {
            purpose: "author".into(),
            system: guide.to_owned(),
            user: if self.node_kinds {
                format!("{}\n\n{}", task.request, task.kinds)
            } else {
                task.request.clone()
            },
            max_output_tokens: self.max_tokens.0,
            temperature: None,
            structure: None,
        };
        // The check sees only the text; the call notes how the reply stopped.
        let stop = Cell::new(None);
        let mut call = |asked: &CallRequest| {
            self.model
                .chat(asked)
                .inspect(|reply| stop.set(Some(reply.stop.clone())))
        };
        let mut judged = None;
        let mut check = |text: &str| {
            let verdict = match stop.take() {
                Some(Stop::CutOff) => {
                    failed("the reply was cut off before the pack was complete".into())
                }
                Some(Stop::Refused) => failed("the request was refused".into()),
                _ => judge(task, &pack_text(text)),
            };
            let seen = repair::Verdict {
                passed: verdict.admitted,
                reasons: verdict.feedback.clone(),
            };
            judged = Some(verdict);
            seen
        };
        let outcome = with_repairs(&mut call, &request, &mut check, self.repairs);
        let (attempts, reply, verdict, failure) = match outcome {
            Outcome::Accepted { reply, attempts }
            | Outcome::Rejected {
                last: reply,
                attempts,
            } => {
                let verdict = judged.expect("the last reply was checked");
                (attempts, reply.text, verdict, None)
            }
            Outcome::Failed { error, attempts } => {
                let word = if error == CallError::Config(NOT_RUN.into()) {
                    "not_run"
                } else {
                    "error"
                };
                (
                    attempts,
                    String::new(),
                    failed(format!("no reply: {error}")),
                    Some(word),
                )
            }
        };
        let mut calls: Vec<Value> = attempts
            .iter()
            .map(|a| {
                let tokens = (a.input_tokens, a.output_tokens);
                entry(a.passed, stop_word(&a.stop), tokens, &a.reasons)
            })
            .collect();
        if let Some(word) = failure {
            calls.push(entry(false, word, (0, 0), &verdict.feedback));
        }
        let tokens = attempts.iter().fold((0, 0), |(i, o), a| {
            (i + a.input_tokens, o + a.output_tokens)
        });
        let record = json!({
            "phase": phase, "guide": guide_id(guide), "language": language, "score": verdict.score,
            "admitted": verdict.admitted, "same": verdict.same, "behaviour": verdict.behaviour,
            "input_tokens": tokens.0, "output_tokens": tokens.1, "stop_reason": calls[calls.len() - 1]["stop_reason"],
            "feedback": verdict.feedback, "first_admitted": calls[0]["admitted"], "attempts": calls,
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
        let request = CallRequest {
            purpose: "reflect".into(),
            system: String::new(),
            user: render_prompt(guide_of(candidate), &dataset, Some(&template)),
            max_output_tokens: self.max_tokens.1,
            temperature: None,
            structure: None,
        };
        let reply = self.model.chat(&request).map_err(|e| e.reason())?;
        if reply.stop == Stop::Refused {
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

#[cfg(test)]
mod trials {
    use super::*;
    use crate::model::Model;
    use lesscode_model::{Reply, Sent};
    use std::path::{Path, PathBuf};

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// Answers with the shipped Java pack, ending as stated.
    struct Ending(Stop);

    impl Model for Ending {
        fn name(&self) -> String {
            "scripted".into()
        }

        fn chat(&self, request: &CallRequest) -> Result<Reply, CallError> {
            let text = std::fs::read_to_string(repo().join("native/packs/java.json"))
                .map_err(|e| CallError::Config(e.to_string()))?;
            Ok(Reply {
                text,
                stop: self.0.clone(),
                input_tokens: 1,
                output_tokens: 2,
                reasoning_tokens: None,
                cached_input_tokens: None,
                structured: None,
                seconds: 0.0,
                waits: Vec::new(),
                sent: Sent {
                    host: "scripted".into(),
                    model: self.name(),
                    max_output_tokens: request.max_output_tokens,
                    temperature: None,
                    reasoning_level: None,
                },
            })
        }
    }

    /// The record of one Java trial with one repair allowed.
    fn trial(name: &str, stop: Stop, max_usd: f64) -> Value {
        let ledger = std::env::temp_dir().join(format!(
            "guide-optimizer-{name}-{}.jsonl",
            std::process::id()
        ));
        let model = Metered::new(
            Box::new(Ending(stop)),
            (0.0, 0.0),
            max_usd,
            f64::INFINITY,
            ledger.clone(),
        );
        // Without control sets the judge runs the admission alone.
        let java = Language {
            name: "java".into(),
            request: "Write the pack for: java".into(),
            kinds: String::new(),
            scopes: Vec::new(),
        };
        let mut authoring = Authoring::new(
            Arc::new(model),
            Arc::new(BTreeMap::from([("java".to_owned(), java)])),
            Vec::new(),
            Vec::new(),
            Arc::new(Rules::new("", Vec::new())),
        );
        authoring.repairs = 1;
        let record = authoring.one("final", "guide", "java").record;
        let _ = std::fs::remove_file(ledger);
        record
    }

    fn calls(record: &Value) -> usize {
        record["attempts"].as_array().map_or(0, Vec::len)
    }

    #[test]
    fn a_reply_that_was_cut_off_or_refused_is_never_admitted() {
        // The same pack, complete, is admitted at once.
        let whole = trial("complete", Stop::Complete, 1.0);
        assert_eq!(whole["admitted"], true);
        assert_eq!(whole["stop_reason"], "complete");
        assert_eq!(
            (calls(&whole), whole["output_tokens"].as_u64()),
            (1, Some(2))
        );
        for (stop, word, reason) in [
            (
                Stop::CutOff,
                "cut_off",
                "the reply was cut off before the pack was complete",
            ),
            (Stop::Refused, "refusal", "the request was refused"),
        ] {
            let record = trial(word, stop, 1.0);
            assert_eq!(record["admitted"], false);
            assert_eq!(record["stop_reason"], word);
            assert_eq!(record["feedback"], json!([reason]));
            // Asked again as it was, and the tokens of both calls counted.
            assert_eq!(
                (calls(&record), record["output_tokens"].as_u64()),
                (2, Some(4))
            );
        }
    }

    #[test]
    fn a_call_the_budget_stops_is_recorded_as_not_run() {
        let record = trial("not-run", Stop::Complete, -1.0);
        assert_eq!(record["stop_reason"], "not_run");
        assert_eq!(record["first_admitted"], false);
        assert_eq!(calls(&record), 1);
        assert_eq!(record["attempts"][0]["stop_reason"], "not_run");
    }
}
