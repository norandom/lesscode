//! Test suite for TypeScript scope metrics and density evaluation.
//!
//! Governed by:
//! - REQ-TS-5.1, REQ-TS-5.2
//! - `.kiro/specs/typescript-deconvolution-adapter/tasks.md` Task 3

use lesscode_core::adapters::typescript::metrics::*;
use lesscode_core::density::ScopeMetrics;

#[test]
fn counts_canonical_and_physical_lines() {
    let source = r#"
// Header comment
/* Multi-line
 * comment */
export function test(): number {
    const x = 1;

    return x + 1;
}
"#;
    assert_eq!(count_physical_lines(source), 9);
    assert_eq!(count_canonical_lines(source), 4);
}

#[test]
fn counts_logical_statements() {
    let source = r#"
export const a = 1;
let b = 2;
if (a > 0) {
    return b;
}
"#;
    assert_eq!(count_ast_statements(source), 4);
}

#[test]
fn qualifies_substantial_whole_responsibility_compression() {
    let baseline = ScopeMetrics {
        function_count: 10,
        call_edge_count: 20,
        write_site_count: 5,
        ast_statements: 52,
        canonical_lines: 201,
        pure_functions: 2,
    };
    let candidate = ScopeMetrics {
        function_count: 8,
        call_edge_count: 12,
        write_site_count: 0,
        ast_statements: 31,
        canonical_lines: 113,
        pure_functions: 8,
    };

    let report = evaluate_typescript_compression(&baseline, &candidate);
    assert!(report.compression_met);
    assert_eq!(report.disposition, "QUALIFIED");
    assert!(report.line_reduction_pct >= 20.0);
    assert!(report.stmt_reduction_pct >= 10.0);
}

#[test]
fn rejects_marginal_reduction_under_50_lines() {
    // Like historical TS01: only 19 lines removed
    let baseline = ScopeMetrics {
        function_count: 13,
        call_edge_count: 26,
        write_site_count: 5,
        ast_statements: 93,
        canonical_lines: 380,
        pure_functions: 5,
    };
    let candidate = ScopeMetrics {
        function_count: 13,
        call_edge_count: 26,
        write_site_count: 4,
        ast_statements: 82,
        canonical_lines: 361, // -19 lines
        pure_functions: 6,
    };

    let report = evaluate_typescript_compression(&baseline, &candidate);
    assert!(!report.compression_met);
    assert_eq!(report.disposition, "REJECTED");
}
