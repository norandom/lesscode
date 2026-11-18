//! Find step of state consolidation (R156): from source files and a rules
//! file, list each shared cell with where it is kept, written and read, the
//! conditions on what was read, and the function that calls the writer and
//! the reader. Nothing here names a language or a project; that is in the
//! rules file.

use lesscode_core::adapters::driver::tree::{ancestors, parse_clean, walk, SgNode, SgRoot};
use lesscode_core::receipt::sha256_hex;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::{env, fs, path::Path, process::ExitCode};

#[derive(Deserialize)]
struct Binding {
    node: String,
    name: String,
    value: String,
}
#[derive(Deserialize)]
struct FieldRule {
    node: String,
    name: String,
    r#type: String,
    receiver: String,
}
#[derive(Deserialize)]
struct Method {
    node: String,
    callee: String,
    access: String,
    receiver: String,
    name: String,
}
#[derive(Deserialize)]
struct Arms {
    of: String,
    node: String,
    pattern: String,
    open: Vec<String>,
}
#[derive(Deserialize)]
struct Call {
    node: String,
    callee: String,
    arguments: String,
}
#[derive(Deserialize)]
struct Parameters {
    field: String,
    name: String,
}
#[derive(Deserialize)]
struct Rules {
    language: String,
    comments: Vec<String>,
    /// Node kind of a unit of code, and the field that names it. An empty
    /// field means the unit is named by the binding it is the value of.
    units: BTreeMap<String, String>,
    binding: Binding,
    field: FieldRule,
    cells: Vec<String>,
    constructor: String,
    method: Method,
    writes: Vec<String>,
    reads: Vec<String>,
    conditions: Vec<(String, String)>,
    arms: Arms,
    call: Call,
    parameters: Parameters,
}

fn line(node: &SgNode<'_>) -> usize {
    node.start_pos().line() + 1
}
fn text(node: &SgNode<'_>) -> String {
    node.text().into_owned()
}
fn inside(inner: &SgNode<'_>, outer: &SgNode<'_>) -> bool {
    let (i, o) = (inner.range(), outer.range());
    o.start <= i.start && i.end <= o.end
}
fn has(list: &[String], item: &str) -> bool {
    list.iter().any(|x| x == item)
}

impl Rules {
    /// `Some("")` for a unit without a name, `None` for a node that is no unit.
    fn unit_name(&self, node: &SgNode<'_>) -> Option<String> {
        let field = self.units.get(node.kind().as_ref())?;
        if !field.is_empty() {
            return Some(node.field(field).map(|n| text(&n)).unwrap_or_default());
        }
        let bound = node.parent().filter(|p| {
            p.kind() == self.binding.node.as_str()
                && p.field(&self.binding.value)
                    .is_some_and(|v| v.range() == node.range())
        });
        Some(
            bound
                .and_then(|p| p.field(&self.binding.name))
                .map(|n| text(&n))
                .unwrap_or_default(),
        )
    }

    /// The nearest enclosing unit that has a name.
    fn holder<'r>(&self, node: &SgNode<'r>) -> Option<(String, SgNode<'r>)> {
        ancestors(node).into_iter().find_map(|a| {
            let name = self.unit_name(&a)?;
            (!name.is_empty()).then_some((name, a))
        })
    }

    fn is_cell_type(&self, written: &str) -> bool {
        written
            .split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .any(|word| has(&self.cells, word))
    }
    fn makes_cell(&self, value: &str) -> bool {
        self.cells
            .iter()
            .any(|c| value.contains(&format!("{c}{}", self.constructor)))
    }

    /// The lines of the conditions decided on `access`, if it is read in one.
    fn guards(&self, access: &SgNode<'_>) -> Vec<usize> {
        for parent in ancestors(access) {
            for (kind, field) in &self.conditions {
                let subject = parent.field(field);
                if parent.kind() == kind.as_str() && subject.is_some_and(|s| inside(access, &s)) {
                    return self.condition_lines(&parent, field);
                }
            }
        }
        Vec::new()
    }

    /// Revision 2: the read is the value of a binding, and a later condition
    /// of the same unit names what was bound. One step only.
    fn guards_by_name(&self, access: &SgNode<'_>) -> Vec<usize> {
        let chain = ancestors(access);
        let value_of = |b: &SgNode<'_>| {
            b.field(&self.binding.value)
                .is_some_and(|v| inside(access, &v))
        };
        let bound = chain
            .iter()
            .find(|a| a.kind() == self.binding.node.as_str() && value_of(a));
        let name = bound
            .and_then(|b| b.field(&self.binding.name))
            .map(|n| text(&n))
            .filter(|n| !has(&self.arms.open, n));
        let unit = chain.iter().find(|a| self.unit_name(a).is_some());
        let (Some(bound), Some(name), Some(unit)) = (bound, name, unit) else {
            return Vec::new();
        };
        let mut lines = Vec::new();
        for node in walk(unit) {
            for (kind, field) in &self.conditions {
                let names = |s: SgNode<'_>| {
                    s.range().start >= bound.range().end
                        && walk(&s)
                            .iter()
                            .any(|l| l.children().count() == 0 && l.text() == name)
                };
                if node.kind() == kind.as_str() && node.field(field).is_some_and(names) {
                    lines.extend(self.condition_lines(&node, field));
                }
            }
        }
        lines
    }

    /// A condition counts at its own line; a node with arms counts once per
    /// arm that names a pattern.
    fn condition_lines(&self, parent: &SgNode<'_>, field: &str) -> Vec<usize> {
        if parent.kind() != self.arms.of.as_str() {
            return parent.field(field).iter().map(line).collect();
        }
        let own = |arm: &SgNode<'_>| {
            let nearest = ancestors(arm)
                .into_iter()
                .find(|a| a.kind() == self.arms.of.as_str());
            nearest.is_some_and(|n| n.range() == parent.range())
        };
        let names = |arm: &SgNode<'_>| {
            arm.field(&self.arms.pattern)
                .is_some_and(|p| !has(&self.arms.open, p.text().trim()))
        };
        walk(parent)
            .iter()
            .filter(|n| n.kind() == self.arms.node.as_str() && own(n) && names(n))
            .map(line)
            .collect()
    }

    fn all_conditions(&self, unit: &SgNode<'_>) -> Vec<usize> {
        let mut lines = Vec::new();
        for node in walk(unit) {
            for (kind, field) in &self.conditions {
                if node.kind() == kind.as_str() {
                    lines.extend(self.condition_lines(&node, field));
                }
            }
        }
        lines.sort_unstable();
        lines
    }

    fn parameters(&self, unit: &SgNode<'_>) -> Vec<String> {
        let Some(list) = unit.field(&self.parameters.field) else {
            return Vec::new();
        };
        list.children()
            .filter_map(|p| p.field(&self.parameters.name))
            .map(|n| text(&n))
            .collect()
    }

    fn callee(&self, call: &SgNode<'_>) -> Option<String> {
        call.field(&self.call.callee).map(|n| text(&n))
    }
}

struct Access {
    line: usize,
    unit: String,
    method: String,
    writes: bool,
    reads: bool,
    guards: Vec<usize>,
    /// Conditions on a name the read was bound to.
    by_name: Vec<usize>,
}

struct Cell<'r> {
    name: String,
    file: String,
    line: usize,
    /// The unit a local cell lives in; `None` for a field.
    owner: Option<(String, SgNode<'r>)>,
    accesses: Vec<Access>,
}

fn cells_of<'r>(rules: &Rules, file: &str, root: &SgNode<'r>) -> Vec<Cell<'r>> {
    let mut cells = Vec::new();
    let nodes = walk(root);
    for node in &nodes {
        let kind = node.kind();
        let local = kind == rules.binding.node.as_str()
            && node
                .field(&rules.binding.value)
                .is_some_and(|v| rules.makes_cell(&v.text()));
        let field = kind == rules.field.node.as_str()
            && node
                .field(&rules.field.r#type)
                .is_some_and(|t| rules.is_cell_type(&t.text()));
        let name_field = if local {
            &rules.binding.name
        } else {
            &rules.field.name
        };
        if !(local || field) {
            continue;
        }
        let Some(name) = node.field(name_field).map(|n| text(&n)) else {
            continue;
        };
        cells.push(Cell {
            name,
            file: file.to_owned(),
            line: line(node),
            owner: if local { rules.holder(node) } else { None },
            accesses: Vec::new(),
        });
    }
    for node in nodes
        .iter()
        .filter(|n| n.kind() == rules.method.node.as_str())
    {
        let Some(callee) = node.field(&rules.method.callee) else {
            continue;
        };
        if callee.kind() != rules.method.access.as_str() {
            continue;
        }
        let part = |f: &str| callee.field(f).map(|n| text(&n)).unwrap_or_default();
        let (receiver, method) = (part(&rules.method.receiver), part(&rules.method.name));
        let (writes, reads) = (has(&rules.writes, &method), has(&rules.reads, &method));
        if !(writes || reads) {
            continue;
        }
        for cell in &mut cells {
            let reached = match &cell.owner {
                Some((_, owner)) => receiver == cell.name && inside(node, owner),
                None => receiver == format!("{}{}", rules.field.receiver, cell.name),
            };
            if reached {
                cell.accesses.push(Access {
                    line: line(node),
                    unit: rules.holder(node).map(|h| h.0).unwrap_or_default(),
                    method: method.clone(),
                    writes,
                    reads,
                    guards: if reads {
                        rules.guards(node)
                    } else {
                        Vec::new()
                    },
                    by_name: if reads {
                        rules.guards_by_name(node)
                    } else {
                        Vec::new()
                    },
                });
            }
        }
    }
    cells
}

/// A named unit of some file.
struct Unit<'r> {
    file: String,
    name: String,
    node: SgNode<'r>,
}

/// The calls in the cell's owner that hand over a writer and a reader, and
/// what the function called does with them.
fn movers(rules: &Rules, cell: &Cell<'_>, units: &[Unit<'_>]) -> Vec<Value> {
    let Some((owner_name, owner)) = &cell.owner else {
        return Vec::new();
    };
    let role = |name: &str| {
        let any = |f: fn(&Access) -> bool| cell.accesses.iter().any(|a| a.unit == name && f(a));
        match (any(|a| a.writes), any(|a| a.reads)) {
            (true, true) => "writes and reads",
            (true, false) => "writes",
            _ => "reads",
        }
    };
    let holders: BTreeSet<&str> = cell
        .accesses
        .iter()
        .map(|a| a.unit.as_str())
        .filter(|u| u != owner_name)
        .collect();
    let mut found = Vec::new();
    for call in walk(owner) {
        let direct = rules.holder(&call).is_some_and(|h| &h.0 == owner_name);
        let Some(arguments) = call.field(&rules.call.arguments) else {
            continue;
        };
        if call.kind() != rules.call.node.as_str() || !direct {
            continue;
        }
        let handed: Vec<Option<&str>> = arguments
            .children()
            .filter(|c| c.is_named() && !has(&rules.comments, &c.kind()))
            .map(|argument| {
                let leaves = walk(&argument);
                holders.iter().copied().find(|h| {
                    leaves
                        .iter()
                        .any(|l| l.children().count() == 0 && l.text() == *h)
                })
            })
            .collect();
        if handed.iter().flatten().collect::<BTreeSet<_>>().len() < 2 {
            continue;
        }
        let written = rules.callee(&call).unwrap_or_default();
        let name = written.rsplit([':', '.']).next().unwrap_or_default();
        for unit in units.iter().filter(|u| u.name == name) {
            let parameters = rules.parameters(&unit.node);
            let mut calls = Vec::new();
            for (parameter, holder) in parameters.iter().zip(&handed) {
                let Some(holder) = holder else { continue };
                for inner in walk(&unit.node) {
                    if inner.kind() == rules.call.node.as_str()
                        && rules.callee(&inner).as_deref() == Some(parameter)
                    {
                        calls.push((line(&inner), parameter.clone(), *holder, role(holder)));
                    }
                }
            }
            calls.sort();
            found.push(json!({
                "handed_over_at": format!("{}:{}", cell.file, line(&call)),
                "to": name, "in": unit.file,
                "calls": calls.iter().map(|(l, p, h, r)| json!({
                    "at": format!("{}:{l}", unit.file), "parameter": p, "is": h, "which": r,
                })).collect::<Vec<_>>(),
                "conditions": rules.all_conditions(&unit.node).iter()
                    .map(|l| format!("{}:{l}", unit.file)).collect::<Vec<_>>(),
            }));
        }
    }
    found
}

fn report(cell: &Cell<'_>, rules: &Rules, units: &[Unit<'_>]) -> Value {
    let at = |l: usize| format!("{}:{l}", cell.file);
    let side = |writes: bool| -> Vec<Value> {
        let wanted = |a: &&Access| if writes { a.writes } else { a.reads };
        let shown = |a: &Access| json!({"at": at(a.line), "unit": a.unit, "method": a.method});
        cell.accesses.iter().filter(wanted).map(shown).collect()
    };
    let units_of = |writes: bool| -> BTreeSet<&str> {
        let wanted = |a: &&Access| if writes { a.writes } else { a.reads };
        cell.accesses
            .iter()
            .filter(wanted)
            .map(|a| a.unit.as_str())
            .collect()
    };
    let (writers, readers) = (units_of(true), units_of(false));
    let apart = writers.iter().any(|w| readers.iter().any(|r| w != r));
    let lines = |of: fn(&Access) -> &Vec<usize>| {
        let all: BTreeSet<usize> = cell.accesses.iter().flat_map(|a| of(a).clone()).collect();
        all.into_iter().collect::<Vec<_>>()
    };
    let (guards, by_name) = (lines(|a| &a.guards), lines(|a| &a.by_name));
    let verdict = match (apart, guards.is_empty() && by_name.is_empty()) {
        (true, false) => "machine",
        (true, true) => "shared, no condition on it",
        (false, _) => "one unit",
    };
    let moved = if apart {
        movers(rules, cell, units)
    } else {
        Vec::new()
    };
    json!({
        "cell": cell.name, "verdict": verdict,
        "kind": if cell.owner.is_some() { "local" } else { "field" },
        "kept": at(cell.line),
        "owner": cell.owner.as_ref().map(|o| o.0.clone()),
        "written": side(true), "read": side(false),
        "units": writers.union(&readers).collect::<Vec<_>>(),
        "guards": guards.iter().map(|l| at(*l)).collect::<Vec<_>>(),
        "guards_on_a_bound_name": by_name.iter().map(|l| at(*l)).collect::<Vec<_>>(),
        "moved_by": moved,
    })
}

/// Calls of a parameter whose name some function of the scope also has: an
/// edge by name would point at that function.
fn parameter_calls(rules: &Rules, file: &str, root: &SgNode<'_>, units: &[Unit<'_>]) -> Vec<Value> {
    let mut out = Vec::new();
    for call in walk(root)
        .iter()
        .filter(|n| n.kind() == rules.call.node.as_str())
    {
        let (Some(name), Some((caller, unit))) = (rules.callee(call), rules.holder(call)) else {
            continue;
        };
        if !rules.parameters(&unit).contains(&name) {
            continue;
        }
        let declared: BTreeSet<&str> = units
            .iter()
            .filter(|u| u.name == name && !rules.units[u.node.kind().as_ref()].is_empty())
            .map(|u| u.file.as_str())
            .collect();
        if !declared.is_empty() {
            out.push(
                json!({"at": format!("{file}:{}", line(call)), "caller": caller,
                            "parameter": name, "functions_of_that_name_in": declared}),
            );
        }
    }
    out
}

fn run() -> Result<Value, String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let [rules_path, root, files @ ..] = args.as_slice() else {
        return Err("usage: find-machine RULES ROOT FILE...".into());
    };
    let rules_text = fs::read_to_string(rules_path).map_err(|e| format!("{rules_path}: {e}"))?;
    let rules: Rules = serde_json::from_str(&rules_text).map_err(|e| e.to_string())?;
    let mut parsed: Vec<(String, SgRoot)> = Vec::new();
    let mut unparsed = Vec::new();
    for file in files {
        let label = file.replace('\\', "/");
        let source =
            fs::read_to_string(Path::new(root).join(file)).map_err(|e| format!("{file}: {e}"))?;
        match parse_clean(&label, &source, &rules.language) {
            Ok(tree) => parsed.push((label, tree)),
            Err(error) => unparsed.push(error),
        }
    }
    let mut units = Vec::new();
    for (file, tree) in &parsed {
        for node in walk(&tree.root()) {
            if let Some(name) = rules.unit_name(&node).filter(|n| !n.is_empty()) {
                units.push(Unit {
                    file: file.clone(),
                    name,
                    node,
                });
            }
        }
    }
    let (mut cells, mut by_name) = (Vec::new(), Vec::new());
    for (file, tree) in &parsed {
        let found = cells_of(&rules, file, &tree.root());
        cells.extend(found.iter().map(|c| report(c, &rules, &units)));
        by_name.extend(parameter_calls(&rules, file, &tree.root(), &units));
    }
    let count = |verdict: &str| cells.iter().filter(|c| c["verdict"] == verdict).count();
    Ok(json!({
        "receipt": "find-machine-r156", "revision": 2,
        "rules_sha256": sha256_hex(rules_text.as_bytes()),
        "files": parsed.len(), "unparsed": unparsed,
        "totals": {"cells": cells.len(), "machines": count("machine"),
                   "shared_without_condition": count("shared, no condition on it"),
                   "in_one_unit": count("one unit"),
                   "parameter_calls_with_a_namesake": by_name.len()},
        "cells": cells, "parameter_calls_with_a_namesake": by_name,
    }))
}

fn main() -> ExitCode {
    match run() {
        Ok(value) => {
            println!("{}", serde_json::to_string_pretty(&value).expect("json"));
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}
