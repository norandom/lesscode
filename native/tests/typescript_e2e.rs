//! End-to-end specimen integration and adverse sensitivity verification for TypeScript.
//!
//! Governed by:
//! - REQ-TS-2.1, REQ-TS-3.2, REQ-TS-4.4, REQ-TS-5.4
//! - `.kiro/specs/typescript-deconvolution-adapter/tasks.md` Task 5

use lesscode_core::adapter_traits::*;
use lesscode_core::adapters::typescript::metrics::*;
use lesscode_core::adapters::typescript::TypeScriptAdapter;
use std::path::Path;

#[test]
fn specimen_candidate_meets_useful_compression_thresholds() {
    let specimen_root = Path::new("../OpenUltraCode");
    if !specimen_root.exists() {
        eprintln!("OpenUltraCode specimen not found at ../OpenUltraCode; skipping e2e disk test");
        return;
    }

    let baseline_path = specimen_root.join("src/degradation.ts");
    let candidate_path = Path::new("adapters/typescript/candidate_degradation.ts");

    let base_text = std::fs::read_to_string(&baseline_path).unwrap();
    let cand_text = std::fs::read_to_string(candidate_path).unwrap();

    let base_metrics = extract_scope_metrics(&base_text);
    let cand_metrics = extract_scope_metrics(&cand_text);

    let report = evaluate_typescript_compression(&base_metrics, &cand_metrics);
    assert!(report.compression_met);
    assert_eq!(report.disposition, "QUALIFIED");
    assert!(report.line_reduction_pct >= 20.0);
    assert!(report.stmt_reduction_pct >= 10.0);
}

// The test that counted a changed text as a killed control was removed with
// the text-matching operators (agnostic-annotation-driver task 7.3). What the
// driver offers for the gate's controls is pinned in `driver_contract.rs`.

#[test]
fn adapter_protocol_event_execution_and_clean_exit() {
    let check = std::process::Command::new("node").arg("--version").output();
    if !check.as_ref().is_ok_and(|o| o.status.success()) {
        eprintln!("node binary not available or failed; skipping node process runner test");
        return;
    }
    let adapter = TypeScriptAdapter::new();
    let req = RunRequest {
        command: "node".into(),
        args: vec![
            "-e".into(),
            r#"console.log(JSON.stringify({"event":"start","protocol_version":1})); console.log(JSON.stringify({"event":"complete","status":"success"}));"#.into(),
        ],
        env: vec![],
        cwd: None,
        timeout_ms: 15_000,
        max_output_bytes: 8192,
    };

    let res = adapter.run(&req);
    assert!(res.success);
    assert_eq!(res.events.len(), 0); // parse_protocol_events parses RunEvent typed records
    assert!(String::from_utf8_lossy(&res.stdout_bytes).contains("complete"));
}
