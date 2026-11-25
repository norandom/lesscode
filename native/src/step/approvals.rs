//! Approvals of write-offs in signed commits (spec signed-approvals).
//!
//! An approval is judged by the commit that last changed its entry in
//! `write-offs.json`: the newest commit on the branch's first-parent line
//! whose entry differs from its first parent's (task 1.5).

use super::revisions::{file_at, history};
use serde_json::Value;
use std::path::Path;

/// The entry for `file` and `fault` in a `write-offs.json`; none where the
/// file is absent, unreadable or has no such entry.
fn entry_in(bytes: Option<&[u8]>, file: &str, fault: &str) -> Option<Value> {
    let parsed: Value = serde_json::from_slice(bytes?).ok()?;
    let same = |w: &&Value| w["file"] == file && w["fault"] == fault;
    parsed["write_offs"].as_array()?.iter().find(same).cloned()
}

/// The entry for `file` and `fault` in `path` as it is at `revision`.
fn entry_at(
    repo: &Path,
    revision: Option<&str>,
    path: &str,
    file: &str,
    fault: &str,
) -> Result<Option<Value>, String> {
    let Some(revision) = revision else {
        return Ok(None);
    };
    let bytes = file_at(repo, revision, path)?;
    Ok(entry_in(bytes.as_deref(), file, fault))
}

/// The commit that last changed the entry for `file` and `fault` in `path`,
/// as `revision` has it: the newest commit whose entry differs from its
/// first parent's. A root commit compares with no entry. None where no
/// commit on the line ever had the entry.
pub fn last_change(
    repo: &Path,
    revision: &str,
    path: &str,
    file: &str,
    fault: &str,
) -> Result<Option<String>, String> {
    for (commit, parent) in history(repo, revision, path)? {
        let now = entry_at(repo, Some(&commit), path, file, fault)?;
        let before = entry_at(repo, parent.as_deref(), path, file, fault)?;
        if now != before {
            return Ok(Some(commit));
        }
    }
    Ok(None)
}
