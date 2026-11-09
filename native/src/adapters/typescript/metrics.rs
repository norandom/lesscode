//! TypeScript source code metric extraction and whole-scope compression scorer.
//!
//! Governed by:
//! - REQ-TS-5.1, REQ-TS-5.2
//! - `.kiro/specs/typescript-deconvolution-adapter/design.md`

use crate::density::{evaluate_density_gate, DensityEvaluationReport, ScopeMetrics};

/// Counts canonical (non-empty, non-comment) lines in TypeScript source.
pub fn count_canonical_lines(source: &str) -> usize {
    source
        .lines()
        .map(str::trim)
        .filter(|line| {
            !line.is_empty()
                && !line.starts_with("//")
                && !line.starts_with("/*")
                && !line.starts_with('*')
        })
        .count()
}

/// Counts total physical lines in TypeScript source.
pub fn count_physical_lines(source: &str) -> usize {
    source.lines().count()
}

/// Counts logical AST statements in TypeScript source.
pub fn count_ast_statements(source: &str) -> usize {
    let mut count = 0;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with("//")
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
        {
            continue;
        }

        let is_statement_start = trimmed.starts_with("export ")
            || trimmed.starts_with("const ")
            || trimmed.starts_with("let ")
            || trimmed.starts_with("function ")
            || trimmed.starts_with("return ")
            || trimmed.starts_with("if ")
            || trimmed.starts_with("for ")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("throw ")
            || trimmed.starts_with("interface ")
            || trimmed.starts_with("type ");

        if is_statement_start {
            count += 1;
        }
    }
    count
}

/// Extracts scope-level metrics from TypeScript source text.
pub fn extract_scope_metrics(source: &str) -> ScopeMetrics {
    let canonical_lines = count_canonical_lines(source);
    let ast_statements = count_ast_statements(source);

    let function_count = source
        .lines()
        .map(str::trim)
        .filter(|l| l.contains("function ") || l.contains("=>"))
        .count();

    let write_site_count = source
        .lines()
        .map(str::trim)
        .filter(|l| l.contains(" = ") && !l.starts_with("const ") && !l.starts_with("let "))
        .count();

    ScopeMetrics {
        function_count,
        call_edge_count: function_count * 2,
        write_site_count,
        ast_statements,
        canonical_lines,
        pure_functions: if write_site_count == 0 { function_count } else { 0 },
    }
}

/// Evaluates whole-responsibility compression against mandatory TypeScript thresholds:
/// at least 20% canonical line reduction AND 10% statement reduction,
/// and at least 50 canonical lines AND 10 statements removed.
pub fn evaluate_typescript_compression(
    baseline: &ScopeMetrics,
    candidate: &ScopeMetrics,
) -> DensityEvaluationReport {
    let mut report = evaluate_density_gate(baseline, candidate, 10.0, 20.0);

    let abs_lines_removed = baseline.canonical_lines.saturating_sub(candidate.canonical_lines);
    let abs_stmts_removed = baseline.ast_statements.saturating_sub(candidate.ast_statements);

    let absolute_target_met = abs_lines_removed >= 50 && abs_stmts_removed >= 10;
    if !absolute_target_met {
        report.compression_met = false;
        report.disposition = "REJECTED".into();
    }

    report
}
