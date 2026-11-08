//! Lean state contract structures and deconvolution outcome evaluation.

#[derive(Clone, Debug, PartialEq)]
pub struct ScopeMetrics {
    pub function_count: usize,
    pub call_edge_count: usize,
    pub write_site_count: usize,
    pub ast_statements: usize,
    pub canonical_lines: usize,
    pub pure_functions: usize,
}

impl ScopeMetrics {
    pub fn pure_function_pct(&self) -> f64 {
        if self.function_count == 0 {
            0.0
        } else {
            (self.pure_functions as f64 / self.function_count as f64) * 100.0
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeconvolutionOutcome {
    pub function_reduction_pct: f64,
    pub call_edge_reduction_pct: f64,
    pub write_site_reduction_pct: f64,
    pub statement_reduction_pct: f64,
    pub canonical_line_reduction_pct: f64,
    pub pure_function_pct: f64,
    pub structurally_consolidated: bool,
    pub compression_met: bool,
}

impl DeconvolutionOutcome {
    pub fn evaluate(
        baseline: &ScopeMetrics,
        candidate: &ScopeMetrics,
        min_stmt_target: f64,
        min_line_target: f64,
    ) -> Self {
        let func_red = if baseline.function_count > 0 {
            ((baseline.function_count as f64 - candidate.function_count as f64)
                / baseline.function_count as f64)
                * 100.0
        } else {
            0.0
        };
        let edge_red = if baseline.call_edge_count > 0 {
            ((baseline.call_edge_count as f64 - candidate.call_edge_count as f64)
                / baseline.call_edge_count as f64)
                * 100.0
        } else {
            0.0
        };
        let write_red = if baseline.write_site_count > 0 {
            ((baseline.write_site_count as f64 - candidate.write_site_count as f64)
                / baseline.write_site_count as f64)
                * 100.0
        } else {
            0.0
        };
        let stmt_red = if baseline.ast_statements > 0 {
            ((baseline.ast_statements as f64 - candidate.ast_statements as f64)
                / baseline.ast_statements as f64)
                * 100.0
        } else {
            0.0
        };
        let line_red = if baseline.canonical_lines > 0 {
            ((baseline.canonical_lines as f64 - candidate.canonical_lines as f64)
                / baseline.canonical_lines as f64)
                * 100.0
        } else {
            0.0
        };

        let structurally_consolidated = func_red > 0.0 || edge_red > 0.0 || write_red > 0.0;
        let compression_met = stmt_red >= min_stmt_target && line_red >= min_line_target;

        Self {
            function_reduction_pct: func_red,
            call_edge_reduction_pct: edge_red,
            write_site_reduction_pct: write_red,
            statement_reduction_pct: stmt_red,
            canonical_line_reduction_pct: line_red,
            pure_function_pct: candidate.pure_function_pct(),
            structurally_consolidated,
            compression_met,
        }
    }
}
