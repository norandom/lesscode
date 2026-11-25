//! What `step protect result` says and what it leaves: the standing of
//! every chosen function, every write-off with its reason, approver and
//! date, and `protection.json`, which a later reader finds without the run.
//!
//! The tool cannot know that a person wrote an approval. It prints every
//! write-off with who approved it and when, so that a false one is seen.

use super::declaration::Revisions;
use super::protect_numbers::{Joined, Row, WriteOff};
use super::protect_stage::{Chosen, Reported, Stage};
use serde::Serialize;

pub const RECEIPT: &str = "step-protection-v1";
pub const ENOUGH: &str = "protected_enough";
pub const NOT_ENOUGH: &str = "not_protected_enough";

const NOT_TESTABLE: &str = "not_testable_as_it_stands";
const ROSE: &str = "`step check` refused the branch because unnoticed faults rose: a test that noticed a fault at the base does not notice it now.";
const ADVISES: &str =
    "The stage advises: a reduction that begins without it is judged as any other.";
const NEXT: &str = "Next: declare the reduction of a chosen function as a step.";

/// `protection.json`: the result of a stage.
#[derive(Debug, Serialize)]
pub struct Protection {
    pub receipt: &'static str,
    pub spec: String,
    pub base: String,
    pub revision: String,
    /// The hash of the verdict the numbers are read from.
    pub verdict_sha256: String,
    /// `protected_enough` or `not_protected_enough`.
    pub result: &'static str,
    /// The verdict refused because unnoticed faults rose.
    pub unnoticed_rose: bool,
    pub functions: Vec<Row>,
    /// Functions with unnoticed faults that were not chosen, with the
    /// plan's numbers. They do not enter `result`.
    pub reported: Vec<Reported>,
    /// Faults the plan counted noticed that the base run of the check did
    /// not notice; they are the stage's like any other.
    pub from_the_base_run: Vec<Joined>,
}

impl Protection {
    /// Protected enough when nothing waits in any chosen function and the
    /// verdict did not refuse.
    pub fn new(
        stage: &Stage,
        revisions: &Revisions,
        verdict_sha256: String,
        functions: Vec<Row>,
        unnoticed_rose: bool,
    ) -> Protection {
        let waits = functions.iter().any(|row| row.waiting > 0);
        Protection {
            receipt: RECEIPT,
            spec: stage.spec.clone(),
            base: revisions.base.clone(),
            revision: revisions.step.clone(),
            verdict_sha256,
            result: match waits || unnoticed_rose {
                true => NOT_ENOUGH,
                false => ENOUGH,
            },
            unnoticed_rose,
            functions,
            reported: stage.reported.clone(),
            from_the_base_run: Vec::new(),
        }
    }

    pub fn enough(&self) -> bool {
        self.result == ENOUGH
    }
}

/// One function: its four numbers, then each fault that waits, is written
/// off, or has a write-off and is noticed.
fn of_row(row: &Row) -> Vec<String> {
    let mut lines = vec![format!(
        "  {}: {}  noticed before {}, noticed now {}, written off {}, waiting {}",
        row.file, row.function, row.noticed_before, row.noticed_now, row.written_off, row.waiting
    )];
    for w in &row.waiting_faults {
        let name = match w.faults.len() {
            0 => format!("{} at line {}", w.operator, w.line),
            1 => w.faults[0].clone(),
            _ => format!(
                "one of {} (the verdict cannot tell them apart: same line, same operator)",
                w.faults.join(", ")
            ),
        };
        lines.push(format!("    waiting: {name}: {}", w.why));
    }
    let written_off = row.write_offs.iter().map(|w| &w.fault);
    lines.extend(written_off.map(|fault| format!("    written off: {fault}")));
    let noticed = row.noticed_write_offs.iter().map(|w| &w.fault);
    lines.extend(
        noticed.map(|fault| {
            format!("    noticed: {fault} has a write-off, and a test notices it now")
        }),
    );
    lines
}

/// What the plan says a function reaches out to, in words.
fn reaches_out(chosen: &Chosen) -> String {
    let mut parts = Vec::new();
    if let Some(out) = &chosen.reaches_out {
        if !out.calls.is_empty() {
            parts.push(format!("calls {}", out.calls.join(", ")));
        }
        if !out.state_writes.is_empty() {
            parts.push(format!("writes state at {}", out.state_writes.join(", ")));
        }
    }
    match parts.is_empty() {
        true => "the plan does not say to what".to_owned(),
        false => parts.join("; "),
    }
}

/// One write-off with its reason, approver and date. For one that cannot
/// be tested as the code stands: what its function reaches out to.
fn of_write_off(w: &WriteOff, stage: &Stage) -> Vec<String> {
    let text = |v: &Option<String>| v.clone().unwrap_or_default();
    let signed = match w.approved() {
        true => format!(
            "Approved by {} on {}.",
            text(&w.approved_by),
            text(&w.approved_at)
        ),
        false => "Not approved.".to_owned(),
    };
    let reason = match text(&w.reason).trim() {
        "" => "No reason is given.".to_owned(),
        reason => reason.to_owned(),
    };
    let mut lines = vec![format!(
        "  {} {} ({}): {reason} {signed}",
        w.file, w.fault, w.kind
    )];
    let owns = |c: &&Chosen| c.file == w.file && c.faults.iter().any(|f| f.id == w.fault);
    if let Some(chosen) = stage
        .chosen
        .iter()
        .find(owns)
        .filter(|_| w.kind == NOT_TESTABLE)
    {
        lines.push(format!(
            "    {} reaches out: {}. A reduction has to keep the behaviour there by other means.",
            chosen.function,
            reaches_out(chosen)
        ));
    }
    lines
}

/// Everything the result says, in its order.
pub fn says(stage: &Stage, branch: &str, protection: &Protection, offs: &[WriteOff]) -> String {
    let mut lines = vec![format!("The stage {} on {branch}:", stage.spec)];
    let joined = protection.from_the_base_run.iter();
    lines.extend(joined.map(|j| {
        format!(
            "From the base run (the plan counted them noticed): {}: {} {}",
            j.file, j.function, j.fault
        )
    }));
    lines.extend(protection.functions.iter().flat_map(of_row));
    if !protection.reported.is_empty() {
        lines.push(format!(
            "Reported ({}), not chosen; they do not count in the result:",
            protection.reported.len()
        ));
        let one = |r: &Reported| {
            format!(
                "  {}: {}  unnoticed {} in the plan, not run again",
                r.file, r.function, r.unnoticed
            )
        };
        lines.extend(protection.reported.iter().map(one));
    }
    if !offs.is_empty() {
        lines.push(format!("Write-offs ({}):", offs.len()));
        lines.extend(offs.iter().flat_map(|w| of_write_off(w, stage)));
    }
    if protection.unnoticed_rose {
        lines.push(ROSE.to_owned());
    }
    let waiting: usize = protection.functions.iter().map(|r| r.waiting).sum();
    match protection.enough() {
        true => {
            lines.push("Protected enough: nothing waits in any chosen function.".to_owned());
            lines.push(NEXT.to_owned());
        }
        false => lines.push(format!("Not protected enough: {waiting} fault(s) wait.")),
    }
    lines.push(ADVISES.to_owned());
    lines.join("\n")
}
