//! `step check` as the run page shows it (`.kiro/specs/run-dashboard`, task
//! 6.1): a listener that turns what the check and its two fault runs tell
//! into the page's snapshot. The check judges; this only tells what it did,
//! and nothing here changes a verdict or an exit code.

use super::check::CheckArgs;
use super::context::Context;
use super::functions::FileFunctions;
use super::plan_page::{class_at, now};
use super::plan_show::{place_of, Showing, Shown};
use super::verdict::{FunctionRow, Verdict, ACCEPTED, REFUSED};
use crate::adapters::driver::pack::Pack;
use crate::adapters::driver::page_log::{Classed, RunLog, StagePlan};
use crate::adapters::driver::page_snapshot::{Refusal, Report};
use crate::adapters::driver::page_subject::{change, written_inside};
use crate::adapters::driver::page_watch::{Event, Watch, Watching};
use crate::adapters::driver::sites::{not_built_spans, Site, Span};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

const CHECKS: &str = "checks";
const VERDICT: &str = "verdict";

/// One side of the step: its name in the snapshot, and its two stages.
struct Side {
    name: &'static str,
    tests: &'static str,
    faults: &'static str,
}

const SIDES: [Side; 2] = [
    Side {
        name: "before",
        tests: "tests-before",
        faults: "faults-before",
    },
    Side {
        name: "after",
        tests: "tests-after",
        faults: "faults-after",
    },
];

/// The stages of `step check`, as a person would name them.
pub const STAGES: &[StagePlan] = &[
    StagePlan {
        id: CHECKS,
        title: "Check what the commits of the step show",
        what: "Nothing is run here. The tool reads the commits of the step: whether only files in its scope change, whether tests and code change in separate commits, whether the scope grows by more than the step allows, and whether state is written outside the declared frame. A step that does not pass here is refused before anything runs.",
    },
    StagePlan {
        id: SIDES[0].tests,
        title: "Copy the code before the step and run your tests on it",
        what: "The commit the step starts from is copied to a place outside the repository, and each copy builds it and runs your tests once. There is nothing to count here. If the tests do not pass, there is nothing to compare the step with.",
    },
    StagePlan {
        id: SIDES[0].faults,
        title: "Plant faults in the code before the step",
        what: "One small fault at a time is put into a copy of the code before the step, the project is built and your tests run. Faults are planted in the functions the step changes, in the functions that call them and in those the step names.",
    },
    StagePlan {
        id: SIDES[1].tests,
        title: "Copy the code after the step and run your tests on it",
        what: "The commit of the step is copied to a place outside the repository, and each copy builds it and runs your tests once. There is nothing to count here. If the tests do not pass, the step is refused.",
    },
    StagePlan {
        id: SIDES[1].faults,
        title: "Plant faults in the code after the step",
        what: "Faults are planted in the same functions as they are after the step. What the tests notice here is compared with what they noticed before.",
    },
    StagePlan {
        id: VERDICT,
        title: "Compare and write the verdict",
        what: "Per function, the faults no test notices after the step are compared with those before it. The commands the step ties to acceptance criteria run in a copy of the step. Then the verdict is written into the repository.",
    },
];

const REMOVE_THE_COPIES: &str =
    "Remove the copies when you are done: delete the directory named under Your code.";

/// How far the fault run of one side has come.
#[derive(Clone, Copy, PartialEq)]
enum Phase {
    /// No fault run has begun, or the last one is over.
    Idle,
    Tests,
    Faults,
}

/// Which side is run, how far it is, and the code of that side.
struct Turn {
    side: usize,
    phase: Phase,
    done: usize,
    total: usize,
    /// Whether anything ran, and whether the comparison has begun.
    ran: bool,
    judging: bool,
    functions: BTreeMap<String, FileFunctions>,
    /// Per file, the code that is not built on this system.
    not_built: BTreeMap<String, Vec<Span>>,
}

pub struct CheckWatch {
    log: Mutex<RunLog>,
    turn: Mutex<Turn>,
    /// Where the copies are, and how many work at once.
    copies: (String, u64),
    /// What reads the code of the step's language.
    pack: &'static Pack,
}

fn lines(texts: &[&str]) -> Vec<String> {
    texts.iter().map(|t| (*t).to_owned()).collect()
}

/// No test noticed any fault in the function before the step.
fn bare_before(row: &&FunctionRow) -> bool {
    row.before
        .is_some_and(|t| t.noticed == 0 && t.unnoticed + t.other > 0)
}

/// What an acceptance says, and what it does not say where nothing was
/// protected before (requirement 6.4).
fn accepted(verdict: &Verdict) -> (String, Vec<String>) {
    let mut detail =
        "Every check passed. In no function are more faults unnoticed after the step than before."
            .to_owned();
    let mut next = lines(&["Put the step on the ledger with `step record`."]);
    let bare: Vec<&str> = verdict
        .functions
        .iter()
        .filter(bare_before)
        .map(|r| r.function.as_str())
        .collect();
    if !bare.is_empty() {
        detail.push_str(&format!(
            " No test noticed any fault before the step in: {}. There the acceptance means only that nothing got worse; nothing was protected before.",
            bare.join(", ")
        ));
        next.push("Tests come first: write tests for the functions named above before another step changes them.".to_owned());
    }
    (detail, next)
}

impl CheckWatch {
    /// A log for one check of the step of `context`, begun with the checks
    /// that need no run.
    pub fn new(context: &Context, args: &CheckArgs, pack: &'static Pack) -> Result<Self, String> {
        let declaration = &context.declaration;
        let config = declaration.run_config(Path::new("."), &BTreeMap::new())?;
        let copies = (
            args.work.to_string_lossy().replace('\\', "/"),
            config.parallel(args.workers) as u64,
        );
        let mut log = RunLog::new("step check", env!("CARGO_PKG_VERSION"), STAGES, now());
        log.set_copies(&copies.0, copies.1);
        let mut commands = config.setup.clone();
        commands.extend(config.build.clone());
        commands.push(config.test.clone());
        commands.extend(declaration.criteria.iter().map(|c| c.command.clone()));
        log.set_commands(commands, !config.test.is_empty());
        log.begin(CHECKS, None, now());
        let turn = Turn {
            side: 0,
            phase: Phase::Idle,
            done: 0,
            total: 0,
            ran: false,
            judging: false,
            functions: BTreeMap::new(),
            not_built: BTreeMap::new(),
        };
        Ok(Self {
            log: Mutex::new(log),
            turn: Mutex::new(turn),
            copies,
            pack,
        })
    }

    fn turn(&self) -> MutexGuard<'_, Turn> {
        self.turn.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// The fault run of one side begins: 0 is the code before the step, 1
    /// the code after. Whoever holds what is returned has this listening.
    pub fn side(self: &Arc<Self>, side: usize, context: &Context) -> Watching {
        let tree = [&context.base_tree, &context.step_tree][side];
        // The code of the side, read for the page alone: what cannot be
        // read leaves a fault without its function and changes no result.
        let (mut functions, mut not_built) = (BTreeMap::new(), BTreeMap::new());
        for file in &context.declaration.scope {
            let text = context.text(tree, file).unwrap_or_default();
            if let Ok(found) = FileFunctions::read(self.pack, file, &text) {
                functions.insert(file.clone(), found);
            }
            let spans = not_built_spans(self.pack, &text, std::env::consts::OS);
            not_built.insert(file.clone(), spans.unwrap_or_default());
        }
        let (mut turn, at) = (self.turn(), now());
        *turn = Turn {
            side,
            phase: Phase::Tests,
            done: 0,
            total: 0,
            ran: true,
            judging: false,
            functions,
            not_built,
        };
        let mut log = self.log();
        if side == 0 {
            log.end(CHECKS, at);
        }
        log.set_side(Some(SIDES[side].name));
        log.set_copies(&self.copies.0, self.copies.1);
        log.begin(SIDES[side].tests, None, at);
        Watching::begin(Arc::clone(self) as Arc<dyn Watch>)
    }

    /// The fault run of the side ended and every fault has a class.
    pub fn side_done(&self) {
        let (mut turn, at) = (self.turn(), now());
        let side = &SIDES[turn.side];
        match turn.phase {
            Phase::Tests => self.log().end(side.tests, at),
            Phase::Faults => self.log().end(side.faults, at),
            Phase::Idle => {}
        }
        turn.phase = Phase::Idle;
    }

    /// Both sides are run; what follows compares them.
    pub fn judging(&self) {
        self.turn().judging = true;
        let mut log = self.log();
        log.set_side(None);
        log.begin(VERDICT, None, now());
    }

    /// The verdict is written: the end of the run, with the verdict's
    /// result, its reasons and its exit code.
    pub fn close(&self, verdict: &Verdict) {
        let (turn, at) = (self.turn(), now());
        let mut log = self.log();
        let reasons = verdict.reasons.join("; ");
        match turn.judging {
            // The comparison was made, whatever it gave.
            true => log.end(VERDICT, at),
            false => log.fail_what_goes_on(&reasons, at),
        }
        let (outcome, detail, mut next) = match verdict.result {
            ACCEPTED => {
                let (detail, next) = accepted(verdict);
                ("Accepted", detail, next)
            }
            REFUSED => {
                let refusal = Refusal {
                    reasons: verdict.reasons.clone(),
                };
                log.set_report(Report {
                    refusal: Some(refusal),
                });
                let next = "Change the step so that the reasons no longer hold, and run this again. Fix the step, not the limit.";
                ("Refused", reasons, lines(&[next]))
            }
            _ => {
                let next = [
                    "Nothing in this run says whether the step is good or bad.",
                    "Run your test command yourself in one of the copies and make it run there.",
                ];
                ("Not judged", reasons, lines(&next))
            }
        };
        if turn.ran {
            next.push(REMOVE_THE_COPIES.to_owned());
        }
        log.finish(outcome, &detail, verdict.exit_code(), next, at);
    }

    /// The check ended with an error, and why. Nothing was judged.
    pub fn failed(&self, why: &str) {
        let mut log = self.log();
        if !log.snapshot().ended {
            let next = lines(&["Nothing in this run says anything about the step."]);
            log.finish("Could not judge", why, 3, next, now());
        }
    }

    fn classed(&self, turn: &Turn, site: &Site, operator: &str, class: &str) -> Classed {
        let functions = turn.functions.get(&site.file);
        let at = (site.start_line, site.start_column);
        let not_built = turn.not_built.get(&site.file);
        Classed {
            file: site.file.clone(),
            function: functions.map(|f| f.owner(at).to_owned()),
            line: site.start_line as u64,
            operator: operator.to_owned(),
            text: site.text.clone(),
            class: class_at(not_built.map_or(&[], Vec::as_slice), site, class),
        }
    }
}

impl Shown for CheckWatch {
    fn log(&self) -> MutexGuard<'_, RunLog> {
        self.log.lock().unwrap_or_else(|p| p.into_inner())
    }
}

impl Watch for CheckWatch {
    fn tell(&self, event: Event<'_>) {
        let (mut turn, at) = (self.turn(), now());
        let mut log = self.log();
        let side = &SIDES[turn.side];
        match event {
            Event::Planned(jobs) => {
                turn.total = jobs.len();
                for job in jobs {
                    let functions = turn.functions.get(&job.site.file);
                    let at = (job.site.start_line, job.site.start_column);
                    log.expect(&job.site.file, functions.map(|f| f.owner(at)), 1);
                }
            }
            Event::Unchanged(report) => match (&report.stopped, turn.side) {
                (None, _) => log.unchanged_tests(report.worker as u64, Ok(None), at),
                (Some(why), 0) => log.unchanged_tests(report.worker as u64, Err(why.clone()), at),
                // After the step the tests are not on unchanged code: a copy
                // that stops says something about the step.
                (Some(why), _) => log.copy_dropped(report.worker as u64, why.clone(), at),
            },
            Event::Classed(result) => {
                // The first judged fault ends the waiting and begins the counting.
                if turn.phase == Phase::Tests {
                    log.end(side.tests, at);
                    log.begin(side.faults, Some(turn.total as u64), at);
                    turn.phase = Phase::Faults;
                }
                log.classed(
                    self.classed(&turn, &result.site, &result.operator, &result.class),
                    at,
                );
                turn.done += 1;
                if turn.done == turn.total {
                    log.end(side.faults, at);
                    turn.phase = Phase::Idle;
                }
            }
        }
    }
}

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
        return Ok(None);
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
