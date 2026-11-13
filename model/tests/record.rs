//! Requirement 9: a record of every call (task 1.4).

mod stub_endpoint;

use lesscode_core::regenerator::Endpoint;
use lesscode_model::record::record;
use lesscode_model::{CallRequest, send};
use std::path::PathBuf;
use stub_endpoint::{chat_reply, ok, start};

const KEY: &str = "test-key-0123456789-do-not-record";

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
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn request() -> CallRequest {
    CallRequest {
        purpose: "record test".into(),
        system: String::new(),
        user: "Say ok.".into(),
        max_output_tokens: 100,
        temperature: Some(0.5),
        structure: None,
    }
}

#[test]
fn a_successful_and_a_failed_call_each_leave_one_line_without_the_key() {
    let refusal = format!(r#"{{"error": {{"message": "no, and your key was {KEY}"}}}}"#);
    let stub = start(vec![ok(chat_reply("ok", "stop")), (403, "", refusal)]);
    let mut endpoint = Endpoint::new("openai", &format!("{}/v1", stub.address), "stub-model", KEY);
    endpoint.reasoning = Some("high".into());
    let file = Scratch::new("two-calls");

    for _ in 0..2 {
        let outcome = send(&endpoint, &request());
        record(Some(&file.0), &endpoint, &request(), &outcome, 1.26).unwrap();
    }

    let text = std::fs::read_to_string(&file.0).unwrap();
    assert!(!text.contains(KEY), "the key is recorded: {text}");
    let lines: Vec<serde_json::Value> = text
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(lines.len(), 2);

    let good = &lines[0];
    assert_eq!(good["ok"], true);
    assert_eq!(good["purpose"], "record test");
    assert_eq!(good["model"], "stub-model");
    assert!(good["host"].as_str().unwrap().starts_with("127.0.0.1:"));
    assert_eq!(
        (&good["input_tokens"], &good["output_tokens"]),
        (&11.into(), &7.into())
    );
    assert_eq!(
        (&good["reasoning_tokens"], &good["cached_input_tokens"]),
        (&5.into(), &4.into())
    );
    assert_eq!(good["sent"]["temperature"], 0.5);
    assert_eq!(good["sent"]["reasoning_level"], "high");
    assert_eq!(good["seconds"], 1.3);
    assert_eq!(good["stop"], "complete");
    assert!(good["time_utc"].as_str().unwrap().ends_with('Z'));
    assert!(good["waits"].as_array().unwrap().is_empty());

    let bad = &lines[1];
    assert_eq!(bad["ok"], false);
    assert!(
        bad["failure"].as_str().unwrap().contains("status 403"),
        "{bad}"
    );
    assert!(bad.get("output_tokens").is_none());
}

#[test]
fn without_a_path_nothing_is_written() {
    let stub = start(vec![ok(chat_reply("ok", "stop"))]);
    let endpoint = Endpoint::new("openai", &format!("{}/v1", stub.address), "stub-model", KEY);
    let outcome = send(&endpoint, &request());
    let before: Vec<_> = std::fs::read_dir(".")
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .collect();
    record(None, &endpoint, &request(), &outcome, 0.1).unwrap();
    let after: Vec<_> = std::fs::read_dir(".")
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .collect();
    assert_eq!(before, after);
}
