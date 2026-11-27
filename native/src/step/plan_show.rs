//! Asking for the run page (`.kiro/specs/run-dashboard`, task 3.2):
//! `--page` writes the report of the run at its end, `--open` also shows
//! the run in the default browser while it happens. Without either, a run
//! is what it was. Nothing here changes what a run returns once it began.
//! The options and the showing are the same for every command that has the
//! page; `begin` is that of `step plan`.

use super::plan::PlanArgs;
use super::plan_page::{now, PlanWatch};
use crate::adapters::driver::config::RunConfig;
use crate::adapters::driver::pack::Pack;
use crate::dashboard::log::RunLog;
use crate::dashboard::open::{open, Opener};
use crate::dashboard::serve::{Served, LOCAL};
use crate::dashboard::snapshot::Snapshot;
use crate::dashboard::subject::{space, subject, would_be_tracked, written_inside};
use crate::dashboard::text::{live, write_report};
use crate::dashboard::watch::{Heart, BEAT};
use std::path::{Path, PathBuf};
use std::sync::{Arc, MutexGuard};

/// The options that ask for the page.
#[derive(clap::Args, Clone, Debug, Default)]
pub struct PageArgs {
    /// Write a page with the report of the run here when it ends; without a place, beside the other output.
    #[arg(long, value_name = "PLACE")]
    pub page: Option<Option<PathBuf>>,
    /// Show the run in the default browser while it happens, and write the page as with --page.
    #[arg(long)]
    pub open: bool,
    /// Show the line of code of the example fault on the page; without this the page has names and no source text.
    #[arg(long = "page-source")]
    pub source: bool,
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
pub fn place_of(show: &PageArgs, out: &Path) -> Result<Option<PathBuf>, String> {
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

/// What listens to a run of some command and keeps its log.
pub trait Shown: Send + Sync + 'static {
    fn log(&self) -> MutexGuard<'_, RunLog>;
}

impl Shown for PlanWatch {
    fn log(&self) -> MutexGuard<'_, RunLog> {
        PlanWatch::log(self)
    }
}

/// A run that is shown: the log, its sign of life, the listener if the
/// person asked to see the run, and where the report goes.
pub struct Showing<W> {
    watch: Arc<W>,
    place: PathBuf,
    work: PathBuf,
    heart: Heart,
    served: Option<Served>,
    /// Whether the person asked for source text on the page.
    source: bool,
}

/// The snapshot as the page gets it: the names of files and functions, and
/// source text only where the person asked for it (requirement 7.9).
fn shown<W: Shown>(watch: &W, source: bool) -> Snapshot {
    let mut snapshot = watch.log().snapshot();
    if let (false, Some(example)) = (source, &mut snapshot.faults.example) {
        example.text.clear();
    }
    snapshot
}

/// The listener and the browser. Neither can stop the run: what fails is
/// said, and the run goes on.
fn serve<W: Shown>(show: &PageArgs, watch: &Arc<W>) -> Option<Served> {
    let (from, source) = (Arc::clone(watch), show.source);
    let latest = move || {
        let snapshot = shown(&*from, source);
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
    args: &PlanArgs,
    config: &RunConfig,
    pack: &Pack,
) -> Result<Option<Showing<PlanWatch>>, String> {
    let show = &args.show;
    let Some(place) = place_of(show, &args.out)? else {
        return Ok(None);
    };
    let copies = config.parallel(args.workers);
    let watch = Arc::new(PlanWatch::new(config, pack, &args.work, copies)?);
    watch.log().set_subject(subject(&config.source_root));
    // What of the run's own output lies among the repository's files.
    let written = [args.out.as_path(), place.as_path(), args.work.as_path()];
    let inside = written_inside(&config.source_root, &written);
    watch.log().set_writes(inside);
    Ok(Some(Showing::start(show, watch, place, &args.work)))
}

impl<W: Shown> Showing<W> {
    /// Shows the run that `watch` listens to: the listener and the browser
    /// if the person asked to see it, and signs of life from now on. The
    /// report goes to `place`; the copies are under `work`.
    pub fn start(show: &PageArgs, watch: Arc<W>, place: PathBuf, work: &Path) -> Self {
        let served = show.open.then(|| serve(show, &watch)).flatten();
        let alive = Arc::clone(&watch);
        Self {
            heart: Heart::start(BEAT, move || alive.log().beat(now())),
            watch,
            place,
            work: work.to_path_buf(),
            served,
            source: show.source,
        }
    }

    pub fn watch(&self) -> Arc<W> {
        Arc::clone(&self.watch)
    }

    /// The run is over: the last snapshot goes down the streams, the
    /// listener ends, and the report is written and its place printed.
    pub fn end(self) {
        drop(self.heart);
        if let Some(bytes) = space(&self.work) {
            self.watch.log().set_space(bytes);
        }
        drop(self.served);
        let snapshot = shown(&*self.watch, self.source);
        match write_report(&self.place, &snapshot) {
            Ok(()) => println!("The page of the run is at {}", self.place.display()),
            Err(why) => eprintln!("The page of the run could not be written ({why})."),
        }
    }
}
