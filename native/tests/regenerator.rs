//! ProbabilisticRegenerator Rust-substrate suite (task 8.5): ports the
//! offline contract cases of `tests/test_regenerator.py` — turn contracts
//! and endpoints, receipt/bindings provenance, the Tier-1 ladder and
//! adaptive schedule, the Tier-2 pinned formula and feedback boundary, the
//! Refine/BestOfN controller with its stops and outcomes, the six adverse
//! controls (zero runtime calls), the openai-compatible adapter through an
//! injected transport — plus the env-gated E4 three-turn replay and the
//! admission verification of emitted receipts (the task 8.5 done-when).
//!
//! Port deltas: the anthropic streaming arm, its token-count guard (the
//! only exit-5 trigger) and `HELD_OUT_MARKERS` (unused in the Python
//! module) do not port; evaluators take no `results` argument (always
//! `{}` in Python).

use lesscode_core::receipt::{self, J};
use lesscode_core::regenerator::{
    self, Adaptation, AttemptParams, Endpoint, RefineBudget, RewardEvaluation, RuntimeOracle,
    TurnOutcome, TurnRequest,
};
use lesscode_core::surface;
use serde_json::json;
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

const GOOD: &str = "def f():\n    return 1\n";
const SYNTAX: &str = "def f(:\n";

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn scratch(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("lc-regen-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

fn obj(pairs: &[(&str, J)]) -> J {
    J::obj(pairs.to_vec())
}

// --- fixtures -----------------------------------------------------------------

/// Frozen tree with `src/mod.py` and an empty surface report.
fn frozen_tree(work: &Path) -> PathBuf {
    let frozen = work.join("frozen");
    fs::create_dir_all(frozen.join("src")).unwrap();
    fs::write(frozen.join("src/mod.py"), GOOD).unwrap();
    fs::write(frozen.join("public-surface.json"), r#"{"modules": {}}"#).unwrap();
    frozen
}

fn request_path(frozen: &Path) -> PathBuf {
    let req = json!({
        "scope_root": frozen.to_string_lossy(),
        "scope_files": ["src/mod.py"],
        "surface_path": frozen.join("public-surface.json").to_string_lossy(),
        "definitions": "module definitions prose",
        "targets": {"written_stmts_max": 50},
        "contract": "all functions declared",
        "acceptance_command": "python -m pytest tests -q",
        "language": "python",
    });
    let path = frozen.join("request.json");
    fs::write(&path, req.to_string()).unwrap();
    path
}

/// The g1r mini fixture: derived Contract-B surface written to the tree.
fn mini_request(work: &Path) -> (TurnRequest, PathBuf) {
    let frozen = work.join("frozen");
    fs::create_dir_all(frozen.join("src")).unwrap();
    fs::write(frozen.join("src/mod.py"), GOOD).unwrap();
    fs::write(frozen.join("src/user.py"), "from mod import f\n").unwrap();
    let report = surface::derive_surface(&frozen, &["src/mod.py".into()], &[], &[]).unwrap();
    fs::write(frozen.join("public-surface.json"), receipt::dumps(&report)).unwrap();
    let req = TurnRequest {
        scope_root: frozen.to_string_lossy().into_owned(),
        scope_files: vec!["src/mod.py".into()],
        surface_path: frozen
            .join("public-surface.json")
            .to_string_lossy()
            .into_owned(),
        definitions: "d".into(),
        targets: vec![],
        contract: "c".into(),
        acceptance_command: "pytest -q".into(),
        feedback: None,
        language: "python".into(),
    };
    (req, frozen)
}

fn response_text(files: &[(&str, &str)], pure: &[&str], shell: &[&str]) -> String {
    json!({
        "files": files.iter().map(|(p, c)| json!({"path": p, "content": c})).collect::<Vec<_>>(),
        "structure": {"pure": pure, "shell": shell},
        "change_log": "test",
    })
    .to_string()
}

fn end_turn() -> J {
    obj(&[("stop_reason", J::str("end_turn"))])
}

/// Scripted `call_fn`: replays (meta, response) steps, records prompts.
struct Scripted {
    steps: VecDeque<Result<(String, J), String>>,
    seen: Vec<(String, AttemptParams)>,
}

impl Scripted {
    fn new(steps: Vec<Result<(String, J), String>>) -> Self {
        Scripted {
            steps: steps.into(),
            seen: vec![],
        }
    }

    fn call(
        &mut self,
        _system: &str,
        user: &str,
        params: &AttemptParams,
    ) -> Result<(String, J), String> {
        self.seen.push((user.to_string(), params.clone()));
        self.steps.pop_front().expect("script exhausted")
    }
}

fn outcome_failed(kind: &str, detail: &str) -> TurnOutcome {
    TurnOutcome {
        ok: false,
        kind: kind.into(),
        detail: Some(J::str(detail)),
        result: None,
        written: vec![],
    }
}

fn outcome_ok(result: Option<lesscode_core::regenerator::TurnResult>) -> TurnOutcome {
    TurnOutcome {
        ok: true,
        kind: String::new(),
        detail: None,
        result,
        written: vec![],
    }
}

fn surface_report(frozen: &Path) -> J {
    // The Python fixture writes the sibling user module before deriving.
    let user = frozen.join("src/user.py");
    if !user.exists() {
        fs::write(&user, "from mod import f\n").unwrap();
    }
    surface::derive_surface(frozen, &["src/mod.py".into()], &[], &[]).unwrap()
}

fn ladder(
    frozen: &Path,
    outcome: TurnOutcome,
    cand: Option<&Path>,
    targets: &[(String, i64)],
) -> RewardEvaluation {
    let req = TurnRequest::load(&request_path(frozen)).unwrap();
    regenerator::evaluate_tier1_static(
        &req,
        &outcome,
        cand.unwrap_or(frozen),
        &surface_report(frozen),
        targets,
    )
    .unwrap()
}

fn eval_verdicts(
    verdicts: Vec<(i32, Option<String>, J)>,
) -> impl FnMut(&Path) -> (i32, Option<String>, J) {
    let mut queue: VecDeque<(i32, Option<String>, J)> = verdicts.into();
    move |_root| queue.pop_front().expect("evaluator script exhausted")
}

fn run_loop(
    req: &TurnRequest,
    frozen: &Path,
    work: &Path,
    label: &str,
    budget: RefineBudget,
    evaluator: Option<lesscode_core::regenerator::RuntimeEvaluator>,
    scripted: &mut Scripted,
) -> Result<lesscode_core::regenerator::RefineOutcome, lesscode_core::regenerator::RegenError> {
    regenerator::run_refine_loop(
        req,
        &work.join("cand"),
        frozen,
        &work.join("receipt.json"),
        label,
        &budget,
        evaluator,
        &mut |s, u, p| scripted.call(s, u, p),
        &[],
        None,
    )
}

// --- TurnRequest and prompt ------------------------------------------------------

#[test]
fn turn_request_language_defaults_and_roundtrips() {
    let work = scratch("lang");
    let frozen = frozen_tree(&work);
    let path = request_path(&frozen);
    assert_eq!(TurnRequest::load(&path).unwrap().language, "python");
    let mut req = TurnRequest::load(&path).unwrap();
    req.language = "rust".into();
    let (_, user) = regenerator::build_prompt(&req).unwrap();
    assert!(user.contains("```rust\n"), "{user}");
    assert!(!user.contains("```python\n"));
}

#[test]
fn build_prompt_fences_labels_and_feedback() {
    let work = scratch("prompt");
    let frozen = frozen_tree(&work);
    let mut req = TurnRequest::load(&request_path(&frozen)).unwrap();
    let (system, user) = regenerator::build_prompt(&req).unwrap();
    assert!(user.contains("```python\n"));
    assert!(user.contains("```json\n"));
    assert!(system.to_lowercase().contains("python"));
    assert!(system.contains("You are the regenerator"));
    assert!(system.contains("Answer with exactly one JSON object"));

    req.feedback = Some(obj(&[
        ("kind", J::str("failed")),
        ("first_divergence", J::str("static:missing_surface")),
        ("measure_gaps", J::obj(vec![])),
    ]));
    let (_, user) = regenerator::build_prompt(&req).unwrap();
    assert!(user.contains("## Feedback from the previous round"));
    assert!(user.contains("Previous round outcome: failed."));

    req.feedback = Some(obj(&[("kind", J::str("bogus"))]));
    let err = regenerator::build_prompt(&req).unwrap_err();
    assert!(err.message().contains("feedback"), "{err:?}");
}

// --- Endpoint ---------------------------------------------------------------------

#[test]
fn endpoint_redacts_key_everywhere() {
    let ep = Endpoint::new(
        "anthropic",
        "https://api.example.com/v1",
        "model-x",
        "sk-secret",
    );
    let display = format!("{ep}");
    assert!(!display.contains("sk-secret"));
    let public = receipt::dumps(&ep.public());
    assert!(!public.contains("sk-secret"));
    assert_eq!(
        ep.public(),
        obj(&[
            ("style", J::str("anthropic")),
            ("host", J::str("api.example.com")),
            ("model", J::str("model-x")),
            ("model_sha256", J::str(regenerator::sha256_text("model-x"))),
        ])
    );
}

#[test]
fn load_endpoint_parses_env_file_with_comments() {
    let work = scratch("env");
    let env = work.join("model.env");
    fs::write(
        &env,
        "LESSCODE_MODEL_API_STYLE=anthropic\n\
         LESSCODE_MODEL_BASE_URL=https://api.example.com/v1\n\
         LESSCODE_MODEL_NAME=model-x\n\
         LESSCODE_MODEL_API_KEY=sk-secret # trailing comment\n",
    )
    .unwrap();
    let ep = regenerator::load_endpoint(Some(&env)).unwrap();
    assert_eq!(ep.host, "api.example.com");
    assert_eq!(ep.api_key(), "sk-secret");
}

#[test]
fn load_endpoint_missing_keys_fail() {
    let work = scratch("env-missing");
    let env = work.join("model.env");
    fs::write(&env, "LESSCODE_MODEL_API_STYLE=anthropic\n").unwrap();
    let err = match regenerator::load_endpoint(Some(&env)) {
        Err(e) => e,
        Ok(_) => panic!("missing keys must fail"),
    };
    assert!(err.contains("missing"), "{err}");
}

// --- Candidate preparation and writes ----------------------------------------------

#[test]
fn prepare_candidate_copies_and_checks_hashes() {
    let work = scratch("prepare");
    let frozen = frozen_tree(&work);
    let cand = work.join("cand");
    let hashes = regenerator::prepare_candidate(&frozen, &cand, &["src/mod.py".into()]).unwrap();
    assert_eq!(fs::read_to_string(cand.join("src/mod.py")).unwrap(), GOOD);
    match hashes {
        J::Obj(pairs) => assert_eq!(pairs.len(), 1),
        other => panic!("{other:?}"),
    }
    fs::write(cand.join("src/mod.py"), "tampered\n").unwrap();
    let err = regenerator::prepare_candidate(&frozen, &cand, &["src/mod.py".into()]).unwrap_err();
    assert!(matches!(
        err,
        lesscode_core::regenerator::RegenError::CopyMismatch(_)
    ));
}

fn turn_result(
    files: &[(&str, &str)],
    pure: &[&str],
    shell: &[&str],
) -> lesscode_core::regenerator::TurnResult {
    regenerator::parse_result(&response_text(files, pure, shell)).unwrap()
}

#[test]
fn write_candidate_refuses_out_of_scope_before_writing() {
    let work = scratch("scope");
    let frozen = frozen_tree(&work);
    let cand = work.join("cand");
    regenerator::prepare_candidate(&frozen, &cand, &["src/mod.py".into()]).unwrap();
    let result = turn_result(
        &[("src/other.py", "x = 1\n"), ("src/mod.py", "y = 2\n")],
        &[],
        &[],
    );
    let err = regenerator::write_candidate(&result, &cand, &["src/mod.py".into()]).unwrap_err();
    assert!(err.message().contains("src/other.py"), "{err:?}");
    assert!(!cand.join("src/other.py").exists());
    assert_eq!(fs::read_to_string(cand.join("src/mod.py")).unwrap(), GOOD);
}

#[test]
fn write_candidate_records_hashes_and_crlf_convention() {
    let work = scratch("crlf");
    let frozen = frozen_tree(&work);
    let cand = work.join("cand");
    regenerator::prepare_candidate(&frozen, &cand, &["src/mod.py".into()]).unwrap();
    fs::write(cand.join("src/mod.py"), b"def f():\r\n    return 1\r\n").unwrap();
    let result = turn_result(&[("src/mod.py", "def g():\r\n    return 2\r\n")], &[], &[]);
    let written = regenerator::write_candidate(&result, &cand, &["src/mod.py".into()]).unwrap();
    match &written[0] {
        J::Obj(pairs) => {
            let crlf = pairs.iter().find(|(k, _)| k == "crlf").unwrap();
            assert_eq!(crlf.1, J::Bool(true));
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(
        fs::read(cand.join("src/mod.py")).unwrap(),
        b"def g():\r\n    return 2\r\n"
    );
}

// --- Receipt writer -----------------------------------------------------------------

fn note_keys(receipt: &Path) -> Vec<String> {
    let data: serde_json::Value = serde_json::from_slice(&fs::read(receipt).unwrap()).unwrap();
    data["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["key"].as_str().unwrap_or_default().to_string())
        .collect()
}

#[test]
fn receipt_fact_and_note_structure() {
    let work = scratch("receipt");
    let receipt = work.join("receipt.json");
    let mut writer = regenerator::ReceiptWriter::new(&receipt).unwrap();
    writer
        .add_fact(obj(&[("key", J::str("inputs")), ("value", J::int(1))]), &[])
        .unwrap();
    writer
        .add_note("turn-1", obj(&[("score", J::Float(0.5))]), &[])
        .unwrap();
    let data: serde_json::Value = serde_json::from_slice(&fs::read(&receipt).unwrap()).unwrap();
    assert_eq!(data["facts"][0]["key"], "inputs");
    assert_eq!(data["notes"][0]["key"], "turn-1");
    assert!(data["notes"][0]["time_utc"]
        .as_str()
        .is_some_and(|t| !t.is_empty()));
}

#[test]
fn receipt_labels_case_insensitively_unique() {
    let work = scratch("labels");
    let mut writer = regenerator::ReceiptWriter::new(&work.join("receipt.json")).unwrap();
    writer
        .add_note("Turn-1", obj(&[("a", J::int(1))]), &[])
        .unwrap();
    let err = writer
        .add_note("tURN-1", obj(&[("b", J::int(2))]), &[])
        .unwrap_err();
    assert!(matches!(
        err,
        lesscode_core::regenerator::RegenError::Usage(_)
    ));
}

#[test]
fn receipt_sidecars_and_bindings_for_admission() {
    let work = scratch("sidecars");
    let receipt = work.join("receipt.json");
    let mut writer = regenerator::ReceiptWriter::new(&receipt).unwrap();
    writer
        .add_note("turn-1", obj(&[("score", J::Float(1.0))]), &[])
        .unwrap();
    let stored = fs::read_to_string(&receipt).unwrap();
    // The native writer ends receipt.json with '\n' where Python did not;
    // the sidecar digests the bytes as stored either way.
    let sidecar = fs::read_to_string(work.join("receipt.json.sha256")).unwrap();
    assert_eq!(
        sidecar,
        format!("{}  receipt.json\n", regenerator::sha256_text(&stored))
    );
    let bindings: serde_json::Value =
        serde_json::from_slice(&fs::read(work.join("bindings.json")).unwrap()).unwrap();
    assert_eq!(bindings["bindings"][0]["artifact"], "receipt.json");
    assert_eq!(
        bindings["bindings"][0]["sidecars"][0],
        "receipt.json.sha256"
    );
}

#[test]
fn save_disk_bytes_equal_hashed_text() {
    let work = scratch("save");
    let digest = regenerator::save(&work, "note.txt", "line\nother\n").unwrap();
    assert_eq!(fs::read(work.join("note.txt")).unwrap(), b"line\nother\n");
    assert_eq!(digest, regenerator::sha256_text("line\nother\n"));
    assert_eq!(
        fs::read_to_string(work.join("note.txt.sha256")).unwrap(),
        format!("{digest}  note.txt\n")
    );
}

#[test]
fn bound_artifacts_cover_saved_turn_files() {
    let work = scratch("bound");
    let receipt = work.join("receipt.json");
    let writer = regenerator::ReceiptWriter::new(&receipt).unwrap();
    let turn_dir = work.join("turns").join("L1");
    regenerator::save(&turn_dir, "system.txt", "sys").unwrap();
    regenerator::save(&turn_dir, "user.txt", "usr").unwrap();
    writer
        .add_bound_artifact(&turn_dir.join("system.txt"))
        .unwrap();
    writer
        .add_bound_artifact(&turn_dir.join("user.txt"))
        .unwrap();
    let bindings: serde_json::Value =
        serde_json::from_slice(&fs::read(work.join("bindings.json")).unwrap()).unwrap();
    let artifacts: Vec<&str> = bindings["bindings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["artifact"].as_str().unwrap())
        .collect();
    assert!(artifacts.contains(&"turns/L1/system.txt"));
    assert!(artifacts.contains(&"turns/L1/user.txt"));
}

// --- Tier-1 ladder --------------------------------------------------------------------

#[test]
fn tier1_failure_rungs_score_by_kind() {
    let work = scratch("t1fail");
    let frozen = frozen_tree(&work);
    for (kind, detail, score) in [
        ("transport", "boom", 0.00),
        ("stop_reason", "max_tokens", 0.00),
        ("invalid_json", "{", 0.10),
        ("out_of_scope", "src/other.py", 0.20),
    ] {
        let reward = ladder(&frozen, outcome_failed(kind, detail), None, &[]);
        assert_eq!(
            (reward.score, reward.failure_kind.as_deref()),
            (score, Some(kind))
        );
    }
}

#[test]
fn tier1_syntax_rung_reports_line_anchored_errors() {
    let work = scratch("t1syntax");
    let frozen = frozen_tree(&work);
    let cand = work.join("cand");
    regenerator::prepare_candidate(&frozen, &cand, &["src/mod.py".into()]).unwrap();
    fs::write(cand.join("src/mod.py"), SYNTAX).unwrap();
    let reward = ladder(&frozen, outcome_ok(None), Some(&cand), &[]);
    assert_eq!(
        (reward.score, reward.failure_kind.as_deref()),
        (0.30, Some("syntax_error"))
    );
    match &reward.diagnostics {
        J::Obj(pairs) => {
            let errors = pairs.iter().find(|(k, _)| k == "errors").unwrap();
            assert!(
                receipt::dumps(&errors.1).contains("src/mod.py:1:"),
                "{errors:?}"
            );
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn tier1_surface_rung_reports_missing_symbol() {
    let work = scratch("t1surface");
    let frozen = frozen_tree(&work);
    let cand = work.join("cand");
    regenerator::prepare_candidate(&frozen, &cand, &["src/mod.py".into()]).unwrap();
    fs::write(cand.join("src/mod.py"), "def g():\n    return 2\n").unwrap();
    let reward = ladder(&frozen, outcome_ok(None), Some(&cand), &[]);
    assert_eq!(
        (reward.score, reward.failure_kind.as_deref()),
        (0.40, Some("surface"))
    );
    match &reward.diagnostics {
        J::Obj(pairs) => {
            let missing = pairs.iter().find(|(k, _)| k == "missing_symbols").unwrap();
            // pairs of (module, name); f must be reported missing from mod
            let text = receipt::dumps(&missing.1);
            assert!(text.contains("\"f\""), "{text}");
            assert!(text.contains("\"mod\""), "{text}");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn tier1_structure_rung_reports_violation() {
    let work = scratch("t1struct");
    let frozen = frozen_tree(&work);
    let cand = work.join("cand");
    regenerator::prepare_candidate(&frozen, &cand, &["src/mod.py".into()]).unwrap();
    fs::write(cand.join("src/mod.py"), "def f(session):\n    return 1\n").unwrap();
    let result = turn_result(&[], &["mod.py::f"], &[]);
    let reward = ladder(&frozen, outcome_ok(Some(result)), Some(&cand), &[]);
    assert_eq!(
        (reward.score, reward.failure_kind.as_deref()),
        (0.45, Some("structure"))
    );
    let why = receipt::dumps(match &reward.diagnostics {
        J::Obj(pairs) => &pairs.iter().find(|(k, _)| k == "why").unwrap().1,
        other => panic!("{other:?}"),
    });
    assert!(why.contains("param session"), "{why}");
}

#[test]
fn tier1_static_targets_rung_and_pass_rung() {
    let work = scratch("t1targets");
    let frozen = frozen_tree(&work);
    let cand = work.join("cand");
    regenerator::prepare_candidate(&frozen, &cand, &["src/mod.py".into()]).unwrap();
    fs::write(
        cand.join("src/mod.py"),
        "def f():\n    return 1\n\n\ndef h():\n    return 2\n",
    )
    .unwrap();
    let gaps = ladder(
        &frozen,
        outcome_ok(None),
        Some(&cand),
        &[("funcs_max".to_string(), 1)],
    );
    assert_eq!(
        (gaps.score, gaps.failure_kind.as_deref()),
        (0.49, Some("static_targets"))
    );
    let text = receipt::dumps(&gaps.diagnostics);
    assert!(text.contains("\"excess\": 1"), "{text}");
    assert!(text.contains("funcs_max"), "{text}");

    let result = turn_result(&[], &["mod.py::f"], &[]);
    let pass = ladder(
        &frozen,
        outcome_ok(Some(result)),
        Some(&cand),
        &[("funcs_max".to_string(), 5)],
    );
    assert_eq!(pass.score, 0.50);
    assert!(pass.passed_tier1);
    assert_eq!(pass.failure_kind, None);
}

#[test]
fn tier1_ladder_is_fast_and_offline() {
    let work = scratch("t1fast");
    let frozen = frozen_tree(&work);
    let cand = work.join("cand");
    regenerator::prepare_candidate(&frozen, &cand, &["src/mod.py".into()]).unwrap();
    let t0 = Instant::now();
    let _ = ladder(&frozen, outcome_ok(None), Some(&cand), &[]);
    assert!(t0.elapsed().as_secs_f64() < 2.0);
}

// --- Adaptive schedule -----------------------------------------------------------------

fn reward(score: f64, kind: Option<&str>, missing: Option<J>) -> RewardEvaluation {
    let mut diagnostics = obj(&[]);
    if let Some(m) = missing {
        diagnostics = obj(&[("missing_symbols", m)]);
    }
    RewardEvaluation {
        score,
        tier: 1,
        passed_tier1: false,
        failure_kind: kind.map(str::to_string),
        diagnostics,
    }
}

#[test]
fn schedule_cheaper_effort_and_concise_directive_on_zero() {
    let prev = reward(0.00, Some("stop_reason"), None);
    let params = regenerator::next_rollout_params(&prev, 0);
    assert_eq!(params.effort.as_deref(), Some("medium"));
    assert!(params
        .directive
        .as_deref()
        .is_some_and(|d| d.to_lowercase().contains("concise")));
}

#[test]
fn schedule_temperature_steps_bounded() {
    let prev = reward(0.30, Some("syntax_error"), None);
    let temps: Vec<Option<f64>> = (0..4)
        .map(|a| regenerator::next_rollout_params(&prev, a).temperature)
        .collect();
    assert_eq!(temps, vec![Some(0.0), Some(0.2), Some(0.4), Some(0.4)]);
}

#[test]
fn schedule_injects_missing_symbols_on_surface_failure() {
    let prev = reward(
        0.40,
        Some("surface"),
        Some(J::arr(vec![J::arr(vec![J::str("mod"), J::str("f")])])),
    );
    let params = regenerator::next_rollout_params(&prev, 1);
    assert_eq!(params.inject_missing_symbols, Some(vec!["f".to_string()]));
}

#[test]
fn schedule_pass_needs_no_adaptation() {
    let prev = RewardEvaluation {
        score: 0.50,
        tier: 1,
        passed_tier1: true,
        failure_kind: None,
        diagnostics: obj(&[]),
    };
    let Adaptation {
        effort,
        directive,
        temperature,
        inject_missing_symbols,
    } = regenerator::next_rollout_params(&prev, 0);
    assert!(
        effort.is_none()
            && directive.is_none()
            && temperature.is_none()
            && inject_missing_symbols.is_none()
    );
}

// --- Tier-2 pinned formula ------------------------------------------------------------

#[test]
fn tier2_infrastructure_never_a_kill() {
    for code in [124, 2, 7] {
        let r = regenerator::evaluate_tier2_runtime(code, None, &obj(&[]));
        assert_eq!(
            (r.score, r.failure_kind.as_deref()),
            (0.50, Some("runtime_infrastructure"))
        );
    }
}

#[test]
fn tier2_formula_on_synthetic_fractions() {
    let cases = [
        (1.0, 0.0, 0.73), // 0.60 + 0.25*(0.5)
        (0.8, 0.6, 0.78), // 0.60 + 0.25*(0.7)
        (1.0, 1.0, 0.85),
        (0.0, 0.0, 0.60),
        (0.5, 0.5, 0.73), // 0.725 half-up
        (0.5, 0.5, 0.73),
    ];
    for (g, s, expected) in cases {
        let r = regenerator::evaluate_tier2_runtime(
            1,
            Some("seq::first"),
            &obj(&[
                ("seed_group_pass_fraction", J::Float(g)),
                ("sequence_pass_fraction", J::Float(s)),
            ]),
        );
        assert_eq!(r.score, expected, "g={g} s={s}");
        assert_eq!(r.tier, 2);
        assert!(receipt::dumps(&r.diagnostics).contains("seq::first"));
    }
    // exact decimal half-up where f64 would misround 0.675 to 0.67
    let r = regenerator::evaluate_tier2_runtime(
        1,
        Some("x"),
        &obj(&[
            ("seed_group_pass_fraction", J::Float(0.4)),
            ("sequence_pass_fraction", J::Float(0.2)),
        ]),
    );
    assert_eq!(r.score, 0.68);
}

#[test]
fn tier2_missing_fraction_scores_zero_fail_closed() {
    let r = regenerator::evaluate_tier2_runtime(
        1,
        None,
        &obj(&[("sequence_pass_fraction", J::Float(1.0))]),
    );
    assert_eq!(r.score, 0.73); // 0.60 + 0.25*(0.5*0 + 0.5*1.0) = 0.725
}

#[test]
fn tier2_accepted_with_gaps_faults_and_full_acceptance() {
    let gaps = regenerator::evaluate_tier2_runtime(
        0,
        None,
        &obj(&[("measure_gaps", obj(&[("stmts_max", obj(&[]))]))]),
    );
    assert_eq!(
        (gaps.score, gaps.failure_kind.as_deref()),
        (0.90, Some("gaps"))
    );
    let faults = regenerator::evaluate_tier2_runtime(
        0,
        None,
        &obj(&[(
            "surviving_faults",
            J::arr(vec![obj(&[("line", J::int(3))])]),
        )]),
    );
    assert_eq!(
        (faults.score, faults.failure_kind.as_deref()),
        (0.95, Some("scored_faults"))
    );
    let full = regenerator::evaluate_tier2_runtime(0, None, &obj(&[]));
    assert_eq!(full.score, 1.00);
    assert_eq!(full.failure_kind, None);
}

// --- Feedback boundary ------------------------------------------------------------------

#[test]
fn feedback_boundary_rejects_held_out_names() {
    let payload = obj(&[
        ("kind", J::str("failed")),
        ("first_divergence", J::str("diff in seq HOLDOUT-7::op")),
        ("measure_gaps", obj(&[])),
    ]);
    let err = regenerator::enforce_feedback_boundary(
        &payload,
        &["HOLDOUT-7".into(), "HOLDOUT-12".into()],
    )
    .unwrap_err();
    assert!(err.message().contains("held"), "{err:?}");

    regenerator::enforce_feedback_boundary(
        &obj(&[
            ("kind", J::str("failed")),
            ("first_divergence", J::str("static:missing_surface:mod::f")),
            ("measure_gaps", obj(&[])),
        ]),
        &["HOLDOUT-7".into()],
    )
    .unwrap();

    let scored = regenerator::enforce_feedback_boundary(
        &obj(&[
            ("kind", J::str("accepted_scored")),
            ("first_divergence", J::Null),
            ("measure_gaps", obj(&[])),
            ("surviving_faults", J::arr(vec![])),
            (
                "counterexamples",
                J::arr(vec![J::str("replay of HOLDOUT-12 differs")]),
            ),
        ]),
        &["HOLDOUT-7".into(), "HOLDOUT-12".into()],
    )
    .unwrap_err();
    assert!(matches!(
        scored,
        lesscode_core::regenerator::RegenError::FeedbackBoundary(_)
    ));
}

#[test]
fn holdout_disjoint_passes_and_overlaps_raise() {
    let train = ["a.py".to_string(), "b.py".to_string()]
        .into_iter()
        .collect();
    let disjoint = ["c.py".to_string()].into_iter().collect();
    regenerator::verify_holdout_disjoint(&train, &disjoint).unwrap();
    let overlap = ["a.py".to_string()].into_iter().collect();
    let err = regenerator::verify_holdout_disjoint(&train, &overlap).unwrap_err();
    assert!(err.message().contains("a.py"), "{err:?}");
}

// --- Refine/BestOfN controller ------------------------------------------------------------

#[test]
fn loop_four_turn_recovery_to_acceptance() {
    let work = scratch("loop4");
    let (req, frozen) = mini_request(&work);
    let script = vec![
        Ok(("".into(), obj(&[("stop_reason", J::str("max_tokens"))]))), // t1 a1: stop
        Ok((
            response_text(&[("src/mod.py", SYNTAX)], &[], &[]),
            end_turn(),
        )), // t1 a2: syntax
        Ok((response_text(&[("src/mod.py", GOOD)], &[], &[]), end_turn())), // t1 a3: tier-1 pass -> tier-2 kill
        Ok((response_text(&[("src/mod.py", GOOD)], &[], &[]), end_turn())), // t2 a1: weaker kill
        Ok((response_text(&[("src/mod.py", GOOD)], &[], &[]), end_turn())), // t3 a1: accepted
    ];
    let mut scripted = Scripted::new(script);
    let mut evaluator_calls = 0;
    let mut evaluator = eval_verdicts(vec![
        (
            1,
            Some("seq::op3".into()),
            obj(&[
                ("seed_group_pass_fraction", J::Float(0.5)),
                ("sequence_pass_fraction", J::Float(0.5)),
            ]),
        ),
        (
            1,
            Some("seq::op7".into()),
            obj(&[
                ("seed_group_pass_fraction", J::Float(0.4)),
                ("sequence_pass_fraction", J::Float(0.2)),
            ]),
        ),
        (0, None, obj(&[])),
    ]);
    let original = &mut evaluator;
    let mut counting = |_root: &Path| {
        evaluator_calls += 1;
        original(_root)
    };
    let outcome = run_loop(
        &req,
        &frozen,
        &work,
        "L1",
        RefineBudget::default(),
        Some(&mut counting),
        &mut scripted,
    )
    .unwrap();
    assert_eq!(outcome.status, "accepted");
    assert_eq!(outcome.best.score, 1.00);
    assert!(!outcome.adopted); // adoption is never claimed (Req 6.7)
    match outcome.rollout_history.last().unwrap() {
        J::Obj(pairs) => {
            let score = pairs.iter().find(|(k, _)| k == "score").unwrap();
            assert_eq!(score.1, J::Float(1.00));
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(evaluator_calls, 3); // tier-2 only after a tier-1 pass
    let labels = note_keys(&work.join("receipt.json"));
    assert!(labels.contains(&"L1-t3-r1-turn".to_string()));
    assert!(labels.iter().all(|k| !k.contains("-a")));
    assert_eq!(scripted.seen[1].1.temperature, Some(0.0)); // adaptive schedule engaged after the stop
}

#[test]
fn loop_concise_directive_dispatched_on_transport_retry() {
    let work = scratch("loop-directive");
    let (req, frozen) = mini_request(&work);
    let mut scripted = Scripted::new(vec![
        Err("RuntimeError: transport down".into()), // t1 a1: transport -> 0.00
        Ok((response_text(&[("src/mod.py", GOOD)], &[], &[]), end_turn())), // t1 a2: recovered
    ]);
    let outcome = run_loop(
        &req,
        &frozen,
        &work,
        "L5",
        RefineBudget::default(),
        Some(&mut eval_verdicts(vec![(0, None, obj(&[]))])),
        &mut scripted,
    )
    .unwrap();
    assert_eq!(outcome.status, "accepted");
    let directive = scripted.seen[1].1.directive.clone().expect("adapted");
    assert!(scripted.seen[0].1.directive.is_none());
    assert!(!scripted.seen[0].0.contains(&directive)); // first attempt: no directive yet
    assert!(scripted.seen[1].0.contains(&directive)); // dispatched in the retry prompt
                                                      // frozen per-attempt view: the directive appears only after adaptation
    let data: serde_json::Value =
        serde_json::from_slice(&fs::read(work.join("receipt.json")).unwrap()).unwrap();
    let attempts: Vec<&serde_json::Value> = data["notes"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|n| n["value"]["attempts"].as_array().into_iter().flatten())
        .collect();
    assert!(attempts[0]["preregistered_params"]
        .get("directive")
        .is_none());
    assert_eq!(
        attempts[1]["preregistered_params"]["directive"].as_str(),
        Some(directive.as_str())
    );
}

#[test]
fn loop_exhaustion_retains_best_trajectory() {
    let work = scratch("loop-exhaust");
    let (req, frozen) = mini_request(&work);
    let text = response_text(&[("src/mod.py", GOOD)], &[], &[]);
    let mut scripted = Scripted::new(vec![Ok((text, end_turn())); 12]);
    let verdict = (
        1,
        Some("seq::x".into()),
        obj(&[
            ("seed_group_pass_fraction", J::Float(0.4)),
            ("sequence_pass_fraction", J::Float(0.2)),
        ]),
    );
    let budget = RefineBudget {
        no_progress_limit: 1,
        ..RefineBudget::default()
    };
    let outcome = run_loop(
        &req,
        &frozen,
        &work,
        "L2",
        budget,
        Some(&mut eval_verdicts(vec![verdict; 12])),
        &mut scripted,
    )
    .unwrap();
    assert_eq!(outcome.status, "exhausted");
    assert_eq!(outcome.best.score, 0.68); // 0.675 half-up
    assert!(!outcome.unresolved_obligations.is_empty()); // divergence retained
}

#[test]
fn loop_exhaustion_k2_halts_and_retains_best() {
    // G1-R checklist item 3 protocol (task 8.6): k = 2 consecutive turns
    // without strict improvement halts as exhausted with the best kept.
    let work = scratch("loop-exhaust-k2");
    let (req, frozen) = mini_request(&work);
    let text = response_text(&[("src/mod.py", GOOD)], &[], &[]);
    let mut scripted = Scripted::new(vec![Ok((text, end_turn())); 8]);
    let verdict = (
        1,
        Some("seq::k2".into()),
        obj(&[
            ("seed_group_pass_fraction", J::Float(0.4)),
            ("sequence_pass_fraction", J::Float(0.4)),
        ]),
    );
    // The default budget is the preregistered k = 2 (no_progress_limit: 2).
    let outcome = run_loop(
        &req,
        &frozen,
        &work,
        "K2",
        RefineBudget::default(),
        Some(&mut eval_verdicts(vec![verdict; 8])),
        &mut scripted,
    )
    .unwrap();
    assert_eq!(outcome.status, "exhausted");
    assert_eq!(outcome.best.score, 0.70); // 0.60 + 0.25 * 0.4, exact-decimal
    assert!(!outcome.unresolved_obligations.is_empty()); // kill divergence retained
    let calls = match &outcome.usage {
        J::Obj(pairs) => pairs
            .iter()
            .find(|(k, _)| k == "calls")
            .map(|(_, v)| match v {
                J::Int(n) => *n,
                _ => -1,
            })
            .unwrap_or(-1),
        _ => -1,
    };
    assert!(calls > 0 && calls <= 4, "calls = {calls}"); // turn 1 + 2 flat turns
}

#[test]
fn loop_hash_mismatch_on_resume_aborts() {
    let work = scratch("loop-mismatch");
    let (req, frozen) = mini_request(&work);
    let cand = work.join("cand");
    regenerator::prepare_candidate(&frozen, &cand, &req.scope_files).unwrap();
    fs::write(cand.join("src/mod.py"), "tampered\n").unwrap(); // stale resume state
    let mut scripted = Scripted::new(vec![Ok((
        response_text(&[("src/mod.py", GOOD)], &[], &[]),
        end_turn(),
    ))]);
    let err = run_loop(
        &req,
        &frozen,
        &work,
        "L3",
        RefineBudget::default(),
        None,
        &mut scripted,
    )
    .unwrap_err();
    assert!(matches!(
        err,
        lesscode_core::regenerator::RegenError::CopyMismatch(_)
    ));
}

#[test]
fn loop_huge_usage_accepted_before_token_budget() {
    let work = scratch("loop-budget");
    let (req, frozen) = mini_request(&work);
    let text = response_text(&[("src/mod.py", GOOD)], &[], &[]);
    let meta = obj(&[
        ("stop_reason", J::str("end_turn")),
        (
            "usage",
            obj(&[
                ("input_tokens", J::int(500_000)),
                ("output_tokens", J::int(200_000)),
            ]),
        ),
    ]);
    let mut scripted = Scripted::new(vec![Ok((text, meta)); 6]);
    let outcome = run_loop(
        &req,
        &frozen,
        &work,
        "L4",
        RefineBudget::default(),
        Some(&mut eval_verdicts(vec![(0, None, obj(&[])); 6])),
        &mut scripted,
    )
    .unwrap();
    assert_eq!(outcome.status, "accepted"); // first clean verdict reaches 1.00 first
}

#[test]
fn loop_bestofn_width_two_picks_max() {
    let work = scratch("loop-bon");
    let (req, frozen) = mini_request(&work);
    let text = response_text(&[("src/mod.py", GOOD)], &[], &[]);
    let mut scripted = Scripted::new(vec![Ok((text, end_turn())); 8]);
    let verdicts = vec![
        (
            1,
            Some("a".into()),
            obj(&[
                ("seed_group_pass_fraction", J::Float(0.2)),
                ("sequence_pass_fraction", J::Float(0.2)),
            ]),
        ),
        (
            1,
            Some("b".into()),
            obj(&[
                ("seed_group_pass_fraction", J::Float(1.0)),
                ("sequence_pass_fraction", J::Float(1.0)),
            ]),
        ),
    ];
    let mut budget = RefineBudget {
        n_rollouts_per_turn: 2,
        max_static_retries: 1,
        max_turns: 1,
        ..RefineBudget::default()
    };
    budget.no_progress_limit = 2;
    let outcome = run_loop(
        &req,
        &frozen,
        &work,
        "L5",
        budget,
        Some(&mut eval_verdicts(verdicts)),
        &mut scripted,
    )
    .unwrap();
    let diagnostics = receipt::dumps(&outcome.best.diagnostics);
    assert!(diagnostics.contains("\"b\""), "{diagnostics}");
}

#[test]
fn loop_holdout_names_never_reach_the_next_prompt() {
    let work = scratch("loop-holdout");
    let (req, frozen) = mini_request(&work);
    let text = response_text(&[("src/mod.py", GOOD)], &[], &[]);
    let mut scripted = Scripted::new(vec![Ok((text, end_turn())); 6]);
    let err = regenerator::run_refine_loop(
        &req,
        &work.join("cand"),
        &frozen,
        &work.join("receipt.json"),
        "L6",
        &RefineBudget::default(),
        Some(&mut eval_verdicts(vec![
            (
                1,
                Some("divergence in HOLDOUT-9 replay".into()),
                obj(&[
                    ("seed_group_pass_fraction", J::Float(0.5)),
                    ("sequence_pass_fraction", J::Float(0.5)),
                ])
            );
            6
        ])),
        &mut |s, u, p| scripted.call(s, u, p),
        &["HOLDOUT-9".to_string()],
        None,
    )
    .unwrap_err();
    assert!(matches!(
        err,
        lesscode_core::regenerator::RegenError::FeedbackBoundary(_)
    ));
}

// --- Six adverse controls (fail closed, zero runtime calls) ---------------------------------

type Script = Vec<Result<(String, J), String>>;

#[test]
fn controls_fail_closed_without_runtime() {
    let cases: Vec<(&str, Script, f64, &str)> = vec![
        (
            "ctrl_max_tokens",
            vec![Ok((
                "".into(),
                obj(&[("stop_reason", J::str("max_tokens"))]),
            ))],
            0.00,
            "stop_reason",
        ),
        (
            "ctrl_out_of_scope",
            vec![Ok((
                response_text(&[("src/other.py", "x = 1\n")], &[], &[]),
                end_turn(),
            ))],
            0.20,
            "out_of_scope",
        ),
        (
            "ctrl_syntax_error",
            vec![Ok((
                response_text(&[("src/mod.py", SYNTAX)], &[], &[]),
                end_turn(),
            ))],
            0.30,
            "syntax_error",
        ),
        (
            "ctrl_missing_surface_b",
            vec![Ok((
                response_text(&[("src/mod.py", "def g():\n    return 2\n")], &[], &[]),
                end_turn(),
            ))],
            0.40,
            "surface",
        ),
        (
            "ctrl_write_in_pure",
            vec![Ok((
                response_text(
                    &[(
                        "src/mod.py",
                        "def f(conn):\n    conn.execute('sel')\n    return 1\n",
                    )],
                    &["mod.py::f"],
                    &[],
                ),
                end_turn(),
            ))],
            0.45,
            "structure",
        ),
    ];
    for (name, script, score, kind) in cases {
        let work = scratch(name);
        let (req, frozen) = mini_request(&work);
        let repeated: Vec<Result<(String, J), String>> =
            (0..4).flat_map(|_| script.iter().cloned()).collect();
        let mut scripted = Scripted::new(repeated);
        let mut evaluator_calls = 0;
        let outcome = run_loop(
            &req,
            &frozen,
            &work,
            name,
            RefineBudget {
                max_turns: 1,
                max_static_retries: 4,
                ..RefineBudget::default()
            },
            Some(&mut |_root: &Path| {
                evaluator_calls += 1;
                (0, None, obj(&[]))
            }),
            &mut scripted,
        )
        .unwrap();
        assert_eq!(outcome.best.score, score, "{name}");
        assert_eq!(outcome.best.failure_kind.as_deref(), Some(kind), "{name}");
        assert_eq!(
            evaluator_calls, 0,
            "{name}: Tier-1 controls never launch the runtime oracle"
        );
    }
}

#[test]
fn control_feedback_leak_rejected_before_dispatch() {
    let payload = obj(&[
        ("kind", J::str("failed")),
        ("first_divergence", J::str("div in HOLDOUT-3")),
        ("measure_gaps", obj(&[])),
    ]);
    let err = regenerator::enforce_feedback_boundary(&payload, &["HOLDOUT-3".into()]).unwrap_err();
    assert!(matches!(
        err,
        lesscode_core::regenerator::RegenError::FeedbackBoundary(_)
    ));
}

// --- OpenAI-compatible adapter (injected transport) -----------------------------------------

#[test]
fn openai_adapter_payload_and_hash_fields() {
    let ep = Endpoint::new(
        "openai",
        "http://localhost:8080/v1",
        "local-model",
        "sk-local",
    );
    type Captured = (String, Vec<(String, String)>, Vec<u8>);
    let captured: std::cell::RefCell<Option<Captured>> = std::cell::RefCell::new(None);
    let transport =
        |url: &str, headers: &[(String, String)], body: &[u8]| -> Result<Vec<u8>, String> {
            *captured.borrow_mut() = Some((url.to_string(), headers.to_vec(), body.to_vec()));
            Ok(json!({
            "choices": [{
                "message": {"content": "{\"files\": [], \"structure\": {}, \"change_log\": \"\"}"},
                "finish_reason": "stop"
            }],
            "usage": {"input_tokens": 10, "output_tokens": 5},
        })
        .to_string()
        .into_bytes())
        };
    let (text, meta) = regenerator::call(
        &ep,
        "system text",
        "user text",
        512,
        Some("low"),
        None,
        None,
        &transport,
    )
    .unwrap();
    let (url, headers, body) = captured.borrow().clone().expect("transport called");
    assert_eq!(url, "http://localhost:8080/v1/chat/completions");
    assert!(headers
        .iter()
        .any(|(k, v)| k == "Authorization" && v == "Bearer sk-local"));
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["model"], "local-model");
    assert_eq!(body["messages"][0]["role"], "system");
    assert_eq!(body["response_format"]["type"], "json_schema");
    assert_eq!(body["response_format"]["json_schema"]["name"], "regenerate");
    assert_eq!(
        body["response_format"]["json_schema"]["schema"]["additionalProperties"],
        false
    );
    assert_eq!(
        body["response_format"]["json_schema"]["schema"]["required"],
        json!(["files", "structure", "change_log"])
    );
    assert!(text.starts_with("{\"files\""));
    match &meta {
        J::Obj(pairs) => {
            let find = |key: &str| pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v);
            assert_eq!(find("stop_reason"), Some(&J::str("stop")));
            assert_eq!(
                find("system_sha256"),
                Some(&J::str(regenerator::sha256_text("system text")))
            );
            assert_eq!(
                find("user_sha256"),
                Some(&J::str(regenerator::sha256_text("user text")))
            );
            assert!(find("request_sha256").is_some());
        }
        other => panic!("{other:?}"),
    }
    assert!(!receipt::dumps(&meta).contains("sk-local"));
}

#[test]
fn openai_request_hash_pinned_against_python_reference() {
    // Reference digest produced by lesscode_baseline.regenerator on the same
    // inputs: json.dumps(request_kwargs(...), sort_keys=True,
    // ensure_ascii=False) -> f41a7d0d… (parity pin for the sorted spaced
    // dumps and the embedded SCHEMA, 2026-11-08).
    let ep = Endpoint::new(
        "openai",
        "http://localhost:8080/v1",
        "local-model",
        "sk-local",
    );
    let kwargs = regenerator::request_kwargs(
        &ep,
        "system text",
        "user text",
        512,
        Some("high"),
        None,
        None,
    );
    let hs = regenerator::hash_set(&kwargs, "RESP", "local-model");
    match hs {
        J::Obj(pairs) => {
            let req = pairs.iter().find(|(k, _)| k == "request_sha256").unwrap();
            assert_eq!(
                req.1,
                J::str("f41a7d0d2b0345643c1e00df477297183a9525599b4ed64262260ae9eb9aa25f")
            );
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn openai_adapter_transport_error_wrapped() {
    let ep = Endpoint::new("openai", "http://localhost:8080/v1", "m", "k");
    let err = regenerator::call(&ep, "s", "u", 64, Some("high"), None, None, &|_, _, _| {
        Err("openai transport: refused".into())
    })
    .unwrap_err();
    assert!(err.contains("transport"), "{err}");
}

// --- E4 three-turn replay (env-gated; task 8.5 done-when) -------------------------------------

fn frozen_response(turns: &Path, turn: &str) -> String {
    let msg: serde_json::Value =
        serde_json::from_slice(&fs::read(turns.join(turn).join("response.json")).unwrap()).unwrap();
    msg["content"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["type"] == "text")
        .and_then(|b| b["text"].as_str())
        .expect("first text block")
        .to_string()
}

fn materialize(work: &Path, response_text: &str, root: &Path, scope: &[String]) -> PathBuf {
    let result = regenerator::parse_result(response_text).unwrap();
    let cand = work.join("cand");
    let _ = fs::remove_dir_all(&cand);
    regenerator::prepare_candidate(root, &cand, scope).unwrap();
    regenerator::write_candidate(&result, &cand, scope).unwrap();
    cand
}

/// The frozen seven-file E4 scope (`surface4.py` SET, read the way s9 did;
/// embedded verbatim with provenance because the Rust suite cannot import
/// the frozen Python module).
const E4_SCOPE: [&str; 7] = [
    "src/facdrone/shells/lifecycle_monitor_runner.py",
    "src/facdrone/shells/lifecycle_store.py",
    "src/facdrone/shells/cli/lifecycle_group.py",
    "src/facdrone/shells/lifecycle_evidence.py",
    "src/facdrone/shells/lifecycle_projection.py",
    "src/facdrone/shells/cppi/lifecycle_seams.py",
    "src/facdrone/shells/wiring/lifecycle_root.py",
];

#[test]
fn e4_three_turn_replay_reproduces_recorded_transitions() {
    let work = scratch("e4-replay");
    let repo = repo();
    let e4_root = repo.join(".runs/e2-facdrone/source");
    let turns = repo.join("docs/experiments/deconvolution/regenerator/results/turns");
    let oracle = repo.join("docs/experiments/deconvolution/e4-lifecycle/e4_oracle.py");
    let frozen_surface =
        repo.join("docs/experiments/deconvolution/e4-lifecycle/results/public-surface.json");
    if !(e4_root.is_dir()
        && oracle.is_file()
        && frozen_surface.is_file()
        && (turns.join("e4-turn1/response.json").is_file()
            && turns.join("e4-turn2/response.json").is_file()
            && turns.join("e4-turn3/response.json").is_file()))
    {
        eprintln!("skipping: frozen E4 replay artifacts absent");
        return;
    }
    let scope: Vec<String> = E4_SCOPE.iter().map(|s| s.to_string()).collect();
    // One byte-copy of the frozen tree serves as the Contract-B root so the
    // recorder oracle sits under root/ (derive_surface resolves recorder
    // files as root/rel); the frozen specimen itself is never modified.
    let contract_root = work.join("frozen-contract");
    regenerator::prepare_candidate(&e4_root, &contract_root, &scope).unwrap();
    let recorder_rel = "docs/experiments/deconvolution/e4-lifecycle/e4_oracle.py";
    let oracle_copy = contract_root.join(recorder_rel);
    fs::create_dir_all(oracle_copy.parent().unwrap()).unwrap();
    fs::copy(&oracle, &oracle_copy).unwrap();
    let contract_b =
        surface::derive_surface(&contract_root, &scope, &[recorder_rel.to_string()], &[]).unwrap();
    let req = TurnRequest {
        scope_root: e4_root.to_string_lossy().into_owned(),
        scope_files: scope.clone(),
        surface_path: frozen_surface.to_string_lossy().into_owned(),
        definitions: String::new(),
        targets: vec![],
        contract: String::new(),
        acceptance_command: String::new(),
        feedback: None,
        language: "python".into(),
    };
    // Turn 1: recorded stop_reason max_tokens (frozen receipt: e4-turn1).
    let t0 = Instant::now();
    let t1 = regenerator::evaluate_tier1_static(
        &req,
        &outcome_failed("stop_reason", "max_tokens"),
        &e4_root,
        &contract_b,
        &[],
    )
    .unwrap();
    assert_eq!(
        (t1.score, t1.failure_kind.as_deref()),
        (0.00, Some("stop_reason"))
    );
    assert!(t0.elapsed().as_secs_f64() < 2.0);
    // Turn 2: the recorded response omitted `labeled_reason` (Contract B catches it).
    let text2 = frozen_response(&turns, "e4-turn2");
    let cand2 = materialize(&work.join("e4t2"), &text2, &contract_root, &scope);
    let t0 = Instant::now();
    let t2 = regenerator::evaluate_tier1_static(&req, &outcome_ok(None), &cand2, &contract_b, &[])
        .unwrap();
    assert_eq!(
        (t2.score, t2.failure_kind.as_deref()),
        (0.40, Some("surface"))
    );
    assert!(receipt::dumps(&t2.diagnostics).contains("labeled_reason"));
    assert!(t0.elapsed().as_secs_f64() < 2.0);
    // Turn 3: the recorded accepted response passes Tier 1; the Tier-2
    // verdict is replayed from the frozen R31 receipt (deterministic).
    let text3 = frozen_response(&turns, "e4-turn3");
    let cand3 = materialize(&work.join("e4t3"), &text3, &contract_root, &scope);
    let t0 = Instant::now();
    let t3 = regenerator::evaluate_tier1_static(&req, &outcome_ok(None), &cand3, &contract_b, &[])
        .unwrap();
    assert_eq!((t3.score, t3.failure_kind.as_deref()), (0.50, None));
    assert!(t3.passed_tier1);
    let tier2 = regenerator::evaluate_tier2_runtime(0, None, &obj(&[]));
    assert_eq!(tier2.score, 1.00);
    assert!(t0.elapsed().as_secs_f64() < 2.0);
}

// --- Runtime oracle through the recording driver (env-gated) ----------------------------------

#[test]
fn runtime_oracle_maps_verdict_files() {
    let work = scratch("oracle");
    let venv = repo().join(".runs/e2-facdrone/venv/Scripts/python.exe");
    if !venv.is_file() {
        eprintln!("skipping: pinned specimen venv absent");
        return;
    }
    let oracle = RuntimeOracle {
        python: venv.clone(),
        script_rel: "oracle.py".into(),
        timeout_s: 60,
    };
    // absent script -> infrastructure, never a kill
    let cand = work.join("absent");
    fs::create_dir_all(&cand).unwrap();
    let (code, kind_hint, _) = {
        let r = oracle.evaluate(&cand);
        (r.0, r.1.clone(), r.2)
    };
    assert_eq!((code, kind_hint.is_some()), (2, true));
    // partial verdict -> kill with pinned fractions
    let cand = work.join("partial");
    fs::create_dir_all(&cand).unwrap();
    fs::write(
        cand.join("oracle.py"),
        r#"import json
v = {"ok": False, "passed": 4, "total": 6,
     "failures": [{"case": 0, "name": "unit_price pear", "error": "AssertionError"}]}
json.dump(v, open("oracle-verdict.json", "w"))
"#,
    )
    .unwrap();
    let (code, divergence, results) = oracle.evaluate(&cand);
    assert_eq!(code, 1);
    assert!(divergence
        .as_deref()
        .unwrap_or_default()
        .starts_with("case 0 (unit_price pear)"));
    assert_eq!(
        results,
        obj(&[
            ("seed_group_pass_fraction", J::Float(0.6667)),
            ("sequence_pass_fraction", J::Float(0.6667)),
        ])
    );
    let kill = regenerator::evaluate_tier2_runtime(code, divergence.as_deref(), &results);
    assert_eq!(kill.score, 0.77); // 0.60 + 0.25*0.6667 = 0.766675, half-up
                                  // ok verdict -> accepted, no divergence
    let cand = work.join("ok");
    fs::create_dir_all(&cand).unwrap();
    fs::write(
        cand.join("oracle.py"),
        r#"import json
json.dump({"ok": True, "passed": 6, "total": 6, "failures": []}, open("oracle-verdict.json", "w"))
"#,
    )
    .unwrap();
    let (code, divergence, _) = oracle.evaluate(&cand);
    assert_eq!((code, divergence.is_none()), (0, true));

    // exit 2 after writing ok verdict -> process failed, infrastructure error
    let cand = work.join("exit2");
    fs::create_dir_all(&cand).unwrap();
    fs::write(
        cand.join("oracle.py"),
        r#"import json
json.dump({"ok": True, "passed": 6, "total": 6, "failures": []}, open("oracle-verdict.json", "w"))
raise SystemExit(2)
"#,
    )
    .unwrap();
    let (code, divergence, results) = oracle.evaluate(&cand);
    assert_eq!(code, 2);
    let reward = regenerator::evaluate_tier2_runtime(code, divergence.as_deref(), &results);
    assert_eq!(reward.failure_kind.as_deref(), Some("runtime_infrastructure"));

    // timeout after writing ok verdict -> process timeout, infrastructure error
    let cand = work.join("timeout");
    fs::create_dir_all(&cand).unwrap();
    fs::write(
        cand.join("oracle.py"),
        r#"import json, time
json.dump({"ok": True, "passed": 6, "total": 6, "failures": []}, open("oracle-verdict.json", "w"))
time.sleep(5)
"#,
    )
    .unwrap();
    let timeout_oracle = RuntimeOracle {
        python: venv.clone(),
        script_rel: "oracle.py".into(),
        timeout_s: 1,
    };
    let (code, divergence, results) = timeout_oracle.evaluate(&cand);
    assert_ne!(code, 0);
    assert_ne!(code, 1);
    let reward = regenerator::evaluate_tier2_runtime(code, divergence.as_deref(), &results);
    assert_eq!(reward.failure_kind.as_deref(), Some("runtime_infrastructure"));

    // contradictory counts: ok=true but passed=0, total=1, failures present
    let cand = work.join("contradictory");
    fs::create_dir_all(&cand).unwrap();
    fs::write(
        cand.join("oracle.py"),
        r#"import json
json.dump({"ok": True, "passed": 0, "total": 1, "failures": [{"case": 1, "error": "mismatch"}]}, open("oracle-verdict.json", "w"))
"#,
    )
    .unwrap();
    let (code, divergence, results) = oracle.evaluate(&cand);
    let reward = regenerator::evaluate_tier2_runtime(code, divergence.as_deref(), &results);
    assert!(reward.score < 1.0);
}

// --- Admission verification of emitted receipts (done-when) ------------------------------------

#[test]
fn loop_receipts_verify_under_admission() {
    let work = scratch("admission");
    let (req, frozen) = mini_request(&work);
    let text = response_text(&[("src/mod.py", GOOD)], &[], &[]);
    let mut scripted = Scripted::new(vec![
        Ok(("".into(), obj(&[("stop_reason", J::str("max_tokens"))]))),
        Ok((
            response_text(&[("src/mod.py", SYNTAX)], &[], &[]),
            end_turn(),
        )),
        Ok((text.clone(), end_turn())),
        Ok((text, end_turn())),
    ]);
    let outcome = run_loop(
        &req,
        &frozen,
        &work,
        "ADM",
        RefineBudget::default(),
        Some(&mut eval_verdicts(vec![
            (
                1,
                Some("seq::a".into()),
                obj(&[
                    ("seed_group_pass_fraction", J::Float(0.5)),
                    ("sequence_pass_fraction", J::Float(0.5)),
                ]),
            ),
            (0, None, obj(&[])),
        ])),
        &mut scripted,
    )
    .unwrap();
    assert_eq!(outcome.status, "accepted");
    assert!(work.join("bindings.json").is_file());
    let out = work.join("admission.json");
    let code = lesscode_core::admission::cli_run(&work, &work.join("bindings.json"), &out)
        .expect("admission run");
    assert_eq!(code, 0, "receipt must verify under lesscode-core admission");
}
