//! Integration tests for Java adverse mutation operators.

use lesscode_core::adapter_traits::{
    AdverseMutantGenerator, MutantOutcome, MutationKind, MutationRequest, MutationSpec,
};
use lesscode_core::adapters::java::JavaMutantGenerator;

#[test]
fn applies_state_mutation_omission_operator() {
    let generator = JavaMutantGenerator::new();
    let source = "void printRaw(Object v) { format.print(v); newRecord = false; }";
    let spec = MutationSpec {
        id: "JAVA_ADV_1".into(),
        location: "CSVPrinter.java:1".into(),
        kind: MutationKind::MissingFailureState,
        description: "Omit state mutation".into(),
    };
    let mutated = generator.apply_mutation(source, &spec).unwrap();
    assert!(!mutated.contains("newRecord = false;"));
    assert!(mutated.contains("// newRecord = false omitted"));
}

#[test]
fn applies_dropped_stream_flush_operator() {
    let generator = JavaMutantGenerator::new();
    let source = "void flush() { if (appendable instanceof Flushable) ((Flushable) appendable).flush(); }";
    let spec = MutationSpec {
        id: "JAVA_ADV_4".into(),
        location: "CSVPrinter.java:2".into(),
        kind: MutationKind::SilentNone,
        description: "Drop stream flush".into(),
    };
    let mutated = generator.apply_mutation(source, &spec).unwrap();
    assert!(!mutated.contains("((Flushable) appendable).flush();"));
    assert!(mutated.contains("// appendable flush omitted"));
}

#[test]
fn applies_inverted_error_order_operator() {
    let generator = JavaMutantGenerator::new();
    let source = "void printRaw(Object v) { format.print(value, appendable, newRecord); }";
    let spec = MutationSpec {
        id: "JAVA_ADV_3".into(),
        location: "CSVPrinter.java:3".into(),
        kind: MutationKind::InvertedErrorOrder,
        description: "Invert delimiter sequencing".into(),
    };
    let mutated = generator.apply_mutation(source, &spec).unwrap();
    assert!(mutated.contains("format.print(value, appendable, true);"));
}

#[test]
fn applies_stream_truncation_operator() {
    let generator = JavaMutantGenerator::new();
    let source = "public void println() { format.println(appendable); }";
    let spec = MutationSpec {
        id: "TS_TRUNC".into(),
        location: "CSVPrinter.java:4".into(),
        kind: MutationKind::TruncatedStream,
        description: "Truncate stream".into(),
    };
    let mutated = generator.apply_mutation(source, &spec).unwrap();
    assert!(mutated.contains("// TRUNCATE_STREAM_BYTES: 16"));
}

#[test]
fn generates_batch_with_denominator_accounting() {
    let generator = JavaMutantGenerator::new();
    let temp_dir = std::env::temp_dir().join("java_mut_batch");
    std::fs::create_dir_all(&temp_dir).unwrap();
    let file_path = temp_dir.join("Target.java");
    std::fs::write(
        &file_path,
        "class Target { void mutate() { newRecord = false; } }",
    )
    .unwrap();

    let request = MutationRequest {
        target_path: file_path.to_str().unwrap().into(),
        controls: vec![
            MutationSpec {
                id: "M1".into(),
                location: "Target.java:1".into(),
                kind: MutationKind::MissingFailureState,
                description: "Valid mutation".into(),
            },
            MutationSpec {
                id: "M2".into(),
                location: "Target.java:1".into(),
                kind: MutationKind::BoundaryOffByOne,
                description: "Missing target".into(),
            },
        ],
    };

    let batch = generator.generate(&request);
    assert_eq!(batch.denominator, 2);
    assert_eq!(batch.killed_count, 1);
    assert!(!batch.all_killed);
    assert!(matches!(batch.results[0].outcome, MutantOutcome::Killed { .. }));
    assert!(matches!(batch.results[1].outcome, MutantOutcome::Invalid { .. }));

    let _ = std::fs::remove_dir_all(&temp_dir);
}
