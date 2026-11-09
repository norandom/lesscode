//! Test suite for bounded pure core verification and Cartesian domain scheduling.
//!
//! Governed by:
//! - REQ-NAT-3.1 through REQ-NAT-3.5
//! - `.kiro/specs/native-deconvolution-closure/tasks.md`

use std::collections::BTreeMap;

use lesscode_core::adapter_traits::*;
use lesscode_core::pure_verifier::*;

#[test]
fn cartesian_domain_generation() {
    let domain = DomainSpec {
        param_names: vec!["x".into(), "y".into()],
        finite_values: vec![
            (
                "x".into(),
                vec![TaggedValue::Int64(1), TaggedValue::Int64(2)],
            ),
            (
                "y".into(),
                vec![
                    TaggedValue::String("a".into()),
                    TaggedValue::String("b".into()),
                    TaggedValue::String("c".into()),
                ],
            ),
        ],
        sample_limit: None,
    };

    let cases = NativePureCoreVerifier::generate_cartesian_cases(&domain);
    assert_eq!(cases.len(), 6); // 2 * 3

    // Test sample limit
    let mut limited = domain.clone();
    limited.sample_limit = Some(4);
    let limited_cases = NativePureCoreVerifier::generate_cartesian_cases(&limited);
    assert_eq!(limited_cases.len(), 4);
}

#[test]
fn pure_equivalence_verified_on_identical_logic() {
    let domain = DomainSpec {
        param_names: vec!["val".into()],
        finite_values: vec![("val".into(), (-5..=5).map(TaggedValue::Int64).collect())],
        sample_limit: None,
    };

    let baseline = |inputs: &BTreeMap<String, TaggedValue>| -> Result<TaggedValue, String> {
        match inputs.get("val").unwrap() {
            TaggedValue::Int64(n) => Ok(TaggedValue::Int64(n * 2)),
            _ => Err("Invalid type".into()),
        }
    };

    let candidate = |inputs: &BTreeMap<String, TaggedValue>| -> Result<TaggedValue, String> {
        match inputs.get("val").unwrap() {
            TaggedValue::Int64(n) => Ok(TaggedValue::Int64(n + n)),
            _ => Err("Invalid type".into()),
        }
    };

    let result = NativePureCoreVerifier::verify_equivalence(&domain, baseline, candidate);
    match result {
        VerificationResult::Verified {
            evidence,
            domain: d,
        } => {
            assert!(evidence.contains("11 exhaustive"));
            assert!(d.contains("val"));
        }
        _ => panic!("Expected Verified"),
    }
}

#[test]
fn pure_equivalence_divergent_on_edge_mismatch() {
    let domain = DomainSpec {
        param_names: vec!["val".into()],
        finite_values: vec![(
            "val".into(),
            vec![
                TaggedValue::from_f64(0.0),
                TaggedValue::from_f64(-0.0),
                TaggedValue::from_f64(f64::NAN),
            ],
        )],
        sample_limit: None,
    };

    // Baseline handles NaN by returning 0, candidate returns 1
    let baseline = |inputs: &BTreeMap<String, TaggedValue>| -> Result<TaggedValue, String> {
        let v = inputs.get("val").unwrap();
        if v.is_nan() {
            Ok(TaggedValue::Int64(0))
        } else {
            Ok(TaggedValue::Int64(42))
        }
    };

    let candidate = |inputs: &BTreeMap<String, TaggedValue>| -> Result<TaggedValue, String> {
        let v = inputs.get("val").unwrap();
        if v.is_nan() {
            Ok(TaggedValue::Int64(1)) // divergence!
        } else {
            Ok(TaggedValue::Int64(42))
        }
    };

    let result = NativePureCoreVerifier::verify_equivalence(&domain, baseline, candidate);
    match result {
        VerificationResult::Divergent { witness } => {
            assert!(witness.contains("Divergence on inputs"));
        }
        _ => panic!("Expected Divergent"),
    }
}

#[test]
fn pure_verifier_trait_dispositions() {
    let verifier = NativePureCoreVerifier::new();

    // 1. Zero timeout -> ExecutionFailed
    let req_zero_timeout = ComparisonRequest {
        entrypoint_baseline: "base".into(),
        entrypoint_candidate: "cand".into(),
        domain: DomainSpec {
            param_names: vec!["x".into()],
            finite_values: vec![("x".into(), vec![TaggedValue::Int64(1)])],
            sample_limit: None,
        },
        timeout_ms: 0,
    };
    assert!(matches!(
        verifier.compare(&req_zero_timeout),
        VerificationResult::ExecutionFailed { .. }
    ));

    // 2. Empty domain -> Unverified
    let req_empty_domain = ComparisonRequest {
        entrypoint_baseline: "base".into(),
        entrypoint_candidate: "cand".into(),
        domain: DomainSpec::default(),
        timeout_ms: 1000,
    };
    assert!(matches!(
        verifier.compare(&req_empty_domain),
        VerificationResult::Unverified { .. }
    ));

    // 3. Candidate simulated divergence -> Divergent
    let req_divergent = ComparisonRequest {
        entrypoint_baseline: "base".into(),
        entrypoint_candidate: "cand_divergent".into(),
        domain: DomainSpec {
            param_names: vec!["x".into()],
            finite_values: vec![("x".into(), vec![TaggedValue::Int64(1)])],
            sample_limit: None,
        },
        timeout_ms: 1000,
    };
    assert!(matches!(
        verifier.compare(&req_divergent),
        VerificationResult::Divergent { .. }
    ));
}
