//! Approvals of write-offs in signed commits (spec signed-approvals). Keys
//! are made per test, without a passphrase; the forge is a stand-in.

mod step_support;

use lesscode_core::step::forge::{Forge, ForgeError};
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
