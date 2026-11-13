//! Requirement 4 (why a call failed) and the parts of requirement 1 about an
//! unusable endpoint file and a key that is never shown (task 1.3).

mod stub_endpoint;

use lesscode_core::regenerator::Endpoint;
use lesscode_model::notice::{failed_line, still_running_line, waiting_line};
use lesscode_model::{CallError, CallRequest, endpoint_from, send};
use std::path::PathBuf;
use stub_endpoint::start;

const KEY: &str = "test-key-0123456789-do-not-show";

fn request() -> CallRequest {
    CallRequest {
        purpose: "test".into(),
        system: String::new(),
        user: "Say ok.".into(),
        max_output_tokens: 100,
        temperature: None,
        structure: None,
    }
}

/// A file in the temp folder that is removed when the test ends.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str, text: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("lesscode-model-{name}-{}.env", std::process::id()));
        std::fs::write(&path, text).unwrap();
        Scratch(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn config(result: Result<Endpoint, CallError>) -> String {
    match result {
        Err(CallError::Config(what)) => what,
        Err(other) => panic!("a configuration failure was expected, got {other:?}"),
        Ok(_) => panic!("a configuration failure was expected, got an endpoint"),
    }
}

#[test]
fn a_missing_file_is_named() {
    let path = std::env::temp_dir().join("lesscode-model-no-such-file.env");
    let what = config(endpoint_from(Some(&path)));
    assert!(what.contains("lesscode-model-no-such-file.env"), "{what}");
}

#[test]
fn a_missing_entry_is_named_with_its_file() {
    let file = Scratch::new(
        "missing-entry",
        "LESSCODE_MODEL_API_STYLE=openai\nLESSCODE_MODEL_BASE_URL=http://127.0.0.1:1/v1\nLESSCODE_MODEL_API_KEY=k\n",
    );
    let what = config(endpoint_from(Some(&file.0)));
    assert!(what.contains("LESSCODE_MODEL_NAME"), "{what}");
    assert!(what.contains("missing-entry"), "{what}");
    assert!(!what.contains("LESSCODE_MODEL_BASE_URL"), "{what}");
}

#[test]
fn an_unknown_style_in_the_file_names_the_known_ones() {
    let file = Scratch::new(
        "unknown-style",
        "LESSCODE_MODEL_API_STYLE=telex\nLESSCODE_MODEL_BASE_URL=http://127.0.0.1:1/v1\nLESSCODE_MODEL_NAME=m\nLESSCODE_MODEL_API_KEY=k\n",
    );
    let what = config(endpoint_from(Some(&file.0)));
    assert!(
        what.contains("telex") && what.contains("openai, anthropic"),
        "{what}"
    );
}

#[test]
fn an_endpoint_that_cannot_be_reached_says_so() {
    // A port nothing listens on: bound once to learn a free one, then closed.
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let endpoint = Endpoint::new("openai", &format!("http://127.0.0.1:{port}/v1"), "m", KEY);
    let error = send(&endpoint, &request()).unwrap_err();
    assert!(matches!(error, CallError::Unreachable { .. }), "{error:?}");
    assert!(
        error.reason().contains("could not be reached"),
        "{}",
        error.reason()
    );
    assert!(error.reason().contains(&format!("127.0.0.1:{port}")));
}

#[test]
fn a_refusal_carries_the_endpoints_message_shortened_and_without_the_key() {
    let long = "x".repeat(2000);
    let body = format!(
        r#"{{"error": {{"message": "Incorrect key provided: {KEY}. The request   is not
            valid here. {long}"}}}}"#
    );
    let stub = start(vec![(401, "retry-after: 7\r\n", body)]);
    let endpoint = Endpoint::new("openai", &format!("{}/v1", stub.address), "m", KEY);
    let error = send(&endpoint, &request()).unwrap_err();
    let CallError::Refused {
        status,
        said,
        retry_after,
        ..
    } = &error
    else {
        panic!("a refusal was expected, got {error:?}");
    };
    assert_eq!(*status, 401);
    assert_eq!(retry_after.as_deref(), Some("7"));
    assert!(said.contains("The request is not valid here."), "{said}");
    assert_eq!(said.chars().count(), 300);
    let everything = format!(
        "{error:?} {} {}",
        error.reason(),
        failed_line(&error.reason())
    );
    assert!(!everything.contains(KEY), "the key is shown: {everything}");
    assert!(
        error.reason().contains("refused the call (status 401)"),
        "{}",
        error.reason()
    );
}

#[test]
fn an_answer_that_cannot_be_read_is_not_a_refusal() {
    let stub = start(vec![(200, "", "this is not a reply".to_owned())]);
    let endpoint = Endpoint::new("openai", &format!("{}/v1", stub.address), "m", KEY);
    let error = send(&endpoint, &request()).unwrap_err();
    assert!(matches!(error, CallError::Unreadable { .. }), "{error:?}");
    assert!(
        error.reason().contains("could not be read"),
        "{}",
        error.reason()
    );
}

#[test]
fn every_kind_of_failure_has_one_line_that_says_which_it_is() {
    let host = "h".to_owned();
    let lines = [
        CallError::NoAnswerInTime {
            host: host.clone(),
            seconds: 3600,
        }
        .reason(),
        CallError::WaitLimit {
            host,
            waited_seconds: 590,
            limit_seconds: 600,
            attempts: 6,
        }
        .reason(),
    ];
    assert!(
        lines[0].contains("did not answer within 3600 s"),
        "{}",
        lines[0]
    );
    assert!(
        lines[1].contains("waited 590 s in 6 attempts") && lines[1].contains("600 s"),
        "{}",
        lines[1]
    );
    assert!(lines.iter().all(|line| !line.contains('\n')));
}

#[test]
fn the_notices_name_the_host_and_the_times() {
    assert_eq!(
        waiting_line("h:1", 20, 10, 2),
        "[model] h:1 asks to come back later (attempt 2); waiting about 20 s, 10 s waited so far"
    );
    assert!(
        still_running_line("h:1", 90).contains("h:1")
            && still_running_line("h:1", 90).contains("90 s")
    );
    assert_eq!(failed_line("why"), "[model] the call failed: why");
}
