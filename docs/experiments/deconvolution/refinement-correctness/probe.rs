//! R39 adversarial probes. Assertions express the required behavior; red is expected at a755712.
use lesscode_core::{receipt::{self, J}, regenerator::{self, AttemptParams, RefineBudget, RuntimeOracle, TurnRequest}, surface};
use serde_json::json;
use std::{fs, path::{Path, PathBuf}};

fn root(name: &str) -> PathBuf {
    let base = PathBuf::from(std::env::var("R39_WORK").expect("R39_WORK"));
    let dir = base.join(format!("{name}-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}
fn oracle_case(name: &str, script: &str, timeout_s: u64) -> (i32, Option<String>, J) {
    let dir = root(name);
    fs::write(dir.join("oracle.py"), script).unwrap();
    RuntimeOracle { python: PathBuf::from(std::env::var("R39_PYTHON").unwrap()), script_rel: "oracle.py".into(), timeout_s }.evaluate(&dir)
}
const OK: &str = "import json\njson.dump({'ok': True, 'passed': 1, 'total': 1, 'failures': []}, open('oracle-verdict.json', 'w'))\n";

#[test]
fn positive_completed_oracle_passes() {
    let (code, divergence, results) = oracle_case("positive", OK, 10);
    assert_eq!(regenerator::evaluate_tier2_runtime(code, divergence.as_deref(), &results).score, 1.0);
}
#[test]
fn failed_process_cannot_pass_from_success_file() {
    let (code, divergence, results) = oracle_case("exit2", &format!("{OK}raise SystemExit(2)\n"), 10);
    let reward = regenerator::evaluate_tier2_runtime(code, divergence.as_deref(), &results);
    assert_eq!(reward.failure_kind.as_deref(), Some("runtime_infrastructure"), "adapter_code={code}; reward={reward:?}");
}
#[test]
fn timeout_cannot_pass_from_success_file() {
    let (code, divergence, results) = oracle_case("timeout", &format!("{OK}import time\ntime.sleep(5)\n"), 1);
    let reward = regenerator::evaluate_tier2_runtime(code, divergence.as_deref(), &results);
    assert_eq!(reward.failure_kind.as_deref(), Some("runtime_infrastructure"), "adapter_code={code}; reward={reward:?}");
}
#[test]
fn contradictory_counts_cannot_pass() {
    let (code, divergence, results) = oracle_case("counts", "import json\njson.dump({'ok': True, 'passed': 0, 'total': 1, 'failures': [{'case': 1, 'error': 'mismatch'}]}, open('oracle-verdict.json','w'))\n", 10);
    let reward = regenerator::evaluate_tier2_runtime(code, divergence.as_deref(), &results);
    assert!(reward.score < 1.0, "adapter_code={code}; reward={reward:?}");
}

fn run_loop(name: &str, sources: &[&str], budget: RefineBudget, runtime: bool) -> (regenerator::RefineOutcome, Vec<String>, String) {
    let work = root(name);
    let frozen = work.join("frozen");
    fs::create_dir_all(frozen.join("src")).unwrap();
    fs::write(frozen.join("src/mod.py"), "def f():\n    return 1\n").unwrap();
    fs::write(frozen.join("src/user.py"), "from mod import f\n").unwrap();
    let report = surface::derive_surface(&frozen, &["src/mod.py".into()], &[], &[]).unwrap();
    fs::write(frozen.join("public-surface.json"), receipt::dumps(&report)).unwrap();
    let req = TurnRequest {
        scope_root: frozen.to_string_lossy().into_owned(), scope_files: vec!["src/mod.py".into()],
        surface_path: "public-surface.json".into(), definitions: "d".into(), targets: vec![],
        contract: "c".into(), acceptance_command: "frozen fixture oracle".into(), feedback: None, language: "python".into(),
    };
    let mut prompts = Vec::new();
    let mut calls = 0;
    let mut model = |_system: &str, prompt: &str, _: &AttemptParams| {
        prompts.push(prompt.to_owned());
        let source = sources[calls]; calls += 1;
        Ok((json!({"files": [{"path": "src/mod.py", "content": source}], "structure": {"pure": [], "shell": []}, "change_log": "probe"}).to_string(), J::obj(vec![("stop_reason", J::str("end_turn"))])))
    };
    let mut evaluator = |cand: &Path| {
        let source = fs::read_to_string(cand.join("src/mod.py")).unwrap();
        if source.contains("return 1") { (0, None, J::obj(vec![])) }
        else { (1, Some("R39_known_counterexample".into()), J::obj(vec![])) }
    };
    let cand = work.join("cand");
    let outcome = regenerator::run_refine_loop(&req, &cand, &frozen, &work.join("receipt.json"), name, &budget,
        if runtime { Some(&mut evaluator) } else { None }, &mut model, &[], None).unwrap();
    (outcome, prompts, fs::read_to_string(cand.join("src/mod.py")).unwrap())
}
#[test]
fn next_turn_receives_runtime_counterexample() {
    let (out, prompts, _) = run_loop("feedback", &["def f():\n    return 2\n", "def f():\n    return 1\n"], RefineBudget { max_turns: 2, max_static_retries: 1, ..Default::default() }, true);
    assert!(prompts[1].contains("R39_known_counterexample"), "status={}; next prompt lacks runtime feedback: {}", out.status, prompts[1]);
}
#[test]
fn accepted_best_matches_delivered_candidate() {
    let (out, _, source) = run_loop("best", &["def f():\n    return 1\n", "def f():\n    return 2\n"], RefineBudget { max_turns: 1, max_static_retries: 1, n_rollouts_per_turn: 2, ..Default::default() }, true);
    assert_eq!(out.status, "accepted");
    assert_eq!(source, "def f():\n    return 1\n", "reported best={} score={} validity={} but last rollout remains", out.best.label, out.best.score, out.validity);
}
#[test]
fn low_search_threshold_cannot_establish_validity() {
    let (out, _, _) = run_loop("threshold", &["def f():\n    return 2\n"], RefineBudget { max_turns: 1, max_static_retries: 1, threshold: 0.5, ..Default::default() }, false);
    assert!(!out.validity, "no runtime evaluator: status={} tier={} score={} validity={}", out.status, out.best.tier, out.best.score, out.validity);
}
#[test]
fn missing_measurement_cannot_establish_compression() {
    let (out, _, _) = run_loop("compression", &["def f():\n    return 1\n"], RefineBudget { max_turns: 1, max_static_retries: 1, ..Default::default() }, true);
    assert!(!out.compression_met, "targets and measurements absent, compression_met={}", out.compression_met);
}

#[test]
fn still_failing_counterexample_remains_unresolved() {
    let (out, _, _) = run_loop("obligation", &["def f():\n    return 2\n"], RefineBudget { max_turns: 1, max_static_retries: 1, ..Default::default() }, true);
    assert!(out.unresolved_obligations.contains(&"R39_known_counterexample".to_string()), "failed witness was marked discharged: {:?}", out.unresolved_obligations);
}
