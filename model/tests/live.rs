//! The first cases against real endpoints (task 5.1; requirements 2.7, 1.6),
//! through the binary. Run by hand, never in CI: without
//! `LESSCODE_LIVE_ENV` both tests say so and end.
//!
//! - `LESSCODE_LIVE_ENV`: the endpoint file of the case, outside the repository.
//! - `LESSCODE_LIVE_TURN`: a directory with `system.txt` and `user.txt`, the
//!   prompts of one turn written by `regenerate run` (the turn of R102).
//! - `LESSCODE_LIVE_OUT`: a directory for what the binary said.
//! - `LESSCODE_MODEL_NAME`, `LESSCODE_MODEL_REASONING_EFFORT`: as for any run.
//!
//! Run with one test thread: no two calls against one upstream side by side.
//! `docs/experiments/model-seam/run-live.ps1` does that for the six cases.

mod binary;

use binary::{Ran, run_as_set};
use serde_json::json;
use std::path::PathBuf;
use std::time::Instant;

struct Live {
    endpoint_file: String,
    out: PathBuf,
    key: String,
}

/// The case named in the environment, or nothing when no endpoint file is.
fn live() -> Option<Live> {
    let Ok(endpoint_file) = std::env::var("LESSCODE_LIVE_ENV") else {
        eprintln!("not run: LESSCODE_LIVE_ENV names no endpoint file");
        return None;
    };
    let out = PathBuf::from(std::env::var("LESSCODE_LIVE_OUT").expect("LESSCODE_LIVE_OUT"));
    std::fs::create_dir_all(&out).expect("the directory for what the binary said");
    let key = std::fs::read_to_string(&endpoint_file)
        .expect("the endpoint file can be read")
        .lines()
        .find_map(|line| line.trim().strip_prefix("LESSCODE_MODEL_API_KEY="))
        .and_then(|value| value.split_whitespace().next())
        .expect("the endpoint file has a key")
        .to_owned();
    Some(Live {
        endpoint_file,
        out,
        key,
    })
}

impl Live {
    /// Keeps what the binary said, then fails if any of it shows the key.
    fn keep(&self, name: &str, ran: &Ran, seconds: f64) {
        let said = format!(
            "exit code: {:?}\nseconds: {seconds:.1}\n--- standard output\n{}\n--- standard error\n{}\n",
            ran.code, ran.out, ran.err
        );
        assert!(!said.contains(&self.key), "{name} shows the key");
        std::fs::write(self.out.join(name), said).expect("what the binary said is kept");
    }
}

#[test]
fn call_answers_the_turn_of_r102() {
    let Some(live) = live() else { return };
    let turn = PathBuf::from(std::env::var("LESSCODE_LIVE_TURN").expect("LESSCODE_LIVE_TURN"));
    let part = |name: &str| std::fs::read_to_string(turn.join(name)).expect("a prompt of the turn");
    let request = json!({
        "protocol": 1,
        "purpose": "first-case",
        "system": part("system.txt"),
        "user": part("user.txt"),
        "max_output_tokens": 64_000,
        "structure": {
            "type": "object",
            "required": ["files", "structure", "change_log"],
            "properties": {
                "files": {
                    "type": "array",
                    "items": {"type": "object", "required": ["path", "content"]},
                },
                "structure": {"type": "object", "required": ["pure", "shell"]},
                "change_log": {"type": "string"},
            },
        },
    });
    let record = live.out.join("record.jsonl");
    let _ = std::fs::remove_file(&record);

    let started = Instant::now();
    let ran = run_as_set(
        &[
            "call",
            "--endpoint-env",
            &live.endpoint_file,
            "--record",
            record.to_str().expect("a path that is text"),
        ],
        &request.to_string(),
    );
    live.keep("call.txt", &ran, started.elapsed().as_secs_f64());

    assert_eq!(ran.code, Some(0), "{}", ran.err);
    let reply = &ran.answer()["reply"];
    assert_eq!(reply["stop"], "complete", "{reply}");
    assert_eq!(reply["structured"], true, "{reply}");
    assert!(reply["output_tokens"].as_u64() > Some(0), "{reply}");
    let recorded = std::fs::read_to_string(&record).expect("the record of the call");
    assert_eq!(recorded.lines().count(), 1, "{recorded}");
    assert!(!recorded.contains(&live.key), "the record shows the key");
}

#[test]
fn check_finds_the_endpoint_usable() {
    let Some(live) = live() else { return };

    let started = Instant::now();
    let ran = run_as_set(&["check", "--endpoint-env", &live.endpoint_file], "");
    live.keep("check.txt", &ran, started.elapsed().as_secs_f64());

    assert_eq!(ran.code, Some(0), "{}\n{}", ran.out, ran.err);
    assert!(ran.out.contains("this endpoint can be used"), "{}", ran.out);
}
