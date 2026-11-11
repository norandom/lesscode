//! `step check`: one answer for a step. The checks that need no run come
//! first; the project's tests and the planted faults follow only if those
//! pass. Every check is in the verdict with its result.

use super::check_runs::judge_by_runs;
use super::context::Context;
use super::trees::{changed_paths, commit_changes};
use super::verdict::{Verdict, ACCEPTED};
use crate::adapter_traits::FrameSpec;
use crate::adapters::driver::analysis::{analyse, outside_frame};
use crate::adapters::driver::measure::measure;
use crate::adapters::driver::pack::Pack;
use crate::adapters::driver::registry::embedded;
use std::path::PathBuf;

/// Arguments of `step check`.
#[derive(clap::Args, Clone, Debug)]
pub struct CheckArgs {
    /// The step declaration, relative to the repository root.
    #[arg(long)]
    pub declaration: String,
    /// The repository.
    #[arg(long, default_value = ".")]
    pub repo: PathBuf,
    /// Where copies and reusable outcomes are kept; outside the repository.
    #[arg(long)]
    pub work: PathBuf,
    /// Copies to run at once.
    #[arg(long)]
    pub workers: Option<usize>,
}

fn short(hash: &str) -> &str {
    &hash[..hash.len().min(7)]
}

/// Files the step changes that are neither in scope, nor tests, nor its own.
fn outside_scope(context: &Context) -> Vec<String> {
    let changed = changed_paths(&context.base_tree, &context.step_tree);
    let stray = changed
        .into_iter()
        .filter(|p| !context.is_judged(p) && !context.is_own(p));
    stray
        .map(|p| format!("{p} is changed and is outside the scope"))
        .collect()
}

/// Commits that change tests and code under test together: a pin in such a
/// commit was never shown against the code as it was.
fn mixed_commits(context: &Context) -> Result<Vec<String>, String> {
    let (base, step) = (&context.revisions.base, &context.revisions.step);
    let declaration = &context.declaration;
    let mut reasons = Vec::new();
    for (commit, paths) in commit_changes(&context.repo, base, step)? {
        let test = paths.iter().find(|p| declaration.is_test_path(p));
        let code = paths.iter().find(|p| declaration.scope.contains(p));
        if let (Some(test), Some(code)) = (test, code) {
            reasons.push(format!(
                "commit {} changes tests ({test}) and code under test ({code}) together",
                short(&commit)
            ));
        }
    }
    Ok(reasons)
}

/// Tokens and nodes of the scope before and after; a reason when the scope
/// grows by more tokens than the declaration allows.
fn size(context: &Context, pack: &Pack, verdict: &mut Verdict) -> Result<Vec<String>, String> {
    for (side, tree) in [&context.base_tree, &context.step_tree].iter().enumerate() {
        for file in &context.declaration.scope {
            let text = context.text(tree, file)?;
            if !text.is_empty() {
                let measured = measure(pack, file, &text)?;
                verdict.size.tokens[side] += measured.tokens;
                verdict.size.nodes[side] += measured.nodes;
            }
        }
    }
    let [before, after] = verdict.size.tokens;
    let allowed = context.declaration.max_token_growth;
    let grows = after > before + allowed;
    let reason = format!(
        "the scope grows from {before} to {after} tokens; the declaration allows {allowed} more"
    );
    Ok(grows.then_some(reason).into_iter().collect())
}

/// State writes of the step that a declared frame specification does not allow.
fn outside_the_frame(context: &Context, pack: &Pack) -> Result<Vec<String>, String> {
    let Some(path) = &context.declaration.frame_spec else {
        return Ok(Vec::new());
    };
    let text = context.text(&context.step_tree, path)?;
    let spec: FrameSpec = serde_json::from_str(&text)
        .map_err(|e| format!("{path} is not a frame specification: {e}"))?;
    let mut reasons = Vec::new();
    for file in &context.declaration.scope {
        let source = context.text(&context.step_tree, file)?;
        let writes = analyse(pack, file, &source)?.write_sites;
        let outside = outside_frame(&writes, &spec);
        reasons.extend(
            outside
                .into_iter()
                .map(|site| format!("{site} writes state outside the declared frame")),
        );
    }
    Ok(reasons)
}

/// Exit 0 accepted, 1 refused, 2 not judged. Bad input, including work on
/// disk that is not committed, is an error (exit 3) and writes nothing.
pub fn run_check(args: CheckArgs) -> Result<u8, String> {
    let context = Context::open(&args.repo, &args.declaration)?;
    let pack = embedded(&context.declaration.language).map_err(|e| e.to_string())?;
    let mut verdict = Verdict::new(&context);
    let (base, step) = (&context.revisions.base, &context.revisions.step);
    verdict.pass(
        "declaration and revisions",
        format!("base {}, step {}", short(base), short(step)),
    );
    verdict.settle(
        "changes stay in scope",
        outside_scope(&context),
        "only scope, tests and the step's own files change",
    );
    verdict.settle(
        "pins apart from the change",
        mixed_commits(&context)?,
        "no commit changes tests and code under test together",
    );
    let grown = size(&context, pack, &mut verdict)?;
    verdict.settle("size", grown, "within the declared growth");
    verdict.settle(
        "state frame",
        outside_the_frame(&context, pack)?,
        "no state write outside the declared frame, or none declared",
    );
    if verdict.result == ACCEPTED {
        judge_by_runs(&context, pack, &args, &mut verdict)?;
    }
    verdict.write(&context)?;
    println!("step {}: {}", verdict.step, verdict.result);
    for reason in &verdict.reasons {
        println!("  {reason}");
    }
    Ok(verdict.exit_code())
}
