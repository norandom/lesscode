//! End-to-end specimen integration and adverse sensitivity verification for Java.
//!
//! Governed by:
//! - REQ-JAVA-2.1, REQ-JAVA-3.2, REQ-JAVA-4.4, REQ-JAVA-5.4
//! - `.kiro/specs/java-deconvolution-unification/tasks.md` Task 5

use lesscode_core::adapter_traits::*;
use lesscode_core::adapters::java::metrics::*;
use lesscode_core::adapters::java::JavaAdapter;
use std::path::Path;

#[test]
fn specimen_candidate_meets_useful_compression_thresholds() {
    let specimen_path = Path::new(
        "C:/Users/mariu/Source/commons-csv/src/main/java/org/apache/commons/csv/CSVPrinter.java",
    );
    let candidate_path = Path::new("adapters/java/candidate_CSVPrinter.java");

    if !specimen_path.exists() || !candidate_path.exists() {
        eprintln!(
            "commons-csv specimen not found at {specimen_path:?}; skipping disk metrics test"
        );
        return;
    }

    let base_text = std::fs::read_to_string(specimen_path).unwrap();
    let cand_text = std::fs::read_to_string(candidate_path).unwrap();

    let base_metrics = extract_scope_metrics(&base_text);
    let cand_metrics = extract_scope_metrics(&cand_text);

    let report = evaluate_java_compression(&base_metrics, &cand_metrics);
    assert!(report.compression_met);
    assert_eq!(report.disposition, "QUALIFIED");
    assert_eq!(report.baseline_lines, 221);
    assert_eq!(report.candidate_lines, 136);
    assert_eq!(report.baseline_stmts, 112);
    assert_eq!(report.candidate_stmts, 81);
    assert!(report.line_reduction_pct >= 20.0);
    assert!(report.stmt_reduction_pct >= 10.0);
}

// The test that counted a changed text as a killed control was removed with
// the text-matching operators (agnostic-annotation-driver task 7.3). What the
// driver offers for the gate's controls is pinned in `driver_contract.rs`.

#[test]
fn adapter_protocol_event_execution_and_clean_exit() {
    let check = std::process::Command::new("java").arg("-version").output();
    if !check.as_ref().is_ok_and(|o| o.status.success()) {
        eprintln!("java binary not available or failed; skipping java process runner test");
        return;
    }
    let adapter = JavaAdapter::new();
    let req = RunRequest {
        command: "java".into(),
        args: vec!["-version".into()],
        env: vec![],
        cwd: None,
        timeout_ms: 15_000,
        max_output_bytes: 8192,
    };

    let res = adapter.run(&req);
    assert!(res.success);
    let out = format!(
        "{}{}",
        String::from_utf8_lossy(&res.stdout_bytes),
        String::from_utf8_lossy(&res.stderr_bytes)
    );
    assert!(out.contains("version"));
}
