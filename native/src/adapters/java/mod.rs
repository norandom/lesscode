//! Java runtime adapter implementation and profile module (`java-junit-v1` profile).
//!
//! Governed by:
//! - `.kiro/specs/java-deconvolution-unification/design.md`
//! - `native/src/adapter_traits.rs`

pub mod metrics;

use crate::adapter_traits::{
    AdverseMutantGenerator, AnalysisRequest, AnalysisResult, MutationBatch, MutationRequest,
    RunRequest, RunResult, SpecimenTraceRunner,
};
use crate::adapters::driver::contract;
use crate::specimen_runner::NativeSpecimenRunner;

/// Main Java language adapter answering the adapter traits; analysis and mutation come from the driver.
pub struct JavaAdapter {
    pub runner: NativeSpecimenRunner,
    pub profile_id: String,
}

impl JavaAdapter {
    pub fn new() -> Self {
        Self {
            runner: NativeSpecimenRunner::new(),
            profile_id: "java-junit-v1".into(),
        }
    }
}

impl Default for JavaAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::adapter_traits::StateFrameAnalyzer for JavaAdapter {
    fn analyze(&self, request: &AnalysisRequest) -> AnalysisResult {
        contract::analyze("java", request)
    }
}

impl AdverseMutantGenerator for JavaAdapter {
    fn generate(&self, request: &MutationRequest) -> MutationBatch {
        contract::generate("java", request)
    }
}

impl SpecimenTraceRunner for JavaAdapter {
    fn run(&self, request: &RunRequest) -> RunResult {
        self.runner.run(request)
    }
}
