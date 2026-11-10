//! Slice B research tooling: optimise the pack-authoring guide with the GEPA
//! engine (copied from dsrust, see vendor/README.md), judged by the driver's
//! admission. Protocol: docs/research/2026-11-10/16-slice-b-guide-optimisation-preregistration.md.
//!
//! Makes real model calls unless --dry-run is given. Not part of lesscode-core.

mod adapter;
mod model;
mod task;

use adapter::{Authoring, COMPONENT, Rules, Trial, guide_id};
use clap::Parser;
use gepa::progress::{Event, Progress};
use gepa::{Candidate, CandidateSelection, ComponentSelection, GepaEngine};
use lesscode_core::adapters::driver::admission::kind_names;
use lesscode_core::adapters::driver::registry::shipped;
use lesscode_core::regenerator::load_endpoint;
use model::{Live, Metered, Model, Scripted, cost, list_price, recorded_spend};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

#[derive(Parser)]
#[command(about = "Optimise the pack-authoring guide (Slice B research tooling)")]
struct Args {
    /// Where candidates, trials, the ledger and the summary are written.
    #[arg(long)]
    out_dir: PathBuf,
    /// The guide to start from.
    #[arg(
        long,
        default_value = "docs/experiments/annotation-driver/pack-guide-v3.md"
    )]
    guide: PathBuf,
    /// Endpoint file with LESSCODE_MODEL_* keys; default as lesscode-core resolves it.
    #[arg(long)]
    endpoint_env: Option<PathBuf>,
    /// Languages the optimiser may learn from.
    #[arg(long, value_delimiter = ',', default_value = "rust,java")]
    train: Vec<String>,
    /// Languages kept apart for the final judgement.
    #[arg(long, value_delimiter = ',', default_value = "python,typescript")]
    held_out: Vec<String>,
    /// Repetitions of each training language in the pool minibatches are drawn from.
    #[arg(long, default_value_t = 3)]
    train_runs: usize,
    /// Repetitions of each training language in the set every kept candidate is scored on.
    #[arg(long, default_value_t = 2)]
    val_runs: usize,
    /// Authoring calls per reflection step.
    #[arg(long, default_value_t = 3)]
    minibatch: usize,
    /// Authoring calls the optimiser may spend (GEPA's metric calls).
    #[arg(long, default_value_t = 60)]
    max_metric_calls: usize,
    /// Fresh authoring calls per language for the final judgement of the chosen guide.
    #[arg(long, default_value_t = 5)]
    final_runs: usize,
    /// Maximum spend of this run in USD.
    #[arg(long, default_value_t = 6.0)]
    max_usd: f64,
    /// Project-wide stop in USD over every recorded ledger.
    #[arg(long, default_value_t = 20.0)]
    project_cap_usd: f64,
    /// USD per million input and output tokens when the model has no list price here.
    #[arg(long, num_args = 2, value_names = ["IN", "OUT"])]
    price: Option<Vec<f64>>,
    #[arg(long, default_value_t = 0)]
    seed: u64,
    /// Use a scripted stand-in instead of the endpoint; no call is made.
    #[arg(long)]
    dry_run: bool,
    /// Output limit of an authoring call and of a reflection call, in tokens.
    #[arg(long, num_args = 2, value_names = ["AUTHOR", "REFLECT"], default_values_t = [4_000, 8_000])]
    max_tokens: Vec<u64>,
    /// Skip optimisation and judge this guide file with fresh calls.
    #[arg(long)]
    judge_only: Option<PathBuf>,
    /// With --judge-only: the languages to call, one entry per call; default --final-runs of each.
    #[arg(long, value_delimiter = ',')]
    judge_list: Vec<String>,
    /// The repository root.
    #[arg(long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))]
    repo: PathBuf,
}

/// The engine's futures never wait on anything but the calling thread.
fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = std::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
            return value;
        }
        std::thread::yield_now();
    }
}

struct Printer;

impl Progress for Printer {
    fn report(&self, event: Event<'_>) {
        println!("  gepa: {}", event.message());
    }
}

fn repeated(languages: &[String], runs: usize) -> Vec<String> {
    (0..runs).flat_map(|_| languages.iter().cloned()).collect()
}

/// Quality, stability and cost of one guide over the trials recorded for it.
fn profile(trials: &[Trial], guide: &str) -> Value {
    let mut per_language: BTreeMap<&str, Vec<&Trial>> = BTreeMap::new();
    let ran = |t: &&Trial| t.record["stop_reason"] != "not_run";
    let not_run = trials
        .iter()
        .filter(|t| t.guide == guide && !ran(t))
        .count();
    for trial in trials.iter().filter(|t| t.guide == guide).filter(ran) {
        per_language.entry(&trial.language).or_default().push(trial);
    }
    let languages: BTreeMap<&str, Value> = per_language
        .iter()
        .map(|(language, rows)| {
            let count = |key: &str| rows.iter().filter(|t| t.record[key] == true).count();
            let behaviours: BTreeSet<&str> = rows.iter().filter_map(|t| t.record["behaviour"].as_str()).filter(|b| !b.is_empty()).collect();
            let tokens: u64 = rows.iter().map(|t| t.record["output_tokens"].as_u64().unwrap_or(0)).sum();
            let value = json!({
                "trials": rows.len(), "admitted": count("admitted"), "same_as_shipped": count("same"),
                "distinct_behaviours": behaviours.len(), "mean_output_tokens": tokens / rows.len().max(1) as u64,
            });
            (*language, value)
        })
        .collect();
    json!({"guide": guide_id(guide), "characters": guide.chars().count(), "not_run": not_run, "languages": languages})
}

fn run(args: Args) -> Result<(), String> {
    let repo = args.repo.as_path();
    let read = |path: &Path| {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            repo.join(path)
        };
        std::fs::read_to_string(&path)
            .map(|t| t.replace("\r\n", "\n"))
            .map_err(|e| format!("{}: {e}", path.display()))
    };
    let baseline = read(&args.guide)?;
    let languages = Arc::new(task::load_languages(repo)?);
    for language in args.train.iter().chain(&args.held_out) {
        if !languages.contains_key(language) {
            return Err(format!("no fixtures for language {language}"));
        }
    }
    let mut banned: Vec<String> = Vec::new();
    for pack in shipped().map_err(|e| e.to_string())? {
        banned.extend(kind_names(pack).into_iter().map(str::to_owned));
    }
    for language in languages.values() {
        banned.extend(
            language
                .scopes
                .iter()
                .flat_map(|s| s.cases.iter().map(|c| c.label.clone())),
        );
    }
    let rules = Arc::new(Rules::new(&baseline, banned));
    rules
        .check(&baseline)
        .map_err(|e| format!("the starting guide fails its own rules: {e}"))?;

    let model: Box<dyn Model> = if args.dry_run {
        Box::new(Scripted {
            packs: repo.join("native/packs"),
        })
    } else {
        Box::new(Live(load_endpoint(args.endpoint_env.as_deref())?))
    };
    let name = model.name();
    let fallback = (15.0, 75.0);
    let price = match (&args.price, list_price(&name)) {
        (Some(given), _) => (given[0], given[1]),
        (None, Some(listed)) => listed,
        (None, None) => {
            println!(
                "no list price known for {name}; counting 15 and 75 USD per million tokens (set --price to correct)"
            );
            fallback
        }
    };
    std::fs::create_dir_all(args.out_dir.join("candidates")).map_err(|e| e.to_string())?;
    if args.out_dir.join("summary.json").exists() {
        return Err("refusing to overwrite an existing run".into());
    }
    let before = recorded_spend(
        &repo.join("docs/experiments/annotation-driver/results"),
        fallback,
    );
    let metered = Arc::new(Metered::new(
        model,
        price,
        args.max_usd,
        args.project_cap_usd - before,
        args.out_dir.join("ledger.jsonl"),
    ));

    let train = repeated(&args.train, args.train_runs);
    let val = repeated(&args.train, args.val_runs);
    let everything: Vec<String> = args.train.iter().chain(&args.held_out).cloned().collect();
    if let Some(path) = &args.judge_only {
        let guide = read(path)?;
        rules
            .check(&guide)
            .map_err(|e| format!("the guide fails the rules: {e}"))?;
        let list = if args.judge_list.is_empty() {
            repeated(&everything, args.final_runs)
        } else {
            args.judge_list.clone()
        };
        let mut judge = Authoring::new(metered.clone(), languages.clone(), train, val, rules);
        judge.max_tokens = (args.max_tokens[0], args.max_tokens[1]);
        let trials = judge.run("final", &guide, &list);
        let lines: Vec<String> = trials
            .iter()
            .map(|t| {
                let mut record = t.record.clone();
                record["reply"] = json!(t.reply);
                record.to_string()
            })
            .collect();
        std::fs::write(args.out_dir.join("trials.jsonl"), lines.join("\n") + "\n")
            .map_err(|e| e.to_string())?;
        let (spent, calls) = metered.spent();
        let summary = json!({
            "receipt": "slice-b-guide-judgement-v1", "preregistration": "R70", "dry_run": args.dry_run, "model": name,
            "judged": profile(&trials, &guide), "calls": calls, "spent_usd": (spent * 100.0).round() / 100.0,
        });
        let text = serde_json::to_string_pretty(&summary).map_err(|e| e.to_string())?;
        std::fs::write(args.out_dir.join("summary.json"), text + "\n")
            .map_err(|e| e.to_string())?;
        println!("{}", summary["judged"]);
        return Ok(());
    }
    // Measured on the first run (R71): about 4,100 input and 2,300 output
    // tokens per authoring call, 5,500 and 5,300 per reflection.
    let final_calls = everything.len() * args.final_runs;
    let reflections = args.max_metric_calls / (2 * args.minibatch).max(1) + 3;
    let final_reserve = final_calls as f64 * cost(price, 4_100, 2_600);
    let expected = args.max_metric_calls as f64 * cost(price, 4_100, 2_300)
        + final_reserve
        + reflections as f64 * cost(price, 5_500, 5_300);
    // The final judgement must not be starved by the optimisation.
    metered.set_limit((args.max_usd - final_reserve).max(0.0));
    println!(
        "model {name}; at most {} authoring calls for optimisation, {final_calls} for the final judgement, about {reflections} reflections",
        args.max_metric_calls
    );
    println!(
        "expected cost about {expected:.2} USD at {} and {} USD per million tokens; this run stops at {:.2} USD; recorded so far {before:.2} of {:.2} USD",
        price.0, price.1, args.max_usd, args.project_cap_usd
    );

    let mut adapter = Authoring::new(
        metered.clone(),
        languages.clone(),
        train.clone(),
        val.clone(),
        rules.clone(),
    );
    adapter.max_tokens = (args.max_tokens[0], args.max_tokens[1]);
    let (log, rejected) = (adapter.log.clone(), adapter.rejected.clone());
    let engine = GepaEngine {
        adapter,
        trainset_size: train.len(),
        valset_size: val.len(),
        minibatch_size: args.minibatch.min(train.len()),
        max_metric_calls: args.max_metric_calls,
        perfect_score: 1.0,
        skip_perfect_score: true,
        seed: args.seed,
        use_merge: false,
        max_merge_invocations: 0,
        candidate_selection_strategy: CandidateSelection::Pareto,
        component_selector: ComponentSelection::RoundRobin,
        track_best_outputs: false,
        progress: Arc::new(Printer),
    };
    let outcome =
        block_on(engine.optimize(Candidate::from([(COMPONENT.to_owned(), baseline.clone())])));

    // The guide to judge: the highest minimum over the training languages of
    // its mean score on the scoring set, then the higher mean, then the shorter.
    let guides: Vec<&str> = outcome
        .candidates
        .iter()
        .map(|c| c.get(COMPONENT).map(String::as_str).unwrap_or_default())
        .collect();
    let rank = |index: usize| {
        let per_language: Vec<f64> = args
            .train
            .iter()
            .map(|language| {
                let scores: Vec<f64> = val
                    .iter()
                    .zip(&outcome.val_subscores[index])
                    .filter(|(l, _)| *l == language)
                    .map(|(_, s)| *s)
                    .collect();
                scores.iter().sum::<f64>() / scores.len().max(1) as f64
            })
            .collect();
        let worst = per_language.iter().cloned().fold(f64::INFINITY, f64::min);
        (
            worst,
            outcome.val_aggregate_scores[index],
            -(guides[index].chars().count() as f64),
        )
    };
    let chosen = (0..guides.len())
        .max_by(|a, b| {
            rank(*a)
                .partial_cmp(&rank(*b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(0);
    for (index, guide) in guides.iter().enumerate() {
        std::fs::write(
            args.out_dir
                .join(format!("candidates/{index:02}-{}.md", guide_id(guide))),
            guide,
        )
        .map_err(|e| e.to_string())?;
    }

    let improved = guides[chosen] != baseline;
    metered.set_limit(args.max_usd);
    let mut judge = Authoring::new(
        metered.clone(),
        languages.clone(),
        train,
        val.clone(),
        rules,
    );
    judge.max_tokens = (args.max_tokens[0], args.max_tokens[1]);
    let mut final_trials = Vec::new();
    if improved {
        println!(
            "final judgement of candidate {chosen} on {} languages, {} runs each",
            everything.len(),
            args.final_runs
        );
        final_trials = judge.run(
            "final",
            guides[chosen],
            &repeated(&everything, args.final_runs),
        );
    }
    let mut trials = log.lock().unwrap().clone();
    trials.extend(final_trials.iter().cloned());
    let lines: Vec<String> = trials
        .iter()
        .map(|t| {
            let mut record = t.record.clone();
            record["reply"] = json!(t.reply);
            record.to_string()
        })
        .collect();
    std::fs::write(args.out_dir.join("trials.jsonl"), lines.join("\n") + "\n")
        .map_err(|e| e.to_string())?;

    let front: BTreeSet<usize> = outcome
        .per_val_instance_best_candidates
        .iter()
        .flatten()
        .copied()
        .collect();
    let optimisation: Vec<Trial> = trials
        .iter()
        .filter(|t| t.phase != "final")
        .cloned()
        .collect();
    let frontier: Vec<Value> = front
        .iter()
        .map(|index| {
            let mut entry = profile(&optimisation, guides[*index]);
            entry["candidate"] = json!(index);
            entry["mean_score_on_scoring_set"] = json!(outcome.val_aggregate_scores[*index]);
            entry
        })
        .collect();
    let judgement = improved.then(|| profile(&final_trials, guides[chosen]));
    let passes = |entry: &Value, languages: &[String]| {
        languages.iter().all(|l| {
            let row = &entry["languages"][l.as_str()];
            let needed = (row["trials"].as_u64().unwrap_or(0) * 4).div_ceil(5);
            row["admitted"].as_u64().unwrap_or(0) >= needed
                && row["same_as_shipped"].as_u64().unwrap_or(0) >= needed
                && needed > 0
        })
    };
    let complete = judgement.as_ref().is_some_and(|j| j["not_run"] == 0);
    let adopted = judgement
        .as_ref()
        .is_some_and(|j| passes(j, &args.train) && passes(j, &args.held_out));
    let (spent, calls) = metered.spent();
    let summary = json!({
        "receipt": "slice-b-guide-optimisation-v1", "preregistration": "R70", "dry_run": args.dry_run, "model": name,
        "baseline_guide": guide_id(&baseline), "train": args.train, "held_out": args.held_out,
        "parameters": {"train_runs": args.train_runs, "val_runs": args.val_runs, "minibatch": args.minibatch,
                       "max_metric_calls": args.max_metric_calls, "final_runs": args.final_runs, "seed": args.seed,
                       "max_usd": args.max_usd, "usd_per_mtok": [price.0, price.1]},
        "engine": {"iterations": outcome.iterations, "metric_calls": outcome.total_num_evals, "candidates": guides.len(),
                   "scores_on_scoring_set": outcome.val_aggregate_scores, "parents": outcome.parents},
        "discarded_proposals": *rejected.lock().unwrap(),
        "frontier": frontier,
        "chosen_candidate": chosen, "chosen_guide": guide_id(guides[chosen]), "improved_on_baseline": improved,
        "final_judgement": judgement, "final_judgement_complete": complete, "adopted": adopted && complete,
        "calls": calls, "spent_usd": (spent * 100.0).round() / 100.0, "recorded_before_usd": (before * 100.0).round() / 100.0,
    });
    let text = serde_json::to_string_pretty(&summary).map_err(|e| e.to_string())?;
    std::fs::write(args.out_dir.join("summary.json"), text + "\n").map_err(|e| e.to_string())?;
    println!(
        "{} candidates, {} on the frontier; chosen {chosen} ({}); final judgement {}; {calls} calls, {spent:.2} USD",
        guides.len(),
        front.len(),
        if improved {
            "differs from the baseline"
        } else {
            "the baseline: no improvement found"
        },
        if !improved {
            "not run"
        } else if !complete {
            "incomplete: the budget stopped it"
        } else if adopted {
            "passed"
        } else {
            "not passed"
        }
    );
    Ok(())
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("guide-optimizer: {error}");
            ExitCode::from(3)
        }
    }
}
