//! What `step protect begin` says before anything runs: the functions that
//! were chosen with the faults no test noticed, the functions with
//! unnoticed faults that are reported, and what running the faults again will cost, from
//! the seconds of the plan (requirements 1.1, 1.2, 1.7 and 5.9), and what one
//! copy of the plan takes on disk.

use super::protect_stage::{slashed, Chosen, Reported, Stage};
use serde_json::Value;
use std::path::{Path, PathBuf};

const WHOLE: &str = "The whole scope was taken: every function of the plan with an unnoticed fault. Name functions with --function to narrow it.";

/// Where the receipt of a plan's run lies. The plan names it as the path it
/// was written to; where that was relative, it is also looked for beside
/// the plan.
pub fn run_of(plan_path: &Path, plan: &Value) -> PathBuf {
    let named = PathBuf::from(plan["run"].as_str().unwrap_or_default());
    match named.is_file() {
        true => named,
        false => plan_path.parent().unwrap_or(Path::new(".")).join(named),
    }
}

/// The bytes of everything under a directory. What cannot be read counts
/// nothing, and a link is not followed.
fn bytes_under(dir: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    let of = |entry: std::fs::DirEntry| match entry.file_type() {
        Ok(kind) if kind.is_dir() => bytes_under(&entry.path()),
        _ => entry.metadata().map_or(0, |m| m.len()),
    };
    entries.flatten().map(of).sum()
}

/// The first copy of a plan's run, beside its receipt, and what it takes on
/// disk with what its commands built in it; nothing where the copy is gone.
fn first_copy(receipt: &Path) -> Option<(PathBuf, u64)> {
    let copy = receipt.parent().unwrap_or(Path::new(".")).join("w1");
    let bytes = copy.is_dir().then(|| bytes_under(&copy))?;
    Some((copy, bytes))
}

/// A size as a person reads it: `4.3 GiB`, `12.0 MiB`, `900 bytes`.
fn size(bytes: u64) -> String {
    let units = [("GiB", 1u64 << 30), ("MiB", 1 << 20), ("KiB", 1 << 10)];
    match units.iter().find(|(_, unit)| bytes >= *unit) {
        Some((name, unit)) => format!("{:.1} {name}", bytes as f64 / *unit as f64),
        None => format!("{bytes} bytes"),
    }
}

/// What the copies of the stage will take, from the plan's first copy. The
/// first pin with a test builds its own copies beside the base run's
/// (R188), so twice the plan's copies stand at once.
fn disk(copy: Option<(PathBuf, u64)>, copies: usize) -> String {
    match copy {
        Some((path, bytes)) => format!(
            "One copy of the plan takes {} on disk ({}). The base run, the pins and the check build copies of that size, {copies} at a time. The first pin with a test builds its own beside the base run's, so allow for {} copies at once, about {}. Remove the plan's copies before the first pin, and the pins' copies before the check: otherwise a third pair stands beside the others.",
            size(bytes),
            slashed(&path),
            2 * copies,
            size(2 * copies as u64 * bytes)
        ),
        None => "The copies of the plan are gone, so what one copy takes on disk is not known here.".to_owned(),
    }
}

/// A time as a person reads it: `40 s`, `17 min 50 s`, `3 h 2 min`.
pub fn about(seconds: f64) -> String {
    let whole = seconds.round() as u64;
    let (hours, minutes, rest) = (whole / 3600, whole % 3600 / 60, whole % 60);
    match (hours, minutes) {
        (0, 0) => format!("{rest} s"),
        (0, _) => format!("{minutes} min {rest} s"),
        _ => format!("{hours} h {minutes} min"),
    }
}

fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or("").trim()
}

/// A chosen function: its counts, each unnoticed fault with its place and
/// the code it was planted in, and what the function reaches out to.
fn chosen(one: &Chosen) -> Vec<String> {
    let mut lines = vec![format!(
        "  {}: {}  (noticed {}, unnoticed {}, other {})",
        one.file, one.function, one.noticed, one.unnoticed, one.other
    )];
    let fault = |f: &super::protect_stage::Fault| {
        format!("    {}  line {}: {}", f.id, f.line, first_line(&f.text))
    };
    lines.extend(one.faults.iter().map(fault));
    if let Some(out) = &one.reaches_out {
        let listed = |what: &str, items: &[String]| match items.is_empty() {
            true => None,
            false => Some(format!("    {what}: {}", items.join("; "))),
        };
        lines.extend(listed("it calls", &out.calls));
        lines.extend(listed("it writes state at", &out.state_writes));
    }
    lines
}

/// The copies the plan's run used.
fn copies_of(run: &Value) -> usize {
    run["workers"].as_array().map_or(0, Vec::len).max(1)
}

/// Counted as a run spends its time, the runs of three stages took up to
/// 1.05 times the figure (R191); the room is for a machine busy with other
/// work.
const SLOWER: f64 = 1.4;

/// The cost of running the faults of the chosen functions again, from the
/// plan's run: its seconds, its copies, and its run of the tests alone.
///
/// A run takes the tests alone in its first copy, then in the other copies,
/// and in every copy once more at the end; the first fault also runs alone
/// (`first_run.rs`). With one copy nothing waits for a turn. The check runs
/// the tests on the base as often again and takes the faults' results there
/// from the base run.
fn cost(stage: &Stage, run: &Value) -> Vec<String> {
    let workers = run["workers"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    let alone = |w: &Value| w["baseline_s"][0].as_f64().unwrap_or(0.0);
    let tests = workers.iter().map(alone).fold(0.0, f64::max);
    let copies = copies_of(run);
    let again = &stage.to_run_again;
    let times = if copies > 1 { 3.0 } else { 2.0 };
    let first = again.seconds / again.faults.max(1) as f64;
    let base = times * tests + first + (again.seconds - first) / copies as f64;
    let check = base + times * tests;
    vec![
        format!(
            "Faults to run again: {}. The base run runs them on the base. The check runs the tests on the base again, takes the faults' results there from the base run in the same work directory, and runs the faults on the step.",
            again.faults
        ),
        format!(
            "In the plan they took {} one after another. With {copies} copies at once, as the plan ran, the tests without a fault ({}) {times} times, as a run makes them, and the first fault alone, that is about {} for the base run and {} for the check. Allow {SLOWER} times as long on a busy machine: about {} and {}.",
            about(again.seconds),
            about(tests),
            about(base),
            about(check),
            about(base * SLOWER),
            about(check * SLOWER)
        ),
        "The base run comes first: the first `step pin` of the stage runs it, once, before any test is judged.".to_owned(),
    ]
}

/// Everything `step protect begin` says once the stage is written.
pub fn says(stage: &Stage, branch: &str, whole_scope: bool, run: &Value, receipt: &Path) -> String {
    let short: String = stage.base.chars().take(12).collect();
    let mut lines = vec![
        format!(
            "The stage {} begins from {short}, on the branch {branch}.",
            stage.spec
        ),
        String::new(),
        format!("Chosen ({}):", stage.chosen.len()),
    ];
    if whole_scope {
        lines.push(WHOLE.to_owned());
    }
    lines.extend(stage.chosen.iter().flat_map(chosen));
    if !stage.reported.is_empty() {
        lines.push(String::new());
        lines.push(format!(
            "Reported ({}): unnoticed faults in the plan, not chosen. They do not hold the stage back, and a reduction of them is not covered.",
            stage.reported.len()
        ));
        let one =
            |r: &Reported| format!("  {}: {}  (unnoticed {})", r.file, r.function, r.unnoticed);
        lines.extend(stage.reported.iter().map(one));
    }
    if !stage.left_out.is_empty() {
        lines.push(String::new());
        lines.push(format!(
            "Left out ({}): wholly in code that is not built on this system. Nothing is known about them here.",
            stage.left_out.len()
        ));
        let named =
            |one: &super::protect_stage::LeftOut| format!("  {}: {}", one.file, one.function);
        lines.extend(stage.left_out.iter().map(named));
    }
    lines.push(String::new());
    lines.extend(cost(stage, run));
    lines.push(disk(first_copy(receipt), copies_of(run)));
    lines.join("\n")
}
