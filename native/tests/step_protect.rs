//! `step protect begin` through the binary (brownfield-test-stage, group 1).
//! A real plan of the toy repository is made first; the command then only
//! reads. Every refusal is exit 3 with its sentence, and writes nothing.

mod step_support;

use serde_json::{json, Value};
use std::fs;
use step_support::planned::Planned;
use step_support::toy;

#[test]
fn named_functions_of_the_plan_are_chosen_and_nothing_is_asked() {
    let planned = Planned::new("protect-chosen", "python", "src/app.py");
    let before = planned.repo.state();
    // Standard input is empty and closed: the command asks nothing.
    let (code, said) = planned.begin(&["src/app.py:helper", "src/app.py:helper"]);
    assert_eq!(code, Some(0), "{said}");
    assert!(said.contains("Chosen (1):"), "{said}");
    assert!(!said.contains("src/app.py: step"), "{said}");

    // What is new in the repository is the two files of the stage.
    let after = planned.repo.state();
    let new: Vec<&str> = after
        .iter()
        .filter(|file| !before.contains(file))
        .map(|file| file.0.as_str())
        .collect();
    let dir = ".kiro/specs/protect-toy";
    assert_eq!(
        new,
        [format!("{dir}/stage.json"), format!("{dir}/step.json")]
    );
    assert_eq!(after.len(), before.len() + 2, "a file was removed");
}

#[test]
fn the_stage_file_holds_the_plan_the_base_and_each_unnoticed_fault() {
    let planned = Planned::new("protect-stage", "python", "src/app.py");
    let (code, said) = planned.begin(&["src/app.py:helper"]);
    assert_eq!(code, Some(0), "{said}");
    let dir = planned.repo.dir.join(".kiro/specs/protect-toy");
    let mut stage = planned.repo.json(&dir.join("stage.json"));

    // What differs from run to run is checked for its form and then set.
    assert_eq!(stage["base"], planned.repo.git(&["rev-parse", "HEAD"]));
    assert_eq!(stage["plan"]["sha256"].as_str().unwrap().len(), 64);
    assert!(stage["plan"]["file"]
        .as_str()
        .unwrap()
        .ends_with("/plan.json"));
    assert_eq!(stage["test"][0], toy());
    // A fault of the toy can take less than a tenth of a second.
    assert!(stage["to_run_again"]["seconds"].as_f64().unwrap() >= 0.0);
    for key in ["base", "plan", "test"] {
        stage[key] = Value::Null;
    }
    stage["to_run_again"]["seconds"] = json!(0);
    for fault in stage["chosen"][0]["faults"].as_array_mut().unwrap() {
        fault["seconds"] = json!(0);
    }
    let expected = json!({
        "receipt": "step-stage-v1", "spec": "protect-toy",
        "base": null, "plan": null, "test": null,
        "chosen": [{
            "file": "src/app.py", "function": "helper",
            "noticed": 0, "unnoticed": 1, "other": 0,
            "reaches_out": {"calls": [], "state_writes": ["line 9: state.count = b"]},
            "faults": [{"id": "drop_statement@9:4", "line": 9, "column": 4,
                        "operator": "drop_statement", "text": "state.count = b", "seconds": 0}],
        }],
        "left_alone": [],
        "left_out": [],
        "to_run_again": {"faults": 1, "seconds": 0},
    });
    assert_eq!(stage, expected, "{stage:#}");
}

#[test]
fn step_check_accepts_the_written_declaration_on_a_branch_without_changes() {
    let planned = Planned::new("protect-declared", "python", "src/app.py");
    let (code, said) = planned.begin_with(&["src/app.py:helper"], &["--branch", "protect/helper"]);
    assert_eq!(code, Some(0), "{said}");
    let repo = &planned.repo;
    let base = repo.git(&["rev-parse", "HEAD"]);
    let step = repo.json(&repo.dir.join(".kiro/specs/protect-toy/step.json"));
    assert_eq!(step["base"], base);
    assert_eq!(step["branch"], "protect/helper");
    assert_eq!(step["scope"], json!(["src/app.py"]));
    assert_eq!(step["focus"], json!({"src/app.py": ["helper"]}));
    assert_eq!(step["max_token_growth"], 0);
    assert_eq!(step["test_paths"], json!(["tests/"]));
    assert_eq!(
        step["run"],
        json!({"timeout_floor_s": 10, "timeout_ceiling_s": 60, "workers": 2})
    );
    assert_eq!(
        step["outputs"]["verdict"],
        ".kiro/specs/protect-toy/verdict.json"
    );

    repo.git(&["checkout", "-q", "-b", "protect/helper"]);
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", "begin the stage"]);
    let (code, out, err) = repo.step("check", "protect-toy", &[]);
    assert_eq!(code, Some(0), "{out}\n{err}");
}

#[test]
fn a_function_the_plan_does_not_have_is_refused_with_the_ones_it_has() {
    let planned = Planned::new("protect-missing", "python", "src/app.py");
    planned.refused(
        &["src/app.py:nothing"],
        &[
            "src/app.py:nothing is not a function of the plan",
            "In that file the plan has",
            "  src/app.py:helper",
            "  src/app.py:step",
        ],
    );
    planned.refused(
        &["src/other.py:helper"],
        &["src/other.py:helper is not a function of the plan. The plan has:"],
    );
    planned.refused(&["helper"], &["write it as <file>:<name>"]);
}

#[test]
fn a_plan_without_a_result_is_refused() {
    let planned = Planned::new("protect-no-result", "python", "src/app.py");
    Planned::edit(
        &planned.run,
        "void",
        json!("the tests fail before any fault"),
    );
    planned.refused(
        &["src/app.py:helper"],
        &[
            "was void (the tests fail before any fault)",
            "A plan with a result is needed first",
        ],
    );
    Planned::edit(&planned.run, "void", Value::Null);
    Planned::edit(&planned.plan, "incomplete", json!(true));
    planned.refused(
        &["src/app.py:helper"],
        &["is incomplete", "A plan with a result is needed first"],
    );
}

#[test]
fn a_file_changed_since_the_plan_is_refused() {
    let planned = Planned::new("protect-changed", "python", "src/app.py");
    let changed = format!("{}\n# later\n", step_support::APP);
    planned
        .repo
        .commit(&[("src/app.py", &changed)], "a change after the plan");
    planned.refused(
        &["src/app.py:helper"],
        &[
            "src/app.py has changed since the plan was made",
            "Make a new plan",
        ],
    );
}

#[test]
fn a_stage_that_was_begun_and_a_test_path_without_a_file_are_refused() {
    let planned = Planned::new("protect-places", "python", "src/app.py");
    planned.configure("python", "src/app.py", &["tests/", "checks/"]);
    planned.refused(
        &["src/app.py:helper"],
        &["The test path checks/ holds no file"],
    );
    planned.configure("python", "src/app.py", &[]);
    planned.refused(&["src/app.py:helper"], &["names no test path"]);

    planned.configure("python", "src/app.py", &["tests/"]);
    planned.repo.commit(
        &[(".kiro/specs/protect-toy/stage.json", "{}")],
        "an earlier stage",
    );
    planned.refused(&["src/app.py:helper"], &["already holds a stage.json"]);
}

#[test]
fn a_function_in_code_not_built_here_is_left_out_and_named() {
    let planned = Planned::new("protect-systems", "rust", "src/lib.rs");
    let elsewhere = match cfg!(windows) {
        true => "src/lib.rs:only_linux",
        false => "src/lib.rs:only_windows",
    };
    planned.refused(
        &[elsewhere],
        &["Every named function lies wholly in code that is not built"],
    );

    let (code, said) = planned.begin(&[elsewhere, "src/lib.rs:everywhere"]);
    assert_eq!(code, Some(0), "{said}");
    assert!(
        said.contains("  src/lib.rs: everywhere  (noticed 0,"),
        "{said}"
    );
    let left_out = format!(
        "Left out (1): wholly in code that is not built on this system. Nothing is known about them here.\n  {}\n",
        elsewhere.replace(':', ": ")
    );
    assert!(said.contains(&left_out), "{said}");

    let dir = planned.repo.dir.join(".kiro/specs/protect-toy");
    let stage = planned.repo.json(&dir.join("stage.json"));
    let (file, function) = elsewhere.split_once(':').unwrap();
    assert_eq!(
        stage["left_out"],
        json!([{"file": file, "function": function, "why": "not built on this system"}])
    );
    assert_eq!(stage["chosen"].as_array().unwrap().len(), 1);
    let step = planned.repo.json(&dir.join("step.json"));
    assert_eq!(step["focus"], json!({"src/lib.rs": ["everywhere"]}));

    // The stand-in test reads the text of `only_windows` and no other
    // function: an unprotected function that is not named is left alone.
    fs::remove_dir_all(&dir).unwrap();
    let (code, said) = planned.begin(&["src/lib.rs:everywhere"]);
    assert_eq!(code, Some(0), "{said}");
    let stage = planned.repo.json(&dir.join("stage.json"));
    assert_eq!(
        stage["left_alone"],
        json!([{"file": "src/lib.rs", "function": "only_linux"}])
    );
    let left_alone = "Left alone (1): unprotected in the plan and not named. Nothing has to be done about them.\n  src/lib.rs: only_linux\n";
    assert!(said.contains(left_alone), "{said}");
}

#[test]
fn begin_says_what_is_chosen_and_what_running_the_faults_again_costs() {
    let planned = Planned::new("protect-says", "python", "src/app.py");
    let copy = planned.run.parent().unwrap().join("w1");
    let (code, said) = planned.begin(&["src/app.py:helper"]);
    assert_eq!(code, Some(0), "{said}");

    // The numbers are the plan's.
    let plan = planned.repo.json(&planned.plan);
    let rows = plan["functions"].as_array().unwrap();
    let helper = rows.iter().find(|r| r["function"] == "helper").unwrap();
    let count = |key: &str| helper[key].as_u64().unwrap();
    let (noticed, unnoticed, other) = (count("noticed"), count("unnoticed"), count("other"));
    let base = planned.repo.git(&["rev-parse", "HEAD"]);
    let lines = [
        format!(
            "The stage protect-toy begins from {}, on the branch protect/protect-toy.",
            &base[..12]
        ),
        "Chosen (1):".to_owned(),
        format!("  src/app.py: helper  (noticed {noticed}, unnoticed {unnoticed}, other {other})"),
        "    drop_statement@9:4  line 9: state.count = b".to_owned(),
        "    it writes state at: line 9: state.count = b".to_owned(),
        format!(
            "Faults to run again: {}. The base run and the check each run them, and no others.",
            noticed + unnoticed + other
        ),
        "The base run comes first: the first `step pin` of the stage runs it, once, before any test is judged.".to_owned(),
        "One copy of the plan takes ".to_owned(),
        "w1). The base run, the pins and the check build copies of that size".to_owned(),
        "Written: step.json and stage.json in ".to_owned(),
    ];
    let mut rest = said.as_str();
    for line in &lines {
        let at = rest.find(line.as_str());
        let at =
            at.unwrap_or_else(|| panic!("{line:?} is missing, or out of its order, in:\n{said}"));
        rest = &rest[at + line.len()..];
    }
    // The time is the plan's: the seconds of the faults, and the copies it ran.
    let run = planned.repo.json(&planned.run);
    let copies = run["workers"].as_array().unwrap().len();
    let took = format!(
        "In the plan they took {} s one after another. With {copies} copies at once",
        {
            let of_helper = run["mutants"].as_array().unwrap().iter();
            let seconds: f64 = of_helper
                .filter(|m| m["site"]["start_line"].as_u64() >= Some(8))
                .map(|m| m["seconds"].as_f64().unwrap())
                .sum();
            ((seconds * 10.0).round() / 10.0).round()
        }
    );
    assert!(said.contains(&took), "{took:?} is not in:\n{said}");
    assert!(
        !said.contains("Left alone") && !said.contains("Left out"),
        "{said}"
    );

    // With the plan's copies removed, the size of one is not known, and said.
    fs::remove_dir_all(&copy).unwrap();
    fs::remove_dir_all(planned.repo.dir.join(".kiro/specs/protect-toy")).unwrap();
    let (code, said) = planned.begin(&["src/app.py:helper"]);
    assert_eq!(code, Some(0), "{said}");
    let gone = "The copies of the plan are gone, so what one copy takes on disk is not known here.";
    assert!(said.contains(gone), "{said}");
}
