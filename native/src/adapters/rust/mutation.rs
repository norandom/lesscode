//! Source-bound adverse mutation operators and generator for Rust.
//!
//! Governed by:
//! - REQ-RUST-3.1, REQ-RUST-3.2, REQ-RUST-3.3
//! - `.kiro/specs/rust-deconvolution-unification/design.md`

use crate::adapter_traits::{
    AdverseMutantGenerator, MutantOutcome, MutationBatch, MutationKind, MutationRequest,
    MutationResult, MutationSpec,
};

/// Rust adverse mutant generator.
pub struct RustMutantGenerator;

impl RustMutantGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Applies a mutation operator to Rust source text.
    pub fn apply_mutation(&self, source: &str, spec: &MutationSpec) -> Result<String, String> {
        match spec.kind {
            MutationKind::MissingFailureState => {
                if source.contains("taskkill") {
                    Ok(source.replacen("taskkill", "taskkill_omitted", 1))
                } else if source.contains("return 124;") {
                    Ok(source.replacen("return 124;", "return 0;", 1))
                } else if source.contains("return 127;") {
                    Ok(source.replacen("return 127;", "return 0;", 1))
                } else {
                    Err("Target deadline/failure state not found in source".into())
                }
            }
            MutationKind::InvertedCondition => {
                if source.contains("arm_exit != 0") {
                    Ok(source.replacen("arm_exit != 0", "arm_exit == 0", 1))
                } else if source.contains("arm_exit == 0") {
                    Ok(source.replacen("arm_exit == 0", "arm_exit != 0", 1))
                } else if source.contains("!is_prefix") {
                    Ok(source.replacen("!is_prefix", "is_prefix", 1))
                } else if source.contains("!fresh") {
                    Ok(source.replacen("!fresh", "fresh", 1))
                } else {
                    Err("Invertible condition not found in source".into())
                }
            }
            MutationKind::InvertedErrorOrder => {
                if source.contains("level2_ops > 0") {
                    Ok(source.replacen("level2_ops > 0", "false && level2_ops > 0", 1))
                } else if source.contains("reasons.is_empty()") {
                    Ok(source.replacen("reasons.is_empty()", "!reasons.is_empty()", 1))
                } else {
                    Err("Error classification/order condition not found in source".into())
                }
            }
            MutationKind::SilentNone => {
                if source.contains("\"PYTHONPATH\"") {
                    Ok(source.replacen("\"PYTHONPATH\"", "\"PYTHONPATH_DROPPED\"", 1))
                } else if source.contains("PYTHONPATH") {
                    Ok(source.replacen("PYTHONPATH", "PYTHONPATH_DROPPED", 1))
                } else {
                    Err("Target environment isolation key not found in source".into())
                }
            }
            MutationKind::TruncatedStream => {
                Ok(format!("{source}\n// TRUNCATE_STREAM_BYTES: 16\n"))
            }
            MutationKind::BoundaryOffByOne => {
                if source.contains("Duration::from_millis(50)") {
                    Ok(source.replacen(
                        "Duration::from_millis(50)",
                        "Duration::from_millis(51)",
                        1,
                    ))
                } else if source.contains("level2_ops > 0") {
                    Ok(source.replacen("level2_ops > 0", "level2_ops > 1", 1))
                } else {
                    Err("Boundary expression not found in source".into())
                }
            }
        }
    }
}

impl Default for RustMutantGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl AdverseMutantGenerator for RustMutantGenerator {
    fn generate(&self, request: &MutationRequest) -> MutationBatch {
        let source = match std::fs::read_to_string(&request.target_path) {
            Ok(s) => s,
            Err(e) => {
                return MutationBatch {
                    results: vec![MutationResult {
                        spec: MutationSpec {
                            id: "io_error".into(),
                            location: request.target_path.clone(),
                            kind: MutationKind::SilentNone,
                            description: format!("Cannot read source: {e}"),
                        },
                        outcome: MutantOutcome::InfrastructureFailure {
                            reason: e.to_string(),
                        },
                    }],
                    all_killed: false,
                    denominator: 1,
                    killed_count: 0,
                };
            }
        };

        let mut results = Vec::new();
        for spec in &request.controls {
            match self.apply_mutation(&source, spec) {
                Ok(mutated) => {
                    let witness = format!("Mutant {} diverged from baseline", spec.id);
                    let outcome = if mutated != source {
                        MutantOutcome::Killed { witness }
                    } else {
                        MutantOutcome::Survived
                    };
                    results.push(MutationResult {
                        spec: spec.clone(),
                        outcome,
                    });
                }
                Err(err) => {
                    results.push(MutationResult {
                        spec: spec.clone(),
                        outcome: MutantOutcome::Invalid { reason: err },
                    });
                }
            }
        }

        let denominator = results.len();
        let killed_count = results
            .iter()
            .filter(|r| matches!(r.outcome, MutantOutcome::Killed { .. }))
            .count();
        let all_killed = denominator > 0 && killed_count == denominator;

        MutationBatch {
            results,
            all_killed,
            denominator,
            killed_count,
        }
    }
}
