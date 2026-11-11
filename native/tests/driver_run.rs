//! `deconvolute run` on a toy project: classification, time limits, void
//! runs, parallel copies, reuse, annotations and the safety rules.
//!
//! Governed by `.kiro/specs/agnostic-annotation-driver/` Requirements 4 and
//! 5 (tasks 5.1 to 6.5). The project's "test command" is the example
//! `toy_check`, which needs no network.

use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const APP: &str = "def step(state, a):\n    if a > 3:\n        raise ValueError(\"big\")\n    state.total = a\n    return a\n";

/// The test stand-in, built beside this test binary.
fn toy() -> String {
    let test_binary = std::env::current_exe().unwrap();
    let profile = test_binary.parent().unwrap().parent().unwrap();
    let name = format!("toy_check{}", std::env::consts::EXE_SUFFIX);
    let path = profile.join("examples").join(name);
    assert!(path.is_file(), "not run: {} was not built", path.display());
    path.to_string_lossy().into_owned()
}

struct Project {
    dir: PathBuf,
}

impl Project {
    fn new(name: &str) -> Project {
        let dir = std::env::temp_dir().join(format!("lc-test-run-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("project/src")).unwrap();
        fs::write(dir.join("project/src/app.py"), APP).unwrap();
        Project { dir }
    }

    /// A configuration whose test passes when the condition and the write
    /// are in the file; `extra` adds or replaces keys.
    fn config(&self, test_extra: &[&str], extra: Value) -> PathBuf {
        let mut test = vec![
            toy(),
            "--file".into(),
            "src/app.py".into(),
            "--print".into(),
            "TOY RAN".into(),
        ];
        for expected in ["if a > 3:", "state.total = a"] {
            test.extend(["--expect".to_owned(), expected.to_owned()]);
        }
        test.extend(test_extra.iter().map(|s| (*s).to_owned()));
        let mut config = json!({
            "language": "python", "source_root": "project", "files": ["src/app.py"], "test": test,
            "timeout_floor_s": 10, "timeout_ceiling_s": 60,
        });
        for (key, value) in extra.as_object().unwrap() {
            config[key] = value.clone();
        }
        let path = self.dir.join("run.json");
        fs::write(&path, config.to_string()).unwrap();
        path
    }

    fn run(&self, config: &Path, work: &str, more: &[&str]) -> (Option<i32>, Value, String) {
        let out = self.dir.join(format!("{work}-receipt.json"));
        let _ = fs::remove_file(&out);
        let output = Command::new(env!("CARGO_BIN_EXE_lesscode-core"))
            .args(["deconvolute", "run", "--config"])
            .arg(config)
            .arg("--work")
            .arg(self.dir.join(work))
            .arg("--out")
            .arg(&out)
            .args(more)
            .output()
            .expect("run lesscode-core binary");
        let receipt = fs::read(&out)
            .ok()
            .map(|b| serde_json::from_slice(&b).unwrap())
            .unwrap_or(Value::Null);
        (
            output.status.code(),
            receipt,
            String::from_utf8_lossy(&output.stderr).into_owned(),
        )
    }

    fn source(&self) -> String {
        fs::read_to_string(self.dir.join("project/src/app.py")).unwrap()
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn classes(receipt: &Value) -> BTreeMap<String, String> {
    let mutants = receipt["mutants"].as_array().unwrap();
    mutants
        .iter()
        .map(|m| {
            (
                format!(
                    "{}:{}",
                    m["site"]["start_line"],
                    m["operator"].as_str().unwrap()
                ),
                m["class"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

#[test]
fn mutants_are_killed_or_survive_and_the_source_is_untouched() {
    let project = Project::new("classes");
    let (code, receipt, stderr) = project.run(&project.config(&[], json!({})), "work", &[]);
    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(receipt["receipt"], "deconvolution-run-v1");
    assert_eq!(receipt["void"], Value::Null);
    assert_eq!(receipt["incomplete"], false);
    let found = classes(&receipt);
    assert_eq!(found["2:negate"], "killed", "{found:?}");
    assert_eq!(found["4:drop_statement"], "killed", "{found:?}");
    assert!(
        found.values().any(|c| c == "survived"),
        "the raise is not checked by the toy test: {found:?}"
    );
    assert_eq!(
        receipt["counts_by_class"]["killed"].as_u64().unwrap()
            + receipt["counts_by_class"]["survived"].as_u64().unwrap(),
        found.len() as u64
    );
    let worker = &receipt["workers"][0];
    assert_eq!(worker["closing_baseline_passed"], true);
    assert_eq!(
        worker["limits_s"][0], 10,
        "the floor, since the unmutated run is fast"
    );
    assert_eq!(project.source(), APP, "the source tree was changed");
    assert_eq!(
        fs::read_to_string(project.dir.join("work/w1/tree/src/app.py")).unwrap(),
        APP,
        "the copy was not restored"
    );
}

#[test]
fn time_limits_invalid_mutants_and_errors_are_told_apart() {
    let project = Project::new("limits");
    let slow = project.config(
        &["--sleep-if-missing-ms", "6000"],
        json!({"timeout_floor_s": 2}),
    );
    let (code, receipt, stderr) = project.run(&slow, "slow", &[]);
    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(classes(&receipt)["2:negate"], "timeout");

    let unloadable = project.config(&["--missing-exit", "4"], json!({"invalid_exit_codes": [4]}));
    let (code, receipt, _) = project.run(&unloadable, "invalid", &[]);
    assert_eq!(code, Some(0));
    assert_eq!(classes(&receipt)["2:negate"], "invalid");

    let contained = project.config(
        &["--missing-exit", "137"],
        json!({"timeout_exit_codes": [137]}),
    );
    let (code, receipt, _) = project.run(&contained, "contained", &[]);
    assert_eq!(code, Some(0));
    assert_eq!(classes(&receipt)["2:negate"], "timeout");

    // The step's limit is handed to a command that asks for it.
    let told = project.config(
        &["--print-if-missing", "limit={limit_s}s"],
        json!({"error_output_patterns": ["limit=10s"]}),
    );
    let (_, receipt, _) = project.run(&told, "told", &[]);
    assert_eq!(
        classes(&receipt)["2:negate"],
        "error",
        "the limit of 10 seconds was not substituted"
    );

    // A failing exit without the runner's own output is not a kill.
    let silent = project.config(
        &[],
        json!({"runner_evidence_patterns": ["tests collected"]}),
    );
    let (code, receipt, _) = project.run(&silent, "silent", &[]);
    assert_eq!(code, Some(2), "errors remain, so the run is incomplete");
    assert_eq!(classes(&receipt)["2:negate"], "error");
    assert_eq!(receipt["incomplete"], true);
    let stored = project
        .dir
        .join("silent/outcomes")
        .read_dir()
        .unwrap()
        .count();
    let not_errors = classes(&receipt).values().filter(|c| *c != "error").count();
    assert_eq!(stored, not_errors, "errors are never stored");

    let exhausted = project.config(
        &["--print-if-missing", "java.lang.OutOfMemoryError"],
        json!({"error_output_patterns": ["OutOfMemoryError"]}),
    );
    let (code, receipt, _) = project.run(&exhausted, "exhausted", &[]);
    assert_eq!(code, Some(2));
    assert_eq!(
        receipt["mutants"][0]["reason"],
        "resource failure during test"
    );
}

#[test]
fn a_run_whose_unmutated_tests_fail_is_void() {
    let project = Project::new("void");
    let failing = project.config(&["--expect", "this text is not in the file"], json!({}));
    let (code, receipt, stderr) = project.run(&failing, "work", &[]);
    assert_eq!(code, Some(3));
    assert!(stderr.contains("void"), "{stderr}");
    assert!(
        receipt["void"]
            .as_str()
            .unwrap()
            .contains("unmutated tests"),
        "{receipt}"
    );
    assert_eq!(
        receipt["mutants"].as_array().unwrap().len(),
        0,
        "nothing was mutated"
    );

    // A mutant's run leaves something behind that makes the unmutated tests
    // fail afterwards: the classes cannot be trusted, and the receipt says
    // which copy and shows the end of the failing output.
    let poisoned = project.config(&["--poison", "left-behind.marker"], json!({}));
    let (code, receipt, stderr) = project.run(&poisoned, "poisoned", &[]);
    assert_eq!(code, Some(3), "{stderr}");
    let void = receipt["void"].as_str().unwrap();
    assert!(
        void.contains("no longer passed after the mutants"),
        "{void}"
    );
    assert!(void.contains("copy 1: test step, exit Some(1)"), "{void}");
    assert!(void.contains("poisoned by an earlier run"), "{void}");
    assert_eq!(receipt["workers"][0]["closing_baseline_passed"], false);
    assert_eq!(project.source(), APP);

    let missing = project.config(&[], json!({"test": ["no-such-program-lesscode-test"]}));
    let (code, receipt, _) = project.run(&missing, "missing", &[]);
    assert_eq!(code, Some(3));
    assert!(receipt["void"].is_string());
    assert_eq!(project.source(), APP);
}

#[test]
fn parallel_copies_give_the_same_classes_and_outcomes_are_reused() {
    let project = Project::new("parallel");
    let config = project.config(&[], json!({}));
    let (code, serial, _) = project.run(&config, "serial", &[]);
    assert_eq!(code, Some(0));
    let (code, parallel, stderr) = project.run(&config, "parallel", &["--workers", "3"]);
    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(classes(&serial), classes(&parallel));
    assert_eq!(parallel["workers"].as_array().unwrap().len(), 3);

    let (code, again, _) = project.run(&config, "serial", &[]);
    assert_eq!(code, Some(0));
    assert_eq!(classes(&again), classes(&serial));
    assert!(again["mutants"]
        .as_array()
        .unwrap()
        .iter()
        .all(|m| m["reused_from"].is_string() && m["worker"].is_null()));
    let (_, fresh, _) = project.run(&config, "serial", &["--no-reuse"]);
    assert!(fresh["mutants"]
        .as_array()
        .unwrap()
        .iter()
        .all(|m| m["reused_from"].is_null()));

    let serial_only = project.config(&[], json!({"serial_only": true, "workers": 4}));
    let (_, receipt, _) = project.run(&serial_only, "one", &["--workers", "4"]);
    assert_eq!(receipt["workers"].as_array().unwrap().len(), 1);
}

#[test]
fn an_annotated_run_mutates_only_accepted_sites() {
    let project = Project::new("annotated");
    let annotations = project.dir.join("annotations.json");
    fs::write(
        &annotations,
        json!([
            {"file": "src/app.py", "start_line": 2, "end_line": 2, "kind": "branch_condition", "anchor": "a > 3", "reason": "the limit"},
            {"file": "src/app.py", "start_line": 4, "end_line": 4, "kind": "branch_condition", "anchor": "state.total = a", "reason": "wrong kind"},
        ])
        .to_string(),
    )
    .unwrap();
    let config = project.config(&[], json!({}));
    let (code, receipt, stderr) = project.run(
        &config,
        "work",
        &["--annotations", annotations.to_str().unwrap()],
    );
    assert_eq!(code, Some(0), "{stderr}");
    let decisions = receipt["annotation_decisions"].as_array().unwrap();
    assert_eq!(
        (
            decisions[0]["accepted"].as_bool(),
            decisions[1]["reason"].as_str()
        ),
        (Some(true), Some("kind_mismatch"))
    );
    assert_eq!(receipt["sites"], 1);
    assert!(
        classes(&receipt).keys().all(|k| k.starts_with("2:")),
        "{receipt}"
    );
}

/// `merge-annotations` keeps the sites enough lists name, counts only
/// accepted annotations, and its output drives a run.
#[test]
fn annotation_lists_are_merged_by_agreement() {
    let project = Project::new("merge");
    let condition = json!({"file": "src/app.py", "start_line": 2, "end_line": 2, "kind": "branch_condition", "anchor": "a > 3", "reason": "r"});
    let write = json!({"file": "src/app.py", "start_line": 4, "end_line": 4, "kind": "state_write", "anchor": "state.total = a", "reason": "r"});
    let raise = json!({"file": "src/app.py", "start_line": 3, "end_line": 3, "kind": "failure_state", "anchor": "raise ValueError(\"big\")", "reason": "r"});
    let wrong = json!({"file": "src/app.py", "start_line": 4, "end_line": 4, "kind": "branch_condition", "anchor": "state.total = a", "reason": "wrong kind"});
    let lists = [
        json!([condition, write, wrong]),
        json!([condition, raise, wrong]),
        json!([condition, write, condition]),
    ];
    let mut paths = Vec::new();
    for (index, list) in lists.iter().enumerate() {
        let path = project.dir.join(format!("list{index}.json"));
        fs::write(&path, list.to_string()).unwrap();
        paths.push(path);
    }
    let config = project.config(&[], json!({}));
    let merge = |min: &str, out: &Path, lists: &[PathBuf]| {
        let output = Command::new(env!("CARGO_BIN_EXE_lesscode-core"))
            .args(["deconvolute", "merge-annotations", "--config"])
            .arg(&config)
            .args(["--min", min, "--out"])
            .arg(out)
            .args(lists)
            .output()
            .unwrap();
        let receipt: Value = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
        (output.status.code(), receipt)
    };
    let merged = project.dir.join("merged.json");

    let (code, receipt) = merge("2", &merged, &paths);
    assert_eq!(code, Some(0));
    assert_eq!(receipt["receipt"], "deconvolution-annotation-merge-v1");
    // Three sites are named in all; the condition by three lists, the write
    // by two, the raise by one. The rejected annotation never counts.
    assert_eq!(
        (
            receipt["sites_named"].as_u64(),
            receipt["sites_kept"].as_u64()
        ),
        (Some(3), Some(2))
    );
    assert_eq!(receipt["lists"][0]["accepted"], 2);
    assert_eq!(
        receipt["lists"][2]["sites"], 2,
        "a site named twice in one list counts once"
    );
    let kept: Vec<Value> = serde_json::from_slice(&fs::read(&merged).unwrap()).unwrap();
    let kinds: Vec<&str> = kept.iter().map(|a| a["kind"].as_str().unwrap()).collect();
    assert_eq!(kinds, ["branch_condition", "state_write"]);

    let (code, receipt) = merge("3", &merged, &paths);
    assert_eq!((code, receipt["sites_kept"].as_u64()), (Some(0), Some(1)));
    let (code, receipt) = merge("1", &merged, &paths);
    assert_eq!((code, receipt["sites_kept"].as_u64()), (Some(0), Some(3)));
    // Nothing agreed on is exit 1; a threshold above the number of lists is bad input.
    let (code, _) = merge(
        "2",
        &merged,
        &paths[1..2]
            .iter()
            .chain(&paths[1..2])
            .cloned()
            .collect::<Vec<_>>()[..1],
    );
    assert_eq!(code, Some(3), "--min 2 with one list");
    let only_raise = project.dir.join("raise.json");
    fs::write(&only_raise, json!([raise]).to_string()).unwrap();
    let (code, receipt) = merge("2", &merged, &[paths[0].clone(), only_raise]);
    assert_eq!((code, receipt["sites_kept"].as_u64()), (Some(1), Some(0)));

    // The merged list drives a run.
    let (_, _) = merge("2", &merged, &paths);
    let (code, receipt, stderr) = project.run(
        &config,
        "work",
        &["--annotations", merged.to_str().unwrap()],
    );
    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(receipt["sites"], 2);
}

#[test]
fn unsafe_work_directories_are_refused_and_interrupted_copies_recovered() {
    let project = Project::new("safety");
    let config = project.config(&[], json!({}));
    let (code, receipt, stderr) = project.run(&config, "project/work", &[]);
    assert_eq!(code, Some(3));
    assert!(stderr.contains("must be outside"), "{stderr}");
    assert_eq!(receipt, Value::Null);
    assert!(!project.dir.join("project/work").exists());

    let (code, _, _) = project.run(&config, "work", &[]);
    assert_eq!(code, Some(0));
    // Leave a mutant planted, as a run killed in the middle would.
    let copy = project.dir.join("work/w1");
    fs::write(copy.join("original.saved"), APP).unwrap();
    let sha = lesscode_core::specimen_runner::sha256_hex_bytes(APP.as_bytes());
    fs::write(
        copy.join("journal.json"),
        json!({"file": "src/app.py", "sha256": sha}).to_string(),
    )
    .unwrap();
    fs::write(copy.join("tree/src/app.py"), "left behind").unwrap();
    let (code, receipt, stderr) = project.run(&config, "work", &["--no-reuse"]);
    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(receipt["workers"][0]["recovered"], "src/app.py");
    assert_eq!(
        fs::read_to_string(copy.join("tree/src/app.py")).unwrap(),
        APP
    );
}

/// `--only` runs the numbered mutants with the numbers and classes they have
/// in a full run; a number that does not exist is bad input.
#[test]
fn single_mutants_can_be_run_by_number() {
    let project = Project::new("only");
    let config = project.config(&[], json!({}));
    let (code, full, stderr) = project.run(&config, "full", &["--no-reuse"]);
    assert_eq!(code, Some(0), "{stderr}");
    let all = full["mutants"].as_array().unwrap();
    assert!(all.len() >= 2);
    let last = &all[all.len() - 1];
    let number = last["number"].as_u64().unwrap();

    let only = number.to_string();
    let (code, one, stderr) = project.run(&config, "one", &["--no-reuse", "--only", &only]);
    assert_eq!(code, Some(0), "{stderr}");
    let run = one["mutants"].as_array().unwrap();
    assert_eq!(run.len(), 1);
    for key in ["number", "site", "operator", "sha256", "class"] {
        assert_eq!(run[0][key], last[key], "{key}");
    }
    assert_eq!(one["mutants_not_selected"], all.len() as u64 - 1);
    assert_eq!(full.get("mutants_not_selected"), None);

    let (code, receipt, stderr) = project.run(&config, "none", &["--only", "9999"]);
    assert_eq!(code, Some(3), "{stderr}");
    assert_eq!(receipt, Value::Null);
    assert!(stderr.contains("there is no mutant 9999"), "{stderr}");
    assert_eq!(project.source(), APP, "the source tree was changed");
}

/// A working copy that is used again is brought up to date first: a test
/// added to the project after the first run must reach the copy, or the
/// mutants are run again against the tests as they were (found on a real
/// step, R93).
#[test]
fn a_changed_test_file_reaches_a_copy_that_is_used_again() {
    let project = Project::new("copy-current");
    let tests = project.dir.join("project/tests");
    fs::create_dir_all(&tests).unwrap();
    fs::write(tests.join("expect.txt"), "def step\n").unwrap();
    fs::write(tests.join("old.txt"), "").unwrap();
    let config = project.config(
        &["--expect-from", "tests/expect.txt"],
        json!({"test_paths": ["tests/"]}),
    );
    let (code, first, stderr) = project.run(&config, "work", &[]);
    assert_eq!(code, Some(0), "{stderr}");
    let survivors = |receipt: &Value| -> Vec<String> {
        let found = classes(receipt);
        let alive = found.iter().filter(|(_, class)| *class == "survived");
        alive.map(|(mutant, _)| mutant.clone()).collect()
    };
    let before = survivors(&first);
    assert!(
        before.iter().any(|m| m.starts_with("3:")),
        "nothing checks the raise yet: {before:?}"
    );

    fs::write(
        tests.join("expect.txt"),
        "def step\nraise ValueError(\"big\")\n",
    )
    .unwrap();
    fs::remove_file(tests.join("old.txt")).unwrap();
    let (code, second, stderr) = project.run(&config, "work", &[]);
    assert_eq!(code, Some(0), "{stderr}");
    let after = survivors(&second);
    assert!(
        !after.iter().any(|m| m.starts_with("3:")),
        "the new expectation did not reach the copy: {after:?}"
    );
    let copy = project.dir.join("work/w1/tree/tests");
    assert!(!copy.join("old.txt").exists(), "a removed file stayed");
    assert_eq!(project.source(), APP, "the source tree was changed");
}

/// A stored outcome is reused only while the test files are what they were
/// (gradual-refactoring-steps, Requirement 9): a changed or added file
/// under a test path makes every mutant run again without `--no-reuse`.
#[test]
fn a_changed_test_file_makes_the_mutants_run_again() {
    let project = Project::new("tests-identity");
    let tests = project.dir.join("project/tests");
    fs::create_dir_all(&tests).unwrap();
    fs::write(tests.join("check.txt"), "first\n").unwrap();
    let config = project.config(&[], json!({"test_paths": ["tests/"]}));
    let reused = |receipt: &Value| -> Vec<bool> {
        let mutants = receipt["mutants"].as_array().unwrap();
        mutants
            .iter()
            .map(|m| m["reused_from"].is_string())
            .collect()
    };

    let (code, first, stderr) = project.run(&config, "work", &[]);
    assert_eq!(code, Some(0), "{stderr}");
    assert!(reused(&first).iter().all(|r| !r));
    let hash = first["test_files_sha256"].as_str().unwrap().to_owned();

    let (_, same, _) = project.run(&config, "work", &[]);
    assert!(reused(&same).iter().all(|r| *r), "unchanged tests reuse");
    assert_eq!(same["test_files_sha256"], hash.as_str());

    fs::write(tests.join("check.txt"), "second\n").unwrap();
    let (code, changed, stderr) = project.run(&config, "work", &[]);
    assert_eq!(code, Some(0), "{stderr}");
    assert!(reused(&changed).iter().all(|r| !r), "a changed test reruns");
    assert_ne!(changed["test_files_sha256"], hash.as_str());
    assert_eq!(changed["config_sha256"], first["config_sha256"]);
    assert_eq!(classes(&changed), classes(&first));

    fs::write(tests.join("more.txt"), "").unwrap();
    let (_, added, _) = project.run(&config, "work", &[]);
    assert!(reused(&added).iter().all(|r| !r), "an added test reruns");

    // A file outside the test paths does not count.
    fs::write(project.dir.join("project/notes.txt"), "x").unwrap();
    let (_, other, _) = project.run(&config, "work", &[]);
    assert!(reused(&other).iter().all(|r| *r));

    // Without test paths there is no such hash, as before.
    let plain = project.config(&[], json!({}));
    let (_, receipt, _) = project.run(&plain, "plain", &[]);
    assert_eq!(receipt.get("test_files_sha256"), None);
}
