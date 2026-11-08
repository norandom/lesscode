//! RecordingCore Rust-substrate suite (task 8.4): ports the host-independent
//! contract cases of `tests/test_recording.py` — verdict truth table, call()
//! deadline enforcement, first_diff structural paths, golden-body writer and
//! signature semantics — and the specimen-venv digest gates (double-record
//! identity against the frozen LF-body digests, frozen-trace replay). The
//! env-gated gates skip with an explicit reason when the pinned specimen venv
//! or the frozen recordings are absent; absence stays absent, never zero.
//!
//! Port delta: `Norm` cases do not port — declared normalization executes
//! inside the frozen oracle envs (their own `Norm`), reached through the
//! embedded shim; the engine port receives already-normalized JSON.

use lesscode_core::receipt::J;
use lesscode_core::recording::{self, ScopePlan};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn scratch(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("lc-recording-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

fn obj(pairs: &[(&str, J)]) -> J {
    J::obj(pairs.to_vec())
}

// --- verdict truth table ----------------------------------------------------

#[test]
fn verdict_truth_table() {
    assert_eq!(recording::verdict(0, true, 0, None), (0, vec![]));
    let stale = recording::verdict(0, false, 0, None);
    assert_eq!(stale.0, 1);
    assert!(stale.1.iter().any(|r| r.contains("fresh")));
    let unbacked = recording::verdict(0, true, 3, None);
    assert_eq!(unbacked.0, 1);
    assert!(unbacked.1.iter().any(|r| r.contains("L2")));
    assert_eq!(
        recording::verdict(2, true, 0, None),
        (2, vec!["arm exit 2".into()])
    );
    assert_eq!(
        recording::verdict(1, true, 0, None),
        (1, vec!["arm exit 1".into()])
    );
}

#[test]
fn verdict_admits_l2_with_bound_query_evidence() {
    let dir = scratch("verdict");
    let evidence = dir.join("query-differential.json");
    fs::write(&evidence, "{}").unwrap();
    assert_eq!(recording::verdict(0, true, 3, Some(&evidence)), (0, vec![]));
    let missing = dir.join("absent-evidence.json");
    let (code, reasons) = recording::verdict(0, true, 3, Some(&missing));
    assert_eq!(code, 1);
    assert!(reasons.iter().any(|r| r.contains("L2")));
}

// --- call() deadline enforcement (taskkill tree kill is Windows-only) --------

#[test]
#[cfg(windows)]
fn call_returns_arm_exit_without_deadline() {
    let code = recording::call("cmd", &["/c", "exit", "7"], &scratch("call-a"), &[], None);
    assert_eq!(code, 7);
}

#[test]
#[cfg(windows)]
fn call_deadline_expiry_returns_124_not_a_verdict() {
    let t0 = Instant::now();
    // ~30s sleeper; the deadline fires after 1s and tree-kills it.
    let code = recording::call(
        "cmd",
        &["/c", "ping", "-n", "30", "127.0.0.1"],
        &scratch("call-b"),
        &[],
        Some(t0 + std::time::Duration::from_secs(1)),
    );
    assert_eq!(code, 124);
    assert!(t0.elapsed().as_secs() < 20);
}

// --- first_diff: structural divergence paths ----------------------------------

#[test]
fn first_diff_reports_structural_paths() {
    let none = recording::first_diff(&obj(&[("a", J::int(1))]), &obj(&[("a", J::int(1))]), "");
    assert!(none.is_none());
    let leaf = recording::first_diff(
        &obj(&[("a", obj(&[("b", J::int(1))]))]),
        &obj(&[("a", obj(&[("b", J::int(2))]))]),
        "",
    )
    .unwrap();
    assert!(leaf.starts_with("/a/b"), "{leaf}");
    let missing = recording::first_diff(
        &obj(&[("a", J::int(1))]),
        &obj(&[("a", J::int(1)), ("k", J::int(2))]),
        "",
    )
    .unwrap();
    assert!(
        missing.starts_with("/k") && missing.contains("one side only"),
        "{missing}"
    );
    let indexed = recording::first_diff(
        &J::arr(vec![J::int(1), J::int(2), J::int(3)]),
        &J::arr(vec![J::int(1), J::int(4), J::int(3)]),
        "",
    )
    .unwrap();
    assert!(indexed.starts_with("[1]"), "{indexed}");
    let length = recording::first_diff(
        &J::arr(vec![J::int(1)]),
        &J::arr(vec![J::int(1), J::int(2)]),
        "",
    )
    .unwrap();
    assert!(
        length.contains("length 1 recorded vs 2 replayed"),
        "{length}"
    );
    let deep = recording::first_diff(
        &obj(&[(
            "seqs",
            J::arr(vec![obj(&[("ops", J::arr(vec![J::str("a")]))])]),
        )]),
        &obj(&[(
            "seqs",
            J::arr(vec![obj(&[(
                "ops",
                J::arr(vec![J::str("a"), J::str("b")]),
            )])]),
        )]),
        "",
    )
    .unwrap();
    assert!(
        deep.starts_with("/seqs[0]/ops") && deep.contains("length"),
        "{deep}"
    );
}

// --- golden body writer (json.dumps sort_keys/indent=0 parity) ----------------

#[test]
fn golden_writer_matches_python_dumps_shape() {
    // Expected body produced by Python:
    //   json.dumps(v, sort_keys=True, indent=0) for the frozen writer shape.
    let value = obj(&[
        (
            "b",
            J::arr(vec![
                J::int(1),
                J::Float(2.5),
                J::str("x"),
                J::Null,
                J::Bool(true),
            ]),
        ),
        ("a", J::obj(vec![])),
    ]);
    let body = recording::golden_dumps(&value);
    let expected = "{\n\"a\": {},\n\"b\": [\n1,\n2.5,\n\"x\",\nnull,\ntrue\n]\n}";
    assert_eq!(body, expected);
}

#[test]
fn float_repr_matches_python_switch_points() {
    // repr switch points: fixed at 1e-4/1e16, scientific outside with
    // two-digit exponents; integral values keep ".0".
    let floats = [
        (1.387_778_780_781_445_7e-17_f64, "1.3877787807814457e-17"),
        (1e-5_f64, "1e-05"),
        (1e-4_f64, "0.0001"),
        (1e16_f64, "1e+16"),
        (1e15_f64, "1000000000000000.0"),
        (2.0_f64, "2.0"),
        (0.5_f64, "0.5"),
    ];
    for (x, want) in floats {
        let body = recording::golden_dumps(&J::Float(x));
        assert_eq!(body, want, "repr of {x}");
    }
}

#[test]
fn canon_writer_is_compact_and_sorted() {
    let value = obj(&[("b", J::int(2)), ("a", J::arr(vec![J::str("s")]))]);
    let mut out = String::new();
    recording::canon_dumps(&value, &mut out);
    assert_eq!(out, "{\"a\":[\"s\"],\"b\":2}");
}

#[test]
fn comma_count_uses_thousands_separators() {
    assert_eq!(recording::comma_count(0), "0");
    assert_eq!(recording::comma_count(999), "999");
    assert_eq!(recording::comma_count(5479067), "5,479,067");
}

// --- signatures --------------------------------------------------------------

#[test]
fn signatures_map_results_and_errors() {
    let seq = obj(&[
        (
            "ops",
            J::arr(vec![
                obj(&[("op", J::str("commit"))]),
                obj(&[("op", J::str("sim_persist"))]),
                obj(&[("op", J::str("slice"))]),
            ]),
        ),
        (
            "observations",
            J::arr(vec![
                obj(&[("result", J::int(1))]),
                obj(&[(
                    "error",
                    J::arr(vec![J::str("ValueError"), J::int(0), J::Null]),
                )]),
                obj(&[(
                    "error",
                    J::arr(vec![
                        J::str("SliceConflictError"),
                        J::int(0),
                        J::str("ValueError"),
                    ]),
                )]),
            ]),
        ),
    ]);
    let sigs = recording::signatures(&[seq]);
    // Error details are Python str() (`None` for null, verbatim strings);
    // the frozen format opens '<' and never closes it.
    assert_eq!(
        sigs,
        vec![
            "commit:ok".to_string(),
            "sim_persist:ValueError<None".to_string(),
            "slice:SliceConflictError<ValueError".to_string()
        ]
    );
}

// --- build_groups ------------------------------------------------------------

#[test]
fn build_groups_orders_and_deselects_like_the_reference() {
    let files = [
        "tests/test_a.py".to_string(),
        "tests/test_b.py".to_string(),
        "packages/facdrone-research/test_r.py".to_string(),
    ];
    let bad = [
        "tests/test_a.py::test_one".to_string(),
        "packages/facdrone-research/test_r.py::test_gone".to_string(),
        "packages/facdrone-dash/test_x.py::test_other".to_string(),
    ];
    let groups = recording::build_groups(
        &files,
        &bad,
        &["research", "dash"],
        Some(Path::new("proto/replay_test.py")),
    );
    assert_eq!(groups.len(), 2); // dash has no seed files -> no group
    assert_eq!(groups[0].0, ".");
    assert_eq!(
        groups[0].1,
        vec![
            "tests/test_a.py".to_string(),
            "tests/test_b.py".to_string(),
            "--deselect=tests/test_a.py::test_one".to_string(),
            "-p".to_string(),
            "facdrone_testing.fixtures".to_string(),
            "proto/replay_test.py".to_string(),
        ]
    );
    assert_eq!(groups[1].0, "packages/facdrone-research");
    assert_eq!(
        groups[1].1,
        vec![
            "test_r.py".to_string(),
            "--deselect=test_r.py::test_gone".to_string(),
        ]
    );
}

// --- specimen-venv digest gates (env-gated; the task 8.4 done-when) ----------

fn specimen_plan(scope: &str, work: &Path) -> Option<ScopePlan> {
    let repo = repo();
    let venv = repo.join(".runs/e2-facdrone/venv/Scripts/python.exe");
    let shim = repo.join("native/shim/recording_shim.py");
    let probe_golden = repo.join(".runs/native-prototype/proto2-e3-a.json");
    if !(venv.is_file() && shim.is_file() && probe_golden.is_file()) {
        eprintln!("skipping: pinned specimen venv or frozen proto2 recordings absent");
        return None;
    }
    Some(ScopePlan {
        scope: scope.into(),
        shim,
        venv,
        work_dir: work.join(scope),
    })
}

/// Cross-process lock so the two records never race the embedded pg server
/// (nextest runs one process per test; the frozen pytest ran them serially).
struct DirLock(PathBuf);
impl DirLock {
    fn acquire(name: &str) -> Self {
        let lock = std::env::temp_dir().join(format!("lc-recording-lock-{name}"));
        let start = Instant::now();
        while fs::create_dir(&lock).is_err() {
            if start.elapsed().as_secs() > 1800 {
                panic!("record lock held for over 30 minutes");
            }
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
        DirLock(lock)
    }
}
impl Drop for DirLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.0);
    }
}

#[test]
fn e3_double_record_reproduces_frozen_lf_body_digest() {
    let work = scratch("e3-record");
    let Some(plan) = specimen_plan("e3", &work) else {
        return;
    };
    let _lock = DirLock::acquire("specimen");
    let first = work.join("proto2-e3-a-rust.json");
    let outcome = recording::record_golden(&plan, &first).expect("e3 record");
    assert_eq!(outcome.sha256_lf_body, recording::FROZEN_E3_LF_BODY);
    assert_eq!((outcome.sequences, outcome.operations), (51, 290));
    // Double-record determinism through the same Rust driver.
    let second = work.join("proto2-e3-b-rust.json");
    let again = recording::record_golden(&plan, &second).expect("e3 re-record");
    assert_eq!(again.sha256_lf_body, recording::FROZEN_E3_LF_BODY);
    assert_eq!(fs::read(&second).unwrap(), fs::read(&first).unwrap());
}

#[test]
fn e4_double_record_reproduces_frozen_lf_body_digest() {
    let work = scratch("e4-record");
    let Some(plan) = specimen_plan("e4", &work) else {
        return;
    };
    let _lock = DirLock::acquire("specimen");
    let first = work.join("proto2-e4-a-rust.json");
    let outcome = recording::record_golden(&plan, &first).expect("e4 record");
    assert_eq!(outcome.sha256_lf_body, recording::FROZEN_E4_LF_BODY);
    assert_eq!((outcome.sequences, outcome.operations), (50, 317));
    let second = work.join("proto2-e4-b-rust.json");
    let again = recording::record_golden(&plan, &second).expect("e4 re-record");
    assert_eq!(again.sha256_lf_body, recording::FROZEN_E4_LF_BODY);
    assert_eq!(fs::read(&second).unwrap(), fs::read(&first).unwrap());
}

#[test]
fn frozen_traces_replay_without_divergence() {
    let work = scratch("replay");
    let repo = repo();
    let e3 = repo.join(".runs/native-prototype/proto2-e3-a.json");
    let e4 = repo.join(".runs/native-prototype/proto2-e4-a.json");
    let Some(mut plan) = specimen_plan("e3", &work) else {
        return;
    };
    if !e4.is_file() {
        eprintln!("skipping: frozen proto2 E4 recording absent");
        return;
    }
    let _lock = DirLock::acquire("specimen");
    assert_eq!(
        recording::replay_golden(&plan, &e3).expect("e3 replay"),
        None,
        "E3 frozen trace diverged"
    );
    plan.scope = "e4".into();
    plan.work_dir = work.join("e4-replay");
    assert_eq!(
        recording::replay_golden(&plan, &e4).expect("e4 replay"),
        None,
        "E4 frozen trace diverged"
    );
}
