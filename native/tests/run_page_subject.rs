//! What the run page says about the person's code, asked from Git
//! (`.kiro/specs/run-dashboard`, task 1.2). Each test makes a small
//! repository of its own.

use lesscode_core::adapters::driver::page_log::{RunLog, StagePlan};
use lesscode_core::adapters::driver::page_subject::subject;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn git(repo: &Path, args: &[&str]) {
    let done = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        done.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&done.stderr)
    );
}

/// A repository with one commit on a branch of the given name.
fn repository(test: &str, branch: &str) -> PathBuf {
    let place = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("run_page_subject_{test}"));
    let _ = fs::remove_dir_all(&place);
    fs::create_dir_all(&place).unwrap();
    git(&place, &["init", "--quiet", "--initial-branch", branch]);
    git(&place, &["config", "user.email", "test@example.invalid"]);
    git(&place, &["config", "user.name", "A test"]);
    git(&place, &["config", "commit.gpgsign", "false"]);
    fs::write(place.join("a.txt"), "one\n").unwrap();
    git(&place, &["add", "a.txt"]);
    git(&place, &["commit", "--quiet", "-m", "first"]);
    place
}

#[test]
fn a_clean_tree_on_the_usual_main_branch() {
    let repo = repository("clean", "main");
    let seen = subject(&repo);
    assert_eq!(seen.branch.as_deref(), Some("main"));
    assert_eq!(seen.on_main_branch, Some(true));
    assert_eq!(seen.dirty, Some(false));
    assert_eq!(seen.commit.as_ref().map(String::len), Some(40));
    let top = seen.repository.expect("the repository's place");
    assert!(
        top.replace('\\', "/").ends_with("run_page_subject_clean"),
        "{top}"
    );
}

#[test]
fn a_changed_tracked_file_is_uncommitted_work_and_a_new_file_is_not() {
    let repo = repository("dirty", "main");
    fs::write(repo.join("untracked.txt"), "new\n").unwrap();
    assert_eq!(
        subject(&repo).dirty,
        Some(false),
        "an untracked file changes nothing that is tracked"
    );
    fs::write(repo.join("a.txt"), "two\n").unwrap();
    assert_eq!(subject(&repo).dirty, Some(true));
}

#[test]
fn another_branch_is_not_the_main_branch() {
    let repo = repository("branch", "main");
    git(&repo, &["checkout", "--quiet", "-b", "work/one"]);
    let seen = subject(&repo);
    assert_eq!(seen.branch.as_deref(), Some("work/one"));
    assert_eq!(seen.on_main_branch, Some(false));
}

#[test]
fn the_remote_says_which_branch_is_the_main_one() {
    let repo = repository("remote", "develop");
    assert_eq!(
        subject(&repo).on_main_branch,
        Some(false),
        "not a usual name, and no remote says so"
    );
    git(
        &repo,
        &["update-ref", "refs/remotes/origin/develop", "HEAD"],
    );
    git(
        &repo,
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/develop",
        ],
    );
    assert_eq!(subject(&repo).on_main_branch, Some(true));
    git(&repo, &["checkout", "--quiet", "-b", "main"]);
    assert_eq!(
        subject(&repo).on_main_branch,
        Some(false),
        "the remote's word goes before the usual names"
    );
}

#[test]
fn a_commit_without_a_branch_has_no_branch_to_judge() {
    let repo = repository("detached", "main");
    git(&repo, &["checkout", "--quiet", "--detach"]);
    let seen = subject(&repo);
    assert_eq!(seen.branch, None);
    assert_eq!(seen.on_main_branch, None);
    assert!(seen.commit.is_some());
}

#[test]
fn a_place_that_is_no_repository_is_not_known() {
    let none = format!("lc-run-page-subject-none-{}", std::process::id());
    let place = std::env::temp_dir().join(none);
    let _ = fs::remove_dir_all(&place);
    fs::create_dir_all(&place).unwrap();
    // The system's temporary directory lies outside every repository; the one
    // Cargo gives tests is inside this repository when the target dir is.
    let seen = subject(&place);
    assert_eq!(
        (
            seen.repository,
            seen.branch,
            seen.commit,
            seen.dirty,
            seen.on_main_branch
        ),
        (None, None, None, None, None)
    );
}

#[test]
fn the_snapshot_carries_what_the_run_says_about_the_code() {
    let repo = repository("snapshot", "main");
    let stages = [StagePlan {
        id: "copy",
        title: "Copy the project",
        what: "Copies are made.",
    }];
    let mut log = RunLog::new("step plan", "2.1.0", &stages, 0);
    log.set_subject(subject(&repo));
    log.set_copies("D:/work/copies", 3);
    log.set_commands(vec![vec!["cargo".into(), "test".into()]], true);
    log.set_writes(vec![".kiro/specs/one/verdict.json".into()]);
    log.set_space(3 * 1_073_741_824);
    let shown = serde_json::to_value(log.snapshot()).unwrap();
    assert_eq!(shown["subject"]["branch"], "main");
    assert_eq!(shown["subject"]["on_main_branch"], true);
    assert_eq!(shown["subject"]["dirty"], false);
    assert_eq!(shown["copies"]["directory"], "D:/work/copies");
    assert_eq!(shown["copies"]["planned"], 3);
    assert_eq!(shown["copies"]["bytes"], 3_221_225_472u64);
    assert_eq!(shown["commands"], serde_json::json!([["cargo", "test"]]));
    assert_eq!(
        shown["writes_in_repository"],
        serde_json::json!([".kiro/specs/one/verdict.json"])
    );
}
