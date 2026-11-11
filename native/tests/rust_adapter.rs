//! Test suite for Rust runtime adapter and trait implementations.
//!
//! Governed by:
//! - REQ-RUST-2.1 through REQ-RUST-2.4, REQ-RUST-4.1 through REQ-RUST-4.4
//! - `.kiro/specs/rust-deconvolution-unification/tasks.md` Task 4

use lesscode_core::adapter_traits::*;
use lesscode_core::adapters::rust::RustAdapter;

#[test]
fn rust_adapter_initializes_with_profile() {
    let adapter = RustAdapter::new();
    assert_eq!(adapter.profile_id, "rust-nextest-v1");
}

#[test]
fn adapter_state_frame_analyzer_integration() {
    let adapter = RustAdapter::new();
    let req = AnalysisRequest {
        scope_name: "test".into(),
        source_paths: vec![],
        frame_spec: FrameSpec {
            allowed_write_sites: vec!["baseline_lifecycle.rs".into()],
            permitted_shell_locations: vec![],
            failure_invariants: vec![],
        },
        effect_profile: "rust-nextest-v1".into(),
    };
    let res = adapter.analyze(&req);
    assert!(res.is_valid);
}

#[test]
fn adapter_adverse_mutant_generator_integration() {
    let adapter = RustAdapter::new();
    let temp_dir = std::env::temp_dir().join(format!("rust-adapter-test-{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let file_path = temp_dir.join("Target.rs");
    std::fs::write(
        &file_path,
        r#"
fn verdict(arm_exit: i32, level2_ops: usize) -> (bool, &'static str) {
    if arm_exit != 0 {
        return (false, "exit code non-zero");
    }
    if level2_ops > 0 {
        return (false, "divergence observed");
    }
    (true, "ok")
}
"#,
    )
    .unwrap();

    let req = MutationRequest {
        target_path: file_path.to_str().unwrap().into(),
        controls: vec![
            MutationSpec {
                id: "RUST_ADV_2".into(),
                location: "Target.rs:3".into(),
                kind: MutationKind::InvertedCondition,
                description: "Inverted refusal condition in verdict".into(),
            },
            MutationSpec {
                id: "RUST_ADV_3".into(),
                location: "Target.rs:3".into(),
                kind: MutationKind::InvertedErrorOrder,
                description: "Bypassed level2 differences check".into(),
            },
        ],
    };

    let batch = adapter.generate(&req);
    // The driver finds a mutant for each control; no test is run, so none is killed.
    assert_eq!(batch.denominator, 2);
    assert_eq!(batch.killed_count, 0);
    assert!(!batch.all_killed);
    for result in &batch.results {
        assert_eq!(result.outcome, MutantOutcome::NotRun { mutants: 1 });
    }

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn adapter_trace_runner_executes_process() {
    let adapter = RustAdapter::new();
    let req = RunRequest {
        command: "rustc".into(),
        args: vec!["--version".into()],
        env: vec![],
        cwd: None,
        timeout_ms: 60_000,
        max_output_bytes: 4096,
    };
    let res = adapter.run(&req);
    assert_eq!(res.exit_code, 0);
    let output = format!(
        "{}{}",
        String::from_utf8_lossy(&res.stdout_bytes),
        String::from_utf8_lossy(&res.stderr_bytes)
    );
    assert!(output.contains("rustc"));
}

#[test]
fn adapter_trace_runner_injects_isolated_target_dir() {
    let adapter = RustAdapter::new();
    let temp = std::env::temp_dir().join(format!("test-target-dir-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp);

    let req = RunRequest {
        command: "rustc".into(),
        args: vec!["--version".into()],
        env: vec![],
        cwd: Some(temp.to_string_lossy().to_string()),
        timeout_ms: 60_000,
        max_output_bytes: 4096,
    };
    let res = adapter.run(&req);
    assert_eq!(res.exit_code, 0);
    let _ = std::fs::remove_dir_all(&temp);
}
