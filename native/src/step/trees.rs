//! The files of two revisions compared, and the copies a step is judged
//! in. Everything is read from commits; a copy is always made afresh.

use super::revisions::{commits, entries, resolve, write_entries, Copied};
use crate::specimen_runner::sha256_hex_bytes;
use std::collections::BTreeMap;
use std::path::Path;

/// The plain files of a revision: path to the hash git gives its content.
pub type Tree = BTreeMap<String, String>;

pub fn tree(repo: &Path, revision: &str) -> Result<Tree, String> {
    let plain = entries(repo, revision, "")?
        .into_iter()
        .filter(|e| e.is_plain_file());
    Ok(plain.map(|e| (e.path, e.hash)).collect())
}

/// The paths whose content differs between two trees, or that are in one
/// of them only.
pub fn changed_paths(before: &Tree, after: &Tree) -> Vec<String> {
    let mut paths: Vec<&String> = before.keys().chain(after.keys()).collect();
    paths.sort();
    paths.dedup();
    let differs = |path: &&String| before.get(*path) != after.get(*path);
    paths.into_iter().filter(differs).cloned().collect()
}

/// Each commit after `base` up to `step` with the paths it changes against
/// its first parent.
pub fn commit_changes(
    repo: &Path,
    base: &str,
    step: &str,
) -> Result<Vec<(String, Vec<String>)>, String> {
    let mut found = Vec::new();
    for commit in commits(repo, base, step)? {
        let before = match resolve(repo, &format!("{commit}^")) {
            Ok(parent) => tree(repo, &parent)?,
            Err(_) => Tree::new(),
        };
        let paths = changed_paths(&before, &tree(repo, &commit)?);
        found.push((commit, paths));
    }
    Ok(found)
}

/// One hash over the files of a tree that `selected` picks, by path and
/// content hash.
pub fn digest(tree: &Tree, selected: impl Fn(&str) -> bool) -> String {
    let lines: Vec<String> = tree
        .iter()
        .filter(|(path, _)| selected(path))
        .map(|(path, hash)| format!("{path}\t{hash}"))
        .collect();
    sha256_hex_bytes(lines.join("\n").as_bytes())
}

fn emptied(dest: &Path) -> Result<(), String> {
    match dest.exists() {
        true => std::fs::remove_dir_all(dest).map_err(|e| format!("{}: {e}", dest.display())),
        false => Ok(()),
    }
}

/// A fresh copy of the whole repository at a revision; an earlier copy at
/// `dest` is replaced.
pub fn fresh_copy(repo: &Path, revision: &str, dest: &Path) -> Result<Copied, String> {
    emptied(dest)?;
    write_entries(repo, entries(repo, revision, "")?, "", dest)
}

/// A fresh copy with the code of `base` and the test files of `step`:
/// what a pin is judged in.
pub fn pin_copy(
    repo: &Path,
    base: &str,
    step: &str,
    is_test: impl Fn(&str) -> bool,
    dest: &Path,
) -> Result<Copied, String> {
    emptied(dest)?;
    let code = entries(repo, base, "")?
        .into_iter()
        .filter(|e| !is_test(&e.path));
    let tests = entries(repo, step, "")?
        .into_iter()
        .filter(|e| is_test(&e.path));
    write_entries(repo, code.chain(tests).collect(), "", dest)
}
