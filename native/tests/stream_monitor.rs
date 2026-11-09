//! Test suite for stream monitor and buffer preservation.
//!
//! Governed by:
//! - REQ-NAT-4.1, REQ-NAT-4.2
//! - `.kiro/specs/native-deconvolution-closure/tasks.md`

use std::collections::BTreeSet;

use lesscode_core::stream_monitor::*;

#[test]
fn exact_stream_match_reports_zero_divergences() {
    let baseline = vec![
        StreamEvent {
            sink: "stdout".into(),
            payload: "Hello world\n".into(),
            seq: 1,
        },
        StreamEvent {
            sink: "stderr".into(),
            payload: "Warn: low disk\n".into(),
            seq: 2,
        },
    ];
    let candidate = baseline.clone();
    let contract = StreamContract::default();

    let divergences = verify_stream_preservation(&baseline, &candidate, &contract);
    assert!(divergences.is_empty());
}

#[test]
fn truncated_stream_catches_dropped_bytes() {
    let baseline = vec![StreamEvent {
        sink: "stdout".into(),
        payload: "Processing complete, exit status 0\n".into(),
        seq: 1,
    }];
    // Candidate drops trailing 8 bytes
    let candidate = vec![StreamEvent {
        sink: "stdout".into(),
        payload: "Processing complete, exit st".into(),
        seq: 1,
    }];
    let contract = StreamContract::default();

    let divergences = verify_stream_preservation(&baseline, &candidate, &contract);
    assert_eq!(divergences.len(), 1);
    assert_eq!(divergences[0].kind, StreamDivergenceKind::Truncated);
    assert!(divergences[0].witness.contains("dropped or failed to flush 7 bytes"));
}

#[test]
fn missing_stream_catches_empty_sink() {
    let baseline = vec![StreamEvent {
        sink: "stderr".into(),
        payload: "Error occurred\n".into(),
        seq: 1,
    }];
    let candidate = vec![];
    let contract = StreamContract::default();

    let divergences = verify_stream_preservation(&baseline, &candidate, &contract);
    assert_eq!(divergences.len(), 1);
    assert_eq!(divergences[0].kind, StreamDivergenceKind::Missing);
    assert!(divergences[0].witness.contains("omitted output on sink 'stderr'"));
}

#[test]
fn order_mismatch_catches_inverted_sinks() {
    let baseline = vec![
        StreamEvent {
            sink: "stdout".into(),
            payload: "Step 1\n".into(),
            seq: 1,
        },
        StreamEvent {
            sink: "stderr".into(),
            payload: "Error in Step 1\n".into(),
            seq: 2,
        },
    ];
    // Candidate inverts order: stderr then stdout
    let candidate = vec![
        StreamEvent {
            sink: "stderr".into(),
            payload: "Error in Step 1\n".into(),
            seq: 1,
        },
        StreamEvent {
            sink: "stdout".into(),
            payload: "Step 1\n".into(),
            seq: 2,
        },
    ];
    let contract = StreamContract::default();

    let divergences = verify_stream_preservation(&baseline, &candidate, &contract);
    assert_eq!(divergences.len(), 1);
    assert_eq!(divergences[0].kind, StreamDivergenceKind::OrderMismatch);
    assert!(divergences[0].witness.contains("Relative ordering between sinks was inverted"));
}

#[test]
fn commutative_sink_permits_line_reordering() {
    let baseline = vec![StreamEvent {
        sink: "audit_log".into(),
        payload: "line1\nline2\n".into(),
        seq: 1,
    }];
    let candidate = vec![StreamEvent {
        sink: "audit_log".into(),
        payload: "line2\nline1\n".into(),
        seq: 1,
    }];
    let mut comm = BTreeSet::new();
    comm.insert("audit_log".into());
    let contract = StreamContract {
        monitored_sinks: vec!["audit_log".into()],
        commutative_sinks: comm,
    };

    let divergences = verify_stream_preservation(&baseline, &candidate, &contract);
    assert!(divergences.is_empty());
}
