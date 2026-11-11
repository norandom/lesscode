//! The step declaration (gradual-refactoring-steps, task 3.1).

use lesscode_core::step::declaration::Declaration;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn valid() -> Value {
    json!({
        "step": "one-exit", "spec": "one-exit", "branch": "step/one-exit", "base": "main",
        "language": "python", "scope": ["src/app.py"], "test_paths": ["tests/"],
        "test": ["python", "-m", "pytest"],
        "run": {"timeout_floor_s": 30, "workers": 2},
        "max_token_growth": 5,
        "out_of_reach": [{"function": "probe", "reason": "starts a process"}],
        "criteria": [{"id": "1.1", "command": ["python", "-m", "pytest", "-k", "exit"]}],
        "outputs": {"verdict": ".kiro/specs/one-exit/verdict.json",
                    "evidence": ".kiro/specs/one-exit/evidence.json",
                    "ledger": ".kiro/steps/ledger.jsonl"}
    })
}

fn parse(value: &Value) -> Result<Declaration, String> {
    Declaration::parse(value.to_string().as_bytes())
}

fn changed(key: &str, value: Value) -> Result<Declaration, String> {
    let mut declaration = valid();
    declaration[key] = value;
    parse(&declaration)
}

#[test]
fn a_declaration_gives_the_drivers_run_configuration() {
    let declaration = parse(&valid()).unwrap();
    assert_eq!(declaration.max_token_growth, 5);
    let functions = BTreeMap::from([("src/app.py".to_owned(), vec!["step".to_owned()])]);
    let config = declaration
        .run_config(Path::new("copy"), &functions)
        .unwrap();
    assert_eq!(config.language, "python");
    assert_eq!(config.files, ["src/app.py"]);
    assert_eq!(config.test_paths, ["tests/"]);
    assert_eq!(config.source_root, Path::new("copy"));
    assert_eq!((config.timeout_floor_s, config.workers), (30, 2));
    assert_eq!(config.functions, functions);
}

#[test]
fn wrong_declarations_say_what_is_wrong() {
    let cases = [
        (changed("step", json!(" ")), "step is empty"),
        (changed("scope", json!([])), "scope is empty"),
        (changed("scope", json!(["../x.py"])), "not a path inside"),
        (changed("scope", json!(["/abs/x.py"])), "not a path inside"),
        (changed("scope", json!(["tests/t.py"])), "under a test path"),
        (changed("test", json!([])), "test is empty"),
        (changed("unknown", json!(1)), "unknown field"),
        (
            changed("run", json!({"files": ["x"]})),
            "set by the declaration",
        ),
        (changed("run", json!({"no_such": 1})), "unknown field"),
        (changed("run", json!({"workers": 0})), "workers is 0"),
        (
            changed("out_of_reach", json!([{"function": "f", "reason": ""}])),
            "has no reason",
        ),
        (
            changed("criteria", json!([{"id": "1.1", "command": []}])),
            "needs an id and a command",
        ),
        (
            changed(
                "outputs",
                json!({"verdict": "../v.json", "evidence": "e.json", "ledger": "l.jsonl"}),
            ),
            "not a path inside",
        ),
    ];
    for (result, expected) in cases {
        let error = result.unwrap_err();
        assert!(error.contains(expected), "{expected}: {error}");
    }
    let mut missing = valid();
    missing.as_object_mut().unwrap().remove("outputs");
    assert!(parse(&missing).unwrap_err().contains("outputs"));
}

fn git(repo: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
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
    assert!(out.status.success(), "git {args:?}");
}

#[test]
fn the_base_must_be_an_ancestor_of_the_branch() {
    let repo: PathBuf =
        std::env::temp_dir().join(format!("lesscode-declaration-{}", std::process::id()));
    let _ = fs::remove_dir_all(&repo);
    fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "-q", "-b", "main"]);
    fs::write(repo.join("a.txt"), "1").unwrap();
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "base"]);
    git(&repo, &["branch", "step/one-exit"]);
    git(&repo, &["branch", "step/other"]);
    fs::write(repo.join("a.txt"), "2").unwrap();
    git(&repo, &["commit", "-q", "-am", "main moves on"]);

    let declaration = parse(&valid()).unwrap();
    let error = declaration.revisions(&repo).unwrap_err();
    assert!(error.contains("not an ancestor"), "{error}");

    let mut from_branch = valid();
    from_branch["base"] = json!("step/other");
    let revisions = parse(&from_branch).unwrap().revisions(&repo).unwrap();
    assert_eq!(revisions.base, revisions.step);
    assert_eq!(revisions.base.len(), 40);

    let mut unknown = valid();
    unknown["branch"] = json!("step/none");
    assert!(parse(&unknown).unwrap().revisions(&repo).is_err());
    let _ = fs::remove_dir_all(&repo);
}
