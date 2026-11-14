//! Requirement 5: `lesscode-model check`, through the binary against a local
//! stub endpoint (task 4.2).

mod binary;
mod stub_endpoint;

use binary::{EndpointFile, KEY, Ran, run_in};
use serde_json::json;
use std::path::PathBuf;
use stub_endpoint::{Answer, chat_reply, ok, start};

/// An empty directory to run the command in, removed when the test ends.
struct Project(PathBuf);

impl Project {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "lesscode-model-project-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        Project(path)
    }

    fn files(&self) -> usize {
        std::fs::read_dir(&self.0).unwrap().count()
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// An answer to the task with `content` as the module, for `path`.
fn answer_with(path: &str, content: &str) -> String {
    json!({
        "files": [{"path": path, "content": content}],
        "structure": {"pure": ["prices.py::total_price"], "shell": []},
        "change_log": "the three branches did the same",
    })
    .to_string()
}

fn good() -> String {
    answer_with(
        "src/prices.py",
        "def total_price(items):\n    return sum(item[\"price\"] * item[\"count\"] for item in items)\n",
    )
}

/// Runs `check` against a stub with `script`, in an empty directory.
fn check(name: &str, script: Vec<Answer>) -> (Ran, Vec<stub_endpoint::Seen>, usize) {
    let stub = start(script);
    let file = EndpointFile::new(name, &stub.address);
    let project = Project::new(name);
    let ran = run_in(
        Some(&project.0),
        &["check", "--endpoint-env", file.path()],
        "",
    );
    ran.shows_no_key();
    (ran, stub.seen(), project.files())
}

#[test]
fn an_answer_that_passes_the_checks_says_the_endpoint_can_be_used() {
    let (ran, seen, files) = check("usable", vec![ok(chat_reply(&good(), "stop"))]);

    assert_eq!(ran.code, Some(0), "{}\n{}", ran.out, ran.err);
    let lines: Vec<&str> = ran.out.lines().collect();
    assert_eq!(lines.len(), 2, "{}", ran.out);
    assert!(
        lines[0].starts_with("model stub-model at 127.0.0.1:"),
        "{}",
        lines[0]
    );
    assert!(lines[0].contains("1 calls answered in "), "{}", lines[0]);
    assert!(
        lines[0].contains("11 tokens in, 7 tokens out, no wait"),
        "{}",
        lines[0]
    );
    assert_eq!(lines[1], "this endpoint can be used");
    assert_eq!(ran.err, "");

    assert_eq!(seen.len(), 1, "one small fixed task");
    let user = seen[0].json()["messages"][1]["content"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(user.contains("def total_price(items):"), "{user}");
    assert_eq!(files, 0, "nothing is written where the command is run");
}

#[test]
fn a_rejected_answer_is_repaired_and_a_fence_around_the_answer_is_no_reason() {
    let fenced = format!("```json\n{}\n```", good());
    let (ran, seen, _) = check(
        "repaired",
        vec![
            ok(chat_reply("ok", "stop")),
            ok(chat_reply(&fenced, "stop")),
        ],
    );

    assert_eq!(ran.code, Some(0), "{}\n{}", ran.out, ran.err);
    assert!(ran.out.contains("2 calls answered in "), "{}", ran.out);
    assert!(
        ran.out.contains("22 tokens in, 14 tokens out"),
        "{}",
        ran.out
    );
    assert!(
        ran.out.ends_with("this endpoint can be used\n"),
        "{}",
        ran.out
    );
    // Why the first answer was rejected is said, though the repair passed.
    let lines: Vec<&str> = ran.out.lines().collect();
    assert_eq!(lines.len(), 3, "{}", ran.out);
    assert!(
        lines[1].contains("call 1: the answer is not the JSON object asked for"),
        "{}",
        ran.out
    );
    assert_eq!(seen.len(), 2);
    let repair = seen[1].json()["messages"][1]["content"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(
        repair.contains("A previous answer to this request was rejected."),
        "{repair}"
    );
    assert!(repair.contains("not the JSON object asked for"), "{repair}");
}

#[test]
fn a_refusal_is_reported_in_the_endpoints_words_with_exit_code_3() {
    let body = format!(r#"{{"error": {{"message": "The key {KEY} may not use this model."}}}}"#);
    let (ran, seen, files) = check("refused", vec![(403, "", body)]);

    assert_eq!(ran.code, Some(3), "{}\n{}", ran.out, ran.err);
    assert!(ran.out.starts_with("the endpoint failed: "), "{}", ran.out);
    assert!(
        ran.out.contains("refused the call (status 403)"),
        "{}",
        ran.out
    );
    assert!(ran.out.contains("may not use this model"), "{}", ran.out);
    assert!(
        ran.err.contains("refused the call (status 403)"),
        "{}",
        ran.err
    );
    assert_eq!(seen.len(), 1);
    assert_eq!(files, 0);
}

#[test]
fn answers_the_checks_reject_end_after_three_calls_with_exit_code_4_and_the_check_named() {
    let (ran, seen, files) = check(
        "rejected",
        vec![
            ok(chat_reply("Here is the function you asked for.", "stop")),
            ok(chat_reply(
                &answer_with("src/prices.py", "def total_price(items:\n    return 0\n"),
                "stop",
            )),
            ok(chat_reply(&answer_with("src/other.py", "x = 1\n"), "stop")),
            ok(chat_reply(&good(), "stop")),
        ],
    );

    assert_eq!(ran.code, Some(4), "{}\n{}", ran.out, ran.err);
    assert_eq!(seen.len(), 3, "at most three calls");
    let lines: Vec<&str> = ran.out.lines().collect();
    assert_eq!(lines.len(), 5, "{}", ran.out);
    assert!(lines[0].contains("3 calls answered in "), "{}", lines[0]);
    assert!(
        lines[1].contains("the check rejected the answer of each of 3 calls"),
        "{}",
        lines[1]
    );
    assert!(
        lines[2].contains("call 1: the answer is not the JSON object asked for"),
        "{}",
        lines[2]
    );
    assert!(lines[3].contains("call 2: syntax_error"), "{}", lines[3]);
    assert!(
        lines[4].contains("call 3: paths outside scope"),
        "{}",
        lines[4]
    );
    assert_eq!(
        ran.err, "",
        "a rejected answer is not a failure of the endpoint"
    );
    assert_eq!(files, 0);
}

#[test]
fn a_wait_before_the_answer_is_reported() {
    let busy = (
        429,
        "retry-after: 1\r\n",
        r#"{"error": {"message": "slow down"}}"#.to_owned(),
    );
    let (ran, seen, _) = check("waited", vec![busy, ok(chat_reply(&good(), "stop"))]);

    assert_eq!(ran.code, Some(0), "{}\n{}", ran.out, ran.err);
    assert!(ran.out.contains("waited 1 s in 1 waits"), "{}", ran.out);
    assert!(ran.err.contains("asks to come back later"), "{}", ran.err);
    assert_eq!(seen.len(), 2);
}

#[test]
fn an_unusable_command_line_or_endpoint_file_ends_with_exit_code_2() {
    let project = Project::new("usage");
    let ran = run_in(Some(&project.0), &["check", "--record", "calls.jsonl"], "");
    assert_eq!(ran.code, Some(2), "{}", ran.err);
    assert!(ran.err.contains("unknown argument --record"), "{}", ran.err);
    assert!(
        ran.err.contains("usage: lesscode-model check"),
        "{}",
        ran.err
    );

    let missing = std::env::temp_dir().join("lesscode-model-check-no-such-file.env");
    let ran = run_in(
        Some(&project.0),
        &["check", "--endpoint-env", missing.to_str().unwrap()],
        "",
    );
    assert_eq!(ran.code, Some(2), "{}\n{}", ran.out, ran.err);
    assert!(
        ran.out.starts_with("the endpoint failed: configuration: "),
        "{}",
        ran.out
    );
    assert!(
        ran.out.contains("lesscode-model-check-no-such-file.env"),
        "{}",
        ran.out
    );
    assert_eq!(project.files(), 0);
}
