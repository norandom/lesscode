//! Java source code metric extraction and whole-scope compression scorer.
//!
//! Governed by:
//! - REQ-JAVA-5.1, REQ-JAVA-5.2
//! - `.kiro/specs/java-deconvolution-unification/design.md`

use crate::density::{evaluate_density_gate, DensityEvaluationReport, ScopeMetrics};

/// Counts canonical (non-empty, non-comment) lines in Java source.
pub fn count_canonical_lines(source: &str) -> usize {
    source
        .lines()
        .map(str::trim)
        .filter(|line| {
            !line.is_empty()
                && !line.starts_with("//")
                && !line.starts_with("/*")
                && !line.starts_with('*')
                && !line.starts_with('#')
        })
        .count()
}

/// Counts total physical lines in Java source.
pub fn count_physical_lines(source: &str) -> usize {
    source.lines().count()
}

/// Counts logical AST statements in Java source per `java-statements-v1`.
pub fn count_ast_statements(source: &str) -> usize {
    let mut in_block_comment = false;
    let mut count = 0;

    for raw_line in source.lines() {
        let mut line = raw_line.trim().to_string();

        if in_block_comment {
            if let Some(pos) = line.find("*/") {
                line = line[pos + 2..].trim().to_string();
                in_block_comment = false;
            } else {
                continue;
            }
        }

        while let Some(start_pos) = line.find("/*") {
            if let Some(end_pos) = line[start_pos + 2..].find("*/") {
                let before = &line[..start_pos];
                let after = &line[start_pos + 2 + end_pos + 2..];
                line = format!("{} {}", before, after).trim().to_string();
            } else {
                line = line[..start_pos].trim().to_string();
                in_block_comment = true;
                break;
            }
        }

        if let Some(pos) = line.find("//") {
            line = line[..pos].trim().to_string();
        }

        if line.is_empty() || line.starts_with("package ") || line.starts_with("import ") {
            continue;
        }

        for kw in &[
            "if (",
            "for (",
            "while (",
            "try {",
            "catch (",
            "finally {",
            "switch (",
            "return",
            "throw ",
        ] {
            if line.contains(kw) {
                count += line.matches(kw).count();
            }
        }

        let is_terminal = line.ends_with(';');
        let excluded = line.contains("return")
            || line.contains("throw ")
            || line.contains("package ")
            || line.contains("import ");

        if is_terminal && !excluded {
            count += 1;
        }
    }

    count
}

/// Extracts scope-level metrics from Java source text.
pub fn extract_scope_metrics(source: &str) -> ScopeMetrics {
    let canonical_lines = count_canonical_lines(source);
    let ast_statements = count_ast_statements(source);

    let function_count = source
        .lines()
        .map(str::trim)
        .filter(|l| {
            (l.starts_with("public ") || l.starts_with("private ") || l.starts_with("protected "))
                && l.contains('(')
                && (l.ends_with('{') || l.ends_with(')'))
        })
        .count();

    let write_site_count = source
        .lines()
        .map(str::trim)
        .filter(|l| {
            l.contains(" = ")
                && !l.starts_with("final ")
                && !l.starts_with("private final ")
                && !l.starts_with("public static final ")
        })
        .count();

    ScopeMetrics {
        function_count,
        call_edge_count: function_count * 2,
        write_site_count,
        ast_statements,
        canonical_lines,
        pure_functions: if write_site_count == 0 {
            function_count
        } else {
            0
        },
    }
}

/// Evaluates whole-responsibility compression against mandatory Java thresholds:
/// at least 20% canonical line reduction AND 10% statement reduction,
/// and at least 50 canonical lines AND 10 statements removed.
pub fn evaluate_java_compression(
    baseline: &ScopeMetrics,
    candidate: &ScopeMetrics,
) -> DensityEvaluationReport {
    let mut report = evaluate_density_gate(baseline, candidate, 10.0, 20.0);

    let abs_lines_removed = baseline
        .canonical_lines
        .saturating_sub(candidate.canonical_lines);
    let abs_stmts_removed = baseline
        .ast_statements
        .saturating_sub(candidate.ast_statements);

    let absolute_target_met = abs_lines_removed >= 50 && abs_stmts_removed >= 10;
    if !absolute_target_met {
        report.compression_met = false;
        report.disposition = "REJECTED".into();
    }

    report
}
