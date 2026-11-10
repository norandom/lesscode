//! Port of `run_controls.py`: control cases derived from hand-made seeds, run
//! through the validator and the mutant classifier.

use crate::pack::{truthy, Pack};
use crate::scope::{py_splitlines, py_strip, Scope};
use crate::tree::{grammar, parse, walk};
use crate::validator::{classify_mutant, validate, Decision, KINDS};
use serde::de::{Deserializer, MapAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::fmt;

const COMPLETED: &str = "+anchor_completed";

/// A JSON object read with its key order kept (Python dict order).
pub struct Ordered(pub Vec<(String, Value)>);

impl<'de> Deserialize<'de> for Ordered {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Ordered;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("an object")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Ordered, A::Error> {
                let mut out = Vec::new();
                while let Some(entry) = map.next_entry()? {
                    out.push(entry);
                }
                Ok(Ordered(out))
            }
        }
        d.deserialize_map(V)
    }
}

#[derive(Deserialize)]
pub struct ControlsFile {
    pub scopes: Ordered,
}

#[derive(Debug, Serialize)]
pub struct Outcome {
    pub scope: String,
    pub control: String,
    pub expected: Value,
    pub got: Value,
    pub pass: bool,
}

type Case = (String, Vec<Value>, Vec<String>);

fn field<'a>(v: &'a Value, key: &str) -> Result<&'a Value, String> {
    v.get(key).ok_or_else(|| format!("controls: missing {key}"))
}

fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    field(v, key)?
        .as_str()
        .ok_or_else(|| format!("controls: {key} is not a string"))
}

fn int(v: &Value, key: &str) -> Result<i64, String> {
    field(v, key)?
        .as_i64()
        .ok_or_else(|| format!("controls: {key} is not an integer"))
}

fn with(seed: &Value, changes: &[(&str, Value)]) -> Value {
    let mut changed = seed.clone();
    if let Some(object) = changed.as_object_mut() {
        for (key, value) in changes {
            object.insert((*key).to_owned(), value.clone());
        }
    }
    changed
}

fn label(prefix: &str, seed: &Value) -> Result<String, String> {
    Ok(format!(
        "{prefix}:{}@{}",
        text(seed, "kind")?,
        int(seed, "start_line")?
    ))
}

fn line_of(scope: &Scope, seed: &Value) -> Result<String, String> {
    let source = scope.source(text(seed, "file")?)?;
    let index = usize::try_from(int(seed, "start_line")? - 1).map_err(|e| e.to_string())?;
    py_splitlines(&source)
        .get(index)
        .map(|l| (*l).to_owned())
        .ok_or_else(|| "controls: seed line beyond the file".to_owned())
}

fn comment_node_at(scope: &Scope, pack: &Pack, file: &str, line: i64) -> Result<Value, String> {
    let root = parse(&scope.source(file)?, grammar(&scope.language)?)?;
    let node = walk(&root.root())
        .into_iter()
        .find(|n| pack.is_comment(&n.kind()) && n.start_pos().line() as i64 == line - 1)
        .ok_or_else(|| format!("no comment node at {file}:{line}; fix controls.json"))?;
    Ok(
        json!({"file": file, "start_line": line, "end_line": node.end_pos().line() + 1,
              "kind": "branch_condition", "anchor": node.text(), "reason": "control: a comment"}),
    )
}

/// A lone bracket that is unique on the seed's line is a token, not a site.
fn punctuation_case(scope: &Scope, seed: &Value) -> Result<Vec<Case>, String> {
    let line = line_of(scope, seed)?;
    let Some(bracket) = "({[".chars().find(|c| line.matches(*c).count() == 1) else {
        return Ok(Vec::new());
    };
    let token = with(
        seed,
        &[
            ("anchor", json!(bracket.to_string())),
            ("end_line", field(seed, "start_line")?.clone()),
        ],
    );
    Ok(vec![(
        "punctuation_anchor".into(),
        vec![token],
        vec!["anchor_mismatch".into()],
    )])
}

/// Pack format version 1: a seed anchor with or without the statement terminator.
fn terminator_cases(scope: &Scope, pack: &Pack, seeds: &[Value]) -> Result<Vec<Case>, String> {
    let terminator = pack.terminator();
    if terminator.is_empty() {
        return Ok(Vec::new());
    }
    let mut completed = Vec::new();
    let mut with_terminator = Vec::new();
    for seed in seeds {
        let anchor = text(seed, "anchor")?;
        if let Some(cut) = anchor.strip_suffix(terminator) {
            let annotation = with(seed, &[("anchor", json!(cut))]);
            completed.push((
                label("anchor_completed", seed)?,
                vec![annotation],
                vec![format!("ok{COMPLETED}")],
            ));
        }
    }
    for seed in seeds {
        let anchor = text(seed, "anchor")?;
        let full = format!("{anchor}{terminator}");
        if int(seed, "start_line")? == int(seed, "end_line")?
            && !anchor.ends_with(terminator)
            && py_strip(&line_of(scope, seed)?) == full
        {
            let annotation = with(seed, &[("anchor", json!(full))]);
            with_terminator.push((
                label("anchor_with_terminator", seed)?,
                vec![annotation],
                vec!["ok".into()],
            ));
        }
    }
    completed.extend(with_terminator);
    Ok(completed)
}

fn annotation_cases(scope: &Scope, pack: &Pack, spec: &Value) -> Result<Vec<Case>, String> {
    let seeds = field(spec, "seeds")?
        .as_array()
        .ok_or("controls: seeds is not a list")?;
    let first = seeds.first().ok_or("controls: no seeds")?;
    let mut cases: Vec<Case> = Vec::new();
    for seed in seeds {
        cases.push((label("seed", seed)?, vec![seed.clone()], vec!["ok".into()]));
    }
    let anchor = text(first, "anchor")?;
    let same_anchor: Vec<&str> = seeds
        .iter()
        .filter(|s| s.get("anchor") == first.get("anchor"))
        .filter_map(|s| s.get("kind").and_then(Value::as_str))
        .collect();
    let other_kind = KINDS
        .into_iter()
        .find(|k| !same_anchor.contains(k) && *k != "error_order")
        .ok_or("controls: no other kind")?;
    let first_kind = text(first, "kind")?;
    let (start, end) = (int(first, "start_line")?, int(first, "end_line")?);
    let partial: String = anchor
        .chars()
        .take(3.max(anchor.chars().count() / 2))
        .collect();
    let one = |label: &str, annotation: Value, reason: &str| {
        (label.to_owned(), vec![annotation], vec![reason.to_owned()])
    };
    cases.push(one(
        "wrong_line",
        with(
            first,
            &[
                ("start_line", json!(start + 3)),
                ("end_line", json!(end + 3)),
            ],
        ),
        "anchor_mismatch",
    ));
    cases.push(one(
        "altered_anchor",
        with(first, &[("anchor", json!(format!("{anchor} x")))]),
        "anchor_mismatch",
    ));
    cases.push(one(
        "partial_anchor",
        with(first, &[("anchor", json!(partial))]),
        "anchor_mismatch",
    ));
    cases.extend(punctuation_case(scope, first)?);
    // A scope file without any comment has no such control (comment_line is null).
    if truthy(spec.get("comment_line").unwrap_or(&Value::Null)) {
        let comment = comment_node_at(
            scope,
            pack,
            text(first, "file")?,
            int(spec, "comment_line")?,
        )?;
        cases.push(one("in_comment", comment, "in_comment_or_string"));
    }
    cases.push(one(
        "test_file",
        with(first, &[("file", field(spec, "test_file")?.clone())]),
        "test_file",
    ));
    cases.push(one(
        "outside_scope",
        with(first, &[("file", field(spec, "outside_file")?.clone())]),
        "outside_scope",
    ));
    let mismatch = if other_kind != first_kind {
        other_kind
    } else {
        "pure_shell_boundary"
    };
    cases.push(one(
        "kind_mismatch",
        with(first, &[("kind", json!(mismatch))]),
        "kind_mismatch",
    ));
    cases.push((
        "duplicate".into(),
        vec![first.clone(), first.clone()],
        vec!["ok".into(), "duplicate".into()],
    ));
    let mut no_kind = first.as_object().cloned().unwrap_or_else(Map::new);
    no_kind.remove("kind");
    cases.push(one(
        "malformed_no_kind",
        Value::Object(no_kind),
        "malformed",
    ));
    cases.push(one(
        "malformed_lines",
        with(first, &[("start_line", json!(end + 1))]),
        "malformed",
    ));
    cases.extend(terminator_cases(scope, pack, seeds)?);
    Ok(cases)
}

fn mutant_cases(
    source: &str,
    spec: &Value,
) -> Result<Vec<(&'static str, String, &'static str)>, String> {
    let change = field(spec, "real_change")?
        .as_array()
        .ok_or("controls: real_change is not a list")?;
    let pair: Vec<&str> = change.iter().filter_map(Value::as_str).collect();
    let [old, new] = pair[..] else {
        return Err("controls: real_change is not two strings".into());
    };
    if !source.contains(old) {
        return Err("real_change text not found; fix controls.json".into());
    }
    let comment = text(spec, "comment_text")?;
    Ok(vec![
        ("identical", source.to_owned(), "no_op"),
        (
            "whitespace_only",
            format!("{}\n\n\n", source.trim_end_matches('\n')),
            "no_op",
        ),
        ("comment_only", format!("{source}\n{comment}\n"), "no_op"),
        ("real_change", source.replacen(old, new, 1), "changed"),
        (
            "unparseable",
            source.replacen(old, &format!("{old} (((("), 1),
            "parse_error",
        ),
    ])
}

fn outcome(decision: &Decision) -> String {
    let mark = if decision.anchor_completed {
        COMPLETED
    } else {
        ""
    };
    format!("{}{mark}", decision.reason)
}

/// Every control of one scope: annotation cases, then mutant cases.
pub fn run_scope(scope: &Scope, pack: &Pack, spec: &Value) -> Result<Vec<Outcome>, String> {
    let mut results = Vec::new();
    for (control, annotations, expected) in annotation_cases(scope, pack, spec)? {
        let got: Vec<String> = validate(scope, pack, &annotations)?
            .iter()
            .map(outcome)
            .collect();
        results.push(Outcome {
            scope: scope.name.clone(),
            control,
            pass: got == expected,
            expected: json!(expected),
            got: json!(got),
        });
    }
    let seed = field(spec, "seeds")?.get(0).ok_or("controls: no seeds")?;
    let original = scope.source(text(seed, "file")?)?;
    for (label, mutated, expected) in mutant_cases(&original, spec)? {
        let got = classify_mutant(pack, &scope.language, &original, &mutated)?;
        results.push(Outcome {
            scope: scope.name.clone(),
            control: format!("mutant:{label}"),
            pass: got == expected,
            expected: json!(expected),
            got: json!(got),
        });
    }
    Ok(results)
}
