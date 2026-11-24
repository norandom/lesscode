//! `step protect begin` through the binary (brownfield-test-stage, group 1).
//! A real plan of the toy repository is made first; the command then only
//! reads. Every refusal is exit 3 with its sentence, and writes nothing.

mod step_support;

use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use step_support::{toy, Repo};

/// A file whose functions are built on one system each, and one built on all.
const SYSTEMS: &str = "#[cfg(target_os = \"windows\")]\npub fn only_windows(a: i32) -> i32 {\n    if a > 3 {\n        return 1;\n    }\n    a\n}\n\n#[cfg(target_os = \"linux\")]\npub fn only_linux(a: i32) -> i32 {\n    if a > 4 {\n        return 2;\n    }\n    a\n}\n\npub fn everywhere(a: i32) -> i32 {\n    if a > 5 {\n        return 3;\n    }\n    a\n}\n";

/// A toy repository with a plan of one of its files.
struct Planned {
    repo: Repo,
    config: PathBuf,
    plan: PathBuf,
    run: PathBuf,
}

impl Planned {
    fn new(name: &str, language: &str, file: &str) -> Planned {
        let repo = Repo::new(name);
        if file != "src/app.py" {
            let files = [(file, SYSTEMS), ("tests/expect.txt", "if a > 3 {\n")];
            repo.commit(&files, "a file of another language");
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

    fn configure(&self, language: &str, file: &str, test_paths: &[&str]) {
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
    fn begin(&self, functions: &[&str]) -> (Option<i32>, String) {
        let (plan, config) = (self.plan.to_string_lossy(), self.config.to_string_lossy());
        let mut args = vec!["protect", "begin", "--plan", &plan, "--config", &config];
        for function in functions {
            args.extend(["--function", function]);
        }
        args.extend(["--spec", "protect-toy"]);
        self.repo.cli(&args, "")
    }

    /// The command is refused with exit 3 and these words, and writes nothing.
    fn refused(&self, functions: &[&str], words: &[&str]) {
        let before = self.repo.state();
        let (code, said) = self.begin(functions);
        assert_eq!(code, Some(3), "{said}");
        for word in words {
            assert!(said.contains(word), "{word:?} is not in: {said}");
        }
        assert_eq!(self.repo.state(), before, "a refusal wrote something");
    }

    fn edit(path: &PathBuf, key: &str, value: Value) {
        let mut json: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        json[key] = value;
        fs::write(path, json.to_string()).unwrap();
    }
}

#[test]
fn named_functions_of_the_plan_are_chosen_and_nothing_is_asked() {
    let planned = Planned::new("protect-chosen", "python", "src/app.py");
    let before = planned.repo.state();
    // Standard input is empty and closed: the command asks nothing.
    let (code, said) = planned.begin(&["src/app.py:helper", "src/app.py:helper"]);
    assert_eq!(code, Some(0), "{said}");
    assert_eq!(
        said.matches("Chosen: src/app.py: helper").count(),
        1,
        "{said}"
    );
    assert!(!said.contains("src/app.py: step"), "{said}");
    assert_eq!(planned.repo.state(), before);
}

#[test]
fn a_function_the_plan_does_not_have_is_refused_with_the_ones_it_has() {
    let planned = Planned::new("protect-missing", "python", "src/app.py");
    planned.refused(
        &["src/app.py:nothing"],
        &[
            "src/app.py:nothing is not a function of the plan",
            "In that file the plan has",
            "  src/app.py:helper",
            "  src/app.py:step",
        ],
    );
    planned.refused(
        &["src/other.py:helper"],
        &["src/other.py:helper is not a function of the plan. The plan has:"],
    );
    planned.refused(&["helper"], &["write it as <file>:<name>"]);
}

#[test]
fn a_plan_without_a_result_is_refused() {
    let planned = Planned::new("protect-no-result", "python", "src/app.py");
    Planned::edit(
        &planned.run,
        "void",
        json!("the tests fail before any fault"),
    );
    planned.refused(
        &["src/app.py:helper"],
        &[
            "was void (the tests fail before any fault)",
            "A plan with a result is needed first",
        ],
    );
    Planned::edit(&planned.run, "void", Value::Null);
    Planned::edit(&planned.plan, "incomplete", json!(true));
    planned.refused(
        &["src/app.py:helper"],
        &["is incomplete", "A plan with a result is needed first"],
    );
}

#[test]
fn a_file_changed_since_the_plan_is_refused() {
    let planned = Planned::new("protect-changed", "python", "src/app.py");
    let changed = format!("{}\n# later\n", step_support::APP);
    planned
        .repo
        .commit(&[("src/app.py", &changed)], "a change after the plan");
    planned.refused(
        &["src/app.py:helper"],
        &[
            "src/app.py has changed since the plan was made",
            "Make a new plan",
        ],
    );
}

#[test]
fn a_stage_that_was_begun_and_a_test_path_without_a_file_are_refused() {
    let planned = Planned::new("protect-places", "python", "src/app.py");
    planned.configure("python", "src/app.py", &["tests/", "checks/"]);
    planned.refused(
        &["src/app.py:helper"],
        &["The test path checks/ holds no file"],
    );
    planned.configure("python", "src/app.py", &[]);
    planned.refused(&["src/app.py:helper"], &["names no test path"]);

    planned.configure("python", "src/app.py", &["tests/"]);
    planned.repo.commit(
        &[(".kiro/specs/protect-toy/stage.json", "{}")],
        "an earlier stage",
    );
    planned.refused(&["src/app.py:helper"], &["already holds a stage.json"]);
}

#[test]
fn a_function_in_code_not_built_here_is_left_out_and_named() {
    let planned = Planned::new("protect-systems", "rust", "src/lib.rs");
    let elsewhere = match cfg!(windows) {
        true => "src/lib.rs:only_linux",
        false => "src/lib.rs:only_windows",
    };
    let (code, said) = planned.begin(&[elsewhere, "src/lib.rs:everywhere"]);
    assert_eq!(code, Some(0), "{said}");
    assert!(said.contains("Chosen: src/lib.rs: everywhere"), "{said}");
    let left_out = format!("Left out: {} lies wholly", elsewhere.replace(':', ": "));
    assert!(said.contains(&left_out), "{said}");
    assert!(said.contains("Nothing is known about it here"), "{said}");

    planned.refused(
        &[elsewhere],
        &["Every named function lies wholly in code that is not built"],
    );
}
