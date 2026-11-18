//! Find step of state consolidation (R156, revision 3 in R158): from source
//! files and a rules file, list each shared cell with where it is kept,
//! written and read, the conditions on what was read, and the function that
//! calls the writer and the reader. A cell is followed through the names it
//! is used under. Nothing here names a language or a project; that is in
//! the rules file.

use lesscode_core::adapters::driver::tree::{ancestors, parse_clean, walk, SgNode, SgRoot};
use lesscode_core::receipt::sha256_hex;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::{env, fs, ops::Range, path::Path, process::ExitCode};

#[derive(Deserialize)]
struct Binding {
    node: String,
    name: String,
    value: String,
}
#[derive(Deserialize)]
struct Bound {
    leaf: String,
    not_field: String,
}
#[derive(Deserialize)]
struct FieldRule {
    node: String,
    name: String,
    r#type: String,
    receiver: String,
}
#[derive(Deserialize)]
struct Tuple {
    node: String,
    skip: Vec<String>,
}
#[derive(Deserialize)]
struct Named {
    node: String,
    name: String,
}
#[derive(Deserialize)]
struct Impl {
    node: String,
    r#type: String,
}
#[derive(Deserialize)]
struct Literal {
    of: String,
    name: String,
    #[serde(rename = "self")]
    own: String,
    shorthand: String,
    node: String,
    field: String,
    value: String,
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
struct LetCondition {
    node: String,
    value: String,
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
    bound: Bound,
    field: FieldRule,
    tuple: Tuple,
    r#struct: Named,
    r#impl: Impl,
    literal: Literal,
    cells: Vec<String>,
    constructor: String,
    /// The forms under which a cell is handed on; `{}` is the cell.
    aliases: Vec<String>,
    method: Method,
    writes: Vec<String>,
    reads: Vec<String>,
    /// Methods that return a handle on the cell, not its value.
    handles: Vec<String>,
    /// Node kind, the field that is tested and, optionally, a field the node
    /// must have to be a condition at all.
    conditions: Vec<Vec<String>>,
    let_condition: LetCondition,
    arms: Arms,
    call: Call,
    parameters: Parameters,
}

type Span = Range<usize>;

fn line(node: &SgNode<'_>) -> usize {
    node.start_pos().line() + 1
}
fn text(node: &SgNode<'_>) -> String {
    node.text().into_owned()
}
fn within(span: &Span, node: &SgNode<'_>) -> bool {
    let r = node.range();
    span.start <= r.start && r.end <= span.end
}
fn inside(inner: &SgNode<'_>, outer: &SgNode<'_>) -> bool {
    within(&outer.range(), inner)
}
fn has(list: &[String], item: &str) -> bool {
    list.iter().any(|x| x == item)
}
fn leaf(node: &SgNode<'_>) -> bool {
    node.children().count() == 0
}
fn word(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| c.is_alphanumeric() || c == '_')
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

    fn above(&self, node: &SgNode<'_>, kind: &str, field: &str) -> Option<String> {
        let found = ancestors(node).into_iter().find(|a| a.kind() == kind)?;
        found.field(field).map(|n| text(&n))
    }
    /// The type an enclosing implementation is for, without its parameters.
    fn impl_type(&self, node: &SgNode<'_>) -> Option<String> {
        let written = self.above(node, &self.r#impl.node, &self.r#impl.r#type)?;
        Some(
            written
                .split('<')
                .next()
                .unwrap_or_default()
                .trim()
                .to_owned(),
        )
    }

    fn is_cell_type(&self, written: &str) -> bool {
        written
            .split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .any(|word| has(&self.cells, word))
    }
    /// The value is a cell's constructor, alone or inside other constructors.
    fn makes_cell(&self, value: &str) -> bool {
        let mut rest = value.trim();
        while let Some(open) = rest.find('(') {
            let Some(path) = rest[..open].strip_suffix(self.constructor.as_str()) else {
                return false;
            };
            if !path.split("::").all(word) {
                return false;
            }
            if has(&self.cells, path.rsplit("::").next().unwrap_or_default()) {
                return true;
            }
            rest = rest[open + 1..].trim_start();
        }
        false
    }

    /// What `written` hands on, for each form it could be.
    fn handed<'t>(&self, written: &'t str) -> Vec<&'t str> {
        let written = written.trim();
        let middle = |form: &String| {
            let (before, after) = form.split_once("{}")?;
            written.strip_prefix(before)?.strip_suffix(after)
        };
        self.aliases.iter().filter_map(middle).collect()
    }

    /// The names a pattern binds.
    fn bound_names(&self, pattern: &SgNode<'_>) -> Vec<String> {
        let binds = |l: &SgNode<'_>| {
            let is_type = l
                .parent()
                .and_then(|p| p.field(&self.bound.not_field))
                .is_some_and(|t| t.range() == l.range());
            leaf(l) && l.kind() == self.bound.leaf.as_str() && !is_type
        };
        walk(pattern)
            .iter()
            .filter(|l| binds(l))
            .map(text)
            .collect()
    }

    /// The node kinds that hold a condition, with the tested part of `node`.
    fn tested<'r>(&self, node: &SgNode<'r>) -> Option<(SgNode<'r>, &str)> {
        self.conditions.iter().find_map(|rule| {
            let needs = rule.get(2).is_none_or(|f| node.field(f).is_some());
            let subject = node.field(&rule[1])?;
            (node.kind() == rule[0].as_str() && needs).then_some((subject, rule[1].as_str()))
        })
    }

    /// The lines of the conditions decided on `access`, if it is read in one.
    /// A test of the handle itself is no condition on the value.
    fn guards(&self, access: &SgNode<'_>, method: &str) -> Vec<usize> {
        for parent in ancestors(access) {
            let Some((subject, field)) = self.tested(&parent) else {
                continue;
            };
            if !inside(access, &subject) {
                continue;
            }
            let value = if subject.kind() == self.let_condition.node.as_str() {
                subject.field(&self.let_condition.value).unwrap_or(subject)
            } else {
                subject
            };
            if has(&self.handles, method) && value.range() == access.range() {
                continue;
            }
            return self.condition_lines(&parent, field);
        }
        Vec::new()
    }

    /// The read is the value of a binding, and a later condition of the same
    /// unit names what was bound. One step only.
    fn guards_by_name(&self, access: &SgNode<'_>) -> Vec<usize> {
        let chain = ancestors(access);
        let value_of = |b: &SgNode<'_>| {
            b.field(&self.binding.value)
                .is_some_and(|v| inside(access, &v))
        };
        let bound = chain
            .iter()
            .find(|a| a.kind() == self.binding.node.as_str() && value_of(a));
        let unit = chain.iter().find(|a| self.unit_name(a).is_some());
        let (Some(bound), Some(unit)) = (bound, unit) else {
            return Vec::new();
        };
        let names = bound
            .field(&self.binding.name)
            .map(|p| self.bound_names(&p))
            .unwrap_or_default();
        let mut lines = Vec::new();
        for node in walk(unit) {
            let Some((subject, field)) = self.tested(&node) else {
                continue;
            };
            let later = subject.range().start >= bound.range().end;
            let named = || {
                walk(&subject)
                    .iter()
                    .any(|l| leaf(l) && has(&names, &l.text()))
            };
            if later && named() {
                lines.extend(self.condition_lines(&node, field));
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
            if let Some((_, field)) = self.tested(&node) {
                lines.extend(self.condition_lines(&node, field));
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

    fn arguments<'r>(&self, call: &SgNode<'r>) -> Vec<SgNode<'r>> {
        let list = call.field(&self.call.arguments);
        let real = |c: &SgNode<'r>| c.is_named() && !has(&self.comments, &c.kind());
        list.map(|l| l.children().filter(real).collect())
            .unwrap_or_default()
    }
}

struct Source<'r> {
    label: String,
    root: SgNode<'r>,
}

/// A named unit of some file.
struct Unit<'r> {
    file: usize,
    name: String,
    /// Named by a field of its own, as a function is.
    declared: bool,
    node: SgNode<'r>,
}

struct Access {
    file: usize,
    start: usize,
    line: usize,
    unit: String,
    method: String,
    writes: bool,
    reads: bool,
    guards: Vec<usize>,
    /// Conditions on a name the read was bound to.
    by_name: Vec<usize>,
}

/// A name the cell is used under, inside `scope` of `file`.
struct Name {
    file: usize,
    scope: Span,
    text: String,
    how: String,
}

enum Home {
    /// Bound in the unit `owner`.
    Local { owner: String },
    /// A field of the type `of`; a positional one is reached through the
    /// type's own receiver only.
    Field { of: String, positional: bool },
}

struct Cell {
    name: String,
    file: usize,
    line: usize,
    home: Home,
    names: Vec<Name>,
    accesses: Vec<Access>,
    /// Where a local binding of the same cell was made.
    also: Vec<String>,
    merged: bool,
}

impl Cell {
    fn reaches(&self, rules: &Rules, file: usize, node: &SgNode<'_>, receiver: &str) -> bool {
        let named = |n: &Name| n.file == file && n.text == receiver && within(&n.scope, node);
        if self.names.iter().any(named) {
            return true;
        }
        let Home::Field { of, positional } = &self.home else {
            return false;
        };
        if receiver == format!("{}{}", rules.field.receiver, self.name) {
            return rules.impl_type(node).as_deref() == Some(of);
        }
        !positional && file == self.file && receiver.ends_with(&format!(".{}", self.name))
    }

    fn add(&mut self, name: Name) -> bool {
        let same = |n: &Name| n.file == name.file && n.scope == name.scope && n.text == name.text;
        let fresh = !self.names.iter().any(same);
        if fresh {
            self.names.push(name);
        }
        fresh
    }
}

fn cells_of(rules: &Rules, file: usize, source: &Source<'_>) -> Vec<Cell> {
    let mut cells = Vec::new();
    let mut cell = |name: String, node: &SgNode<'_>, home: Home, names: Vec<Name>| {
        cells.push(Cell {
            name,
            file,
            line: line(node),
            home,
            names,
            accesses: Vec::new(),
            also: Vec::new(),
            merged: false,
        });
    };
    let of = |node: &SgNode<'_>| {
        rules
            .above(node, &rules.r#struct.node, &rules.r#struct.name)
            .unwrap_or_default()
    };
    for node in walk(&source.root) {
        let kind = node.kind();
        let part = |f: &str| node.field(f).map(|n| text(&n));
        if kind == rules.binding.node.as_str() {
            let made = part(&rules.binding.value).is_some_and(|v| rules.makes_cell(&v));
            let (Some(name), true) = (part(&rules.binding.name).filter(|n| word(n)), made) else {
                continue;
            };
            let (owner, scope) = match rules.holder(&node) {
                Some((owner, unit)) => (owner, unit.range()),
                None => (String::new(), source.root.range()),
            };
            let declared = Name {
                file,
                scope,
                text: name.clone(),
                how: "declared".into(),
            };
            cell(name, &node, Home::Local { owner }, vec![declared]);
        } else if kind == rules.field.node.as_str() {
            let typed = part(&rules.field.r#type).is_some_and(|t| rules.is_cell_type(&t));
            if let (Some(name), true) = (part(&rules.field.name), typed) {
                let home = Home::Field {
                    of: of(&node),
                    positional: false,
                };
                cell(name, &node, home, Vec::new());
            }
        } else if kind == rules.tuple.node.as_str() {
            let slots = node
                .children()
                .filter(|c| c.is_named() && !has(&rules.tuple.skip, &c.kind()))
                .filter(|c| !has(&rules.comments, &c.kind()));
            for (index, slot) in slots.enumerate() {
                if rules.is_cell_type(&slot.text()) {
                    let home = Home::Field {
                        of: of(&node),
                        positional: true,
                    };
                    cell(index.to_string(), &slot, home, Vec::new());
                }
            }
        }
    }
    cells
}

/// The function a call by name means: one of the same file, or the only one
/// of that name anywhere.
fn called<'u, 'r>(units: &'u [Unit<'r>], file: usize, written: &str) -> Vec<&'u Unit<'r>> {
    let name = written.rsplit([':', '.']).next().unwrap_or_default();
    let all: Vec<_> = units
        .iter()
        .filter(|u| u.declared && u.name == name)
        .collect();
    let near: Vec<_> = all.iter().copied().filter(|u| u.file == file).collect();
    match (near.is_empty(), all.len()) {
        (false, _) => near,
        (true, 1) => all,
        _ => Vec::new(),
    }
}

/// One round of following names: a binding to a form of the cell gives a
/// name in the binding's unit; an argument that is a form of the cell gives
/// the parameter's name in the function called. Returns whether any is new.
fn follow(rules: &Rules, sources: &[Source<'_>], units: &[Unit<'_>], cells: &mut [Cell]) -> bool {
    let mut grew = false;
    for (file, source) in sources.iter().enumerate() {
        let at = |node: &SgNode<'_>| format!("{}:{}", source.label, line(node));
        for node in walk(&source.root) {
            let kind = node.kind();
            if kind == rules.binding.node.as_str() {
                let (Some(value), Some(pattern)) = (
                    node.field(&rules.binding.value),
                    node.field(&rules.binding.name),
                ) else {
                    continue;
                };
                let (Some((_, unit)), true) = (rules.holder(&node), leaf(&pattern)) else {
                    continue;
                };
                let value = text(&value);
                for cell in cells.iter_mut() {
                    let handed = rules.handed(&value);
                    if handed.iter().any(|x| cell.reaches(rules, file, &node, x)) {
                        grew |= cell.add(Name {
                            file,
                            scope: unit.range(),
                            text: text(&pattern),
                            how: format!("bound at {}", at(&node)),
                        });
                    }
                }
            } else if kind == rules.call.node.as_str() {
                let arguments = rules.arguments(&node);
                let written = rules.callee(&node).unwrap_or_default();
                for (position, argument) in arguments.iter().enumerate() {
                    let argument = text(argument);
                    for cell in cells.iter_mut() {
                        let handed = rules.handed(&argument);
                        if !handed.iter().any(|x| cell.reaches(rules, file, &node, x)) {
                            continue;
                        }
                        for unit in called(units, file, &written) {
                            let Some(parameter) =
                                rules.parameters(&unit.node).get(position).cloned()
                            else {
                                continue;
                            };
                            grew |= cell.add(Name {
                                file: unit.file,
                                scope: unit.node.range(),
                                text: parameter,
                                how: format!("passed at {}", at(&node)),
                            });
                        }
                    }
                }
            }
        }
    }
    grew
}

fn accesses(rules: &Rules, sources: &[Source<'_>], cells: &mut [Cell]) {
    for (file, source) in sources.iter().enumerate() {
        for node in walk(&source.root) {
            let Some(callee) = node.field(&rules.method.callee) else {
                continue;
            };
            if node.kind() != rules.method.node.as_str()
                || callee.kind() != rules.method.access.as_str()
            {
                continue;
            }
            let part = |f: &str| callee.field(f).map(|n| text(&n)).unwrap_or_default();
            let (receiver, method) = (part(&rules.method.receiver), part(&rules.method.name));
            let (writes, reads) = (has(&rules.writes, &method), has(&rules.reads, &method));
            if !(writes || reads) {
                continue;
            }
            for cell in cells.iter_mut() {
                if !cell.reaches(rules, file, &node, &receiver) {
                    continue;
                }
                let some = |lines: Vec<usize>| if reads { lines } else { Vec::new() };
                cell.accesses.push(Access {
                    file,
                    start: node.range().start,
                    line: line(&node),
                    unit: rules.holder(&node).map(|h| h.0).unwrap_or_default(),
                    method: method.clone(),
                    writes,
                    reads,
                    guards: some(rules.guards(&node, &method)),
                    by_name: some(rules.guards_by_name(&node)),
                });
            }
        }
    }
}

/// A local cell put into a field of a struct literal, or into a slot of a
/// tuple struct, is that field's cell: `(local, field)` by index.
fn joined(rules: &Rules, sources: &[Source<'_>], cells: &[Cell]) -> Vec<(usize, usize)> {
    let field = |of: &str, name: &str, slot: bool| {
        cells.iter().position(|c| {
            matches!(&c.home, Home::Field { of: o, positional } if o == of && *positional == slot)
                && c.name == name
        })
    };
    let mut pairs = Vec::new();
    for (index, cell) in cells.iter().enumerate() {
        if !matches!(cell.home, Home::Local { .. }) {
            continue;
        }
        let file = cell.file;
        let is_cell = |node: &SgNode<'_>, written: &str| {
            let handed = rules.handed(written);
            handed.iter().any(|x| cell.reaches(rules, file, node, x))
        };
        let own = |node: &SgNode<'_>, written: String| {
            if written == rules.literal.own {
                rules.impl_type(node).unwrap_or_default()
            } else {
                written
            }
        };
        for node in walk(&sources[file].root) {
            let kind = node.kind();
            let part = |f: &str| node.field(f).map(|n| text(&n)).unwrap_or_default();
            let named = kind == rules.literal.node.as_str();
            let target = if named || kind == rules.literal.shorthand.as_str() {
                let (name, value) = if named {
                    (part(&rules.literal.field), part(&rules.literal.value))
                } else {
                    (text(&node).trim().to_owned(), text(&node))
                };
                let of = rules.above(&node, &rules.literal.of, &rules.literal.name);
                let of = own(&node, of.unwrap_or_default());
                is_cell(&node, &value)
                    .then(|| field(&of, &name, false))
                    .flatten()
            } else if kind == rules.call.node.as_str() {
                let of = own(&node, rules.callee(&node).unwrap_or_default());
                let slot = rules
                    .arguments(&node)
                    .iter()
                    .position(|a| is_cell(&node, &a.text()));
                slot.and_then(|s| field(&of, &s.to_string(), true))
            } else {
                None
            };
            if let Some(target) = target {
                pairs.push((index, target));
            }
        }
    }
    pairs
}

/// The calls in the cell's owner that hand over a writer and a reader, and
/// what the function called does with them.
fn movers(rules: &Rules, cell: &Cell, sources: &[Source<'_>], units: &[Unit<'_>]) -> Vec<Value> {
    let (Home::Local { owner }, Some(declared)) = (&cell.home, cell.names.first()) else {
        return Vec::new();
    };
    let label = |file: usize| sources[file].label.as_str();
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
        .filter(|u| u != owner)
        .collect();
    let mut found = Vec::new();
    for call in walk(&sources[cell.file].root) {
        let direct = rules.holder(&call).is_some_and(|h| &h.0 == owner);
        if call.kind() != rules.call.node.as_str() || !direct || !within(&declared.scope, &call) {
            continue;
        }
        let handed: Vec<Option<&str>> = rules
            .arguments(&call)
            .iter()
            .map(|argument| {
                let leaves = walk(argument);
                let named = |h: &&str| leaves.iter().any(|l| leaf(l) && l.text() == *h);
                holders.iter().copied().find(named)
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
            let there = label(unit.file);
            found.push(json!({
                "handed_over_at": format!("{}:{}", label(cell.file), line(&call)),
                "to": name, "in": there,
                "calls": calls.iter().map(|(l, p, h, r)| json!({
                    "at": format!("{there}:{l}"), "parameter": p, "is": h, "which": r,
                })).collect::<Vec<_>>(),
                "conditions": rules.all_conditions(&unit.node).iter()
                    .map(|l| format!("{there}:{l}")).collect::<Vec<_>>(),
            }));
        }
    }
    found
}

fn report(cell: &Cell, rules: &Rules, sources: &[Source<'_>], units: &[Unit<'_>]) -> Value {
    let at = |file: usize, l: usize| format!("{}:{l}", sources[file].label);
    let side = |writes: bool| -> Vec<Value> {
        let wanted = |a: &&Access| if writes { a.writes } else { a.reads };
        let shown =
            |a: &Access| json!({"at": at(a.file, a.line), "unit": a.unit, "method": a.method});
        cell.accesses.iter().filter(wanted).map(shown).collect()
    };
    let units_of = |writes: bool| -> BTreeSet<(usize, &str)> {
        let wanted = |a: &&Access| if writes { a.writes } else { a.reads };
        let all = cell.accesses.iter().filter(wanted);
        all.map(|a| (a.file, a.unit.as_str())).collect()
    };
    let (writers, readers) = (units_of(true), units_of(false));
    let apart = writers.iter().any(|w| readers.iter().any(|r| w != r));
    let lines = |of: fn(&Access) -> &Vec<usize>| {
        let all: BTreeSet<String> = cell
            .accesses
            .iter()
            .flat_map(|a| of(a).iter().map(|l| at(a.file, *l)))
            .collect();
        all.into_iter().collect::<Vec<_>>()
    };
    let (guards, by_name) = (lines(|a| &a.guards), lines(|a| &a.by_name));
    let verdict = match (apart, guards.is_empty() && by_name.is_empty()) {
        (true, false) => "machine",
        (true, true) => "shared, no condition on it",
        (false, _) => "one unit",
    };
    let moved = if apart {
        movers(rules, cell, sources, units)
    } else {
        Vec::new()
    };
    let (kind, owner, of) = match &cell.home {
        Home::Local { owner } => ("local", Some(owner), None),
        Home::Field { of, .. } => ("field", None, Some(of)),
    };
    let reached: Vec<Value> = cell
        .names
        .iter()
        .filter(|n| n.how != "declared")
        .map(|n| json!({"as": n.text, "in": sources[n.file].label, "how": n.how}))
        .collect();
    let files: BTreeSet<usize> = cell.accesses.iter().map(|a| a.file).collect();
    json!({
        "cell": cell.name, "verdict": verdict, "kind": kind,
        "kept": at(cell.file, cell.line), "also_bound_at": cell.also,
        "owner": owner, "of": of, "reached_as": reached,
        "written": side(true), "read": side(false),
        "units": writers.union(&readers).map(|u| u.1).collect::<BTreeSet<_>>(),
        "files_with_an_access": files.len(),
        "guards": guards, "guards_on_a_bound_name": by_name,
        "moved_by": moved,
    })
}

/// Calls of a parameter whose name some function of the scope also has: an
/// edge by name would point at that function.
fn parameter_calls(rules: &Rules, sources: &[Source<'_>], units: &[Unit<'_>]) -> Vec<Value> {
    let mut out = Vec::new();
    for source in sources {
        let calls = walk(&source.root);
        for call in calls
            .iter()
            .filter(|n| n.kind() == rules.call.node.as_str())
        {
            let (Some(name), Some((caller, unit))) = (rules.callee(call), rules.holder(call))
            else {
                continue;
            };
            if !rules.parameters(&unit).contains(&name) {
                continue;
            }
            let declared: BTreeSet<&str> = units
                .iter()
                .filter(|u| u.declared && u.name == name)
                .map(|u| sources[u.file].label.as_str())
                .collect();
            if !declared.is_empty() {
                out.push(
                    json!({"at": format!("{}:{}", source.label, line(call)), "caller": caller,
                                "parameter": name, "functions_of_that_name_in": declared}),
                );
            }
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
    let sources: Vec<Source<'_>> = parsed
        .iter()
        .map(|(label, tree)| Source {
            label: label.clone(),
            root: tree.root(),
        })
        .collect();
    let (mut units, mut cells) = (Vec::new(), Vec::new());
    for (file, source) in sources.iter().enumerate() {
        for node in walk(&source.root) {
            if let Some(name) = rules.unit_name(&node).filter(|n| !n.is_empty()) {
                let declared = !rules.units[node.kind().as_ref()].is_empty();
                units.push(Unit {
                    file,
                    name,
                    declared,
                    node,
                });
            }
        }
        cells.extend(cells_of(&rules, file, source));
    }
    let mut rounds = 0;
    while rounds < 3 && follow(&rules, &sources, &units, &mut cells) {
        rounds += 1;
    }
    accesses(&rules, &sources, &mut cells);
    for (local, field) in joined(&rules, &sources, &cells) {
        if local == field || cells[local].merged {
            continue;
        }
        let moved = std::mem::take(&mut cells[local].accesses);
        let bound = format!("{}:{}", sources[cells[local].file].label, cells[local].line);
        cells[local].merged = true;
        let target = &mut cells[field];
        for access in moved {
            let seen = |a: &Access| a.file == access.file && a.start == access.start;
            if !target.accesses.iter().any(seen) {
                target.accesses.push(access);
            }
        }
        target.also.push(bound);
    }
    for cell in &mut cells {
        cell.accesses.sort_by_key(|a| (a.file, a.start));
    }
    let listed: Vec<Value> = cells
        .iter()
        .filter(|c| !c.merged)
        .map(|c| report(c, &rules, &sources, &units))
        .collect();
    let by_name = parameter_calls(&rules, &sources, &units);
    let count = |verdict: &str| listed.iter().filter(|c| c["verdict"] == verdict).count();
    Ok(json!({
        "receipt": "find-machine-r158", "revision": 3,
        "rules_sha256": sha256_hex(rules_text.as_bytes()),
        "files": parsed.len(), "unparsed": unparsed,
        "totals": {"cells": listed.len(), "machines": count("machine"),
                   "shared_without_condition": count("shared, no condition on it"),
                   "in_one_unit": count("one unit"),
                   "parameter_calls_with_a_namesake": by_name.len()},
        "cells": listed, "parameter_calls_with_a_namesake": by_name,
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
