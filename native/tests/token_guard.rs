//! Test suite for token limit guard and causal refinement feedback bridge.
//!
//! Governed by:
//! - REQ-NAT-6.1, REQ-NAT-6.2
//! - `.kiro/specs/native-deconvolution-closure/tasks.md`

use lesscode_core::jsonval::J;
use lesscode_core::state_frame::{StateViolation, ViolationKind};
use lesscode_core::stream_monitor::{StreamDivergence, StreamDivergenceKind};
use lesscode_core::token_guard::*;

#[test]
fn token_guard_estimation_and_limit() {
    let guard = TokenGuard::new(100);

    let short_text = "fn compute(x: i32) -> i32 { x * 2 }";
    let counted = guard.check_input_limit(short_text).unwrap();
    assert!(counted < 100);
    assert!(counted > 0);

    let huge_text = "a".repeat(1000); // ~250 tokens > 100 limit
    let err = guard.check_input_limit(&huge_text).unwrap_err();
    assert_eq!(err.max_tokens, 100);
    assert!(err.counted_tokens > 100);
    assert!(err.reason.contains("exceeds maximum token ceiling"));
}

#[test]
fn streaming_chunk_decoder_sse_and_split_chunks() {
    let mut decoder = StreamingChunkDecoder::new();

    // Chunk 1: partial SSE event
    let chunk1 = b"data: {\"seq\":1,\"delta\":\"";
    let lines1 = decoder.feed_bytes(chunk1);
    assert!(lines1.is_empty());

    // Chunk 2: completing the event and starting another
    let chunk2 = b"Hello\"}\ndata: {\"seq\":2,\"delta\":\"World\"}\n";
    let lines2 = decoder.feed_bytes(chunk2);
    assert_eq!(lines2.len(), 2);
    assert_eq!(lines2[0], "{\"seq\":1,\"delta\":\"Hello\"}");
    assert_eq!(lines2[1], "{\"seq\":2,\"delta\":\"World\"}");

    // Remaining flush
    let remaining = decoder.flush_remaining();
    assert!(remaining.is_none());
}

#[test]
fn causal_feedback_bridge_formats_witnesses() {
    let violations = vec![StateViolation {
        location: "audit_log".into(),
        kind: ViolationKind::MissingFailureInvariant,
        witness: "AuditedAbsence missing in audit_log".into(),
        line: None,
    }];

    let json_viols = CausalFeedbackBridge::format_state_violations(&violations);
    match json_viols {
        J::Arr(items) => {
            assert_eq!(items.len(), 1);
            match &items[0] {
                J::Obj(map) => {
                    let map_ref: std::collections::BTreeMap<_, _> =
                        map.iter().map(|(k, v)| (k.as_str(), v)).collect();
                    assert_eq!(map_ref["location"], &J::str("audit_log"));
                    assert_eq!(map_ref["type"], &J::str("state_violation"));
                }
                _ => panic!("Expected object"),
            }
        }
        _ => panic!("Expected array"),
    }

    let divergences = vec![StreamDivergence {
        sink: "stdout".into(),
        kind: StreamDivergenceKind::Truncated,
        witness: "Dropped 8 bytes".into(),
        expected: "all".into(),
        actual: "al".into(),
    }];
    let json_divs = CausalFeedbackBridge::format_stream_divergences(&divergences);
    assert!(matches!(json_divs, J::Arr(items) if items.len() == 1));

    let pure_ce = CausalFeedbackBridge::format_pure_counterexample("inputs val=-0.0 mismatch");
    assert!(matches!(pure_ce, J::Obj(..)));
}
