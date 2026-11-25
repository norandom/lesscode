//! `step protect result` through the binary (brownfield-test-stage, group 2).
//! A stage is begun from a real plan of the toy repository and committed on
//! its branch; the verdict is the one `step check` writes. The result reads
//! committed files only, so the verdict and each write-off are committed.

mod step_support;

use lesscode_core::step::protect_result::{run_result_with, ResultArgs};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use step_support::planned::Planned;
use step_support::signing::{Key, StandIn};

const STAGE: &str = ".kiro/specs/protect-toy";
const LIST: &str = ".lesscode/allowed_signers";
const TITLE: &str = "lesscode approvals";
const OLD_WAY: &str = "approved the old way, which no longer counts: approve it in a signed commit";

/// A `write-offs.json` with one entry for the fault of `helper`, with these
/// fields added.
fn offs_of(kind: &str, fields: Value) -> String {
    let mut entry = json!({
        "file": "src/app.py", "function": "helper", "fault": "drop_statement@9:4",
        "kind": kind, "reason": "Nothing reads the count.", "proposed_by": "agent",
    });
    for (key, value) in fields.as_object().unwrap() {
        entry[key] = value.clone();
    }
    json!({ "schema": 1, "base": "", "write_offs": [entry] }).to_string()
}

/// Signed approvals (signed-approvals, task 3.1): the binary never reaches
/// the forge in these tests; a counted approval is judged through the
/// library entry, with a stand-in forge that lists the owner's key.
impl Planned {
    /// The owner's signing key, made once per test beside the repository.
    fn owner(&self) -> Key {
        Key::kept(&self.repo.dir.parent().unwrap().join("keys"), "owner")
    }

    /// Commit the files and a signer list naming the owner's key, signed
    /// with that key; returns the commit.
    fn signed(&self, files: &[(&str, &str)], message: &str) -> String {
        let owner = self.owner();
        let list = format!(
            "github.com/owner namespaces=\"git\" {}\n",
            owner.public_key()
        );
        let mut all = files.to_vec();
        all.push((LIST, &list));
        self.repo.signed_commit(&owner, &all, message)
    }

    /// The write-off of `helper` approved in a signed commit.
    fn signed_write_off(&self, kind: &str) -> String {
        let path = format!("{STAGE}/write-offs.json");
        let offs = offs_of(kind, json!({"approved": true}));
        self.signed(&[(&path, &offs)], "the approval, signed")
    }

    /// The result through the library entry, the forge listing the owner's
    /// key under its title.
    fn result_with_forge(&self, out: Option<PathBuf>) -> (Option<i32>, String) {
        let forge = StandIn::listing(&[(&self.owner(), TITLE)]);
        let args = ResultArgs {
            declaration: format!("{STAGE}/step.json"),
            repo: self.repo.dir.clone(),
            out,
        };
        let mut said = Vec::new();
        let code = run_result_with(args, &forge, &mut said);
        let mut said = String::from_utf8_lossy(&said).into_owned();
        match code {
            Ok(code) => (Some(i32::from(code)), said),
            Err(error) => {
                said.push_str(&format!("error: {error}"));
                (Some(3), said)
            }
        }
    }

    /// The same as `results_in`, through the library entry.
    fn signed_results_in(&self, code: i32, words: &[&str]) {
        let (found, said) = self.result_with_forge(None);
        assert_eq!(found, Some(code), "{said}");
        for word in words {
            assert!(said.contains(word), "{word:?} is not in: {said}");
        }
    }

    /// What the receipt says of a write-off approved in `commit`.
    fn approved_in(&self, commit: &str) -> String {
        let date = self.repo.git(&["log", "-1", "--format=%cI", commit]);
        format!(
            "Approved with the key \"{TITLE}\" ({}) of github.com/owner, in {commit} on {date}.",
            self.owner().fingerprint()
        )
    }
}

impl Planned {
    /// A toy repository with a stage for `helper`, committed on its branch.
    fn staged(name: &str) -> Planned {
        Planned::new(name, "python", "src/app.py").stage("src/app.py:helper")
    }

    /// Begin the stage for one function and commit it on its branch.
    fn stage(self, function: &str) -> Planned {
        let (code, said) = self.begin(&[function]);
        assert_eq!(code, Some(0), "{said}");
        let repo = &self.repo;
        repo.git(&["checkout", "-q", "-b", "protect/protect-toy"]);
        repo.git(&["add", "-A"]);
        repo.git(&["commit", "-q", "-m", "begin the stage"]);
        self
    }

    /// `step check` accepts the branch, and its verdict is committed.
    fn checked(&self) {
        let (code, out, err) = self.repo.step("check", "protect-toy", &[]);
        assert_eq!(code, Some(0), "{out}\n{err}");
        self.repo.commit(&[], "the verdict");
    }

    fn result(&self) -> (Option<i32>, String) {
        let declaration = format!("{STAGE}/step.json");
        let args = ["protect", "result", "--declaration", &declaration];
        self.repo.cli(&args, "")
    }

    /// The result is exit 2 with these words, and nothing is written.
    fn not_judged(&self, words: &[&str]) {
        let before = self.repo.state();
        let (code, said) = self.result();
        assert_eq!(code, Some(2), "{said}");
        for word in words {
            assert!(said.contains(word), "{word:?} is not in: {said}");
        }
        assert_eq!(self.repo.state(), before, "exit 2 wrote something");
        let receipt = self.repo.dir.join(STAGE).join("protection.json");
        assert!(!receipt.exists(), "exit 2 left a protection.json");
    }
}

impl Planned {
    /// Write `write-offs.json` with one entry for the fault of `helper`, and
    /// leave it uncommitted.
    fn written(&self, kind: &str, approved: bool) {
        let offs = offs_of(kind, json!({"approved": approved}));
        let path = format!("{STAGE}/write-offs.json");
        self.repo.write(&path, &offs);
    }

    /// The same, committed.
    fn write_off(&self, kind: &str, approved: bool) {
        self.written(kind, approved);
        self.repo.commit(&[], "a write-off");
    }

    /// The result has this exit code and these words.
    fn results_in(&self, code: i32, words: &[&str]) {
        let (found, said) = self.result();
        assert_eq!(found, Some(code), "{said}");
        for word in words {
            assert!(said.contains(word), "{word:?} is not in: {said}");
        }
    }
}

#[test]
fn an_unnoticed_fault_waits_until_its_write_off_is_approved() {
    let planned = Planned::staged("result-waiting");
    planned.checked();
    let numbers = "src/app.py: helper  noticed before 0, noticed now 0";

    // Left without a proposal: it waits, and is named.
    planned.results_in(
        1,
        &[
            &format!("{numbers}, written off 0, waiting 1"),
            "waiting: drop_statement@9:4: no test notices it and no write-off is proposed",
            "Not protected enough: 1 fault(s) wait.",
        ],
    );
    // A proposal nobody approved: it still waits.
    planned.write_off("nothing_a_caller_sees", false);
    planned.results_in(
        1,
        &[
            &format!("{numbers}, written off 0, waiting 1"),
            "waiting: drop_statement@9:4: its write-off is not approved",
        ],
    );
    // An approval that is written and not committed does not count.
    planned.written("nothing_a_caller_sees", true);
    planned.results_in(
        1,
        &["waiting: drop_statement@9:4: its write-off is not approved"],
    );
    // Approved and committed without a signature: it waits, with the repair.
    planned.write_off("nothing_a_caller_sees", true);
    planned.results_in(
        1,
        &[
            "waiting: drop_statement@9:4: its approval in ",
            " is not signed: sign it with `git commit --amend -S --no-edit`",
            "Nothing reads the count. Not counted: its approval in ",
        ],
    );
    // Approved the old way, by fields that no longer decide: it waits.
    let path = format!("{STAGE}/write-offs.json");
    let old = json!({"approved_by": "the owner", "approved_at": "2026-11-24"});
    let old = offs_of("nothing_a_caller_sees", old);
    planned
        .repo
        .commit(&[(&path, &old)], "approved the old way");
    planned.results_in(
        1,
        &[
            &format!("waiting: drop_statement@9:4: {OLD_WAY}"),
            &format!("Nothing reads the count. Not counted: {OLD_WAY}."),
        ],
    );
    // Approved in a signed commit by a key the list and the forge have:
    // written off, with its reason, key and commit.
    let commit = planned.signed_write_off("nothing_a_caller_sees");
    planned.signed_results_in(
        0,
        &[
            &format!("{numbers}, written off 1, waiting 0"),
            "written off: drop_statement@9:4",
            &format!(
                "src/app.py drop_statement@9:4 (nothing_a_caller_sees): Nothing reads the count. {}",
                planned.approved_in(&commit)
            ),
            "Protected enough: nothing waits in any chosen function.",
        ],
    );
    // A kind that is not one of the two is bad input, approved or not.
    planned.write_off("too_hard", true);
    planned.results_in(
        3,
        &[
            "the kind `too_hard` of the write-off for src/app.py drop_statement@9:4",
            "nothing_a_caller_sees, not_testable_as_it_stands",
        ],
    );
}

/// A verdict that refused because unnoticed faults rose: not protected
/// enough, whatever is written off. Refused for another reason: no result.
#[test]
fn a_verdict_that_refused_is_not_protected_enough_or_not_judged() {
    let planned = Planned::staged("result-refused");
    let repo = &planned.repo;
    planned.checked();
    planned.write_off("nothing_a_caller_sees", true);
    let verdict = repo.dir.join(STAGE).join("verdict.json");
    let refused = |check: &str| json!([{"name": check, "result": "refused", "detail": "why"}]);
    Planned::edit(&verdict, "result", json!("refused"));
    Planned::edit(&verdict, "checks", refused("planted faults"));
    repo.commit(&[], "a verdict that refused");
    planned.results_in(
        1,
        &["`step check` refused the branch because unnoticed faults rose"],
    );
    Planned::edit(&verdict, "checks", refused("size"));
    repo.commit(&[], "a verdict that refused otherwise");
    planned.results_in(
        2,
        &["Not judged: `step check` refused protect/protect-toy for a reason other than faults: size: why."],
    );
}

#[test]
fn a_fault_a_test_notices_is_noticed_also_where_it_has_a_write_off() {
    let planned = Planned::staged("result-noticed");
    let repo = &planned.repo;
    let expected = "if a > 3:\n    state.count = b\n";
    repo.commit(&[("tests/expect.txt", expected)], "a test for helper");
    planned.checked();
    let numbers = "src/app.py: helper  noticed before 0, noticed now 1, written off 0, waiting 0";
    planned.results_in(0, &[numbers, "Protected enough"]);

    planned.write_off("not_testable_as_it_stands", true);
    planned.results_in(
        0,
        &[
            numbers,
            "noticed: drop_statement@9:4 has a write-off, and a test notices it now",
        ],
    );
}

/// A function with three statements a fault can be planted in.
const THREE: &str = "def three(state, a):\n    state.first = a\n    state.second = a\n    state.third = a\n    return a\n";

/// A stage for `three` with one fault of each standing: the first has a
/// test, the second an approved write-off, the third a write-off that is
/// approved or not; approved in one signed commit, which is returned.
fn three(name: &str, third_approved: bool) -> (Planned, Value, String) {
    let expect = "def three(state, a):\n";
    let files = [("src/three.py", THREE), ("tests/expect.txt", expect)];
    let planned = Planned::with(name, "python", "src/three.py", &files);
    let planned = planned.stage("src/three.py:three");
    let tested = format!("{expect}    state.first = a\n");
    let repo = &planned.repo;
    repo.commit(&[("tests/expect.txt", &tested)], "a test for the first");
    planned.checked();
    let off = |line: usize, kind: &str, reason: &str, yes: bool| {
        json!({
            "file": "src/three.py", "function": "three",
            "fault": format!("drop_statement@{line}:4"), "kind": kind, "reason": reason,
            "proposed_by": "agent", "approved": yes,
            "approved_by": null, "approved_at": null,
        })
    };
    let reason = "The second is read by code we cannot run.";
    let second = off(3, "not_testable_as_it_stands", reason, true);
    let third = off(
        4,
        "nothing_a_caller_sees",
        "Nobody reads the third.",
        third_approved,
    );
    let offs = json!({"schema": 1, "base": "", "write_offs": [second, third]});
    let path = format!("{STAGE}/write-offs.json");
    let commit = planned.signed(&[(&path, &offs.to_string())], "the write-offs");
    (planned, second, commit)
}

/// The receipt of the stage of `three` while its third fault waits: the
/// four numbers, the write-off with its signed approval, the waiting fault,
/// and which content and which verdict they are for.
fn receipt_while_one_waits(planned: &Planned, mut second: Value, commit: &str) {
    let repo = &planned.repo;
    second["approval"] = json!({
        "account": "owner", "title": TITLE, "fingerprint": planned.owner().fingerprint(),
        "commit": commit, "date": repo.git(&["log", "-1", "--format=%cI", commit]),
    });
    let mut receipt = repo.json(&repo.dir.join(STAGE).join("protection.json"));
    assert_eq!(receipt["base"], repo.git(&["rev-parse", "main"]));
    assert_eq!(receipt["revision"], repo.git(&["rev-parse", "HEAD"]));
    assert_eq!(receipt["verdict_sha256"].as_str().unwrap().len(), 64);
    for key in ["base", "revision", "verdict_sha256"] {
        receipt[key] = Value::Null;
    }
    let expected = json!({
        "receipt": "step-protection-v1", "spec": "protect-toy",
        "base": null, "revision": null, "verdict_sha256": null,
        "result": "not_protected_enough", "unnoticed_rose": false,
        "functions": [{
            "file": "src/three.py", "function": "three",
            "noticed_before": 0, "noticed_now": 1, "written_off": 1, "waiting": 1,
            "write_offs": [second],
            "waiting_faults": [{"faults": ["drop_statement@4:4"], "line": 4,
                                "operator": "drop_statement",
                                "why": "its write-off is not approved"}],
            "noticed_write_offs": [],
        }],
        "reported": [],
        "from_the_base_run": [],
        "signers": {"account": "owner", "listed": 1, "on_forge": 1},
    });
    assert_eq!(receipt, expected, "{receipt:#}");
}

#[test]
fn the_result_names_each_standing_and_leaves_a_receipt() {
    let (planned, second, commit) = three("result-says", false);
    let approved = format!(
        "  src/three.py drop_statement@3:4 (not_testable_as_it_stands): The second is read by code we cannot run. {}",
        planned.approved_in(&commit)
    );
    planned.signed_results_in(1, &[
        "The stage protect-toy on protect/protect-toy:",
        "  src/three.py: three  noticed before 0, noticed now 1, written off 1, waiting 1",
        "    waiting: drop_statement@4:4: its write-off is not approved",
        "    written off: drop_statement@3:4",
        "Write-offs (2):",
        &approved,
        "    three reaches out: writes state at line 2: state.first = a, line 3: state.second = a, line 4: state.third = a. A reduction has to keep the behaviour there by other means.",
        "  src/three.py drop_statement@4:4 (nothing_a_caller_sees): Nobody reads the third. Not approved.",
        "Signer list: github.com/owner, 1 of 1 listed key(s) on the forge.",
        "Not protected enough: 1 fault(s) wait.",
        "The stage advises: a reduction that begins without it is judged as any other.",
        "Written: ",
    ]);
    receipt_while_one_waits(&planned, second, &commit);
}

/// signed-approvals 3.2 (requirements 3.1 to 3.4): an approval that does not
/// count stands beside the one that does, in the receipt and in
/// `protection.json`, with its reason, commit and key; the signer list is
/// named with how many of its keys the forge has.
#[test]
fn the_receipt_shows_each_refusal_beside_the_counted_approvals_and_the_signer_list() {
    let (planned, second, commit) = three("result-refusal", true);
    let repo = &planned.repo;
    // A key the list does not name changes the third write-off and signs it.
    let stranger = Key::kept(&repo.dir.parent().unwrap().join("keys"), "stranger");
    let path = format!("{STAGE}/write-offs.json");
    let mut offs = repo.json(&repo.dir.join(&path));
    offs["write_offs"][1]["reason"] = json!("Nobody reads the third, as far as we know.");
    let by_stranger = repo.signed_commit(&stranger, &[(&path, &offs.to_string())], "by a stranger");
    let refused = format!(
        "its approval in {} is signed with {}, which is not in the signer list",
        &by_stranger[..12],
        stranger.fingerprint()
    );
    planned.signed_results_in(1, &[
        &format!(
            "  src/three.py drop_statement@3:4 (not_testable_as_it_stands): The second is read by code we cannot run. {}",
            planned.approved_in(&commit)
        ),
        &format!(
            "  src/three.py drop_statement@4:4 (nothing_a_caller_sees): Nobody reads the third, as far as we know. Not counted: {refused}."
        ),
        &format!("    waiting: drop_statement@4:4: {refused}"),
        "Signer list: github.com/owner, 1 of 1 listed key(s) on the forge.",
        "Not protected enough: 1 fault(s) wait.",
    ]);
    let receipt = repo.json(&repo.dir.join(STAGE).join("protection.json"));
    assert_eq!(receipt["receipt"], "step-protection-v1");
    assert_eq!(
        receipt["signers"],
        json!({"account": "owner", "listed": 1, "on_forge": 1})
    );
    let row = &receipt["functions"][0];
    assert_eq!(row["write_offs"][0]["fault"], second["fault"]);
    assert_eq!(row["write_offs"][0]["approval"]["commit"], commit.as_str());
    assert_eq!(
        row["write_offs"][0]["approval"]["fingerprint"],
        planned.owner().fingerprint()
    );
    assert_eq!(
        row["waiting_faults"][0]["refusal"],
        json!({
            "kind": "not_listed", "commit": by_stranger,
            "key": stranger.fingerprint(), "repair": null,
        })
    );
    assert_eq!(row["waiting_faults"][0]["why"], refused.as_str());
}

/// A list whose keys signed no approval: the forge is not asked, and the
/// receipt says so.
#[test]
fn a_signer_list_whose_keys_signed_nothing_is_named_without_the_forge() {
    let (planned, _, _) = three("result-unlisted-only", false);
    let repo = &planned.repo;
    let stranger = Key::kept(&repo.dir.parent().unwrap().join("keys"), "stranger");
    let path = format!("{STAGE}/write-offs.json");
    let mut offs = repo.json(&repo.dir.join(&path));
    offs["write_offs"][0]["reason"] = json!("Read by code we cannot run, as we found.");
    repo.signed_commit(&stranger, &[(&path, &offs.to_string())], "by a stranger");
    planned.signed_results_in(1, &[
        "Signer list: github.com/owner, 1 listed key(s); the forge was not asked, since no approval is signed with a listed key.",
        "Not protected enough: 2 fault(s) wait.",
    ]);
    let receipt = repo.json(&repo.dir.join(STAGE).join("protection.json"));
    assert_eq!(
        receipt["signers"],
        json!({"account": "owner", "listed": 1, "on_forge": null})
    );
}

#[test]
fn a_protected_stage_names_the_next_step_and_writes_where_it_is_asked_to() {
    let (planned, _, _) = three("result-next", true);
    let repo = &planned.repo;
    let out = repo.dir.parent().unwrap().join("elsewhere.json");
    let (code, said) = planned.result_with_forge(Some(out.clone()));
    assert_eq!(code, Some(0), "{said}");
    for line in [
        "written off 2, waiting 0",
        "Protected enough: nothing waits in any chosen function.",
        "Next: declare the reduction of a chosen function as a step.",
        "The stage advises:",
    ] {
        assert!(said.contains(line), "{line:?} is not in:\n{said}");
    }
    assert_eq!(repo.json(&out)["result"], "protected_enough");
    let beside = repo.dir.join(STAGE).join("protection.json");
    assert!(
        !beside.exists(),
        "--out was given, and the receipt is also beside the declaration"
    );
}

#[test]
fn reported_functions_are_listed_and_do_not_change_the_result() {
    let planned = Planned::staged("result-reported");
    planned.checked();
    planned.signed_write_off("nothing_a_caller_sees");
    let plan = planned.repo.json(&planned.plan);
    let rows = plan["functions"].as_array().unwrap();
    let step = rows.iter().find(|r| r["function"] == "step").unwrap();
    let unnoticed = step["unnoticed"].as_u64().unwrap();
    assert!(unnoticed > 0, "step has no unnoticed fault in the toy plan");

    planned.signed_results_in(
        0,
        &[
            "Reported (1), not chosen; they do not count in the result:",
            &format!("  src/app.py: step  unnoticed {unnoticed} in the plan, not run again"),
            "Protected enough: nothing waits in any chosen function.",
        ],
    );
    let receipt = planned
        .repo
        .json(&planned.repo.dir.join(STAGE).join("protection.json"));
    assert_eq!(
        receipt["reported"],
        json!([{"file": "src/app.py", "function": "step", "unnoticed": unnoticed}])
    );
    let functions = receipt["functions"].as_array().unwrap();
    assert!(
        functions.iter().all(|f| f["function"] == "helper"),
        "{receipt:#}"
    );
}

/// Added 2026-11-25 (task 6.5, R189): the plan counted a fault noticed, so
/// `stage.json` lacks it, and the base run of the check does not notice it.
#[test]
fn a_fault_the_plan_missed_but_the_base_run_did_not_notice_can_be_written_off() {
    let planned = Planned::new("result-base-run", "python", "src/app.py");
    let (code, said) = planned.begin(&["src/app.py:helper"]);
    assert_eq!(code, Some(0), "{said}");
    let repo = &planned.repo;
    let path = repo.dir.join(STAGE).join("stage.json");
    let mut stage = repo.json(&path);
    stage["chosen"][0]["faults"] = json!([]);
    fs::write(&path, stage.to_string()).unwrap();
    repo.git(&["checkout", "-q", "-b", "protect/protect-toy"]);
    repo.git(&["add", "-A"]);
    repo.git(&[
        "commit",
        "-q",
        "-m",
        "begin the stage, as a plan that miscounted",
    ]);
    planned.checked();

    // The verdict names what the base run did not notice.
    let verdict = repo.json(&repo.dir.join(STAGE).join("verdict.json"));
    let rows = verdict["functions"].as_array().unwrap();
    let helper = rows.iter().find(|r| r["function"] == "helper").unwrap();
    let before = helper["unnoticed_before"].as_array().unwrap();
    assert_eq!(before.len(), 1, "{helper:#}");
    assert!(before[0]
        .as_str()
        .unwrap()
        .ends_with(" line 9 drop_statement"));

    planned.results_in(
        1,
        &[
            "From the base run (the plan counted them noticed): src/app.py: helper drop_statement@9",
            "waiting: drop_statement@9: no test notices it and no write-off is proposed",
        ],
    );
    let offs = json!({ "schema": 1, "base": "", "write_offs": [{
        "file": "src/app.py", "function": "helper", "fault": "drop_statement@9",
        "kind": "nothing_a_caller_sees", "reason": "Nothing reads the count.",
        "proposed_by": "agent", "approved": true,
    }]});
    planned.signed(
        &[(&format!("{STAGE}/write-offs.json"), &offs.to_string())],
        "a write-off, approved in a signed commit",
    );
    planned.signed_results_in(0, &["written off: drop_statement@9", "Protected enough"]);
    let receipt = repo.json(&repo.dir.join(STAGE).join("protection.json"));
    assert_eq!(
        receipt["from_the_base_run"],
        json!([{"file": "src/app.py", "function": "helper", "fault": "drop_statement@9"}])
    );
}

#[test]
fn a_stage_without_a_verdict_is_not_judged() {
    let planned = Planned::staged("result-no-verdict");
    planned.not_judged(&[
        "Not judged: there is no committed verdict at .kiro/specs/protect-toy/verdict.json.",
        "Run `step check` on protect/protect-toy and commit its verdict first.",
    ]);
}

#[test]
fn a_verdict_for_other_content_than_the_branch_has_is_not_judged() {
    let planned = Planned::staged("result-other-content");
    let repo = &planned.repo;
    planned.checked();
    // The verdict is the one for the branch as it is: there is a result.
    let (code, said) = planned.result();
    assert_ne!(code, Some(2), "{said}");
    assert!(!said.contains("Not judged"), "{said}");

    // A test file is added after the verdict was written. The result that
    // was left for the content before it is taken away first.
    fs::remove_file(repo.dir.join(STAGE).join("protection.json")).unwrap();
    repo.commit(&[("tests/more.txt", "more\n")], "one more test file");
    planned.not_judged(&[
        "is not the one for what protect/protect-toy has committed",
        "a test file changed after the verdict was written",
        "Run `step check` again.",
    ]);
}

#[test]
fn a_branch_that_changed_a_code_file_is_not_judged() {
    let planned = Planned::staged("result-code-changed");
    let repo = &planned.repo;
    let code = fs::read_to_string(repo.dir.join("src/app.py")).unwrap();
    repo.commit(
        &[("src/app.py", &format!("{code}\n# a note\n"))],
        "change the code",
    );
    // Whatever the check makes of the change, its verdict is for this content.
    let (_, out, err) = repo.step("check", "protect-toy", &[]);
    let verdict = repo.dir.join(STAGE).join("verdict.json");
    assert!(verdict.is_file(), "{out}\n{err}");
    repo.commit(&[], "the verdict");
    planned.not_judged(&[
        "src/app.py differs between the base and protect/protect-toy",
        "the result does not hold for the functions of that file",
    ]);
}

/// signed-approvals 3.1 (requirement 2.6): the command takes the signing
/// keys from GitHub only, and has no option that names another forge.
#[test]
fn the_command_has_no_option_for_another_forge() {
    let repo = step_support::Repo::new("result-help");
    let (code, said) = repo.cli(&["protect", "result", "--help"], "");
    assert_eq!(code, Some(0), "{said}");
    // The usage line names `--declaration` once more; each option counts once.
    let options: std::collections::BTreeSet<&str> = said
        .split_whitespace()
        .filter(|w| w.starts_with("--"))
        .collect();
    let expected = ["--declaration", "--help", "--out", "--repo"];
    assert_eq!(options.into_iter().collect::<Vec<_>>(), expected, "{said}");
    let lower = said.to_lowercase();
    assert!(
        !lower.contains("forge") && !lower.contains("github"),
        "{said}"
    );
}
