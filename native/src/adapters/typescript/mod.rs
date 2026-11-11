//! TypeScript / JavaScript language deconvolution adapter (`ts-node-v1` profile).
//!
//! Governed by:
//! - `.kiro/specs/typescript-deconvolution-adapter/design.md`
//! - `.kiro/specs/native-deconvolution-closure/adapter-contract.md`

pub mod metrics;

use crate::adapter_traits::{
    AdverseMutantGenerator, AnalysisRequest, AnalysisResult, MutationBatch, MutationRequest,
    RunRequest, RunResult, SpecimenTraceRunner,
};
use crate::adapters::driver::contract;
use crate::specimen_runner::NativeSpecimenRunner;

/// Main TypeScript language adapter answering the adapter traits; analysis and mutation come from the driver.
pub struct TypeScriptAdapter {
    pub runner: NativeSpecimenRunner,
    pub profile_id: String,
}

impl TypeScriptAdapter {
    pub fn new() -> Self {
        Self {
            runner: NativeSpecimenRunner::new(),
            profile_id: "ts-node-v1".into(),
        }
    }
}

impl Default for TypeScriptAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::adapter_traits::StateFrameAnalyzer for TypeScriptAdapter {
    fn analyze(&self, request: &AnalysisRequest) -> AnalysisResult {
        contract::analyze("typescript", request)
    }
}

impl AdverseMutantGenerator for TypeScriptAdapter {
    fn generate(&self, request: &MutationRequest) -> MutationBatch {
        contract::generate("typescript", request)
    }
}

impl SpecimenTraceRunner for TypeScriptAdapter {
    fn run(&self, request: &RunRequest) -> RunResult {
        self.runner.run(request)
    }
}
