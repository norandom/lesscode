//! The call a caller makes: `send`, with a refusal that asks to come back
//! later waited out. Before each wait the user is told for how long; the
//! waits of one call have a limit; every call leaves one line in the record.
//! Nothing here names an endpoint: the wait is what the endpoint states, in
//! a header or in its message, else a first step, and never less than twice
//! the wait that has just failed (R106).

use crate::client::send;
use crate::error::CallError;
use crate::notice::Notice;
use crate::record::record;
use crate::request::{CallRequest, Reply, Wait};
use lesscode_core::regenerator::Endpoint;
use std::path::Path;
use std::time::{Duration, Instant};

/// The first wait when the endpoint states none.
pub const FIRST_STEP_SECONDS: f64 = 10.0;

/// No wait is shorter than this, so that a stated wait of nothing does not
/// turn into requests without a pause.
pub const SHORTEST_WAIT_SECONDS: f64 = 1.0;

#[derive(Debug, Clone, PartialEq)]
pub struct WaitPolicy {
    /// Refused attempts and the pauses between them may take this long in
    /// one call.
    pub total_seconds: u64,
}

impl Default for WaitPolicy {
    fn default() -> Self {
        WaitPolicy { total_seconds: 600 }
    }
}

/// The limits of one call.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Limits {
    pub wait: WaitPolicy,
}

/// Where a call takes its time from. A test gives one that does not sleep.
pub trait Time {
    fn now(&self) -> Instant;
    fn sleep(&self, wait: Duration);
}

/// The time of the machine.
pub struct Clock;

impl Time for Clock {
    fn now(&self) -> Instant {
        Instant::now()
    }

    fn sleep(&self, wait: Duration) {
        std::thread::sleep(wait);
    }
}

/// The seconds an endpoint asks for: in its header, else after the words
/// "retry after" in its message. A header that holds a date states none.
pub fn stated_wait(retry_after: Option<&str>, said: &str) -> Option<f64> {
    let in_header = retry_after
        .and_then(|value| value.trim().parse::<f64>().ok())
        .filter(|seconds| seconds.is_finite() && *seconds >= 0.0);
    in_header.or_else(|| {
        let lower = said.to_lowercase();
        let rest = lower.split("retry after ").nth(1)?;
        let number: String = rest
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        number.parse().ok()
    })
}

/// One request at a time until the endpoint accepts one, the limit is
/// reached, or an attempt fails for a reason that waiting does not mend.
fn attempts(
    endpoint: &Endpoint,
    request: &CallRequest,
    policy: &WaitPolicy,
    notice: &dyn Notice,
    time: &dyn Time,
) -> Result<Reply, CallError> {
    let started = time.now();
    let (mut waits, mut previous) = (Vec::new(), 0.0_f64);
    loop {
        let (host, said, retry_after) = match send(endpoint, request) {
            Ok(reply) => return Ok(Reply { waits, ..reply }),
            Err(CallError::Refused {
                host,
                status: 429 | 503,
                said,
                retry_after,
            }) => (host, said, retry_after),
            Err(other) => return Err(other),
        };
        let stated = stated_wait(retry_after.as_deref(), &said);
        let next = stated
            .unwrap_or(FIRST_STEP_SECONDS)
            .max(previous * 2.0)
            .max(SHORTEST_WAIT_SECONDS);
        let spent = time.now().duration_since(started).as_secs_f64();
        let attempt = waits.len() as u32 + 1;
        if spent + next > policy.total_seconds as f64 {
            return Err(CallError::WaitLimit {
                host,
                waited_seconds: spent.round() as u64,
                limit_seconds: policy.total_seconds,
                attempts: attempt,
            });
        }
        notice.waiting(&host, next.ceil() as u64, spent.round() as u64, attempt);
        time.sleep(Duration::from_secs_f64(next));
        waits.push(Wait {
            seconds: next,
            stated_by_endpoint: stated.is_some(),
        });
        previous = next;
    }
}

/// One call with the time of the machine. See `call_with`.
pub fn call(
    endpoint: &Endpoint,
    request: &CallRequest,
    limits: &Limits,
    notice: &dyn Notice,
    record_to: Option<&Path>,
) -> Result<Reply, CallError> {
    call_with(endpoint, request, limits, notice, record_to, &Clock)
}

/// One call: refusals with "too many requests" or "service unavailable" are
/// waited out, each wait announced before it starts. The reply's seconds
/// include the waits. A failure is told to `notice` once. With a path, the
/// call leaves exactly one line there, whether it succeeded or not; a line
/// that cannot be written fails the call.
pub fn call_with(
    endpoint: &Endpoint,
    request: &CallRequest,
    limits: &Limits,
    notice: &dyn Notice,
    record_to: Option<&Path>,
    time: &dyn Time,
) -> Result<Reply, CallError> {
    let started = time.now();
    let outcome = attempts(endpoint, request, &limits.wait, notice, time);
    let seconds = time.now().duration_since(started).as_secs_f64();
    let outcome = outcome.map(|reply| Reply { seconds, ..reply });
    let written = record(record_to, endpoint, request, &outcome, seconds);
    let outcome = outcome.and_then(|reply| match written {
        Ok(()) => Ok(reply),
        Err(problem) => Err(CallError::Config(format!(
            "the record of calls{} could not be written: {problem}",
            record_to.map_or(String::new(), |path| format!(" {}", path.display()))
        ))),
    });
    if let Err(error) = &outcome {
        notice.failed(&error.reason());
    }
    outcome
}

#[cfg(test)]
mod tests {
    use super::stated_wait;

    #[test]
    fn the_header_comes_before_the_message_and_a_date_states_nothing() {
        assert_eq!(stated_wait(Some(" 2 "), "retry after 7 seconds"), Some(2.0));
        assert_eq!(
            stated_wait(None, "Please Retry After 1.5 seconds"),
            Some(1.5)
        );
        assert_eq!(
            stated_wait(Some("Fri, 13 Nov 2026 15:00:00 GMT"), "slow down"),
            None
        );
        assert_eq!(stated_wait(Some("-3"), "slow down"), None);
        assert_eq!(stated_wait(Some("inf"), "retry after 4"), Some(4.0));
        assert_eq!(stated_wait(None, "retry after a while"), None);
    }
}
