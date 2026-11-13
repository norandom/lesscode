//! Why a call failed. No variant holds the key or anything derived from it.

/// The longest part of an endpoint's message that is shown or recorded.
pub const SAID_LIMIT: usize = 300;

#[derive(Debug, Clone, PartialEq)]
pub enum CallError {
    /// The endpoint file or a setting of the run: what is wrong and where.
    Config(String),
    Unreachable {
        host: String,
        detail: String,
    },
    /// The endpoint answered with a refusal.
    Refused {
        host: String,
        status: u16,
        /// The endpoint's message, shortened.
        said: String,
        /// The wait the endpoint stated in a header, as it wrote it.
        retry_after: Option<String>,
    },
    /// The endpoint answered and the reply could not be read.
    Unreadable {
        host: String,
        detail: String,
    },
    NoAnswerInTime {
        host: String,
        seconds: u64,
    },
    WaitLimit {
        host: String,
        waited_seconds: u64,
        limit_seconds: u64,
        attempts: u32,
    },
}

/// White space collapsed and the text cut to `SAID_LIMIT` characters.
pub fn shortened(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(SAID_LIMIT)
        .collect()
}
