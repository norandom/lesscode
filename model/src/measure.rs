//! Is this model good enough for this kind of task: alone, with the repair
//! loop, or not as it is. A task set goes through its check for a number of
//! trials; the counts decide. "Good enough" has no meaning without the task
//! set and its check, so a report names both.

use crate::error::CallError;
use crate::protocol::{EXIT_ANSWERED, EXIT_FAILED, EXIT_REJECTED};
use crate::repair::{self, Outcome, with_repairs};
use crate::request::{CallRequest, Reply};
use crate::taskset::{Task, TaskSet};
use serde_json::{Value, json};

/// Good enough direct: at least this share of the first replies passed.
/// Good enough with the repair loop: at least `WITH_REPAIRS` of the trials
/// passed within the repairs allowed.
///
/// Decided by the owner on 2026-11-13 from the runs of 2026-11-10 to
/// 2026-11-12 (R74, R78, R99): 15 of 20 first replies accepted for the model
/// the tool was built against and 7 to 13 of 20 for another; 19 to 20 of 20
/// accepted within two repairs; a model that ended at 15 of 20 with repairs
/// was not improved by an optimiser.
pub const DIRECT: (u32, u32) = (3, 4);
pub const WITH_REPAIRS: (u32, u32) = (9, 10);

/// More than this share of the trials not run: no verdict.
pub const NOT_RUN_AT_MOST: (u32, u32) = (1, 5);

/// The quick mode calls a model not good enough when at most this share of
/// the trials that ran passed: two of five.
pub const QUICK_NOT_ABOVE: (u32, u32) = (2, 5);

pub const TRIALS: u32 = 20;
pub const QUICK_TRIALS: u32 = 5;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Counts {
    pub trials: u32,
    /// A call of the trial failed for a reason of the endpoint, or the run
    /// was ended before the trial.
    pub not_run: u32,
    /// The first reply passed the check.
    pub first_passed: u32,
    /// A reply passed within the repairs allowed.
    pub passed: u32,
    /// The calls that were answered, in all trials.
    pub calls: u32,
    pub output_tokens: u64,
    /// Waits included.
    pub seconds: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    GoodEnoughDirect,
    GoodEnoughWithRepair,
    /// An optimiser around the instruction is indicated.
    NotGoodEnoughAsIs,
    /// Quick mode: every trial passed.
    GoodEnoughSoFar,
    /// Quick mode: at most two of five passed.
    NotGoodEnough,
    /// Quick mode: anything between.
    Undecided,
    NoVerdict {
        why: String,
    },
}

impl Verdict {
    pub fn words(&self) -> &'static str {
        match self {
            Verdict::GoodEnoughDirect => "good enough direct",
            Verdict::GoodEnoughWithRepair => "good enough with the repair loop",
            Verdict::NotGoodEnoughAsIs => "not good enough as is",
            Verdict::GoodEnoughSoFar => "good enough so far",
            Verdict::NotGoodEnough => "not good enough",
            Verdict::Undecided => "undecided",
            Verdict::NoVerdict { .. } => "no verdict",
        }
    }

    /// 0 for the three verdicts that say good enough, 3 for no verdict, 4
    /// for the others.
    pub fn exit(&self) -> u8 {
        match self {
            Verdict::GoodEnoughDirect
            | Verdict::GoodEnoughWithRepair
            | Verdict::GoodEnoughSoFar => EXIT_ANSWERED,
            Verdict::NoVerdict { .. } => EXIT_FAILED,
            _ => EXIT_REJECTED,
        }
    }
}

/// Whether `part` of `whole` is more than the share `(numerator, denominator)`.
fn above(part: u32, whole: u32, share: (u32, u32)) -> bool {
    u64::from(part) * u64::from(share.1) > u64::from(whole) * u64::from(share.0)
}

/// Whether `part` of `whole` is at least the share.
fn at_least(part: u32, whole: u32, share: (u32, u32)) -> bool {
    u64::from(part) * u64::from(share.1) >= u64::from(whole) * u64::from(share.0)
}

/// The verdict for the counts. Rates are taken over the trials that ran.
pub fn verdict(counts: &Counts, quick: bool) -> Verdict {
    let ran = counts.trials - counts.not_run;
    if ran == 0 || above(counts.not_run, counts.trials, NOT_RUN_AT_MOST) {
        return Verdict::NoVerdict {
            why: format!(
                "{} of {} trials did not run, a verdict allows at most one fifth",
                counts.not_run, counts.trials
            ),
        };
    }
    if quick {
        return if counts.passed == counts.trials {
            Verdict::GoodEnoughSoFar
        } else if above(counts.passed, ran, QUICK_NOT_ABOVE) {
            Verdict::Undecided
        } else {
            Verdict::NotGoodEnough
        };
    }
    if at_least(counts.first_passed, ran, DIRECT) {
        Verdict::GoodEnoughDirect
    } else if at_least(counts.passed, ran, WITH_REPAIRS) {
        Verdict::GoodEnoughWithRepair
    } else {
        Verdict::NotGoodEnoughAsIs
    }
}

/// `trials` trials spread evenly over the tasks of `set`, each of them
/// `with_repairs` on one task. A trial whose call fails is not run and does
/// not count against the model. Once more than a fifth of the trials have
/// failed no verdict is possible, and the trials left are not started.
pub fn measure(
    call: &mut dyn FnMut(&CallRequest) -> Result<Reply, CallError>,
    set: &TaskSet,
    check: &mut dyn FnMut(&Task, &str) -> repair::Verdict,
    trials: u32,
    repairs: usize,
) -> Counts {
    let mut counts = Counts {
        trials,
        ..Counts::default()
    };
    let (mut ran, mut failed) = (0, 0);
    for trial in 0..trials {
        if set.tasks.is_empty() || above(failed, trials, NOT_RUN_AT_MOST) {
            break;
        }
        let task = &set.tasks[trial as usize % set.tasks.len()];
        let mut counted = |request: &CallRequest| {
            call(request).inspect(|reply| {
                counts.calls += 1;
                counts.output_tokens += reply.output_tokens;
                counts.seconds += reply.seconds;
            })
        };
        let outcome = with_repairs(
            &mut counted,
            &task.request,
            &mut |text| check(task, text),
            repairs,
        );
        match outcome {
            Outcome::Accepted { attempts, .. } => {
                ran += 1;
                counts.passed += 1;
                counts.first_passed += u32::from(attempts.len() == 1);
            }
            Outcome::Rejected { .. } => ran += 1,
            Outcome::Failed { .. } => failed += 1,
        }
    }
    counts.not_run = trials - ran;
    counts
}

/// One measurement with what it was made on, as the command reports it.
#[derive(Debug, Clone, PartialEq)]
pub struct Measured {
    pub task_set: String,
    pub check: String,
    pub model: String,
    pub host: String,
    pub repairs: usize,
    pub quick: bool,
    pub counts: Counts,
}

fn share(part: u32, whole: u32) -> String {
    match whole {
        0 => format!("{part} of 0"),
        _ => format!(
            "{part} of {whole} ({:.0} %)",
            f64::from(part) * 100.0 / f64::from(whole)
        ),
    }
}

impl Measured {
    pub fn verdict(&self) -> Verdict {
        verdict(&self.counts, self.quick)
    }

    /// The report for the terminal: task set, check, model and trials
    /// beside the counts, the cost per accepted answer and the verdict.
    pub fn lines(&self) -> Vec<String> {
        let counts = &self.counts;
        let ran = counts.trials - counts.not_run;
        let verdict = self.verdict();
        let cost = match counts.passed {
            0 => "no answer was accepted".to_owned(),
            accepted => format!(
                "{:.1} calls, {:.0} output tokens, {:.1} s",
                f64::from(counts.calls) / f64::from(accepted),
                counts.output_tokens as f64 / f64::from(accepted),
                counts.seconds / f64::from(accepted)
            ),
        };
        let mut lines = vec![
            format!("task set: {}", self.task_set),
            format!("check: {}", self.check),
            format!("model: {} at {}", self.model, self.host),
            format!(
                "trials: {}{}, {} not run",
                counts.trials,
                if self.quick { " (quick)" } else { "" },
                counts.not_run
            ),
            format!("first reply passed: {}", share(counts.first_passed, ran)),
            format!(
                "passed within {} repairs: {}",
                self.repairs,
                share(counts.passed, ran)
            ),
            format!("per accepted answer: {cost}"),
            format!("verdict: {}", verdict.words()),
        ];
        match verdict {
            Verdict::NoVerdict { why } => lines.push(format!("  {why}")),
            Verdict::NotGoodEnoughAsIs => lines.push(
                "  an optimiser around the instruction is indicated; measure its result with \
                 this same command"
                    .to_owned(),
            ),
            _ => {}
        }
        lines
    }

    /// The same as JSON, for `--out`.
    pub fn json(&self) -> Value {
        let counts = &self.counts;
        let verdict = self.verdict();
        let why = match &verdict {
            Verdict::NoVerdict { why } => Some(why.clone()),
            _ => None,
        };
        json!({
            "task_set": self.task_set,
            "check": self.check,
            "model": self.model,
            "host": self.host,
            "quick": self.quick,
            "repairs": self.repairs,
            "trials": counts.trials,
            "not_run": counts.not_run,
            "first_passed": counts.first_passed,
            "passed": counts.passed,
            "calls": counts.calls,
            "output_tokens": counts.output_tokens,
            "seconds": (counts.seconds * 10.0).round() / 10.0,
            "verdict": verdict.words(),
            "why": why,
        })
    }
}
