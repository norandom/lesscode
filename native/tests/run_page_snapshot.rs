//! The snapshot the run page shows, built from what a run tells
//! (`.kiro/specs/run-dashboard`, task 1.1). Each test replays a small run
//! with stated moments and compares the snapshot with a written expectation.

use lesscode_core::adapters::driver::page_log::{Class, Classed, RunLog, StagePlan};
use lesscode_core::adapters::driver::page_snapshot::Snapshot;
use serde_json::{json, Value};

const STAGES: &[StagePlan] = &[
    StagePlan {
        id: "copy",
        title: "Copy the project",
        what: "Copies are made.",
    },
    StagePlan {
        id: "tests",
        title: "Run your tests on the unchanged code",
        what: "Each copy builds and runs the tests.",
    },
    StagePlan {
        id: "faults",
        title: "Plant faults",
        what: "One fault at a time.",
    },
    StagePlan {
        id: "close",
        title: "Check the copies",
        what: "The tests run once more.",
    },
];

/// Milliseconds since 1970 for a second of the test's own clock.
fn at(second: u64) -> u64 {
    1_795_443_600_000 + second * 1_000
}

fn fault(file: &str, function: &str, line: u64, class: Class) -> Classed {
    Classed {
        file: file.to_owned(),
        function: Some(function.to_owned()),
        line,
        operator: "negate".to_owned(),
        text: "a < b".to_owned(),
        class,
    }
}

fn begun() -> RunLog {
    let mut log = RunLog::new("step plan", "2.1.0", STAGES, at(0));
    log.set_copies("D:/work", 2);
    log.set_commands(vec![vec!["cargo".into(), "test".into()]], true);
    log
}

fn shown(snapshot: &Snapshot) -> Value {
    serde_json::to_value(snapshot).unwrap()
}

fn statuses(snapshot: &Value) -> Vec<&str> {
    let stages = snapshot["stages"].as_array().unwrap();
    stages
        .iter()
        .map(|s| s["status"].as_str().unwrap())
        .collect()
}

#[test]
fn a_run_that_has_only_begun_shows_every_stage_as_not_begun() {
    let first = shown(&begun().snapshot());
    assert_eq!(first["schema"], 1);
    assert_eq!(first["ended"], false);
    assert_eq!(first["started"], "2026-11-23T14:20:00.000Z");
    assert_eq!(statuses(&first), ["not begun"; 4]);
}

/// A short run after two of its three faults.
fn midway_through() -> RunLog {
    let mut log = begun();
    log.begin("copy", None, at(1));
    log.end("copy", at(3));
    log.begin("tests", None, at(3));
    log.unchanged_tests(1, Ok(Some(12)), at(60));
    log.unchanged_tests(2, Ok(Some(12)), at(61));
    log.end("tests", at(61));
    log.expect("src/a.rs", Some("one"), 2);
    log.expect("src/a.rs", Some("two"), 1);
    log.begin("faults", Some(3), at(61));
    log.classed(fault("src/a.rs", "one", 7, Class::Noticed), at(70));
    log.classed(fault("src/a.rs", "one", 9, Class::Unnoticed), at(80));
    log
}

#[test]
fn midway_one_stage_is_going_on_with_its_count_and_no_time_for_the_rest() {
    let midway = shown(&midway_through().snapshot());
    assert_eq!(
        midway["stages"][2],
        json!({
            "id": "faults", "title": "Plant faults", "what": "One fault at a time.", "status": "going on",
            "done": 2, "total": 3, "seconds": null, "seconds_so_far": 19.0, "seconds_left": null, "reason": null
        })
    );
    assert_eq!(midway["heartbeat"], "2026-11-23T14:21:20.000Z");
    assert_eq!(
        midway["faults"]["example"],
        json!({
            "file": "src/a.rs", "function": "one", "line": 7, "what_was_changed": "negate", "text": "a < b", "outcome": "noticed: a test failed"
        })
    );
}

#[test]
fn at_the_end_nothing_is_going_on_and_the_faults_are_counted_per_function() {
    let mut log = midway_through();
    log.classed(fault("src/a.rs", "two", 20, Class::Unnoticed), at(90));
    log.end("faults", at(90));
    log.begin("close", None, at(90));
    log.finish(
        "The plan is written",
        "1 of 2 functions is unprotected.",
        0,
        vec!["Write tests first.".into()],
        at(95),
    );

    let last = shown(&log.snapshot());
    assert_eq!(last["ended"], true);
    assert_eq!(last["exit_code"], 0);
    assert_eq!(last["seconds"], 95.0);
    assert_eq!(
        last["tests"],
        json!({"command_given": true, "passed_unchanged": true, "tests_run": 12, "reason": null})
    );
    assert_eq!(
        last["copies"],
        json!({"directory": "D:/work", "planned": 2, "working": 2, "dropped": [], "bytes": null})
    );
    assert_eq!(statuses(&last), ["ended"; 4]);
    assert_eq!(last["stages"][0]["seconds"], 2.0);
    assert_eq!(last["faults"]["total"], 3);
    assert_eq!(
        (
            last["faults"]["noticed"].clone(),
            last["faults"]["unnoticed"].clone()
        ),
        (json!(1), json!(2))
    );
    assert_eq!(
        last["faults"]["functions"],
        json!([
            {"file": "src/a.rs", "function": "one", "noticed": 1, "unnoticed": 1, "not_built": 0, "timed_out": 0, "not_judged": 0, "left": 0, "unprotected": false, "reaches_out": []},
            {"file": "src/a.rs", "function": "two", "noticed": 0, "unnoticed": 1, "not_built": 0, "timed_out": 0, "not_judged": 0, "left": 0, "unprotected": true, "reaches_out": []}
        ])
    );
    assert_eq!(last["faults"]["files"][0]["unprotected"], false);
    assert_eq!(last["next"], json!(["Write tests first."]));
}

#[test]
fn the_time_for_the_rest_is_given_only_after_five_items() {
    let mut log = begun();
    log.expect("src/a.rs", None, 10);
    log.begin("faults", Some(10), at(0));
    for done in 1..=4 {
        log.classed(
            fault("src/a.rs", "one", done, Class::Noticed),
            at(done * 10),
        );
        assert_eq!(
            shown(&log.snapshot())["stages"][2]["seconds_left"],
            Value::Null,
            "after {done}"
        );
    }
    log.classed(fault("src/a.rs", "one", 5, Class::Noticed), at(50));
    // Five done in 50 seconds, five left.
    assert_eq!(shown(&log.snapshot())["stages"][2]["seconds_left"], 50.0);
}

#[test]
fn a_copy_that_drops_out_is_named_and_the_run_goes_on() {
    let mut log = begun();
    log.begin("tests", None, at(0));
    log.unchanged_tests(1, Err("the test step failed, exit 100".into()), at(30));
    log.unchanged_tests(2, Ok(Some(12)), at(31));
    log.end("tests", at(31));
    let seen = shown(&log.snapshot());
    assert_eq!(seen["copies"]["working"], 1);
    assert_eq!(
        seen["copies"]["dropped"],
        json!([
            {"copy": 1, "at": "2026-11-23T14:20:30.000Z", "reason": "the test step failed, exit 100"}
        ])
    );
    assert_eq!(
        seen["tests"]["passed_unchanged"], true,
        "one copy is enough to go on"
    );
    assert_eq!(seen["ended"], false);
}

#[test]
fn a_void_run_says_that_nothing_was_judged() {
    let mut log = begun();
    log.begin("tests", None, at(0));
    log.unchanged_tests(1, Err("exit 100".into()), at(30));
    log.unchanged_tests(2, Err("exit 100".into()), at(31));
    log.finish(
        "Could not judge",
        "Your tests do not pass on the unchanged code.",
        3,
        Vec::new(),
        at(32),
    );
    let seen = shown(&log.snapshot());
    assert_eq!(seen["tests"]["passed_unchanged"], false);
    assert_eq!(seen["tests"]["reason"], "exit 100");
    assert_eq!(seen["copies"]["working"], 0);
    assert_eq!(seen["exit_code"], 3);
    let statuses: Vec<&str> = seen["stages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["status"].as_str().unwrap())
        .collect();
    assert_eq!(statuses, ["left out", "failed", "left out", "left out"]);
    assert_eq!(
        seen["stages"][1]["reason"],
        "Your tests do not pass on the unchanged code."
    );
    assert_eq!(seen["faults"]["done"], 0);
}

#[test]
fn a_run_without_a_test_command_says_so_before_any_stage() {
    let mut log = RunLog::new("step plan", "2.1.0", STAGES, at(0));
    log.set_commands(Vec::new(), false);
    log.finish(
        "Could not judge",
        "No test command was given.",
        3,
        Vec::new(),
        at(0),
    );
    let seen = shown(&log.snapshot());
    assert_eq!(seen["tests"]["command_given"], false);
    assert_eq!(seen["tests"]["passed_unchanged"], Value::Null);
    let statuses: Vec<&str> = seen["stages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["status"].as_str().unwrap())
        .collect();
    assert_eq!(statuses, ["left out"; 4]);
}

#[test]
fn every_change_and_every_sign_of_life_makes_another_snapshot() {
    let mut log = begun();
    let before = log.snapshot();
    log.beat(at(4));
    let after = log.snapshot();
    assert!(after.sequence > before.sequence);
    assert_eq!(after.heartbeat, "2026-11-23T14:20:04.000Z");
    assert_eq!(after.seconds, 4.0);
}
