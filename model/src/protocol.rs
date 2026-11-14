//! The process protocol of `lesscode-model call`: one request as JSON on
//! standard input, one answer as JSON on standard output, and an exit code.
//! A program that must not link a client library reaches a model this way.

use crate::error::CallError;
use crate::request::{CallRequest, Reply};
use serde_json::{Value, json};

/// The number both sides state. A request with another number is refused,
/// and a caller refuses an answer with another.
pub const PROTOCOL: u64 = 1;

/// The call was answered.
pub const EXIT_ANSWERED: u8 = 0;
/// The command line, the request or the configuration cannot be used;
/// nothing was called.
pub const EXIT_USAGE: u8 = 2;
/// The call was made and failed.
pub const EXIT_FAILED: u8 = 3;

/// The request on standard input: a `CallRequest` with the protocol number.
/// The error says what is wrong with the text.
pub fn read_request(text: &str) -> Result<CallRequest, String> {
    let value: Value = serde_json::from_str(text)
        .map_err(|problem| format!("the request is not JSON: {problem}"))?;
    match value.get("protocol") {
        Some(number) if number.as_u64() == Some(PROTOCOL) => {}
        Some(number) => {
            return Err(format!(
                "the request states protocol {number}, this lesscode-model speaks protocol {PROTOCOL}"
            ));
        }
        None => {
            return Err(format!(
                "the request states no protocol; expected {PROTOCOL}"
            ));
        }
    }
    serde_json::from_value(value)
        .map_err(|problem| format!("the request cannot be read: {problem}"))
}

/// The word for a kind of failure, for a caller that reads the answer.
pub fn kind(error: &CallError) -> &'static str {
    match error {
        CallError::Config(_) => "configuration",
        CallError::Unreachable { .. } => "unreachable",
        CallError::Refused { .. } => "refused",
        CallError::Unreadable { .. } => "unreadable",
        CallError::NoAnswerInTime { .. } => "no_answer_in_time",
        CallError::WaitLimit { .. } => "wait_limit",
    }
}

/// The answer for a request that could not be used at all.
pub fn unusable(reason: &str) -> Value {
    json!({"protocol": PROTOCOL, "ok": false, "error": {"kind": "request", "reason": reason}})
}

/// The answer on standard output for the outcome of a call.
pub fn answer(outcome: &Result<Reply, CallError>) -> Value {
    match outcome {
        Ok(reply) => json!({"protocol": PROTOCOL, "ok": true, "reply": reply}),
        Err(error) => json!({
            "protocol": PROTOCOL,
            "ok": false,
            "error": {"kind": kind(error), "reason": error.reason()},
        }),
    }
}

/// The exit code for the outcome of a call. A configuration that cannot be
/// used ends like a command line that cannot be used.
pub fn exit_code(outcome: &Result<Reply, CallError>) -> u8 {
    match outcome {
        Ok(_) => EXIT_ANSWERED,
        Err(CallError::Config(_)) => EXIT_USAGE,
        Err(_) => EXIT_FAILED,
    }
}
