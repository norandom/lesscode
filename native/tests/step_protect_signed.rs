//! Approvals of write-offs in signed commits (spec signed-approvals). Keys
//! are made per test, without a passphrase; the forge is a stand-in.

mod step_support;

use lesscode_core::step::forge::{Forge, ForgeError};
use lesscode_core::step::{approvals, revisions, sshsig};
use serde_json::json;
use std::process::Command;
use step_support::planned::Planned;
use step_support::protect::{offs_of, three, LIST, OLD_WAY, STAGE, TITLE};
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

// The eleven cases of the design's testing strategy (signed-approvals,
// task 3.3), through the library entry on the toy stage of `helper`, each
// with its exit code and words.

const KIND: &str = "nothing_a_caller_sees";
const WAITING: &str = "waiting: drop_statement@9:4: ";

/// The write-off of `helper`, marked approved.
fn marked() -> String {
    offs_of(KIND, json!({"approved": true}))
}

/// The stage of `helper` with its verdict committed.
fn checked(name: &str) -> Planned {
    let planned = Planned::staged(name);
    planned.checked();
    planned
}

/// The fault waits for this refusal, and the write-off says it beside the
/// counted ones.
fn waits_for(planned: &Planned, forge: &StandIn, refused: &str, more: &[&str]) -> String {
    let waiting = format!("{WAITING}{refused}");
    let not_counted = format!("Nothing reads the count. Not counted: {refused}.");
    let mut words = vec![waiting.as_str(), not_counted.as_str()];
    words.extend(more);
    words.push("Not protected enough: 1 fault(s) wait.");
    planned.through_results_in(forge, 1, &words)
}

/// Exit 2 or 3 with these words, and nothing written.
fn writes_nothing(planned: &Planned, forge: &StandIn, code: i32, words: &[&str]) {
    let before = planned.repo.state();
    planned.through_results_in(forge, code, words);
    assert_eq!(planned.repo.state(), before, "exit {code} wrote something");
    let receipt = planned.repo.dir.join(STAGE).join("protection.json");
    assert!(!receipt.exists(), "exit {code} left a protection.json");
}

/// Case 1 (requirements 1.1, 3.1, 3.2).
#[test]
fn a_signed_approval_by_a_listed_key_on_the_forge_is_written_off() {
    let planned = checked("signed-counted");
    let commit = planned.signed_write_off(KIND);
    let forge = planned.owners_forge();
    planned.through_results_in(
        &forge,
        0,
        &[
            "src/app.py: helper  noticed before 0, noticed now 0, written off 1, waiting 0",
            "    written off: drop_statement@9:4",
            &format!("Nothing reads the count. {}", planned.approved_in(&commit)),
            "Signer list: github.com/owner, 1 of 1 listed key(s) on the forge.",
            "Protected enough: nothing waits in any chosen function.",
        ],
    );
    assert_eq!(forge.asked.get(), 1);
}

/// Case 2 (1.3, 4.2): unsigned on the head, with the amend as its repair;
/// the repair counts.
#[test]
fn an_unsigned_approval_waits_until_the_amend_it_names_signs_it() {
    let planned = checked("signed-unsigned");
    let owner = planned.owner();
    let list = Planned::list_of(&[&owner]);
    let marked = marked();
    let files = [(OFFS, marked.as_str()), (LIST, list.as_str())];
    let approval = planned.repo.commit(&files, "the approval, unsigned");
    let forge = planned.owners_forge();
    let refused = format!(
        "its approval in {} is not signed: sign it with `git commit --amend -S --no-edit`",
        &approval[..12]
    );
    let said = waits_for(&planned, &forge, &refused, &[]);
    assert!(!said.contains("Signer list"), "{said}");
    assert_eq!(forge.asked.get(), 0);
    let amended = planned.repo.amended_signed(&owner);
    planned.through_results_in(&forge, 0, &[&planned.approved_in(&amended)]);
}

/// Case 3 (1.3): a signature the owner made over other content, grafted
/// onto the approval.
#[test]
fn a_signature_that_does_not_verify_is_refused() {
    let planned = checked("signed-grafted");
    let (repo, owner) = (&planned.repo, planned.owner());
    let list = Planned::list_of(&[&owner]);
    let unsigned = repo.commit(&[(OFFS, &marked()), (LIST, &list)], "the approval");
    repo.git(&["checkout", "-q", "-b", "elsewhere"]);
    let signed = repo.signed_commit(&owner, &[("notes.txt", "x\n")], "signed elsewhere");
    let grafted = repo.grafted(&signed, &unsigned);
    repo.git(&["checkout", "-q", "protect/protect-toy"]);
    repo.git(&["reset", "-q", "--hard", &grafted]);
    let forge = planned.owners_forge();
    let refused = format!(
        "the signature of its approval in {} does not verify",
        &grafted[..12]
    );
    waits_for(&planned, &forge, &refused, &[]);
    assert_eq!(forge.asked.get(), 0);
}

/// Case 4 (1.3): a key the forge has, but the list does not name.
#[test]
fn a_key_not_in_the_signer_list_is_refused() {
    let planned = checked("signed-unlisted");
    let (owner, stranger) = (planned.owner(), planned.key("stranger"));
    let list = Planned::list_of(&[&owner]);
    let marked = marked();
    let files = [(OFFS, marked.as_str()), (LIST, list.as_str())];
    let commit = planned
        .repo
        .signed_commit(&stranger, &files, "approved by a stranger");
    let forge = StandIn::listing(&[(&owner, TITLE), (&stranger, "stranger")]);
    let refused = format!(
        "its approval in {} is signed with {}, which is not in the signer list",
        &commit[..12],
        stranger.fingerprint()
    );
    let list_line = "Signer list: github.com/owner, 1 listed key(s); the forge was not asked";
    waits_for(&planned, &forge, &refused, &[list_line]);
    assert_eq!(forge.asked.get(), 0);
}

/// Case 5 (2.2): listed, but not among the account's keys on the forge;
/// named by its fingerprint.
#[test]
fn a_listed_key_the_forge_does_not_have_is_refused() {
    let planned = checked("signed-not-on-forge");
    let commit = planned.signed_write_off(KIND);
    let forge = StandIn::listing(&[]);
    let refused = format!(
        "its approval in {} is signed with {}, which is not among the account's signing keys on the forge",
        &commit[..12],
        planned.owner().fingerprint()
    );
    let list_line = "Signer list: github.com/owner, 0 of 1 listed key(s) on the forge.";
    waits_for(&planned, &forge, &refused, &[list_line]);
    assert_eq!(forge.asked.get(), 1);
}

/// Case 6 (2.5): nothing is kept between runs; a key removed from the
/// forge stops counting at the next run.
#[test]
fn a_key_removed_from_the_forge_between_two_runs_stops_counting() {
    let planned = checked("signed-removed");
    let commit = planned.signed_write_off(KIND);
    let first = planned.owners_forge();
    planned.through_results_in(&first, 0, &[&planned.approved_in(&commit)]);
    let removed = StandIn::listing(&[]);
    let refused = format!(
        "its approval in {} is signed with {}, which is not among the account's signing keys on the forge",
        &commit[..12],
        planned.owner().fingerprint()
    );
    waits_for(&planned, &removed, &refused, &["0 of 1 listed key(s)"]);
    assert_eq!((first.asked.get(), removed.asked.get()), (1, 1));
    let receipt = planned
        .repo
        .json(&planned.repo.dir.join(STAGE).join("protection.json"));
    let row = &receipt["functions"][0];
    assert_eq!(row["waiting_faults"][0]["refusal"]["kind"], "not_on_forge");
    assert_eq!(receipt["signers"]["on_forge"], 0);
}

/// Case 7 (1.2): the last change decides; an unsigned change of a signed
/// approval's reason undoes it.
#[test]
fn an_unsigned_change_after_a_signed_approval_waits() {
    let planned = checked("signed-then-changed");
    planned.signed_write_off(KIND);
    let reason = json!({"approved": true, "reason": "Nothing reads the count, we think."});
    let changed = offs_of(KIND, reason);
    let commit = planned
        .repo
        .commit(&[(OFFS, &changed)], "the reason, unsigned");
    let refused = format!(
        "its approval in {} is not signed: sign it with",
        &commit[..12]
    );
    planned.through_results_in(
        &planned.owners_forge(),
        1,
        &[
            &format!("{WAITING}{refused}"),
            "Nothing reads the count, we think. Not counted: its approval in ",
        ],
    );
}

/// Case 8 (1.6): one signed commit approves two entries.
#[test]
fn one_signed_commit_approves_two_entries() {
    let (planned, _, commit) = three("signed-two-entries", true);
    let forge = planned.owners_forge();
    let approved = planned.approved_in(&commit);
    planned.through_results_in(
        &forge,
        0,
        &[
            "src/three.py: three  noticed before 0, noticed now 1, written off 2, waiting 0",
            &format!("The second is read by code we cannot run. {approved}"),
            &format!("Nobody reads the third. {approved}"),
            "Protected enough",
        ],
    );
    assert_eq!(forge.asked.get(), 1);
}

/// Case 9 (2.4): the forge cannot be reached: not judged, nothing written.
#[test]
fn an_unreachable_forge_is_not_judged() {
    let planned = checked("signed-unreachable");
    planned.signed_write_off(KIND);
    let words = ["Not judged: ", "GitHub cannot be reached"];
    writes_nothing(&planned, &StandIn::unreachable(), 2, &words);
}

/// Case 10 (2.3): a marked entry and no signer list is not judged; a list
/// with `cert-authority` is bad input.
#[test]
fn a_missing_signer_list_is_not_judged_and_cert_authority_is_bad_input() {
    let planned = checked("signed-no-list");
    let owner = planned.owner();
    let marked = marked();
    let files = [(OFFS, marked.as_str())];
    planned
        .repo
        .signed_commit(&owner, &files, "approved, no list");
    let forge = planned.owners_forge();
    let words = [
        "Not judged: ",
        "no signer list at .lesscode/allowed_signers",
    ];
    writes_nothing(&planned, &forge, 2, &words);
    assert_eq!(forge.asked.get(), 0);
    let bad = format!("github.com/owner cert-authority {}\n", owner.public_key());
    planned
        .repo
        .commit(&[(LIST, &bad)], "a list with an option");
    writes_nothing(&planned, &forge, 3, &["error: ", "cert-authority"]);
}

/// Case 11 (6.1): only the fields of before, even in a signed commit.
#[test]
fn an_approval_by_the_old_fields_only_waits() {
    let planned = checked("signed-old-way");
    let owner = planned.owner();
    let old = offs_of(
        KIND,
        json!({"approved_by": "the owner", "approved_at": "2026-11-24"}),
    );
    let list = Planned::list_of(&[&owner]);
    let files = [(OFFS, old.as_str()), (LIST, list.as_str())];
    planned
        .repo
        .signed_commit(&owner, &files, "approved the old way, signed");
    let forge = planned.owners_forge();
    waits_for(&planned, &forge, OLD_WAY, &[]);
    assert_eq!(forge.asked.get(), 0);
}
