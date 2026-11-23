//! The list of several reports: what the list page says about each
//! (`.kiro/specs/run-dashboard`, task 7.1; requirements 8.1 to 8.3).
//!
//! A row is taken from the run inside a report as that report stands in
//! the list's directory, so a name left out of the report cannot be in the
//! list. A refused run and one in which a measure got worse give the same
//! row as any other.

use serde_json::{json, Value};
use std::collections::BTreeSet;

/// The version of the list's shape. Its page reads only this one.
pub const SCHEMA: u32 = 1;

/// The project a run was on: the name of its repository's directory.
pub fn project_of(run: &Value) -> Option<String> {
    let place = run.pointer("/subject/repository")?.as_str()?;
    let name = place
        .trim_end_matches(['/', '\\'])
        .rsplit(['/', '\\'])
        .next()?;
    (!name.is_empty()).then(|| name.to_owned())
}

/// The files the run was on: those whose functions it counted or planted
/// faults in.
fn scope(run: &Value) -> Vec<String> {
    let places = [
        "/report/graph/before/files",
        "/report/graph/after/files",
        "/faults/files",
    ];
    let mut files = BTreeSet::new();
    for place in places {
        let rows = run.pointer(place).and_then(Value::as_array);
        let names = rows.into_iter().flatten().filter_map(|row| row.get("file"));
        files.extend(names.filter_map(Value::as_str).map(str::to_owned));
    }
    files.into_iter().collect()
}

/// One report of the list. `link` is where the report stands beside the
/// list.
pub fn row(run: &Value, link: &str, project: &str, names_left_out: bool) -> Value {
    let of = |at: &str| run.pointer(at).cloned().unwrap_or(Value::Null);
    json!({
        "link": link,
        "project": project,
        "names_left_out": names_left_out,
        "scope": scope(run),
        "date": of("/started"),
        "tool": of("/tool"),
        "command": of("/command"),
        "branch": of("/subject/branch"),
        "commit": of("/subject/commit"),
        "compared_with": of("/subject/compared_with"),
        "outcome": of("/outcome"),
        "outcome_detail": of("/outcome_detail"),
        "exit_code": of("/exit_code"),
        "reasons": of("/report/refusal/reasons"),
        "lenses": of("/report/lenses"),
    })
}

/// The list as its page reads it.
pub fn list(version: &str, written: String, reports: Vec<Value>) -> Value {
    json!({
        "schema": SCHEMA,
        "tool": {"name": "lesscode-core", "version": version},
        "written": written,
        "reports": reports,
    })
}
