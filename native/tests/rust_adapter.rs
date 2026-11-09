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
fn adapter_pure_verifier_handles_finite_domains() {
    let adapter = RustAdapter::new();
    let domain = DomainSpec {
        param_names: vec![
            "exit_code".into(),
            "is_fresh".into(),
            "l2_divergences".into(),
            "arm_env_set".into(),
        ],
        finite_values: vec![
            (
                "exit_code".into(),
                vec![
                    TaggedValue::Int64(0),
                    TaggedValue::Int64(1),
                    TaggedValue::Int64(2),
                    TaggedValue::Int64(137),
                    TaggedValue::Int64(255),
                ],
            ),
            (
                "is_fresh".into(),
                vec![TaggedValue::Bool(true), TaggedValue::Bool(false)],
            ),
            (
                "l2_divergences".into(),
                vec![
                    TaggedValue::Int64(0),
                    TaggedValue::Int64(1),
                    TaggedValue::Int64(2),
                    TaggedValue::Int64(5),
                    TaggedValue::Int64(10),
                ],
            ),
            (
                "arm_env_set".into(),
                vec![TaggedValue::Bool(true), TaggedValue::Bool(false)],
            ),
        ],
        sample_limit: None,
    };
    let req = ComparisonRequest {
        entrypoint_baseline: "baseline_lifecycle.rs:verdict".into(),
        entrypoint_candidate: "candidate_lifecycle.rs:verdict".into(),
        domain,
        timeout_ms: 5000,
    };

    let res = adapter.compare(&req);
    match res {
        VerificationResult::Verified { evidence, domain } => {
            assert!(evidence.contains("100 cases verified"));
            assert_eq!(domain, "exit_code, is_fresh, l2_divergences, arm_env_set");
        }
        _ => panic!("Expected Verified outcome"),
    }
}

#[test]
fn adapter_pure_verifier_rejects_empty_domain() {
    let adapter = RustAdapter::new();
    let domain = DomainSpec {
        param_names: vec![],
        finite_values: vec![],
        sample_limit: None,
    };
    let req = ComparisonRequest {
        entrypoint_baseline: "baseline_lifecycle.rs:verdict".into(),
        entrypoint_candidate: "candidate_lifecycle.rs:verdict".into(),
        domain,
        timeout_ms: 1000,
    };

    let res = adapter.compare(&req);
    match res {
        VerificationResult::Unverified { reason } => {
            assert!(reason.contains("Empty finite domain"));
        }
        _ => panic!("Expected Unverified outcome"),
    }
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
                location: "Target.rs:2".into(),
                kind: MutationKind::InvertedCondition,
                description: "Inverted refusal condition in verdict".into(),
            },
            MutationSpec {
                id: "RUST_ADV_3".into(),
                location: "Target.rs:5".into(),
                kind: MutationKind::InvertedErrorOrder,
                description: "Bypassed level2 differences check".into(),
            },
        ],
    };

    let batch = adapter.generate(&req);
    assert_eq!(batch.denominator, 2);
    assert_eq!(batch.killed_count, 2);
    assert!(batch.all_killed);

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
        timeout_ms: 5000,
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
        timeout_ms: 5000,
        max_output_bytes: 4096,
    };
    let res = adapter.run(&req);
    assert_eq!(res.exit_code, 0);
    let _ = std::fs::remove_dir_all(&temp);
}
