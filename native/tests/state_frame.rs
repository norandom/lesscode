//! Test suite for native state frame analysis and density engine.
//!
//! Governed by:
//! - REQ-NAT-1.1 through REQ-NAT-1.5, REQ-NAT-2.1 through REQ-NAT-2.5
//! - `.kiro/specs/native-deconvolution-closure/tasks.md`

use std::collections::BTreeMap;

use lesscode_core::adapter_traits::*;
use lesscode_core::state_frame::*;

#[test]
fn dynamic_state_verification_clean_transition() {
    let mut before = BTreeMap::new();
    before.insert("store_count".into(), TaggedValue::Int64(10));
    before.insert(
        "read_only_config".into(),
        TaggedValue::String("locked".into()),
    );

    let mut after = before.clone();
    after.insert("store_count".into(), TaggedValue::Int64(11));

    let frame = FrameSpec {
        allowed_write_sites: vec!["store_count".into()],
        permitted_shell_locations: vec!["shell.rs".into()],
        failure_invariants: vec![],
    };

    let violations = NativeStateFrameAnalyzer::verify_dynamic_state(&before, &after, None, &frame);
    assert!(violations.is_empty());
}

#[test]
fn dynamic_state_verification_catches_unauthorized_mutation() {
    let mut before = BTreeMap::new();
    before.insert("unprotected_state".into(), TaggedValue::Int64(1));

    let mut after = before.clone();
    after.insert("unprotected_state".into(), TaggedValue::Int64(2));

    let frame = FrameSpec {
        allowed_write_sites: vec!["authorized_only".into()],
        permitted_shell_locations: vec!["shell.rs".into()],
        failure_invariants: vec![],
    };

    let violations = NativeStateFrameAnalyzer::verify_dynamic_state(&before, &after, None, &frame);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].kind, ViolationKind::UnauthorizedWrite);
    assert_eq!(violations[0].location, "unprotected_state");
}

#[test]
fn dynamic_state_verification_catches_missing_failure_invariant() {
    let before = BTreeMap::new();
    let mut after = BTreeMap::new();
    after.insert(
        "audit_log".into(),
        TaggedValue::Array(vec![TaggedValue::String("NormalOp".into())]),
    );

    let frame = FrameSpec {
        allowed_write_sites: vec!["audit_log".into()],
        permitted_shell_locations: vec!["shell.rs".into()],
        failure_invariants: vec!["audit_log:AuditedAbsence".into()],
    };

    // When an exception occurs, AuditedAbsence must be present in audit_log
    let violations = NativeStateFrameAnalyzer::verify_dynamic_state(
        &before,
        &after,
        Some("Serialization failure"),
        &frame,
    );
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].kind, ViolationKind::MissingFailureInvariant);
    assert!(violations[0].witness.contains("AuditedAbsence"));

    // Now satisfy it
    after.insert(
        "audit_log".into(),
        TaggedValue::Array(vec![TaggedValue::String(
            "Error: AuditedAbsence recorded".into(),
        )]),
    );
    let violations_clean = NativeStateFrameAnalyzer::verify_dynamic_state(
        &before,
        &after,
        Some("Serialization failure"),
        &frame,
    );
    assert!(violations_clean.is_empty());
}

#[test]
fn static_state_frame_analyzer_trait() {
    let analyzer = NativeStateFrameAnalyzer::new();
    let req = AnalysisRequest {
        scope_name: "facdrone_lifecycle".into(),
        source_paths: vec!["lifecycle_store.py".into(), "memory_leak.py".into()],
        frame_spec: FrameSpec {
            allowed_write_sites: vec!["lifecycle_store.py".into()],
            permitted_shell_locations: vec!["shell.py".into()],
            failure_invariants: vec![],
        },
        effect_profile: "python_v1".into(),
    };

    let res = analyzer.analyze(&req);
    assert!(!res.is_valid);
    assert_eq!(res.violations.len(), 1);
    assert!(res.violations[0].contains("memory_leak.py"));
}
