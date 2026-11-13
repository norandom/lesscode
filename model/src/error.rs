//! Why a call failed, in one line a user can act on. No variant holds the
//! key, and text that came from an endpoint has the key removed before it
//! gets here.

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

impl CallError {
    /// One line: whether the endpoint could not be reached, refused the call,
    /// answered unreadably or did not answer in time, with its own words
    /// where it gave any.
    pub fn reason(&self) -> String {
        match self {
            CallError::Config(what) => format!("configuration: {what}"),
            CallError::Unreachable { host, detail } => {
                format!("the endpoint {host} could not be reached: {detail}")
            }
            CallError::Refused {
                host, status, said, ..
            } => format!("the endpoint {host} refused the call (status {status}): {said}"),
            CallError::Unreadable { host, detail } => {
                format!("the endpoint {host} answered and the reply could not be read: {detail}")
            }
            CallError::NoAnswerInTime { host, seconds } => {
                format!("the endpoint {host} did not answer within {seconds} s")
            }
            CallError::WaitLimit {
                host,
                waited_seconds,
                limit_seconds,
                attempts,
            } => format!(
                "the endpoint {host} kept asking to come back later: waited {waited_seconds} s \
                 in {attempts} attempts, the limit is {limit_seconds} s"
            ),
        }
    }
}

impl std::fmt::Display for CallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.reason())
    }
}

/// Text from an endpoint or a transport, made fit to show: the key removed
/// (some services repeat it in a refusal), white space collapsed, cut to
/// `SAID_LIMIT` characters.
pub fn shown(text: &str, key: &str) -> String {
    let without_key = if key.is_empty() {
        text.to_owned()
    } else {
        text.replace(key, "[key]")
    };
    without_key
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(SAID_LIMIT)
        .collect()
}
