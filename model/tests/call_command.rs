//! The process protocol of `lesscode-model call`, through the binary against
//! a local stub endpoint (task 4.1; requirement 4.4).

mod stub_endpoint;

use serde_json::{Value, json};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use stub_endpoint::{chat_reply, ok, start};

const KEY: &str = "test-key-0123456789-do-not-show";

/// An endpoint file in the temp folder that is removed when the test ends.
struct EndpointFile(PathBuf);

impl EndpointFile {
    fn new(name: &str, address: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "lesscode-model-command-{name}-{}.env",
            std::process::id()
        ));
        let text = format!(
            "LESSCODE_MODEL_API_STYLE=openai\nLESSCODE_MODEL_BASE_URL={address}/v1\n\
             LESSCODE_MODEL_NAME=stub-model\nLESSCODE_MODEL_API_KEY={KEY}\n"
        );
        std::fs::write(&path, text).unwrap();
        EndpointFile(path)
    }
}

impl Drop for EndpointFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

struct Ran {
    code: Option<i32>,
    out: String,
    err: String,
}

impl Ran {
    /// Standard output is one JSON object and nothing else.
    fn answer(&self) -> Value {
        serde_json::from_str(&self.out).unwrap_or_else(|problem| {
            panic!("standard output is not JSON ({problem}): {}", self.out)
        })
    }

    fn shows_no_key(&self) {
        assert!(
            !self.out.contains(KEY) && !self.err.contains(KEY),
            "{}\n{}",
            self.out,
            self.err
        );
    }
}

/// Runs the binary with `args`, `input` on standard input, and no setting of
/// a model or a reasoning level from the environment of the test run.
fn run(args: &[&str], input: &str) -> Ran {
    let mut child = Command::new(env!("CARGO_BIN_EXE_lesscode-model"))
        .args(args)
        .env_remove("LESSCODE_MODEL_ENV")
        .env_remove("LESSCODE_MODEL_NAME")
        .env_remove("LESSCODE_MODEL_REASONING_EFFORT")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary starts");
    // A command line that cannot be used ends before it reads its input.
    let _ = child
        .stdin
        .take()
        .expect("standard input")
        .write_all(input.as_bytes());
    let output = child.wait_with_output().expect("the binary ends");
    Ran {
        code: output.status.code(),
        out: String::from_utf8(output.stdout).expect("standard output is text"),
        err: String::from_utf8(output.stderr).expect("standard error is text"),
    }
}

fn request() -> String {
    json!({
        "protocol": 1,
        "purpose": "test",
        "system": "Answer briefly.",
        "user": "Say ok.",
        "max_output_tokens": 100,
    })
    .to_string()
}

#[test]
fn an_answered_call_is_one_reply_on_standard_output_and_exit_code_0() {
    let stub = start(vec![ok(chat_reply("ok", "stop"))]);
    let file = EndpointFile::new("answered", &stub.address);
    let ran = run(
        &["call", "--endpoint-env", file.0.to_str().unwrap()],
        &request(),
    );

    assert_eq!(ran.code, Some(0), "{}", ran.err);
    let answer = ran.answer();
    assert_eq!(answer["protocol"], 1);
    assert_eq!(answer["ok"], true);
    assert_eq!(answer["reply"]["text"], "ok");
    assert_eq!(answer["reply"]["stop"], "complete");
    assert_eq!(answer["reply"]["output_tokens"], 7);
    assert_eq!(answer["reply"]["sent"]["model"], "stub-model");
    assert_eq!(answer["reply"]["sent"]["max_output_tokens"], 100);
    assert_eq!(
        ran.err, "",
        "an answered call says nothing on standard error"
    );
    ran.shows_no_key();

    let seen = stub.seen();
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].json()["messages"][1]["content"], "Say ok.");
    assert!(seen[0].json().get("temperature").is_none());
}

#[test]
fn a_failed_call_states_its_reason_on_standard_error_and_ends_with_exit_code_3() {
    let body = format!(r#"{{"error": {{"message": "The key {KEY} may not use this model."}}}}"#);
    let stub = start(vec![(403, "", body)]);
    let file = EndpointFile::new("failed", &stub.address);
    let ran = run(
        &["call", "--endpoint-env", file.0.to_str().unwrap()],
        &request(),
    );

    assert_eq!(ran.code, Some(3), "{}", ran.err);
    let answer = ran.answer();
    assert_eq!(answer["protocol"], 1);
    assert_eq!(answer["ok"], false);
    assert_eq!(answer["error"]["kind"], "refused");
    let reason = answer["error"]["reason"].as_str().unwrap();
    assert!(reason.contains("refused the call (status 403)"), "{reason}");
    assert!(reason.contains("may not use this model"), "{reason}");
    assert!(ran.err.contains(reason), "{}", ran.err);
    assert_eq!(
        ran.err.lines().count(),
        1,
        "the reason is said once: {}",
        ran.err
    );
    ran.shows_no_key();
    assert_eq!(stub.seen().len(), 1);
}

#[test]
fn the_record_and_the_limits_named_on_the_command_line_are_used() {
    let stub = start(vec![(
        429,
        "retry-after: 5\r\n",
        r#"{"error": {"message": "slow down"}}"#.to_owned(),
    )]);
    let file = EndpointFile::new("limits", &stub.address);
    let record = std::env::temp_dir().join(format!(
        "lesscode-model-command-{}.jsonl",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&record);
    let ran = run(
        &[
            "call",
            "--endpoint-env",
            file.0.to_str().unwrap(),
            "--record",
            record.to_str().unwrap(),
            "--max-wait",
            "2",
        ],
        &request(),
    );
    let lines = std::fs::read_to_string(&record).unwrap_or_default();
    let _ = std::fs::remove_file(&record);

    assert_eq!(ran.code, Some(3), "{}", ran.err);
    assert_eq!(ran.answer()["error"]["kind"], "wait_limit");
    assert!(ran.err.contains("the limit is 2 s"), "{}", ran.err);
    assert_eq!(stub.seen().len(), 1, "a wait past the limit is not made");
    assert_eq!(lines.lines().count(), 1, "{lines}");
    let line: Value = serde_json::from_str(lines.trim()).unwrap();
    assert_eq!(line["ok"], false);
    assert_eq!(line["purpose"], "test");
    assert!(!lines.contains(KEY));
}

#[test]
fn a_request_that_cannot_be_read_ends_with_exit_code_2_and_nothing_is_called() {
    let stub = start(vec![ok(chat_reply("ok", "stop"))]);
    let file = EndpointFile::new("unreadable", &stub.address);
    let args = ["call", "--endpoint-env", file.0.to_str().unwrap()];
    let other_protocol = request().replace(r#""protocol":1"#, r#""protocol":2"#);
    let no_protocol = request().replace(r#""protocol":1,"#, "");
    let no_user = json!({"protocol": 1, "purpose": "test", "system": "", "max_output_tokens": 100})
        .to_string();

    for (input, says) in [
        ("this is not a request", "not JSON"),
        (other_protocol.as_str(), "protocol 2"),
        (no_protocol.as_str(), "no protocol"),
        (no_user.as_str(), "user"),
    ] {
        let ran = run(&args, input);
        assert_eq!(ran.code, Some(2), "{input}: {}", ran.err);
        assert!(ran.err.contains(says), "{input}: {}", ran.err);
        let answer = ran.answer();
        assert_eq!(answer["ok"], false, "{input}");
        assert_eq!(answer["error"]["kind"], "request", "{input}");
        assert!(
            answer["error"]["reason"].as_str().unwrap().contains(says),
            "{input}"
        );
    }
    assert_eq!(stub.seen().len(), 0);
}

#[test]
fn an_unusable_command_line_or_endpoint_file_ends_with_exit_code_2() {
    for args in [
        &[][..],
        &["telex"][..],
        &["call", "--nonsense", "1"][..],
        &["call", "--max-wait"][..],
        &["call", "--max-wait", "soon"][..],
    ] {
        let ran = run(args, &request());
        assert_eq!(ran.code, Some(2), "{args:?}: {}", ran.err);
        assert!(
            ran.err.contains("usage: lesscode-model call"),
            "{args:?}: {}",
            ran.err
        );
        assert_eq!(ran.out, "", "{args:?}");
    }

    let missing = std::env::temp_dir().join("lesscode-model-command-no-such-file.env");
    let ran = run(
        &["call", "--endpoint-env", missing.to_str().unwrap()],
        &request(),
    );
    assert_eq!(ran.code, Some(2), "{}", ran.err);
    assert_eq!(ran.answer()["error"]["kind"], "configuration");
    assert!(
        ran.err.contains("lesscode-model-command-no-such-file.env"),
        "{}",
        ran.err
    );
}
