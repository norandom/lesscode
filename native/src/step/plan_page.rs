//! `step plan` as the run page shows it (`.kiro/specs/run-dashboard`, task
//! 2.1): a listener that turns what the batch runner tells into the page's
//! snapshot. It knows the functions of the files, so each fault is counted
//! under its function as soon as it is judged.

use super::functions::FileFunctions;
use crate::adapters::driver::classes::{INVALID, KILLED, SURVIVED, TIMEOUT};
use crate::adapters::driver::config::RunConfig;
use crate::adapters::driver::pack::Pack;
use crate::adapters::driver::page_log::{Class, Classed, RunLog, StagePlan};
use crate::adapters::driver::page_snapshot::Snapshot;
use crate::adapters::driver::page_watch::{Event, Heart, Watch};
use crate::adapters::driver::sites::Site;
use crate::adapters::driver::source::SourceSet;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const PREPARE: &str = "prepare";
const FAULTS: &str = "faults";
const CLOSE: &str = "close";

/// The stages of `step plan`, as a person would name them.
pub const STAGES: &[StagePlan] = &[
    StagePlan {
        id: PREPARE,
        title: "Copy the project and run your tests on the unchanged code",
        what: "Your project is copied to a place outside the repository, and each copy builds it and runs your tests once. This is the slowest stage and has nothing to count. If the tests do not pass here, nothing can be judged.",
    },
    StagePlan {
        id: FAULTS,
        title: "Plant faults",
        what: "One small fault at a time is put into a copy, the project is built and your tests run. Each fault is then taken out again.",
    },
    StagePlan {
        id: CLOSE,
        title: "Check the copies and write the plan",
        what: "Your tests run once more on the unchanged code in each copy, to see that no fault was left behind.",
    },
];

/// What the person can do after a plan.
const AFTER_A_PLAN: &[&str] = &[
    "Read the tab Unprotected. In those functions a change can break the code without any test failing.",
    "Write tests for a function before you change it, and run this again to see that they notice its faults.",
    "Remove the copies when you are done: delete the directory named under Your code.",
];

/// What the person can do when nothing could be judged.
const AFTER_NO_JUDGEMENT: &[&str] = &[
    "Run your test command yourself in one of the copies and make it pass there.",
    "Nothing in this run says anything about your tests or your code.",
];

fn lines(texts: &[&str]) -> Vec<String> {
    texts.iter().map(|t| (*t).to_owned()).collect()
}

/// Milliseconds since 1970.
pub fn now() -> u64 {
    let since = SystemTime::now().duration_since(UNIX_EPOCH);
    since.map_or(0, |d| d.as_millis() as u64)
}

fn class_of(class: &str) -> Class {
    match class {
        KILLED => Class::Noticed,
        SURVIVED => Class::Unnoticed,
        INVALID => Class::NotBuilt,
        TIMEOUT => Class::TimedOut,
        _ => Class::NotJudged,
    }
}

pub struct PlanWatch {
    log: Mutex<RunLog>,
    functions: BTreeMap<String, FileFunctions>,
}

impl std::fmt::Debug for PlanWatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PlanWatch")
    }
}

impl PlanWatch {
    /// A log for one run of `step plan` on the configured files, in `copies`
    /// copies under `work`.
    pub fn new(
        config: &RunConfig,
        pack: &Pack,
        work: &Path,
        copies: usize,
    ) -> Result<Self, String> {
        let source = SourceSet {
            root: config.source_root.clone(),
            files: config.files.clone(),
            test_paths: config.test_paths.clone(),
        };
        let mut functions = BTreeMap::new();
        for file in &config.files {
            let read = FileFunctions::read(pack, file, &source.raw(file)?)?;
            functions.insert(file.clone(), read);
        }
        let mut log = RunLog::new("step plan", env!("CARGO_PKG_VERSION"), STAGES, now());
        log.set_copies(&work.to_string_lossy().replace('\\', "/"), copies as u64);
        let mut commands = config.setup.clone();
        commands.extend(config.build.clone());
        commands.push(config.test.clone());
        log.set_commands(commands, !config.test.is_empty());
        Ok(Self {
            log: Mutex::new(log),
            functions,
        })
    }

    pub fn log(&self) -> MutexGuard<'_, RunLog> {
        self.log
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn snapshot(&self) -> Snapshot {
        self.log().snapshot()
    }

    /// Signs of life every `every`, until what is returned is dropped.
    pub fn heart(self: &Arc<Self>, every: Duration) -> Heart {
        let watch = Arc::clone(self);
        Heart::start(every, move || watch.log().beat(now()))
    }

    /// The plan is written: what each unprotected function reaches out to,
    /// and the end of the run with the plan's exit code.
    pub fn close<'a>(
        &self,
        reaching: impl Iterator<Item = (&'a str, &'a str, &'a [String])>,
        code: u8,
    ) {
        let mut log = self.log();
        for (file, function, calls) in reaching {
            log.reaches_out(file, function, calls.to_vec());
        }
        let functions = log.snapshot().faults.functions;
        let bare = functions.iter().filter(|f| f.unprotected).count();
        let detail = format!(
            "{bare} of {} functions with planted faults are unprotected.",
            functions.len()
        );
        let outcome = match code {
            0 => "The plan is written",
            _ => "The plan is written, and some faults could not be run",
        };
        log.finish(outcome, &detail, code, lines(AFTER_A_PLAN), now());
    }

    /// The run could not judge anything, and why.
    pub fn failed(&self, why: &str) {
        self.log()
            .finish("Could not judge", why, 3, lines(AFTER_NO_JUDGEMENT), now());
    }

    /// The function a site lies in, by the file's functions as they are
    /// before any fault.
    fn owner(&self, site: &Site) -> Option<&str> {
        let functions = self.functions.get(&site.file)?;
        Some(functions.owner((site.start_line, site.start_column)))
    }
}

impl Watch for PlanWatch {
    fn tell(&self, event: Event<'_>) {
        let (mut log, at) = (self.log(), now());
        match event {
            Event::Planned(jobs) => {
                for job in jobs {
                    log.expect(&job.site.file, self.owner(&job.site), 1);
                }
                log.begin(PREPARE, None, at);
            }
            Event::Unchanged(report) => {
                let outcome = report.stopped.clone().map_or(Ok(None), Err);
                log.unchanged_tests(report.worker as u64, outcome, at);
            }
            Event::Classed(result) => {
                // The first judged fault ends the waiting and begins the counting.
                let (done, total) = log.counted();
                if done == 0 {
                    log.end(PREPARE, at);
                    log.begin(FAULTS, Some(total), at);
                }
                log.classed(
                    Classed {
                        file: result.site.file.clone(),
                        function: self.owner(&result.site).map(str::to_owned),
                        line: result.site.start_line as u64,
                        operator: result.operator.clone(),
                        text: result.site.text.clone(),
                        class: class_of(&result.class),
                    },
                    at,
                );
                if done + 1 == total {
                    log.end(FAULTS, at);
                    log.begin(CLOSE, None, at);
                }
            }
        }
    }
}
