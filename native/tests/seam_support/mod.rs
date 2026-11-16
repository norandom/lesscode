//! The scripted stand-in for `lesscode-model` (example `scripted_model`),
//! shared by the launcher's tests and those of `regenerate run`: a script of
//! what it answers, and what it was given.
#![allow(dead_code)]

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// The stand-in, built beside the test binary.
pub fn scripted() -> PathBuf {
    let test_binary = std::env::current_exe().unwrap();
    let profile = test_binary.parent().unwrap().parent().unwrap();
    let name = format!("scripted_model{}", std::env::consts::EXE_SUFFIX);
    let path = profile.join("examples").join(name);
    assert!(path.is_file(), "not run: {} was not built", path.display());
    path
}

/// What the stand-in answers: standard output, standard error, exit code.
/// The file and what the stand-in saw are removed when the test ends.
pub struct Script(pub PathBuf);

impl Script {
    pub fn new(dir: &Path, name: &str, stdout: &str, stderr: &str, exit: u8) -> Script {
        fs::create_dir_all(dir).unwrap();
        let path = dir.join(format!("{name}.script.json"));
        let script = json!({"stdout": stdout, "stderr": stderr, "exit": exit});
        fs::write(&path, script.to_string()).unwrap();
        Script(path)
    }

    fn seen_path(&self) -> PathBuf {
        let mut path = self.0.clone().into_os_string();
        path.push(".seen");
        PathBuf::from(path)
    }

    fn seen_value(&self) -> Value {
        serde_json::from_slice(&fs::read(self.seen_path()).unwrap()).unwrap()
    }

    /// The arguments and the request of the last call, the request parsed.
    pub fn seen(&self) -> (Vec<String>, Value) {
        let seen = self.seen_value();
        let args = seen["args"].as_array().unwrap();
        let args = args
            .iter()
            .map(|a| a.as_str().unwrap().to_owned())
            .collect();
        (args, serde_json::from_str(&self.seen_request()).unwrap())
    }

    /// The request of the last call as it was written to standard input.
    pub fn seen_request(&self) -> String {
        self.seen_value()["request"].as_str().unwrap().to_owned()
    }

    pub fn was_called(&self) -> bool {
        self.seen_path().is_file()
    }
}

impl Drop for Script {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.seen_path());
        let _ = fs::remove_file(&self.0);
    }
}

/// What `lesscode-model call` reports as sent when no temperature and no
/// reasoning level were sent.
pub fn sent() -> Value {
    json!({
        "host": "localhost", "model": "scripted-model", "max_output_tokens": 64000,
        "temperature": null, "reasoning_level": null,
    })
}

/// The answer of an answered call with this text and this `sent`.
pub fn answered_with(text: &str, sent: Value) -> String {
    json!({"protocol": 1, "ok": true, "reply": {
        "text": text, "stop": "complete", "input_tokens": 120, "output_tokens": 30,
        "reasoning_tokens": null, "cached_input_tokens": null, "structured": true,
        "seconds": 1.5, "waits": [], "sent": sent,
    }})
    .to_string()
}

pub fn answered(text: &str) -> String {
    answered_with(text, sent())
}

/// The answer of a failed call.
pub fn failed(kind: &str, reason: &str) -> String {
    json!({"protocol": 1, "ok": false, "error": {"kind": kind, "reason": reason}}).to_string()
}
