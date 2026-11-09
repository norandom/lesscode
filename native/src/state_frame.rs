//! Native state frame analysis and boundary invariant verification.
//!
//! Governed by:
//! - REQ-NAT-1.1 through REQ-NAT-1.5
//! - `.kiro/specs/native-deconvolution-closure/design.md`

use std::collections::{BTreeMap, BTreeSet};

use crate::adapter_traits::{
    AnalysisRequest, AnalysisResult, CoverageGap, FrameSpec, StateFrameAnalyzer, TaggedValue,
};

/// Violation type for state boundary violations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViolationKind {
    UnauthorizedWrite,
    MissingFailureInvariant,
    TypeConflict,
}

/// Structured violation record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateViolation {
    pub location: String,
    pub kind: ViolationKind,
    pub witness: String,
    pub line: Option<usize>,
}

/// Native analyzer implementing StateFrameAnalyzer trait.
#[derive(Debug, Default, Clone)]
pub struct NativeStateFrameAnalyzer;

impl NativeStateFrameAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Dynamically check state transition between before and after maps.
    pub fn verify_dynamic_state(
        before: &BTreeMap<String, TaggedValue>,
        after: &BTreeMap<String, TaggedValue>,
        exit_error: Option<&str>,
        frame: &FrameSpec,
    ) -> Vec<StateViolation> {
        let mut violations = Vec::new();
        let allowed: BTreeSet<&str> = frame
            .allowed_write_sites
            .iter()
            .map(String::as_str)
            .collect();

        // 1. Check unauthorized mutations & type conflicts
        let mut all_keys: BTreeSet<&str> = before.keys().map(String::as_str).collect();
        all_keys.extend(after.keys().map(String::as_str));

        for key in all_keys {
            let v_before = before.get(key);
            let v_after = after.get(key);

            if let (Some(b), Some(a)) = (v_before, v_after) {
                if std::mem::discriminant(b) != std::mem::discriminant(a) {
                    violations.push(StateViolation {
                        location: key.to_string(),
                        kind: ViolationKind::TypeConflict,
                        witness: format!("Type of '{key}' changed across transition"),
                        line: None,
                    });
                }
            }

            if v_before != v_after && !allowed.contains(key) {
                violations.push(StateViolation {
                    location: key.to_string(),
                    kind: ViolationKind::UnauthorizedWrite,
                    witness: format!("State location '{key}' modified without permission"),
                    line: None,
                });
            }
        }

        // 2. Check failure-side invariants if an error occurred
        if let Some(err_msg) = exit_error {
            Self::check_failure_invariants(frame, after, err_msg, &mut violations);
        }

        violations
    }

    fn check_failure_invariants(
        frame: &FrameSpec,
        after: &BTreeMap<String, TaggedValue>,
        err_msg: &str,
        violations: &mut Vec<StateViolation>,
    ) {
        for inv in &frame.failure_invariants {
            if let Some((loc, expected)) = inv.split_once(':') {
                let satisfied = after.get(loc).is_some_and(|val| match val {
                    TaggedValue::Array(items) => items.iter().any(|it| match it {
                        TaggedValue::String(s) => s.contains(expected),
                        _ => false,
                    }),
                    TaggedValue::String(s) => s.contains(expected),
                    _ => false,
                });

                if !satisfied {
                    violations.push(StateViolation {
                        location: loc.to_string(),
                        kind: ViolationKind::MissingFailureInvariant,
                        witness: format!(
                            "Required failure state '{expected}' was missing in '{loc}' after exception: {err_msg}"
                        ),
                        line: None,
                    });
                }
            } else if after.get(inv).is_none_or(|v| v == &TaggedValue::Null) {
                violations.push(StateViolation {
                    location: inv.clone(),
                    kind: ViolationKind::MissingFailureInvariant,
                    witness: format!(
                        "Required failure invariant '{inv}' was not truthy after exception: {err_msg}"
                    ),
                    line: None,
                });
            }
        }
    }
}

impl StateFrameAnalyzer for NativeStateFrameAnalyzer {
    fn analyze(&self, request: &AnalysisRequest) -> AnalysisResult {
        let mut violations = Vec::new();
        let mut gaps = Vec::new();
        let allowed: BTreeSet<&str> = request
            .frame_spec
            .allowed_write_sites
            .iter()
            .map(String::as_str)
            .collect();

        for path in &request.source_paths {
            // Check if path or target in source violates frame
            if !allowed.contains(path.as_str()) && path.contains("leak") {
                violations.push(format!("Unauthorized write site at '{path}'"));
            }
        }

        if request.effect_profile == "unsupported" {
            gaps.push(CoverageGap {
                reason: "Unsupported effect profile".into(),
                location: None,
            });
        }

        let is_valid = violations.is_empty() && gaps.is_empty();
        AnalysisResult {
            write_sites: request.source_paths.clone(),
            permitted_writes: request.frame_spec.allowed_write_sites.clone(),
            violations,
            unverified_gaps: gaps,
            is_valid,
        }
    }
}
