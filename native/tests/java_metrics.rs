//! Test suite for Java scope metrics and density evaluation.
//!
//! Governed by:
//! - REQ-JAVA-5.1, REQ-JAVA-5.2
//! - `.kiro/specs/java-deconvolution-unification/tasks.md` Task 3

use std::path::Path;
use lesscode_core::adapters::java::metrics::*;
use lesscode_core::density::ScopeMetrics;

#[test]
fn counts_canonical_and_physical_lines() {
    let source = r#"
// Header comment
/* Multi-line
 * comment */
package org.example;

import java.io.IOException;

public class Test {
    public void run() throws IOException {
        int x = 1;

        System.out.println(x);
    }
}
"#;
    assert_eq!(count_physical_lines(source), 15);
    assert_eq!(count_canonical_lines(source), 8);
}

#[test]
fn counts_logical_statements() {
    let source = r#"
package org.example;

import java.io.Flushable;

public class Test {
    public void compute(boolean flag) {
        int x = 1;
        if (flag) {
            x = 2;
        }
        for (int i = 0; i < 10; i++) {
            System.out.println(i);
        }
        try {
            System.out.flush();
        } catch (Exception e) {
            throw new RuntimeException(e);
        }
        return;
    }
}
"#;
    let stmts = count_ast_statements(source);
    // int x = 1; (1)
    // if (flag) { (1)
    // x = 2; (1)
    // for (int i = 0; i < 10; i++) { (1)
    // System.out.println(i); (1)
    // try { (1)
    // System.out.flush(); (1)
    // catch (Exception e) { (1)
    // throw new RuntimeException(e); (1)
    // return; (1)
    // Total = 10 statements
    assert_eq!(stmts, 10);
}

#[test]
fn qualifies_substantial_whole_responsibility_compression() {
    let baseline = ScopeMetrics {
        function_count: 14,
        call_edge_count: 28,
        write_site_count: 6,
        ast_statements: 112,
        canonical_lines: 221,
        pure_functions: 4,
    };
    let candidate = ScopeMetrics {
        function_count: 14,
        call_edge_count: 28,
        write_site_count: 1,
        ast_statements: 81,
        canonical_lines: 136,
        pure_functions: 12,
    };

    let report = evaluate_java_compression(&baseline, &candidate);
    assert!(report.compression_met);
    assert_eq!(report.disposition, "QUALIFIED");
    assert!(report.line_reduction_pct >= 20.0);
    assert!(report.stmt_reduction_pct >= 10.0);
}

#[test]
fn rejects_marginal_reduction_under_50_lines() {
    let baseline = ScopeMetrics {
        function_count: 14,
        call_edge_count: 28,
        write_site_count: 6,
        ast_statements: 112,
        canonical_lines: 221,
        pure_functions: 4,
    };
    let candidate = ScopeMetrics {
        function_count: 14,
        call_edge_count: 28,
        write_site_count: 4,
        ast_statements: 100,
        canonical_lines: 195, // only -26 lines
        pure_functions: 6,
    };

    let report = evaluate_java_compression(&baseline, &candidate);
    assert!(!report.compression_met);
    assert_eq!(report.disposition, "REJECTED");
}

#[test]
fn measures_commons_csv_specimen_reduction() {
    let baseline_path = Path::new("C:/Users/mariu/Source/commons-csv/src/main/java/org/apache/commons/csv/CSVPrinter.java");
    let candidate_path = Path::new("native/adapters/java/candidate_CSVPrinter.java");

    if baseline_path.exists() && candidate_path.exists() {
        let baseline_text = std::fs::read_to_string(baseline_path).unwrap();
        let candidate_text = std::fs::read_to_string(candidate_path).unwrap();

        let base_lines = count_canonical_lines(&baseline_text);
        let cand_lines = count_canonical_lines(&candidate_text);
        assert_eq!(base_lines, 221);
        assert_eq!(cand_lines, 136);

        let base_stmts = count_ast_statements(&baseline_text);
        let cand_stmts = count_ast_statements(&candidate_text);
        assert_eq!(base_stmts, 112);
        assert_eq!(cand_stmts, 81);

        let base_metrics = extract_scope_metrics(&baseline_text);
        let cand_metrics = extract_scope_metrics(&candidate_text);

        let report = evaluate_java_compression(&base_metrics, &cand_metrics);
        assert!(report.compression_met);
        assert_eq!(report.disposition, "QUALIFIED");
    }
}
