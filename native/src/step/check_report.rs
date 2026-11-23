//! What `step check` found, as the report of the run page shows it
//! (`.kiro/specs/run-dashboard`, task 6.2): the lenses the tool has, before
//! and after the step, and the two it does not measure, which are shown
//! only from a file the person hands in. Everything here is read from the
//! verdict and the commits; nothing is judged again.

use super::check_handed::Handed;
use super::check_page::SIDES;
use super::context::Context;
use super::functions::Tally;
use super::trees::Tree;
use super::verdict::{Verdict, PLANTED_FAULTS, REFUSED};
use crate::adapters::driver::calls::{calls_among, FileCalls, ScopeTotal};
use crate::adapters::driver::pack::Pack;
use crate::adapters::driver::page_report::{
    Calls, Counts, FileNames, Graph, Lens, Protection, Refusal, Report, BEHAVIOUR, LOCAL,
    PROTECTION, SIZE, STATE, STRUCTURE,
};
use crate::adapters::driver::page_snapshot::{Stage, Status};
use serde_json::Value;

/// The commands the numbers are from, as a person would run them.
const CHECK: &str = "lesscode-core step check";
const SCOPE: &str = "lesscode-core deconvolute measure-scope";

const NOT_HANDED_IN: &str =
    "This run does not measure it. Numbers measured elsewhere are shown when a file with them is named with --measures.";

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
) -> Result<(ScopeTotal, Calls), String> {
    let mut texts = Vec::new();
    for file in &context.declaration.scope {
        let text = context.text(tree, file)?;
        if !text.is_empty() {
            texts.push((file.clone(), text));
        }
    }
    let counted = calls_among(pack, commit, &texts)?;
    let named = |(file, calls): (String, FileCalls)| FileNames {
        file,
        functions: calls.names,
    };
    let drawn = Calls {
        files: counted.files.into_iter().map(named).collect(),
        edges: counted.edges.into_iter().collect(),
    };
    Ok((counted.total, drawn))
}

/// Functions and calls of the scope, counted as `measure-scope` counts
/// them: the rows of the lens, and what the page draws.
fn structure(context: &Context, pack: &Pack) -> (Vec<Lens>, Option<Graph>) {
    let revisions = &context.revisions;
    let before = calls_at(context, pack, &context.base_tree, &revisions.base);
    let after = calls_at(context, pack, &context.step_tree, &revisions.step);
    let ((before, drawn_before), (after, drawn_after)) = match (before, after) {
        (Ok(before), Ok(after)) => (before, after),
        (Err(why), _) | (_, Err(why)) => {
            let none = Lens::not_measured(STRUCTURE, "functions and calls", "falls", &why);
            return (vec![none], None);
        }
    };
    let row = |measure: &str, of: fn(&ScopeTotal) -> usize| {
        Lens::falls(STRUCTURE, measure, SCOPE, [of(&before), of(&after)])
    };
    let rows = vec![
        row("functions", |t| t.functions),
        row("calls from one function to another", |t| t.edges),
        row("pairs of files with a call between them", |t| t.module_hops),
    ];
    let graph = Graph {
        before: drawn_before,
        after: drawn_after,
    };
    (rows, Some(graph))
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
    handed: &Handed,
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
    let (structure, graph) = structure(context, pack);
    lenses.extend(structure);
    let state = "ways the state can be written, is reached and is seen by the tests";
    lenses.extend(handed_or_not(&handed.lenses, STATE, state, "falls"));
    let (local, limit) = (
        "per routine: decisions, nesting, parameters and length",
        "no routine above its limit gets worse",
    );
    lenses.extend(handed_or_not(&handed.lenses, LOCAL, local, limit));
    lenses.push(behaviour(context, verdict, stages));
    lenses.push(protection);
    let refused = verdict.result == REFUSED;
    Report {
        lenses,
        functions,
        refusal: refused.then(|| Refusal {
            reasons: verdict.reasons.clone(),
        }),
        graph,
        shapes: handed.shapes.clone(),
    }
}
