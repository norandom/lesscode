//! A small git repository to take steps in, shared by the step tests. Its
//! "test suite" is the toy check with the expectations of
//! `tests/expect.txt`: a test file a step can add pins to.
#![allow(dead_code)]

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const APP: &str = "def step(state, a):\n    if a > 3:\n        raise ValueError(\"big\")\n    state.total = a\n    return a\n\n\ndef helper(state, b):\n    state.count = b\n    return b\n";

/// The test stand-in, built beside the test binary.
pub fn toy() -> String {
    let test_binary = std::env::current_exe().unwrap();
    let profile = test_binary.parent().unwrap().parent().unwrap();
    let name = format!("toy_check{}", std::env::consts::EXE_SUFFIX);
    let path = profile.join("examples").join(name);
    assert!(path.is_file(), "not run: {} was not built", path.display());
    path.to_string_lossy().into_owned()
}

pub struct Repo {
    pub dir: PathBuf,
    pub work: PathBuf,
}

impl Repo {
    /// A repository whose main branch has the application and one expectation.
    pub fn new(name: &str) -> Repo {
        let root = std::env::temp_dir().join(format!("lc-step-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let repo = Repo {
            dir: root.join("repo"),
            work: root.join("work"),
        };
        fs::create_dir_all(&repo.dir).unwrap();
        repo.git(&["init", "-q", "-b", "main"]);
        repo.git(&["config", "core.autocrlf", "false"]);
        repo.commit(
            &[("src/app.py", APP), ("tests/expect.txt", "if a > 3:\n")],
            "base",
        );
        repo
    }

    pub fn git(&self, args: &[&str]) -> String {
        let out = Command::new("git")
            .arg("-C")
            .arg(&self.dir)
            .args([
                "-c",
                "user.name=test",
                "-c",
                "user.email=test@example.invalid",
            ])
            .args(["-c", "commit.gpgsign=false"])
            .args(args)
            .output()
            .unwrap();
        let said = String::from_utf8_lossy(&out.stderr);
        assert!(out.status.success(), "git {args:?}: {said}");
        String::from_utf8_lossy(&out.stdout).trim().to_owned()
    }

    pub fn write(&self, path: &str, text: &str) {
        let full = self.dir.join(path);
        fs::create_dir_all(full.parent().unwrap()).unwrap();
        fs::write(full, text).unwrap();
    }

    /// Write the files and commit them; returns the commit.
    pub fn commit(&self, files: &[(&str, &str)], message: &str) -> String {
        for (path, text) in files {
            self.write(path, text);
        }
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", message]);
        self.git(&["rev-parse", "HEAD"])
    }

    pub fn declaration_path(step: &str) -> String {
        format!(".kiro/specs/{step}/step.json")
    }

    /// The declaration of a step on the branch `step/<name>`; `extra` adds
    /// or replaces keys.
    pub fn declaration(step: &str, extra: Value) -> String {
        let mut declaration = json!({
            "step": step, "spec": step, "branch": format!("step/{step}"), "base": "main",
            "language": "python", "scope": ["src/app.py"], "test_paths": ["tests/"],
            "test": [toy(), "--file", "src/app.py", "--print", "TOY RAN",
                     "--expect-from", "tests/expect.txt"],
            "run": {"timeout_floor_s": 10, "timeout_ceiling_s": 60, "workers": 2},
            "focus": {"src/app.py": ["step"]},
            "outputs": {"verdict": format!(".kiro/specs/{step}/verdict.json"),
                        "evidence": format!(".kiro/specs/{step}/evidence.json"),
                        "ledger": ".kiro/steps/ledger.jsonl"}
        });
        for (key, value) in extra.as_object().unwrap() {
            declaration[key] = value.clone();
        }
        serde_json::to_string_pretty(&declaration).unwrap()
    }

    /// Start the branch of a step from main and commit its declaration.
    pub fn begin(&self, step: &str, extra: Value) {
        self.git(&["checkout", "-q", "-b", &format!("step/{step}"), "main"]);
        let path = Repo::declaration_path(step);
        self.commit(
            &[(&path, &Repo::declaration(step, extra))],
            "declare the step",
        );
    }

    /// Run a `step` command on a step of this repository.
    pub fn step(&self, command: &str, step: &str, more: &[&str]) -> (Option<i32>, String, String) {
        let output = Command::new(env!("CARGO_BIN_EXE_lesscode-core"))
            .args(["step", command, "--declaration"])
            .arg(Repo::declaration_path(step))
            .arg("--repo")
            .arg(&self.dir)
            .arg("--work")
            .arg(self.work.join(step))
            .args(more)
            .output()
            .expect("run lesscode-core binary");
        (
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        )
    }

    pub fn json(&self, path: &Path) -> Value {
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    }

    /// Every file of the working tree and the refs, to show nothing changed.
    pub fn state(&self) -> Vec<(String, Vec<u8>)> {
        let mut found = vec![("refs".to_owned(), self.git(&["show-ref"]).into_bytes())];
        let mut stack = vec![self.dir.clone()];
        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                let name = path
                    .strip_prefix(&self.dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                if name == ".git" {
                    continue;
                }
                match path.is_dir() {
                    true => stack.push(path),
                    false => found.push((name, fs::read(&path).unwrap())),
                }
            }
        }
        found.sort();
        found
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(self.dir.parent().unwrap());
    }
}
