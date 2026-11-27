//! The run page: what a run reads, asked from Git without changing
//! anything (`.kiro/specs/run-dashboard`, requirement 4.2 and 4.5). The
//! page tells the person which repository, branch and commit a run is
//! about, whether work was uncommitted, and whether the branch is the
//! repository's main branch.

use super::snapshot::Subject;
use std::path::Path;
use std::process::{Command, Stdio};

/// The git commands this module may start. None of them writes to the
/// repository, and none takes a lock that another git process waits for.
const READ_ONLY: &[&str] = &["rev-parse", "symbolic-ref", "status", "check-ignore"];

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

/// Whether a file written at `place` would show up among a repository's
/// files: it lies inside a repository and Git is not told to ignore it.
pub fn would_be_tracked(place: &Path) -> bool {
    let beside = match place.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    let Some(name) = place.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    let inside = ask(beside, &["rev-parse", "--show-toplevel"]).is_some();
    // `check-ignore` succeeds when the name is ignored.
    inside && ask(beside, &["check-ignore", "--quiet", "--", name]).is_none()
}

/// Those of `places` that lie inside the repository `repo` belongs to,
/// named from the repository's top. Empty where there is no repository.
pub fn written_inside(repo: &Path, places: &[&Path]) -> Vec<String> {
    let top = ask(repo, &["rev-parse", "--show-toplevel"]);
    let Some(top) = top.and_then(|t| std::fs::canonicalize(t).ok()) else {
        return Vec::new();
    };
    let named = |place: &&Path| {
        let beside = match place.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => parent,
            _ => Path::new("."),
        };
        let full = std::fs::canonicalize(beside).ok()?.join(place.file_name()?);
        let inside = full.strip_prefix(&top).ok()?;
        Some(inside.to_string_lossy().replace('\\', "/"))
    };
    places.iter().filter_map(named).collect()
}

/// The space the files under `dir` take, or `None` when it cannot be read
/// to the end. Links are not followed.
pub fn space(dir: &Path) -> Option<u64> {
    let mut sum = 0;
    for entry in std::fs::read_dir(dir).ok()? {
        let entry = entry.ok()?;
        let kind = entry.file_type().ok()?;
        if kind.is_dir() {
            sum += space(&entry.path())?;
        } else if kind.is_file() {
            sum += entry.metadata().ok()?.len();
        }
    }
    Some(sum)
}

/// Whether `branch` is the repository's main branch: the one its remote
/// names, or one of the usual names where the remote does not say.
fn is_main(repo: &Path, branch: &str) -> bool {
    let remote = [
        "symbolic-ref",
        "--quiet",
        "--short",
        "refs/remotes/origin/HEAD",
    ];
    let named = ask(repo, &remote).and_then(|r| r.split_once('/').map(|(_, b)| b.to_owned()));
    match named {
        Some(main) => branch == main,
        None => USUAL.contains(&branch),
    }
}

/// What the run reads. Every part that cannot be found out is `None`, and
/// the page then says "not known".
pub fn subject(repo: &Path) -> Subject {
    // A commit checked out without a branch answers `HEAD`.
    let branch = ask(repo, &["rev-parse", "--abbrev-ref", "HEAD"]).filter(|b| b != "HEAD");
    let changes = ["status", "--porcelain", "--untracked-files=no"];
    Subject {
        repository: ask(repo, &["rev-parse", "--show-toplevel"]),
        on_main_branch: branch.as_ref().map(|b| is_main(repo, b)),
        branch,
        commit: ask(repo, &["rev-parse", "HEAD"]),
        dirty: ask(repo, &changes).map(|listed| !listed.is_empty()),
        compared_with: None,
    }
}

/// What a run that judges a change reads: the change's branch and commit,
/// whatever is checked out, and the commit it is compared with.
pub fn change(repo: &Path, branch: &str, commit: &str, compared_with: &str) -> Subject {
    Subject {
        on_main_branch: Some(is_main(repo, branch)),
        branch: Some(branch.to_owned()),
        commit: Some(commit.to_owned()),
        compared_with: Some(compared_with.to_owned()),
        ..subject(repo)
    }
}
