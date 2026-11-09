//! Static AST effect analyzer and frame validator for TypeScript.
//!
//! Governed by:
//! - REQ-TS-1.1, REQ-TS-1.2, REQ-TS-1.3, REQ-TS-1.4
//! - `.kiro/specs/typescript-deconvolution-adapter/design.md`

use crate::adapter_traits::{
    AnalysisRequest, AnalysisResult, CoverageGap, StateFrameAnalyzer,
};

/// Static effect and frame analyzer for TypeScript / JavaScript ASTs.
pub struct TypeScriptEffectAnalyzer;

impl TypeScriptEffectAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Scans source text for write sites and unsupported dynamic constructs.
    pub fn scan_source(&self, path: &str, source: &str) -> (Vec<String>, Vec<CoverageGap>) {
        let mut write_sites = Vec::new();
        let mut gaps = Vec::new();

        for (line_idx, line) in source.lines().enumerate() {
            let line_num = line_idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                continue;
            }

            self.check_dynamic_gaps(trimmed, path, line_num, &mut gaps);
            self.check_write_sites(trimmed, path, line_num, &mut write_sites);
        }

        (write_sites, gaps)
    }

    fn check_dynamic_gaps(
        &self,
        trimmed: &str,
        path: &str,
        line_num: usize,
        gaps: &mut Vec<CoverageGap>,
    ) {
        if trimmed.contains("eval(") {
            gaps.push(CoverageGap {
                reason: "Dynamic code execution via eval() is unsupported".into(),
                location: Some(format!("{path}:{line_num}")),
            });
        }
        if trimmed.contains("new Proxy(") {
            gaps.push(CoverageGap {
                reason: "Dynamic proxy objects are unsupported".into(),
                location: Some(format!("{path}:{line_num}")),
            });
        }
        if trimmed.contains("import(") {
            gaps.push(CoverageGap {
                reason: "Dynamic asynchronous module imports are unsupported".into(),
                location: Some(format!("{path}:{line_num}")),
            });
        }
        if trimmed.contains("new Worker(") {
            gaps.push(CoverageGap {
                reason: "Multi-threaded Web Workers are unsupported".into(),
                location: Some(format!("{path}:{line_num}")),
            });
        }
    }

    fn check_write_sites(
        &self,
        trimmed: &str,
        path: &str,
        line_num: usize,
        write_sites: &mut Vec<String>,
    ) {
        if trimmed.contains(" = ")
            && !trimmed.starts_with("const ")
            && !trimmed.starts_with("let ")
            && !trimmed.starts_with("readonly ")
        {
            write_sites.push(format!("{path}:{line_num}"));
        } else if trimmed.contains(".push(")
            || trimmed.contains(".splice(")
            || trimmed.contains("delete ")
        {
            write_sites.push(format!("{path}:{line_num}"));
        }
    }
}

impl Default for TypeScriptEffectAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl StateFrameAnalyzer for TypeScriptEffectAnalyzer {
    fn analyze(&self, request: &AnalysisRequest) -> AnalysisResult {
        let mut all_write_sites = Vec::new();
        let mut unverified_gaps = Vec::new();
        let mut violations = Vec::new();

        for path in &request.source_paths {
            if let Ok(source) = std::fs::read_to_string(path) {
                let (sites, gaps) = self.scan_source(path, &source);
                for site in sites {
                    let file_only = site.split(':').next().unwrap_or(&site);
                    let allowed = request
                        .frame_spec
                        .allowed_write_sites
                        .iter()
                        .any(|a| a == file_only || site.starts_with(a));
                    if !allowed {
                        violations.push(format!("Unauthorized write site: {site}"));
                    }
                    all_write_sites.push(site);
                }
                unverified_gaps.extend(gaps);
            }
        }

        let is_valid = violations.is_empty() && unverified_gaps.is_empty();

        AnalysisResult {
            write_sites: all_write_sites,
            permitted_writes: request.frame_spec.allowed_write_sites.clone(),
            violations,
            unverified_gaps,
            is_valid,
        }
    }
}
