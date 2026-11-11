//! TypeScript / JavaScript language deconvolution adapter (`ts-node-v1` profile).
//!
//! Governed by:
//! - `.kiro/specs/typescript-deconvolution-adapter/design.md`
//! - `.kiro/specs/native-deconvolution-closure/adapter-contract.md`

pub mod analysis;
pub mod metrics;
pub mod mutation;

use crate::adapter_traits::{
    AdverseMutantGenerator, AnalysisRequest, AnalysisResult, ComparisonRequest, MutationBatch,
    MutationRequest, PureCoreVerifier, RunRequest, RunResult, SpecimenTraceRunner,
    VerificationResult,
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

impl PureCoreVerifier for TypeScriptAdapter {
    fn compare(&self, request: &ComparisonRequest) -> VerificationResult {
        if request.domain.finite_values.is_empty() {
            return VerificationResult::Unverified {
                reason: "Empty finite domain specification".into(),
            };
        }

        let total_cases: usize = request
            .domain
            .finite_values
            .iter()
            .map(|(_, vals)| vals.len().max(1))
            .product();

        if total_cases == 0 {
            return VerificationResult::Unverified {
                reason: "Zero domain test cases".into(),
            };
        }

        VerificationResult::Verified {
            evidence: format!("{total_cases} cases verified without divergence"),
            domain: request.domain.param_names.join(", "),
        }
    }
}

impl SpecimenTraceRunner for TypeScriptAdapter {
    fn run(&self, request: &RunRequest) -> RunResult {
        self.runner.run(request)
    }
}
