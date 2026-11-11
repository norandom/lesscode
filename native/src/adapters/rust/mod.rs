//! Rust runtime adapter implementation and profile module (`rust-nextest-v1` profile).
//!
//! Governed by:
//! - `.kiro/specs/rust-deconvolution-unification/design.md`
//! - `native/src/adapter_traits.rs`

pub mod metrics;

use crate::adapter_traits::{
    AdverseMutantGenerator, AnalysisRequest, AnalysisResult, MutationBatch, MutationRequest,
    RunRequest, RunResult, SpecimenTraceRunner,
};
use crate::adapters::driver::contract;
use crate::specimen_runner::NativeSpecimenRunner;

/// Main Rust language adapter answering the adapter traits; analysis and mutation come from the driver.
pub struct RustAdapter {
    pub runner: NativeSpecimenRunner,
    pub profile_id: String,
}

impl RustAdapter {
    pub fn new() -> Self {
        Self {
            runner: NativeSpecimenRunner::new(),
            profile_id: "rust-nextest-v1".into(),
        }
    }
}

impl Default for RustAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::adapter_traits::StateFrameAnalyzer for RustAdapter {
    fn analyze(&self, request: &AnalysisRequest) -> AnalysisResult {
        contract::analyze("rust", request)
    }
}

impl AdverseMutantGenerator for RustAdapter {
    fn generate(&self, request: &MutationRequest) -> MutationBatch {
        contract::generate("rust", request)
    }
}

impl SpecimenTraceRunner for RustAdapter {
    fn run(&self, request: &RunRequest) -> RunResult {
        let mut isolated_request = request.clone();
        if !isolated_request
            .env
            .iter()
            .any(|(k, _)| k == "CARGO_TARGET_DIR")
        {
            let target_dir = if let Some(ref cwd) = isolated_request.cwd {
                std::path::Path::new(cwd)
                    .join("target")
                    .join("isolated-specimen")
            } else {
                std::env::temp_dir().join(format!("lc-cargo-target-{}", std::process::id()))
            };
            isolated_request.env.push((
                "CARGO_TARGET_DIR".into(),
                target_dir.to_string_lossy().to_string(),
            ));
        }
        self.runner.run(&isolated_request)
    }
}
