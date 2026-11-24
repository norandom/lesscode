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
use super::revisions::blob;
use super::verdict::NOT_JUDGED;
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

/// The verdict the declaration names: the file in the working tree where
/// there is one, as `step record` takes it, else the committed one. Which
/// content it speaks for is decided by its hashes, not by where it lies.
fn verdict_bytes(context: &Context) -> Result<Option<Vec<u8>>, String> {
    let path = &context.declaration.outputs.verdict;
    if let Ok(bytes) = std::fs::read(context.repo.join(path)) {
        return Ok(Some(bytes));
    }
    let committed = context.step_tree.get(path);
    committed.map(|hash| blob(&context.repo, hash)).transpose()
}

/// The verdict a result may be read from, or why there is none. In this
/// order: no verdict; a verdict for other content than the branch has, by
/// the hashes `step guard` compares; a scope file the branch changed; a
/// verdict that did not judge.
pub fn found(context: &Context) -> Result<Found, String> {
    let path = &context.declaration.outputs.verdict;
    let branch = &context.declaration.branch;
    let Some(bytes) = verdict_bytes(context)? else {
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
    Ok(Found::Verdict(verdict))
}

/// Exit 2 where there is no result to read. Bad input, including a
/// declaration that is not committed on its branch, is an error (exit 3).
pub fn run_result(args: ResultArgs) -> Result<u8, String> {
    let context = Context::open(&args.repo, &args.declaration)?;
    match found(&context)? {
        Found::NotJudged(why) => {
            println!("Not judged: {why}");
            Ok(2)
        }
        Found::Verdict(_) => {
            println!(
                "The verdict at {} is the one for what {} has committed.",
                context.declaration.outputs.verdict, context.declaration.branch
            );
            Ok(0)
        }
    }
}
