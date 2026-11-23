//! The run page: what a run tells about itself, kept as the snapshot the
//! page shows (`.kiro/specs/run-dashboard`, design: RunLog). The page
//! explains and shows; it never judges. No input and no output
//! here: every moment is handed in as milliseconds since 1970, so that a
//! test can replay a run exactly.

use super::page_snapshot::{
    stamp, Copies, Dropped, Example, Faults, Snapshot, Stage, Status, Subject, Tally, Tests, Tool,
    FEW, SCHEMA,
};
use std::collections::BTreeMap;

/// A stage of a command, as the person would name it.
pub struct StagePlan {
    pub id: &'static str,
    pub title: &'static str,
    pub what: &'static str,
}

/// What became of a planted fault.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    Noticed,
    Unnoticed,
    NotBuilt,
    TimedOut,
    /// Its run failed twice; the run as a whole is then incomplete.
    NotJudged,
    /// It lies in code that is not built on this system, whatever its run
    /// gave.
    NotCompiledHere,
}

impl Class {
    fn in_words(self) -> &'static str {
        match self {
            Self::Noticed => "noticed: a test failed",
            Self::Unnoticed => "not noticed: every test passed",
            Self::NotBuilt => "did not build",
            Self::TimedOut => "ran over its time limit",
            Self::NotJudged => "could not be run",
            Self::NotCompiledHere => "lies in code that is not built on this system",
        }
    }
}

/// One planted fault that has been judged.
pub struct Classed {
    pub file: String,
    pub function: Option<String>,
    pub line: u64,
    pub operator: String,
    pub text: String,
    pub class: Class,
}

pub struct RunLog {
    /// The parts that are stated; the parts that are computed are filled in
    /// by `snapshot`.
    stated: Snapshot,
    started: u64,
    last: u64,
    begun: BTreeMap<String, u64>,
    /// The faults of the whole run, of each file and of each function.
    all: Tally,
    files: BTreeMap<String, Tally>,
    functions: BTreeMap<(String, String), Tally>,
}

impl RunLog {
    pub fn new(command: &str, version: &str, stages: &[StagePlan], now: u64) -> Self {
        let stage = |s: &StagePlan| Stage {
            id: s.id.to_owned(),
            title: s.title.to_owned(),
            what: s.what.to_owned(),
            status: Status::NotBegun,
            done: None,
            total: None,
            seconds: None,
            seconds_so_far: None,
            seconds_left: None,
            reason: None,
        };
        let stated = Snapshot {
            schema: SCHEMA,
            sequence: 0,
            tool: Tool {
                name: "lesscode-core".to_owned(),
                version: version.to_owned(),
            },
            command: command.to_owned(),
            started: stamp(now),
            heartbeat: stamp(now),
            ended: false,
            outcome: None,
            outcome_detail: None,
            exit_code: None,
            seconds: 0.0,
            subject: Subject::default(),
            copies: Copies::default(),
            commands: Vec::new(),
            writes_in_repository: Vec::new(),
            tests: Tests::default(),
            stages: stages.iter().map(stage).collect(),
            faults: Faults::default(),
            next: Vec::new(),
        };
        Self {
            stated,
            started: now,
            last: now,
            begun: BTreeMap::new(),
            all: Tally::default(),
            files: BTreeMap::new(),
            functions: BTreeMap::new(),
        }
    }

    fn changed(&mut self, now: u64) {
        self.last = self.last.max(now);
        self.stated.sequence += 1;
    }

    fn stage(&mut self, id: &str) -> Option<&mut Stage> {
        self.stated.stages.iter_mut().find(|s| s.id == id)
    }

    pub fn set_subject(&mut self, subject: Subject) {
        self.stated.subject = subject;
    }

    pub fn set_copies(&mut self, directory: &str, planned: u64) {
        self.stated.copies.directory = Some(directory.to_owned());
        self.stated.copies.planned = planned;
        self.stated.copies.working = planned;
    }

    /// The project's commands the run executes, and whether one of them
    /// runs tests.
    pub fn set_commands(&mut self, commands: Vec<Vec<String>>, test_command_given: bool) {
        self.stated.commands = commands;
        self.stated.tests.command_given = test_command_given;
    }

    pub fn set_writes(&mut self, files: Vec<String>) {
        self.stated.writes_in_repository = files;
    }

    /// The space the copies take, once someone has measured it.
    pub fn set_space(&mut self, bytes: u64) {
        self.stated.copies.bytes = Some(bytes);
    }

    /// Faults that will be planted in a file, and in a function where known.
    pub fn expect(&mut self, file: &str, function: Option<&str>, faults: u64) {
        self.stated.faults.total += faults;
        self.files.entry(file.to_owned()).or_default().left += faults;
        if let Some(function) = function {
            let key = (file.to_owned(), function.to_owned());
            self.functions.entry(key).or_default().left += faults;
        }
    }

    /// What a function calls and writes outside itself.
    pub fn reaches_out(&mut self, file: &str, function: &str, what: Vec<String>) {
        let key = (file.to_owned(), function.to_owned());
        self.functions.entry(key).or_default().reaches_out = what;
    }

    /// A stage begins; `total` is given where it has items to count.
    pub fn begin(&mut self, id: &str, total: Option<u64>, now: u64) {
        self.begun.insert(id.to_owned(), now);
        if let Some(stage) = self.stage(id) {
            stage.status = Status::GoingOn;
            stage.total = total;
            stage.done = total.map(|_| 0);
        }
        self.changed(now);
    }

    fn close(&mut self, id: &str, status: Status, reason: Option<&str>, now: u64) {
        let took = self.begun.get(id).map(|b| seconds(*b, now));
        if let Some(stage) = self.stage(id) {
            stage.status = status;
            stage.seconds = took;
            stage.reason = reason.map(str::to_owned);
        }
        self.changed(now);
    }

    pub fn end(&mut self, id: &str, now: u64) {
        self.close(id, Status::Ended, None, now);
    }

    pub fn fail(&mut self, id: &str, reason: &str, now: u64) {
        self.close(id, Status::Failed, Some(reason), now);
    }

    /// The unchanged tests ended in one copy: how many ran, where that is
    /// known, or why the copy drops out.
    pub fn unchanged_tests(&mut self, copy: u64, outcome: Result<Option<u64>, String>, now: u64) {
        let (copies, tests) = (&mut self.stated.copies, &mut self.stated.tests);
        match outcome {
            Ok(ran) => {
                tests.passed_unchanged = Some(true);
                tests.tests_run = ran.or(tests.tests_run);
            }
            Err(reason) => {
                copies.working = copies.working.saturating_sub(1);
                if copies.working == 0 {
                    tests.passed_unchanged = Some(false);
                    tests.reason = Some(reason.clone());
                }
                copies.dropped.push(Dropped {
                    copy,
                    at: stamp(now),
                    reason,
                });
            }
        }
        self.changed(now);
    }

    pub fn classed(&mut self, fault: Classed, now: u64) {
        count(&mut self.all, fault.class);
        count(
            self.files.entry(fault.file.clone()).or_default(),
            fault.class,
        );
        if let Some(function) = &fault.function {
            let key = (fault.file.clone(), function.clone());
            count(self.functions.entry(key).or_default(), fault.class);
        }
        // The first fault that is judged becomes the example the page explains
        // with. One in code that is not built here explains nothing.
        if fault.class != Class::NotCompiledHere {
            let _ = self.stated.faults.example.get_or_insert_with(|| Example {
                file: fault.file.clone(),
                function: fault.function.clone(),
                line: fault.line,
                what_was_changed: fault.operator.clone(),
                text: fault.text.clone(),
                outcome: fault.class.in_words().to_owned(),
            });
        }
        // The stage that counts faults is the one going on with a total.
        let counting = |s: &&mut Stage| s.status == Status::GoingOn && s.total.is_some();
        if let Some(stage) = self.stated.stages.iter_mut().find(counting) {
            stage.done = stage.done.map(|d| d + 1);
        }
        self.changed(now);
    }

    /// Faults judged so far, and faults in all.
    pub fn counted(&self) -> (u64, u64) {
        (self.all.done(), self.stated.faults.total)
    }

    /// A sign of life while nothing else changes.
    pub fn beat(&mut self, now: u64) {
        self.changed(now);
    }

    /// The run has ended. Nothing is shown as going on after this: a stage
    /// that was going on ended with the run, or failed with it; a stage that
    /// never began was left out.
    pub fn finish(
        &mut self,
        outcome: &str,
        detail: &str,
        exit_code: u8,
        next: Vec<String>,
        now: u64,
    ) {
        let open: Vec<String> = self.stated.stages.iter().map(|s| s.id.clone()).collect();
        for id in open {
            let status = self.stage(&id).map(|s| s.status);
            match status {
                Some(Status::GoingOn) if exit_code == 0 => self.end(&id, now),
                Some(Status::GoingOn) => self.fail(&id, detail, now),
                Some(Status::NotBegun) => self.close(&id, Status::LeftOut, None, now),
                _ => {}
            }
        }
        self.stated.ended = true;
        self.stated.outcome = Some(outcome.to_owned());
        self.stated.outcome_detail = Some(detail.to_owned());
        self.stated.exit_code = Some(exit_code);
        self.stated.next = next;
        self.changed(now);
    }

    pub fn snapshot(&self) -> Snapshot {
        let mut shown = self.stated.clone();
        shown.heartbeat = stamp(self.last);
        shown.seconds = seconds(self.started, self.last);
        shown.faults = Faults {
            done: self.all.done(),
            noticed: self.all.noticed,
            unnoticed: self.all.unnoticed,
            not_built: self.all.not_built,
            timed_out: self.all.timed_out,
            not_judged: self.all.not_judged,
            not_compiled_here: Some(self.all.not_compiled_here).filter(|n| *n > 0),
            ..shown.faults
        };
        for stage in shown
            .stages
            .iter_mut()
            .filter(|s| s.status == Status::GoingOn)
        {
            let so_far = self.begun.get(&stage.id).map(|b| seconds(*b, self.last));
            stage.seconds_so_far = so_far;
            stage.seconds_left = rest(stage.done, stage.total, so_far);
        }
        let file = |(file, tally): (&String, &Tally)| settled(tally, file, None);
        shown.faults.files = self.files.iter().map(file).collect();
        let function =
            |((file, name), tally): (&(String, String), &Tally)| settled(tally, file, Some(name));
        shown.faults.functions = self.functions.iter().map(function).collect();
        shown
    }
}

/// One more judged fault in a tally.
fn count(tally: &mut Tally, class: Class) {
    tally.left = tally.left.saturating_sub(1);
    match class {
        Class::Noticed => tally.noticed += 1,
        Class::Unnoticed => tally.unnoticed += 1,
        Class::NotBuilt => tally.not_built += 1,
        Class::TimedOut => tally.timed_out += 1,
        Class::NotJudged => tally.not_judged += 1,
        Class::NotCompiledHere => tally.not_compiled_here += 1,
    }
}

fn seconds(from: u64, to: u64) -> f64 {
    to.saturating_sub(from) as f64 / 1_000.0
}

/// The time for the rest of a counted stage, from the items done so far.
/// Nothing is said before `FEW` items are done.
fn rest(done: Option<u64>, total: Option<u64>, so_far: Option<f64>) -> Option<f64> {
    let (done, total, so_far) = (done?, total?, so_far?);
    (done >= FEW && total > done).then(|| so_far / done as f64 * (total - done) as f64)
}

/// A tally as it is shown: named, and unprotected when every fault has been
/// run and none was noticed. Faults in code that is not built here say
/// nothing about protection.
fn settled(tally: &Tally, file: &str, function: Option<&String>) -> Tally {
    Tally {
        file: file.to_owned(),
        function: function.cloned(),
        unprotected: tally.left == 0 && tally.noticed == 0 && tally.judged() > 0,
        ..tally.clone()
    }
}
