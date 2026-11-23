//! State holders of the types declared in a scope (R165, prototype): for each
//! struct and enum, which fields hold state, in how many shapes the type can
//! stand, and which functions write and which decide on each holder.
//!
//! A shape counts the kind of a value, not its content: `bool` has two,
//! `Option<T>` one more than `T`, an enum one per variant, anything else one.
//! Nothing here names a language; that is in the rules file.

use lesscode_core::adapters::driver::tree::{ancestors, parse_clean, walk, SgNode, SgRoot};
use lesscode_core::receipt::sha256_hex;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::{env, fs, path::Path, process::ExitCode};

#[derive(Deserialize)]
struct Named {
    node: String,
    name: String,
}
#[derive(Deserialize)]
struct FieldRule {
    node: String,
    name: String,
    r#type: String,
}
#[derive(Deserialize)]
struct EnumRule {
    node: String,
    name: String,
    variant: String,
}
#[derive(Deserialize)]
struct ImplRule {
    node: String,
    r#type: String,
}
#[derive(Deserialize)]
struct TestScope {
    node: String,
    attribute: String,
    marker: String,
}
#[derive(Deserialize)]
struct Access {
    node: String,
    receiver: String,
    name: String,
    own: String,
}
#[derive(Deserialize)]
struct Assign {
    nodes: Vec<String>,
    left: String,
}
#[derive(Deserialize)]
struct Method {
    node: String,
    callee: String,
}
#[derive(Deserialize)]
struct Rules {
    language: String,
    r#struct: Named,
    field: FieldRule,
    r#enum: EnumRule,
    function: Named,
    r#impl: ImplRule,
    /// Code inside such a node is test code and is left out.
    test_scope: TestScope,
    /// Types with a fixed number of values.
    counted: BTreeMap<String, u128>,
    /// One more shape than what is held.
    optional: Vec<String>,
    /// Wrappers that change nothing about what is held.
    transparent: Vec<String>,
    /// Types that can be changed through a shared reference.
    cells: Vec<String>,
    access: Access,
    assignment: Assign,
    method: Method,
    writes: Vec<String>,
    /// Node kind, the field that is tested and, optionally, a field the node
    /// must have to be a condition at all.
    conditions: Vec<Vec<String>>,
}

/// Above this a count is reported as open.
const MANY: u128 = 1_000_000_000_000;

fn text(node: &SgNode<'_>) -> String {
    node.text().into_owned()
}
fn line(node: &SgNode<'_>) -> usize {
    node.start_pos().line() + 1
}
fn has(list: &[String], item: &str) -> bool {
    list.iter().any(|x| x == item)
}
fn inside(inner: &SgNode<'_>, outer: &SgNode<'_>) -> bool {
    let (i, o) = (inner.range(), outer.range());
    o.start <= i.start && i.end <= o.end
}

/// The first type argument of `A, B` at the top level.
fn first_argument(inner: &str) -> &str {
    let mut depth = 0usize;
    for (at, c) in inner.char_indices() {
        match c {
            '<' | '(' | '[' => depth += 1,
            '>' | ')' | ']' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => return inner[..at].trim(),
            _ => {}
        }
    }
    inner.trim()
}

/// The last name of the type's path, and its first argument.
fn head_and_argument(written: &str) -> (&str, Option<&str>) {
    let mut t = written.trim();
    while let Some(rest) = t.strip_prefix('&') {
        t = rest.trim_start();
        if t.starts_with('\'') {
            t = t.split_once(' ').map_or("", |x| x.1).trim_start();
        }
        t = t.strip_prefix("mut ").unwrap_or(t).trim_start();
    }
    let (path, argument) = match t.find('<') {
        Some(open) if t.ends_with('>') => {
            (&t[..open], Some(first_argument(&t[open + 1..t.len() - 1])))
        }
        _ => (t, None),
    };
    (path.rsplit("::").next().unwrap_or_default().trim(), argument)
}

struct Declared {
    file: String,
    line: usize,
    /// Name and written type of each field; empty for an enum.
    fields: Vec<(String, String)>,
    /// Variants, for an enum.
    variants: Option<u128>,
}

struct Scope<'a> {
    rules: &'a Rules,
    types: BTreeMap<String, Declared>,
}

impl Scope<'_> {
    /// In how many shapes a value of the written type can stand.
    fn shapes(&self, written: &str, depth: usize) -> u128 {
        let (head, argument) = head_and_argument(written);
        let held = |s: &Self| argument.map_or(1, |a| s.shapes(a, depth + 1));
        if depth > 8 {
            return 1;
        }
        if let Some(n) = self.rules.counted.get(head) {
            return *n;
        }
        if has(&self.rules.optional, head) {
            return held(self).saturating_add(1);
        }
        if has(&self.rules.transparent, head) || has(&self.rules.cells, head) {
            return held(self);
        }
        match self.types.get(head) {
            Some(Declared {
                variants: Some(n), ..
            }) => *n,
            Some(declared) => self.product(&declared.fields, depth + 1),
            None => 1,
        }
    }

    fn product(&self, fields: &[(String, String)], depth: usize) -> u128 {
        fields.iter().fold(1u128, |all, (_, written)| {
            all.saturating_mul(self.shapes(written, depth))
        })
    }

    /// What kind of holder a field of this type is, if it is one.
    fn kind(&self, written: &str) -> Option<&'static str> {
        let mut at = written;
        loop {
            let (head, argument) = head_and_argument(at);
            if has(&self.rules.cells, head) {
                return Some("cell");
            }
            if has(&self.rules.transparent, head) {
                match argument {
                    Some(inner) => at = inner,
                    None => return None,
                }
                continue;
            }
            if self.rules.counted.contains_key(head) || has(&self.rules.optional, head) {
                return Some("flag");
            }
            return match self.types.get(head) {
                Some(Declared {
                    variants: Some(_), ..
                }) => Some("enumeration"),
                Some(_) if self.shapes(at, 0) > 1 => Some("compound"),
                _ => None,
            };
        }
    }
}

/// The node sits in test code.
fn in_tests(rules: &Rules, node: &SgNode<'_>) -> bool {
    ancestors(node).iter().any(|a| {
        if a.kind() != rules.test_scope.node.as_str() {
            return false;
        }
        let mut before = a.prev();
        while let Some(p) = before {
            if p.kind() != rules.test_scope.attribute.as_str() {
                break;
            }
            if text(&p).contains(&rules.test_scope.marker) {
                return true;
            }
            before = p.prev();
        }
        false
    })
}

fn declarations(rules: &Rules, label: &str, root: &SgNode<'_>, types: &mut BTreeMap<String, Declared>, twice: &mut BTreeSet<String>) {
    for node in walk(root) {
        let kind = node.kind();
        let is_struct = kind == rules.r#struct.node.as_str();
        let is_enum = kind == rules.r#enum.node.as_str();
        if !(is_struct || is_enum) || in_tests(rules, &node) {
            continue;
        }
        let name_field = if is_struct { &rules.r#struct.name } else { &rules.r#enum.name };
        let Some(name) = node.field(name_field).map(|n| text(&n)) else {
            continue;
        };
        let below = walk(&node);
        let declared = if is_struct {
            let fields = below
                .iter()
                .filter(|f| f.kind() == rules.field.node.as_str())
                .filter_map(|f| {
                    Some((text(&f.field(&rules.field.name)?), text(&f.field(&rules.field.r#type)?)))
                })
                .collect();
            Declared { file: label.to_owned(), line: line(&node), fields, variants: None }
        } else {
            let variants = below.iter().filter(|v| v.kind() == rules.r#enum.variant.as_str()).count();
            Declared { file: label.to_owned(), line: line(&node), fields: Vec::new(), variants: Some(variants as u128) }
        };
        if types.insert(name.clone(), declared).is_some() {
            twice.insert(name);
        }
    }
}

#[derive(Default)]
struct Use {
    writers: BTreeSet<String>,
    deciders: BTreeSet<String>,
    writes: usize,
    conditions: usize,
    reads: usize,
}

/// The enclosing function as `Type::name`, and the type it is implemented for.
fn unit(rules: &Rules, node: &SgNode<'_>) -> (String, Option<String>) {
    let above = ancestors(node);
    let function = above
        .iter()
        .find(|a| a.kind() == rules.function.node.as_str())
        .and_then(|f| f.field(&rules.function.name))
        .map(|n| text(&n))
        .unwrap_or_else(|| "(outside a function)".into());
    let of = above
        .iter()
        .find(|a| a.kind() == rules.r#impl.node.as_str())
        .and_then(|i| i.field(&rules.r#impl.r#type))
        .map(|t| head_and_argument(&text(&t)).0.to_owned());
    match &of {
        Some(t) => (format!("{t}::{function}"), of),
        None => (function, None),
    }
}

fn is_write(rules: &Rules, access: &SgNode<'_>) -> bool {
    let assigned = ancestors(access).iter().any(|a| {
        has(&rules.assignment.nodes, &a.kind())
            && a.field(&rules.assignment.left).is_some_and(|l| inside(access, &l))
    });
    let through_method = || {
        let Some(p) = access.parent() else { return false };
        let receiver = p.kind() == rules.access.node.as_str()
            && p.field(&rules.access.receiver).is_some_and(|r| r.range() == access.range());
        let named = p.field(&rules.access.name).is_some_and(|n| has(&rules.writes, &n.text()));
        let called = p.parent().is_some_and(|c| {
            c.kind() == rules.method.node.as_str()
                && c.field(&rules.method.callee).is_some_and(|f| f.range() == p.range())
        });
        receiver && named && called
    };
    assigned || through_method()
}

fn is_condition(rules: &Rules, access: &SgNode<'_>) -> bool {
    for a in ancestors(access) {
        if a.kind() == rules.function.node.as_str() {
            return false;
        }
        let tested = rules.conditions.iter().any(|rule| {
            a.kind() == rule[0].as_str()
                && rule.get(2).is_none_or(|f| a.field(f).is_some())
                && a.field(&rule[1]).is_some_and(|s| inside(access, &s))
        });
        if tested {
            return true;
        }
    }
    false
}

fn run() -> Result<Value, String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let [rules_path, root, files @ ..] = args.as_slice() else {
        return Err("usage: state-holders RULES ROOT FILE...".into());
    };
    let rules_text = fs::read_to_string(rules_path).map_err(|e| format!("{rules_path}: {e}"))?;
    let rules: Rules = serde_json::from_str(&rules_text).map_err(|e| e.to_string())?;
    let mut parsed: Vec<(String, SgRoot)> = Vec::new();
    let mut unparsed = Vec::new();
    for file in files {
        let label = file.replace('\\', "/");
        let source = fs::read_to_string(Path::new(root).join(file)).map_err(|e| format!("{file}: {e}"))?;
        match parse_clean(&label, &source, &rules.language) {
            Ok(tree) => parsed.push((label, tree)),
            Err(error) => unparsed.push(error),
        }
    }
    let mut types = BTreeMap::new();
    let mut twice = BTreeSet::new();
    for (label, tree) in &parsed {
        declarations(&rules, label, &tree.root(), &mut types, &mut twice);
    }
    let scope = Scope { rules: &rules, types };

    // Which types declare a field of a given name.
    let mut owners: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (name, declared) in &scope.types {
        for (field, _) in &declared.fields {
            owners.entry(field.as_str()).or_default().push(name.as_str());
        }
    }
    let mut uses: BTreeMap<(String, String), Use> = BTreeMap::new();
    let mut ambiguous = 0usize;
    for (_, tree) in &parsed {
        for access in walk(&tree.root()) {
            if access.kind() != rules.access.node.as_str() || in_tests(&rules, &access) {
                continue;
            }
            let Some(field) = access.field(&rules.access.name).map(|n| text(&n)) else {
                continue;
            };
            let Some(candidates) = owners.get(field.as_str()) else {
                continue;
            };
            let (label, of) = unit(&rules, &access);
            let own = access.field(&rules.access.receiver).is_some_and(|r| r.text() == rules.access.own.as_str());
            // `self.x` inside an implementation of a type that has `x` is that type's field.
            // Otherwise the field name decides, when only one type in the scope has it.
            let owner = match (&of, candidates.as_slice()) {
                (Some(t), _) if own && candidates.contains(&t.as_str()) => t.clone(),
                (_, [only]) => (*only).to_owned(),
                _ => {
                    ambiguous += 1;
                    continue;
                }
            };
            let entry = uses.entry((owner, field)).or_default();
            if is_write(&rules, &access) {
                entry.writes += 1;
                entry.writers.insert(label.clone());
            } else {
                entry.reads += 1;
            }
            if is_condition(&rules, &access) {
                entry.conditions += 1;
                entry.deciders.insert(label);
            }
        }
    }

    let shown = |n: u128| if n > MANY { json!("open") } else { json!(n) };
    let mut listed = Vec::new();
    let (mut holders, mut flags, mut cells, mut with_holders) = (0usize, 0usize, 0usize, 0usize);
    for (name, declared) in &scope.types {
        let mut rows = Vec::new();
        for (field, written) in &declared.fields {
            let Some(kind) = scope.kind(written) else { continue };
            let used = uses.get(&(name.clone(), field.clone()));
            let list = |set: Option<&BTreeSet<String>>| set.map(|s| s.iter().cloned().collect::<Vec<_>>()).unwrap_or_default();
            rows.push(json!({
                "field": field, "type": written, "kind": kind,
                "shapes": shown(scope.shapes(written, 0)),
                "writers": list(used.map(|u| &u.writers)), "deciders": list(used.map(|u| &u.deciders)),
                "write_sites": used.map_or(0, |u| u.writes), "condition_sites": used.map_or(0, |u| u.conditions),
                "other_reads": used.map_or(0, |u| u.reads),
            }));
        }
        let count = |k: &str| rows.iter().filter(|r| r["kind"] == k).count();
        let shapes = match declared.variants {
            Some(n) => n,
            None => scope.product(&declared.fields, 0),
        };
        holders += rows.len();
        flags += count("flag");
        cells += count("cell");
        with_holders += usize::from(!rows.is_empty());
        if rows.is_empty() && declared.variants.is_none() {
            continue;
        }
        listed.push(json!({
            "type": name, "file": declared.file, "line": declared.line,
            "is": if declared.variants.is_some() { "enumeration" } else { "struct" },
            "fields": declared.fields.len(), "flags": count("flag"), "cells": count("cell"),
            "writable_shapes": shown(shapes), "holders": rows,
        }));
    }
    Ok(json!({
        "receipt": "state-holders-r165", "revision": 1,
        "rules_sha256": sha256_hex(rules_text.as_bytes()),
        "files": parsed.len(), "unparsed": unparsed,
        "totals": {"types_declared": scope.types.len(), "structs_with_holders": with_holders,
                   "holders": holders, "flags": flags, "cells": cells,
                   "accesses_left_out_as_ambiguous": ambiguous},
        "declared_twice": twice, "types": listed,
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
            ExitCode::from(3)
        }
    }
}
