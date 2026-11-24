//! A toy repository with a real plan of one of its files: what the tests
//! of `step protect` begin from.

use super::{toy, Repo};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// A file whose functions are built on one system each, and one built on all.
pub const SYSTEMS: &str = "#[cfg(target_os = \"windows\")]\npub fn only_windows(a: i32) -> i32 {\n    if a > 3 {\n        return 1;\n    }\n    a\n}\n\n#[cfg(target_os = \"linux\")]\npub fn only_linux(a: i32) -> i32 {\n    if a > 4 {\n        return 2;\n    }\n    a\n}\n\npub fn everywhere(a: i32) -> i32 {\n    if a > 5 {\n        return 3;\n    }\n    a\n}\n";

/// A toy repository with a plan of one of its files.
pub struct Planned {
    pub repo: Repo,
    pub config: PathBuf,
    pub plan: PathBuf,
    pub run: PathBuf,
}

impl Planned {
    pub fn new(name: &str, language: &str, file: &str) -> Planned {
        let other = [(file, SYSTEMS), ("tests/expect.txt", "if a > 3 {\n")];
        match file {
            "src/app.py" => Planned::with(name, language, file, &[]),
            _ => Planned::with(name, language, file, &other),
        }
    }

    /// The toy repository with these files committed too, and a plan of `file`.
    pub fn with(name: &str, language: &str, file: &str, files: &[(&str, &str)]) -> Planned {
        let repo = Repo::new(name);
        if !files.is_empty() {
            repo.commit(files, "files of this test");
        }
        let root = repo.dir.parent().unwrap().to_path_buf();
        let planned = Planned {
            config: root.join("run.json"),
            plan: root.join("plan.json"),
            run: root.join("plan-work/plan-run.json"),
            repo,
        };
        planned.configure(language, file, &["tests/"]);
        let output = Command::new(env!("CARGO_BIN_EXE_lesscode-core"))
            .args(["step", "plan", "--config"])
            .arg(&planned.config)
            .arg("--work")
            .arg(root.join("plan-work"))
            .arg("--out")
            .arg(&planned.plan)
            .output()
            .expect("run lesscode-core binary");
        let said = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(0), "the plan: {said}");
        planned
    }

    pub fn configure(&self, language: &str, file: &str, test_paths: &[&str]) {
        let config = json!({
            "language": language, "source_root": "repo", "files": [file],
            "test_paths": test_paths,
            "test": [toy(), "--file", file, "--print", "TOY RAN",
                     "--expect-from", "tests/expect.txt"],
            "timeout_floor_s": 10, "timeout_ceiling_s": 60, "workers": 2,
        });
        fs::write(&self.config, config.to_string()).unwrap();
    }

    /// `step protect begin` for the functions, as the spec `protect-toy`.
    pub fn begin(&self, functions: &[&str]) -> (Option<i32>, String) {
        self.begin_with(functions, &[])
    }

    pub fn begin_with(&self, functions: &[&str], more: &[&str]) -> (Option<i32>, String) {
        let (plan, config) = (self.plan.to_string_lossy(), self.config.to_string_lossy());
        let mut args = vec!["protect", "begin", "--plan", &plan, "--config", &config];
        for function in functions {
            args.extend(["--function", function]);
        }
        args.extend(["--spec", "protect-toy"]);
        args.extend(more);
        self.repo.cli(&args, "")
    }

    /// The command is refused with exit 3 and these words, and writes nothing.
    pub fn refused(&self, functions: &[&str], words: &[&str]) {
        let before = self.repo.state();
        let (code, said) = self.begin(functions);
        assert_eq!(code, Some(3), "{said}");
        for word in words {
            assert!(said.contains(word), "{word:?} is not in: {said}");
        }
        assert_eq!(self.repo.state(), before, "a refusal wrote something");
    }

    pub fn edit(path: &PathBuf, key: &str, value: Value) {
        let mut json: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        json[key] = value;
        fs::write(path, json.to_string()).unwrap();
    }
}
