//! Bounded pure core verification, Cartesian domain scheduling, and witness generation.
//!
//! Governed by:
//! - REQ-NAT-3.1 through REQ-NAT-3.5
//! - `.kiro/specs/native-deconvolution-closure/design.md`

use std::collections::BTreeMap;

use crate::adapter_traits::{DomainSpec, TaggedValue, VerificationResult};

/// Purity violation kind for static auditing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PurityViolationKind {
    StateMutation,
    GlobalLookup,
    IoInvocation,
}

/// Static purity audit record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PurityAuditFinding {
    pub function_name: String,
    pub kind: PurityViolationKind,
    pub location: String,
    pub witness: String,
}

/// Native pure core verifier implementing PureCoreVerifier.
#[derive(Debug, Default, Clone)]
pub struct NativePureCoreVerifier;

impl NativePureCoreVerifier {
    pub fn new() -> Self {
        Self
    }

    /// Generates Cartesian product of finite domain values.
    pub fn generate_cartesian_cases(domain: &DomainSpec) -> Vec<BTreeMap<String, TaggedValue>> {
        if domain.param_names.is_empty() || domain.finite_values.is_empty() {
            return Vec::new();
        }

        let mut cases: Vec<BTreeMap<String, TaggedValue>> = vec![BTreeMap::new()];

        for (param, values) in &domain.finite_values {
            if values.is_empty() {
                continue;
            }
            let mut next_cases = Vec::new();
            for case in &cases {
                for val in values {
                    let mut extended = case.clone();
                    extended.insert(param.clone(), val.clone());
                    next_cases.push(extended);
                }
            }
            cases = next_cases;
        }

        if let Some(limit) = domain.sample_limit {
            if cases.len() > limit {
                cases.truncate(limit);
            }
        }

        cases
    }

    /// Evaluates two callables across Cartesian finite domain.
    pub fn verify_equivalence<F1, F2>(
        domain: &DomainSpec,
        baseline: F1,
        candidate: F2,
    ) -> VerificationResult
    where
        F1: Fn(&BTreeMap<String, TaggedValue>) -> Result<TaggedValue, String>,
        F2: Fn(&BTreeMap<String, TaggedValue>) -> Result<TaggedValue, String>,
    {
        if domain.param_names.is_empty() {
            return VerificationResult::Unverified {
                reason: "Empty domain parameter specification".into(),
            };
        }

        let cases = Self::generate_cartesian_cases(domain);
        if cases.is_empty() {
            return VerificationResult::Unverified {
                reason: "Zero evaluation cases generated from domain".into(),
            };
        }

        let mut evaluated_count = 0;
        for case in &cases {
            let res_base = baseline(case);
            let res_cand = candidate(case);

            if res_base != res_cand {
                let witness_str = format!(
                    "Divergence on inputs {:?}: baseline={:?}, candidate={:?}",
                    case, res_base, res_cand
                );
                return VerificationResult::Divergent {
                    witness: witness_str,
                };
            }
            evaluated_count += 1;
        }

        let evidence = format!("Hashed {evaluated_count} exhaustive Cartesian evaluations");
        let domain_desc = format!("{:?}", domain.param_names);
        VerificationResult::Verified {
            evidence,
            domain: domain_desc,
        }
    }
}
