//! The border between lesscode's core and its dashboard (`.kiro/specs/report-set`, task 1.2;
//! design, "The border between core and dashboard"). Read from the source tree:
//!
//! - core names the dashboard only in the two runners, which tell its watch what happens;
//! - the dashboard does not name `step`: `step` decides, the dashboard shows;
//! - no file of the run page is back among the driver's files.

use std::fs;
use std::path::{Path, PathBuf};

/// The files of core that may name the dashboard, and why.
const CROSSINGS: [&str; 2] = [
    // Both tell the dashboard's watch what happens in a run.
    "adapters/driver/run_command.rs",
    "adapters/driver/runner.rs",
];

fn src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Every Rust file below `dir`, as a path relative to `src` with forward slashes.
fn rust_files(dir: &Path) -> Vec<(String, String)> {
    let mut found = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let name = path
                    .strip_prefix(src())
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                found.push((name, fs::read_to_string(&path).unwrap()));
            }
        }
    }
    found.sort();
    found
}

/// Whether a file of core is the dashboard's or the commands' own.
fn is_core(name: &str) -> bool {
    !(name.starts_with("dashboard/") || name.starts_with("step/") || name == "lib.rs")
}

#[test]
fn core_names_the_dashboard_only_in_the_two_runners() {
    let all = rust_files(&src());
    let core: Vec<_> = all.iter().filter(|(name, _)| is_core(name)).collect();
    assert!(core.len() > 50, "the scan must have seen the core");
    let naming: Vec<&str> = core
        .iter()
        .filter(|(_, text)| text.contains("dashboard::"))
        .map(|(name, _)| name.as_str())
        .collect();
    assert_eq!(
        naming, CROSSINGS,
        "only the two runners of core may name the dashboard"
    );
}

#[test]
fn the_dashboard_does_not_name_step() {
    let dashboard = rust_files(&src().join("dashboard"));
    assert!(
        dashboard.len() >= 10,
        "the scan must have seen the dashboard"
    );
    for (name, text) in &dashboard {
        assert!(
            !text.contains("crate::step"),
            "{name} names step: step decides, the dashboard shows"
        );
    }
}

#[test]
fn no_file_of_the_run_page_is_among_the_drivers_files() {
    for entry in fs::read_dir(src().join("adapters/driver")).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().into_owned();
        assert!(
            !name.starts_with("page_"),
            "adapters/driver/{name}: the dashboard's files are in src/dashboard"
        );
    }
}
