//! The run page: what a run reads, asked from Git without changing
//! anything (`.kiro/specs/run-dashboard`, requirement 4.2 and 4.5). The
//! page tells the person which repository, branch and commit a run is
//! about, whether work was uncommitted, and whether the branch is the
//! repository's main branch.

use super::page_snapshot::Subject;
use std::path::Path;
use std::process::{Command, Stdio};

/// The git commands this module may start. None of them writes to the
/// repository, and none takes a lock that another git process waits for.
const READ_ONLY: &[&str] = &["rev-parse", "symbolic-ref", "status"];

/// Names a main branch usually has, for a repository whose remote does not
/// say which one it is.
const USUAL: &[&str] = &["main", "master", "trunk"];

/// What git answers, or `None` when it fails or the place is no repository.
fn ask(repo: &Path, args: &[&str]) -> Option<String> {
    if !args.first().is_some_and(|a| READ_ONLY.contains(a)) {
        return None;
    }
    let done = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    let said = String::from_utf8_lossy(&done.stdout).trim().to_owned();
    done.status.success().then_some(said)
}

/// What the run reads. Every part that cannot be found out is `None`, and
/// the page then says "not known".
pub fn subject(repo: &Path) -> Subject {
    // A commit checked out without a branch answers `HEAD`.
    let branch = ask(repo, &["rev-parse", "--abbrev-ref", "HEAD"]).filter(|b| b != "HEAD");
    let remote = [
        "symbolic-ref",
        "--quiet",
        "--short",
        "refs/remotes/origin/HEAD",
    ];
    let named = ask(repo, &remote).and_then(|r| r.split_once('/').map(|(_, b)| b.to_owned()));
    let is_main = |branch: &String| match &named {
        Some(main) => branch == main,
        None => USUAL.contains(&branch.as_str()),
    };
    let changes = ["status", "--porcelain", "--untracked-files=no"];
    Subject {
        repository: ask(repo, &["rev-parse", "--show-toplevel"]),
        on_main_branch: branch.as_ref().map(is_main),
        branch,
        commit: ask(repo, &["rev-parse", "HEAD"]),
        dirty: ask(repo, &changes).map(|listed| !listed.is_empty()),
    }
}
