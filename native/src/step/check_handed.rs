//! Numbers measured elsewhere, handed to `step check` for its report
//! (`.kiro/specs/run-dashboard`, tasks 6.2 and 6.3): rows of the two lenses
//! the tool does not measure, and the shapes of a type's state. The file is
//! named with `--measures`; the report marks what is from it as handed in.
//!
//! ```json
//! {"schema": 1,
//!  "lenses": [{"lens": "State", "measure": "ways T can be written", "before": 16, "after": 6,
//!              "expected": "falls", "command": "the command that counted them"}],
//!  "shapes": [{"type": "T", "side": "before", "command": "the command that counted them",
//!              "cells": [{"shape": "flag no, region none", "reached": true, "seen": false}]}]}
//! ```

use crate::dashboard::report::{Lens, TypeShapes, LOCAL, STATE};
use serde::Deserialize;
use serde_json::Value;
use std::path::Path;

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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    schema: u32,
    #[serde(default)]
    lenses: Vec<HandedLens>,
    #[serde(default)]
    shapes: Vec<TypeShapes>,
}

/// What a file named with `--measures` holds; nothing without a file.
#[derive(Default)]
pub struct Handed {
    pub lenses: Vec<Lens>,
    pub shapes: Vec<TypeShapes>,
}

fn lens(row: HandedLens) -> Result<Lens, String> {
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

/// The shapes of a type need the type, its shapes and where they are from.
fn shapes(of: &TypeShapes) -> Result<(), String> {
    let side = of.side.as_deref();
    let named = !of.name.trim().is_empty() && !of.command.trim().is_empty();
    match (named, of.cells.is_empty(), side) {
        (false, _, _) => Err("shapes need their type and the command that counted them".into()),
        (_, true, _) => Err(format!("shapes of {}: no shape is listed", of.name)),
        (_, _, None | Some("before" | "after")) => Ok(()),
        (_, _, Some(other)) => Err(format!(
            "shapes of {}: side is before or after, not {other}",
            of.name
        )),
    }
}

/// Reads the file named with `--measures`. A file that cannot be read as
/// such is bad input.
pub fn read_handed(path: Option<&Path>) -> Result<Handed, String> {
    let Some(path) = path else {
        return Ok(Handed::default());
    };
    let named = |why: String| format!("{}: {why}", path.display());
    let bytes = std::fs::read(path).map_err(|e| named(e.to_string()))?;
    let file: File = serde_json::from_slice(&bytes).map_err(|e| named(e.to_string()))?;
    if file.schema != 1 {
        return Err(named("schema must be 1".into()));
    }
    file.shapes.iter().try_for_each(shapes).map_err(named)?;
    let rows = file.lenses.into_iter().map(lens);
    Ok(Handed {
        lenses: rows.collect::<Result<_, _>>().map_err(named)?,
        shapes: file.shapes,
    })
}
