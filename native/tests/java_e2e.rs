//! End-to-end specimen integration and adverse sensitivity verification for Java.
//!
//! Governed by:
//! - REQ-JAVA-2.1, REQ-JAVA-3.2, REQ-JAVA-4.4, REQ-JAVA-5.4
//! - `.kiro/specs/java-deconvolution-unification/tasks.md` Task 5

use lesscode_core::adapter_traits::*;
use lesscode_core::adapters::java::JavaAdapter;

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
