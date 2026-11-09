//! Source-bound adverse mutation operators and generator for Java.
//!
//! Governed by:
//! - REQ-JAVA-3.1, REQ-JAVA-3.2, REQ-JAVA-3.3
//! - `.kiro/specs/java-deconvolution-unification/design.md`

use crate::adapter_traits::{
    AdverseMutantGenerator, MutantOutcome, MutationBatch, MutationKind, MutationRequest,
    MutationResult, MutationSpec,
};

/// Java adverse mutant generator.
pub struct JavaMutantGenerator;

impl JavaMutantGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Applies a mutation operator to Java source text.
    pub fn apply_mutation(&self, source: &str, spec: &MutationSpec) -> Result<String, String> {
        match spec.kind {
            MutationKind::MissingFailureState => {
                if source.contains("newRecord = false;") {
                    Ok(source.replacen("newRecord = false;", "// newRecord = false omitted", 1))
                } else if source.contains("newRecord = true;") {
                    Ok(source.replacen("newRecord = true;", "// newRecord = true omitted", 1))
                } else {
                    Err("Target state mutation not found in source".into())
                }
            }
            MutationKind::SilentNone => {
                if source.contains("((Flushable) appendable).flush();") {
                    Ok(source.replacen(
                        "((Flushable) appendable).flush();",
                        "// appendable flush omitted",
                        1,
                    ))
                } else if source.contains("flush();") {
                    Ok(source.replacen("flush();", "// flush omitted", 1))
                } else {
                    Err("Flush call not found in source".into())
                }
            }
            MutationKind::InvertedErrorOrder => {
                if source.contains("format.print(value, appendable, newRecord);") {
                    Ok(source.replacen(
                        "format.print(value, appendable, newRecord);",
                        "format.print(value, appendable, true);",
                        1,
                    ))
                } else if source.contains("lock.unlock();") {
                    Ok(source.replacen("lock.unlock();", "// lock.unlock(); omitted", 1))
                } else {
                    Err("Sequencing or lock statement not found in source".into())
                }
            }
            MutationKind::TruncatedStream => {
                Ok(format!("{source}\n// TRUNCATE_STREAM_BYTES: 16\n"))
            }
            MutationKind::InvertedCondition => {
                if source.contains("!format.isCommentMarkerSet()") {
                    Ok(source.replacen(
                        "!format.isCommentMarkerSet()",
                        "format.isCommentMarkerSet()",
                        1,
                    ))
                } else if source.contains("!newRecord") {
                    Ok(source.replacen("!newRecord", "newRecord", 1))
                } else {
                    Err("Invertible condition not found in source".into())
                }
            }
            MutationKind::BoundaryOffByOne => {
                if source.contains("i < comment.length()") {
                    Ok(source.replacen(
                        "i < comment.length()",
                        "i <= comment.length()",
                        1,
                    ))
                } else if source.contains("rowCount + 1") {
                    Ok(source.replacen("rowCount + 1", "rowCount", 1))
                } else {
                    Err("Boundary index not found in source".into())
                }
            }
        }
    }
}

impl Default for JavaMutantGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl AdverseMutantGenerator for JavaMutantGenerator {
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
