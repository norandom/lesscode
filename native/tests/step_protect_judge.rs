//! The judge of approvals (signed-approvals, task 2.2), through the library
//! on a step branch of the toy repository: real Git and OpenSSH, keys made
//! per test without a passphrase, the stand-in forge.

mod step_support;

use lesscode_core::step::approvals::{judge, Judged, RefusalKind};
use lesscode_core::step::context::Context;
use lesscode_core::step::protect_numbers::WriteOff;
use serde_json::{json, Value};
use step_support::signing::{Key, StandIn};
use step_support::Repo;

const DECLARATION: &str = ".kiro/specs/protect-toy/step.json";
const OFFS: &str = ".kiro/specs/protect-toy/write-offs.json";
const LIST: &str = ".lesscode/allowed_signers";

/// A step branch with its declaration, and the owner's key beside it.
fn staged(name: &str) -> (Repo, Key) {
    let repo = Repo::new(name);
    repo.begin("protect-toy", json!({}));
    let owner = Key::new(&repo.dir.parent().unwrap().join("keys"), "owner");
    (repo, owner)
}

fn other_key(repo: &Repo, name: &str) -> Key {
    Key::new(&repo.dir.parent().unwrap().join("keys"), name)
}

/// A `write-offs.json` with one entry per (fault, approved).
fn offs(entries: &[(&str, bool)]) -> String {
    let entry = |(fault, approved): &(&str, bool)| {
        json!({ "file": "src/app.py", "function": "helper", "fault": fault,
                "kind": "nothing_a_caller_sees", "reason": "Nothing reads it.",
                "proposed_by": "agent", "approved": approved })
    };
    let all: Vec<Value> = entries.iter().map(entry).collect();
    serde_json::to_string_pretty(&json!({ "schema": 1, "write_offs": all })).unwrap()
}

/// A signer list for the account `owner` with these keys.
fn list(keys: &[&Key]) -> String {
    let line = |k: &&Key| format!("github.com/owner namespaces=\"git\" {}\n", k.public_key());
    keys.iter().map(line).collect()
}

/// Judge the write-offs as the branch has them.
fn judged(repo: &Repo, forge: &StandIn) -> (Result<Result<Judged, String>, String>, Vec<WriteOff>) {
    let context = Context::open(&repo.dir, DECLARATION).unwrap();
    let bytes = std::fs::read(repo.dir.join(OFFS)).unwrap();
    let file: Value = serde_json::from_slice(&bytes).unwrap();
    let mut offs: Vec<WriteOff> = serde_json::from_value(file["write_offs"].clone()).unwrap();
    let judgement = judge(&context, OFFS, &mut offs, forge);
    (judgement, offs)
}

fn refusal_of(w: &WriteOff) -> (RefusalKind, Option<&str>, Option<&str>) {
    let r = w.refusal.as_ref().expect("a refusal");
    assert!(w.approval.is_none());
    (r.kind, r.key.as_deref(), r.repair.as_deref())
}

#[test]
fn a_signed_approval_by_a_listed_key_on_the_forge_counts() {
    let (repo, owner) = staged("judge-counted");
    repo.commit(
        &[
            (OFFS, &offs(&[("a@1:1", false), ("b@2:2", false)])),
            (LIST, &list(&[&owner])),
        ],
        "proposals and the signer list",
    );
    let first = repo.signed_commit(
        &owner,
        &[(OFFS, &offs(&[("a@1:1", true), ("b@2:2", false)]))],
        "approve a",
    );
    let second = repo.signed_commit(
        &owner,
        &[(OFFS, &offs(&[("a@1:1", true), ("b@2:2", true)]))],
        "approve b",
    );
    let forge = StandIn::listing(&[(&owner, "lesscode approvals")]);
    let (judgement, offs) = judged(&repo, &forge);
    let judged = judgement.unwrap().unwrap();
    assert_eq!((judged.counted, judged.on_forge), (2, Some(1)));
    assert_eq!(judged.list.unwrap().account, "owner");
    // Two approvals, two commits, one question to the forge.
    assert_eq!(forge.asked.get(), 1);
    let commits: Vec<&str> = offs
        .iter()
        .map(|w| w.approval.as_ref().unwrap().commit.as_str())
        .collect();
    assert_eq!(commits, [first.as_str(), second.as_str()]);
    let a = offs[0].approval.as_ref().unwrap();
    assert_eq!(
        (a.account.as_str(), a.title.as_str(), a.fingerprint.clone()),
        ("owner", "lesscode approvals", owner.fingerprint())
    );
    let date = repo.git(&["log", "-1", "--format=%cI", &first]);
    assert_eq!(a.date, date);
    assert!(offs.iter().all(|w| w.approved() && w.refusal.is_none()));
}

#[test]
fn an_unsigned_approval_is_refused_without_the_list_or_the_forge() {
    let (repo, _owner) = staged("judge-unsigned");
    repo.commit(&[(OFFS, &offs(&[("a@1:1", false)]))], "proposal");
    let approval = repo.commit(&[(OFFS, &offs(&[("a@1:1", true)]))], "approval, unsigned");
    let forge = StandIn::unreachable();

    // No signer list on the branch, an unreachable forge: still judged.
    let (judgement, offs) = judged(&repo, &forge);
    assert_eq!(judgement.unwrap().unwrap(), Judged::default());
    let amend = "sign it with `git commit --amend -S --no-edit`";
    assert_eq!(
        refusal_of(&offs[0]),
        (RefusalKind::NotSigned, None, Some(amend))
    );
    assert_eq!(offs[0].refusal.as_ref().unwrap().commit, approval);
    assert_eq!(forge.asked.get(), 0);

    // Once another commit follows, amending would not reach it.
    repo.commit(&[("notes.txt", "later\n")], "something else");
    let (_, offs) = judged(&repo, &forge);
    let twice = "set `approved` to false in one commit and back to true in a signed one";
    assert_eq!(
        refusal_of(&offs[0]),
        (RefusalKind::NotSigned, None, Some(twice))
    );
}

#[test]
fn a_signature_that_does_not_verify_is_refused() {
    let (repo, owner) = staged("judge-grafted");
    repo.commit(
        &[(OFFS, &offs(&[("a@1:1", false)])), (LIST, &list(&[&owner]))],
        "proposal",
    );
    let unsigned = repo.commit(&[(OFFS, &offs(&[("a@1:1", true)]))], "approval");
    // A signature the owner made over other content, grafted onto the approval.
    repo.git(&["checkout", "-q", "-b", "elsewhere"]);
    let signed = repo.signed_commit(&owner, &[("notes.txt", "x\n")], "signed elsewhere");
    let grafted = repo.grafted(&signed, &unsigned);
    repo.git(&["checkout", "-q", "step/protect-toy"]);
    repo.git(&["reset", "-q", "--hard", &grafted]);
    let forge = StandIn::listing(&[(&owner, "lesscode approvals")]);
    let (judgement, offs) = judged(&repo, &forge);
    assert_eq!(judgement.unwrap().unwrap().counted, 0);
    assert_eq!(
        refusal_of(&offs[0]),
        (RefusalKind::DoesNotVerify, None, None)
    );
    assert_eq!(offs[0].refusal.as_ref().unwrap().commit, grafted);
    assert_eq!(forge.asked.get(), 0);
}

#[test]
fn a_key_not_in_the_list_is_refused_and_the_forge_not_asked() {
    let (repo, owner) = staged("judge-unlisted");
    let intruder = other_key(&repo, "intruder");
    repo.commit(
        &[(OFFS, &offs(&[("a@1:1", false)])), (LIST, &list(&[&owner]))],
        "proposal",
    );
    repo.signed_commit(&intruder, &[(OFFS, &offs(&[("a@1:1", true)]))], "approval");
    let forge = StandIn::listing(&[(&owner, "lesscode approvals"), (&intruder, "intruder")]);
    let (judgement, offs) = judged(&repo, &forge);
    let judged = judgement.unwrap().unwrap();
    assert_eq!((judged.counted, judged.on_forge), (0, None));
    let fingerprint = intruder.fingerprint();
    assert_eq!(
        refusal_of(&offs[0]),
        (RefusalKind::NotListed, Some(fingerprint.as_str()), None)
    );
    assert_eq!(forge.asked.get(), 0);
}

#[test]
fn a_listed_key_the_forge_does_not_have_is_refused_by_its_fingerprint() {
    let (repo, owner) = staged("judge-not-on-forge");
    let ci = other_key(&repo, "ci");
    repo.commit(
        &[
            (OFFS, &offs(&[("a@1:1", false)])),
            (LIST, &list(&[&owner, &ci])),
        ],
        "proposal",
    );
    repo.signed_commit(&owner, &[(OFFS, &offs(&[("a@1:1", true)]))], "approval");
    let forge = StandIn::listing(&[(&ci, "GitHub Actions Signing Key")]);
    let (judgement, offs) = judged(&repo, &forge);
    let judged = judgement.unwrap().unwrap();
    assert_eq!((judged.counted, judged.on_forge), (0, Some(1)));
    let fingerprint = owner.fingerprint();
    assert_eq!(
        refusal_of(&offs[0]),
        (RefusalKind::NotOnForge, Some(fingerprint.as_str()), None)
    );
    assert_eq!(forge.asked.get(), 1);
}

#[test]
fn a_missing_list_or_forge_is_not_judged_and_a_bad_list_is_bad_input() {
    let (repo, owner) = staged("judge-not-judged");
    repo.commit(&[(OFFS, &offs(&[("a@1:1", false)]))], "proposal");
    repo.signed_commit(&owner, &[(OFFS, &offs(&[("a@1:1", true)]))], "approval");
    let forge = StandIn::listing(&[(&owner, "lesscode approvals")]);
    let (judgement, _) = judged(&repo, &forge);
    let why = judgement.unwrap().unwrap_err();
    assert!(
        why.contains("no signer list at .lesscode/allowed_signers"),
        "{why}"
    );
    assert_eq!(forge.asked.get(), 0);

    repo.commit(&[(LIST, &list(&[&owner]))], "the signer list");
    let (judgement, _) = judged(&repo, &StandIn::unreachable());
    let why = judgement.unwrap().unwrap_err();
    assert!(why.contains("GitHub cannot be reached"), "{why}");

    let bad = format!("github.com/owner cert-authority {}\n", owner.public_key());
    repo.commit(&[(LIST, &bad)], "a list with an option");
    let (judgement, _) = judged(&repo, &forge);
    assert!(judgement.unwrap_err().contains("cert-authority"));
}

#[test]
fn unmarked_write_offs_read_nothing_and_count_nothing() {
    let (repo, owner) = staged("judge-unmarked");
    repo.signed_commit(
        &owner,
        &[(OFFS, &offs(&[("a@1:1", false)]))],
        "a signed proposal",
    );
    let forge = StandIn::unreachable();
    let (judgement, offs) = judged(&repo, &forge);
    assert_eq!(judgement.unwrap().unwrap(), Judged::default());
    assert!(offs
        .iter()
        .all(|w| w.approval.is_none() && w.refusal.is_none()));
    assert_eq!(forge.asked.get(), 0);
}
