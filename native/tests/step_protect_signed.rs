//! Approvals of write-offs in signed commits (spec signed-approvals). Keys
//! are made per test, without a passphrase; the forge is a stand-in.

mod step_support;

use lesscode_core::step::forge::{Forge, ForgeError};
use lesscode_core::step::{approvals, revisions, sshsig};
use std::process::Command;
use step_support::signing::{Key, StandIn};
use step_support::Repo;

/// Git's own check of one commit against an allowed-signers file naming
/// `key`, set for this command only: whether Git calls it good.
fn git_accepts(repo: &Repo, key: &Key, commit: &str) -> bool {
    let allowed = repo.dir.join(".git").join("allowed-for-the-test");
    let line = format!(
        "test@example.invalid namespaces=\"git\" {}\n",
        key.public_key()
    );
    std::fs::write(&allowed, line).unwrap();
    let setting = format!("gpg.ssh.allowedSignersFile={}", allowed.display());
    let done = Command::new("git")
        .arg("-C")
        .arg(&repo.dir)
        .args(["-c", &setting, "verify-commit", commit])
        .env_remove("SSH_AUTH_SOCK")
        .output()
        .unwrap();
    done.status.success()
}

#[test]
fn the_support_makes_signed_unsigned_and_grafted_commits() {
    let repo = Repo::new("signed-support");
    let keys = repo.dir.parent().unwrap().join("keys");
    let (owner, agent) = (Key::new(&keys, "owner"), Key::new(&keys, "agent"));
    assert_ne!(owner.fingerprint(), agent.fingerprint());

    let signed = repo.signed_commit(&owner, &[("notes.txt", "approved\n")], "signed");
    let unsigned = repo.commit(&[("notes.txt", "changed\n")], "unsigned");
    let grafted = repo.grafted(&signed, &unsigned);

    assert!(git_accepts(&repo, &owner, &signed), "the signed commit");
    assert!(
        !git_accepts(&repo, &agent, &signed),
        "signed with another key"
    );
    assert!(
        !git_accepts(&repo, &owner, &unsigned),
        "the unsigned commit"
    );
    assert!(
        !git_accepts(&repo, &owner, &grafted),
        "a signature over other content"
    );

    // Signing was switched on for one command; the repository keeps no setting.
    let local = repo.git(&["config", "--local", "--list"]);
    assert!(!local.contains("gpg."), "{local}");
    assert!(!local.contains("signingkey"), "{local}");
}

#[test]
fn the_stand_in_forge_lists_keys_under_their_titles() {
    let dir = std::env::temp_dir().join(format!("lc-standin-{}", std::process::id()));
    let owner = Key::new(&dir, "owner");
    let forge = StandIn::listing(&[(&owner, "lesscode approvals")]);
    let keys = forge.signing_keys("norandom").unwrap();
    assert_eq!(forge.asked.get(), 1);
    assert_eq!(keys, forge.keys());
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].title, "lesscode approvals");
    assert!(keys[0].key.starts_with("ssh-ed25519 "), "{}", keys[0].key);
    assert!(
        !keys[0].key.contains("owner"),
        "the comment is not part of the key"
    );
    let unreachable = StandIn::unreachable().signing_keys("norandom");
    assert!(matches!(unreachable, Err(ForgeError::Unreachable(_))));
    let _ = std::fs::remove_dir_all(dir);
}

const OFFS: &str = ".kiro/specs/protect-toy/write-offs.json";

/// A `write-offs.json` with one entry per (fault, reason, approved).
fn offs(entries: &[(&str, &str, bool)]) -> String {
    let entry = |(fault, reason, approved): &(&str, &str, bool)| {
        serde_json::json!({ "file": "src/app.py", "function": "helper", "fault": fault,
            "kind": "nothing_a_caller_sees", "reason": reason, "approved": approved })
    };
    let all: Vec<_> = entries.iter().map(entry).collect();
    serde_json::to_string_pretty(&serde_json::json!({ "schema": 1, "write_offs": all })).unwrap()
}

fn last(repo: &Repo, fault: &str) -> Option<String> {
    approvals::last_change(&repo.dir, "HEAD", OFFS, "src/app.py", fault).unwrap()
}

fn keys_beside(repo: &Repo) -> Key {
    Key::new(&repo.dir.parent().unwrap().join("keys"), "owner")
}

#[test]
fn the_signature_git_made_is_taken_apart_and_checked() {
    let repo = Repo::new("signed-split");
    let owner = keys_beside(&repo);
    let signed = repo.signed_commit(&owner, &[("notes.txt", "x\n")], "signed\n\n body");
    let split = sshsig::split(&revisions::commit_object(&repo.dir, &signed).unwrap());
    assert!(split.is_ssh());
    let checked = sshsig::check(&split.payload, split.signature.as_deref().unwrap()).unwrap();
    let fingerprint = owner.fingerprint();
    assert_eq!(checked, sshsig::Checked::Good { fingerprint });
    let unsigned = repo.commit(&[("notes.txt", "y\n")], "unsigned");
    let raw = revisions::commit_object(&repo.dir, &unsigned).unwrap();
    assert_eq!(sshsig::split(&raw).signature, None);
    let grafted = repo.grafted(&signed, &unsigned);
    let split = sshsig::split(&revisions::commit_object(&repo.dir, &grafted).unwrap());
    let checked = sshsig::check(&split.payload, split.signature.as_deref().unwrap()).unwrap();
    assert!(matches!(checked, sshsig::Checked::Bad(_)), "{checked:?}");
}

#[test]
fn the_last_change_of_an_entry_is_found_per_entry() {
    let repo = Repo::new("signed-history");
    let owner = keys_beside(&repo);
    assert_eq!(last(&repo, "a@1:1"), None, "no file yet");
    repo.commit(&[(OFFS, &offs(&[("a@1:1", "r", false)]))], "proposal");
    let approved = offs(&[("a@1:1", "r", true)]);
    let signed = repo.signed_commit(&owner, &[(OFFS, &approved)], "approval");
    assert_eq!(last(&repo, "a@1:1").as_deref(), Some(signed.as_str()));

    // Another entry added: the first keeps its commit.
    let two = offs(&[("a@1:1", "r", true), ("b@2:2", "s", false)]);
    let added = repo.commit(&[(OFFS, &two)], "a second proposal");
    assert_eq!(last(&repo, "a@1:1").as_deref(), Some(signed.as_str()));
    assert_eq!(last(&repo, "b@2:2").as_deref(), Some(added.as_str()));

    // The same content written differently is no change.
    let compact = serde_json::to_string(&serde_json::from_str::<serde_json::Value>(&two).unwrap());
    repo.commit(&[(OFFS, &compact.unwrap())], "reformatted");
    assert_eq!(last(&repo, "a@1:1").as_deref(), Some(signed.as_str()));

    // One commit changing both entries is the last change of both.
    let both = offs(&[("a@1:1", "r2", true), ("b@2:2", "s", true)]);
    let together = repo.signed_commit(&owner, &[(OFFS, &both)], "both approved");
    assert_eq!(last(&repo, "a@1:1").as_deref(), Some(together.as_str()));
    assert_eq!(last(&repo, "b@2:2").as_deref(), Some(together.as_str()));

    // A later unsigned change of the reason is the last change.
    let changed = offs(&[("a@1:1", "r2, changed", true), ("b@2:2", "s", true)]);
    let after = repo.commit(&[(OFFS, &changed)], "reason changed");
    assert_eq!(last(&repo, "a@1:1").as_deref(), Some(after.as_str()));
    assert_eq!(last(&repo, "b@2:2").as_deref(), Some(together.as_str()));
}

#[test]
fn a_merge_that_brings_a_change_is_the_last_change() {
    let repo = Repo::new("signed-merge");
    let owner = keys_beside(&repo);
    repo.commit(&[(OFFS, &offs(&[("a@1:1", "r", false)]))], "proposal");
    repo.git(&["checkout", "-q", "-b", "side"]);
    let approved = offs(&[("a@1:1", "r", true)]);
    let signed = repo.signed_commit(&owner, &[(OFFS, &approved)], "approval on a side branch");
    repo.git(&["checkout", "-q", "main"]);
    repo.commit(&[("other.txt", "x\n")], "something else on main");
    repo.git(&["merge", "-q", "--no-ff", "-m", "merge", "side"]);
    let merge = repo.git(&["rev-parse", "HEAD"]);
    assert_ne!(merge, signed);
    assert_eq!(last(&repo, "a@1:1").as_deref(), Some(merge.as_str()));
}
