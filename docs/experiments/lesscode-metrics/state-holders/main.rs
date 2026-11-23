//! State holders of the types declared in a scope (R165, revision 2): for
//! each struct and enum, which fields hold state, which of them are changed
//! after the value is built, in how many shapes the type can stand, and which
//! functions change and which decide on each holder.
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
struct FunctionRule {
    node: String,
    name: String,
    returns: String,
    body: String,
    /// Node kind of the receiver among the parameters.
    receiver: String,
}
#[derive(Deserialize)]
struct ImplRule {
    node: String,
    r#type: String,
    own_type: String,
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
struct PathRule {
    node: String,
    path: String,
    name: String,
}
#[derive(Deserialize)]
struct Binding {
    node: String,
    name: String,
    r#type: String,
    value: String,
}
#[derive(Deserialize)]
struct Rules {
    language: String,
    r#struct: Named,
    field: FieldRule,
    r#enum: EnumRule,
    function: FunctionRule,
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
    path: PathRule,
    literal: Named,
    /// Node kind of a plain name.
    name: String,
    parameter: FieldRule,
    binding: Binding,
    /// Node kind that marks a binding or a reference as changeable.
    mutable: String,
    /// Node kinds that hand on the value of their first named child.
    pass: Vec<String>,
    /// Node kinds that reach into the value of their first named child.
    reaches: Vec<String>,
    /// Further types that hold one value, for following a type only.
    holds: Vec<String>,
    /// Methods that hand on what the receiver holds.
    through: Vec<String>,
    /// Methods that give a handle through which the value can be changed.
    handles: Vec<String>,
    /// Methods that put another value in place.
    replacing: Vec<String>,
    /// Methods that change the value in place.
    changing: Vec<String>,
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
fn same(a: &SgNode<'_>, b: &SgNode<'_>) -> bool {
    a.range() == b.range()
}
fn first_named<'r>(node: &SgNode<'r>) -> Option<SgNode<'r>> {
    node.children().find(|c| c.is_named())
}

/// The first type argument of `'a, A, B` at the top level; a lifetime is none.
fn first_argument(inner: &str) -> &str {
    let (mut depth, mut from) = (0usize, 0usize);
    for (at, c) in inner.char_indices() {
        match c {
            '<' | '(' | '[' => depth += 1,
            '>' | ')' | ']' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                let argument = inner[from..at].trim();
                if !argument.starts_with('\'') {
                    return argument;
                }
                from = at + 1;
            }
            _ => {}
        }
    }
    inner[from..].trim()
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

#[derive(Default)]
struct Use {
    changers: BTreeSet<String>,
    deciders: BTreeSet<String>,
    replaced: usize,
    reached: usize,
    conditions: usize,
    reads: usize,
}

struct Scope<'a> {
    rules: &'a Rules,
    types: BTreeMap<String, Declared>,
    /// Written return type of a function, by the type it is implemented for
    /// (empty for none) and its name.
    returns: BTreeMap<(String, String), String>,
    /// Whether a function the scope declares may change its receiver.
    declared: BTreeMap<(String, String), bool>,
    /// Functions that hand out a field of their receiver: the field, and
    /// whether what they hand out is a handle.
    accessors: BTreeMap<(String, String), (String, bool)>,
    uses: BTreeMap<(String, String), Use>,
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
            Some(declared) => declared.fields.iter().fold(1u128, |all, (_, w)| {
                all.saturating_mul(self.shapes(w, depth + 1))
            }),
            None => 1,
        }
    }

    /// The shapes a type can move between after it is built.
    fn changing(&self, name: &str, depth: usize) -> u128 {
        let Some(declared) = self.types.get(name) else {
            return 1;
        };
        if depth > 8 || declared.variants.is_some() {
            return 1;
        }
        declared.fields.iter().fold(1u128, |all, (field, written)| {
            let used = self.uses.get(&(name.to_owned(), field.clone()));
            let replaced = used.is_some_and(|u| u.replaced > 0);
            let reached = used.is_some_and(|u| u.reached > 0);
            let inner = self.struct_of(written, None).filter(|s| self.types[s].variants.is_none());
            let own = match inner {
                _ if replaced => self.shapes(written, 0),
                Some(s) => self.changing(&s, depth + 1),
                None if reached => self.shapes(written, 0),
                None => 1,
            };
            all.saturating_mul(own)
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

    /// The type of the scope a written type holds, through wrappers.
    fn struct_of(&self, written: &str, own: Option<&str>) -> Option<String> {
        let r = self.rules;
        let mut at = written;
        for _ in 0..8 {
            let (head, argument) = head_and_argument(at);
            if head == r.r#impl.own_type {
                return own.map(str::to_owned);
            }
            if self.types.contains_key(head) {
                return Some(head.to_owned());
            }
            let wraps = has(&r.transparent, head)
                || has(&r.cells, head)
                || has(&r.optional, head)
                || has(&r.holds, head);
            at = argument.filter(|_| wraps)?;
        }
        None
    }

    /// The type an enclosing implementation is for.
    fn own_type(&self, node: &SgNode<'_>) -> Option<String> {
        ancestors(node)
            .iter()
            .find(|a| a.kind() == self.rules.r#impl.node.as_str())
            .and_then(|i| i.field(&self.rules.r#impl.r#type))
            .map(|t| head_and_argument(&text(&t)).0.to_owned())
    }

    /// The type of the scope a plain name stands for at `at`.
    fn bound(&self, name: &str, at: &SgNode<'_>, depth: usize) -> Option<String> {
        let r = self.rules;
        let function = ancestors(at)
            .into_iter()
            .find(|a| a.kind() == r.function.node.as_str())?;
        let own = self.own_type(at);
        let named = |pattern: Option<SgNode<'_>>| {
            pattern.is_some_and(|p| {
                let written = text(&p);
                written.strip_prefix("mut ").unwrap_or(&written).trim() == name
            })
        };
        let below = walk(&function);
        let latest = below
            .iter()
            .filter(|b| b.kind() == r.binding.node.as_str() && named(b.field(&r.binding.name)))
            .filter(|b| b.range().end <= at.range().start)
            .max_by_key(|b| b.range().start);
        if let Some(binding) = latest {
            if let Some(written) = binding.field(&r.binding.r#type) {
                return self.struct_of(&text(&written), own.as_deref());
            }
            return self.type_of(&binding.field(&r.binding.value)?, depth + 1);
        }
        below
            .iter()
            .find(|p| p.kind() == r.parameter.node.as_str() && named(p.field(&r.parameter.name)))
            .and_then(|p| p.field(&r.parameter.r#type))
            .and_then(|t| self.struct_of(&text(&t), own.as_deref()))
    }

    /// The type of the scope an expression has, where it can be followed.
    fn type_of(&self, node: &SgNode<'_>, depth: usize) -> Option<String> {
        let r = self.rules;
        if depth > 12 {
            return None;
        }
        let kind = node.kind();
        let own = || self.own_type(node);
        if node.text() == r.access.own.as_str() {
            return own();
        }
        if kind == r.name.as_str() {
            return self.bound(&node.text(), node, depth);
        }
        if kind == r.access.node.as_str() {
            let of = self.type_of(&node.field(&r.access.receiver)?, depth + 1)?;
            let field = text(&node.field(&r.access.name)?);
            let (_, written) = self.types.get(&of)?.fields.iter().find(|(f, _)| *f == field)?;
            return self.struct_of(written, Some(&of));
        }
        if kind == r.method.node.as_str() {
            let callee = node.field(&r.method.callee)?;
            let returned = |of: String, name: String| {
                let written = self.returns.get(&(of.clone(), name))?;
                self.struct_of(written, Some(&of).filter(|o| !o.is_empty()).map(String::as_str))
            };
            if callee.kind() == r.access.node.as_str() {
                let method = text(&callee.field(&r.access.name)?);
                let of = self.type_of(&callee.field(&r.access.receiver)?, depth + 1)?;
                if has(&r.through, &method) || has(&r.handles, &method) {
                    return Some(of);
                }
                return returned(of, method);
            }
            if callee.kind() == r.path.node.as_str() {
                let path = text(&callee.field(&r.path.path)?);
                let of = if path == r.r#impl.own_type { own()? } else { head_and_argument(&path).0.to_owned() };
                return returned(of, text(&callee.field(&r.path.name)?));
            }
            return returned(String::new(), text(&callee));
        }
        if kind == r.literal.node.as_str() {
            return self.struct_of(&text(&node.field(&r.literal.name)?), own().as_deref());
        }
        if has(&r.pass, &kind) || has(&r.reaches, &kind) {
            return self.type_of(&first_named(node)?, depth + 1);
        }
        None
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

/// The field of its receiver a function hands out as its value, and whether
/// through a handle: `fn progress(&self) -> … { self.progress.lock().unwrap() }`.
fn handed_out(rules: &Rules, function: &SgNode<'_>) -> Option<(String, bool)> {
    let body = function.field(&rules.function.body)?;
    let mut at = body.children().filter(|c| c.is_named()).last()?;
    let mut handle = false;
    loop {
        if at.kind() == rules.method.node.as_str() {
            let callee = at.field(&rules.method.callee)?;
            if callee.kind() != rules.access.node.as_str() {
                return None;
            }
            let method = text(&callee.field(&rules.access.name)?);
            if has(&rules.handles, &method) {
                handle = true;
            } else if !has(&rules.through, &method) {
                return None;
            }
            at = callee.field(&rules.access.receiver)?;
            continue;
        }
        if has(&rules.pass, &at.kind()) {
            at = first_named(&at)?;
            continue;
        }
        let own = at.kind() == rules.access.node.as_str()
            && at.field(&rules.access.receiver).is_some_and(|r| r.text() == rules.access.own.as_str());
        return own.then(|| Some((text(&at.field(&rules.access.name)?), handle))).flatten();
    }
}

fn declarations(rules: &Rules, label: &str, root: &SgNode<'_>, scope: &mut Scope<'_>, twice: &mut BTreeSet<String>) {
    for node in walk(root) {
        let kind = node.kind();
        if in_tests(rules, &node) {
            continue;
        }
        if kind == rules.function.node.as_str() {
            let Some(name) = node.field(&rules.function.name).map(|n| text(&n)) else {
                continue;
            };
            let of = scope.own_type(&node).unwrap_or_default();
            let changes = walk(&node).iter().any(|p| {
                p.kind() == rules.function.receiver.as_str()
                    && p.children().any(|c| c.kind() == rules.mutable.as_str())
            });
            scope.declared.insert((of.clone(), name.clone()), changes);
            if let Some(returns) = node.field(&rules.function.returns) {
                scope.returns.insert((of.clone(), name.clone()), text(&returns));
            }
            if let Some(handed) = handed_out(rules, &node) {
                scope.accessors.insert((of, name), handed);
            }
            continue;
        }
        let is_struct = kind == rules.r#struct.node.as_str();
        let is_enum = kind == rules.r#enum.node.as_str();
        if !(is_struct || is_enum) {
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
        if scope.types.insert(name.clone(), declared).is_some() {
            twice.insert(name);
        }
    }
}

/// The enclosing function, as `Type::name` inside an implementation.
fn unit(scope: &Scope<'_>, node: &SgNode<'_>) -> String {
    let rules = scope.rules;
    let function = ancestors(node)
        .iter()
        .find(|a| a.kind() == rules.function.node.as_str())
        .and_then(|f| f.field(&rules.function.name))
        .map(|n| text(&n))
        .unwrap_or_else(|| "(outside a function)".into());
    match scope.own_type(node) {
        Some(t) => format!("{t}::{function}"),
        None => function,
    }
}

enum Touch {
    Read,
    /// Another value is put in place.
    Replaced,
    /// The value is changed in place, or something below it is.
    Reached,
}

/// What the code does with the holder it names at `access`. `handle` says
/// that `access` already is a handle on the holder.
fn touch(scope: &Scope<'_>, access: &SgNode<'_>, mut handle: bool) -> Touch {
    let rules = scope.rules;
    let mut at = access.clone();
    let mut deeper = false;
    while let Some(p) = at.parent() {
        let kind = p.kind();
        let receives = kind == rules.access.node.as_str()
            && p.field(&rules.access.receiver).is_some_and(|r| same(&r, &at));
        if receives {
            let call = p.parent().filter(|c| {
                c.kind() == rules.method.node.as_str()
                    && c.field(&rules.method.callee).is_some_and(|f| same(&f, &p))
            });
            let Some(call) = call else {
                deeper = true;
                at = p;
                continue;
            };
            let method = p.field(&rules.access.name).map(|n| text(&n)).unwrap_or_default();
            // A method the scope declares for what is held: its receiver says whether it changes it.
            let own = scope.type_of(&at, 0).and_then(|of| scope.declared.get(&(of, method.clone())));
            match own {
                Some(true) => return Touch::Reached,
                Some(false) => return Touch::Read,
                None => {}
            }
            if has(&rules.replacing, &method) {
                return if deeper { Touch::Reached } else { Touch::Replaced };
            }
            if has(&rules.changing, &method) {
                return Touch::Reached;
            }
            if has(&rules.handles, &method) {
                handle = true;
            } else if !has(&rules.through, &method) {
                return Touch::Read;
            }
            at = call;
            continue;
        }
        if has(&rules.assignment.nodes, &kind) {
            let left = p.field(&rules.assignment.left).is_some_and(|l| same(&l, &at));
            return match (left, deeper) {
                (true, true) => Touch::Reached,
                (true, false) => Touch::Replaced,
                _ => Touch::Read,
            };
        }
        let changeable = |n: &SgNode<'_>| n.children().any(|c| c.kind() == rules.mutable.as_str());
        if has(&rules.pass, &kind) {
            // `&mut x` hands the holder out to be changed.
            if changeable(&p) {
                return Touch::Reached;
            }
            at = p;
            continue;
        }
        if has(&rules.reaches, &kind) && first_named(&p).is_some_and(|c| same(&c, &at)) {
            deeper = true;
            at = p;
            continue;
        }
        let bound = kind == rules.binding.node.as_str()
            && p.field(&rules.binding.value).is_some_and(|v| same(&v, &at));
        if !(bound && handle && changeable(&p)) {
            return Touch::Read;
        }
        // `let mut held = x.lock()…; *held = …` puts another value in place.
        let name = p.field(&rules.binding.name).map(|n| text(&n)).unwrap_or_default();
        let whole = format!("*{name}");
        let function = ancestors(&p).into_iter().find(|a| a.kind() == rules.function.node.as_str());
        let replaced = function.is_some_and(|f| {
            walk(&f).iter().any(|a| {
                has(&rules.assignment.nodes, &a.kind())
                    && a.range().start >= p.range().end
                    && a.field(&rules.assignment.left).is_some_and(|l| l.text() == whole.as_str())
            })
        });
        return if replaced { Touch::Replaced } else { Touch::Reached };
    }
    Touch::Read
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
    let mut scope = Scope {
        rules: &rules,
        types: BTreeMap::new(),
        returns: BTreeMap::new(),
        declared: BTreeMap::new(),
        accessors: BTreeMap::new(),
        uses: BTreeMap::new(),
    };
    let mut twice = BTreeSet::new();
    for (label, tree) in &parsed {
        declarations(&rules, label, &tree.root(), &mut scope, &mut twice);
    }

    // Which types declare a field of a given name.
    let mut owners: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (name, declared) in &scope.types {
        for (field, _) in &declared.fields {
            owners.entry(field.clone()).or_default().push(name.clone());
        }
    }
    let (mut by_type, mut by_name) = (0usize, 0usize);
    let mut left_out = Vec::new();
    let mut uses: BTreeMap<(String, String), Use> = BTreeMap::new();
    for (file, tree) in &parsed {
        for access in walk(&tree.root()) {
            if access.kind() != rules.access.node.as_str() || in_tests(&rules, &access) {
                continue;
            }
            let Some(named) = access.field(&rules.access.name).map(|n| text(&n)) else {
                continue;
            };
            let receiver = access.field(&rules.access.receiver).and_then(|r| scope.type_of(&r, 0));
            let call = access.parent().filter(|c| {
                c.kind() == rules.method.node.as_str()
                    && c.field(&rules.method.callee).is_some_and(|f| same(&f, &access))
            });
            // A call is a use only of the field its function hands out.
            let (owner, field, start, handle) = if let Some(call) = call {
                let handed = receiver.and_then(|of| Some((scope.accessors.get(&(of.clone(), named))?.clone(), of)));
                let Some(((field, handle), of)) = handed else {
                    continue;
                };
                by_type += 1;
                (of, field, call, handle)
            } else {
                let Some(candidates) = owners.get(&named) else {
                    continue;
                };
                let owner = match (receiver.filter(|t| candidates.contains(t)), candidates.as_slice()) {
                    (Some(t), _) => {
                        by_type += 1;
                        t
                    }
                    (None, [only]) => {
                        by_name += 1;
                        only.clone()
                    }
                    _ => {
                        left_out.push(json!({"file": file, "line": line(&access), "text": text(&access),
                                             "types_with_the_field": candidates}));
                        continue;
                    }
                };
                (owner, named, access.clone(), false)
            };
            let access = start;
            let label = unit(&scope, &access);
            let entry = uses.entry((owner, field)).or_default();
            match touch(&scope, &access, handle) {
                Touch::Replaced => {
                    entry.replaced += 1;
                    entry.changers.insert(label.clone());
                }
                Touch::Reached => {
                    entry.reached += 1;
                    entry.changers.insert(label.clone());
                }
                Touch::Read => entry.reads += 1,
            }
            if is_condition(&rules, &access) {
                entry.conditions += 1;
                entry.deciders.insert(label);
            }
        }
    }
    scope.uses = uses;

    let shown = |n: u128| if n > MANY { json!("open") } else { json!(n) };
    let mut listed = Vec::new();
    let (mut holders, mut changed, mut flags, mut cells, mut with_holders) = (0usize, 0usize, 0usize, 0usize, 0usize);
    for (name, declared) in &scope.types {
        let mut rows = Vec::new();
        for (field, written) in &declared.fields {
            let Some(kind) = scope.kind(written) else { continue };
            let used = scope.uses.get(&(name.clone(), field.clone()));
            let list = |set: Option<&BTreeSet<String>>| set.map(|s| s.iter().cloned().collect::<Vec<_>>()).unwrap_or_default();
            let touched = used.is_some_and(|u| u.replaced + u.reached > 0);
            changed += usize::from(touched);
            rows.push(json!({
                "field": field, "type": written, "kind": kind,
                "shapes": shown(scope.shapes(written, 0)),
                "changed": touched,
                "changers": list(used.map(|u| &u.changers)), "deciders": list(used.map(|u| &u.deciders)),
                "replaced_sites": used.map_or(0, |u| u.replaced), "reached_sites": used.map_or(0, |u| u.reached),
                "condition_sites": used.map_or(0, |u| u.conditions), "other_reads": used.map_or(0, |u| u.reads),
            }));
        }
        let count = |k: &str| rows.iter().filter(|r| r["kind"] == k).count();
        let shapes = match declared.variants {
            Some(n) => n,
            None => declared.fields.iter().fold(1u128, |all, (_, w)| all.saturating_mul(scope.shapes(w, 0))),
        };
        holders += rows.len();
        flags += count("flag");
        cells += count("cell");
        with_holders += usize::from(!rows.is_empty());
        if rows.is_empty() && declared.variants.is_none() {
            continue;
        }
        let moving = if declared.variants.is_some() { Value::Null } else { shown(scope.changing(name, 0)) };
        listed.push(json!({
            "type": name, "file": declared.file, "line": declared.line,
            "is": if declared.variants.is_some() { "enumeration" } else { "struct" },
            "fields": declared.fields.len(), "flags": count("flag"), "cells": count("cell"),
            "changed_holders": rows.iter().filter(|r| r["changed"] == true).count(),
            "writable_shapes": shown(shapes), "changing_shapes": moving, "holders": rows,
        }));
    }
    Ok(json!({
        "receipt": "state-holders-r165", "revision": 3,
        "rules_sha256": sha256_hex(rules_text.as_bytes()),
        "files": parsed.len(), "unparsed": unparsed,
        "totals": {"types_declared": scope.types.len(), "structs_with_holders": with_holders,
                   "holders": holders, "changed_holders": changed, "flags": flags, "cells": cells,
                   "uses_tied_by_type": by_type, "uses_tied_by_name": by_name,
                   "uses_left_out_as_ambiguous": left_out.len()},
        "declared_twice": twice, "types": listed, "left_out": left_out,
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
