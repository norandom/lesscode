//! The call a caller makes: one request at a time, with a refusal that asks
//! to come back later waited out. Before each wait the user is told for how
//! long; the waits of one call have a limit; an answer that takes long is
//! announced at intervals and has a limit too; every call leaves one line in
//! the record.
//! Nothing here names an endpoint: the wait is what the endpoint states, in
//! a header or in its message, else a first step, and never less than twice
//! the wait that has just failed (R106).

use crate::client::{Attempt, begin, host_of};
use crate::error::CallError;
use crate::notice::Notice;
use crate::record::record;
use crate::request::{CallRequest, Reply, Wait};
use lesscode_core::regenerator::Endpoint;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, PoisonError};
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

/// An answer may take this long when no call of this process has been
/// answered yet; after that the user is told it is still running.
pub const EXPECTED_AT_FIRST_SECONDS: f64 = 60.0;

/// The interval of the "still running" notices.
pub const STILL_RUNNING_EVERY_SECONDS: f64 = 30.0;

/// The limits of one call.
#[derive(Debug, Clone, PartialEq)]
pub struct Limits {
    pub wait: WaitPolicy,
    /// One request may wait this long for its answer.
    pub answer_seconds: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            wait: WaitPolicy::default(),
            answer_seconds: 3600,
        }
    }
}

/// Where a call takes its time from. A test gives one that does not sleep.
pub trait Time {
    fn now(&self) -> Instant;
    fn sleep(&self, wait: Duration);
    /// Waits for the answer of a request on its way, for at most `step`.
    fn answer_within(
        &self,
        attempt: &mut Attempt,
        step: Duration,
    ) -> Option<Result<Reply, CallError>> {
        attempt.answer_within(step)
    }
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

/// How long the answered requests of this process took, in seconds.
static ANSWERED: Mutex<Vec<f64>> = Mutex::new(Vec::new());

/// How long an answer may take before the user is told that it is still
/// running: twice the median of the answers so far.
pub fn expected_seconds(answered: &[f64]) -> f64 {
    if answered.is_empty() {
        return EXPECTED_AT_FIRST_SECONDS;
    }
    let mut sorted = answered.to_vec();
    sorted.sort_by(f64::total_cmp);
    // The two middle values, which are one and the same for an odd count:
    // their sum is twice the median.
    let middle = sorted.len() / 2;
    sorted[middle] + sorted[sorted.len() - 1 - middle]
}

fn answered() -> MutexGuard<'static, Vec<f64>> {
    ANSWERED.lock().unwrap_or_else(PoisonError::into_inner)
}

/// One request and its answer. Past the expected time the user is told at
/// intervals that it is still running; past the limit the request is given
/// up.
fn answer(
    endpoint: &Endpoint,
    request: &CallRequest,
    answer_seconds: u64,
    notice: &dyn Notice,
    time: &dyn Time,
) -> Result<Reply, CallError> {
    let mut attempt = begin(endpoint, request)?;
    let host = host_of(endpoint);
    let limit = answer_seconds as f64;
    let mut quiet_until = expected_seconds(&answered());
    let started = time.now();
    loop {
        let running = time.now().duration_since(started).as_secs_f64();
        if running >= limit {
            return Err(CallError::NoAnswerInTime {
                host,
                seconds: answer_seconds,
            });
        }
        if running >= quiet_until {
            notice.still_running(&host, running.round() as u64);
            quiet_until = running + STILL_RUNNING_EVERY_SECONDS;
        }
        let step = Duration::from_secs_f64(quiet_until.min(limit) - running);
        if let Some(answer) = time.answer_within(&mut attempt, step) {
            if answer.is_ok() {
                answered().push(time.now().duration_since(started).as_secs_f64());
            }
            return answer;
        }
    }
}

/// One request at a time until the endpoint accepts one, the limit is
/// reached, or an attempt fails for a reason that waiting does not mend.
fn attempts(
    endpoint: &Endpoint,
    request: &CallRequest,
    limits: &Limits,
    notice: &dyn Notice,
    time: &dyn Time,
) -> Result<Reply, CallError> {
    let policy = &limits.wait;
    let started = time.now();
    let (mut waits, mut previous) = (Vec::new(), 0.0_f64);
    loop {
        let sent = answer(endpoint, request, limits.answer_seconds, notice, time);
        let (host, said, retry_after) = match sent {
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
/// waited out, each wait announced before it starts. A request whose answer
/// takes longer than expected is announced at intervals and given up at
/// `limits.answer_seconds`. The reply's seconds
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
    let outcome = attempts(endpoint, request, limits, notice, time);
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
    use super::{Limits, expected_seconds, stated_wait};

    #[test]
    fn the_limits_are_ten_minutes_of_waiting_and_sixty_for_an_answer() {
        let limits = Limits::default();
        assert_eq!(
            (limits.wait.total_seconds, limits.answer_seconds),
            (600, 3600)
        );
    }

    #[test]
    fn an_answer_is_expected_within_twice_the_median_of_the_answers_so_far() {
        assert_eq!(expected_seconds(&[]), 60.0);
        assert_eq!(expected_seconds(&[7.0]), 14.0);
        assert_eq!(expected_seconds(&[6.0, 4.0]), 10.0);
        assert_eq!(expected_seconds(&[100.0, 1.0, 3.0]), 6.0);
    }

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
