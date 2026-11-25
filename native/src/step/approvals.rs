//! Approvals of write-offs in signed commits (spec signed-approvals).
//!
//! An approval is judged by the commit that last changed its entry in
//! `write-offs.json`: the newest commit on the branch's first-parent line
//! whose entry differs from its first parent's (task 1.5). It counts when
//! that commit carries a good SSH signature by a key that is both in the
//! branch's signer list and among the account's signing keys on the forge
//! (task 2.2).

use super::context::Context;
use super::forge::{Forge, ForgeKey};
use super::protect_numbers::WriteOff;
use super::revisions::{commit_object, committer_date, file_at, history};
use super::signers::{self, Listed, SignerList};
use super::sshsig::{check, fingerprint, split, Checked};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
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

/// The entries of `path` as it is at `revision`; none where the revision,
/// the file or its list is absent or unreadable.
fn entries_at(repo: &Path, revision: Option<&str>, path: &str) -> Result<Vec<Value>, String> {
    let Some(revision) = revision else {
        return Ok(Vec::new());
    };
    let bytes = file_at(repo, revision, path)?;
    let parsed = bytes.and_then(|b| serde_json::from_slice::<Value>(&b).ok());
    let list = parsed.and_then(|p| p["write_offs"].as_array().cloned());
    Ok(list.unwrap_or_default())
}

/// The entry for `file` and `fault` among `entries`.
fn entry_of<'e>(entries: &'e [Value], (file, fault): &(&str, &str)) -> Option<&'e Value> {
    entries
        .iter()
        .find(|w| w["file"] == *file && w["fault"] == *fault)
}

/// For each (file, fault), the commit that last changed its entry in
/// `path`, as `revision` has it: the newest commit whose entry differs from
/// its first parent's. A root commit compares with no entry. None where no
/// commit on the line ever had the entry. The history is read once.
pub fn last_changes(
    repo: &Path,
    revision: &str,
    path: &str,
    wanted: &[(&str, &str)],
) -> Result<Vec<Option<String>>, String> {
    let mut found: Vec<Option<String>> = vec![None; wanted.len()];
    for (commit, parent) in history(repo, revision, path)? {
        if found.iter().all(Option::is_some) {
            break;
        }
        let now = entries_at(repo, Some(&commit), path)?;
        let before = entries_at(repo, parent.as_deref(), path)?;
        for (slot, entry) in found.iter_mut().zip(wanted) {
            if slot.is_none() && entry_of(&now, entry) != entry_of(&before, entry) {
                *slot = Some(commit.clone());
            }
        }
    }
    Ok(found)
}

/// The commit that last changed the entry for `file` and `fault`.
pub fn last_change(
    repo: &Path,
    revision: &str,
    path: &str,
    file: &str,
    fault: &str,
) -> Result<Option<String>, String> {
    Ok(last_changes(repo, revision, path, &[(file, fault)])?
        .pop()
        .flatten())
}

/// What the judge found beside the write-offs it filled in.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Judged {
    /// The signer list, where a good signature needed it.
    pub list: Option<SignerList>,
    /// How many of the listed keys the forge has, where it was asked.
    pub on_forge: Option<usize>,
    /// The approvals that count.
    pub counted: usize,
}

/// Bad input is `Err` (exit 3); the inner `Err` is "not judged" (exit 2).
pub type Judgement = Result<Result<Judged, String>, String>;

const AMEND: &str = "sign it with `git commit --amend -S --no-edit`";
const TWICE: &str = "set `approved` to false in one commit and back to true in a signed one";

/// What one approving commit's signature says before the list and the forge.
#[derive(Clone)]
enum Seen {
    Refused(RefusalKind),
    Good(String),
}

/// Take the commit apart and check its signature. The inner `Err` is a
/// program that cannot be started: not judged.
fn seen(repo: &Path, commit: &str) -> Result<Result<Seen, String>, String> {
    let split = split(&commit_object(repo, commit)?);
    let signature = split.signature.as_deref().filter(|_| split.is_ssh());
    let Some(signature) = signature else {
        return Ok(Ok(Seen::Refused(RefusalKind::NotSigned)));
    };
    Ok(
        check(&split.payload, signature).map(|checked| match checked {
            Checked::Good { fingerprint } => Seen::Good(fingerprint),
            Checked::Bad(_) => Seen::Refused(RefusalKind::DoesNotVerify),
        }),
    )
}

/// The title of each of the forge's keys, by fingerprint.
fn titles(keys: &[ForgeKey]) -> BTreeMap<String, String> {
    let titled = |k: &ForgeKey| {
        let blob = k.key.split_whitespace().nth(1)?;
        Some((fingerprint(blob).ok()?, k.title.clone()))
    };
    keys.iter().filter_map(titled).collect()
}

/// A refusal, with the key where one verified, and the repair of an
/// unsigned approval: amend where its commit is the branch's last, else
/// approve again.
fn refused(kind: RefusalKind, commit: String, key: Option<String>, head: &str) -> Refusal {
    let repair = (kind == RefusalKind::NotSigned).then(|| match commit == head {
        true => AMEND.to_owned(),
        false => TWICE.to_owned(),
    });
    Refusal {
        kind,
        commit,
        key,
        repair,
    }
}

/// A marked write-off whose approving commit carries a good signature.
struct Good {
    index: usize,
    commit: String,
    fingerprint: String,
}

/// The last change of every marked write-off and its signature: refusals
/// are filled in, good signatures returned. The inner `Err` is not judged.
fn signatures(
    repo: &Path,
    head: &str,
    path: &str,
    offs: &mut [WriteOff],
) -> Result<Result<Vec<Good>, String>, String> {
    let marked: Vec<usize> = (0..offs.len()).filter(|&i| offs[i].marked()).collect();
    if marked.is_empty() {
        return Ok(Ok(Vec::new()));
    }
    let wanted: Vec<(&str, &str)> = marked
        .iter()
        .map(|&i| (offs[i].file.as_str(), offs[i].fault.as_str()))
        .collect();
    let commits = last_changes(repo, head, path, &wanted)?;
    let mut by_commit: BTreeMap<String, Seen> = BTreeMap::new();
    let mut good = Vec::new();
    for (index, commit) in marked.into_iter().zip(commits) {
        let w = &offs[index];
        let none = || {
            format!(
                "{path}: no commit has the write-off for {} {}",
                w.file, w.fault
            )
        };
        let commit = commit.ok_or_else(none)?;
        if !by_commit.contains_key(&commit) {
            match seen(repo, &commit)? {
                Ok(s) => by_commit.insert(commit.clone(), s),
                Err(why) => return Ok(Err(why)),
            };
        }
        match by_commit[&commit].clone() {
            Seen::Good(fingerprint) => good.push(Good {
                index,
                commit,
                fingerprint,
            }),
            Seen::Refused(kind) => offs[index].refusal = Some(refused(kind, commit, None, head)),
        }
    }
    Ok(Ok(good))
}

/// Ask the forge once for the account's keys: each good signature by one
/// of them is counted with the key's title, any other refused by its
/// fingerprint. How many listed keys the forge has, and how many counted.
fn against_forge(
    repo: &Path,
    head: &str,
    list: &SignerList,
    good: Vec<Good>,
    offs: &mut [WriteOff],
    forge: &dyn Forge,
) -> Result<Result<(usize, usize), String>, String> {
    let titles = match forge.signing_keys(&list.account) {
        Ok(keys) => titles(&keys),
        Err(why) => return Ok(Err(why.to_string())),
    };
    let on_forge = list
        .keys
        .iter()
        .filter(|k| titles.contains_key(&k.fingerprint));
    let mut counted = 0;
    for Good {
        index,
        commit,
        fingerprint,
    } in good
    {
        let Some(title) = titles.get(&fingerprint) else {
            let refusal = refused(RefusalKind::NotOnForge, commit, Some(fingerprint), head);
            offs[index].refusal = Some(refusal);
            continue;
        };
        offs[index].approval = Some(Signed {
            account: list.account.clone(),
            title: title.clone(),
            fingerprint,
            date: committer_date(repo, &commit)?,
            commit,
        });
        counted += 1;
    }
    Ok(Ok((on_forge.count(), counted)))
}

/// Fill in `approval` or `refusal` of every write-off of `path` marked
/// approved: its last change, that commit's signature, the signer list read
/// once when a good signature needs it, the forge asked once when a listed
/// key needs it. A stage with no marked write-off reads neither.
pub fn judge(context: &Context, path: &str, offs: &mut [WriteOff], forge: &dyn Forge) -> Judgement {
    let (repo, head) = (&context.repo, &context.revisions.step);
    for w in offs.iter_mut() {
        (w.approval, w.refusal) = (None, None);
    }
    let good = match signatures(repo, head, path, offs)? {
        Ok(good) => good,
        Err(why) => return Ok(Err(why)),
    };
    if good.is_empty() {
        return Ok(Ok(Judged::default()));
    }
    let list = match signers::read(context)? {
        Listed::List(list) => list,
        Listed::NotJudged(why) => return Ok(Err(why)),
    };
    let listed = |g: &Good| list.keys.iter().any(|k| k.fingerprint == g.fingerprint);
    let (good, unlisted): (Vec<Good>, Vec<Good>) = good.into_iter().partition(listed);
    for g in unlisted {
        let refusal = refused(RefusalKind::NotListed, g.commit, Some(g.fingerprint), head);
        offs[g.index].refusal = Some(refusal);
    }
    let mut judged = Judged::default();
    if !good.is_empty() {
        match against_forge(repo, head, &list, good, offs, forge)? {
            Ok((on_forge, counted)) => {
                (judged.on_forge, judged.counted) = (Some(on_forge), counted)
            }
            Err(why) => return Ok(Err(why)),
        }
    }
    judged.list = Some(list);
    Ok(Ok(judged))
}
