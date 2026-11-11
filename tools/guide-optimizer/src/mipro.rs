//! MIPROv2 as far as it applies to this task: one instruction, no
//! demonstrations to search over, and a scoring set far below the size at
//! which MIPROv2 scores on subsamples.
//!
//! What is kept from dsrust's MIPROv2 (crates/dsrust/src/optimize/mipro, see
//! vendor/README.md): the proposer's wording and its six tips, candidate
//! zero being the starting instruction, and the trial loop of `search.rs`
//! in its whole-set form, driven by the copied TPE sampler. What is left
//! out because this task has nothing for it: bootstrapped demonstration
//! sets, the program and module descriptions, minibatch trials. The
//! dataset description is written by hand instead of by model calls.

use crate::adapter::{Authoring, guide_id};
use crate::task::pack_text;
use gepa::extract_new_instruction;
use serde_json::{Value, json};

/// dsrust `grounded.rs` TIPS, in its order; the empty first entry means no tip.
const TIPS: [&str; 6] = [
    "",
    "Don't be afraid to be creative when creating the new instruction!",
    "Keep the instruction clear and concise.",
    "Make sure your instruction is very informative and descriptive.",
    "The instruction should include a high stakes scenario in which the LM must solve the task!",
    "Include a persona that is relevant to the task in the instruction (ie. \"You are a ...\")",
];

/// dsrust `signatures.rs`, `generate_module_instruction`.
const PROPOSER: &str = "Use the information below to learn about a task that we are trying to solve using calls to an LM, then generate a new instruction that will be used to prompt a Language Model to better solve the task.";

const DATASET: &str = "Each input asks for a language pack for one programming language: a small JSON file that tells a mutation-testing engine which syntax nodes of that language are conditions, failure signals, state writes and adjacent checks, and how to mutate them. Two packs for other languages are shown as examples. The output is one JSON pack. It is checked mechanically: shape, size, that every node kind exists in the grammar, and that recorded control cases get their recorded outcome.";

pub struct Settings {
    pub candidates: usize,
    pub trials: usize,
    pub seed: u64,
    pub max_metric_calls: usize,
}

pub struct Outcome {
    /// Candidate zero is the starting guide.
    pub guides: Vec<String>,
    /// Per candidate, the mean score at each position of the scoring set.
    pub subscores: Vec<Vec<f64>>,
    pub aggregate: Vec<f64>,
    /// Every trial in order: candidate and its mean score.
    pub trials: Vec<(usize, f64)>,
    pub metric_calls: usize,
    pub proposals: usize,
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len().max(1) as f64
}

/// The request for one proposed instruction, in the proposer's field order.
fn proposal_request(guide: &str, demo: &str, tip: &str, max_chars: usize) -> String {
    let fenced = format!("```\n{guide}\n```");
    let mut fields = vec![
        (
            "dataset_description",
            "A description of the dataset that we are using.",
            DATASET,
        ),
        ("task_demos", "Example inputs/outputs of our module.", demo),
        ("basic_instruction", "Basic instruction.", fenced.as_str()),
    ];
    if !tip.is_empty() {
        fields.push((
            "tip",
            "A suggestion for how to go about generating the new instruction.",
            tip,
        ));
    }
    let mut text = String::new();
    for (name, description, value) in fields {
        text.push_str(&format!("[[ ## {name} ## ]] ({description})\n{value}\n\n"));
    }
    text.push_str(&format!(
        "Limits the proposed instruction must respect, or it is discarded unread:\n\
         - Keep the first paragraph, which states the purpose, exactly as it is.\n\
         - Keep the sections \"Fields\", \"Operators\", \"Rules\" and \"Output\".\n\
         - It applies to every language: do not name a programming language and do not write syntax node names that belong to one grammar. Describe constructs in words.\n\
         - At most {max_chars} characters.\n\n\
         Provide the proposed instruction within ``` blocks."
    ));
    text
}

/// Step 2 of MIPROv2: propose instructions. A proposal that fails the
/// rules is discarded and recorded; it is not retried.
fn propose(
    adapter: &Authoring,
    baseline: &str,
    demo: &str,
    settings: &Settings,
) -> (Vec<String>, usize) {
    let mut guides = vec![baseline.to_owned()];
    let mut calls = 0;
    for index in 1..settings.candidates {
        // dsrust draws the tip from its generator; here the seed and the
        // candidate's number choose it, which is as arbitrary and replayable.
        let tip = TIPS[(settings.seed as usize + index) % TIPS.len()];
        let request = proposal_request(baseline, demo, tip, adapter.rules.max_chars);
        calls += 1;
        let Ok(reply) = adapter
            .model
            .chat("propose", PROPOSER, &request, adapter.max_tokens.1)
        else {
            break;
        };
        let proposed = extract_new_instruction(&reply.text);
        let reason = match adapter.rules.check(&proposed) {
            Err(reason) => reason,
            Ok(()) if guides.contains(&proposed) => "the same text as an earlier candidate".into(),
            Ok(()) => {
                guides.push(proposed);
                continue;
            }
        };
        let entry =
            json!({"reason": reason, "tip": tip, "guide": guide_id(&proposed), "text": proposed});
        adapter.rejected.lock().unwrap().push(entry);
    }
    (guides, calls)
}

/// Steps 2 and 3. The trial loop is dsrust `search.rs` without minibatches:
/// tell the sampler the starting guide's score, then ask, score on the whole
/// scoring set, tell.
pub fn optimise(adapter: &Authoring, baseline: &str, settings: &Settings) -> Outcome {
    let size = adapter.val.len();
    let first = adapter.run("valset", baseline, &adapter.val);
    let demo = first
        .iter()
        .find(|t| t.verdict_score > 0.0)
        .map(|t| {
            format!(
                "Input: Write the pack for: {}\nOutput:\n{}",
                t.language,
                pack_text(&t.reply)
            )
        })
        .unwrap_or_else(|| {
            "None of the outputs under the basic instruction passed the checks.".into()
        });
    let (guides, proposals) = propose(adapter, baseline, &demo, settings);

    let mut scored: Vec<Vec<Vec<f64>>> = vec![Vec::new(); guides.len()];
    scored[0].push(first.iter().map(|t| t.verdict_score).collect());
    let default_score = mean(&scored[0][0]);
    let mut sampler = tpe::TpeSampler::new(
        settings.seed as u32,
        vec![("0_predictor_instruction".to_owned(), guides.len())],
    );
    sampler.tell(vec![0], default_score);
    let mut trials = vec![(0, default_score)];
    let mut metric_calls = size;
    for _ in 0..settings.trials {
        if guides.len() < 2 || metric_calls + size > settings.max_metric_calls {
            break;
        }
        let params = sampler.ask();
        let run = adapter.run("valset", &guides[params[0]], &adapter.val);
        metric_calls += size;
        let scores: Vec<f64> = run.iter().map(|t| t.verdict_score).collect();
        let score = mean(&scores);
        scored[params[0]].push(scores);
        sampler.tell(params.clone(), score);
        trials.push((params[0], score));
    }
    let subscores: Vec<Vec<f64>> = scored
        .iter()
        .map(|runs| {
            (0..size)
                .map(|at| mean(&runs.iter().map(|run| run[at]).collect::<Vec<_>>()))
                .collect()
        })
        .collect();
    // A candidate the sampler never asked for has no score and cannot be chosen.
    let aggregate = scored
        .iter()
        .zip(&subscores)
        .map(|(runs, means)| if runs.is_empty() { -1.0 } else { mean(means) })
        .collect();
    Outcome {
        guides,
        subscores,
        aggregate,
        trials,
        metric_calls,
        proposals,
    }
}

impl Outcome {
    /// What the summary records about the engine.
    pub fn report(&self) -> Value {
        json!({"engine": "mipro", "metric_calls": self.metric_calls, "candidates": self.guides.len(),
               "proposal_calls": self.proposals, "scores_on_scoring_set": self.aggregate,
               "trials": self.trials.iter().map(|(c, s)| json!({"candidate": c, "score": s})).collect::<Vec<_>>()})
    }
}
