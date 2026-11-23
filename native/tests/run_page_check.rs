//! `step check` tells the run page (`.kiro/specs/run-dashboard`, tasks 6.1
//! and 6.2): through the binary, on the fixture steps of the step tests, the
//! snapshot inside the page that `--page` writes. An accepted step, a step
//! refused before any run, and steps refused or not judged by their runs.

mod step_support;

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use step_support::{toy, Repo, APP};

const PINNED: &str = "if a > 3:\nstate.total = a\n";
const MORE: (&str, &str) = ("    return a\n", "    state.seen = True\n    return a\n");
const REMOVE: &str =
    "Remove the copies when you are done: delete the directory named under Your code.";
const TOKENS: &str = "tokens of the files in scope";

/// The snapshot inside a written report.
fn inside(report: &Path) -> Value {
    let text = fs::read_to_string(report).unwrap();
    let (_, rest) = text.split_once("const data = ").unwrap();
    let (data, _) = rest.split_once(";\n").unwrap();
    serde_json::from_str(data).unwrap()
}

/// Where the page of a step goes: beside the repository, not in it; or
/// into the directory `LESSCODE_CHECK_REPORTS` names, where the check of
/// the page in a browser (`native/page/states.spec.mjs`) reads the reports.
fn place(repo: &Repo, step: &str) -> PathBuf {
    let root = repo.dir.parent().unwrap();
    let Some(kept) = std::env::var_os("LESSCODE_CHECK_REPORTS") else {
        return root.join(format!("{step}.html"));
    };
    // The repository's name without the number of the process, so that
    // two tests with a step of the same name keep two pages.
    let name = root.file_name().unwrap().to_string_lossy();
    let test = name.trim_end_matches(|c: char| c.is_ascii_digit() || c == '-');
    PathBuf::from(kept).join(format!("{test}-{step}.html"))
}

fn verdict_path(repo: &Repo, step: &str) -> PathBuf {
    repo.dir.join(format!(".kiro/specs/{step}/verdict.json"))
}

/// What a check with the page gave: the exit code, the snapshot in the
/// page, the verdict, and what was said.
struct Checked {
    code: Option<i32>,
    shown: Value,
    verdict: Value,
    said: String,
}

/// Check a step with the page asked for, and with further options.
fn checked_with(repo: &Repo, step: &str, more: &[&str]) -> Checked {
    let place = place(repo, step);
    // A page kept from an earlier run of this test is not this run's.
    let _ = fs::remove_file(&place);
    let options = [&["--page", place.to_str().unwrap()], more].concat();
    let (code, stdout, stderr) = repo.step("check", step, &options);
    let said = format!("{stdout}{stderr}");
    assert!(place.is_file(), "no page was written: {said}");
    Checked {
        code,
        shown: inside(&place),
        verdict: repo.json(&verdict_path(repo, step)),
        said,
    }
}

fn checked(repo: &Repo, step: &str) -> Checked {
    checked_with(repo, step, &[])
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

/// The rows of a lens in the report.
fn lenses<'a>(shown: &'a Value, lens: &str) -> Vec<&'a Value> {
    let rows = shown["report"]["lenses"].as_array().unwrap();
    rows.iter().filter(|l| l["lens"] == lens).collect()
}

/// One row of a lens, by its measure.
fn lens<'a>(shown: &'a Value, lens: &str, measure: &str) -> &'a Value {
    let rows = lenses(shown, lens);
    let found = rows.iter().find(|l| l["measure"] == measure);
    found.unwrap_or_else(|| panic!("no {lens}, {measure}: {rows:?}"))
}

/// The one row of a lens that has one.
fn only<'a>(shown: &'a Value, lens: &str) -> &'a Value {
    let rows = lenses(shown, lens);
    assert_eq!(rows.len(), 1, "{lens}: {rows:?}");
    rows[0]
}

fn texts(list: &Value) -> Vec<&str> {
    let list = list.as_array().unwrap();
    list.iter().map(|x| x.as_str().unwrap()).collect()
}

/// The command tied to the criterion of the accepted step.
fn criterion() -> Vec<String> {
    let rest = ["--file", "src/app.py", "--expect", "state.total = a"];
    let mut command = vec![toy()];
    command.extend(rest.map(str::to_owned));
    command
}

/// A step that pins a write and then makes the function smaller; returns
/// the step's commit.
fn accepted_step(repo: &Repo) -> String {
    repo.begin(
        "smaller",
        json!({"criteria": [{"id": "1.1", "command": criterion()}]}),
    );
    repo.commit(&[("tests/expect.txt", PINNED)], "pin the write");
    let simpler = APP.replace("raise ValueError(\"big\")", "raise ValueError");
    repo.commit(&[("src/app.py", &simpler)], "the change")
}

/// Task 6.1: the run as it was told, for an accepted step.
fn the_run_is_told(shown: &Value, verdict: &Value, revision: &str) {
    assert_eq!(shown["command"], "step check");
    assert_eq!(shown["ended"], true);
    assert_eq!(shown["exit_code"], 0);
    assert_eq!(shown["outcome"], "Accepted");
    assert!(stages(shown).iter().all(|s| s.ends_with(": ended")));
    let ids: Vec<&str> = shown["stages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["id"].as_str().unwrap())
        .collect();
    let expected = [
        "checks",
        "tests-before",
        "faults-before",
        "tests-after",
        "faults-after",
        "verdict",
    ];
    assert_eq!(ids, expected);
    // What is judged: the step's branch and commit, and what it is compared with.
    let subject = &shown["subject"];
    assert_eq!(subject["branch"], "step/smaller");
    assert_eq!(subject["commit"], revision);
    assert_eq!(subject["compared_with"], verdict["base"]);
    assert_eq!(subject["on_main_branch"], false);
    // What the run writes into the repository, and where it works.
    let written = [
        ".kiro/specs/smaller/verdict.json",
        ".kiro/specs/smaller/evidence.json",
    ];
    assert_eq!(shown["writes_in_repository"], json!(written));
    let copies = &shown["copies"];
    let directory = copies["directory"].as_str().unwrap();
    assert!(directory.ends_with("work/smaller"), "{directory}");
    assert!(copies["bytes"].as_u64() > Some(0), "{copies}");
    let commands = shown["commands"].as_array().unwrap();
    assert_eq!(commands.len(), 2, "the test command and the criterion's");
    assert_eq!(commands[1], json!(criterion()));
    assert_eq!(shown["tests"]["passed_unchanged"], true);
}

/// Task 6.1: the faults of each side are those of the verdict.
fn the_faults_are_the_verdicts(shown: &Value, verdict: &Value) {
    let row = &verdict["functions"][0];
    assert_eq!(row["function"], "step");
    let mut planted = 0;
    for side in ["before", "after"] {
        let (page, judged) = (tally(shown, side, "step"), &row[side]);
        assert_eq!(page["noticed"], judged["noticed"], "{side}");
        assert_eq!(page["unnoticed"], judged["unnoticed"], "{side}");
        assert_eq!(page["left"], 0, "{side}");
        let all = ["noticed", "unnoticed", "other"];
        planted += all.iter().map(|k| judged[k].as_u64().unwrap()).sum::<u64>();
        // The same numbers in the report, per function and as the lens.
        let reported = &shown["report"]["functions"][0][side];
        assert_eq!(reported["noticed"], judged["noticed"], "{side}");
        assert_eq!(reported["unnoticed"], judged["unnoticed"], "{side}");
        assert_eq!(reported["neither"], judged["other"], "{side}");
        let of = |k: &str| judged[k].as_u64().unwrap();
        let sum = format!("{}; {}; {}", of("noticed"), of("unnoticed"), of("other"));
        assert_eq!(only(shown, "Protection")[side], sum.as_str(), "{side}");
    }
    assert!(planted > 0);
    assert_eq!(shown["faults"]["total"], planted);
    assert_eq!(shown["faults"]["done"], planted);
    assert_eq!(only(shown, "Protection")["moved"], true);
}

/// Task 6.2: every lens, measured or marked as not measured, with the
/// command its numbers are from.
fn the_report_has_every_lens(shown: &Value, verdict: &Value) {
    assert_eq!(shown["report"]["refusal"], Value::Null);
    let tokens = lens(shown, "Size", TOKENS);
    assert_eq!(tokens["before"], verdict["size"]["tokens"][0]);
    assert_eq!(tokens["after"], verdict["size"]["tokens"][1]);
    assert_eq!(tokens["expected"], "falls");
    assert_eq!(tokens["moved"], true);
    assert_eq!(tokens["command"], "lesscode-core step check");
    assert_eq!(tokens["handed_in"], false);
    let functions = lens(shown, "Structure", "functions");
    assert_eq!(functions["before"], 2);
    assert_eq!(functions["after"], 2);
    assert_eq!(functions["moved"], false, "the same number is no fall");
    let scope = "lesscode-core deconvolute measure-scope";
    assert_eq!(functions["command"], scope);
    for name in ["State", "Local complexity"] {
        let row = only(shown, name);
        assert_eq!(row["measured"], false, "{name}");
        assert_eq!(row["before"], Value::Null, "{name}");
        assert!(row["note"].as_str().unwrap().contains("--measures"));
    }
    let kept = only(shown, "Behaviour kept");
    assert_eq!(kept["before"], "pass");
    assert_eq!(kept["after"], "pass");
    assert_eq!(kept["moved"], true);
    assert!(kept["command"].as_str().unwrap().contains("--expect-from"));
    // Names and no source text, unless asked (requirement 7.9).
    assert_eq!(shown["faults"]["example"]["text"], "");
    assert_eq!(shown["faults"]["example"]["file"], "src/app.py");
    // The tests noticed faults before the step, and the page says no more.
    let detail = shown["outcome_detail"].as_str().unwrap();
    assert!(detail.starts_with("Every check passed."), "{detail}");
    assert!(!detail.contains("nothing was protected before"), "{detail}");
    let next = texts(&shown["next"]);
    assert!(next[0].contains("step record"), "{next:?}");
    assert_eq!(next.last(), Some(&REMOVE));
}

#[test]
fn an_accepted_step_gives_the_snapshot_of_its_two_runs() {
    let repo = Repo::new("page-accepted");
    let revision = accepted_step(&repo);

    // Without the page first: the verdict the page must not change.
    let _ = fs::remove_file(place(&repo, "smaller"));
    let (code, _, said) = repo.step("check", "smaller", &[]);
    assert_eq!(code, Some(0), "{said}");
    let plain = fs::read(verdict_path(&repo, "smaller")).unwrap();
    assert!(!place(&repo, "smaller").exists(), "no page unless asked");

    let run = checked(&repo, "smaller");
    assert_eq!(run.code, Some(0), "{}", run.said);
    assert_eq!(fs::read(verdict_path(&repo, "smaller")).unwrap(), plain);
    assert!(run.said.contains("The page of the run is at"));
    the_run_is_told(&run.shown, &run.verdict, &revision);
    the_faults_are_the_verdicts(&run.shown, &run.verdict);
    the_report_has_every_lens(&run.shown, &run.verdict);
}

/// A file of numbers measured elsewhere, for the two lenses the tool does
/// not measure.
fn measures(repo: &Repo, state: &str) -> PathBuf {
    let handed = json!({"schema": 1, "lenses": [
        {"lens": state, "measure": "ways Frame can be written", "before": 16, "after": 6,
         "expected": "falls", "command": "state-holders --type Frame"},
        {"lens": "Local complexity", "measure": "step: decisions", "before": 1, "after": 3,
         "expected": "no routine above its limit gets worse", "moved": true,
         "command": "kiro-cli code check"}],
      "shapes": [{"type": "Frame", "side": "before", "command": "state-walk --type Frame",
         "cells": [{"shape": "flag no", "reached": true, "seen": false},
                   {"shape": "flag yes", "reached": false}]}]});
    let path = repo.dir.parent().unwrap().join("measures.json");
    fs::write(&path, handed.to_string()).unwrap();
    path
}

/// Task 6.3: the functions and calls of both sides as the tool counts
/// them, and the shapes as they were handed in.
fn the_drawings_have_their_numbers(report: &Value) {
    let file = |side: &str| &report["graph"][side]["files"][0];
    for side in ["before", "after"] {
        assert_eq!(file(side)["file"], "src/app.py", "{side}");
        assert_eq!(file(side)["functions"], json!(["helper", "step"]));
        assert_eq!(report["graph"][side]["edges"], json!([]), "{side}");
    }
    let shapes = &report["shapes"];
    assert_eq!(shapes.as_array().unwrap().len(), 1);
    assert_eq!(shapes[0]["type"], "Frame");
    assert_eq!(shapes[0]["side"], "before");
    assert_eq!(shapes[0]["command"], "state-walk --type Frame");
    let cells = shapes[0]["cells"].as_array().unwrap();
    assert_eq!(cells.len(), 2);
    assert_eq!(cells[0]["seen"], false);
    assert_eq!(cells[1]["seen"], Value::Null, "not measured stays so");
}

#[test]
fn numbers_handed_in_and_source_text_are_shown_only_when_asked_for() {
    let repo = Repo::new("page-handed");
    accepted_step(&repo);
    let file = measures(&repo, "State");
    let more = ["--page-source", "--measures", file.to_str().unwrap()];
    let run = checked_with(&repo, "smaller", &more);
    assert_eq!(run.code, Some(0), "{}", run.said);
    assert_ne!(run.shown["faults"]["example"]["text"], "");
    let state = only(&run.shown, "State");
    assert_eq!(state["handed_in"], true);
    assert_eq!(state["before"], 16);
    assert_eq!(state["after"], 6);
    assert_eq!(state["moved"], true, "computed from the numbers");
    assert_eq!(state["command"], "state-holders --type Frame");
    let local = only(&run.shown, "Local complexity");
    assert_eq!(local["measured"], true);
    assert_eq!(local["moved"], true, "as the file says");
    assert_eq!(lens(&run.shown, "Size", TOKENS)["handed_in"], false);
    the_drawings_have_their_numbers(&run.shown["report"]);

    // A file that hands in a lens the run measures itself, and a file
    // without the page, are bad input: nothing runs.
    let file = measures(&repo, "Size");
    let place = place(&repo, "smaller");
    let page = ["--page", place.to_str().unwrap()];
    let handed = ["--measures", file.to_str().unwrap()];
    let (code, _, stderr) = repo.step("check", "smaller", &[page, handed].concat());
    assert_eq!(code, Some(3), "{stderr}");
    assert!(stderr.contains("can be handed in"), "{stderr}");
    let (code, _, stderr) = repo.step("check", "smaller", &handed);
    assert_eq!(code, Some(3), "{stderr}");
    assert!(stderr.contains("--page"), "{stderr}");
}

#[test]
fn shapes_without_a_cell_or_on_no_side_are_bad_input() {
    let repo = Repo::new("page-shapes");
    accepted_step(&repo);
    let place = place(&repo, "smaller");
    let file = repo.dir.parent().unwrap().join("shapes.json");
    let cell = json!({"shape": "flag no", "reached": true});
    let cases = [
        (
            json!({"type": "Frame", "command": "walk", "cells": []}),
            "no shape is listed",
        ),
        (
            json!({"type": "Frame", "side": "sideways", "command": "walk", "cells": [cell]}),
            "before or after",
        ),
        (
            json!({"type": "Frame", "command": " ", "cells": [cell]}),
            "the command that counted them",
        ),
    ];
    for (shapes, why) in cases {
        let handed = json!({"schema": 1, "shapes": [shapes]});
        fs::write(&file, handed.to_string()).unwrap();
        let options = [
            "--page",
            place.to_str().unwrap(),
            "--measures",
            file.to_str().unwrap(),
        ];
        let (code, _, stderr) = repo.step("check", "smaller", &options);
        assert_eq!(code, Some(3), "{stderr}");
        assert!(stderr.contains(why), "{why}: {stderr}");
        assert!(
            !verdict_path(&repo, "smaller").exists(),
            "nothing was judged"
        );
    }
}

#[test]
fn an_acceptance_where_nothing_was_protected_says_so() {
    // A step on a function no test looks at: accepted, and the page says
    // what that acceptance is worth (requirement 6.4).
    let repo = Repo::new("page-bare");
    repo.begin("pins", json!({"focus": {"src/app.py": ["helper"]}}));
    repo.commit(&[("tests/expect.txt", PINNED)], "a pin elsewhere");
    let run = checked(&repo, "pins");
    assert_eq!(run.code, Some(0), "{}\n{}", run.said, run.verdict);
    let shown = &run.shown;
    assert_eq!(tally(shown, "before", "helper")["noticed"], 0);
    assert_eq!(tally(shown, "before", "helper")["unprotected"], true);
    let detail = shown["outcome_detail"].as_str().unwrap();
    let named = "No test noticed any fault before the step in: helper.";
    assert!(detail.contains(named), "{detail}");
    let worth = "means only that nothing got worse; nothing was protected before";
    assert!(detail.contains(worth), "{detail}");
    let next = texts(&shown["next"]);
    assert!(next[1].starts_with("Tests come first"), "{next:?}");
}

/// The report of a refusal before any run has the same rows: what the
/// commits show is measured, what needs a run is marked as not measured.
fn the_report_of_an_early_refusal(shown: &Value) {
    assert_eq!(lens(shown, "Size", TOKENS)["moved"], false, "it grew");
    assert_eq!(lens(shown, "Structure", "functions")["measured"], true);
    for name in ["State", "Local complexity", "Behaviour kept", "Protection"] {
        let row = only(shown, name);
        assert_eq!(row["measured"], false, "{name}");
        assert!(!row["note"].as_str().unwrap().is_empty(), "{name}");
    }
    assert_eq!(shown["report"]["functions"], json!([]));
    // The drawing of functions and calls needs no run; no shapes were handed in.
    let functions = &shown["report"]["graph"]["after"]["files"][0]["functions"];
    assert_eq!(functions, &json!(["helper", "step"]));
    assert_eq!(shown["report"]["shapes"], json!([]));
}

#[test]
fn a_step_refused_before_any_run_ends_the_page_at_once_with_the_reasons() {
    let repo = Repo::new("page-static");
    let more = APP.replace(MORE.0, MORE.1);
    repo.begin("grows", json!({}));
    repo.commit(&[("src/app.py", &more)], "add a write");

    let run = checked(&repo, "grows");
    let (shown, verdict) = (&run.shown, &run.verdict);
    assert_eq!(run.code, Some(1), "{}", run.said);
    assert_eq!(shown["ended"], true);
    assert_eq!(shown["exit_code"], 1);
    assert_eq!(shown["outcome"], "Refused");
    let said = stages(shown);
    assert_eq!(said[0], "checks: failed");
    assert!(
        said[1..].iter().all(|s| s.ends_with(": left out")),
        "{said:?}"
    );
    // The reasons are the verdict's, where the refusal is shown and at the
    // stage that failed.
    let reasons = &shown["report"]["refusal"]["reasons"];
    assert_eq!(reasons, &verdict["reasons"]);
    let reason = texts(reasons)[0];
    assert!(reason.contains("the declaration allows 0 more"), "{reason}");
    assert_eq!(shown["stages"][0]["reason"], reason);
    assert_eq!(shown["outcome_detail"], reason);
    the_report_of_an_early_refusal(shown);
    // Nothing ran: no fault, no copy, and only the verdict was written.
    assert_eq!(shown["faults"]["total"], 0);
    assert_eq!(shown["copies"]["bytes"], Value::Null);
    assert!(!repo.work.join("grows").exists());
    let written = [".kiro/specs/grows/verdict.json"];
    assert_eq!(shown["writes_in_repository"], json!(written));
    let next = texts(&shown["next"]);
    assert_eq!(next.len(), 1, "no copies to remove: {next:?}");
    assert!(next[0].contains("Fix the step, not the limit."), "{next:?}");
}

#[test]
fn a_page_beside_the_verdict_is_refused_before_anything_is_judged() {
    // Without a named place the page would lie beside the verdict, among
    // the repository's files.
    let repo = Repo::new("page-place");
    repo.begin("again", json!({}));
    repo.commit(&[("tests/expect.txt", PINNED)], "a pin");
    let (code, stdout, stderr) = repo.step("check", "again", &["--page"]);
    let said = format!("{stdout}{stderr}");
    assert_eq!(code, Some(3), "{said}");
    assert!(said.contains("Refused"), "{said}");
    assert!(said.contains("--page <place>"), "{said}");
    assert!(!verdict_path(&repo, "again").exists());
    assert!(!verdict_path(&repo, "again").with_extension("html").exists());
}

#[test]
fn a_step_whose_faults_refuse_it_shows_every_stage_and_the_reasons() {
    // More faults go unnoticed after the step: every stage ended, and the
    // refusal is the verdict's.
    let repo = Repo::new("page-faults");
    let more = APP.replace(MORE.0, MORE.1);
    repo.begin("unprotected", json!({"max_token_growth": 50}));
    repo.commit(&[("src/app.py", &more)], "add a write");
    let run = checked(&repo, "unprotected");
    let shown = &run.shown;
    assert_eq!(run.code, Some(1), "{}", run.said);
    assert_eq!(shown["outcome"], "Refused");
    assert!(stages(shown).iter().all(|s| s.ends_with(": ended")));
    let reasons = &shown["report"]["refusal"]["reasons"];
    assert_eq!(reasons, &run.verdict["reasons"]);
    assert!(texts(reasons)[0].contains("unnoticed faults rise from"));
    let was = tally(shown, "before", "step")["unnoticed"].as_u64();
    let is = tally(shown, "after", "step")["unnoticed"].as_u64();
    assert!(is > was, "{was:?} to {is:?}");
    assert_eq!(only(shown, "Protection")["moved"], false);
    assert_eq!(texts(&shown["next"]).last(), Some(&REMOVE));
}

#[test]
fn a_step_whose_tests_fail_or_cannot_run_shows_how_far_it_came() {
    let repo = Repo::new("page-runs");
    // The tests fail after the step: that stage fails with the reason,
    // and the tests on the unchanged code are not said to fail.
    repo.begin("broken", json!({}));
    let broken = APP.replace("a > 3", "a > 4");
    repo.commit(&[("src/app.py", &broken)], "change the limit");
    let run = checked(&repo, "broken");
    let shown = &run.shown;
    assert_eq!(run.code, Some(1), "{}", run.said);
    assert_eq!(shown["outcome"], "Refused");
    let said = stages(shown);
    assert_eq!(said[2], "faults-before: ended");
    assert_eq!(said[3], "tests-after: failed");
    assert_eq!(said[4..], ["faults-after: left out", "verdict: left out"]);
    let reason = shown["stages"][3]["reason"].as_str().unwrap();
    assert!(reason.contains("tests do not pass on the step"), "{reason}");
    assert_eq!(shown["tests"]["passed_unchanged"], true);
    let kept = only(shown, "Behaviour kept");
    assert_eq!(kept["before"], "pass");
    assert_eq!(kept["after"], "do not pass");
    assert_eq!(kept["moved"], false);
    assert_eq!(only(shown, "Protection")["measured"], false);
    assert_eq!(shown["copies"]["working"], 0);
    assert!(!shown["copies"]["dropped"].as_array().unwrap().is_empty());

    // A test command that cannot run: not judged, and nothing is refused.
    repo.begin("void", json!({"test": ["no-such-command-for-this-test"]}));
    repo.commit(&[("tests/expect.txt", PINNED)], "a pin");
    let run = checked(&repo, "void");
    let shown = &run.shown;
    assert_eq!(run.code, Some(2), "{}", run.said);
    assert_eq!(shown["outcome"], "Not judged");
    assert_eq!(shown["exit_code"], 2);
    assert_eq!(shown["report"]["refusal"], Value::Null);
    assert_eq!(only(shown, "Behaviour kept")["measured"], false);
    assert_eq!(stages(shown)[1], "tests-before: failed");
    let reasons = texts(&run.verdict["reasons"]).join("; ");
    assert_eq!(shown["outcome_detail"], reasons.as_str());
}
