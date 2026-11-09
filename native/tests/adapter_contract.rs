//! Test suite for normative adapter traits and wire contracts.
//!
//! Governed by:
//! - `.kiro/specs/native-deconvolution-closure/adapter-contract.md`
//! - REQ-NAT-1.1, REQ-NAT-3.1, REQ-NAT-5.1, REQ-NAT-6.3

use lesscode_core::adapter_traits::*;

#[test]
fn verification_result_roundtrip_json() {
    let verified = VerificationResult::Verified {
        evidence: "hash123".into(),
        domain: "[-10, 10]".into(),
    };
    let json = serde_json::to_string(&verified).unwrap();
    let back: VerificationResult = serde_json::from_str(&json).unwrap();
    assert_eq!(verified, back);

    let divergent = VerificationResult::Divergent {
        witness: "param x=-0.0 yielded mismatch".into(),
    };
    let json_div = serde_json::to_string(&divergent).unwrap();
    let back_div: VerificationResult = serde_json::from_str(&json_div).unwrap();
    assert_eq!(divergent, back_div);

    let unverified = VerificationResult::Unverified {
        reason: "incomplete domain".into(),
    };
    let json_unv = serde_json::to_string(&unverified).unwrap();
    let back_unv: VerificationResult = serde_json::from_str(&json_unv).unwrap();
    assert_eq!(unverified, back_unv);

    let failed = VerificationResult::ExecutionFailed {
        failure: "timeout 5000ms".into(),
    };
    let json_fail = serde_json::to_string(&failed).unwrap();
    let back_fail: VerificationResult = serde_json::from_str(&json_fail).unwrap();
    assert_eq!(failed, back_fail);
}

#[test]
fn tagged_value_ieee754_and_special_values() {
    let nan_val = TaggedValue::from_f64(f64::NAN);
    assert!(nan_val.is_nan());
    assert!(nan_val.as_f64().unwrap().is_nan());

    let pos_zero = TaggedValue::from_f64(0.0);
    let neg_zero = TaggedValue::from_f64(-0.0);
    assert_ne!(pos_zero, neg_zero); // distinct bit patterns in FloatBits
    assert_eq!(pos_zero.as_f64().unwrap(), 0.0);
    assert_eq!(neg_zero.as_f64().unwrap(), -0.0);

    let pos_inf = TaggedValue::from_f64(f64::INFINITY);
    let neg_inf = TaggedValue::from_f64(f64::NEG_INFINITY);
    assert!(pos_inf.as_f64().unwrap().is_infinite());
    assert!(neg_inf.as_f64().unwrap().is_infinite());
    assert_ne!(pos_inf, neg_inf);

    let bytes = TaggedValue::Bytes(vec![0x00, 0xFF, 0x42]);
    let json_bytes = serde_json::to_string(&bytes).unwrap();
    let back_bytes: TaggedValue = serde_json::from_str(&json_bytes).unwrap();
    assert_eq!(bytes, back_bytes);
}

#[test]
fn mock_state_frame_analyzer_implementation() {
    struct MockAnalyzer;
    impl StateFrameAnalyzer for MockAnalyzer {
        fn analyze(&self, request: &AnalysisRequest) -> AnalysisResult {
            let mut violations = Vec::new();
            for write in &request.source_paths {
                if !request.frame_spec.allowed_write_sites.contains(write) {
                    violations.push(format!("Unauthorized write: {write}"));
                }
            }
            AnalysisResult {
                write_sites: request.source_paths.clone(),
                permitted_writes: request.frame_spec.allowed_write_sites.clone(),
                violations: violations.clone(),
                unverified_gaps: Vec::new(),
                is_valid: violations.is_empty(),
            }
        }
    }

    let analyzer = MockAnalyzer;
    let req = AnalysisRequest {
        scope_name: "test_scope".into(),
        source_paths: vec!["store.py".into(), "evil_leak.py".into()],
        frame_spec: FrameSpec {
            allowed_write_sites: vec!["store.py".into()],
            permitted_shell_locations: vec!["shell.py".into()],
            failure_invariants: vec!["AuditedAbsence".into()],
        },
        effect_profile: "python_v1".into(),
    };
    let res = analyzer.analyze(&req);
    assert!(!res.is_valid);
    assert_eq!(res.violations.len(), 1);
    assert!(res.violations[0].contains("evil_leak.py"));
}

#[test]
fn mock_pure_core_verifier_implementation() {
    struct MockVerifier;
    impl PureCoreVerifier for MockVerifier {
        fn compare(&self, request: &ComparisonRequest) -> VerificationResult {
            if request.domain.param_names.is_empty() {
                return VerificationResult::Unverified {
                    reason: "Empty domain".into(),
                };
            }
            VerificationResult::Verified {
                evidence: "mock_eval_hash".into(),
                domain: "finite_cartesian".into(),
            }
        }
    }

    let verifier = MockVerifier;
    let req = ComparisonRequest {
        entrypoint_baseline: "base_fn".into(),
        entrypoint_candidate: "cand_fn".into(),
        domain: DomainSpec {
            param_names: vec!["x".into()],
            finite_values: vec![("x".into(), vec![TaggedValue::Int64(1)])],
            sample_limit: None,
        },
        timeout_ms: 1000,
    };
    let res = verifier.compare(&req);
    match res {
        VerificationResult::Verified { domain, .. } => assert_eq!(domain, "finite_cartesian"),
        _ => panic!("Expected verified"),
    }
}
