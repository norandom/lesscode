//! The batch runner: plants each mutant in a working copy, runs the
//! project's own commands and classifies the result. A run is void when the
//! unmutated tests do not pass or a mutant could not be run.

pub use super::classes::{classify, ERROR, INVALID, KILLED, SURVIVED, TIMEOUT};
use super::config::RunConfig;
pub use super::first_run::TESTS_FAIL;
use super::first_run::{self, tail, Turns};
use super::interrupt;
use super::page_watch::{tell, Event};
use super::process::{run, Finished};
use super::sites::Site;
use super::store::{Store, Stored};
use super::sync;
use super::workspace::Workspace;
use serde::Serialize;
use std::collections::VecDeque;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

/// In a command's arguments: replaced by the step's time limit in seconds.
pub const LIMIT_PLACEHOLDER: &str = "{limit_s}";

/// One mutant to run.
#[derive(Debug, Clone)]
pub struct Job {
    pub number: usize,
    pub site: Site,
    pub operator: String,
    pub text: String,
    pub sha256: String,
    /// The store key of the mutant.
    pub key: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MutantResult {
    pub number: usize,
    pub site: Site,
    pub operator: String,
    pub sha256: String,
    pub class: String,
    pub reason: Option<String>,
    pub exit_code: Option<i32>,
    pub seconds: f64,
    pub worker: Option<usize>,
    /// The run whose outcome was reused, if the mutant was not run now.
    pub reused_from: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct WorkerReport {
    pub worker: usize,
    /// A file restored from an interrupted earlier run.
    pub recovered: Option<String>,
    /// Seconds of each unmutated step before the mutants.
    pub baseline_s: Vec<f64>,
    pub limits_s: Vec<u64>,
    pub mutants_run: usize,
    pub closing_baseline_passed: Option<bool>,
    /// The step that failed in the closing run, and the end of its output.
    pub closing_failure: Option<String>,
    /// Why the worker stopped early, if it did.
    pub stopped: Option<String>,
    /// What the first attempt of the unmutated run said, where a second was made.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retried: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BatchOutput {
    pub mutants: Vec<MutantResult>,
    pub workers: Vec<WorkerReport>,
    /// Why nothing in this run may be relied on, if so.
    pub void: Option<String>,
}

fn steps(config: &RunConfig) -> Vec<(&'static str, &Vec<String>)> {
    let build = config.build.as_ref().map(|command| ("build", command));
    build.into_iter().chain([("test", &config.test)]).collect()
}

/// Run the steps in order; a step that does not pass ends the sequence.
fn run_steps(
    config: &RunConfig,
    tree: &Path,
    limits: &[Duration],
) -> Vec<(&'static str, Finished)> {
    let mut done = Vec::new();
    for ((name, command), limit) in steps(config).into_iter().zip(limits) {
        // A command that carries its own limit (work in a container cannot be
        // killed from outside) gets the limit in its arguments, and a margin
        // before the driver gives up on it.
        let seconds = limit.as_secs().to_string();
        let inner = command.iter().any(|a| a.contains(LIMIT_PLACEHOLDER));
        let filled: Vec<String> = command
            .iter()
            .map(|a| a.replace(LIMIT_PLACEHOLDER, &seconds))
            .collect();
        let outer = *limit + Duration::from_secs(if inner { 15 } else { 0 });
        let finished = run(&filled, tree, &config.env, outer);
        let passed = finished.passed();
        done.push((name, finished));
        if !passed {
            break;
        }
    }
    done
}

/// What the workers share.
struct Shared<'a> {
    config: &'a RunConfig,
    work: &'a Path,
    progress: Mutex<Progress>,
    store: Option<&'a Store>,
    run_id: &'a str,
    turns: Turns,
}

/// How far the batch is.
#[derive(Default)]
struct Progress {
    queue: VecDeque<Job>,
    results: Vec<MutantResult>,
    /// A failed restoration or a stop from outside: the run ends and is void.
    fatal: Option<String>,
}

impl Progress {
    /// The next mutant to run; none once the run has stopped.
    fn next(&mut self) -> Option<Job> {
        if self.fatal.is_some() {
            return None;
        }
        self.queue.pop_front()
    }
}

impl Shared<'_> {
    fn progress(&self) -> MutexGuard<'_, Progress> {
        self.progress.lock().unwrap()
    }

    fn open_copy(&self, report: &mut WorkerReport) -> Result<Workspace, String> {
        let name = format!("w{}", report.worker);
        let (home, config) = (self.work.join(&name), self.config);
        if home.exists() {
            let (space, recovered) = Workspace::open(self.work, &name)?;
            report.recovered = recovered;
            // A copy that is used again gets the files that changed since;
            // the setup runs again only then.
            if sync::refresh(&home, &config.source_root, &config.exclusions)? > 0 {
                self.set_up(&space)?;
            }
            return Ok(space);
        }
        let space = Workspace::create(&config.source_root, self.work, &name, &config.exclusions)?;
        sync::remember(&home, &config.source_root, &config.exclusions)?;
        self.set_up(&space)?;
        Ok(space)
    }

    /// The setup commands, in the copy.
    fn set_up(&self, space: &Workspace) -> Result<(), String> {
        let ceiling = Duration::from_secs(self.config.timeout_ceiling_s);
        for command in &self.config.setup {
            let finished = run(command, &space.tree, &self.config.env, ceiling);
            if !finished.passed() {
                return Err(format!(
                    "setup command {:?} did not succeed (exit {:?}{}): {}",
                    command.first(),
                    finished.exit_code,
                    finished
                        .launch_error
                        .as_deref()
                        .map(|e| format!(", {e}"))
                        .unwrap_or_default(),
                    tail(&finished.output)
                ));
            }
        }
        Ok(())
    }

    /// Plant, run, restore and classify one mutant; an error is retried once.
    fn one(
        &self,
        space: &Workspace,
        job: &Job,
        limits: &[Duration],
        worker: usize,
    ) -> Result<MutantResult, String> {
        let mut outcome = (ERROR, None, None, 0.0);
        for _attempt in 0..2 {
            space.plant(&job.site.file, &job.text)?;
            let planted = || run_steps(self.config, &space.tree, limits);
            let done = self.turns.later_run(worker, planted);
            space
                .restore()
                .inspect_err(|e| self.progress().fatal = Some(e.clone()))?;
            if interrupt::requested() {
                // The mutant's run was cut short: it has no class.
                self.progress().fatal = Some(interrupt::STOPPED.into());
                return Err(interrupt::STOPPED.into());
            }
            let (class, reason) = classify(self.config, &done);
            let seconds = done.iter().map(|(_, f)| f.seconds).sum();
            outcome = (
                class,
                reason,
                done.last().and_then(|(_, f)| f.exit_code),
                seconds,
            );
            if class != ERROR {
                break;
            }
        }
        let (class, reason, exit_code, seconds) = outcome;
        if let Some(store) = self.store {
            let stored = Stored {
                class: class.into(),
                reason: reason.clone(),
                exit_code,
                seconds,
                run: self.run_id.into(),
            };
            store.put(&job.key, &stored)?;
        }
        Ok(MutantResult {
            number: job.number,
            site: job.site.clone(),
            operator: job.operator.clone(),
            sha256: job.sha256.clone(),
            class: class.into(),
            reason,
            exit_code,
            seconds,
            worker: Some(worker),
            reused_from: None,
        })
    }

    fn worker(&self, worker: usize) -> WorkerReport {
        let mut report = WorkerReport {
            worker,
            ..WorkerReport::default()
        };
        let space = match self.open_copy(&mut report) {
            Ok(space) => space,
            Err(why) => {
                report.stopped = Some(why);
                return report;
            }
        };
        let ceiling =
            vec![Duration::from_secs(self.config.timeout_ceiling_s); steps(self.config).len()];
        let evidence = &self.config.runner_evidence_patterns;
        let unchanged = || run_steps(self.config, &space.tree, &ceiling);
        let first = self.turns.first_run(evidence, self.work, worker, unchanged);
        (report.baseline_s, report.stopped, report.retried) =
            (first.seconds, first.stopped, first.retried);
        tell(Event::Unchanged(&report));
        if report.stopped.is_some() {
            return report;
        }
        report.limits_s = report
            .baseline_s
            .iter()
            .map(|s| self.config.limit_s(*s))
            .collect();
        let limits: Vec<Duration> = report
            .limits_s
            .iter()
            .map(|s| Duration::from_secs(*s))
            .collect();
        loop {
            let Some(job) = self.progress().next() else {
                break;
            };
            match self.one(&space, &job, &limits, worker) {
                Ok(result) => {
                    tell(Event::Classed(&result));
                    self.progress().results.push(result);
                    report.mutants_run += 1;
                }
                Err(why) => {
                    self.progress().queue.push_front(job);
                    report.stopped = Some(why);
                    return report;
                }
            }
        }
        let closing = self.turns.later_run(worker, unchanged);
        report.closing_baseline_passed =
            Some(closing.len() == ceiling.len() && closing.iter().all(|(_, f)| f.passed()));
        report.closing_failure = failure(&closing);
        report
    }
}

/// The step that did not pass in a run of the unmutated commands, and the
/// end of its output.
fn failure(done: &[(&'static str, Finished)]) -> Option<String> {
    done.iter().find(|(_, f)| !f.passed()).map(|(step, f)| {
        let timed_out = if f.timed_out { ", timed out" } else { "" };
        let output = tail(&f.output);
        format!("{step} step, exit {:?}{timed_out}: {output}", f.exit_code)
    })
}

/// Why nothing in a run may be relied on, if so.
fn void_reason(reports: &[WorkerReport], left: usize, fatal: Option<String>) -> Option<String> {
    if fatal.is_some() {
        return fatal;
    }
    if reports.iter().all(|r| r.limits_s.is_empty()) {
        let reasons: Vec<&str> = reports
            .iter()
            .filter_map(|r| r.stopped.as_deref().map(first_run::sentence))
            .collect();
        return Some(format!(
            "no working copy could run the unmutated tests: {}",
            reasons.join("; ")
        ));
    }
    if left > 0 {
        return Some(format!("{left} mutants could not be run"));
    }
    let failed_after = reports
        .iter()
        .find(|r| r.closing_baseline_passed == Some(false))?;
    let detail = failed_after.closing_failure.as_deref().unwrap_or_default();
    Some(format!(
        "the unmutated tests no longer passed after the mutants (copy {}: {detail})",
        failed_after.worker
    ))
}

/// Run the jobs in `workers` copies under `work`. Outcomes found in the
/// store are reused and not run again.
pub fn run_batch(
    config: &RunConfig,
    work: &Path,
    jobs: Vec<Job>,
    store: Option<&Store>,
    workers: usize,
    run_id: &str,
) -> BatchOutput {
    let mut progress = Progress::default();
    for job in jobs {
        match store.and_then(|s| s.get(&job.key)) {
            Some(stored) => progress.results.push(MutantResult {
                number: job.number,
                site: job.site,
                operator: job.operator,
                sha256: job.sha256,
                class: stored.class,
                reason: stored.reason,
                exit_code: stored.exit_code,
                seconds: stored.seconds,
                worker: None,
                reused_from: Some(stored.run),
            }),
            None => progress.queue.push_back(job),
        }
    }
    let found = progress.results.iter();
    found.for_each(|reused| tell(Event::Classed(reused)));
    let shared = Shared {
        config,
        work,
        progress: Mutex::new(progress),
        store,
        run_id,
        turns: Turns::default(),
    };
    let reports: Vec<WorkerReport> = std::thread::scope(|scope| {
        let handles: Vec<_> = (1..=workers.max(1))
            .map(|id| {
                let shared = &shared;
                scope.spawn(move || shared.worker(id))
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("a worker does not panic"))
            .collect()
    });
    let progress = shared.progress.into_inner().unwrap();
    // A stop during setup or an unmutated run reaches no mutant; say so anyway.
    let stopped = interrupt::requested().then(|| interrupt::STOPPED.to_owned());
    let void = void_reason(&reports, progress.queue.len(), stopped.or(progress.fatal));
    let mut mutants = progress.results;
    mutants.sort_by_key(|m| m.number);
    BatchOutput {
        mutants,
        workers: reports,
        void,
    }
}
