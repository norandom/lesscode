//! Test suite for Java runtime adapter and trait implementations.
//!
//! Governed by:
//! - REQ-JAVA-2.1 through REQ-JAVA-2.4, REQ-JAVA-4.1 through REQ-JAVA-4.4
//! - `.kiro/specs/java-deconvolution-unification/tasks.md` Task 4

use lesscode_core::adapter_traits::*;
use lesscode_core::adapters::java::JavaAdapter;

#[test]
fn java_adapter_initializes_with_profile() {
    let adapter = JavaAdapter::new();
    assert_eq!(adapter.profile_id, "java-junit-v1");
}

#[test]
fn adapter_state_frame_analyzer_integration() {
    let adapter = JavaAdapter::new();
    let req = AnalysisRequest {
        scope_name: "test".into(),
        source_paths: vec![],
        frame_spec: FrameSpec {
            allowed_write_sites: vec!["CSVPrinter.java".into()],
            permitted_shell_locations: vec![],
            failure_invariants: vec![],
        },
        effect_profile: "java-junit-v1".into(),
    };
    let res = adapter.analyze(&req);
    assert!(res.is_valid);
}

#[test]
fn adapter_adverse_mutant_generator_integration() {
    let adapter = JavaAdapter::new();
    let temp_dir = std::env::temp_dir().join(format!("java-adapter-test-{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let file_path = temp_dir.join("Target.java");
    std::fs::write(
        &file_path,
        "class Target { void mutate() { newRecord = false; } }",
    )
    .unwrap();

    let req = MutationRequest {
        target_path: file_path.to_str().unwrap().into(),
        controls: vec![MutationSpec {
            id: "JAVA_ADV_1".into(),
            location: "Target.java:1".into(),
            kind: MutationKind::SilentNone,
            description: "State mutation omission".into(),
        }],
    };

    let batch = adapter.generate(&req);
    // The driver finds the dropped write; no test is run, so it is not killed.
    assert_eq!(batch.denominator, 1);
    assert_eq!(batch.killed_count, 0);
    assert!(!batch.all_killed);
    assert_eq!(
        batch.results[0].outcome,
        MutantOutcome::NotRun { mutants: 1 }
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn adapter_trace_runner_executes_process() {
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
        max_output_bytes: 4096,
    };
    let res = adapter.run(&req);
    assert_eq!(res.exit_code, 0);
    let output = format!(
        "{}{}",
        String::from_utf8_lossy(&res.stdout_bytes),
        String::from_utf8_lossy(&res.stderr_bytes)
    );
    assert!(output.contains("version"));
}
