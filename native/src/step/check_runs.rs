//! The part of `step check` that runs things: the project's tests on the
//! base and on the step, planted faults in the functions the step changes
//! and in their callers, and the commands tied to acceptance criteria.

use super::check::CheckArgs;
use super::context::Context;
use super::faults::{run, Ended, Fault, FaultRun, Request};
use super::functions::{changed, FileFunctions, Tally};
use super::ledger::{last_accepted, read};
use super::trees::{fresh_copy, Tree};
use super::verdict::{CriterionEvidence, FunctionRow, Verdict};
use crate::adapters::driver::pack::Pack;
use crate::adapters::driver::process;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::time::Duration;

type Names = BTreeMap<String, Vec<String>>;

/// The functions of the scope at one revision.
fn functions_at(
    context: &Context,
    pack: &Pack,
    tree: &Tree,
) -> Result<BTreeMap<String, FileFunctions>, String> {
    let mut found = BTreeMap::new();
    for file in &context.declaration.scope {
        let text = context.text(tree, file)?;
        found.insert(file.clone(), FileFunctions::read(pack, file, &text)?);
    }
    Ok(found)
}

/// Per file the functions to run faults in: the changed ones, their
/// callers in the scope and the ones the declaration names.
fn functions_to_run(
    context: &Context,
    before: &BTreeMap<String, FileFunctions>,
    after: &BTreeMap<String, FileFunctions>,
) -> Result<BTreeMap<String, BTreeSet<String>>, String> {
    let mut changed_names = BTreeSet::new();
    for (file, base) in before {
        changed_names.extend(changed(base, &after[file]));
    }
    let mut wanted = BTreeMap::new();
    for (file, base) in before {
        let step = &after[file];
        let known = |name: &String| {
            base.bodies().contains_key(name.as_str()) || step.bodies().contains_key(name.as_str())
        };
        let focus = context
            .declaration
            .focus
            .get(file)
            .cloned()
            .unwrap_or_default();
        if let Some(missing) = focus.iter().find(|name| !known(name)) {
            return Err(format!("focus: {file} has no function {missing}"));
        }
        let mut names: BTreeSet<String> =
            changed_names.iter().filter(|n| known(n)).cloned().collect();
        names.extend(base.callers_of(&changed_names));
        names.extend(step.callers_of(&changed_names));
        names.extend(focus);
        wanted.insert(file.clone(), names);
    }
    Ok(wanted)
}

/// The wanted functions that exist at one revision.
fn present(
    wanted: &BTreeMap<String, BTreeSet<String>>,
    at: &BTreeMap<String, FileFunctions>,
) -> Names {
    let of_file = |(file, names): (&String, &BTreeSet<String>)| {
        let bodies = at[file].bodies();
        let there = names.iter().filter(|n| bodies.contains_key(n.as_str()));
        (file.clone(), there.cloned().collect())
    };
    wanted.iter().map(of_file).collect()
}

/// What a run that did not end complete means for the verdict.
fn ended(verdict: &mut Verdict, side: &str, run: &FaultRun) -> bool {
    let name = "tests and fault runs";
    match &run.ended {
        Ended::Complete => return true,
        Ended::TestsFail => {
            let reason = format!("the project's tests do not pass on the {side}");
            verdict.settle(name, vec![reason], "");
        }
        Ended::Incomplete => {
            verdict.not_judged(name, format!("faults on the {side} could not be classed"));
        }
        Ended::Void(why) => {
            verdict.not_judged(name, format!("the run on the {side} is void: {why}"))
        }
    }
    false
}

/// One side of a comparison: the functions of a file at a revision and
/// the faults run in that file.
struct Side<'a> {
    functions: &'a FileFunctions,
    faults: Vec<&'a Fault>,
}

impl<'a> Side<'a> {
    fn new(functions: &'a FileFunctions, faults: &'a [Fault], file: &str) -> Side<'a> {
        let faults = faults.iter().filter(|f| f.file == file).collect();
        Side { functions, faults }
    }

    /// The faults of a function, when the function exists on this side.
    fn tally(&self, name: &str) -> Option<Tally> {
        let exists = self.functions.bodies().contains_key(name);
        let own = self
            .faults
            .iter()
            .filter(|f| self.functions.owner(f.at) == name);
        let table = self.functions.tally(own.map(|f| (f.at, f.class.as_str())));
        exists.then(|| table.get(name).copied().unwrap_or_default())
    }

    fn unnoticed(&self, name: &str) -> Vec<String> {
        let own = self
            .faults
            .iter()
            .filter(|f| f.class == "survived" && self.functions.owner(f.at) == name);
        own.map(|f| f.label()).collect()
    }
}

/// Compare the faults of one file's functions before and after.
fn compare(
    context: &Context,
    file: &str,
    names: &BTreeSet<String>,
    [before, after]: [Side; 2],
    ledger: &[Value],
    verdict: &mut Verdict,
) -> Vec<String> {
    let mut reasons = Vec::new();
    let (mut total, mut unmatched) = ([0, 0], Vec::new());
    for name in names {
        let (was, is) = (before.tally(name), after.tally(name));
        let unnoticed_after = after.unnoticed(name);
        let reach = &context.declaration.out_of_reach;
        let out_of_reach = reach.iter().find(|o| o.function == *name);
        total[0] += was.map_or(0, |t| t.unnoticed);
        total[1] += is.map_or(0, |t| t.unnoticed);
        // The lower of the base and the best number already on the ledger.
        let held = last_accepted(ledger, file, name);
        let allowed = was.map(|w| held.map_or(w.unnoticed, |h| h.min(w.unnoticed)));
        match (allowed, is) {
            (Some(allowed), Some(is)) if is.unnoticed > allowed && out_of_reach.is_none() => {
                reasons.push(format!(
                    "{file}: {name}: unnoticed faults rise from {} to {}: {}",
                    allowed,
                    is.unnoticed,
                    unnoticed_after.join(", ")
                ));
            }
            (Some(_), Some(_)) => {}
            _ => unmatched.push(format!("{file}: {name}")),
        }
        verdict.functions.push(FunctionRow {
            file: file.to_owned(),
            function: name.clone(),
            before: was,
            after: is,
            out_of_reach: out_of_reach.map(|o| o.reason.clone()),
            unnoticed_after,
        });
    }
    if !unmatched.is_empty() && total[1] > total[0] {
        reasons.push(format!(
            "{file}: functions could not be matched by name and the unnoticed faults of the file rise from {} to {}",
            total[0], total[1]
        ));
    }
    verdict.unmatched_functions.extend(unmatched);
    reasons
}
/// Run the commands tied to acceptance criteria in a copy of the step.
fn evidence(context: &Context, tree: &Path, verdict: &mut Verdict) -> Result<(), String> {
    let declaration = &context.declaration;
    if declaration.criteria.is_empty() {
        return Ok(());
    }
    let config = declaration.run_config(tree, &BTreeMap::new())?;
    let limit = Duration::from_secs(config.timeout_ceiling_s);
    let (mut rows, mut reasons) = (Vec::new(), Vec::new());
    for criterion in &declaration.criteria {
        let done = process::run(&criterion.command, tree, &declaration.env, limit);
        let passed = done.exit_code == Some(0) && !done.timed_out;
        if !passed {
            reasons.push(format!(
                "the command for criterion {} does not pass",
                criterion.id
            ));
        }
        rows.push(CriterionEvidence {
            id: criterion.id.clone(),
            command: criterion.command.clone(),
            exit_code: done.exit_code,
            passed,
        });
    }
    verdict.settle(
        "evidence for criteria",
        reasons,
        "every tied command passes",
    );
    verdict.write_evidence(context, rows)
}

/// Tests and faults on both revisions, the comparison, the evidence.
pub fn judge_by_runs(
    context: &Context,
    pack: &Pack,
    args: &CheckArgs,
    verdict: &mut Verdict,
) -> Result<(), String> {
    let before = functions_at(context, pack, &context.base_tree)?;
    let after = functions_at(context, pack, &context.step_tree)?;
    let wanted = functions_to_run(context, &before, &after)?;
    let run_names = |names: &BTreeSet<String>| names.iter().cloned().collect();
    verdict.functions_run = wanted
        .iter()
        .map(|(f, n)| (f.clone(), run_names(n)))
        .collect();

    let mut runs = Vec::new();
    let sides = [
        ("base", &context.revisions.base, &before),
        ("step", &context.revisions.step, &after),
    ];
    for (side, revision, functions) in sides {
        let copy = args.work.join(side);
        fresh_copy(&context.repo, revision, &copy)?;
        let request = Request {
            name: side,
            copy: &copy,
            functions: &present(&wanted, functions),
            only: Vec::new(),
            workers: args.workers,
        };
        let done = run(&context.declaration, &args.work, request)?;
        if !ended(verdict, side, &done) {
            return Ok(());
        }
        runs.push(done);
    }
    verdict.pass(
        "tests and fault runs",
        "the project's tests pass on the base and on the step",
    );
    let ledger = read(&context.repo.join(&context.declaration.outputs.ledger))?;
    let mut reasons = Vec::new();
    for (file, names) in &wanted {
        let sides = [
            Side::new(&before[file], &runs[0].faults, file),
            Side::new(&after[file], &runs[1].faults, file),
        ];
        reasons.extend(compare(context, file, names, sides, &ledger, verdict));
    }
    verdict.settle(
        "planted faults",
        reasons,
        "in no function are more faults unnoticed after the step than before",
    );
    let built = args.work.join("step-run").join("w1").join("tree");
    let tree = if built.is_dir() {
        built
    } else {
        args.work.join("step")
    };
    evidence(context, &tree, verdict)
}
