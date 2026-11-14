//! Requirements 3.5 and 4.3: an answer that takes long (task 2.2). The user
//! is told at intervals that the call is still running, and a limit ends a
//! call whose answer does not come. Time is injected: all tests but the last
//! do not sleep.
//!
//! How long an answer is expected to take depends on the answers this
//! process has had, so each test needs a process of its own, as nextest
//! gives it (docs/testing.md).

mod stub_endpoint;

use lesscode_core::regenerator::Endpoint;
use lesscode_model::notice::{failed_line, still_running_line};
use lesscode_model::{
    Attempt, CallError, CallRequest, Limits, Notice, Reply, Time, WaitPolicy, call_with,
};
use std::cell::{Cell, RefCell};
use std::path::Path;
use std::time::{Duration, Instant};
use stub_endpoint::{Stub, chat_reply, ok, start};

/// Takes the place of the terminal and of the machine's time. The answer of
/// a request is `left` away on this clock: waiting for it moves the clock
/// and nothing else, and only once the clock is there is the stub's answer
/// read. A request given up before that never reaches the stub.
struct Slow {
    now: Cell<Instant>,
    left: Cell<Duration>,
    events: RefCell<Vec<String>>,
}

impl Notice for Slow {
    fn waiting(&self, host: &str, _expected: u64, _waited: u64, _attempt: u32) {
        self.events.borrow_mut().push(format!("waiting for {host}"));
    }

    fn still_running(&self, host: &str, running_seconds: u64) {
        self.events
            .borrow_mut()
            .push(still_running_line(host, running_seconds));
    }

    fn failed(&self, reason: &str) {
        self.events.borrow_mut().push(failed_line(reason));
    }
}

impl Time for Slow {
    fn now(&self) -> Instant {
        self.now.get()
    }

    fn sleep(&self, wait: Duration) {
        self.now.set(self.now.get() + wait);
    }

    fn answer_within(
        &self,
        attempt: &mut Attempt,
        step: Duration,
    ) -> Option<Result<Reply, CallError>> {
        let left = self.left.get();
        if left > step {
            self.left.set(left - step);
            self.now.set(self.now.get() + step);
            return None;
        }
        self.left.set(Duration::ZERO);
        self.now.set(self.now.get() + left);
        attempt.answer_within(Duration::from_secs(5))
    }
}

fn request() -> CallRequest {
    CallRequest {
        purpose: "slow answer test".into(),
        system: String::new(),
        user: "Say ok.".into(),
        max_output_tokens: 100,
        temperature: None,
        structure: None,
    }
}

fn host_of(stub: &Stub) -> String {
    stub.address.trim_start_matches("http://").to_owned()
}

/// One call whose answer takes `answer_takes` seconds, with `answer_seconds`
/// as the limit for it. Returns the outcome and what the user was told.
fn run(
    stub: &Stub,
    answer_takes: u64,
    answer_seconds: u64,
    record_to: Option<&Path>,
) -> (Result<Reply, CallError>, Vec<String>) {
    let started = Instant::now();
    let endpoint = Endpoint::new(
        "openai",
        &format!("{}/v1", stub.address),
        "stub-model",
        "not-a-key",
    );
    let slow = Slow {
        now: Cell::new(started),
        left: Cell::new(Duration::from_secs(answer_takes)),
        events: RefCell::new(Vec::new()),
    };
    let limits = Limits {
        wait: WaitPolicy::default(),
        answer_seconds,
    };
    let result = call_with(&endpoint, &request(), &limits, &slow, record_to, &slow);
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "the test slept: {:?}",
        started.elapsed()
    );
    (result, slow.events.into_inner())
}

#[test]
fn with_no_earlier_answer_the_user_is_told_after_a_minute_and_then_every_half_minute() {
    let stub = start(vec![ok(chat_reply("ok", "stop"))]);
    let host = host_of(&stub);
    let (result, events) = run(&stub, 130, 3600, None);
    let reply = result.expect("the answer comes in the end");
    assert_eq!(reply.text, "ok");
    assert_eq!(reply.seconds, 130.0);
    assert_eq!(
        events,
        [
            still_running_line(&host, 60),
            still_running_line(&host, 90),
            still_running_line(&host, 120),
        ]
    );
    assert!(events[0].contains(host.as_str()) && events[0].contains("60 s so far"));
    assert_eq!(stub.seen().len(), 1, "the request is sent once");
}

#[test]
fn an_answer_within_the_expected_time_is_not_announced() {
    let stub = start(vec![ok(chat_reply("ok", "stop"))]);
    let (result, events) = run(&stub, 59, 3600, None);
    assert_eq!(result.unwrap().seconds, 59.0);
    assert_eq!(events, [] as [String; 0]);
}

#[test]
fn the_expected_time_is_twice_the_median_of_the_earlier_answers() {
    let stub = start(vec![ok(chat_reply("ok", "stop"))]);
    let host = host_of(&stub);
    // Two answers of four and six seconds: the median is five, so the third
    // is expected within ten.
    for seconds in [4, 6] {
        let (result, events) = run(&stub, seconds, 3600, None);
        result.unwrap();
        assert_eq!(events, [] as [String; 0]);
    }
    let (result, events) = run(&stub, 45, 3600, None);
    assert_eq!(result.unwrap().seconds, 45.0);
    assert_eq!(
        events,
        [still_running_line(&host, 10), still_running_line(&host, 40)]
    );
}

#[test]
fn the_limit_ends_a_call_whose_answer_does_not_come() {
    let stub = start(vec![ok(chat_reply("ok", "stop"))]);
    let host = host_of(&stub);
    let path = std::env::temp_dir().join(format!(
        "lesscode-model-no-answer-{}.jsonl",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    let (result, events) = run(&stub, 500, 100, Some(&path));
    let error = result.expect_err("the limit ends the call");
    assert_eq!(
        error,
        CallError::NoAnswerInTime {
            host: host.clone(),
            seconds: 100,
        }
    );
    let reason = error.reason();
    assert!(reason.contains("did not answer within 100 s"), "{reason}");
    assert_eq!(
        events,
        [
            still_running_line(&host, 60),
            still_running_line(&host, 90),
            failed_line(&reason),
        ]
    );
    let record = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_file(&path);
    let lines: Vec<serde_json::Value> = record
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0]["ok"], false);
    assert_eq!(lines[0]["failure"], reason);
    assert_eq!(lines[0]["seconds"], 100.0);
}

#[test]
fn an_answer_that_fails_is_not_counted_among_the_answers() {
    let refused = (
        400,
        "",
        r#"{"error":{"message":"not valid here"}}"#.to_owned(),
    );
    let stub = start(vec![refused, ok(chat_reply("ok", "stop"))]);
    let host = host_of(&stub);
    let (result, _) = run(&stub, 2, 3600, None);
    assert!(matches!(
        result,
        Err(CallError::Refused { status: 400, .. })
    ));
    // Had the refusal counted, the next answer would be expected within
    // four seconds; it is still the minute of a process without answers.
    let (result, events) = run(&stub, 70, 3600, None);
    result.unwrap();
    assert_eq!(events, [still_running_line(&host, 60)]);
}

/// The one test here with the machine's time: it waits a fifth of a second.
#[test]
fn a_request_without_an_answer_stays_on_its_way_between_steps() {
    let silent = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = Endpoint::new(
        "openai",
        &format!("http://{}/v1", silent.local_addr().unwrap()),
        "stub-model",
        "not-a-key",
    );
    let mut attempt = lesscode_model::begin(&endpoint, &request()).unwrap();
    let step = Duration::from_millis(100);
    assert!(attempt.answer_within(step).is_none());
    assert!(attempt.answer_within(step).is_none());
    let (_connection, _) = silent.accept().expect("the request was sent");
    silent.set_nonblocking(true).unwrap();
    assert!(silent.accept().is_err(), "and it was sent once");
}
