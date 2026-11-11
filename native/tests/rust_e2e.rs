//! End-to-end specimen integration and adverse sensitivity verification for Rust.
//!
//! Governed by:
//! - REQ-RUST-2.1, REQ-RUST-3.2, REQ-RUST-4.4, REQ-RUST-5.4
//! - `.kiro/specs/rust-deconvolution-unification/tasks.md` Task 5

use lesscode_core::adapter_traits::*;
use lesscode_core::adapters::rust::RustAdapter;

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
