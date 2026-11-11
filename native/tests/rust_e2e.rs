//! End-to-end specimen integration and adverse sensitivity verification for Rust.
//!
//! Governed by:
//! - REQ-RUST-2.1, REQ-RUST-3.2, REQ-RUST-4.4, REQ-RUST-5.4
//! - `.kiro/specs/rust-deconvolution-unification/tasks.md` Task 5

use lesscode_core::adapter_traits::*;
use lesscode_core::adapters::rust::metrics::*;
use lesscode_core::adapters::rust::RustAdapter;
use std::path::Path;

#[test]
fn specimen_candidate_meets_useful_compression_thresholds() {
    let baseline_path = if Path::new("adapters/rust/baseline_lifecycle.rs").exists() {
        Path::new("adapters/rust/baseline_lifecycle.rs").to_path_buf()
    } else {
        Path::new("native/adapters/rust/baseline_lifecycle.rs").to_path_buf()
    };
    let candidate_path = if Path::new("adapters/rust/candidate_lifecycle.rs").exists() {
        Path::new("adapters/rust/candidate_lifecycle.rs").to_path_buf()
    } else {
        Path::new("native/adapters/rust/candidate_lifecycle.rs").to_path_buf()
    };

    assert!(baseline_path.exists(), "Baseline specimen must exist");
    assert!(candidate_path.exists(), "Candidate specimen must exist");

    let base_text = std::fs::read_to_string(&baseline_path).unwrap();
    let cand_text = std::fs::read_to_string(&candidate_path).unwrap();

    let base_metrics = extract_scope_metrics(&base_text);
    let cand_metrics = extract_scope_metrics(&cand_text);

    let report = evaluate_rust_compression(&base_metrics, &cand_metrics);
    assert!(report.compression_met);
    assert_eq!(report.disposition, "QUALIFIED");
    assert_eq!(report.baseline_lines, 248);
    assert_eq!(report.candidate_lines, 148);
    assert_eq!(report.baseline_stmts, 83);
    assert_eq!(report.candidate_stmts, 70);
    assert!(report.line_reduction_pct >= 20.0);
    assert!(report.stmt_reduction_pct >= 10.0);
}

// The test that counted a changed text as a killed control was removed with
// the text-matching operators (agnostic-annotation-driver task 7.3). What the
// driver offers for the gate's controls is pinned in `driver_contract.rs`.

#[test]
fn adapter_protocol_event_execution_and_clean_exit() {
    let adapter = RustAdapter::new();
    let req = RunRequest {
        command: "rustc".into(),
        args: vec!["--version".into()],
        env: vec![],
        cwd: None,
        timeout_ms: 60_000,
        max_output_bytes: 8192,
    };

    let res = adapter.run(&req);
    assert!(res.success);
    let out = format!(
        "{}{}",
        String::from_utf8_lossy(&res.stdout_bytes),
        String::from_utf8_lossy(&res.stderr_bytes)
    );
    assert!(out.contains("rustc"));
}
