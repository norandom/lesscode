//! Output stream buffer and event order preservation monitor.
//!
//! Governed by:
//! - REQ-NAT-4.1, REQ-NAT-4.2
//! - `.kiro/specs/native-deconvolution-closure/design.md`

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

/// Type of stream divergence observed across execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamDivergenceKind {
    Truncated,
    Missing,
    OrderMismatch,
    ContentMismatch,
}

/// Individual recorded stream event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamEvent {
    pub sink: String,
    pub payload: String,
    pub seq: usize,
}

/// Declarative contract defining monitored and commutative streams.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StreamContract {
    pub monitored_sinks: Vec<String>,
    pub commutative_sinks: BTreeSet<String>,
}

/// Attributable divergence finding on a stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamDivergence {
    pub sink: String,
    pub kind: StreamDivergenceKind,
    pub witness: String,
    pub expected: String,
    pub actual: String,
}

fn check_sink_content(
    sink: &str,
    base_text: &str,
    cand_text: &str,
    is_commutative: bool,
    divergences: &mut Vec<StreamDivergence>,
) {
    if cand_text.is_empty() && !base_text.is_empty() {
        divergences.push(StreamDivergence {
            sink: sink.to_string(),
            kind: StreamDivergenceKind::Missing,
            witness: format!("Candidate completely omitted output on sink '{sink}'"),
            expected: base_text.to_string(),
            actual: cand_text.to_string(),
        });
    } else if is_commutative {
        let mut base_lines: Vec<&str> = base_text.lines().collect();
        let mut cand_lines: Vec<&str> = cand_text.lines().collect();
        base_lines.sort_unstable();
        cand_lines.sort_unstable();
        if base_lines != cand_lines {
            divergences.push(StreamDivergence {
                sink: sink.to_string(),
                kind: StreamDivergenceKind::ContentMismatch,
                witness: format!("Commutative sink '{sink}' content mismatch"),
                expected: base_text.to_string(),
                actual: cand_text.to_string(),
            });
        }
    } else if base_text != cand_text {
        if base_text.starts_with(cand_text) {
            let dropped = base_text.len() - cand_text.len();
            divergences.push(StreamDivergence {
                sink: sink.to_string(),
                kind: StreamDivergenceKind::Truncated,
                witness: format!(
                    "Candidate dropped or failed to flush {dropped} bytes on sink '{sink}' prior to completion/abort"
                ),
                expected: base_text.to_string(),
                actual: cand_text.to_string(),
            });
        } else {
            divergences.push(StreamDivergence {
                sink: sink.to_string(),
                kind: StreamDivergenceKind::ContentMismatch,
                witness: format!("Output on sink '{sink}' did not match original output"),
                expected: base_text.to_string(),
                actual: cand_text.to_string(),
            });
        }
    }
}

fn check_stream_ordering(
    baseline: &[StreamEvent],
    candidate: &[StreamEvent],
    contract: &StreamContract,
    divergences: &mut Vec<StreamDivergence>,
) {
    let base_non_comm: Vec<&str> = baseline
        .iter()
        .filter(|e| !contract.commutative_sinks.contains(&e.sink))
        .map(|e| e.sink.as_str())
        .collect();
    let cand_non_comm: Vec<&str> = candidate
        .iter()
        .filter(|e| !contract.commutative_sinks.contains(&e.sink))
        .map(|e| e.sink.as_str())
        .collect();

    if base_non_comm != cand_non_comm {
        let mut sorted_base = base_non_comm.clone();
        let mut sorted_cand = cand_non_comm.clone();
        sorted_base.sort_unstable();
        sorted_cand.sort_unstable();

        if sorted_base == sorted_cand {
            divergences.push(StreamDivergence {
                sink: "stream".into(),
                kind: StreamDivergenceKind::OrderMismatch,
                witness: format!(
                    "Relative ordering between sinks was inverted: expected sequence {:?}, got {:?}",
                    base_non_comm, cand_non_comm
                ),
                expected: format!("{base_non_comm:?}"),
                actual: format!("{cand_non_comm:?}"),
            });
        }
    }
}

/// Verifies that candidate output stream events preserve baseline content and order.
pub fn verify_stream_preservation(
    baseline: &[StreamEvent],
    candidate: &[StreamEvent],
    contract: &StreamContract,
) -> Vec<StreamDivergence> {
    let mut divergences = Vec::new();
    let mut base_by_sink: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut cand_by_sink: BTreeMap<&str, Vec<&str>> = BTreeMap::new();

    for ev in baseline {
        base_by_sink.entry(&ev.sink).or_default().push(&ev.payload);
    }
    for ev in candidate {
        cand_by_sink.entry(&ev.sink).or_default().push(&ev.payload);
    }

    let mut all_sinks: BTreeSet<&str> = base_by_sink.keys().copied().collect();
    all_sinks.extend(cand_by_sink.keys().copied());

    for sink in all_sinks {
        let base_text: String = base_by_sink.get(sink).map_or(String::new(), |v| v.concat());
        let cand_text: String = cand_by_sink.get(sink).map_or(String::new(), |v| v.concat());
        let is_comm = contract.commutative_sinks.contains(sink);
        check_sink_content(sink, &base_text, &cand_text, is_comm, &mut divergences);
    }

    check_stream_ordering(baseline, candidate, contract, &mut divergences);
    divergences
}
