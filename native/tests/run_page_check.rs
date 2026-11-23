//! `step check` tells the run page (`.kiro/specs/run-dashboard`, task 6.1):
//! through the binary, on the fixture steps of the step tests, the snapshot
//! inside the page that `--page` writes. An accepted step, a step refused
//! before any run, and steps refused or not judged by their runs.

mod step_support;

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use step_support::{toy, Repo, APP};

const PINNED: &str = "if a > 3:\nstate.total = a\n";
const MORE: (&str, &str) = ("    return a\n", "    state.seen = True\n    return a\n");
const REMOVE: &str =
    "Remove the copies when you are done: delete the directory named under Your code.";

/// The snapshot inside a written report.
fn inside(report: &Path) -> Value {
    let text = fs::read_to_string(report).unwrap();
    let (_, rest) = text.split_once("const data = ").unwrap();
    let (data, _) = rest.split_once(";\n").unwrap();
    serde_json::from_str(data).unwrap()
}

/// Where the page of a step goes: beside the repository, not in it.
fn place(repo: &Repo, step: &str) -> PathBuf {
    repo.dir.parent().unwrap().join(format!("{step}.html"))
}

fn verdict_path(repo: &Repo, step: &str) -> PathBuf {
    repo.dir.join(format!(".kiro/specs/{step}/verdict.json"))
}

/// Check a step with the page asked for: the exit code, the snapshot in
/// the page, the verdict, and what was said.
fn checked(repo: &Repo, step: &str) -> (Option<i32>, Value, Value, String) {
    let place = place(repo, step);
    let (code, stdout, stderr) = repo.step("check", step, &["--page", place.to_str().unwrap()]);
    let said = format!("{stdout}{stderr}");
    assert!(place.is_file(), "no page was written: {said}");
    let verdict = repo.json(&verdict_path(repo, step));
    (code, inside(&place), verdict, said)
}

fn stages(shown: &Value) -> Vec<String> {
    let stages = shown["stages"].as_array().unwrap();
    let said = |s: &Value| {
        format!(
            "{}: {}",
            s["id"].as_str().unwrap(),
            s["status"].as_str().unwrap()
        )
    };
    stages.iter().map(said).collect()
}

/// The tally of a function on one side.
fn tally<'a>(shown: &'a Value, side: &str, function: &str) -> &'a Value {
    let functions = shown["faults"]["functions"].as_array().unwrap();
    let found = functions
        .iter()
        .find(|f| f["side"] == side && f["function"] == function);
    found.unwrap_or_else(|| panic!("no {function} {side}: {functions:?}"))
}

fn texts(list: &Value) -> Vec<&str> {
    let list = list.as_array().unwrap();
    list.iter().map(|x| x.as_str().unwrap()).collect()
}

#[test]
fn an_accepted_step_gives_the_snapshot_of_its_two_runs() {
    let repo = Repo::new("page-accepted");
    let criterion = [
        toy(),
        "--file".into(),
        "src/app.py".into(),
        "--expect".into(),
        "state.total = a".into(),
    ];
    repo.begin(
        "smaller",
        json!({"criteria": [{"id": "1.1", "command": criterion}]}),
    );
    repo.commit(&[("tests/expect.txt", PINNED)], "pin the write");
    let simpler = APP.replace("raise ValueError(\"big\")", "raise ValueError");
    let revision = repo.commit(&[("src/app.py", &simpler)], "the change");

    // Without the page first: the verdict the page must not change.
    let (code, _, said) = repo.step("check", "smaller", &[]);
    assert_eq!(code, Some(0), "{said}");
    let plain = fs::read(verdict_path(&repo, "smaller")).unwrap();
    assert!(!place(&repo, "smaller").exists(), "no page unless asked");

    let (code, shown, verdict, said) = checked(&repo, "smaller");
    assert_eq!(code, Some(0), "{said}");
    assert_eq!(fs::read(verdict_path(&repo, "smaller")).unwrap(), plain);
    assert!(said.contains("The page of the run is at"), "{said}");

    assert_eq!(shown["command"], "step check");
    assert_eq!(shown["ended"], true);
    assert_eq!(shown["exit_code"], 0);
    assert_eq!(shown["outcome"], "Accepted");
    assert_eq!(
        stages(&shown),
        [
            "checks: ended",
            "tests-before: ended",
            "faults-before: ended",
            "tests-after: ended",
            "faults-after: ended",
            "verdict: ended"
        ]
    );
    // What is judged: the step's branch and commit, and what it is compared with.
    let subject = &shown["subject"];
    assert_eq!(subject["branch"], "step/smaller");
    assert_eq!(subject["commit"], revision.as_str());
    assert_eq!(subject["compared_with"], verdict["base"]);
    assert_eq!(subject["on_main_branch"], false);
    // What the run writes into the repository, and where it works.
    assert_eq!(
        shown["writes_in_repository"],
        json!([
            ".kiro/specs/smaller/verdict.json",
            ".kiro/specs/smaller/evidence.json"
        ])
    );
    let copies = &shown["copies"];
    let directory = copies["directory"].as_str().unwrap();
    assert!(directory.ends_with("work/smaller"), "{directory}");
    assert!(copies["bytes"].as_u64() > Some(0), "{copies}");
    let commands = shown["commands"].as_array().unwrap();
    assert_eq!(commands.len(), 2, "the test command and the criterion's");
    assert_eq!(commands[1], json!(criterion));

    // The faults of each side are those of the verdict.
    let row = &verdict["functions"][0];
    assert_eq!(row["function"], "step");
    let mut planted = 0;
    for side in ["before", "after"] {
        let (page, judged) = (tally(&shown, side, "step"), &row[side]);
        assert_eq!(page["noticed"], judged["noticed"], "{side}");
        assert_eq!(page["unnoticed"], judged["unnoticed"], "{side}");
        assert_eq!(page["left"], 0, "{side}");
        let all = ["noticed", "unnoticed", "other"];
        planted += all.iter().map(|k| judged[k].as_u64().unwrap()).sum::<u64>();
    }
    assert!(planted > 0);
    assert_eq!(shown["faults"]["total"], planted);
    assert_eq!(shown["faults"]["done"], planted);
    assert_eq!(shown["tests"]["passed_unchanged"], true);

    // Nothing was refused, and the tests noticed faults before the step.
    assert_eq!(shown.get("report"), None);
    let detail = shown["outcome_detail"].as_str().unwrap();
    assert!(detail.starts_with("Every check passed."), "{detail}");
    assert!(!detail.contains("nothing was protected before"), "{detail}");
    let next = texts(&shown["next"]);
    assert!(next[0].contains("step record"), "{next:?}");
    assert_eq!(next.last(), Some(&REMOVE));

    // A step on a function no test looks at: accepted, and the page says
    // what that acceptance is worth (requirement 6.4).
    repo.begin("pins", json!({"focus": {"src/app.py": ["helper"]}}));
    repo.commit(&[("tests/expect.txt", PINNED)], "a pin elsewhere");
    let (code, shown, verdict, said) = checked(&repo, "pins");
    assert_eq!(code, Some(0), "{said}\n{verdict}");
    assert_eq!(tally(&shown, "before", "helper")["noticed"], 0);
    assert_eq!(tally(&shown, "before", "helper")["unprotected"], true);
    let detail = shown["outcome_detail"].as_str().unwrap();
    assert!(
        detail.contains("No test noticed any fault before the step in: helper."),
        "{detail}"
    );
    assert!(
        detail.contains("means only that nothing got worse; nothing was protected before"),
        "{detail}"
    );
    let next = texts(&shown["next"]);
    assert!(next[1].starts_with("Tests come first"), "{next:?}");
}

#[test]
fn a_step_refused_before_any_run_ends_the_page_at_once_with_the_reasons() {
    let repo = Repo::new("page-static");
    let more = APP.replace(MORE.0, MORE.1);
    repo.begin("grows", json!({}));
    repo.commit(&[("src/app.py", &more)], "add a write");

    let (code, shown, verdict, said) = checked(&repo, "grows");
    assert_eq!(code, Some(1), "{said}");
    assert_eq!(shown["ended"], true);
    assert_eq!(shown["exit_code"], 1);
    assert_eq!(shown["outcome"], "Refused");
    assert_eq!(
        stages(&shown),
        [
            "checks: failed",
            "tests-before: left out",
            "faults-before: left out",
            "tests-after: left out",
            "faults-after: left out",
            "verdict: left out"
        ]
    );
    // The reasons are the verdict's, where the refusal is shown and at the
    // stage that failed.
    let reasons = &shown["report"]["refusal"]["reasons"];
    assert_eq!(reasons, &verdict["reasons"]);
    let reason = texts(reasons)[0];
    assert!(reason.contains("the declaration allows 0 more"), "{reason}");
    assert_eq!(shown["stages"][0]["reason"], reason);
    assert_eq!(shown["outcome_detail"], reason);
    // Nothing ran: no fault, no copy, and only the verdict was written.
    assert_eq!(shown["faults"]["total"], 0);
    assert_eq!(shown["copies"]["bytes"], Value::Null);
    assert!(!repo.work.join("grows").exists());
    assert_eq!(
        shown["writes_in_repository"],
        json!([".kiro/specs/grows/verdict.json"])
    );
    let next = texts(&shown["next"]);
    assert_eq!(next.len(), 1, "no copies to remove: {next:?}");
    assert!(next[0].contains("Fix the step, not the limit."), "{next:?}");

    // Without a named place the page would lie beside the verdict, among
    // the repository's files: refused before anything is judged or written.
    repo.begin("again", json!({}));
    repo.commit(&[("src/app.py", &more)], "add a write");
    let (code, stdout, stderr) = repo.step("check", "again", &["--page"]);
    let said = format!("{stdout}{stderr}");
    assert_eq!(code, Some(3), "{said}");
    assert!(
        said.contains("Refused") && said.contains("--page <place>"),
        "{said}"
    );
    assert!(!verdict_path(&repo, "again").exists());
    assert!(!verdict_path(&repo, "again").with_extension("html").exists());
}

#[test]
fn a_step_its_runs_refuse_or_cannot_judge_shows_how_far_it_came() {
    let repo = Repo::new("page-runs");
    let more = APP.replace(MORE.0, MORE.1);

    // More faults go unnoticed after the step: every stage ended, and the
    // refusal is the verdict's.
    repo.begin("unprotected", json!({"max_token_growth": 50}));
    repo.commit(&[("src/app.py", &more)], "add a write");
    let (code, shown, verdict, said) = checked(&repo, "unprotected");
    assert_eq!(code, Some(1), "{said}");
    assert_eq!(shown["outcome"], "Refused");
    assert!(stages(&shown).iter().all(|s| s.ends_with(": ended")));
    let reasons = &shown["report"]["refusal"]["reasons"];
    assert_eq!(reasons, &verdict["reasons"]);
    assert!(texts(reasons)[0].contains("unnoticed faults rise from"));
    let (was, is) = (
        tally(&shown, "before", "step"),
        tally(&shown, "after", "step"),
    );
    assert!(is["unnoticed"].as_u64() > was["unnoticed"].as_u64());
    assert_eq!(texts(&shown["next"]).last(), Some(&REMOVE));

    // The tests fail after the step: that stage fails with the reason,
    // and the tests on the unchanged code are not said to fail.
    repo.begin("broken", json!({}));
    repo.commit(
        &[("src/app.py", &APP.replace("a > 3", "a > 4"))],
        "change the limit",
    );
    let (code, shown, _, said) = checked(&repo, "broken");
    assert_eq!(code, Some(1), "{said}");
    assert_eq!(shown["outcome"], "Refused");
    assert_eq!(
        stages(&shown),
        [
            "checks: ended",
            "tests-before: ended",
            "faults-before: ended",
            "tests-after: failed",
            "faults-after: left out",
            "verdict: left out"
        ]
    );
    let reason = shown["stages"][3]["reason"].as_str().unwrap();
    assert!(reason.contains("tests do not pass on the step"), "{reason}");
    assert_eq!(shown["tests"]["passed_unchanged"], true);
    assert_eq!(shown["copies"]["working"], 0);
    assert!(!shown["copies"]["dropped"].as_array().unwrap().is_empty());

    // A test command that cannot run: not judged, and nothing is refused.
    repo.begin("void", json!({"test": ["no-such-command-for-this-test"]}));
    repo.commit(&[("tests/expect.txt", PINNED)], "a pin");
    let (code, shown, verdict, said) = checked(&repo, "void");
    assert_eq!(code, Some(2), "{said}");
    assert_eq!(shown["outcome"], "Not judged");
    assert_eq!(shown["exit_code"], 2);
    assert_eq!(shown.get("report"), None);
    assert_eq!(
        stages(&shown)[..2],
        ["checks: ended", "tests-before: failed"]
    );
    assert_eq!(
        shown["outcome_detail"],
        texts(&verdict["reasons"]).join("; ")
    );
}
