//! The four numbers of a stage where the toy repository cannot show them:
//! two faults the verdict cannot tell apart, a fault the stage did not
//! begin with, and a write-off for a fault the stage does not have
//! (brownfield-test-stage, task 2.2; the owner's decision of 2026-11-24).

use lesscode_core::step::protect_numbers::{from_the_base_run, numbers, write_offs, Row, WriteOff};
use lesscode_core::step::protect_stage::Stage;
use serde_json::{json, Value};

/// A stage with one function whose two unnoticed faults share line and operator.
fn stage() -> Stage {
    let fault = |column: usize| {
        json!({"id": format!("negate@5:{column}"), "line": 5, "column": column,
               "operator": "negate", "text": "a && b", "seconds": 0.1})
    };
    // The form of a stage begun before `reported`: `left_alone` is read past.
    serde_json::from_value(json!({
        "receipt": "step-stage-v1", "spec": "s", "base": "b",
        "plan": {"file": "plan.json", "sha256": ""}, "test": [],
        "chosen": [{"file": "src/a.rs", "function": "decide", "noticed": 1, "unnoticed": 2,
                    "other": 0, "faults": [fault(3), fault(9)]}],
        "left_alone": [], "left_out": [], "to_run_again": {"faults": 3, "seconds": 0.3},
    }))
    .unwrap()
}

fn verdict(unnoticed_after: &[&str]) -> Value {
    json!({"functions": [{
        "file": "src/a.rs", "function": "decide",
        "before": {"noticed": 1, "unnoticed": 2, "other": 0},
        "after": {"noticed": 2, "unnoticed": 1, "other": 0},
        "unnoticed_after": unnoticed_after,
    }]})
}

/// Approved write-offs for the faults at these columns of line 5.
fn approved(columns: &[usize]) -> Vec<WriteOff> {
    let entry = |column: &usize| {
        json!({"file": "src/a.rs", "function": "decide", "fault": format!("negate@5:{column}"),
               "kind": "nothing_a_caller_sees", "reason": "r",
               "approved_by": "the owner", "approved_at": "2026-11-24"})
    };
    let entries: Vec<Value> = columns.iter().map(entry).collect();
    let file = json!({"schema": 1, "write_offs": entries});
    write_offs(file.to_string().as_bytes(), &stage()).unwrap()
}

fn row(unnoticed_after: &[&str], offs: &[WriteOff]) -> Row {
    let rows = numbers(&stage(), &verdict(unnoticed_after), offs).unwrap();
    rows.into_iter().next().unwrap()
}

#[test]
fn two_faults_of_one_line_and_operator_wait_or_are_written_off_together() {
    // One of the two is still unnoticed; the verdict does not say which.
    let waits = row(&["#2 line 5 negate"], &[]);
    assert_eq!((waits.noticed_before, waits.noticed_now), (1, 2));
    assert_eq!((waits.written_off, waits.waiting), (0, 1));
    assert_eq!(waits.waiting_faults[0].faults, ["negate@5:3", "negate@5:9"]);

    // A write-off for one of them does not say it is the unnoticed one.
    let one = row(&["#2 line 5 negate"], &approved(&[3]));
    assert_eq!((one.written_off, one.waiting), (0, 1));
    assert_eq!(one.waiting_faults[0].why, "its write-off is not approved");

    let both = row(&["#2 line 5 negate"], &approved(&[3, 9]));
    assert_eq!((both.written_off, both.waiting), (1, 0));
    assert_eq!(both.write_offs.len(), 2);
    assert!(both.noticed_write_offs.is_empty());
}

#[test]
fn a_fault_the_stage_did_not_begin_with_waits_and_cannot_be_written_off() {
    let rose = row(&["#7 line 8 drop_statement"], &approved(&[3, 9]));
    assert_eq!((rose.written_off, rose.waiting), (0, 1));
    let waiting = &rose.waiting_faults[0];
    assert!(waiting.faults.is_empty());
    assert_eq!(
        (waiting.line, waiting.operator.as_str()),
        (8, "drop_statement")
    );
    assert_eq!(
        waiting.why,
        "it is not among the unnoticed faults the stage began with"
    );
    // Both write-offs are for faults a test notices now.
    assert_eq!(rose.noticed_write_offs.len(), 2);
}

#[test]
fn a_write_off_for_a_fault_the_stage_does_not_have_is_bad_input() {
    let file = json!({"write_offs": [{"file": "src/a.rs", "fault": "negate@6:1",
                                      "kind": "nothing_a_caller_sees"}]});
    let refused = write_offs(file.to_string().as_bytes(), &stage()).unwrap_err();
    assert!(refused.contains("the stage has no unnoticed fault negate@6:1 in src/a.rs"));
}

/// A verdict that names what the base run did not notice (from 2026-11-25).
fn verdict_with_base(before: &[&str], after: &[&str]) -> Value {
    let mut verdict = verdict(after);
    verdict["functions"][0]["unnoticed_before"] = json!(before);
    verdict
}

/// Added 2026-11-25 (task 6.5, R189): the plan counted a fault noticed that
/// the base run of the check did not notice.
#[test]
fn a_fault_the_base_run_also_missed_joins_the_stage_and_can_be_written_off() {
    let before = [
        "#2 line 5 negate",
        "#7 line 8 drop_statement",
        "#9 line 12 swap:<-><=@12:4",
    ];
    let after = ["#7 line 8 drop_statement", "#9 line 12 swap:<-><=@12:4"];
    let verdict = verdict_with_base(&before, &after);
    let mut stage = stage();
    let joined = from_the_base_run(&mut stage, &verdict);
    let ids: Vec<&str> = joined.iter().map(|j| j.fault.as_str()).collect();
    assert_eq!(ids, ["drop_statement@8", "swap:<-><=@12:4"]);
    assert!(joined
        .iter()
        .all(|j| j.file == "src/a.rs" && j.function == "decide"));

    // They wait, as any unnoticed fault without a proposal does.
    let waits = &numbers(&stage, &verdict, &[]).unwrap()[0];
    assert_eq!((waits.written_off, waits.waiting), (0, 2));
    assert!(waits
        .waiting_faults
        .iter()
        .all(|w| w.why == "no test notices it and no write-off is proposed"));

    // And a write-off by their ids is taken.
    let entry = |fault: &str| {
        json!({"file": "src/a.rs", "function": "decide", "fault": fault,
               "kind": "nothing_a_caller_sees", "reason": "r",
               "approved_by": "the owner", "approved_at": "2026-11-25"})
    };
    let file = json!({"write_offs": [entry("drop_statement@8"), entry("swap:<-><=@12:4")]});
    let offs = write_offs(file.to_string().as_bytes(), &stage).unwrap();
    let done = &numbers(&stage, &verdict, &offs).unwrap()[0];
    assert_eq!((done.written_off, done.waiting), (2, 0));
}

#[test]
fn a_fault_missed_only_after_the_step_does_not_join_the_stage() {
    let checked = verdict_with_base(&["#2 line 5 negate"], &["#7 line 8 drop_statement"]);
    let mut stage = stage();
    assert!(from_the_base_run(&mut stage, &checked).is_empty());
    let rose = &numbers(&stage, &checked, &[]).unwrap()[0];
    assert_eq!(
        rose.waiting_faults[0].why,
        "it is not among the unnoticed faults the stage began with"
    );
    // A verdict without `unnoticed_before` adds nothing either.
    assert!(from_the_base_run(&mut stage, &verdict(&["#7 line 8 drop_statement"])).is_empty());
}

#[test]
fn a_function_the_verdict_has_no_numbers_for_is_named() {
    let none = numbers(&stage(), &json!({"functions": []}), &[]).unwrap_err();
    assert_eq!(none, "src/a.rs: decide");
}
