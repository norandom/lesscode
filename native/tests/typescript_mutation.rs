//! Test suite for TypeScript adverse mutation operators.
//!
//! Governed by:
//! - REQ-TS-3.1, REQ-TS-3.2, REQ-TS-3.3
//! - `.kiro/specs/typescript-deconvolution-adapter/tasks.md` Task 2

use lesscode_core::adapter_traits::*;
use lesscode_core::adapters::typescript::mutation::TypeScriptMutantGenerator;

#[test]
fn applies_severity_inversion_operator() {
    let generator = TypeScriptMutantGenerator::new();
    let source = r#"export const notice = { severity: "warning" };"#;
    let spec = MutationSpec {
        id: "TS_ADV_1".into(),
        location: "test.ts:1".into(),
        kind: MutationKind::MissingFailureState,
        description: "Invert warning to notice".into(),
    };
    let mutated = generator.apply_mutation(source, &spec).unwrap();
    assert!(mutated.contains("\"notice\""));
    assert!(!mutated.contains("\"warning\""));
}

#[test]
fn applies_slugify_drop_operator() {
    let generator = TypeScriptMutantGenerator::new();
    let source = r#"const slug = v.replace(/[^a-z0-9]+/g, "-");"#;
    let spec = MutationSpec {
        id: "TS_ADV_2".into(),
        location: "test.ts:1".into(),
        kind: MutationKind::SilentNone,
        description: "Drop regex sanitization".into(),
    };
    let mutated = generator.apply_mutation(source, &spec).unwrap();
    assert_eq!(mutated, r#"const slug = v;"#);
}

#[test]
fn applies_error_order_inversion_operator() {
    let generator = TypeScriptMutantGenerator::new();
    let source = r#"const blk = i.notices.filter((n) => n.severity === "blocked");"#;
    let spec = MutationSpec {
        id: "TS_ADV_4".into(),
        location: "test.ts:1".into(),
        kind: MutationKind::InvertedErrorOrder,
        description: "Invert error list ordering".into(),
    };
    let mutated = generator.apply_mutation(source, &spec).unwrap();
    assert!(mutated.contains(".reverse()"));
}

#[test]
fn applies_stream_truncation_operator() {
    let generator = TypeScriptMutantGenerator::new();
    let source = "console.log('data');";
    let spec = MutationSpec {
        id: "TS_ADV_3".into(),
        location: "test.ts:1".into(),
        kind: MutationKind::TruncatedStream,
        description: "Simulate stream truncation".into(),
    };
    let mutated = generator.apply_mutation(source, &spec).unwrap();
    assert!(mutated.contains("TRUNCATE_STREAM_BYTES: 16"));
}
