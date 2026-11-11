//! Test suite for TypeScript runtime adapter and trait implementations.
//!
//! Governed by:
//! - REQ-TS-2.1 through REQ-TS-2.4, REQ-TS-4.1 through REQ-TS-4.4
//! - `.kiro/specs/typescript-deconvolution-adapter/tasks.md` Task 4

use lesscode_core::adapter_traits::*;
use lesscode_core::adapters::typescript::TypeScriptAdapter;

#[test]
fn typescript_adapter_initializes_with_profile() {
    let adapter = TypeScriptAdapter::new();
    assert_eq!(adapter.profile_id, "ts-node-v1");
}

#[test]
fn adapter_state_frame_analyzer_integration() {
    let adapter = TypeScriptAdapter::new();
    let req = AnalysisRequest {
        scope_name: "test".into(),
        source_paths: vec![],
        frame_spec: FrameSpec {
            allowed_write_sites: vec!["authorized.ts".into()],
            permitted_shell_locations: vec!["shell.ts".into()],
            failure_invariants: vec![],
        },
        effect_profile: "ts-node-v1".into(),
    };
    let res = adapter.analyze(&req);
    assert!(res.is_valid);
}

#[test]
fn adapter_trace_runner_executes_process() {
    let check = std::process::Command::new("node").arg("--version").output();
    if !check.as_ref().is_ok_and(|o| o.status.success()) {
        eprintln!("node binary not available or failed; skipping node process runner test");
        return;
    }
    let adapter = TypeScriptAdapter::new();
    let req = RunRequest {
        command: "node".into(),
        args: vec!["-e".into(), "console.log('runner_ok')".into()],
        env: vec![],
        cwd: None,
        timeout_ms: 15_000,
        max_output_bytes: 4096,
    };
    let res = adapter.run(&req);
    assert_eq!(res.exit_code, 0);
    assert!(String::from_utf8_lossy(&res.stdout_bytes).contains("runner_ok"));
}
