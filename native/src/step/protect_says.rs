//! What `step protect begin` says before anything runs: the functions that
//! were chosen with the faults no test noticed, the unprotected functions
//! that are left alone, and what running the faults again will cost, from
//! the seconds of the plan (requirements 1.1, 1.2, 1.7 and 5.9).

use super::protect_stage::{Chosen, Stage};
use serde_json::Value;

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

/// The cost of running the faults of the chosen functions again, from the
/// plan's run: its seconds, its copies, and its run of the tests alone.
fn cost(stage: &Stage, run: &Value) -> Vec<String> {
    let workers = run["workers"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    let alone = |w: &Value| w["baseline_s"][0].as_f64().unwrap_or(0.0);
    let tests = workers.iter().map(alone).fold(0.0, f64::max);
    let copies = workers.len().max(1);
    let again = &stage.to_run_again;
    let at_once = again.seconds / copies as f64 + tests;
    vec![
        format!(
            "Faults to run again: {}. The base run and the check each run them, and no others.",
            again.faults
        ),
        format!(
            "In the plan they took {} one after another. With {copies} copies at once, as the plan ran, and the tests once without a fault ({}), that is about {} for each of the two runs.",
            about(again.seconds),
            about(tests),
            about(at_once)
        ),
        "The base run comes first: the first `step pin` of the stage runs it, once, before any test is judged.".to_owned(),
    ]
}

/// Everything `step protect begin` says once the stage is written.
pub fn says(stage: &Stage, branch: &str, run: &Value) -> String {
    let short: String = stage.base.chars().take(12).collect();
    let mut lines = vec![
        format!(
            "The stage {} begins from {short}, on the branch {branch}.",
            stage.spec
        ),
        String::new(),
        format!("Chosen ({}):", stage.chosen.len()),
    ];
    lines.extend(stage.chosen.iter().flat_map(chosen));
    if !stage.left_alone.is_empty() {
        lines.push(String::new());
        lines.push(format!(
            "Left alone ({}): unprotected in the plan and not named. Nothing has to be done about them.",
            stage.left_alone.len()
        ));
        lines.extend(stage.left_alone.iter().map(|one| format!("  {one}")));
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
    lines.join("\n")
}
