//! A stage of `step protect` on the toy repository, committed on its branch
//! with its verdict, and its result through the library entry with a
//! stand-in forge (signed-approvals, tasks 3.1 to 3.3).

use super::planned::Planned;
use super::signing::{Key, StandIn};
use lesscode_core::step::protect_result::{run_result_with, ResultArgs};
use serde_json::{json, Value};
use std::path::PathBuf;

pub const STAGE: &str = ".kiro/specs/protect-toy";
pub const LIST: &str = ".lesscode/allowed_signers";
pub const TITLE: &str = "lesscode approvals";
pub const OLD_WAY: &str =
    "approved the old way, which no longer counts: approve it in a signed commit";

/// A function with three statements a fault can be planted in.
pub const THREE: &str = "def three(state, a):\n    state.first = a\n    state.second = a\n    state.third = a\n    return a\n";

/// A `write-offs.json` with one entry for the fault of `helper`, with these
/// fields added.
pub fn offs_of(kind: &str, fields: Value) -> String {
    let mut entry = json!({
        "file": "src/app.py", "function": "helper", "fault": "drop_statement@9:4",
        "kind": kind, "reason": "Nothing reads the count.", "proposed_by": "agent",
    });
    for (key, value) in fields.as_object().unwrap() {
        entry[key] = value.clone();
    }
    json!({ "schema": 1, "base": "", "write_offs": [entry] }).to_string()
}

impl Planned {
    /// A toy repository with a stage for `helper`, committed on its branch.
    pub fn staged(name: &str) -> Planned {
        Planned::new(name, "python", "src/app.py").stage("src/app.py:helper")
    }

    /// Begin the stage for one function and commit it on its branch.
    pub fn stage(self, function: &str) -> Planned {
        let (code, said) = self.begin(&[function]);
        assert_eq!(code, Some(0), "{said}");
        let repo = &self.repo;
        repo.git(&["checkout", "-q", "-b", "protect/protect-toy"]);
        repo.git(&["add", "-A"]);
        repo.git(&["commit", "-q", "-m", "begin the stage"]);
        self
    }

    /// `step check` accepts the branch, and its verdict is committed.
    pub fn checked(&self) {
        let (code, out, err) = self.repo.step("check", "protect-toy", &[]);
        assert_eq!(code, Some(0), "{out}\n{err}");
        self.repo.commit(&[], "the verdict");
    }

    /// A key made once per test beside the repository; `owner` is the one
    /// the signer list names.
    pub fn key(&self, name: &str) -> Key {
        Key::kept(&self.repo.dir.parent().unwrap().join("keys"), name)
    }

    /// The owner's signing key.
    pub fn owner(&self) -> Key {
        self.key("owner")
    }

    /// A signer list for the account `owner` with these keys.
    pub fn list_of(keys: &[&Key]) -> String {
        let line = |k: &&Key| format!("github.com/owner namespaces=\"git\" {}\n", k.public_key());
        keys.iter().map(line).collect()
    }

    /// Commit the files and a signer list naming the owner's key, signed
    /// with that key; returns the commit.
    pub fn signed(&self, files: &[(&str, &str)], message: &str) -> String {
        let owner = self.owner();
        let list = Planned::list_of(&[&owner]);
        let mut all = files.to_vec();
        all.push((LIST, &list));
        self.repo.signed_commit(&owner, &all, message)
    }

    /// The write-off of `helper` approved in a signed commit.
    pub fn signed_write_off(&self, kind: &str) -> String {
        let path = format!("{STAGE}/write-offs.json");
        let offs = offs_of(kind, json!({"approved": true}));
        self.signed(&[(&path, &offs)], "the approval, signed")
    }

    /// The forge listing the owner's key under its title.
    pub fn owners_forge(&self) -> StandIn {
        StandIn::listing(&[(&self.owner(), TITLE)])
    }

    /// The result through the library entry with this forge: its exit code
    /// (3 for an error, whose words are appended) and what it said.
    pub fn result_through(&self, forge: &StandIn, out: Option<PathBuf>) -> (i32, String) {
        let args = ResultArgs {
            declaration: format!("{STAGE}/step.json"),
            repo: self.repo.dir.clone(),
            out,
        };
        let mut said = Vec::new();
        let code = run_result_with(args, forge, &mut said);
        let mut said = String::from_utf8_lossy(&said).into_owned();
        match code {
            Ok(code) => (i32::from(code), said),
            Err(error) => {
                said.push_str(&format!("error: {error}"));
                (3, said)
            }
        }
    }

    /// The result through the library entry, the forge listing the owner's
    /// key under its title.
    pub fn result_with_forge(&self, out: Option<PathBuf>) -> (Option<i32>, String) {
        let (code, said) = self.result_through(&self.owners_forge(), out);
        (Some(code), said)
    }

    /// The result through the library entry has this exit code and these
    /// words; what it said is returned.
    pub fn through_results_in(&self, forge: &StandIn, code: i32, words: &[&str]) -> String {
        let (found, said) = self.result_through(forge, None);
        assert_eq!(found, code, "{said}");
        for word in words {
            assert!(said.contains(word), "{word:?} is not in: {said}");
        }
        said
    }

    /// The same as `results_in`, through the library entry.
    pub fn signed_results_in(&self, code: i32, words: &[&str]) {
        self.through_results_in(&self.owners_forge(), code, words);
    }

    /// What the receipt says of a write-off approved in `commit`.
    pub fn approved_in(&self, commit: &str) -> String {
        let date = self.repo.git(&["log", "-1", "--format=%cI", commit]);
        format!(
            "Approved with the key \"{TITLE}\" ({}) of github.com/owner, in {commit} on {date}.",
            self.owner().fingerprint()
        )
    }
}

/// A stage for `three` with one fault of each standing: the first has a
/// test, the second an approved write-off, the third a write-off that is
/// approved or not; approved in one signed commit, which is returned.
pub fn three(name: &str, third_approved: bool) -> (Planned, Value, String) {
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
