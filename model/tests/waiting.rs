//! Requirement 3: waiting with an estimate (task 2.1), the five cases of
//! R106 against the stub. Time is injected: no test sleeps.

mod stub_endpoint;

use lesscode_core::regenerator::Endpoint;
use lesscode_model::notice::{failed_line, waiting_line};
use lesscode_model::{
    CallError, CallRequest, Limits, Notice, Reply, Time, Wait, WaitPolicy, call_with,
};
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use stub_endpoint::{Answer, Stub, chat_reply, ok, start};

const SAID_ONE: &str = r#"{"error":{"message":"Upstream rate limit persisted; retry after 1 seconds","type":"rate_limit_error"}}"#;
const SAID_NOTHING: &str = r#"{"error":{"message":"slow down"}}"#;
const BAD_REQUEST: &str = r#"{"error":{"message":"the request is not valid here"}}"#;

fn refusal(status: u16, headers: &'static str, body: &str) -> Answer {
    (status, headers, body.to_owned())
}

fn request() -> CallRequest {
    CallRequest {
        purpose: "waiting test".into(),
        system: String::new(),
        user: "Say ok.".into(),
        max_output_tokens: 100,
        temperature: None,
        structure: None,
    }
}

/// Takes the place of the terminal and of the machine's time: it keeps what
/// the user would have been told and every pause in the order they came,
/// and a pause only moves its own clock. At each pause it notes how many
/// requests the stub has answered.
struct Watch<'a> {
    stub: &'a Stub,
    now: Cell<Instant>,
    events: RefCell<Vec<String>>,
}

impl Notice for Watch<'_> {
    fn waiting(&self, host: &str, expected_seconds: u64, waited_so_far: u64, attempt: u32) {
        self.events
            .borrow_mut()
            .push(waiting_line(host, expected_seconds, waited_so_far, attempt));
    }

    fn still_running(&self, _host: &str, _running_seconds: u64) {}

    fn failed(&self, reason: &str) {
        self.events.borrow_mut().push(failed_line(reason));
    }
}

impl Time for Watch<'_> {
    fn now(&self) -> Instant {
        self.now.get()
    }

    fn sleep(&self, wait: Duration) {
        self.now.set(self.now.get() + wait);
        self.events.borrow_mut().push(format!(
            "pause of {} s after {} answered requests",
            wait.as_secs_f64(),
            self.stub.seen().len()
        ));
    }
}

struct Outcome {
    result: Result<Reply, CallError>,
    events: Vec<String>,
    requests: usize,
    host: String,
}

/// One call against a stub that answers from `script`, with at most
/// `total_seconds` of waiting.
fn run(script: Vec<Answer>, total_seconds: u64, record_to: Option<&Path>) -> Outcome {
    let started = Instant::now();
    let stub = start(script);
    let endpoint = Endpoint::new(
        "openai",
        &format!("{}/v1", stub.address),
        "stub-model",
        "not-a-key",
    );
    let watch = Watch {
        stub: &stub,
        now: Cell::new(started),
        events: RefCell::new(Vec::new()),
    };
    let limits = Limits {
        wait: WaitPolicy { total_seconds },
    };
    let result = call_with(&endpoint, &request(), &limits, &watch, record_to, &watch);
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "the test slept: {:?}",
        started.elapsed()
    );
    Outcome {
        result,
        events: watch.events.into_inner(),
        requests: stub.seen().len(),
        host: stub.address.trim_start_matches("http://").to_owned(),
    }
}

fn waits(outcome: &Outcome) -> Vec<(f64, bool)> {
    let reply = outcome.result.as_ref().expect("the call is answered");
    assert_eq!(reply.text, "ok");
    reply
        .waits
        .iter()
        .map(
            |Wait {
                 seconds,
                 stated_by_endpoint,
             }| (*seconds, *stated_by_endpoint),
        )
        .collect()
}

#[test]
fn a_wait_stated_in_a_header_is_made() {
    let outcome = run(
        vec![
            refusal(429, "retry-after: 2\r\n", SAID_NOTHING),
            ok(chat_reply("ok", "stop")),
        ],
        600,
        None,
    );
    assert_eq!(waits(&outcome), [(2.0, true)]);
    assert_eq!(outcome.requests, 2);
}

#[test]
fn a_wait_stated_in_the_message_is_made() {
    let outcome = run(
        vec![refusal(429, "", SAID_ONE), ok(chat_reply("ok", "stop"))],
        600,
        None,
    );
    assert_eq!(waits(&outcome), [(1.0, true)]);
    assert_eq!(outcome.requests, 2);
}

#[test]
fn with_no_wait_stated_the_first_step_is_made_and_then_doubled() {
    let outcome = run(
        vec![
            refusal(429, "", SAID_NOTHING),
            refusal(503, "", SAID_NOTHING),
            ok(chat_reply("ok", "stop")),
        ],
        600,
        None,
    );
    assert_eq!(waits(&outcome), [(10.0, false), (20.0, false)]);
    let reply = outcome.result.as_ref().unwrap();
    assert_eq!(reply.seconds, 30.0, "the reply's seconds include the waits");
    // Before each wait the user is told the host and the estimate, and at
    // each pause every request sent so far has been answered: one at a time.
    let host = &outcome.host;
    assert_eq!(
        outcome.events,
        [
            waiting_line(host, 10, 0, 1),
            "pause of 10 s after 1 answered requests".to_owned(),
            waiting_line(host, 20, 10, 2),
            "pause of 20 s after 2 answered requests".to_owned(),
        ]
    );
    assert!(outcome.events[0].contains(host.as_str()) && outcome.events[0].contains("10 s"));
    assert_eq!(outcome.requests, 3);
}

#[test]
fn a_stated_wait_of_nothing_still_pauses() {
    let outcome = run(
        vec![
            refusal(429, "retry-after: 0\r\n", SAID_NOTHING),
            refusal(429, "retry-after: 0\r\n", SAID_NOTHING),
            ok(chat_reply("ok", "stop")),
        ],
        600,
        None,
    );
    assert_eq!(waits(&outcome), [(1.0, true), (2.0, true)]);
}

#[test]
fn refused_for_good_the_limit_ends_the_call_with_its_report() {
    // The endpoint states one second every time; the wait made doubles, and
    // the third would pass the limit of six seconds.
    let outcome = run(vec![refusal(429, "", SAID_ONE)], 6, None);
    let host = outcome.host.clone();
    let error = outcome.result.expect_err("the limit ends the call");
    assert_eq!(
        error,
        CallError::WaitLimit {
            host: host.clone(),
            waited_seconds: 3,
            limit_seconds: 6,
            attempts: 3,
        }
    );
    let reason = error.reason();
    assert!(
        reason.contains("waited 3 s") && reason.contains("limit is 6 s"),
        "{reason}"
    );
    assert_eq!(
        outcome.events,
        [
            waiting_line(&host, 1, 0, 1),
            "pause of 1 s after 1 answered requests".to_owned(),
            waiting_line(&host, 2, 1, 2),
            "pause of 2 s after 2 answered requests".to_owned(),
            failed_line(&reason),
        ]
    );
    assert_eq!(outcome.requests, 3);
}

#[test]
fn refused_for_another_reason_is_not_waited_out() {
    let outcome = run(vec![refusal(400, "", BAD_REQUEST)], 600, None);
    let error = outcome
        .result
        .expect_err("a refusal that waiting does not mend");
    assert!(
        matches!(error, CallError::Refused { status: 400, .. }),
        "{error:?}"
    );
    assert_eq!(outcome.events, [failed_line(&error.reason())]);
    assert_eq!(outcome.requests, 1);
}

/// A path in the temp folder; the file is removed when the test ends.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "lesscode-model-{name}-{}.jsonl",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        Scratch(path)
    }

    fn lines(&self) -> Vec<serde_json::Value> {
        std::fs::read_to_string(&self.0)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[test]
fn the_waits_are_in_the_reply_and_in_the_one_line_of_the_record() {
    let file = Scratch::new("waited");
    let outcome = run(
        vec![
            refusal(429, "retry-after: 2\r\n", SAID_NOTHING),
            refusal(429, "", SAID_NOTHING),
            ok(chat_reply("ok", "stop")),
        ],
        600,
        Some(&file.0),
    );
    // The second refusal states nothing: the first step, ten seconds, is
    // more than twice the wait that failed.
    assert_eq!(waits(&outcome), [(2.0, true), (10.0, false)]);
    let lines = file.lines();
    assert_eq!(lines.len(), 1, "one line for the call, not one per attempt");
    assert_eq!(lines[0]["ok"], true);
    assert_eq!(lines[0]["seconds"], 12.0);
    assert_eq!(
        lines[0]["waits"],
        serde_json::json!([
            {"seconds": 2.0, "stated_by_endpoint": true},
            {"seconds": 10.0, "stated_by_endpoint": false},
        ])
    );
}

#[test]
fn a_call_ended_by_the_limit_leaves_one_line_with_the_report() {
    let file = Scratch::new("limit");
    let outcome = run(vec![refusal(503, "", SAID_ONE)], 6, Some(&file.0));
    assert!(outcome.result.is_err());
    let lines = file.lines();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0]["ok"], false);
    let failure = lines[0]["failure"].as_str().unwrap();
    assert!(failure.contains("in 3 attempts"), "{failure}");
}

#[test]
fn a_record_that_cannot_be_written_fails_the_call_and_names_the_path() {
    let nowhere = std::env::temp_dir()
        .join("lesscode-model-no-such-folder")
        .join("calls.jsonl");
    let outcome = run(vec![ok(chat_reply("ok", "stop"))], 600, Some(&nowhere));
    match outcome.result {
        Err(CallError::Config(what)) => assert!(what.contains("calls.jsonl"), "{what}"),
        other => panic!("a configuration failure was expected, got {other:?}"),
    }
}
