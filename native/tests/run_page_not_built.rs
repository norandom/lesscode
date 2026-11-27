//! Faults in code that is not built on this system
//! (`.kiro/specs/run-dashboard`, task 5.3). The pack names the conditions
//! that are certain for a system; a fault under one of them is counted apart.

use lesscode_core::adapters::driver::registry::embedded;
use lesscode_core::adapters::driver::sites::{file_sites, not_built_spans, within, Site};
use lesscode_core::dashboard::log::{Class, Classed, RunLog};
use lesscode_core::dashboard::text::write_report;
use lesscode_core::step::functions::FileFunctions;
use lesscode_core::step::plan_page::{class_at, STAGES};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// The pack of the language the examples below are written in.
const LANGUAGE: &str = "rust";

/// One function per way of standing under a condition. Each has one
/// condition and one write to plant faults in.
const SOURCE: &str = r#"
pub fn plain(a: i32) -> bool {
    if a > 3 {
        return true;
    }
    false
}

#[cfg(target_os = "linux")]
mod on_linux {
    pub fn inner(a: i32) -> bool {
        if a > 3 {
            return true;
        }
        false
    }
}

#[cfg(windows)]
#[inline]
pub fn on_windows(a: i32) -> bool {
    if a > 3 {
        return true;
    }
    false
}

#[cfg(any(target_os = "linux", windows))]
pub fn on_both(a: i32) -> bool {
    if a > 3 {
        return true;
    }
    false
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn only_an_attribute(a: i32) -> bool {
    if a > 3 {
        return true;
    }
    false
}
"#;

/// The functions whose sites lie in code that is not built on `system`.
fn not_built_on(system: &str) -> Vec<String> {
    let pack = embedded(LANGUAGE).unwrap();
    let spans = not_built_spans(pack, SOURCE, system).unwrap();
    let functions = FileFunctions::read(pack, "a.rs", SOURCE).unwrap();
    let sites = file_sites(pack, "a.rs", SOURCE).unwrap();
    assert!(sites.len() >= 5, "a site in every function: {sites:?}");
    let mut found: Vec<String> = sites
        .iter()
        .filter(|s| within(&spans, (s.start_line, s.start_column)))
        .map(|s| functions.owner((s.start_line, s.start_column)).to_owned())
        .collect();
    found.sort();
    found.dedup();
    found
}

#[test]
fn code_under_a_condition_that_is_false_on_a_system_is_found_and_no_other() {
    assert_eq!(not_built_on("windows"), ["inner"]);
    assert_eq!(not_built_on("linux"), ["on_windows"]);
    assert_eq!(not_built_on("macos"), ["inner", "on_windows"]);
    // A system the pack does not name: nothing is known, so nothing is found.
    assert!(not_built_on("freebsd").is_empty());
}

#[test]
fn a_pack_without_such_conditions_finds_nothing() {
    let pack = embedded("python").unwrap();
    let spans = not_built_spans(pack, "def f(a):\n    return a\n", "windows").unwrap();
    assert!(spans.is_empty());
}

fn site(line: usize) -> Site {
    let pack = embedded(LANGUAGE).unwrap();
    let sites = file_sites(pack, "a.rs", SOURCE).unwrap();
    let found = sites.into_iter().find(|s| s.start_line == line);
    found.unwrap_or_else(|| panic!("no site on line {line}"))
}

#[test]
fn a_fault_in_such_code_is_counted_apart_and_protects_or_exposes_nothing() {
    let pack = embedded(LANGUAGE).unwrap();
    let spans = not_built_spans(pack, SOURCE, "windows").unwrap();
    let (plain, inner) = (site(3), site(12));
    assert_eq!(class_at(&spans, &plain, "survived"), Class::Unnoticed);
    // Whatever the run made of it: every test passes when the code is not built.
    assert_eq!(class_at(&spans, &inner, "survived"), Class::NotCompiledHere);
    assert_eq!(class_at(&spans, &inner, "invalid"), Class::NotCompiledHere);

    let mut log = RunLog::new("step plan", "0", STAGES, 0);
    log.expect("a.rs", Some("inner"), 2);
    log.expect("a.rs", Some("plain"), 1);
    log.begin("faults", Some(3), 1);
    let fault = |name: &str, at: &Site, class: &str| Classed {
        file: "a.rs".into(),
        function: Some(name.into()),
        line: at.start_line as u64,
        operator: "negate".into(),
        text: at.text.clone(),
        class: class_at(&spans, at, class),
    };
    log.classed(fault("inner", &inner, "survived"), 2);
    log.classed(fault("inner", &inner, "invalid"), 3);
    log.classed(fault("plain", &plain, "survived"), 4);
    let shown = serde_json::to_value(log.snapshot()).unwrap();
    let faults = &shown["faults"];
    assert_eq!(faults["not_compiled_here"], 2);
    assert_eq!(
        (&faults["unnoticed"], &faults["not_built"]),
        (&1.into(), &0.into())
    );
    assert_eq!(faults["done"], 3);
    // The example is the first fault that says something.
    assert_eq!(faults["example"]["function"], "plain");
    let functions = faults["functions"].as_array().unwrap();
    let of = |name: &str| functions.iter().find(|f| f["function"] == name).unwrap();
    assert_eq!(of("inner")["not_compiled_here"], 2);
    assert_eq!(
        of("inner")["unprotected"],
        false,
        "nothing is known about it"
    );
    assert_eq!(of("inner")["left"], 0);
    assert_eq!(of("plain")["unprotected"], true);
}

#[test]
fn a_run_without_such_faults_says_nothing_about_them() {
    let mut log = RunLog::new("step plan", "0", STAGES, 0);
    let plain = site(3);
    log.classed(
        Classed {
            file: "a.rs".into(),
            function: Some("plain".into()),
            line: 3,
            operator: "negate".into(),
            text: plain.text,
            class: class_at(&[], &site(3), "killed"),
        },
        1,
    );
    let shown = serde_json::to_value(log.snapshot()).unwrap();
    assert_eq!(shown["faults"]["not_compiled_here"], Value::Null);
}

/// The recorded run of R164 on the renderer of another project, told to the
/// page again as a run on Windows: 268 faults, of which 66 lie in code that
/// is built on Linux only or not on Windows. Needs the run's record and the sources of its
/// commit, which are not in this repository:
/// `LESSCODE_RECORDED_RUN` is the run's `plan-run.json`, `LESSCODE_RECORDED_SOURCE`
/// the root its file names start from, `LESSCODE_RECORDED_PAGE` where the
/// report is written.
#[test]
#[ignore = "needs the record of a run that is not in the repository"]
fn the_recorded_run_on_the_renderer_shows_its_faults_in_linux_only_code_apart() {
    let named = |name: &str| {
        PathBuf::from(std::env::var(name).unwrap_or_else(|_| panic!("{name} is not set")))
    };
    let run: Value =
        serde_json::from_slice(&std::fs::read(named("LESSCODE_RECORDED_RUN")).unwrap()).unwrap();
    let source = named("LESSCODE_RECORDED_SOURCE");
    let pack = embedded(run["language"].as_str().unwrap()).unwrap();
    let mutants = run["mutants"].as_array().unwrap();
    let (mut spans, mut functions) = (BTreeMap::new(), BTreeMap::new());
    let mut log = RunLog::new("step plan", env!("CARGO_PKG_VERSION"), STAGES, 0);
    // The record does not keep the commands; the run had a test command.
    log.set_commands(Vec::new(), true);
    let at = |m: &Value| {
        let line = m["site"]["start_line"].as_u64().unwrap() as usize;
        (line, m["site"]["start_column"].as_u64().unwrap() as usize)
    };
    for m in mutants {
        let file = m["site"]["file"].as_str().unwrap();
        if !spans.contains_key(file) {
            let text = std::fs::read_to_string(source.join(file)).unwrap();
            spans.insert(
                file.to_owned(),
                not_built_spans(pack, &text, "windows").unwrap(),
            );
            functions.insert(
                file.to_owned(),
                FileFunctions::read(pack, file, &text).unwrap(),
            );
        }
        log.expect(file, Some(functions[file].owner(at(m))), 1);
    }
    log.begin("faults", Some(mutants.len() as u64), 1);
    let mut apart: BTreeMap<(String, String), u64> = BTreeMap::new();
    for (number, m) in mutants.iter().enumerate() {
        let file = m["site"]["file"].as_str().unwrap();
        let class = match within(&spans[file], at(m)) {
            true => Class::NotCompiledHere,
            false => match m["class"].as_str().unwrap() {
                "killed" => Class::Noticed,
                "survived" => Class::Unnoticed,
                "invalid" => Class::NotBuilt,
                "timeout" => Class::TimedOut,
                _ => Class::NotJudged,
            },
        };
        if class == Class::NotCompiledHere {
            *apart
                .entry((file.to_owned(), m["class"].as_str().unwrap().to_owned()))
                .or_insert(0) += 1;
        }
        log.classed(
            Classed {
                file: file.to_owned(),
                function: Some(functions[file].owner(at(m)).to_owned()),
                line: at(m).0 as u64,
                operator: m["operator"].as_str().unwrap().to_owned(),
                text: m["site"]["text"].as_str().unwrap().to_owned(),
                class,
            },
            2 + number as u64,
        );
    }
    log.end("faults", 300);
    log.finish(
        "The plan is written",
        "A recorded run, told again.",
        0,
        Vec::new(),
        301,
    );
    let snapshot = log.snapshot();
    println!("apart, by file and by what the run made of them: {apart:?}");
    println!(
        "noticed {}, unnoticed {}, not built {}, apart {:?}, of {}",
        snapshot.faults.noticed,
        snapshot.faults.unnoticed,
        snapshot.faults.not_built,
        snapshot.faults.not_compiled_here,
        snapshot.faults.done
    );
    write_report(&named("LESSCODE_RECORDED_PAGE"), &snapshot).unwrap();
    assert_eq!(mutants.len(), 268);
    // R164 counted 64 by the lines of two modules. Two more are single
    // statements under a condition, which that count took for built code
    // that did not build.
    assert_eq!(snapshot.faults.not_compiled_here, Some(66));
    let rest = &snapshot.faults;
    assert_eq!((rest.noticed, rest.unnoticed, rest.not_built), (72, 130, 0));
    let by_class = |class: &str| {
        apart
            .iter()
            .filter(|((_, c), _)| c == class)
            .map(|(_, n)| n)
            .sum::<u64>()
    };
    assert_eq!((by_class("survived"), by_class("invalid")), (63, 3));
}
