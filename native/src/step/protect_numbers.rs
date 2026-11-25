//! The four numbers of a stage of `step protect`, per chosen function:
//! faults noticed before, noticed now, written off, and waiting. They are
//! read off the verdict of `step check`, `stage.json` and `write-offs.json`.
//!
//! The verdict names an unnoticed fault by its line and operator, and
//! `stage.json` by `<operator>@<line>:<column>`. The two are matched by
//! file, line and operator. Where two faults of a function share a line and
//! an operator the verdict cannot tell them apart: they are then written
//! off together or wait together, and the result says so.

use super::approvals::{Refusal, Signed};
use super::protect_stage::{Chosen, Fault, Stage};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The two kinds a write-off can have.
pub const KINDS: [&str; 2] = ["nothing_a_caller_sees", "not_testable_as_it_stands"];

const NOT_PROPOSED: &str = "no test notices it and no write-off is proposed";
const NOT_APPROVED: &str = "its write-off is not approved";
pub const THE_OLD_WAY: &str =
    "approved the old way, which no longer counts: approve it in a signed commit";
const NOT_OF_THE_STAGE: &str = "it is not among the unnoticed faults the stage began with";

/// One entry of `write-offs.json`: a fault no test is asked to notice.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct WriteOff {
    pub file: String,
    #[serde(default)]
    pub function: String,
    /// The id of the fault in `stage.json`.
    pub fault: String,
    pub kind: String,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub proposed_by: Option<String>,
    /// The mark of an approval; it counts only as `approval` says.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved: Option<bool>,
    /// The fields of approvals before signed commits: still read, so older
    /// files parse; they decide nothing.
    #[serde(default)]
    pub approved_by: Option<String>,
    #[serde(default)]
    pub approved_at: Option<String>,
    /// The judged approval, filled in by the judge and never read from the file.
    #[serde(default, skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub approval: Option<Signed>,
    /// Why the marked approval does not count, filled in by the judge.
    #[serde(default, skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub refusal: Option<Refusal>,
}

fn given(v: &Option<String>) -> bool {
    v.as_deref().is_some_and(|t| !t.trim().is_empty())
}

impl WriteOff {
    /// Whether it is marked `"approved": true`.
    pub fn marked(&self) -> bool {
        self.approved == Some(true)
    }

    /// It counts with a reason and an approval the judge counted.
    pub fn approved(&self) -> bool {
        self.approval.is_some() && given(&self.reason)
    }

    /// Approved only by the fields of before signed commits.
    pub fn the_old_way(&self) -> bool {
        !self.marked() && (given(&self.approved_by) || given(&self.approved_at))
    }
}

#[derive(Deserialize)]
struct WriteOffs {
    #[serde(default)]
    write_offs: Vec<WriteOff>,
}

/// The entries of `write-offs.json`. A kind that is not one of the two, and
/// a fault the stage does not have, are bad input.
pub fn write_offs(bytes: &[u8], stage: &Stage) -> Result<Vec<WriteOff>, String> {
    let file: WriteOffs = serde_json::from_slice(bytes)
        .map_err(|e| format!("write-offs.json is not a list of write-offs: {e}"))?;
    for w in &file.write_offs {
        if !KINDS.contains(&w.kind.as_str()) {
            return Err(format!(
                "write-offs.json: the kind `{}` of the write-off for {} {} is not one of: {}.",
                w.kind,
                w.file,
                w.fault,
                KINDS.join(", ")
            ));
        }
        let has = |c: &Chosen| c.file == w.file && c.faults.iter().any(|f| f.id == w.fault);
        if !stage.chosen.iter().any(has) {
            return Err(format!(
                "write-offs.json: the stage has no unnoticed fault {} in {}.",
                w.fault, w.file
            ));
        }
    }
    Ok(file.write_offs)
}

/// A fault that is still unnoticed and not written off.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Waiting {
    /// The id of the fault in `stage.json`: several where the verdict cannot
    /// tell them apart, none where the stage does not have the fault.
    pub faults: Vec<String>,
    pub line: usize,
    pub operator: String,
    pub why: String,
    /// The approval of its write-off that does not count, where it was marked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<Refusal>,
}

/// Why a fault of the stage waits, from the write-offs of the faults it
/// can be: a refused approval first, then one approved the old way.
fn why_waiting(entries: &[&WriteOff]) -> (String, Option<Refusal>) {
    let refused = entries.iter().find_map(|w| w.refusal.clone());
    let why = match &refused {
        Some(refusal) => refusal.words(),
        None if entries.iter().any(|w| w.the_old_way()) => THE_OLD_WAY.to_owned(),
        None if entries.is_empty() => NOT_PROPOSED.to_owned(),
        None => NOT_APPROVED.to_owned(),
    };
    (why, refused)
}

/// The standing of one chosen function.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Row {
    pub file: String,
    pub function: String,
    pub noticed_before: u64,
    pub noticed_now: u64,
    pub written_off: usize,
    pub waiting: usize,
    /// The approved write-offs of faults that are still unnoticed.
    pub write_offs: Vec<WriteOff>,
    pub waiting_faults: Vec<Waiting>,
    /// Write-offs whose fault a test notices now.
    pub noticed_write_offs: Vec<WriteOff>,
}

/// The line and the operator of a verdict's `#N line L <operator>`.
fn place(label: &str) -> Option<(usize, &str)> {
    let (_, rest) = label.split_once(" line ")?;
    let (line, operator) = rest.split_once(' ')?;
    Some((line.parse().ok()?, operator))
}

/// Put one fault the verdict calls unnoticed where it stands: written off
/// when every fault of the stage it can be has an approved write-off.
fn stand(row: &mut Row, label: &str, faults: &[Fault], offs: &[WriteOff]) -> Vec<String> {
    let (line, operator) = place(label).unwrap_or((0, label));
    let same = |f: &&Fault| f.line == line && f.operator == operator;
    let ids: Vec<String> = faults.iter().filter(same).map(|f| f.id.clone()).collect();
    let file = row.file.clone();
    let of = |id: &String| {
        let (file, id) = (file.clone(), id.clone());
        offs.iter().filter(move |w| w.file == file && w.fault == id)
    };
    let approved: Vec<WriteOff> = ids
        .iter()
        .filter_map(|id| of(id).find(|w| w.approved()).cloned())
        .collect();
    if !ids.is_empty() && approved.len() == ids.len() {
        row.written_off += 1;
        let new = approved.into_iter().filter(|w| !row.write_offs.contains(w));
        let new: Vec<WriteOff> = new.collect();
        row.write_offs.extend(new);
        return ids;
    }
    let (why, refusal) = match ids.is_empty() {
        true => (NOT_OF_THE_STAGE.to_owned(), None),
        false => why_waiting(&ids.iter().flat_map(of).collect::<Vec<_>>()),
    };
    row.waiting += 1;
    row.waiting_faults.push(Waiting {
        faults: ids.clone(),
        line,
        operator: operator.to_owned(),
        why,
        refusal,
    });
    ids
}

/// A fault the plan counted noticed that the base run of the check did not
/// notice: it joined the stage from the verdict.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Joined {
    pub file: String,
    pub function: String,
    pub fault: String,
}

/// The labels of a verdict row's list `key`.
fn labels<'v>(found: &'v Value, key: &str) -> Vec<&'v str> {
    let list = found[key].as_array().map(Vec::as_slice).unwrap_or(&[]);
    list.iter().filter_map(Value::as_str).collect()
}

/// Every fault the verdict calls unnoticed before and after the step, in a
/// chosen function, that `stage.json` lacks, joins the stage. Its id is the
/// operator where that names its place, else `<operator>@<line>`: the
/// verdict gives no column. A verdict without `unnoticed_before` adds none.
pub fn from_the_base_run(stage: &mut Stage, verdict: &Value) -> Vec<Joined> {
    let rows = verdict["functions"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let mut joined = Vec::new();
    for chosen in &mut stage.chosen {
        let is = |r: &&Value| {
            r["file"] == chosen.file.as_str() && r["function"] == chosen.function.as_str()
        };
        let Some(found) = rows.iter().find(is) else {
            continue;
        };
        let before = labels(found, "unnoticed_before");
        for label in labels(found, "unnoticed_after")
            .into_iter()
            .filter(|l| before.contains(l))
        {
            let Some((line, operator)) = place(label) else {
                continue;
            };
            let known = |f: &Fault| f.line == line && f.operator == operator;
            if chosen.faults.iter().any(known) {
                continue;
            }
            let id = match operator.contains('@') {
                true => operator.to_owned(),
                false => format!("{operator}@{line}"),
            };
            chosen.faults.push(Fault {
                id: id.clone(),
                line,
                column: 0,
                operator: operator.to_owned(),
                text: String::new(),
                seconds: 0.0,
            });
            joined.push(Joined {
                file: chosen.file.clone(),
                function: chosen.function.clone(),
                fault: id,
            });
        }
    }
    joined
}

/// The numbers of one chosen function, if the verdict has them.
fn row(chosen: &Chosen, verdict: &Value, offs: &[WriteOff]) -> Option<Row> {
    let rows = verdict["functions"].as_array()?;
    let is =
        |r: &&Value| r["file"] == chosen.file.as_str() && r["function"] == chosen.function.as_str();
    let found = rows.iter().find(is)?;
    let mut row = Row {
        file: chosen.file.clone(),
        function: chosen.function.clone(),
        noticed_before: found["before"]["noticed"].as_u64()?,
        noticed_now: found["after"]["noticed"].as_u64()?,
        written_off: 0,
        waiting: 0,
        write_offs: Vec::new(),
        waiting_faults: Vec::new(),
        noticed_write_offs: Vec::new(),
    };
    let labels = found["unnoticed_after"].as_array()?;
    let mut unnoticed = Vec::new();
    for label in labels.iter().filter_map(Value::as_str) {
        unnoticed.extend(stand(&mut row, label, &chosen.faults, offs));
    }
    let noticed = |w: &&WriteOff| {
        let here = w.file == chosen.file && chosen.faults.iter().any(|f| f.id == w.fault);
        here && !unnoticed.contains(&w.fault)
    };
    row.noticed_write_offs = offs.iter().filter(noticed).cloned().collect();
    Some(row)
}

/// The numbers of every chosen function; else the function the verdict has
/// no numbers for, before and after.
pub fn numbers(stage: &Stage, verdict: &Value, offs: &[WriteOff]) -> Result<Vec<Row>, String> {
    let one = |chosen: &Chosen| {
        row(chosen, verdict, offs).ok_or_else(|| format!("{}: {}", chosen.file, chosen.function))
    };
    stage.chosen.iter().map(one).collect()
}
