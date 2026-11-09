//! Whole-responsibility density scorer and compression outcome evaluation.
//!
//! Governed by:
//! - REQ-NAT-2.1 through REQ-NAT-2.5
//! - `.kiro/specs/native-deconvolution-closure/design.md`

use serde::{Deserialize, Serialize};

pub use crate::state_contract::{DeconvolutionOutcome, ScopeMetrics};

/// Count canonical (non-empty, non-comment) lines from source text.
pub fn count_canonical_lines(source: &str) -> usize {
    source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with("//"))
        .count()
}

/// Structured summary of deconvolution evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DensityEvaluationReport {
    pub baseline_lines: usize,
    pub candidate_lines: usize,
    pub line_reduction_pct: f64,
    pub baseline_stmts: usize,
    pub candidate_stmts: usize,
    pub stmt_reduction_pct: f64,
    pub pure_function_pct: f64,
    pub compression_met: bool,
    pub structurally_consolidated: bool,
    pub disposition: String,
}

/// Evaluates scope compression against target thresholds (default: 20% lines, 10% stmts).
pub fn evaluate_density_gate(
    baseline: &ScopeMetrics,
    candidate: &ScopeMetrics,
    min_stmt_red: f64,
    min_line_red: f64,
) -> DensityEvaluationReport {
    let outcome = DeconvolutionOutcome::evaluate(baseline, candidate, min_stmt_red, min_line_red);

    let disposition = if outcome.compression_met && outcome.structurally_consolidated {
        "QUALIFIED".to_string()
    } else {
        "REJECTED".to_string()
    };

    DensityEvaluationReport {
        baseline_lines: baseline.canonical_lines,
        candidate_lines: candidate.canonical_lines,
        line_reduction_pct: outcome.canonical_line_reduction_pct,
        baseline_stmts: baseline.ast_statements,
        candidate_stmts: candidate.ast_statements,
        stmt_reduction_pct: outcome.statement_reduction_pct,
        pure_function_pct: outcome.pure_function_pct,
        compression_met: outcome.compression_met,
        structurally_consolidated: outcome.structurally_consolidated,
        disposition,
    }
}
