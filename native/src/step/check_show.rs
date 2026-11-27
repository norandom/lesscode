//! Asking for the run page of `step check` (`.kiro/specs/run-dashboard`,
//! task 6.1): the same options as for `step plan`, and what a check reads
//! and writes. Nothing here changes a verdict.

use super::check::CheckArgs;
use super::check_page::CheckWatch;
use super::context::Context;
use super::plan_show::{place_of, Showing, Shown};
use crate::adapters::driver::pack::Pack;
use crate::dashboard::subject::{change, written_inside};
use std::sync::Arc;

/// Begins showing a check if the options ask for it. An `Err` is a refusal
/// before anything ran. Without a named place the page would go beside the
/// verdict, among the repository's files, and that is refused.
pub fn begin(
    args: &CheckArgs,
    context: &Context,
    pack: &'static Pack,
) -> Result<Option<Showing<CheckWatch>>, String> {
    let declaration = &context.declaration;
    let verdict = context.repo.join(&declaration.outputs.verdict);
    let Some(place) = place_of(&args.show, &verdict)? else {
        return match args.measures {
            Some(_) => Err("--measures is shown on the page; ask for it with --page".into()),
            None => Ok(None),
        };
    };
    let watch = Arc::new(CheckWatch::new(context, args, pack)?);
    let (branch, revisions) = (&declaration.branch, &context.revisions);
    let subject = change(&context.repo, branch, &revisions.step, &revisions.base);
    // The verdict, the evidence where criteria are tied, and what of the
    // page and the copies lies among the repository's files.
    let mut written = vec![declaration.outputs.verdict.clone()];
    if !declaration.criteria.is_empty() {
        written.push(declaration.outputs.evidence.clone());
    }
    let own = [place.as_path(), args.work.as_path()];
    written.extend(written_inside(&context.repo, &own));
    {
        let mut log = watch.log();
        log.set_subject(subject);
        log.set_writes(written);
    }
    Ok(Some(Showing::start(&args.show, watch, place, &args.work)))
}
