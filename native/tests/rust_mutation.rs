//! Integration tests for Rust adverse mutation operators.

use lesscode_core::adapter_traits::{
    AdverseMutantGenerator, MutantOutcome, MutationKind, MutationRequest, MutationSpec,
};
use lesscode_core::adapters::rust::RustMutantGenerator;

#[test]
fn applies_deadline_kill_omission_operator() {
    let generator = RustMutantGenerator::new();
    let source = r#"
if Instant::now() >= deadline {
    let _ = Command::new("taskkill").args(["/F", "/T", "/PID"]).output();
    return 124;
}
"#;
    let spec = MutationSpec {
        id: "RUST_ADV_1".into(),
        location: "lifecycle.rs:3".into(),
        kind: MutationKind::MissingFailureState,
        description: "Omit taskkill deadline kill".into(),
    };
    let mutated = generator.apply_mutation(source, &spec).unwrap();
    assert!(!mutated.contains("\"taskkill\""));
    assert!(mutated.contains("taskkill_omitted"));
}

#[test]
fn applies_inverted_verdict_condition_operator() {
    let generator = RustMutantGenerator::new();
    let source = "if arm_exit != 0 { reasons.push(format!(\"arm exit {arm_exit}\")); }";
    let spec = MutationSpec {
        id: "RUST_ADV_2".into(),
        location: "lifecycle.rs:5".into(),
        kind: MutationKind::InvertedCondition,
        description: "Invert verdict refusal condition".into(),
    };
    let mutated = generator.apply_mutation(source, &spec).unwrap();
    assert!(mutated.contains("arm_exit == 0"));
}

#[test]
fn applies_inverted_error_order_operator() {
    let generator = RustMutantGenerator::new();
    let source = "if level2_ops > 0 && !matches!(query_evidence, Some(p) if p.is_file()) {";
    let spec = MutationSpec {
        id: "RUST_ADV_3".into(),
        location: "lifecycle.rs:8".into(),
        kind: MutationKind::InvertedErrorOrder,
        description: "Bypass level2 differences check".into(),
    };
    let mutated = generator.apply_mutation(source, &spec).unwrap();
    assert!(mutated.contains("false && level2_ops > 0"));
}

#[test]
fn applies_dropped_environment_isolation_operator() {
    let generator = RustMutantGenerator::new();
    let source = "(\"PYTHONPATH\", pythonpath.as_str())";
    let spec = MutationSpec {
        id: "RUST_ADV_4".into(),
        location: "lifecycle.rs:12".into(),
        kind: MutationKind::SilentNone,
        description: "Drop PYTHONPATH from arm_env".into(),
    };
    let mutated = generator.apply_mutation(source, &spec).unwrap();
    assert!(!mutated.contains("\"PYTHONPATH\""));
    assert!(mutated.contains("\"PYTHONPATH_DROPPED\""));
}

#[test]
fn generates_batch_with_denominator_accounting() {
    let generator = RustMutantGenerator::new();
    let temp_dir = std::env::temp_dir().join(format!("rust-mutation-test-{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let file_path = temp_dir.join("Target.rs");
    std::fs::write(
        &file_path,
        "pub fn check(arm_exit: i32) { if arm_exit != 0 { panic!(); } }",
    )
    .unwrap();

    let request = MutationRequest {
        target_path: file_path.to_str().unwrap().into(),
        controls: vec![
            MutationSpec {
                id: "M1".into(),
                location: "Target.rs:1".into(),
                kind: MutationKind::InvertedCondition,
                description: "Inverted condition".into(),
            },
            MutationSpec {
                id: "M2".into(),
                location: "Target.rs:1".into(),
                kind: MutationKind::BoundaryOffByOne,
                description: "Missing target".into(),
            },
        ],
    };

    let batch = generator.generate(&request);
    assert_eq!(batch.denominator, 2);
    assert_eq!(batch.killed_count, 1);
    assert!(!batch.all_killed);
    assert!(matches!(
        batch.results[0].outcome,
        MutantOutcome::Killed { .. }
    ));
    assert!(matches!(
        batch.results[1].outcome,
        MutantOutcome::Invalid { .. }
    ));

    let _ = std::fs::remove_dir_all(&temp_dir);
}
