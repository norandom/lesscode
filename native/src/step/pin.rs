//! `step pin`: is a test a pin for a named unnoticed fault? It is when the
//! project's tests, with it, pass on the code as it was and fail on that
//! fault. The tooling judges pins; it writes none.

use super::declaration::Declaration;
use super::faults::{run, Ended, Fault, Request};
use super::trees::{fresh_copy, pin_copy};
use crate::adapters::driver::cli::emit_json;
use serde::Serialize;
use std::path::PathBuf;

/// Arguments of `step pin`.
#[derive(clap::Args, Clone, Debug)]
pub struct PinArgs {
    /// The step declaration, relative to the repository root.
    #[arg(long)]
    pub declaration: String,
    /// The number of an unnoticed fault, as `step pin` or `step check` list them for the base.
    #[arg(long)]
    pub fault: usize,
    /// The repository.
    #[arg(long, default_value = ".")]
    pub repo: PathBuf,
    /// Where copies and reusable outcomes are kept; outside the repository.
    #[arg(long)]
    pub work: PathBuf,
    /// Copies to run at once.
    #[arg(long)]
    pub workers: Option<usize>,
    /// Where to write the result; printed only when absent.
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Serialize)]
struct PinReceipt {
    receipt: &'static str,
    step: String,
    base: String,
    tests_of: String,
    fault: String,
    /// `admitted`, `rejected` or `not_judged`.
    result: &'static str,
    reason: String,
    /// Unnoticed faults of the base that the tests of the step still let through.
    remaining_unnoticed: Vec<String>,
}

/// The result and its reason for one fault after the run with the pin.
fn judged(ended: &Ended, with_pin: Option<&Fault>) -> (&'static str, String) {
    let class = with_pin.map_or("", |f| f.class.as_str());
    match (ended, class) {
        (Ended::TestsFail, _) => (
            "rejected",
            "the tests with the pin fail on the unchanged code".into(),
        ),
        (Ended::Void(why), _) => ("not_judged", format!("the run is void: {why}")),
        (_, "killed") => (
            "admitted",
            "the tests with the pin pass on the unchanged code and fail on the fault".into(),
        ),
        (_, "survived") => (
            "rejected",
            "the tests with the pin pass on the fault".into(),
        ),
        (_, other) => ("not_judged", format!("the fault has the class {other:?}")),
    }
}

/// The faults of the base that its own tests do not notice, in the
/// functions the step is about.
fn base_unnoticed(
    args: &PinArgs,
    declaration: &Declaration,
    base: &str,
) -> Result<Vec<Fault>, String> {
    let copy = args.work.join("base");
    fresh_copy(&args.repo, base, &copy)?;
    let request = Request {
        name: "base",
        copy: &copy,
        functions: &declaration.focus,
        only: Vec::new(),
        workers: args.workers,
    };
    let run = run(declaration, &args.work, request)?;
    if run.ended != Ended::Complete {
        return Err(format!(
            "the faults of the base could not be run: {:?}",
            run.ended
        ));
    }
    let unnoticed = run.faults.into_iter().filter(|f| f.class == "survived");
    Ok(unnoticed.collect())
}

/// Exit 0 admitted, 1 rejected, 2 not judged; bad input is an error.
pub fn run_pin(args: PinArgs) -> Result<u8, String> {
    let declaration = Declaration::read(&args.repo.join(&args.declaration))?;
    let revisions = declaration.revisions(&args.repo)?;
    let unnoticed = base_unnoticed(&args, &declaration, &revisions.base)?;
    let Some(target) = unnoticed.iter().find(|f| f.number == args.fault) else {
        let known: Vec<String> = unnoticed.iter().map(Fault::label).collect();
        return Err(format!(
            "fault {} is not an unnoticed fault of the base; unnoticed are: {}",
            args.fault,
            known.join(", ")
        ));
    };

    let copy = args.work.join("pin");
    let is_test = |path: &str| declaration.is_test_path(path);
    pin_copy(&args.repo, &revisions.base, &revisions.step, is_test, &copy)?;
    let request = Request {
        name: "pin",
        copy: &copy,
        functions: &declaration.focus,
        only: unnoticed.iter().map(|f| f.number).collect(),
        workers: args.workers,
    };
    let pinned = run(&declaration, &args.work, request)?;
    let with_pin = pinned.faults.iter().find(|f| f.number == args.fault);
    let (result, reason) = judged(&pinned.ended, with_pin);
    let still = pinned.faults.iter().filter(|f| f.class == "survived");
    let receipt = PinReceipt {
        receipt: "step-pin-v1",
        step: declaration.step.clone(),
        base: revisions.base,
        tests_of: revisions.step,
        fault: target.label(),
        result,
        reason,
        remaining_unnoticed: still.map(Fault::label).collect(),
    };
    println!("{}: {} ({})", receipt.fault, receipt.result, receipt.reason);
    for fault in &receipt.remaining_unnoticed {
        println!("still unnoticed: {fault}");
    }
    emit_json(&receipt, args.out.as_deref())?;
    Ok(match result {
        "admitted" => 0,
        "rejected" => 1,
        _ => 2,
    })
}
