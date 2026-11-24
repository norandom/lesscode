//! `step protect result`: whether the chosen functions of a stage are
//! protected enough (`.kiro/specs/brownfield-test-stage`, group 2).
//!
//! The result is read off the verdict of `step check`, so it holds only
//! where that verdict does: for exactly the scope, the tests and the
//! declaration the branch has committed, and for code the stage left as it
//! was. Where it does not hold the command says why, exits 2 and writes
//! nothing. It runs nothing.

use super::context::Context;
use super::guard::moved;
use super::protect_numbers::{numbers, write_offs, Row, WriteOff};
use super::protect_stage::Stage;
use super::revisions::blob;
use super::verdict::{NOT_JUDGED, PLANTED_FAULTS, REFUSED};
use serde_json::Value;
use std::path::PathBuf;

/// Arguments of `step protect result`.
#[derive(clap::Args, Clone, Debug)]
pub struct ResultArgs {
    /// The declaration of the stage, relative to the repository root: <specs-dir>/<name>/step.json.
    #[arg(long)]
    pub declaration: String,
    /// The repository.
    #[arg(long, default_value = ".")]
    pub repo: PathBuf,
}

/// What there is to read a result from.
pub enum Found {
    /// The verdict for the committed content of the branch.
    Verdict(Value),
    /// Why there is no result.
    NotJudged(String),
}

/// A file of the stage: the one in the working tree where there is one, as
/// `step record` takes a verdict, else the one committed on the branch.
/// Which content a verdict speaks for is decided by its hashes, not by
/// where it lies.
fn on_branch(context: &Context, path: &str) -> Result<Option<Vec<u8>>, String> {
    if let Ok(bytes) = std::fs::read(context.repo.join(path)) {
        return Ok(Some(bytes));
    }
    let committed = context.step_tree.get(path);
    committed.map(|hash| blob(&context.repo, hash)).transpose()
}

/// The checks of a verdict that refused, other than the one on planted
/// faults, each with what it found.
fn refused_otherwise(verdict: &Value) -> Vec<String> {
    let checks = verdict["checks"].as_array().map(Vec::as_slice);
    let other = |c: &&Value| c["result"] == REFUSED && c["name"] != PLANTED_FAULTS;
    let said = |c: &Value| {
        let text = |key: &str| c[key].as_str().unwrap_or_default().to_owned();
        format!("{}: {}", text("name"), text("detail"))
    };
    checks
        .unwrap_or(&[])
        .iter()
        .filter(other)
        .map(said)
        .collect()
}

/// The verdict a result may be read from, or why there is none. In this
/// order: no verdict; a verdict for other content than the branch has, by
/// the hashes `step guard` compares; a scope file the branch changed; a
/// verdict that did not judge, or refused for a reason other than faults.
pub fn found(context: &Context) -> Result<Found, String> {
    let path = &context.declaration.outputs.verdict;
    let branch = &context.declaration.branch;
    let Some(bytes) = on_branch(context, path)? else {
        return Ok(Found::NotJudged(format!(
            "there is no verdict at {path}. Run `step check` on {branch} first."
        )));
    };
    let verdict: Value =
        serde_json::from_slice(&bytes).map_err(|e| format!("{path} is not JSON: {e}"))?;
    let now = context.hashes_of(&context.step_tree);
    if let Some(what) = moved(&verdict, &now, &context.revisions.base) {
        return Ok(Found::NotJudged(format!(
            "the verdict at {path} is not the one for what {branch} has committed: {what}. \
             Run `step check` again."
        )));
    }
    let differs = |file: &&String| context.base_tree.get(*file) != context.step_tree.get(*file);
    let changed: Vec<&str> = context
        .declaration
        .scope
        .iter()
        .filter(differs)
        .map(String::as_str)
        .collect();
    if !changed.is_empty() {
        return Ok(Found::NotJudged(format!(
            "{} differs between the base and {branch}, so the result does not hold for the \
             functions of that file. A stage adds tests and leaves the code as it is; a change \
             of the code is a step of its own.",
            changed.join(", ")
        )));
    }
    if verdict["result"] == NOT_JUDGED {
        let reasons = verdict["reasons"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let reasons: Vec<&str> = reasons.iter().filter_map(Value::as_str).collect();
        return Ok(Found::NotJudged(format!(
            "`step check` did not judge {branch}: {}.",
            reasons.join("; ")
        )));
    }
    let otherwise = refused_otherwise(&verdict);
    if !otherwise.is_empty() {
        return Ok(Found::NotJudged(format!(
            "`step check` refused {branch} for a reason other than faults: {}.",
            otherwise.join("; ")
        )));
    }
    Ok(Found::Verdict(verdict))
}

/// `stage.json` and the entries of `write-offs.json`, both beside the
/// declaration. A stage without write-offs has none.
fn stage_of(context: &Context) -> Result<(Stage, Vec<WriteOff>), String> {
    let dir = context.path.rsplit_once('/').map_or("", |(dir, _)| dir);
    let path = format!("{dir}/stage.json");
    let not_a_stage = || format!("{path} is not there: this is not a stage of `step protect`.");
    let bytes = on_branch(context, &path)?.ok_or_else(not_a_stage)?;
    let stage: Stage =
        serde_json::from_slice(&bytes).map_err(|e| format!("{path} is not a stage: {e}"))?;
    let offs = match on_branch(context, &format!("{dir}/write-offs.json"))? {
        Some(bytes) => write_offs(&bytes, &stage)?,
        None => Vec::new(),
    };
    Ok((stage, offs))
}

/// What the result says of one function: its four numbers, then each fault
/// that waits, is written off, or has a write-off and is noticed.
fn says(row: &Row) -> String {
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
    for w in &row.write_offs {
        let reason = w.reason.as_deref().unwrap_or_default();
        lines.push(format!("    written off: {}: {reason}", w.fault));
    }
    for w in &row.noticed_write_offs {
        let fault = &w.fault;
        lines.push(format!(
            "    noticed: {fault} has a write-off, and a test notices it now"
        ));
    }
    lines.join("\n")
}

/// Exit 0 when nothing waits in any chosen function, 1 otherwise, 2 where
/// there is no result to read. Bad input, including a declaration that is
/// not committed on its branch, is an error (exit 3).
pub fn run_result(args: ResultArgs) -> Result<u8, String> {
    let context = Context::open(&args.repo, &args.declaration)?;
    let not_judged = |why: String| {
        println!("Not judged: {why}");
        Ok(2)
    };
    let verdict = match found(&context)? {
        Found::NotJudged(why) => return not_judged(why),
        Found::Verdict(verdict) => verdict,
    };
    let (stage, offs) = stage_of(&context)?;
    let rows = match numbers(&stage, &verdict, &offs) {
        Ok(rows) => rows,
        Err(function) => {
            let why = "the verdict has no numbers before and after for";
            return not_judged(format!("{why} {function}. Run `step check` again."));
        }
    };
    println!(
        "The stage {} on {}:",
        stage.spec, context.declaration.branch
    );
    for row in &rows {
        println!("{}", says(row));
    }
    let waiting: usize = rows.iter().map(|r| r.waiting).sum();
    let rose = verdict["result"] == REFUSED;
    if rose {
        println!("`step check` refused the branch because unnoticed faults rose: a test that noticed a fault at the base does not notice it now.");
    }
    if waiting == 0 && !rose {
        println!("Protected enough: nothing waits in any chosen function.");
        return Ok(0);
    }
    println!("Not protected enough: {waiting} fault(s) wait.");
    Ok(1)
}
