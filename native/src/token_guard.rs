//! Token limit guard, streaming chunk decoder, and causal refinement feedback bridge.
//!
//! Governed by:
//! - REQ-NAT-6.1, REQ-NAT-6.2
//! - `.kiro/specs/native-deconvolution-closure/design.md`

use serde::{Deserialize, Serialize};

use crate::jsonval::J;
use crate::state_frame::StateViolation;
use crate::stream_monitor::StreamDivergence;

pub const DEFAULT_MAX_INPUT_TOKENS: i64 = 120_000;

/// Error when input or cumulative token limit is exceeded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenLimitExceeded {
    pub counted_tokens: i64,
    pub max_tokens: i64,
    pub reason: String,
}

impl std::fmt::Display for TokenLimitExceeded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Token limit exceeded: counted {} > max {} ({})",
            self.counted_tokens, self.max_tokens, self.reason
        )
    }
}
impl std::error::Error for TokenLimitExceeded {}

/// Token guard enforcing model provider input limits.
#[derive(Debug, Clone)]
pub struct TokenGuard {
    pub max_input_tokens: i64,
}

impl Default for TokenGuard {
    fn default() -> Self {
        Self {
            max_input_tokens: DEFAULT_MAX_INPUT_TOKENS,
        }
    }
}

impl TokenGuard {
    pub fn new(max_input_tokens: i64) -> Self {
        Self { max_input_tokens }
    }

    /// Estimates token count based on whitespace and byte length.
    pub fn estimate_tokens(text: &str) -> i64 {
        if text.is_empty() {
            return 0;
        }
        // Conservative upper bound: ~3.5 chars per token for code/JSON
        let char_count = text.chars().count();
        ((char_count + 3) / 4) as i64
    }

    /// Checks whether request prompt complies with the input token ceiling.
    pub fn check_input_limit(&self, prompt: &str) -> Result<i64, TokenLimitExceeded> {
        let counted = Self::estimate_tokens(prompt);
        if counted > self.max_input_tokens {
            Err(TokenLimitExceeded {
                counted_tokens: counted,
                max_tokens: self.max_input_tokens,
                reason: format!("Prompt exceeds maximum token ceiling of {}", self.max_input_tokens),
            })
        } else {
            Ok(counted)
        }
    }
}

/// Stateful chunk decoder handling streaming HTTP / SSE chunks across UTF-8 boundaries.
#[derive(Debug, Default)]
pub struct StreamingChunkDecoder {
    buffer: Vec<u8>,
}

impl StreamingChunkDecoder {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Feeds raw bytes into the decoder, yielding decoded SSE payload lines.
    pub fn feed_bytes(&mut self, chunk: &[u8]) -> Vec<String> {
        self.buffer.extend_from_slice(chunk);
        let mut lines = Vec::new();

        let mut start = 0;
        for i in 0..self.buffer.len() {
            if self.buffer[i] == b'\n' {
                let slice = &self.buffer[start..i];
                let line_bytes = if !slice.is_empty() && slice[slice.len() - 1] == b'\r' {
                    &slice[..slice.len() - 1]
                } else {
                    slice
                };

                if let Ok(line_str) = std::str::from_utf8(line_bytes) {
                    if let Some(data) = line_str.strip_prefix("data: ") {
                        lines.push(data.trim().to_string());
                    } else if !line_str.trim().is_empty() {
                        lines.push(line_str.trim().to_string());
                    }
                }
                start = i + 1;
            }
        }

        self.buffer.drain(..start);
        lines
    }

    /// Flushes any remaining bytes.
    pub fn flush_remaining(&mut self) -> Option<String> {
        if self.buffer.is_empty() {
            None
        } else {
            let res = String::from_utf8_lossy(&self.buffer).to_string();
            self.buffer.clear();
            Some(res)
        }
    }
}

/// Formats causal failure witnesses into structured feedback for prompt refinement.
pub struct CausalFeedbackBridge;

impl CausalFeedbackBridge {
    pub fn format_state_violations(violations: &[StateViolation]) -> J {
        let items: Vec<J> = violations
            .iter()
            .map(|v| {
                J::obj(vec![
                    ("type".into(), J::str("state_violation")),
                    ("location".into(), J::str(&v.location)),
                    ("witness".into(), J::str(&v.witness)),
                ])
            })
            .collect();
        J::arr(items)
    }

    pub fn format_stream_divergences(divergences: &[StreamDivergence]) -> J {
        let items: Vec<J> = divergences
            .iter()
            .map(|d| {
                J::obj(vec![
                    ("type".into(), J::str("stream_divergence")),
                    ("sink".into(), J::str(&d.sink)),
                    ("witness".into(), J::str(&d.witness)),
                ])
            })
            .collect();
        J::arr(items)
    }

    pub fn format_pure_counterexample(witness: &str) -> J {
        J::obj(vec![
            ("type".into(), J::str("pure_core_divergence")),
            ("witness".into(), J::str(witness)),
        ])
    }
}
