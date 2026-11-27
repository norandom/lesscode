//! `step reports` (`.kiro/specs/run-dashboard`, task 7.1): through the
//! binary, a list of three reports the tool wrote for fixture steps, one
//! accepted and two refused; and the option that leaves the names of a
//! project out of the list and of its reports.

mod step_support;

use lesscode_core::dashboard::names::leave_out;
use lesscode_core::dashboard::text::data_of;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use step_support::{toy, Repo, APP};

const PINNED: &str = "if a > 3:\nstate.total = a\n";
const MORE: (&str, &str) = ("    return a\n", "    state.seen = True\n    return a\n");
const LEFT: &str = "a command of the project, left out";
/// Names of the fixture project that no sentence of the tool has.
const NAMES: [&str; 4] = ["app.py", "expect.txt", "helper", "toy_check"];

/// A run with names inside names, a function named like a common word, a
/// row that was handed in, and shapes of a type.
fn a_run() -> Value {
    json!({
        "schema": 1, "tool": {"name": "lesscode-core", "version": "2.1.0"},
        "command": "step check", "exit_code": 1, "outcome": "Refused",
        "outcome_detail": "src/data.py: run: faults rise; runner in src/a.py is not covered",
        "subject": {"repository": "D:/Source/secret-project/", "branch": "step/run"},
        "commands": [["pytest", "tests/test_data.py::test_run"]],
        "writes_in_repository": [".kiro/specs/run/verdict.json"],
        "stages": [{"id": "checks", "title": "Check what the commits of the run show",
            "what": "Nothing is run here.", "reason": "Frame.total is written in run"}],
        "faults": {
            "example": {"file": "src/data.py", "function": "run", "text": "if a > b:",
                "outcome": "noticed: a test failed"},
            "functions": [{"file": "src/data.py", "function": "run",
                "reaches_out": ["Frame.send"]}]},
        "report": {
            "lenses": [
                {"lens": "Size", "measure": "tokens of the files, in one run", "note": null,
                    "handed_in": false,
                    "command": "lesscode-core deconvolute measure-scope --file src/a.py"},
                {"lens": "State", "measure": "writes to Frame outside run",
                    "note": "counted for run", "handed_in": true,
                    "command": "kiro-cli state src/data.py"}],
            "functions": [{"file": "src/a.py", "function": "Frame.send"}],
            "graph": {
                "before": {
                    "files": [{"file": "src/a.py", "functions": ["Frame.send", "runner"]}],
                    "edges": [["src/data.py", "run", "src/a.py", "Frame.send"]]},
                "after": {"files": [], "edges": []}},
            "shapes": [{"type": "Frame", "command": "kiro-cli shapes",
                "cells": [{"shape": "total=0, sent=false"}, {"shape": "total>0, sent=true"}]}],
            "refusal": {"reasons": ["src/data.py: run: faults rise"]}},
        "next": ["Fix the step, then run the check again."]
    })
}

#[test]
fn every_name_of_a_run_gets_a_number_wherever_it_stands() {
    let mut run = a_run();
    leave_out(&mut run);
    // Files by their place in the alphabet, then functions, then types.
    let detail = "file 2: function 2: faults rise; function 3 in file 1 is not covered";
    assert_eq!(run["outcome_detail"], detail);
    assert_eq!(
        run["report"]["refusal"]["reasons"],
        json!(["file 2: function 2: faults rise"])
    );
    let reason = "type 1.total is written in function 2";
    assert_eq!(run["stages"][0]["reason"], reason);
    let fault = &run["faults"]["functions"][0];
    assert_eq!(fault["file"], "file 2");
    assert_eq!(fault["function"], "function 2");
    assert_eq!(fault["reaches_out"], json!(["function 1"]));
    let before = &run["report"]["graph"]["before"];
    let names = json!(["function 1", "function 3"]);
    assert_eq!(before["files"][0]["functions"], names);
    let edge = json!([["file 2", "function 2", "file 1", "function 1"]]);
    assert_eq!(before["edges"], edge);
    // A row that was handed in is the project's text; its command is left out.
    let handed = &run["report"]["lenses"][1];
    assert_eq!(handed["measure"], "writes to type 1 outside function 2");
    assert_eq!(handed["note"], "counted for function 2");
    assert_eq!(handed["command"], LEFT);
    let text = run.to_string();
    for name in ["a.py", "Frame", "runner", "pytest", "verdict.json", "sent="] {
        assert!(!text.contains(name), "{name} is still in {text}");
    }
}

#[test]
fn what_is_left_out_as_a_whole_and_what_the_tool_says_stays() {
    let (was, mut run) = (a_run(), a_run());
    leave_out(&mut run);
    assert_eq!(run["commands"], json!([[LEFT]]));
    let written = json!(["a file the run wrote, name left out"]);
    assert_eq!(run["writes_in_repository"], written);
    assert_eq!(run["faults"]["example"]["text"], "");
    let shapes = &run["report"]["shapes"][0];
    assert_eq!(shapes["type"], "type 1");
    assert_eq!(shapes["command"], LEFT);
    assert_eq!(
        shapes["cells"],
        json!([{"shape": "shape 1"}, {"shape": "shape 2"}])
    );
    let project = "secret-project; the names of its files and functions are left out";
    assert_eq!(run["subject"]["repository"], project);
    // The tool's own sentences keep the word that is also a function's name.
    for at in [
        "/command",
        "/outcome",
        "/next",
        "/stages/0/title",
        "/stages/0/what",
        "/subject/branch",
        "/faults/example/outcome",
        "/report/lenses/0/measure",
    ] {
        assert_eq!(run.pointer(at), was.pointer(at), "{at}");
    }
    // A command of the tool keeps its words and loses the file.
    let command = "lesscode-core deconvolute measure-scope --file file 1";
    assert_eq!(run["report"]["lenses"][0]["command"], command);
    // The last sentence of an accepted run is the tool's own.
    let mut accepted = a_run();
    accepted["exit_code"] = json!(0);
    accepted["outcome_detail"] = json!("In no function did the run find more.");
    leave_out(&mut accepted);
    let detail = "In no function did the run find more.";
    assert_eq!(accepted["outcome_detail"], detail);
}

/// The run or the list inside a written page.
fn inside(page: &Path) -> Value {
    data_of(&fs::read_to_string(page).unwrap()).unwrap()
}

/// Checks a step on a new branch and returns its report.
fn report(repo: &Repo, step: &str, extra: Value, files: &[(&str, &str)], code: i32) -> PathBuf {
    repo.begin(step, extra);
    for file in files {
        repo.commit(&[*file], "a commit of the step");
    }
    let place = repo.dir.parent().unwrap().join(format!("{step}.html"));
    let (found, out, err) = repo.step("check", step, &["--page", place.to_str().unwrap()]);
    assert_eq!(found, Some(code), "{step}: {out}{err}");
    place
}

/// Three reports of one repository: a step that is accepted, one that its
/// planted faults refuse, and one that is refused before anything runs.
fn three_reports(repo: &Repo) -> Vec<PathBuf> {
    let simpler = APP.replace("raise ValueError(\"big\")", "raise ValueError");
    let more = APP.replace(MORE.0, MORE.1);
    let tied = json!([toy(), "--file", "src/app.py", "--expect", "state.total = a"]);
    let criteria = json!({"criteria": [{"id": "1.1", "command": tied}]});
    let pin = ("tests/expect.txt", PINNED);
    let smaller = [pin, ("src/app.py", simpler.as_str())];
    let grown = [("src/app.py", more.as_str())];
    let room = json!({"max_token_growth": 50});
    let accepted = report(repo, "smaller", criteria, &smaller, 0);
    let unprotected = report(repo, "unprotected", room, &grown, 1);
    vec![
        accepted,
        unprotected,
        report(repo, "grows", json!({}), &grown, 1),
    ]
}

/// Where a list goes: beside the repository; or into the directory
/// `LESSCODE_CHECK_REPORTS` names, where the check of the page in a browser
/// (`native/page/list.spec.mjs`) opens it.
fn place(repo: &Repo, name: &str) -> PathBuf {
    let out = match std::env::var_os("LESSCODE_CHECK_REPORTS") {
        Some(kept) => PathBuf::from(kept).join(name),
        None => repo.dir.parent().unwrap().join(name),
    };
    let _ = fs::remove_dir_all(&out);
    out
}

/// `step reports` on `pages`, with further options.
fn listed(out: &Path, pages: &[PathBuf], more: &[&str]) -> (Option<i32>, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_lesscode-core"))
        .args(["step", "reports", "--out"])
        .arg(out)
        .args(more)
        .args(pages)
        .output()
        .expect("run lesscode-core binary");
    let said = [output.stdout, output.stderr].concat();
    (
        output.status.code(),
        String::from_utf8_lossy(&said).into_owned(),
    )
}

fn rows(out: &Path) -> Vec<Value> {
    let list = inside(&out.join("index.html"));
    assert_eq!(list["schema"], 1);
    assert_eq!(list["tool"]["version"], env!("CARGO_PKG_VERSION"));
    list["reports"].as_array().unwrap().clone()
}

fn keys(row: &Value) -> Vec<String> {
    row.as_object().unwrap().keys().cloned().collect()
}

/// Requirements 8.1 to 8.3: project, scope, date and lenses of each; a
/// link to each; a refusal and a measure that got worse like the others.
fn all_three_stand_alike(repo: &Repo, pages: &[PathBuf]) {
    let out = place(repo, "list");
    let (code, said) = listed(&out, pages, &[]);
    assert_eq!(code, Some(0), "{said}");
    assert!(said.contains("The list of 3 reports is at"), "{said}");
    let rows = rows(&out);
    assert_eq!(rows.len(), 3);
    for (at, (row, page)) in rows.iter().zip(pages).enumerate() {
        let run = inside(page);
        assert_eq!(keys(row), keys(&rows[0]));
        assert_eq!(row["link"], format!("report-{:02}.html", at + 1));
        // The report beside the list is the report as it was written.
        let copy = out.join(row["link"].as_str().unwrap());
        assert_eq!(fs::read(copy).unwrap(), fs::read(page).unwrap());
        assert_eq!(row["project"], "repo");
        assert_eq!(row["names_left_out"], false);
        assert_eq!(row["scope"], json!(["src/app.py"]));
        assert_eq!(row["date"], run["started"]);
        assert_eq!(row["compared_with"], run["subject"]["compared_with"]);
        assert_eq!(row["lenses"], run["report"]["lenses"]);
        assert!(!row["lenses"].as_array().unwrap().is_empty());
        assert_eq!(row["reasons"], run["report"]["refusal"]["reasons"]);
    }
    let outcomes: Vec<&Value> = rows.iter().map(|row| &row["outcome"]).collect();
    assert_eq!(outcomes, ["Accepted", "Refused", "Refused"]);
    assert_eq!(rows[0]["reasons"], Value::Null);
    assert!(!rows[1]["reasons"].as_array().unwrap().is_empty());
    // The step that grows: its size is a row like any other, and says so.
    let size = &rows[2]["lenses"][0];
    assert_eq!(
        (&size["lens"], &size["moved"]),
        (&json!("Size"), &json!(false))
    );
}

/// What a report without names still says, and what it no longer says.
fn the_report_has_numbers_for_names(run: &Value, was: &Value) {
    for at in ["/faults/functions", "/report/functions"] {
        for row in run.pointer(at).unwrap().as_array().unwrap() {
            assert_eq!(row["file"], "file 1");
            let function = row["function"].as_str().unwrap();
            assert!(function.starts_with("function "), "{at}: {function}");
        }
    }
    let names = &run["report"]["graph"]["after"]["files"][0];
    assert_eq!(names["functions"], json!(["function 1", "function 2"]));
    assert_eq!(names["file"], "file 1");
    assert!(run["commands"].to_string().contains(LEFT));
    // What the tool says in its own words is as it was.
    for at in [
        "/next",
        "/stages/0/title",
        "/stages/0/what",
        "/outcome",
        "/exit_code",
    ] {
        assert_eq!(run.pointer(at), was.pointer(at), "{at}");
    }
    let measures = |run: &Value| -> Vec<Value> {
        let rows = run["report"]["lenses"].as_array().unwrap();
        let of = |row: &Value| json!([row["lens"], row["measure"], row["before"], row["after"]]);
        rows.iter().map(of).collect()
    };
    assert_eq!(measures(run), measures(was));
}

/// Requirement 8.4: no name of the named project in the list or in its
/// reports; the page around the data of a report is as it was written.
fn the_option_takes_every_name_out(repo: &Repo, pages: &[PathBuf]) {
    let out = place(repo, "list-without-names");
    let (code, said) = listed(&out, pages, &["--without-names-of", "repo"]);
    assert_eq!(code, Some(0), "{said}");
    let rows = rows(&out);
    let mut texts = vec![Value::from(rows.clone()).to_string()];
    for (row, page) in rows.iter().zip(pages) {
        assert_eq!(row["names_left_out"], true);
        assert_eq!(row["scope"], json!(["file 1"]));
        let copy = out.join(row["link"].as_str().unwrap());
        let (written, original) = (fs::read_to_string(&copy).unwrap(), fs::read_to_string(page));
        let around = |page: &str| page.split_once("const data = ").unwrap().0.len();
        assert_eq!(around(&written), around(&original.unwrap()));
        let run = inside(&copy);
        the_report_has_numbers_for_names(&run, &inside(page));
        assert_eq!(row["lenses"], run["report"]["lenses"]);
        texts.push(run.to_string());
    }
    for text in &texts {
        for name in NAMES {
            assert!(!text.contains(name), "{name} is still in {text}");
        }
    }
    // The reason of the refusal names the function by its number.
    let reason = rows[1]["reasons"][0].as_str().unwrap();
    let begins = "file 1: function 2: unnoticed faults rise from";
    assert!(reason.starts_with(begins), "{reason}");
}

/// Only the reports of the named project lose their names; a project that
/// no report is of is an error, and nothing is written.
fn only_the_named_project_loses_its_names(repo: &Repo, pages: &[PathBuf]) {
    let root = repo.dir.parent().unwrap();
    let other = root.join("other.html");
    let page = fs::read_to_string(&pages[0]).unwrap();
    assert!(page.contains("/repo\",\"branch\""));
    fs::write(
        &other,
        page.replace("/repo\",\"branch\"", "/other\",\"branch\""),
    )
    .unwrap();
    let pages = [other, pages[1].clone()];

    let out = root.join("two-projects");
    let (code, said) = listed(&out, &pages, &["--without-names-of", "other"]);
    assert_eq!(code, Some(0), "{said}");
    let rows = rows(&out);
    let said_of = |row: &Value| json!([row["project"], row["names_left_out"], row["scope"]]);
    assert_eq!(said_of(&rows[0]), json!(["other", true, ["file 1"]]));
    assert_eq!(said_of(&rows[1]), json!(["repo", false, ["src/app.py"]]));
    let copy = fs::read(out.join("report-02.html")).unwrap();
    assert_eq!(copy, fs::read(&pages[1]).unwrap());

    let out = root.join("nobody");
    let (code, said) = listed(&out, &pages, &["--without-names-of", "nobody"]);
    assert_eq!(code, Some(3), "{said}");
    assert!(said.contains("the reports are of: other, repo"), "{said}");
    assert!(!out.exists());
}

/// A file that is no report, and a directory that is not new, are bad
/// input; nothing is written.
fn what_cannot_be_listed_is_refused(repo: &Repo, pages: &[PathBuf]) {
    let root = repo.dir.parent().unwrap();
    let text = root.join("notes.html");
    fs::write(&text, "<p>not a report</p>").unwrap();
    let out = root.join("with-notes");
    let (code, said) = listed(&out, &[pages[0].clone(), text], &[]);
    assert_eq!(code, Some(3), "{said}");
    assert!(said.contains("notes.html"), "{said}");
    assert!(said.contains("it has no data inside"), "{said}");
    assert!(!out.exists());

    // The repository is a directory with something in it.
    let (code, said) = listed(&repo.dir, pages, &[]);
    assert_eq!(code, Some(3), "{said}");
    assert!(said.contains("is not a new or empty directory"), "{said}");
    assert!(!repo.dir.join("index.html").exists());
}

#[test]
fn a_list_of_three_fixture_reports() {
    let repo = Repo::new("page-list");
    let pages = three_reports(&repo);
    all_three_stand_alike(&repo, &pages);
    the_option_takes_every_name_out(&repo, &pages);
    only_the_named_project_loses_its_names(&repo, &pages);
    what_cannot_be_listed_is_refused(&repo, &pages);
}
