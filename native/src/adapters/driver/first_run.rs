//! The first run of a copy: the project's commands on the unchanged code,
//! before any fault. It is the heaviest thing a copy does, a full build, so
//! the copies take turns where the machine's memory asks for it. A first run
//! that does not get through is kept whole in a file, said in one sentence,
//! and tried once more when no other copy is in its first run.

use super::interrupt;
use super::memory;
use super::process::Finished;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Condvar, Mutex, MutexGuard};
use std::time::Duration;

/// Why a copy stops when the project's tests fail before any mutant.
pub const TESTS_FAIL: &str = "the unmutated tests do not pass in this copy";
/// Why a copy stops when its commands end before a test has run.
pub const NOT_REACHED: &str = "the unmutated run ended before any test ran in this copy";

/// The steps of one run of the project's commands, as far as they came.
pub type Steps = Vec<(&'static str, Finished)>;

/// What kept a first run from getting through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A command could not be started at all.
    Start,
    /// The commands ended before any test ran: a build failed or was ended.
    BeforeTests,
    /// Tests ran and did not all pass.
    Tests,
    /// A step ran over its limit.
    Time,
}

/// A first run that did not get through.
#[derive(Debug, Clone)]
pub struct Failure {
    pub kind: Kind,
    /// The output has a sign that the system ended a command.
    pub ended_by_system: bool,
    step: &'static str,
    exit_code: Option<i32>,
    launch: Option<String>,
    tail: String,
}

/// Signs in a command's output that the system ended it or one of its
/// children, in lower case: exit statuses of Windows from 0xC0000000 on, a
/// kill on Unix, and what allocators and linkers print without memory.
const ENDED_BY_SYSTEM: &[&str] = &[
    "exit code: 0xc",
    "exit code: 0xe",
    "signal: 9",
    "sigkill",
    "out of memory",
    "memory allocation of",
    "paging file",
];

/// The end of a command's output, for a message.
pub fn tail(output: &str) -> String {
    let skip = output.chars().count().saturating_sub(400);
    let end: String = output.chars().skip(skip).collect();
    end.trim().to_owned()
}

/// Why the run of the unchanged commands cannot be used, if so. `evidence`
/// are the fragments of output that show the test runner itself ran.
pub fn failure(evidence: &[String], done: &Steps) -> Option<Failure> {
    let found = |kind, step, f: &Finished| {
        let low = f.output.to_lowercase();
        let status = f.exit_code.is_some_and(|c| (c as u32) >= 0xC000_0000);
        Failure {
            kind,
            ended_by_system: status || ENDED_BY_SYSTEM.iter().any(|sign| low.contains(sign)),
            step,
            exit_code: f.exit_code,
            launch: f.launch_error.clone(),
            tail: tail(&f.output),
        }
    };
    if let Some((step, f)) = done.iter().find(|(_, f)| f.launch_error.is_some()) {
        return Some(found(Kind::Start, step, f));
    }
    // A run of the steps ends at the first one that does not pass.
    let (step, f) = done.iter().find(|(_, f)| !f.passed())?;
    let ran = evidence.iter().any(|sign| f.output.contains(sign.as_str()));
    let kind = if f.timed_out {
        Kind::Time
    } else if *step != "test" || (!evidence.is_empty() && !ran) {
        Kind::BeforeTests
    } else {
        Kind::Tests
    };
    Some(found(kind, step, f))
}

impl Failure {
    /// One line that says what happened, then the end of the output.
    pub fn said(&self, kept: Option<&Path>) -> String {
        if let (Kind::Start, Some(why)) = (self.kind, &self.launch) {
            return format!("a command could not be started: {why}");
        }
        let head = match self.kind {
            Kind::BeforeTests => NOT_REACHED,
            _ => TESTS_FAIL,
        };
        let timed_out = if self.kind == Kind::Time {
            ", timed out"
        } else {
            ""
        };
        let system = if self.ended_by_system {
            ", with a sign that the system ended a command, as happens when memory runs out"
        } else {
            ""
        };
        let kept = kept
            .map(|path| format!("; the whole output is in {}", path.display()))
            .unwrap_or_default();
        format!(
            "{head} ({} step, exit {:?}{timed_out}{system}{kept})\n{}",
            self.step, self.exit_code, self.tail
        )
    }
}

/// The first line of what a failure said: the sentence without the output.
pub fn sentence(said: &str) -> &str {
    said.lines().next().unwrap_or_default()
}

/// Writes the whole output of a first run beside the copies.
fn keep(work: &Path, worker: usize, attempt: usize, done: &Steps) -> Option<PathBuf> {
    let path = work.join(format!("first-run-w{worker}-{attempt}.log"));
    let text: String = done
        .iter()
        .map(|(step, f)| {
            format!(
                "== {step} step, exit {:?}, {:.1} s ==\n{}\n",
                f.exit_code, f.seconds, f.output
            )
        })
        .collect();
    std::fs::write(&path, text).ok().map(|()| path)
}

/// What a copy's first run came to.
#[derive(Debug, Default)]
pub struct First {
    /// Seconds of each step of the attempt that counts.
    pub seconds: Vec<f64>,
    /// Why the copy cannot be used, if so.
    pub stopped: Option<String>,
    /// What the first attempt said, where a second one was made.
    pub retried: Option<String>,
}

/// Which run of a copy asks for its turn: the first, a full build, or a
/// later one, which builds what one planted fault changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Run {
    First = 0,
    Later = 1,
}

#[derive(Default)]
struct State {
    running: usize,
    /// Memory set aside for the runs that are going on.
    set_aside: u64,
    /// The one that runs is with no other: it is measured, or a second attempt.
    alone: bool,
    /// What was free when none of the runs was going on, as last seen.
    idle: Option<u64>,
    /// Memory one run of each kind took: not yet measured, unknown on this
    /// system, or a number of bytes.
    need: [Option<Option<u64>>; 2],
    /// A second attempt failed too: the fault is not the load.
    retry_failed: bool,
}

impl State {
    /// Whether a run of this kind may start now, with `free` bytes available.
    fn admits(&mut self, kind: Run, alone: bool, free: Option<u64>) -> bool {
        if free.is_none() {
            // The system does not tell: nothing can be chosen.
            self.need[kind as usize] = Some(None);
        }
        let need = self.need[kind as usize];
        if self.running == 0 {
            self.idle = free;
            // A run that is not measured yet goes with no other beside it.
            self.alone = alone || need.is_none();
            return true;
        }
        let fits = match (need, self.idle) {
            (Some(Some(need)), Some(idle)) => idle >= self.set_aside + need + need / 4,
            (Some(None), _) => true,
            _ => false,
        };
        !alone && !self.alone && fits
    }

    /// Tells the person why a copy waits, where memory is the reason.
    fn says_why(&self, worker: usize, kind: Run) -> bool {
        let (Some(Some(need)), Some(idle)) = (self.need[kind as usize], self.idle) else {
            return false;
        };
        eprintln!(
            "copy {worker} waits for its turn: a run like this took about {}, {} were free with nothing going on, and {} are set aside",
            gib(need),
            gib(idle),
            gib(self.set_aside)
        );
        true
    }
}

/// Whose turn a run of the project's commands is.
pub struct Turns {
    state: Mutex<State>,
    changed: Condvar,
    available: fn() -> Option<u64>,
}

impl Default for Turns {
    fn default() -> Turns {
        Turns::reading(memory::available)
    }
}

fn gib(bytes: u64) -> String {
    format!("{:.1} GiB", bytes as f64 / (1u64 << 30) as f64)
}

impl Turns {
    fn reading(available: fn() -> Option<u64>) -> Turns {
        Turns {
            state: Mutex::default(),
            changed: Condvar::new(),
            available,
        }
    }

    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap()
    }

    /// Waits for this run's turn and says how much memory is set aside for
    /// it; `None` when it is the one that is measured.
    ///
    /// The first run of each kind goes with no other beside it and is
    /// measured. After that a run starts when nothing is going on, or when
    /// what was free with nothing going on holds the runs that are going on,
    /// this one, and a quarter of this one to spare.
    fn enter(&self, worker: usize, kind: Run, alone: bool) -> Option<u64> {
        let mut state = self.state();
        let mut told = alone;
        while !state.admits(kind, alone, (self.available)()) {
            // Said once, and only where memory is what the copy waits for.
            told = told || state.says_why(worker, kind);
            state = self
                .changed
                .wait_timeout(state, Duration::from_secs(2))
                .unwrap()
                .0;
        }
        state.running += 1;
        let aside = state.need[kind as usize].map(|need| need.unwrap_or(0));
        state.set_aside += aside.unwrap_or(0);
        aside
    }

    fn leave(&self, kind: Run, aside: Option<u64>, took: Option<u64>) {
        let mut state = self.state();
        state.running -= 1;
        state.alone = false;
        match aside {
            Some(aside) => state.set_aside -= aside,
            None => state.need[kind as usize] = Some(took),
        }
        self.changed.notify_all();
    }

    /// One run in its turn. `None` when a second attempt is no longer worth
    /// making because another copy's second attempt failed in the meantime.
    fn in_turn(
        &self,
        worker: usize,
        kind: Run,
        again: bool,
        run: &impl Fn() -> Steps,
    ) -> Option<Steps> {
        let aside = self.enter(worker, kind, again);
        if again && self.state().retry_failed {
            self.leave(kind, aside, None);
            return None;
        }
        let before = aside.is_none().then(|| (self.available)()).flatten();
        let Some(before) = before else {
            let done = run();
            self.leave(kind, aside, None);
            return Some(done);
        };
        // The least that was free while the run went on says what it took.
        let (stop, stopped) = mpsc::channel::<()>();
        let available = self.available;
        let (done, least) = std::thread::scope(|scope| {
            let sampler = scope.spawn(move || {
                let mut least = before;
                while stopped.recv_timeout(Duration::from_millis(500))
                    == Err(mpsc::RecvTimeoutError::Timeout)
                {
                    least = least.min(available().unwrap_or(least));
                }
                least.min(available().unwrap_or(least))
            });
            let done = run();
            drop(stop);
            (done, sampler.join().unwrap_or(before))
        });
        self.leave(kind, aside, Some(before.saturating_sub(least)));
        Some(done)
    }

    /// A run after the first: the commands with a planted fault, or the
    /// unchanged commands once more at the end.
    pub fn later_run(&self, worker: usize, run: impl Fn() -> Steps) -> Steps {
        self.in_turn(worker, Run::Later, false, &run)
            .unwrap_or_default()
    }

    /// The first run of a copy, and a second attempt where that can help.
    pub fn first_run(
        &self,
        evidence: &[String],
        work: &Path,
        worker: usize,
        run: impl Fn() -> Steps,
    ) -> First {
        let seconds = |done: &Steps| -> Vec<f64> { done.iter().map(|(_, f)| f.seconds).collect() };
        let done = self
            .in_turn(worker, Run::First, false, &run)
            .unwrap_or_default();
        let Some(first) = failure(evidence, &done) else {
            return First {
                seconds: seconds(&done),
                ..First::default()
            };
        };
        let said = first.said(keep(work, worker, 1, &done).as_deref());
        let stopped = |seconds, said: String, retried| First {
            seconds,
            stopped: Some(said),
            retried,
        };
        // A command that cannot start, or a step over its limit, is not
        // helped by a quieter machine.
        let hopeless = matches!(first.kind, Kind::Start | Kind::Time);
        if hopeless || interrupt::requested() || self.state().retry_failed {
            return stopped(seconds(&done), said, None);
        }
        eprintln!(
            "copy {worker}: {}; trying once more when no other copy is in its first run",
            sentence(&said)
        );
        let Some(again) = self.in_turn(worker, Run::First, true, &run) else {
            return stopped(seconds(&done), said, None);
        };
        let retried = Some(sentence(&said).to_owned());
        match failure(evidence, &again) {
            None => {
                eprintln!("copy {worker}: the second attempt got through");
                First {
                    seconds: seconds(&again),
                    stopped: None,
                    retried,
                }
            }
            Some(second) => {
                self.state().retry_failed = true;
                let said = second.said(keep(work, worker, 2, &again).as_deref());
                stopped(seconds(&again), said, retried)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

    fn step(exit_code: Option<i32>, output: &str) -> Steps {
        vec![(
            "test",
            Finished {
                exit_code,
                timed_out: false,
                seconds: 1.5,
                output: output.into(),
                launch_error: None,
            },
        )]
    }

    fn evidence() -> Vec<String> {
        vec!["Summary [".into()]
    }

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("lesscode-first-run-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_build_that_ends_is_told_from_a_test_that_fails() {
        assert!(failure(&evidence(), &step(Some(0), "Summary [ 1s] 3 tests run")).is_none());

        let built = "error: could not compile `x`\n (exit code: 0xe0000001)\nwarning: build failed";
        let before = failure(&evidence(), &step(Some(101), built)).unwrap();
        assert_eq!(
            (before.kind, before.ended_by_system),
            (Kind::BeforeTests, true)
        );
        let said = before.said(Some(Path::new("work/first-run-w1-1.log")));
        assert!(sentence(&said).starts_with(NOT_REACHED), "{said}");
        assert!(sentence(&said).contains("a sign that the system ended a command"));
        assert!(sentence(&said).contains("the whole output is in work"));
        assert!(said.ends_with("warning: build failed"));

        let failed = failure(
            &evidence(),
            &step(
                Some(100),
                "Summary [ 2s] 536 tests run: 535 passed, 1 failed",
            ),
        )
        .unwrap();
        assert_eq!((failed.kind, failed.ended_by_system), (Kind::Tests, false));
        assert!(failed.said(None).starts_with(TESTS_FAIL));

        // Without a sign of the runner in the configuration the two cannot be told apart.
        assert_eq!(
            failure(&[], &step(Some(101), built)).unwrap().kind,
            Kind::Tests
        );

        let mut late = step(None, "");
        late[0].1.timed_out = true;
        let late = failure(&evidence(), &late).unwrap();
        assert_eq!(late.kind, Kind::Time);
        assert!(late.said(None).contains("timed out"));

        let mut absent = step(None, "");
        absent[0].1.launch_error = Some("no such program".into());
        let absent = failure(&evidence(), &absent).unwrap();
        assert_eq!(
            absent.said(None),
            "a command could not be started: no such program"
        );
    }

    #[test]
    fn a_first_run_that_fails_once_is_tried_again_and_its_output_is_kept() {
        let (work, calls) = (scratch("again"), AtomicUsize::new(0));
        let turns = Turns::reading(|| None);
        let first = turns.first_run(&evidence(), &work, 1, || {
            match calls.fetch_add(1, Ordering::SeqCst) {
                0 => step(Some(101), "error: linking failed (exit code: 0xc0000409)"),
                _ => step(Some(0), "Summary [ 1s] 3 tests run: 3 passed"),
            }
        });
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(first.stopped, None);
        assert!(first.retried.unwrap().starts_with(NOT_REACHED));
        let kept = std::fs::read_to_string(work.join("first-run-w1-1.log")).unwrap();
        assert!(
            kept.contains("== test step, exit Some(101), 1.5 s ==")
                && kept.contains("linking failed")
        );
        assert!(!work.join("first-run-w1-2.log").exists());
    }

    #[test]
    fn when_a_second_attempt_fails_too_no_other_copy_makes_one() {
        let (work, calls) = (scratch("twice"), AtomicUsize::new(0));
        let turns = Turns::reading(|| None);
        let failing = || {
            calls.fetch_add(1, Ordering::SeqCst);
            step(Some(100), "Summary [ 1s] 3 tests run: 2 passed, 1 failed")
        };
        let one = turns.first_run(&evidence(), &work, 1, failing);
        assert!(one.stopped.unwrap().starts_with(TESTS_FAIL));
        assert!(one.retried.is_some());
        assert!(work.join("first-run-w1-2.log").exists());
        let two = turns.first_run(&evidence(), &work, 2, failing);
        assert!(two.stopped.is_some() && two.retried.is_none());
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    /// Three copies whose run says how many were in it at once.
    fn most_at_once(turns: &Turns, work: &Path, during: impl Fn() + Sync) -> usize {
        let (now, most) = (AtomicUsize::new(0), AtomicUsize::new(0));
        std::thread::scope(|scope| {
            for worker in 1..=3 {
                let (now, most, during) = (&now, &most, &during);
                scope.spawn(move || {
                    let first = turns.first_run(&evidence(), work, worker, || {
                        most.fetch_max(now.fetch_add(1, Ordering::SeqCst) + 1, Ordering::SeqCst);
                        during();
                        now.fetch_sub(1, Ordering::SeqCst);
                        step(Some(0), "Summary [ 1s] 3 tests run: 3 passed")
                    });
                    assert_eq!(first.stopped, None);
                });
            }
        });
        most.load(Ordering::SeqCst)
    }

    #[test]
    fn copies_take_turns_when_the_memory_does_not_hold_two_first_runs() {
        // 10 GiB free; a first run takes 8 of them while it goes on.
        static FREE: AtomicU64 = AtomicU64::new(10 << 30);
        let turns = Turns::reading(|| Some(FREE.load(Ordering::SeqCst)));
        let most = most_at_once(&turns, &scratch("turns"), || {
            FREE.store(2 << 30, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(700));
            FREE.store(10 << 30, Ordering::SeqCst);
        });
        assert_eq!(most, 1);
        assert_eq!(turns.state().need[Run::First as usize], Some(Some(8 << 30)));
        assert_eq!(turns.state().set_aside, 0);
    }

    #[test]
    fn copies_run_side_by_side_after_the_first_when_the_memory_holds_them() {
        // 10 GiB free; a first run takes 1 of them. The second and third meet inside their runs.
        static FREE: AtomicU64 = AtomicU64::new(10 << 30);
        static INSIDE: AtomicUsize = AtomicUsize::new(0);
        let turns = Turns::reading(|| Some(FREE.load(Ordering::SeqCst)));
        let most = most_at_once(&turns, &scratch("beside"), || {
            if INSIDE.fetch_add(1, Ordering::SeqCst) == 0 {
                // The first, measured with no other beside it.
                FREE.store(9 << 30, Ordering::SeqCst);
                std::thread::sleep(Duration::from_millis(700));
                FREE.store(10 << 30, Ordering::SeqCst);
            } else {
                // Each of the other two waits a while for the other to come in.
                for _ in 0..100 {
                    if INSIDE.load(Ordering::SeqCst) == 3 {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
        });
        assert_eq!(most, 2);
    }
}
