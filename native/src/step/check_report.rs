//! What `step check` found, as the report of the run page shows it
//! (`.kiro/specs/run-dashboard`, task 6.2): the lenses the tool has, before
//! and after the step, and the two it does not measure, which are shown
//! only from a file the person hands in. Everything here is read from the
//! verdict and the commits; nothing is judged again.

use super::check_page::SIDES;
use super::context::Context;
use super::functions::Tally;
use super::trees::Tree;
use super::verdict::{Verdict, PLANTED_FAULTS, REFUSED};
use crate::adapters::driver::calls::{calls_among, ScopeCalls, ScopeTotal};
use crate::adapters::driver::pack::Pack;
use crate::adapters::driver::page_report::{
    Counts, Lens, Protection, Refusal, Report, BEHAVIOUR, LOCAL, PROTECTION, SIZE, STATE, STRUCTURE,
};
use crate::adapters::driver::page_snapshot::{Stage, Status};
use serde::Deserialize;
use serde_json::Value;
use std::path::Path;

/// The commands the numbers are from, as a person would run them.
const CHECK: &str = "lesscode-core step check";
const SCOPE: &str = "lesscode-core deconvolute measure-scope";

const NOT_HANDED_IN: &str =
    "This run does not measure it. Numbers measured elsewhere are shown when a file with them is named with --measures.";

/// One measure of a lens the tool does not measure itself.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HandedLens {
    lens: String,
    measure: String,
    before: Value,
    after: Value,
    /// `falls`, `rises` or `stays`, or the direction in other words.
    expected: String,
    /// Whether it moved that way, where the direction is in other words.
    #[serde(default)]
    moved: Option<bool>,
    /// The command that produced the numbers.
    command: String,
}

/// Numbers measured elsewhere, for the report.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Handed {
    schema: u32,
    #[serde(default)]
    lenses: Vec<HandedLens>,
}

fn handed_lens(row: HandedLens) -> Result<Lens, String> {
    if ![STATE, LOCAL].contains(&row.lens.as_str()) {
        return Err(format!(
            "lens {:?}: only {STATE:?} and {LOCAL:?} can be handed in; the run measures the others itself",
            row.lens
        ));
    }
    let plain = |v: &Value| v.is_number() || v.is_string();
    if !plain(&row.before) || !plain(&row.after) {
        return Err(format!(
            "{}: before and after are a number or a text",
            row.measure
        ));
    }
    if row.measure.trim().is_empty() || row.command.trim().is_empty() {
        return Err("a row needs its measure and the command that produced its numbers".into());
    }
    let numbers = row.before.as_f64().zip(row.after.as_f64());
    let moved = match (row.expected.as_str(), numbers) {
        ("falls", Some((before, after))) => Some(after < before),
        ("rises", Some((before, after))) => Some(after > before),
        ("stays", _) => Some(row.before == row.after),
        _ => row.moved,
    };
    Ok(Lens {
        lens: row.lens,
        measure: row.measure,
        before: Some(row.before),
        after: Some(row.after),
        expected: row.expected,
        moved,
        measured: true,
        command: Some(row.command),
        handed_in: true,
        note: None,
    })
}

/// The rows of the file named with `--measures`; none without a file. A
/// file that cannot be read as such rows is bad input.
pub fn read_handed(path: Option<&Path>) -> Result<Vec<Lens>, String> {
    let Some(path) = path else {
        return Ok(Vec::new());
    };
    let named = |why: String| format!("{}: {why}", path.display());
    let bytes = std::fs::read(path).map_err(|e| named(e.to_string()))?;
    let handed: Handed = serde_json::from_slice(&bytes).map_err(|e| named(e.to_string()))?;
    if handed.schema != 1 {
        return Err(named("schema must be 1".into()));
    }
    let rows = handed.lenses.into_iter().map(handed_lens);
    rows.collect::<Result<_, _>>().map_err(named)
}

/// The rows handed in for a lens, or the one row that says it was not measured.
fn handed_or_not(handed: &[Lens], lens: &str, measure: &str, expected: &str) -> Vec<Lens> {
    let rows: Vec<Lens> = handed.iter().filter(|l| l.lens == lens).cloned().collect();
    match rows.is_empty() {
        true => vec![Lens::not_measured(lens, measure, expected, NOT_HANDED_IN)],
        false => rows,
    }
}

/// The calls among the scope's files as they are in one commit.
fn calls_at(
    context: &Context,
    pack: &Pack,
    tree: &Tree,
    commit: &str,
) -> Result<ScopeCalls, String> {
    let mut texts = Vec::new();
    for file in &context.declaration.scope {
        let text = context.text(tree, file)?;
        if !text.is_empty() {
            texts.push((file.clone(), text));
        }
    }
    calls_among(pack, commit, &texts)
}

/// Functions and calls of the scope, counted as `measure-scope` counts them.
fn structure(context: &Context, pack: &Pack) -> Vec<Lens> {
    let revisions = &context.revisions;
    let before = calls_at(context, pack, &context.base_tree, &revisions.base);
    let after = calls_at(context, pack, &context.step_tree, &revisions.step);
    let (before, after) = match (before, after) {
        (Ok(before), Ok(after)) => (before.total, after.total),
        (Err(why), _) | (_, Err(why)) => {
            return vec![Lens::not_measured(
                STRUCTURE,
                "functions and calls",
                "falls",
                &why,
            )]
        }
    };
    let row = |measure: &str, of: fn(&ScopeTotal) -> usize| {
        Lens::falls(STRUCTURE, measure, SCOPE, [of(&before), of(&after)])
    };
    vec![
        row("functions", |t| t.functions),
        row("calls from one function to another", |t| t.edges),
        row("pairs of files with a call between them", |t| t.module_hops),
    ]
}

/// Whether the project's tests pass with no fault planted, on each side,
/// read from how the two stages of the tests ended.
fn behaviour(context: &Context, verdict: &Verdict, stages: &[Stage]) -> Lens {
    let passed = |id: &str| match stages.iter().find(|s| s.id == id)?.status {
        Status::Ended => Some(true),
        // A check that is not judged did not find out.
        Status::Failed if verdict.result == REFUSED => Some(false),
        _ => None,
    };
    let sides = [passed(SIDES[0].tests), passed(SIDES[1].tests)];
    let measure = "your tests on the code, with no fault planted";
    if sides == [None, None] {
        let why = "The tests were not run, or could not be run.";
        return Lens::not_measured(BEHAVIOUR, measure, "stays", why);
    }
    let word = |side: Option<bool>| {
        side.map(|passed| Value::from(if passed { "pass" } else { "do not pass" }))
    };
    Lens {
        lens: BEHAVIOUR.to_owned(),
        measure: measure.to_owned(),
        before: word(sides[0]),
        after: word(sides[1]),
        expected: "stays".to_owned(),
        moved: match sides {
            [Some(true), after] => after,
            _ => None,
        },
        measured: true,
        command: Some(context.declaration.test.join(" ")),
        ..Lens::default()
    }
}

fn counts(tally: Option<Tally>) -> Option<Counts> {
    tally.map(|t| Counts {
        noticed: t.noticed as u64,
        unnoticed: t.unnoticed as u64,
        neither: t.other as u64,
    })
}

/// The planted faults of the functions the step was judged on: the row of
/// the lens, and the functions one by one.
fn protection(verdict: &Verdict) -> (Lens, Vec<Protection>) {
    let rows = verdict.functions.iter().map(|row| Protection {
        file: row.file.clone(),
        function: row.function.clone(),
        before: counts(row.before),
        after: counts(row.after),
        out_of_reach: row.out_of_reach.clone(),
    });
    let rows: Vec<Protection> = rows.collect();
    let (measure, expected) = (
        "planted faults in the functions of the step: noticed; unnoticed; neither",
        "unnoticed faults do not rise in any function",
    );
    let compared = verdict.checks.iter().find(|c| c.name == PLANTED_FAULTS);
    let Some(compared) = compared else {
        let why = "The faults of the two sides were not compared.";
        return (Lens::not_measured(PROTECTION, measure, expected, why), rows);
    };
    let sum = |side: fn(&Protection) -> Option<Counts>| {
        let all: Vec<Counts> = rows.iter().filter_map(side).collect();
        let of = |part: fn(&Counts) -> u64| all.iter().map(part).sum::<u64>();
        let (noticed, unnoticed, neither) =
            (of(|c| c.noticed), of(|c| c.unnoticed), of(|c| c.neither));
        Some(Value::from(format!("{noticed}; {unnoticed}; {neither}")))
    };
    let lens = Lens {
        lens: PROTECTION.to_owned(),
        measure: measure.to_owned(),
        before: sum(|row| row.before),
        after: sum(|row| row.after),
        expected: expected.to_owned(),
        moved: Some(compared.result == "pass"),
        measured: true,
        command: Some(CHECK.to_owned()),
        ..Lens::default()
    };
    (lens, rows)
}

/// The report of a check that has its verdict. `handed` are the rows of
/// `read_handed`; `stages` are the stages of the run as they ended.
pub fn report(
    context: &Context,
    pack: &Pack,
    verdict: &Verdict,
    handed: &[Lens],
    stages: &[Stage],
) -> Report {
    let (protection, functions) = protection(verdict);
    let size = &verdict.size;
    let mut lenses = vec![
        Lens::falls(SIZE, "tokens of the files in scope", CHECK, size.tokens),
        Lens::falls(
            SIZE,
            "syntax nodes of the files in scope",
            CHECK,
            size.nodes,
        ),
    ];
    lenses.extend(structure(context, pack));
    let state = "ways the state can be written, is reached and is seen by the tests";
    lenses.extend(handed_or_not(handed, STATE, state, "falls"));
    let (local, limit) = (
        "per routine: decisions, nesting, parameters and length",
        "no routine above its limit gets worse",
    );
    lenses.extend(handed_or_not(handed, LOCAL, local, limit));
    lenses.push(behaviour(context, verdict, stages));
    lenses.push(protection);
    let refused = verdict.result == REFUSED;
    Report {
        lenses,
        functions,
        refusal: refused.then(|| Refusal {
            reasons: verdict.reasons.clone(),
        }),
    }
}
