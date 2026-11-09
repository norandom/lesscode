//! Static AST effect analyzer and frame validator for Rust.
//!
//! Governed by:
//! - REQ-RUST-1.1, REQ-RUST-1.2, REQ-RUST-1.3, REQ-RUST-1.4
//! - `.kiro/specs/rust-deconvolution-unification/design.md`

use crate::adapter_traits::{
    AnalysisRequest, AnalysisResult, CoverageGap, StateFrameAnalyzer,
};

/// Static effect and frame analyzer for Rust modules and functions.
pub struct RustEffectAnalyzer;

impl RustEffectAnalyzer {
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
        if trimmed.starts_with("unsafe ") || trimmed.contains("unsafe {") || trimmed.contains("unsafe fn ") {
            gaps.push(CoverageGap {
                reason: "Unsafe code blocks/functions are unsupported in rust-nextest-v1".into(),
                location: Some(format!("{path}:{line_num}")),
            });
        }
        if trimmed.contains("*const ") || trimmed.contains("*mut ") {
            gaps.push(CoverageGap {
                reason: "Raw pointers are unsupported in rust-nextest-v1".into(),
                location: Some(format!("{path}:{line_num}")),
            });
        }
        if trimmed.contains("dyn ") || trimmed.contains("Box<dyn ") {
            gaps.push(CoverageGap {
                reason: "Dynamic trait object dispatch is unsupported in rust-nextest-v1".into(),
                location: Some(format!("{path}:{line_num}")),
            });
        }
        if trimmed.contains("thread::spawn(") || trimmed.contains("std::thread::spawn(") {
            gaps.push(CoverageGap {
                reason: "Unmonitored thread dispatch is unsupported".into(),
                location: Some(format!("{path}:{line_num}")),
            });
        }
        if trimmed.contains("RefCell<") || trimmed.contains("Mutex<") || trimmed.contains("RwLock<") {
            gaps.push(CoverageGap {
                reason: "Interior mutability requires explicit state roots".into(),
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
        if (trimmed.contains(" = ") && !trimmed.starts_with("let ") && !trimmed.starts_with("const "))
            || trimmed.contains(" += ")
            || trimmed.contains(" -= ")
            || trimmed.contains(" *= ")
            || trimmed.contains(".push(")
            || trimmed.contains(".insert(")
            || trimmed.contains(".extend(")
            || trimmed.contains(".clear()")
            || trimmed.contains(".truncate(")
        {
            write_sites.push(format!("{path}:{line_num}"));
        }
    }
}

impl Default for RustEffectAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl StateFrameAnalyzer for RustEffectAnalyzer {
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
