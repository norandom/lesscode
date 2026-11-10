//! The batch runner: plants each mutant in a working copy, runs the
//! project's own commands and classifies the result. A run is void when the
//! unmutated tests do not pass or a mutant could not be run.

use super::config::RunConfig;
use super::process::{run, Finished};
use super::sites::Site;
use super::store::{Store, Stored};
use super::workspace::Workspace;
use serde::Serialize;
use std::collections::VecDeque;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

pub const KILLED: &str = "killed";
pub const SURVIVED: &str = "survived";
pub const INVALID: &str = "invalid";
pub const TIMEOUT: &str = "timeout";
/// The machine or the command failed; this says nothing about the mutant.
pub const ERROR: &str = "error";

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
    /// Why the worker stopped early, if it did.
    pub stopped: Option<String>,
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
        let finished = run(command, tree, &config.env, *limit);
        let passed = finished.passed();
        done.push((name, finished));
        if !passed {
            break;
        }
    }
    done
}

fn contains_any(output: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|p| output.contains(p.as_str()))
}

/// The class of a mutant from what its steps did, with the reason where the
/// class alone does not say it.
pub fn classify(
    config: &RunConfig,
    done: &[(&'static str, Finished)],
) -> (&'static str, Option<String>) {
    for (name, finished) in done {
        if let Some(why) = &finished.launch_error {
            return (ERROR, Some(format!("{name} could not start: {why}")));
        }
        if contains_any(&finished.output, &config.error_output_patterns) {
            return (ERROR, Some(format!("resource failure during {name}")));
        }
        if finished.timed_out {
            return (TIMEOUT, Some(format!("{name} reached its time limit")));
        }
        if *name == "build" && finished.exit_code != Some(0) {
            return (INVALID, Some("did not build".into()));
        }
    }
    let Some((_, test)) = done.last().filter(|(name, _)| *name == "test") else {
        return (ERROR, Some("the test step did not run".into()));
    };
    if test.exit_code == Some(0) {
        return (SURVIVED, None);
    }
    let cannot_load = test
        .exit_code
        .is_some_and(|code| config.invalid_exit_codes.contains(&code));
    if cannot_load || contains_any(&test.output, &config.invalid_output_patterns) {
        return (
            INVALID,
            Some("the tests could not be built or loaded".into()),
        );
    }
    let evidence = &config.runner_evidence_patterns;
    if !evidence.is_empty() && !contains_any(&test.output, evidence) {
        return (ERROR, Some("no sign that the test runner ran".into()));
    }
    (KILLED, None)
}

/// What the workers share.
struct Shared<'a> {
    config: &'a RunConfig,
    work: &'a Path,
    queue: Mutex<VecDeque<Job>>,
    results: Mutex<Vec<MutantResult>>,
    /// A failed restoration: the run stops and is void.
    fatal: Mutex<Option<String>>,
    store: Option<&'a Store>,
    run_id: &'a str,
}

impl Shared<'_> {
    fn open_copy(&self, report: &mut WorkerReport) -> Result<Workspace, String> {
        let name = format!("w{}", report.worker);
        if self.work.join(&name).exists() {
            let (space, recovered) = Workspace::open(self.work, &name)?;
            report.recovered = recovered;
            return Ok(space);
        }
        let space = Workspace::create(
            &self.config.source_root,
            self.work,
            &name,
            &self.config.exclusions,
        )?;
        let ceiling = Duration::from_secs(self.config.timeout_ceiling_s);
        for command in &self.config.setup {
            let finished = run(command, &space.tree, &self.config.env, ceiling);
            if !finished.passed() {
                return Err(format!("setup command {command:?} did not succeed"));
            }
        }
        Ok(space)
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
            let done = run_steps(self.config, &space.tree, limits);
            space
                .restore()
                .inspect_err(|e| *self.fatal.lock().unwrap() = Some(e.clone()))?;
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
        let baseline = run_steps(self.config, &space.tree, &ceiling);
        report.baseline_s = baseline.iter().map(|(_, f)| f.seconds).collect();
        if baseline.len() < ceiling.len() || !baseline.iter().all(|(_, f)| f.passed()) {
            report.stopped = Some("the unmutated tests do not pass in this copy".into());
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
            if self.fatal.lock().unwrap().is_some() {
                break;
            }
            let Some(job) = self.queue.lock().unwrap().pop_front() else {
                break;
            };
            match self.one(&space, &job, &limits, worker) {
                Ok(result) => {
                    self.results.lock().unwrap().push(result);
                    report.mutants_run += 1;
                }
                Err(why) => {
                    self.queue.lock().unwrap().push_front(job);
                    report.stopped = Some(why);
                    return report;
                }
            }
        }
        let closing = run_steps(self.config, &space.tree, &ceiling);
        report.closing_baseline_passed =
            Some(closing.len() == ceiling.len() && closing.iter().all(|(_, f)| f.passed()));
        report
    }
}

/// Why nothing in a run may be relied on, if so.
fn void_reason(reports: &[WorkerReport], left: usize, fatal: Option<String>) -> Option<String> {
    if fatal.is_some() {
        return fatal;
    }
    if reports.iter().all(|r| r.limits_s.is_empty()) {
        let reasons: Vec<&str> = reports
            .iter()
            .filter_map(|r| r.stopped.as_deref())
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
        .any(|r| r.closing_baseline_passed == Some(false));
    failed_after.then(|| "the unmutated tests no longer passed after the mutants".to_owned())
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
    let mut reused = Vec::new();
    let mut queue = VecDeque::new();
    for job in jobs {
        match store.and_then(|s| s.get(&job.key)) {
            Some(stored) => reused.push(MutantResult {
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
            None => queue.push_back(job),
        }
    }
    let shared = Shared {
        config,
        work,
        queue: Mutex::new(queue),
        results: Mutex::new(reused),
        fatal: Mutex::new(None),
        store,
        run_id,
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
    let left = shared.queue.lock().unwrap().len();
    let fatal = shared.fatal.lock().unwrap().clone();
    let void = void_reason(&reports, left, fatal);
    let mut mutants = shared.results.into_inner().unwrap();
    mutants.sort_by_key(|m| m.number);
    BatchOutput {
        mutants,
        workers: reports,
        void,
    }
}
