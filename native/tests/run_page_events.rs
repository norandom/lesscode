//! What the batch runner tells while it runs, against its own receipt
//! (`.kiro/specs/run-dashboard`, task 2.1). The project's "test command" is
//! the example `toy_check`, as in the driver's own suites.

use lesscode_core::adapters::driver::config::RunConfig;
use lesscode_core::adapters::driver::page_watch::{Event, Watch, Watching};
use lesscode_core::adapters::driver::registry::embedded;
use lesscode_core::adapters::driver::run_command::{run_run, RunArgs};
use lesscode_core::step::plan_page::PlanWatch;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const APP: &str = "def step(state, a):\n    if a > 3:\n        raise ValueError(\"big\")\n    state.total = a\n    return a\n\n\ndef other(a):\n    return a + 1\n";

fn toy() -> String {
    let test_binary = std::env::current_exe().unwrap();
    let profile = test_binary.parent().unwrap().parent().unwrap();
    let name = format!("toy_check{}", std::env::consts::EXE_SUFFIX);
    let path = profile.join("examples").join(name);
    assert!(path.is_file(), "not run: {} was not built", path.display());
    path.to_string_lossy().into_owned()
}

/// A toy project with a run configuration; `passes` says whether its test
/// passes on the unchanged code.
fn project(name: &str, passes: bool) -> (PathBuf, PathBuf) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("run_page_events_{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("project/src")).unwrap();
    fs::write(dir.join("project/src/app.py"), APP).unwrap();
    let expected = if passes {
        "state.total = a"
    } else {
        "this line is not in the file"
    };
    let test = [
        toy().as_str(),
        "--file",
        "src/app.py",
        "--print",
        "TOY RAN",
        "--expect",
        "if a > 3:",
        "--expect",
        expected,
    ]
    .map(str::to_owned);
    let config = json!({
        "language": "python", "source_root": "project", "files": ["src/app.py"], "test": test,
        "timeout_floor_s": 10, "timeout_ceiling_s": 60, "workers": 2,
    });
    let path = dir.join("run.json");
    fs::write(&path, config.to_string()).unwrap();
    (dir, path)
}

fn arguments(dir: &Path, config: &Path) -> RunArgs {
    RunArgs {
        config: config.to_path_buf(),
        work: dir.join("work"),
        workers: None,
        annotations: None,
        no_reuse: false,
        only: Vec::new(),
        out: dir.join("receipt.json"),
    }
}

fn receipt(dir: &Path) -> Value {
    serde_json::from_slice(&fs::read(dir.join("receipt.json")).unwrap()).unwrap()
}

/// Keeps what it is told, as text.
#[derive(Default)]
struct Recorder {
    planned: Mutex<Vec<usize>>,
    unchanged: Mutex<Vec<(usize, Option<String>)>>,
    classed: Mutex<Vec<(usize, String)>>,
}

impl Watch for Recorder {
    fn tell(&self, event: Event<'_>) {
        match event {
            Event::Planned(jobs) => self
                .planned
                .lock()
                .unwrap()
                .extend(jobs.iter().map(|j| j.number)),
            Event::Unchanged(report) => self
                .unchanged
                .lock()
                .unwrap()
                .push((report.worker, report.stopped.clone())),
            Event::Classed(result) => self
                .classed
                .lock()
                .unwrap()
                .push((result.number, result.class.clone())),
        }
    }
}

fn classes(receipt: &Value) -> BTreeMap<usize, String> {
    let mutants = receipt["mutants"].as_array().unwrap();
    mutants
        .iter()
        .map(|m| {
            (
                m["number"].as_u64().unwrap() as usize,
                m["class"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

#[test]
fn what_is_told_matches_the_receipt_fault_by_fault() {
    let (dir, config) = project("match", true);
    let recorder = Arc::new(Recorder::default());
    let watching = Watching::begin(recorder.clone());
    let code = run_run(arguments(&dir, &config)).unwrap();
    drop(watching);
    assert_eq!(code, 0);
    let receipt = receipt(&dir);
    let written = classes(&receipt);
    assert!(
        written.len() >= 4,
        "the toy has faults to plant: {written:?}"
    );
    let told: BTreeMap<usize, String> = recorder.classed.lock().unwrap().iter().cloned().collect();
    assert_eq!(
        told, written,
        "every fault once, with the class of the receipt"
    );
    assert_eq!(
        recorder.classed.lock().unwrap().len(),
        written.len(),
        "none told twice"
    );
    let mut planned = recorder.planned.lock().unwrap().clone();
    planned.sort_unstable();
    assert_eq!(planned, written.keys().copied().collect::<Vec<_>>());
    let unchanged = recorder.unchanged.lock().unwrap().clone();
    assert_eq!(
        unchanged.len(),
        receipt["workers"].as_array().unwrap().len()
    );
    assert!(unchanged.iter().all(|(_, stopped)| stopped.is_none()));

    // Found from the earlier run: told again, and still once each.
    let second = Arc::new(Recorder::default());
    let watching = Watching::begin(second.clone());
    run_run(arguments(&dir, &config)).unwrap();
    drop(watching);
    let told: BTreeMap<usize, String> = second.classed.lock().unwrap().iter().cloned().collect();
    assert_eq!(told, written);
    assert_eq!(second.classed.lock().unwrap().len(), written.len());
}

#[test]
fn copies_whose_tests_fail_are_told_with_the_reason() {
    let (dir, config) = project("void", false);
    let recorder = Arc::new(Recorder::default());
    let watching = Watching::begin(recorder.clone());
    let failed = run_run(arguments(&dir, &config)).unwrap_err();
    drop(watching);
    assert!(failed.contains("void"), "{failed}");
    let receipt = receipt(&dir);
    let unchanged = recorder.unchanged.lock().unwrap().clone();
    let stopped: Vec<String> = receipt["workers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["stopped"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(unchanged.len(), stopped.len(), "each dropped copy is told");
    for (_, reason) in &unchanged {
        assert!(
            stopped.contains(reason.as_ref().unwrap()),
            "the reason is the receipt's"
        );
    }
    assert!(
        recorder.classed.lock().unwrap().is_empty(),
        "no fault was run"
    );
}

#[test]
fn nothing_is_told_without_a_listener_and_the_receipt_is_the_same() {
    let (dir, config) = project("silent", true);
    run_run(arguments(&dir, &config)).unwrap();
    let alone = classes(&receipt(&dir));
    let (dir, config) = project("heard", true);
    let watching = Watching::begin(Arc::new(Recorder::default()));
    run_run(arguments(&dir, &config)).unwrap();
    drop(watching);
    assert_eq!(
        classes(&receipt(&dir)),
        alone,
        "a listener changes no class"
    );
}

#[test]
fn the_plan_listener_counts_each_fault_under_its_function() {
    let (dir, config_path) = project("plan", true);
    let config = RunConfig::read(&config_path).unwrap();
    let pack = embedded(&config.language).unwrap();
    let watch = Arc::new(PlanWatch::new(&config, pack, &dir.join("work"), 2).unwrap());
    let watching = Watching::begin(watch.clone());
    run_run(arguments(&dir, &config_path)).unwrap();
    drop(watching);
    let receipt = receipt(&dir);
    let written = classes(&receipt);
    let count = |class: &str| written.values().filter(|c| c.as_str() == class).count() as u64;
    let shown = watch.snapshot();
    assert_eq!(shown.faults.total, written.len() as u64);
    assert_eq!(shown.faults.done, written.len() as u64);
    assert_eq!(shown.faults.noticed, count("killed"));
    assert_eq!(shown.faults.unnoticed, count("survived"));
    assert_eq!(shown.faults.not_built, count("invalid"));
    assert_eq!(shown.tests.passed_unchanged, Some(true));
    assert_eq!(shown.copies.working, 2);
    assert!(
        shown.faults.example.is_some(),
        "the first judged fault is the example"
    );
    // Every fault lies in a function of the file, and the tallies add up.
    let names: Vec<&str> = shown
        .faults
        .functions
        .iter()
        .filter_map(|f| f.function.as_deref())
        .collect();
    assert!(names.contains(&"step"), "{names:?}");
    let per_function: u64 = shown
        .faults
        .functions
        .iter()
        .map(|f| f.noticed + f.unnoticed + f.not_built + f.timed_out + f.not_judged)
        .sum();
    assert_eq!(per_function, written.len() as u64);
    assert!(shown.faults.functions.iter().all(|f| f.left == 0));
    let stages: Vec<String> = shown
        .stages
        .iter()
        .map(|s| format!("{}: {:?}", s.id, s.status))
        .collect();
    assert_eq!(
        stages,
        ["prepare: Ended", "faults: Ended", "close: GoingOn"],
        "the run has not been ended yet"
    );
    let commands = shown.commands;
    assert_eq!(
        commands.len(),
        1,
        "the test command, and no setup or build in this configuration"
    );
}
