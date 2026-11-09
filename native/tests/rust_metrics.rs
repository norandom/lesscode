//! Test suite for Rust scope metrics and density evaluation.
//!
//! Governed by:
//! - REQ-RUST-5.1, REQ-RUST-5.2
//! - `.kiro/specs/rust-deconvolution-unification/tasks.md` Task 3

use lesscode_core::adapters::rust::metrics::*;
use lesscode_core::density::ScopeMetrics;
use std::path::Path;

#[test]
fn counts_canonical_and_physical_lines() {
    let source = r#"
// Header comment
/* Multi-line
 * comment */
pub fn add(a: i32, b: i32) -> i32 {
    let mut sum = 0;

    sum = a + b;
    sum
}
"#;
    assert_eq!(count_physical_lines(source), 10);
    assert_eq!(count_canonical_lines(source), 5);
}

#[test]
fn counts_logical_statements() {
    let source = r#"
pub fn compute(flag: bool) -> i32 {
    let mut x = 1;
    if flag {
        x += 1;
    }
    match x {
        1 => 10,
        _ => 20,
    }
}
"#;
    let stmts = count_ast_statements(source);
    // let mut x = 1; (1)
    // if flag { (1)
    // x += 1; (1)
    // match x { (1)
    // 1 => 10, (1)
    // _ => 20, (1)
    // Total = 6
    assert_eq!(stmts, 6);
}

#[test]
fn qualifies_substantial_whole_responsibility_compression() {
    let baseline = ScopeMetrics {
        function_count: 6,
        call_edge_count: 12,
        write_site_count: 8,
        ast_statements: 83,
        canonical_lines: 248,
        pure_functions: 2,
    };
    let candidate = ScopeMetrics {
        function_count: 6,
        call_edge_count: 12,
        write_site_count: 3,
        ast_statements: 70,
        canonical_lines: 148,
        pure_functions: 5,
    };

    let report = evaluate_rust_compression(&baseline, &candidate);
    assert!(report.compression_met);
    assert_eq!(report.disposition, "QUALIFIED");
    assert!(report.line_reduction_pct >= 20.0);
    assert!(report.stmt_reduction_pct >= 10.0);
}

#[test]
fn rejects_marginal_reduction_under_50_lines() {
    let baseline = ScopeMetrics {
        function_count: 6,
        call_edge_count: 12,
        write_site_count: 8,
        ast_statements: 83,
        canonical_lines: 248,
        pure_functions: 2,
    };
    let candidate = ScopeMetrics {
        function_count: 6,
        call_edge_count: 12,
        write_site_count: 5,
        ast_statements: 75,
        canonical_lines: 220, // -28 lines only
        pure_functions: 3,
    };

    let report = evaluate_rust_compression(&baseline, &candidate);
    assert!(!report.compression_met);
    assert_eq!(report.disposition, "REJECTED");
}

#[test]
fn measures_specimen_lifecycle_reduction() {
    let baseline_path = Path::new("adapters/rust/baseline_lifecycle.rs");
    let candidate_path = Path::new("adapters/rust/candidate_lifecycle.rs");

    if baseline_path.exists() && candidate_path.exists() {
        let baseline_text = std::fs::read_to_string(baseline_path).unwrap();
        let candidate_text = std::fs::read_to_string(candidate_path).unwrap();

        let base_lines = count_canonical_lines(&baseline_text);
        let cand_lines = count_canonical_lines(&candidate_text);
        assert_eq!(base_lines, 248);
        assert_eq!(cand_lines, 148);

        let base_stmts = count_ast_statements(&baseline_text);
        let cand_stmts = count_ast_statements(&candidate_text);
        assert_eq!(base_stmts, 83);
        assert_eq!(cand_stmts, 70);

        let base_metrics = extract_scope_metrics(&baseline_text);
        let cand_metrics = extract_scope_metrics(&candidate_text);

        let report = evaluate_rust_compression(&base_metrics, &cand_metrics);
        assert!(report.compression_met);
        assert_eq!(report.disposition, "QUALIFIED");
    }
}
