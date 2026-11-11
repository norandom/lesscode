//! Reading a repository at a revision without changing it
//! (gradual-refactoring-steps, task 1.2).

use lesscode_core::step::revisions::{copy_tree, is_ancestor, resolve, READ_ONLY};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn scratch(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("lesscode-revisions-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Git as the test's own tool, to build a repository to read from.
fn git(repo: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args([
            "-c",
            "user.name=test",
            "-c",
            "user.email=test@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.autocrlf=false",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

/// Two commits: the second changes one file and adds another.
fn repository(name: &str) -> (PathBuf, String, String) {
    let repo = scratch(name);
    git(&repo, &["init", "-q", "-b", "main"]);
    fs::create_dir_all(repo.join("pkg/src")).unwrap();
    fs::write(repo.join("pkg/src/a.txt"), "one\r\ntwo\n").unwrap();
    fs::write(repo.join("pkg/bin.dat"), [0u8, 159, 146, 150, 10, 0]).unwrap();
    fs::write(repo.join("top.txt"), "outside the root\n").unwrap();
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "first"]);
    let first = git(&repo, &["rev-parse", "HEAD"]);
    fs::write(repo.join("pkg/src/a.txt"), "changed\n").unwrap();
    fs::write(repo.join("pkg/src/b.txt"), "").unwrap();
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "second"]);
    let second = git(&repo, &["rev-parse", "HEAD"]);
    (repo, first, second)
}

/// Every file below a directory with its bytes, and the refs and index.
fn state(repo: &Path) -> Vec<(String, Vec<u8>)> {
    let mut found = Vec::new();
    let mut stack = vec![repo.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let name = path
                    .strip_prefix(repo)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                found.push((name, fs::read(&path).unwrap()));
            }
        }
    }
    found.sort();
    found
}

#[test]
fn a_revision_is_copied_byte_for_byte_and_the_repository_is_untouched() {
    let (repo, first, second) = repository("copy");
    let before = state(&repo);
    let work = scratch("copy-work");

    assert_eq!(resolve(&repo, "HEAD").unwrap(), second);
    assert_eq!(resolve(&repo, &first[..8]).unwrap(), first);
    assert!(resolve(&repo, "no-such-revision").is_err());
    assert!(is_ancestor(&repo, &first, &second).unwrap());
    assert!(!is_ancestor(&repo, &second, &first).unwrap());

    // The first revision, although the working tree holds the second.
    let copied = copy_tree(&repo, &first, "pkg", &work.join("base")).unwrap();
    assert_eq!((copied.files, copied.skipped.len()), (2, 0));
    assert_eq!(
        fs::read(work.join("base/src/a.txt")).unwrap(),
        b"one\r\ntwo\n"
    );
    assert_eq!(
        fs::read(work.join("base/bin.dat")).unwrap(),
        [0u8, 159, 146, 150, 10, 0]
    );
    assert!(!work.join("base/top.txt").exists());
    assert!(!work.join("base/src/b.txt").exists());

    let copied = copy_tree(&repo, &second, "pkg", &work.join("step")).unwrap();
    assert_eq!(copied.files, 3);
    assert_eq!(fs::read(work.join("step/src/a.txt")).unwrap(), b"changed\n");
    assert_eq!(fs::read(work.join("step/src/b.txt")).unwrap(), b"");

    // The whole tree when no root is named.
    let copied = copy_tree(&repo, &second, "", &work.join("all")).unwrap();
    assert_eq!(copied.files, 4);
    assert!(work.join("all/top.txt").exists());

    // An existing destination is never written into.
    assert!(copy_tree(&repo, &second, "pkg", &work.join("step")).is_err());
    // A destination inside the repository is refused.
    assert!(copy_tree(&repo, &second, "pkg", &repo.join("copy-here")).is_err());
    assert!(!repo.join("copy-here").exists());

    assert_eq!(state(&repo), before, "the repository changed");
    let _ = fs::remove_dir_all(&repo);
    let _ = fs::remove_dir_all(&work);
}

#[test]
fn only_reading_git_commands_are_allowed() {
    let writing = [
        "commit",
        "checkout",
        "switch",
        "branch",
        "merge",
        "push",
        "reset",
        "worktree",
        "add",
        "stash",
        "tag",
        "rebase",
        "fetch",
        "pull",
        "update-ref",
        "archive",
    ];
    for name in writing {
        assert!(!READ_ONLY.contains(&name), "{name}");
    }
}
