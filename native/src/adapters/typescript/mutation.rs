//! Source-bound adverse mutation operators and generator for TypeScript.
//!
//! Governed by:
//! - REQ-TS-3.1, REQ-TS-3.2, REQ-TS-3.3
//! - `.kiro/specs/typescript-deconvolution-adapter/design.md`

use crate::adapter_traits::{
    AdverseMutantGenerator, MutantOutcome, MutationBatch, MutationKind, MutationRequest,
    MutationResult, MutationSpec,
};

/// TypeScript adverse mutant generator.
pub struct TypeScriptMutantGenerator;

impl TypeScriptMutantGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Applies a mutation operator to TypeScript source text.
    pub fn apply_mutation(&self, source: &str, spec: &MutationSpec) -> Result<String, String> {
        match spec.kind {
            MutationKind::MissingFailureState => {
                if source.contains("\"warning\"") {
                    Ok(source.replacen("\"warning\"", "\"notice\"", 1))
                } else if source.contains("\"blocked\"") {
                    Ok(source.replacen("\"blocked\"", "\"warning\"", 1))
                } else {
                    Err("Target severity literal not found in source".into())
                }
            }
            MutationKind::SilentNone => {
                if source.contains(".replace(/[^a-z0-9]+/g, \"-\")") {
                    Ok(source.replacen(".replace(/[^a-z0-9]+/g, \"-\")", "", 1))
                } else {
                    Err("Slugify regex pattern not found in source".into())
                }
            }
            MutationKind::InvertedErrorOrder => {
                if source.contains(".filter((n) => n.severity === \"blocked\")") {
                    Ok(source.replacen(
                        ".filter((n) => n.severity === \"blocked\")",
                        ".filter((n) => n.severity === \"blocked\").reverse()",
                        1,
                    ))
                } else {
                    Err("Filter call not found in source for order inversion".into())
                }
            }
            MutationKind::TruncatedStream => {
                Ok(format!("{source}\n// TRUNCATE_STREAM_BYTES: 16\n"))
            }
            MutationKind::InvertedCondition => {
                if source.contains("!i.enabled") {
                    Ok(source.replacen("!i.enabled", "i.enabled", 1))
                } else {
                    Err("Condition !i.enabled not found".into())
                }
            }
            MutationKind::BoundaryOffByOne => {
                if source.contains("n.length === 1") {
                    Ok(source.replacen("n.length === 1", "n.length === 0", 1))
                } else {
                    Err("Boundary check not found".into())
                }
            }
        }
    }
}

impl Default for TypeScriptMutantGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl AdverseMutantGenerator for TypeScriptMutantGenerator {
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
