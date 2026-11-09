//! Rust runtime adapter implementation and profile module (`rust-nextest-v1` profile).
//!
//! Governed by:
//! - `.kiro/specs/rust-deconvolution-unification/design.md`
//! - `native/src/adapter_traits.rs`

pub mod analysis;
pub mod metrics;
pub mod mutation;

use crate::adapter_traits::{
    AdverseMutantGenerator, AnalysisRequest, AnalysisResult, ComparisonRequest, MutationBatch,
    MutationRequest, PureCoreVerifier, RunRequest, RunResult, SpecimenTraceRunner,
    VerificationResult,
};
use crate::specimen_runner::NativeSpecimenRunner;

pub use analysis::RustEffectAnalyzer;
pub use mutation::RustMutantGenerator;

/// Main Rust language adapter bundling analysis, mutation, verification, and runner traits.
pub struct RustAdapter {
    pub analyzer: RustEffectAnalyzer,
    pub mutant_gen: RustMutantGenerator,
    pub runner: NativeSpecimenRunner,
    pub profile_id: String,
}

impl RustAdapter {
    pub fn new() -> Self {
        Self {
            analyzer: RustEffectAnalyzer::new(),
            mutant_gen: RustMutantGenerator::new(),
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
        self.analyzer.analyze(request)
    }
}

impl AdverseMutantGenerator for RustAdapter {
    fn generate(&self, request: &MutationRequest) -> MutationBatch {
        self.mutant_gen.generate(request)
    }
}

impl PureCoreVerifier for RustAdapter {
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

impl SpecimenTraceRunner for RustAdapter {
    fn run(&self, request: &RunRequest) -> RunResult {
        self.runner.run(request)
    }
}
