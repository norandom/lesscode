//! `step guard` and `step record` through the binary
//! (gradual-refactoring-steps, tasks 5.1 and 5.2).

mod step_support;

use serde_json::{json, Value};
use std::fs;
use step_support::{Repo, APP};

const PINNED: &str = "if a > 3:\nstate.total = a\n";
const DECLARATION: &str = ".kiro/specs/smaller/step.json";
const LEDGER: &str = ".kiro/steps/ledger.jsonl";

fn guard(repo: &Repo, branch: &str) -> (Option<i32>, String) {
    repo.cli(&["guard", "--branch", branch], "")
}

fn record(repo: &Repo, more: &[&str]) -> (Option<i32>, String) {
    let mut args = vec!["record", "--declaration", DECLARATION];
    args.extend(more);
    repo.cli(&args, "")
}

fn ledger(repo: &Repo) -> Vec<Value> {
    let text = fs::read_to_string(repo.dir.join(LEDGER)).unwrap_or_default();
    text.lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

/// A repository with one step that is pinned, changed and checked.
fn checked(name: &str) -> Repo {
    let repo = Repo::new(name);
    repo.begin("smaller", json!({}));
    repo.commit(&[("tests/expect.txt", PINNED)], "pin the write");
    let simpler = APP.replace("raise ValueError(\"big\")", "raise ValueError");
    repo.commit(&[("src/app.py", &simpler)], "the change");
    let (code, _, said) = repo.step("check", "smaller", &[]);
    assert_eq!(code, Some(0), "{said}");
    repo
}

#[test]
fn a_branch_stands_only_with_a_verdict_for_exactly_its_content() {
    let repo = checked("guard");
    let before = repo.state();
    let (code, said) = guard(&repo, "step/smaller");
    assert_eq!(code, Some(0), "{said}");
    assert!(said.contains("step/smaller: accepted"), "{said}");
    assert_eq!(repo.state(), before, "the guard wrote something");

    // A branch no declaration names is not a step branch.
    let (code, said) = guard(&repo, "main");
    assert_eq!(code, Some(0), "{said}");
    assert!(said.contains("main: not a step branch"), "{said}");

    // On the ledger; a second record adds nothing.
    let (code, said) = record(&repo, &[]);
    assert_eq!(code, Some(0), "{said}");
    let (code, said) = record(&repo, &[]);
    assert_eq!(code, Some(0), "{said}");
    assert!(said.contains("recorded already"), "{said}");
    let entries = ledger(&repo);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["result"], "accepted");
    assert_eq!(entries[0]["previous"], "");
    assert_eq!(entries[0]["functions"][0]["function"], "step");
    assert_eq!(entries[0]["verdict_sha256"].as_str().unwrap().len(), 64);

    // Committing verdict and ledger moves the branch, not what was judged.
    repo.commit(&[], "verdict and ledger");
    let (code, said) = guard(&repo, "step/smaller");
    assert_eq!(code, Some(0), "{said}");

    // As a pre-push hook: the pushed branches come on standard input.
    let tip = repo.git(&["rev-parse", "HEAD"]);
    let zero = "0".repeat(40);
    let input = format!(
        "refs/heads/step/smaller {tip} refs/heads/step/smaller {zero}\nrefs/heads/gone {zero} refs/heads/gone {tip}\nrefs/tags/v1 {tip} refs/tags/v1 {zero}\n"
    );
    let (code, said) = repo.cli(&["guard", "--pushed"], &input);
    assert_eq!(code, Some(0), "{said}");
    assert_eq!(said.matches("accepted").count(), 1, "{said}");

    // A test changed after the verdict: stale.
    repo.commit(&[("tests/expect.txt", "if a > 3:\n")], "drop the pin");
    let (code, said) = guard(&repo, "step/smaller");
    assert_eq!(code, Some(1), "{said}");
    assert!(
        said.contains("refused: a test file changed after the verdict"),
        "{said}"
    );
    let (code, said) = record(&repo, &[]);
    assert_eq!(code, Some(1), "a stale verdict is not recorded: {said}");

    // An override needs a reason, is on the ledger, and is never "accepted".
    let (code, _) = record(&repo, &["--override"]);
    assert_eq!(code, Some(3));
    let (code, _) = record(&repo, &["--override", "--reason", "  "]);
    assert_eq!(code, Some(3));
    let (code, said) = record(&repo, &["--override", "--reason", "hotfix, pin follows"]);
    assert_eq!(code, Some(0), "{said}");
    // The guard judges what is committed: the override counts once the
    // ledger that holds it is.
    let (code, _) = guard(&repo, "step/smaller");
    assert_eq!(code, Some(1));
    repo.commit(&[], "ledger with the override");
    let (code, said) = guard(&repo, "step/smaller");
    assert_eq!(code, Some(0), "{said}");
    assert!(
        said.contains("overridden (hotfix, pin follows)") && !said.contains("accepted"),
        "{said}"
    );
    let entries = ledger(&repo);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[1]["result"], "overridden");
    assert_eq!(entries[1]["reason"], "hotfix, pin follows");

    // The override holds for that content only.
    repo.commit(&[("src/app.py", APP)], "undo the change");
    let (code, said) = guard(&repo, "step/smaller");
    assert_eq!(code, Some(1), "{said}");
    assert!(
        said.contains("src/app.py changed after the verdict"),
        "{said}"
    );

    // A step that was never checked.
    repo.begin("unchecked", json!({}));
    let (code, said) = guard(&repo, "step/unchecked");
    assert_eq!(code, Some(1), "{said}");
    assert!(said.contains("there is no verdict"), "{said}");
}

#[test]
fn the_ledger_holds_the_best_number_and_cannot_be_rewritten() {
    let repo = checked("ledger");
    let before = repo.state();
    let (code, said) = record(&repo, &[]);
    assert_eq!(code, Some(0), "{said}");
    // Recording writes the ledger and nothing else.
    let after = repo.state();
    let new: Vec<&str> = after
        .iter()
        .filter(|entry| !before.contains(entry))
        .map(|(name, _)| name.as_str())
        .collect();
    assert_eq!(new, [LEDGER]);
    assert!(before.iter().all(|entry| after.contains(entry)));
    repo.commit(&[], "verdict and ledger");
    let held = ledger(&repo)[0]["functions"][0]["after"]["unnoticed"]
        .as_u64()
        .unwrap();

    // The step is merged; later the pin is dropped on main outside any step.
    repo.git(&["checkout", "-q", "main"]);
    repo.git(&["merge", "-q", "--ff-only", "step/smaller"]);
    repo.commit(&[("tests/expect.txt", "if a > 3:\n")], "tests weakened");

    // A step that changes nothing is equal to its base, and is still
    // refused, because the function was better protected once.
    repo.begin("later", json!({}));
    let (code, said, _) = repo.step("check", "later", &[]);
    assert_eq!(code, Some(1), "{said}");
    let expected = format!(
        "src/app.py: step: unnoticed faults rise from {held} to {}",
        held + 1
    );
    assert!(said.contains(&expected), "{said}");

    // A release lists the steps since the one before.
    repo.git(&["checkout", "-q", "step/smaller"]);
    let (code, said) = repo.cli(&["record", "--release", "1.2.0", "--ledger", LEDGER], "");
    assert_eq!(code, Some(0), "{said}");
    assert!(
        said.contains("release 1.2.0: 1 steps") && said.contains("smaller"),
        "{said}"
    );
    let (_, said) = repo.cli(&["record", "--release", "1.3.0", "--ledger", LEDGER], "");
    assert!(said.contains("release 1.3.0: 0 steps"), "{said}");
    let entries = ledger(&repo);
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[1]["steps"], json!(["smaller"]));

    // An edited earlier entry stops every later use of the ledger.
    let path = repo.dir.join(LEDGER);
    let text = fs::read_to_string(&path).unwrap();
    fs::write(&path, text.replacen("\"smaller\"", "\"bigger\"", 1)).unwrap();
    let (code, said) = repo.cli(&["record", "--release", "1.4.0", "--ledger", LEDGER], "");
    assert_eq!(code, Some(3), "{said}");
    assert!(said.contains("the ledger was rewritten"), "{said}");
    assert_eq!(
        fs::read_to_string(&path).unwrap().lines().count(),
        3,
        "nothing was appended"
    );
}

/// The hook stops a real push by exit code, and lets a judged step through.
#[test]
fn the_installed_hook_stops_a_push_without_a_matching_verdict() {
    let repo = checked("hook");
    let remote = repo.dir.parent().unwrap().join("remote.git");
    let remote_text = remote.to_string_lossy().into_owned();
    repo.git(&["init", "-q", "--bare", &remote_text]);
    repo.git(&["remote", "add", "origin", &remote_text]);

    let (code, said) = repo.cli(&["install-hook"], "");
    assert_eq!(code, Some(0), "{said}");
    let hook = fs::read_to_string(repo.dir.join(".git/hooks/pre-push")).unwrap();
    assert!(hook.contains("exec lesscode-core step pre-push"));
    // A hook that is there already is not replaced unasked.
    let (code, said) = repo.cli(&["install-hook"], "");
    assert_eq!(code, Some(1), "{said}");
    let (code, _) = repo.cli(&["install-hook", "--force"], "");
    assert_eq!(code, Some(0));

    // The hook finds the binary on the path, as an installed one would be.
    let binary = std::path::Path::new(env!("CARGO_BIN_EXE_lesscode-core"));
    let mut paths = vec![binary.parent().unwrap().to_path_buf()];
    paths.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap()));
    let path = std::env::join_paths(paths).unwrap();
    let push = |branch: &str| {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(&repo.dir)
            .args(["push", "-q", "origin", branch])
            .env("PATH", &path)
            .output()
            .unwrap();
        let said = [out.stdout, out.stderr].concat();
        (
            out.status.success(),
            String::from_utf8_lossy(&said).into_owned(),
        )
    };
    let on_remote = |branch: &str| {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(&remote)
            .args(["rev-parse", "--verify", "--quiet", branch])
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).trim().to_owned()
    };

    // The judged step goes through; so does a branch that is no step.
    let (pushed, said) = push("step/smaller");
    assert!(pushed, "{said}");
    assert!(said.contains("step/smaller: accepted"), "{said}");
    assert!(said.contains("Pre-push verification passed"), "{said}");
    assert_eq!(
        on_remote("step/smaller"),
        repo.git(&["rev-parse", "step/smaller"])
    );
    let (pushed, said) = push("main");
    assert!(pushed, "{said}");

    // Code changed after the verdict: the push is stopped, nothing arrives.
    let judged = repo.git(&["rev-parse", "step/smaller"]);
    repo.commit(&[("src/app.py", APP)], "one more change");
    let (pushed, said) = push("step/smaller");
    assert!(!pushed, "{said}");
    assert!(said.contains("step/smaller: refused"), "{said}");
    // A refused push is never announced as passed.
    assert!(!said.contains("verification passed"), "{said}");
    assert_eq!(on_remote("step/smaller"), judged);
}

/// The step tooling starts git only through the module that lists the
/// reading commands, and starts nothing else itself.
#[test]
fn only_the_revisions_module_starts_git() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/step");
    let mut scanned = 0;
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let text = fs::read_to_string(&path).unwrap();
        let starts = text.contains("Command::new");
        let is_revisions = path.file_name().is_some_and(|n| n == "revisions.rs");
        assert_eq!(starts, is_revisions, "{}", path.display());
        // Other programs run only where the project's own commands are
        // run in a copy, and for the spec audit of the hook.
        let runs = text.contains("process::run");
        let may_run = ["check_runs.rs", "hook.rs"];
        let allowed = path
            .file_name()
            .is_some_and(|n| may_run.iter().any(|m| n == *m));
        assert!(!runs || allowed, "{}", path.display());
        scanned += 1;
    }
    assert!(scanned >= 10, "scanned {scanned} files");
}
