//! Running the binary `lesscode-model` in a test: an endpoint file for a
//! stub, the command, and what it said and how it ended.
#![allow(dead_code)]

use serde_json::Value;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const KEY: &str = "test-key-0123456789-do-not-show";

/// An endpoint file in the temp folder that is removed when the test ends.
pub struct EndpointFile(pub PathBuf);

impl EndpointFile {
    pub fn new(name: &str, address: &str) -> Self {
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

    pub fn path(&self) -> &str {
        self.0.to_str().expect("a path that is text")
    }
}

impl Drop for EndpointFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

pub struct Ran {
    pub code: Option<i32>,
    pub out: String,
    pub err: String,
}

impl Ran {
    /// Standard output is one JSON object and nothing else.
    pub fn answer(&self) -> Value {
        serde_json::from_str(&self.out).unwrap_or_else(|problem| {
            panic!("standard output is not JSON ({problem}): {}", self.out)
        })
    }

    pub fn shows_no_key(&self) {
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
pub fn run(args: &[&str], input: &str) -> Ran {
    run_in(None, args, input)
}

/// The same, started in `directory` when one is named.
pub fn run_in(directory: Option<&Path>, args: &[&str], input: &str) -> Ran {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lesscode-model"));
    if let Some(directory) = directory {
        command.current_dir(directory);
    }
    let mut child = command
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
