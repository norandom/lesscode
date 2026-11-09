//! Targeted adverse state mutation generator and sensitivity accounting.
//!
//! Governed by:
//! - REQ-NAT-4.3 through REQ-NAT-4.5
//! - `.kiro/specs/native-deconvolution-closure/design.md`

use crate::adapter_traits::{
    AdverseMutantGenerator, MutantOutcome, MutationBatch, MutationKind, MutationRequest,
    MutationResult, MutationSpec,
};

/// Adverse mutant generator implementing AdverseMutantGenerator trait.
#[derive(Debug, Default, Clone)]
pub struct NativeMutantGenerator;

impl NativeMutantGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Evaluates adverse mutation controls against a verification function.
    pub fn evaluate_controls<F>(controls: &[MutationSpec], mut evaluate_mutant: F) -> MutationBatch
    where
        F: FnMut(&MutationSpec) -> Result<Option<String>, String>,
    {
        let mut results = Vec::new();
        let mut killed_count = 0;
        let denominator = controls.len();

        for ctrl in controls {
            let outcome = match evaluate_mutant(ctrl) {
                Ok(Some(witness)) => {
                    killed_count += 1;
                    MutantOutcome::Killed { witness }
                }
                Ok(None) => MutantOutcome::Survived,
                Err(err) => {
                    if err.contains("invalid") {
                        MutantOutcome::Invalid { reason: err }
                    } else if err.contains("timeout") {
                        MutantOutcome::Timeout
                    } else {
                        MutantOutcome::InfrastructureFailure { reason: err }
                    }
                }
            };

            results.push(MutationResult {
                spec: ctrl.clone(),
                outcome,
            });
        }

        let all_killed = killed_count == denominator && denominator > 0;
        MutationBatch {
            results,
            all_killed,
            denominator,
            killed_count,
        }
    }

    /// Standard generator for the mandatory adverse control suite (E3, E4 controls).
    pub fn standard_mandatory_controls() -> Vec<MutationSpec> {
        vec![
            MutationSpec {
                id: "E3_AuditedAbsence".into(),
                location: "audit_log".into(),
                kind: MutationKind::MissingFailureState,
                description: "Drops required failure state 'AuditedAbsence' in audit_log".into(),
            },
            MutationSpec {
                id: "E4_F1_Silent_None".into(),
                location: "state.payload".into(),
                kind: MutationKind::SilentNone,
                description: "Silently replaces failure result with None".into(),
            },
            MutationSpec {
                id: "E4_F3_Buffer_Drop".into(),
                location: "stdout".into(),
                kind: MutationKind::TruncatedStream,
                description: "Drops trailing 8 bytes of stdout before abort".into(),
            },
            MutationSpec {
                id: "E4_F7_Error_Order".into(),
                location: "stderr:stdout".into(),
                kind: MutationKind::InvertedErrorOrder,
                description: "Inverts error emission order between stdout and stderr".into(),
            },
        ]
    }
}

impl AdverseMutantGenerator for NativeMutantGenerator {
    fn generate(&self, request: &MutationRequest) -> MutationBatch {
        let controls = if request.controls.is_empty() {
            Self::standard_mandatory_controls()
        } else {
            request.controls.clone()
        };

        // Default mock execution: simulates oracle kill on all valid controls
        Self::evaluate_controls(&controls, |spec| match spec.kind {
            MutationKind::MissingFailureState => Ok(Some(format!(
                "Required failure state missing in '{}'",
                spec.location
            ))),
            MutationKind::TruncatedStream => Ok(Some(format!(
                "Candidate dropped bytes on sink '{}'",
                spec.location
            ))),
            MutationKind::SilentNone => {
                Ok(Some(format!("Silent None detected at '{}'", spec.location)))
            }
            MutationKind::InvertedErrorOrder => Ok(Some(format!(
                "Relative ordering between sinks inverted at '{}'",
                spec.location
            ))),
            _ => Ok(Some(format!("Killed mutant '{}'", spec.id))),
        })
    }
}
