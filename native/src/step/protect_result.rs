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
use super::protect_numbers::{from_the_base_run, numbers, write_offs, Joined, WriteOff};
use super::protect_receipt::{says, Protection};
use super::protect_stage::{slashed, Stage};
use super::revisions::blob;
use super::verdict::{NOT_JUDGED, PLANTED_FAULTS, REFUSED};
use crate::adapters::driver::cli::emit_json;
use crate::specimen_runner::sha256_hex_bytes;
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
    /// Where the result is written; protection.json beside the declaration when absent.
    #[arg(long)]
    pub out: Option<PathBuf>,
}

/// What there is to read a result from.
pub enum Found {
    /// The verdict for the committed content of the branch, and its hash.
    Verdict(Value, String),
    /// Why there is no result.
    NotJudged(String),
}

/// A file of the stage as it is committed on the branch, and no other copy
/// (the owner's decision of 2026-11-24): a verdict or an approval that is
/// written and not committed does not count.
fn on_branch(context: &Context, path: &str) -> Result<Option<Vec<u8>>, String> {
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
            "there is no committed verdict at {path}. Run `step check` on {branch} and commit \
             its verdict first."
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
    Ok(Found::Verdict(verdict, sha256_hex_bytes(&bytes)))
}

/// The directory of the stage spec: where the declaration lies.
fn stage_dir(context: &Context) -> &str {
    context.path.rsplit_once('/').map_or("", |(dir, _)| dir)
}

/// `stage.json`, with the faults the verdict's base run did not notice and
/// the plan did, and the entries of `write-offs.json`, both beside the
/// declaration. A stage without write-offs has none.
fn stage_of(
    context: &Context,
    verdict: &Value,
) -> Result<(Stage, Vec<Joined>, Vec<WriteOff>), String> {
    let dir = stage_dir(context);
    let path = format!("{dir}/stage.json");
    let not_a_stage = || format!("{path} is not there: this is not a stage of `step protect`.");
    let bytes = on_branch(context, &path)?.ok_or_else(not_a_stage)?;
    let mut stage: Stage =
        serde_json::from_slice(&bytes).map_err(|e| format!("{path} is not a stage: {e}"))?;
    let joined = from_the_base_run(&mut stage, verdict);
    let offs = match on_branch(context, &format!("{dir}/write-offs.json"))? {
        Some(bytes) => write_offs(&bytes, &stage)?,
        None => Vec::new(),
    };
    Ok((stage, joined, offs))
}

/// Exit 0 when nothing waits in any chosen function, 1 otherwise; either
/// way the result is said and left in `protection.json`. Exit 2 where there
/// is no result to read, and nothing is written. Bad input, including a
/// declaration that is not committed on its branch, is an error (exit 3).
pub fn run_result(args: ResultArgs) -> Result<u8, String> {
    let context = Context::open(&args.repo, &args.declaration)?;
    let not_judged = |why: String| {
        println!("Not judged: {why}");
        Ok(2)
    };
    let (verdict, sha256) = match found(&context)? {
        Found::NotJudged(why) => return not_judged(why),
        Found::Verdict(verdict, sha256) => (verdict, sha256),
    };
    let (stage, joined, offs) = stage_of(&context, &verdict)?;
    let rows = match numbers(&stage, &verdict, &offs) {
        Ok(rows) => rows,
        Err(function) => {
            let why = "the verdict has no numbers before and after for";
            return not_judged(format!("{why} {function}. Run `step check` again."));
        }
    };
    let rose = verdict["result"] == REFUSED;
    let mut protection = Protection::new(&stage, &context.revisions, sha256, rows, rose);
    protection.from_the_base_run = joined;
    println!(
        "{}",
        says(&stage, &context.declaration.branch, &protection, &offs)
    );
    let beside = || {
        context
            .repo
            .join(stage_dir(&context))
            .join("protection.json")
    };
    let out = args.out.unwrap_or_else(beside);
    emit_json(&protection, Some(&out))?;
    println!("Written: {}", slashed(&out));
    Ok(u8::from(!protection.enough()))
}
