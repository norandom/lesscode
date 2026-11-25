//! Approvals of write-offs in signed commits (spec signed-approvals).
//!
//! An approval is judged by the commit that last changed its entry in
//! `write-offs.json`: the newest commit on the branch's first-parent line
//! whose entry differs from its first parent's (task 1.5).

use super::revisions::{file_at, history};
use serde::Serialize;
use serde_json::Value;
use std::path::Path;

/// An approval that counts: the key it was signed with, and its commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Signed {
    pub account: String,
    /// The key's title on the forge.
    pub title: String,
    pub fingerprint: String,
    pub commit: String,
    /// The commit's committer date, strict ISO 8601.
    pub date: String,
}

/// Why an approval does not count (requirement 1.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RefusalKind {
    NotSigned,
    DoesNotVerify,
    NotListed,
    NotOnForge,
}

/// An approval that does not count: why, its commit, the key it was signed
/// with where one verified, and how to repair it where that is known.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Refusal {
    pub kind: RefusalKind,
    pub commit: String,
    pub key: Option<String>,
    pub repair: Option<String>,
}

impl Refusal {
    /// What a fault whose write-off was refused waits for.
    pub fn words(&self) -> String {
        let commit = &self.commit[..self.commit.len().min(12)];
        let key = self.key.as_deref().unwrap_or("an unknown key");
        let said = match self.kind {
            RefusalKind::NotSigned => format!("its approval in {commit} is not signed"),
            RefusalKind::DoesNotVerify => {
                format!("the signature of its approval in {commit} does not verify")
            }
            RefusalKind::NotListed => format!(
                "its approval in {commit} is signed with {key}, which is not in the signer list"
            ),
            RefusalKind::NotOnForge => format!(
                "its approval in {commit} is signed with {key}, which is not among the \
                 account's signing keys on the forge"
            ),
        };
        match &self.repair {
            Some(repair) => format!("{said}: {repair}"),
            None => said,
        }
    }
}

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
