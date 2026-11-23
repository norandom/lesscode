//! Asking for the run page (`.kiro/specs/run-dashboard`, task 3.2):
//! `--page` writes the report of the run at its end, `--open` also shows
//! the run in the default browser while it happens. Without either, a run
//! is what it was. Nothing here changes what a run returns once it began.

use super::plan_page::PlanWatch;
use crate::adapters::driver::config::RunConfig;
use crate::adapters::driver::pack::Pack;
use crate::adapters::driver::page_open::{open, Opener};
use crate::adapters::driver::page_serve::{Served, LOCAL};
use crate::adapters::driver::page_subject::{subject, would_be_tracked};
use crate::adapters::driver::page_text::{live, write_report};
use crate::adapters::driver::page_watch::{Heart, BEAT};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// The options that ask for the page.
#[derive(clap::Args, Clone, Debug, Default)]
pub struct PageArgs {
    /// Write a page with the report of the run here when it ends; without a place, beside the other output.
    #[arg(long, value_name = "PLACE")]
    pub page: Option<Option<PathBuf>>,
    /// Show the run in the default browser while it happens, and write the page as with --page.
    #[arg(long)]
    pub open: bool,
    /// The address to listen on; this machine's own on a free port when absent.
    #[arg(skip)]
    pub listen: Option<String>,
    /// What starts the browser; the platform's opener when absent.
    #[arg(skip)]
    pub opener: Option<Opener>,
}

/// Where the report goes, or `None` when the page was not asked for.
/// `out` is the run's other output; the page goes beside it unless a place
/// is named.
fn place_of(show: &PageArgs, out: &Path) -> Result<Option<PathBuf>, String> {
    let place = match show.page.as_ref().and_then(Option::as_ref) {
        Some(place) => place.clone(),
        None if show.page.is_none() && !show.open => return Ok(None),
        None => {
            let beside = out.with_extension("html");
            if would_be_tracked(&beside) {
                return Err(format!(
                    "Refused: the page would be written to {}, among the files of a repository. Name the place: --page <place>.",
                    beside.display()
                ));
            }
            beside
        }
    };
    let holds = place
        .parent()
        .is_none_or(|d| d.as_os_str().is_empty() || d.is_dir());
    match holds {
        true => Ok(Some(place)),
        false => Err(format!(
            "The page cannot be written to {}: its directory does not exist.",
            place.display()
        )),
    }
}

/// A run that is shown: the log, its sign of life, the listener if the
/// person asked to see the run, and where the report goes.
pub struct Showing {
    watch: Arc<PlanWatch>,
    place: PathBuf,
    heart: Heart,
    served: Option<Served>,
}

/// The listener and the browser. Neither can stop the run: what fails is
/// said, and the run goes on.
fn serve(show: &PageArgs, watch: &Arc<PlanWatch>) -> Option<Served> {
    let from = Arc::clone(watch);
    let latest = move || {
        let snapshot = from.snapshot();
        let text = serde_json::to_string(&snapshot).unwrap_or_default();
        (snapshot.sequence, text)
    };
    let listen = show.listen.as_deref().unwrap_or(LOCAL);
    let served = match Served::start(listen, live, Arc::new(latest)) {
        Ok(served) => served,
        Err(why) => {
            eprintln!("The run cannot be shown while it happens ({why}). It goes on, and the page is written at its end.");
            return None;
        }
    };
    println!("The run is shown at {}", served.address());
    if let Err(why) = show.opener.unwrap_or(open)(served.address()) {
        eprintln!("The browser could not be started ({why}). The run goes on; open the address above to see it.");
    }
    Some(served)
}

/// Begins showing a run of `step plan` if the options ask for it. An
/// `Err` is a refusal before anything ran.
pub fn begin(
    show: &PageArgs,
    config: &RunConfig,
    pack: &Pack,
    work: &Path,
    out: &Path,
    copies: usize,
) -> Result<Option<Showing>, String> {
    let Some(place) = place_of(show, out)? else {
        return Ok(None);
    };
    let watch = Arc::new(PlanWatch::new(config, pack, work, copies)?);
    watch.log().set_subject(subject(&config.source_root));
    let served = show.open.then(|| serve(show, &watch)).flatten();
    Ok(Some(Showing {
        heart: watch.heart(BEAT),
        watch,
        place,
        served,
    }))
}

impl Showing {
    pub fn watch(&self) -> Arc<PlanWatch> {
        Arc::clone(&self.watch)
    }

    /// The run is over: the last snapshot goes down the streams, the
    /// listener ends, and the report is written and its place printed.
    pub fn end(self) {
        drop(self.heart);
        drop(self.served);
        match write_report(&self.place, &self.watch.snapshot()) {
            Ok(()) => println!("The page of the run is at {}", self.place.display()),
            Err(why) => eprintln!("The page of the run could not be written ({why})."),
        }
    }
}
