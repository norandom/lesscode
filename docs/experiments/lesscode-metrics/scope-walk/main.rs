//! Writes a walk over the functions of a subsystem (R169): from its files
//! and a rules file, one test module inside each file and one that runs the
//! walk. What the modules look like is in the two templates; which syntax
//! means what, how a holder is read and which values are tried is in the
//! rules. Nothing here names a project.

use lesscode_core::adapters::driver::tree::{ancestors, parse_clean, walk, SgNode};
use lesscode_core::receipt::sha256_hex;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::{env, fs, path::Path, process::ExitCode};

const PART: &str = include_str!("part_template.rs");
const WALK: &str = include_str!("walk_template.rs");
const PART_MODULE: &str = "state_walk_part";
const WALK_MODULE: &str = "state_walk";

#[derive(Deserialize)]
struct StructRule {
    node: String,
    name: String,
    generics: String,
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
    variant_name: String,
    variant_body: String,
}
#[derive(Deserialize)]
struct ImplRule {
    node: String,
    r#type: String,
    r#trait: String,
    own_type: String,
}
#[derive(Deserialize)]
struct FunctionRule {
    node: String,
    name: String,
    returns: String,
    parameters: String,
    generics: String,
    modifiers: String,
    left_out: Vec<String>,
}
#[derive(Deserialize)]
struct ReceiverRule {
    node: String,
    shared: String,
    changing: String,
}
#[derive(Deserialize)]
struct ParameterRule {
    node: String,
    r#type: String,
}
#[derive(Deserialize)]
struct ModuleRule {
    node: String,
    body: String,
}
#[derive(Deserialize)]
struct Derives {
    default: Vec<String>,
    clone: Vec<String>,
}
#[derive(Deserialize)]
struct Rules {
    language: String,
    r#struct: StructRule,
    field: FieldRule,
    r#enum: EnumRule,
    r#impl: ImplRule,
    function: FunctionRule,
    receiver: ReceiverRule,
    parameter: ParameterRule,
    module: ModuleRule,
    attribute: String,
    comments: Vec<String>,
    test_marker: String,
    conditional_marker: String,
    derives: Derives,
    default_trait: String,
    length: usize,
    budget: u64,
    seconds: u64,
    rounds: usize,
    observe: BTreeMap<String, String>,
    enter: BTreeMap<String, String>,
    variant: String,
    pick: String,
    /// The wrapper under which a value is shared, and its full path.
    shared: String,
    shared_path: String,
    /// How a returned wrapper is opened; `{then}` is what to do with `r`.
    unwrap: BTreeMap<String, String>,
    /// How a path names the module above.
    parent: String,
    values: BTreeMap<String, Vec<String>>,
    values_by_head: BTreeMap<String, Vec<String>>,
}

fn text(node: &SgNode<'_>) -> String {
    node.text().into_owned()
}
fn has(list: &[String], item: &str) -> bool {
    list.iter().any(|x| x == item)
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

/// Whether the type is taken by reference, whether changeably, and the rest.
fn without_reference(written: &str) -> (bool, bool, &str) {
    let t = written.trim();
    let Some(mut rest) = t.strip_prefix('&') else {
        return (false, false, t);
    };
    rest = rest.trim_start();
    if rest.starts_with('\'') {
        rest = rest.split_once(' ').map_or("", |x| x.1).trim_start();
    }
    match rest.strip_prefix("mut ") {
        Some(inner) => (true, true, inner.trim_start()),
        None => (true, false, rest),
    }
}

/// The last name of the type's path, and its first argument.
fn head_and_argument(written: &str) -> (&str, Option<&str>) {
    let t = without_reference(written).2;
    let (path, argument) = match t.find('<') {
        Some(open) if t.ends_with('>') => {
            (&t[..open], Some(first_argument(&t[open + 1..t.len() - 1])))
        }
        _ => (t, None),
    };
    let argument = argument.filter(|a| !a.is_empty() && !a.starts_with('\''));
    (path.rsplit("::").next().unwrap_or_default().trim(), argument)
}

#[derive(Clone, Copy, PartialEq)]
enum Receiver {
    None,
    Shared,
    Changing,
    Owned,
}

struct Struct {
    file: usize,
    fields: Vec<(String, String)>,
    named: bool,
    default: bool,
    clone: bool,
    /// Why no value of it can be kept in the world, if so.
    not_kept: Option<String>,
}

struct Function {
    file: usize,
    /// The type it is implemented for.
    of: Option<String>,
    name: String,
    receiver: Receiver,
    parameters: Vec<String>,
    returns: Option<String>,
    left_out: Option<String>,
}

impl Function {
    fn label(&self) -> String {
        match &self.of {
            Some(t) => format!("{t}::{}", self.name),
            None => self.name.clone(),
        }
    }
}

#[derive(Default)]
struct Model {
    structs: BTreeMap<String, Struct>,
    enums: BTreeMap<String, Vec<(String, bool)>>,
    functions: Vec<Function>,
    twice: BTreeSet<String>,
}

fn attributes(rules: &Rules, node: &SgNode<'_>) -> Vec<String> {
    let mut out = Vec::new();
    let mut before = node.prev();
    while let Some(p) = before {
        let kind = p.kind();
        if kind == rules.attribute.as_str() {
            out.push(text(&p));
        } else if !has(&rules.comments, &kind) {
            break;
        }
        before = p.prev();
    }
    out
}

/// Why an item is not taken as it stands: test code, a condition, a module
/// written inside the file. `Some("")` means test code, which is not counted.
fn standing(rules: &Rules, node: &SgNode<'_>) -> Option<String> {
    let mut chain = vec![node.clone()];
    chain.extend(ancestors(node));
    let marks: Vec<String> = chain.iter().flat_map(|n| attributes(rules, n)).collect();
    if marks.iter().any(|m| m.contains(&rules.test_marker)) {
        return Some(String::new());
    }
    if marks.iter().any(|m| m.contains(&rules.conditional_marker)) {
        return Some("compiled only under a condition".into());
    }
    let inline = ancestors(node).iter().any(|a| a.kind() == rules.module.node.as_str() && a.field(&rules.module.body).is_some());
    inline.then(|| "inside a module written in the file".into())
}

fn read(rules: &Rules, file: usize, root: &SgNode<'_>, model: &mut Model) {
    let all = |parts: &[String], marks: &[String]| marks.iter().any(|m| parts.iter().all(|p| m.contains(p.as_str())));
    for node in walk(root) {
        let kind = node.kind();
        let is_struct = kind == rules.r#struct.node.as_str();
        let is_enum = kind == rules.r#enum.node.as_str();
        let is_function = kind == rules.function.node.as_str();
        let is_impl = kind == rules.r#impl.node.as_str();
        if !(is_struct || is_enum || is_function || is_impl) {
            continue;
        }
        let state = standing(rules, &node);
        if state.as_deref() == Some("") {
            continue;
        }
        if is_impl {
            // `impl Default for T`, written out.
            let of = node.field(&rules.r#impl.r#type).map(|t| head_and_argument(&text(&t)).0.to_owned());
            let is_default = node.field(&rules.r#impl.r#trait).is_some_and(|t| t.text() == rules.default_trait.as_str());
            if let (Some(of), true) = (of, is_default) {
                if let Some(s) = model.structs.get_mut(&of) {
                    s.default = true;
                }
            }
        } else if is_struct {
            let Some(name) = node.field(&rules.r#struct.name).map(|n| text(&n)) else {
                continue;
            };
            let below = walk(&node);
            let fields: Vec<(String, String)> = below
                .iter()
                .filter(|f| f.kind() == rules.field.node.as_str())
                .filter_map(|f| Some((text(&f.field(&rules.field.name)?), text(&f.field(&rules.field.r#type)?))))
                .collect();
            let marks = attributes(rules, &node);
            let named = !node.children().any(|c| c.kind().starts_with("ordered_"));
            let not_kept = state.or_else(|| node.field(&rules.r#struct.generics).map(|_| "it has parameters of its own".to_owned()));
            let declared = Struct { file, fields, named, default: all(&rules.derives.default, &marks), clone: all(&rules.derives.clone, &marks), not_kept };
            if model.structs.insert(name.clone(), declared).is_some() {
                model.twice.insert(name);
            }
        } else if is_enum {
            let Some(name) = node.field(&rules.r#enum.name).map(|n| text(&n)) else {
                continue;
            };
            let variants = walk(&node)
                .iter()
                .filter(|v| v.kind() == rules.r#enum.variant.as_str())
                .filter_map(|v| Some((text(&v.field(&rules.r#enum.variant_name)?), v.field(&rules.r#enum.variant_body).is_none())))
                .collect();
            model.enums.insert(name, variants);
        } else {
            let above = ancestors(&node);
            // A function written inside another is part of that one.
            if above.iter().any(|a| a.kind() == rules.function.node.as_str()) {
                continue;
            }
            let Some(name) = node.field(&rules.function.name).map(|n| text(&n)) else {
                continue;
            };
            let implemented = above.iter().find(|a| a.kind() == rules.r#impl.node.as_str());
            let of = implemented.and_then(|i| i.field(&rules.r#impl.r#type)).map(|t| head_and_argument(&text(&t)).0.to_owned());
            let modifiers = node.children().find(|c| c.kind() == rules.function.modifiers.as_str()).map(|m| text(&m)).unwrap_or_default();
            let left_out = state
                .or_else(|| implemented.and_then(|i| i.field(&rules.r#impl.r#trait)).map(|_| "implements a trait".to_owned()))
                .or_else(|| node.field(&rules.function.generics).map(|_| "it has parameters of its own".to_owned()))
                .or_else(|| rules.function.left_out.iter().find(|w| modifiers.split_whitespace().any(|m| m == w.as_str())).map(|w| format!("it is `{w}`")));
            let listed = node.field(&rules.function.parameters);
            let own: Vec<SgNode<'_>> = listed.map(|p| p.children().collect()).unwrap_or_default();
            let receiver = match own.iter().find(|p| p.kind() == rules.receiver.node.as_str()).map(text) {
                None => Receiver::None,
                Some(written) if written.starts_with(&rules.receiver.shared) && written.contains(&rules.receiver.changing) => Receiver::Changing,
                Some(written) if written.starts_with(&rules.receiver.shared) => Receiver::Shared,
                Some(_) => Receiver::Owned,
            };
            let parameters = own
                .iter()
                .filter(|p| p.kind() == rules.parameter.node.as_str())
                .filter_map(|p| p.field(&rules.parameter.r#type).map(|t| text(&t)))
                .collect();
            model.functions.push(Function { file, of, name, receiver, parameters, returns: node.field(&rules.function.returns).map(|t| text(&t)), left_out });
        }
    }
}

/// A value for a parameter.
enum Supply {
    /// Written values to try.
    Values(Vec<String>),
    /// A value of the world: what it is kept as, how it is handed over
    /// (`{a}` is the local it was taken into), and whether that uses it up.
    World { kept_as: String, hand: &'static str, used_up: bool },
}

struct Entry {
    label: String,
    code: String,
}

struct Maker<'a> {
    rules: &'a Rules,
    model: &'a Model,
    /// Types whose value is kept behind the shared wrapper.
    shared: BTreeSet<String>,
}

impl Maker<'_> {
    fn kept(&self, name: &str) -> Option<&Struct> {
        self.model.structs.get(name).filter(|s| s.not_kept.is_none() && !self.model.twice.contains(name))
    }

    /// A written type as the module inside the file has to write it.
    fn fix(&self, written: &str, own: Option<&str>) -> String {
        let mut out = String::new();
        let parent = self.rules.parent.as_str();
        let mut rest = written;
        while let Some(at) = rest.find(parent) {
            let first = !rest[..at].ends_with(':');
            out.push_str(&rest[..at]);
            if first {
                out.push_str(parent);
            }
            out.push_str(parent);
            rest = &rest[at + parent.len()..];
        }
        out.push_str(rest);
        match own {
            Some(own) => out.replace(&self.rules.r#impl.own_type, own),
            None => out,
        }
    }

    /// The type of the scope a written type is, bare or behind the shared
    /// wrapper: its name and whether it was written shared.
    fn scope_type(&self, written: &str, own: Option<&str>) -> Option<(String, bool)> {
        let (head, argument) = head_and_argument(written);
        let named = |h: &str| if h == self.rules.r#impl.own_type { own.map(str::to_owned) } else { Some(h.to_owned()) };
        if head == self.rules.shared {
            let inner = head_and_argument(argument?);
            let name = named(inner.0).filter(|_| inner.1.is_none())?;
            return self.model.structs.contains_key(&name).then_some((name, true));
        }
        let name = named(head).filter(|_| argument.is_none() || head == self.rules.r#impl.own_type)?;
        self.model.structs.contains_key(&name).then_some((name, false))
    }

    /// What a type of the scope is kept as, written for a module inside a file
    /// that names the type as `named`.
    fn kept_as(&self, name: &str, named: &str) -> String {
        if self.shared.contains(name) {
            format!("{}<{named}>", self.rules.shared_path)
        } else {
            named.to_owned()
        }
    }

    fn supply(&self, written: &str, own: Option<&str>) -> Result<Supply, String> {
        let plain: String = written.split_whitespace().collect::<Vec<_>>().join(" ");
        if let Some(values) = self.rules.values.get(&plain) {
            return Ok(Supply::Values(values.clone()));
        }
        let (by_reference, changeable, core) = without_reference(written);
        let (head, argument) = head_and_argument(core);
        if let Some((name, written_shared)) = self.scope_type(core, own) {
            let Some(info) = self.kept(&name) else {
                return Err(format!("no value of `{name}` can be kept"));
            };
            let shared = self.shared.contains(&name);
            // The type as this file names it, without the wrapper.
            let named = self.fix(if written_shared { argument.unwrap_or(core) } else { core }, own);
            let kept_as = self.kept_as(&name, &named);
            let world = |hand, used_up| Ok(Supply::World { kept_as: kept_as.clone(), hand, used_up });
            return match (written_shared, shared, by_reference, changeable) {
                (true, true, true, false) => world("&{a}", false),
                (true, true, true, true) => world("&mut {a}", false),
                (true, true, false, _) => world("{a}.clone()", false),
                (false, false, true, false) => world("&{a}", false),
                (false, false, true, true) => world("&mut {a}", false),
                (false, false, false, _) if info.clone => world("{a}.clone()", false),
                (false, false, false, _) => world("{a}", true),
                (false, true, true, false) => world("&*{a}", false),
                (false, true, false, _) if info.clone => world("(*{a}).clone()", false),
                _ => Err(format!("`{written}` is wanted, and the value of `{name}` is kept another way")),
            };
        }
        let lead = match (by_reference, changeable) {
            (true, true) => "&mut ",
            (true, false) => "&",
            _ => "",
        };
        if let (Some(variants), None) = (self.model.enums.get(head), argument) {
            let named = self.fix(core, own);
            let plain: Vec<String> = variants.iter().filter(|(_, unit)| *unit).take(3).map(|(v, _)| format!("{lead}{named}::{v}")).collect();
            if !plain.is_empty() {
                return Ok(Supply::Values(plain));
            }
        }
        if let Some(values) = self.rules.values_by_head.get(head) {
            return Ok(Supply::Values(values.iter().map(|v| format!("{lead}{v}")).collect()));
        }
        Err(format!("no value for a parameter of type `{written}`"))
    }

    /// The type a function returns, with the wrappers to open first.
    fn returned<'t>(&self, written: &'t str) -> (Vec<&'t str>, &'t str) {
        let mut wrappers = Vec::new();
        let mut at = written.trim().trim_start_matches("->").trim();
        loop {
            let (head, argument) = head_and_argument(at);
            match (self.rules.unwrap.contains_key(head), argument) {
                (true, Some(inner)) if !at.starts_with('&') => {
                    wrappers.push(head);
                    at = inner;
                }
                _ => return (wrappers, at),
            }
        }
    }

    /// The calls of one function: one per list of values, and the same call
    /// as a builder when it returns a type of the scope.
    fn calls(&self, f: &Function) -> Result<(Vec<Entry>, Option<Entry>, Vec<String>), String> {
        let own = f.of.as_deref();
        let mut world: Vec<(String, &'static str, bool)> = Vec::new();
        let plain_variants = |t: &str| -> Vec<String> {
            let variants = self.model.enums.get(t).map(Vec::as_slice).unwrap_or_default();
            variants.iter().filter(|(_, unit)| *unit).take(3).map(|(v, _)| format!("{t}::{v}")).collect()
        };
        // What is called, and how the call is named.
        let callees: Vec<(String, String)> = match (f.receiver, own) {
            (Receiver::None, Some(t)) => vec![(format!("{t}::{}(", f.name), f.label())],
            (Receiver::None, None) => vec![(format!("{}(", f.name), f.label())],
            (_, Some(t)) if !plain_variants(t).is_empty() => {
                plain_variants(t).into_iter().map(|v| (format!("{v}.{}(", f.name), format!("{v}.{}", f.name))).collect()
            }
            (receiver, Some(t)) => {
                if self.kept(t).is_none() {
                    return Err(format!("no value of `{t}` can be kept"));
                }
                let shared = self.shared.contains(t);
                let used_up = match (receiver, shared) {
                    (Receiver::Shared, _) | (Receiver::Changing, false) => false,
                    (Receiver::Owned, false) => true,
                    (Receiver::Changing, true) => return Err("needs `&mut`, and the value is shared".into()),
                    _ => return Err("takes the value itself, and it is shared".into()),
                };
                world.push((self.kept_as(t, t), "{a}", used_up));
                vec![(format!("a0.{}(", f.name), f.label())]
            }
            (_, None) => return Err("a receiver outside an implementation".into()),
        };
        // Values and world values, in the order of the parameters.
        let mut slots: Vec<Result<Vec<String>, usize>> = Vec::new();
        for p in &f.parameters {
            match self.supply(p, own)? {
                Supply::Values(v) => slots.push(Ok(v)),
                Supply::World { kept_as, hand, used_up } => {
                    if world.iter().any(|(k, _, _)| *k == kept_as) {
                        return Err("needs two values of one type".into());
                    }
                    slots.push(Err(world.len()));
                    world.push((kept_as, hand, used_up));
                }
            }
        }
        let middle: Vec<usize> = slots.iter().map(|s| s.as_ref().map_or(0, |v| 1.min(v.len() - 1))).collect();
        let mut picks = vec![middle.clone()];
        for (at, slot) in slots.iter().enumerate() {
            if let Ok(values) = slot {
                for other in (0..values.len()).filter(|&i| i != middle[at]) {
                    let mut pick = middle.clone();
                    pick[at] = other;
                    picks.push(pick);
                }
            }
        }
        let written = |pick: &[usize], shown: bool| {
            let parts: Vec<String> = slots
                .iter()
                .zip(pick)
                .map(|(slot, &i)| match slot {
                    Ok(values) => values[i].clone(),
                    Err(local) if shown => "_".to_owned(),
                    Err(local) => world[*local].1.replace("{a}", &format!("a{local}")),
                })
                .collect();
            parts.join(", ")
        };
        let callee = &callees[0].0;
        // What is done with what comes back.
        let (wrappers, inner) = f.returns.as_deref().map(|r| self.returned(r)).unwrap_or_default();
        let target = self.scope_type(inner, own).filter(|(name, _)| self.kept(name).is_some() && !inner.trim_start().starts_with('&'));
        let store = target.as_ref().map(|(name, written_shared)| {
            let wrap = self.shared.contains(name) && !written_shared;
            let mut then = if wrap { format!("w.put({}::new(r));", self.rules.shared_path) } else { "w.put(r);".to_owned() };
            for wrapper in wrappers.iter().rev() {
                then = self.rules.unwrap[*wrapper].replace("{then}", &then);
            }
            then
        });
        let assemble = |expression: &str, store: Option<&str>| {
            let mut code = String::new();
            for (i, (kept_as, _, _)) in world.iter().enumerate() {
                let back: String = (0..i).map(|j| format!("w.put(a{j}); ")).collect();
                code.push_str(&format!("let Some(mut a{i}) = w.take::<{kept_as}>() else {{ {back}return; }}; "));
            }
            code.push_str(&match store {
                Some(_) => format!("let r = {expression}; "),
                None => format!("let _ = {expression}; "),
            });
            for (i, (_, _, used_up)) in world.iter().enumerate() {
                if !used_up {
                    code.push_str(&format!("w.put(a{i}); "));
                }
            }
            code.push_str(store.unwrap_or_default());
            code
        };
        let entries: Vec<Entry> = callees
            .iter()
            .flat_map(|(code, label)| picks.iter().map(move |pick| (code, label, pick)))
            .map(|(code, label, pick)| Entry {
                label: format!("{label}({})", written(pick, true)),
                code: assemble(&format!("{code}{})", written(pick, false)), store.as_deref()),
            })
            .collect();
        let builder = target.as_ref().zip(store.as_deref()).map(|((name, _), store)| {
            // The receiver must not be what is built.
            let named = self.fix(if head_and_argument(inner).0 == self.rules.shared { head_and_argument(inner).1.unwrap_or(inner) } else { inner }, own);
            let guard = format!("if w.has::<{}>() {{ return; }} ", self.kept_as(name, &named));
            Entry { label: format!("build {} by {}", name, entries[0].label), code: guard + &assemble(&format!("{callee}{})", written(&middle, false)), Some(store)) }
        });
        // A returned type of the scope that cannot be kept: its methods, at once.
        let mut chained = Vec::new();
        let mut chained_names = Vec::new();
        let guard_type = self.scope_type(inner, own).map(|(n, _)| n).filter(|n| self.kept(n).is_none() && target.is_none());
        if let Some(guard_type) = guard_type {
            for n in self.model.functions.iter().filter(|n| n.of.as_deref() == Some(guard_type.as_str()) && n.left_out.is_none()) {
                if !matches!(n.receiver, Receiver::Shared | Receiver::Changing) {
                    continue;
                }
                let values: Result<Vec<Vec<String>>, String> = n
                    .parameters
                    .iter()
                    .map(|p| match self.supply(p, Some(&guard_type))? {
                        Supply::Values(v) => Ok(v),
                        Supply::World { .. } => Err("a value of the world, on something returned".to_owned()),
                    })
                    .collect();
                let Ok(values) = values else { continue };
                let middle: Vec<usize> = values.iter().map(|v| 1.min(v.len() - 1)).collect();
                let mut picks = vec![middle.clone()];
                for (at, v) in values.iter().enumerate() {
                    for other in (0..v.len()).filter(|&i| i != middle[at]) {
                        let mut pick = middle.clone();
                        pick[at] = other;
                        picks.push(pick);
                    }
                }
                chained_names.push(n.label());
                for pick in picks {
                    let more: Vec<String> = values.iter().zip(&pick).map(|(v, &i)| v[i].clone()).collect();
                    let more = more.join(", ");
                    chained.push(Entry {
                        label: format!("{} > {}({more})", entries[0].label, n.name),
                        code: assemble(&format!("{callee}{}).{}({more})", written(&middle, false), n.name), None),
                    });
                }
            }
        }
        let mut all = entries;
        all.extend(chained);
        Ok((all, builder, chained_names))
    }

    /// The holders below a place, each with the expression that reads it.
    fn observe(&self, file: usize, place: &str, written: &str, name: &str, depth: usize, out: &mut Vec<(String, String)>) {
        let (head, argument) = head_and_argument(written);
        if depth > 6 {
            return;
        }
        if let Some(how) = self.rules.observe.get(head) {
            out.push((name.to_owned(), format!("u8::from({})", how.replace("{x}", place))));
        } else if let Some(how) = self.rules.enter.get(head) {
            if let Some(inner) = argument {
                self.observe(file, &how.replace("{x}", place), inner, name, depth + 1, out);
            }
        } else if let Some(declared) = self.model.structs.get(head).filter(|s| s.file == file && s.named && argument.is_none()) {
            for (field, of) in &declared.fields {
                self.observe(file, &format!("{place}.{field}"), of, &format!("{name}.{field}"), depth + 1, out);
            }
        } else if let (Some(variants), None) = (self.model.enums.get(head), argument) {
            let named = self.fix(without_reference(written).2, None);
            let arms: Vec<String> = variants
                .iter()
                .enumerate()
                .map(|(n, (v, _))| self.rules.variant.replace("{E}", &named).replace("{V}", v).replace("{n}", &n.to_string()))
                .collect();
            out.push((name.to_owned(), self.rules.pick.replace("{x}", place).replace("{arms}", &arms.join(" "))));
        }
    }
}

fn row(entry: &Entry) -> String {
    format!("        ({:?}, (|w: &mut World| {{ {} }}) as fn(&mut World)),", entry.label, entry.code)
}

fn run() -> Result<Value, String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let [rules_path, root, root_module, files @ ..] = args.as_slice() else {
        return Err("usage: scope-walk RULES ROOT_DIRECTORY ROOT_MODULE FILE...".into());
    };
    let rules_text = fs::read_to_string(rules_path).map_err(|e| format!("{rules_path}: {e}"))?;
    let rules: Rules = serde_json::from_str(&rules_text).map_err(|e| e.to_string())?;
    let mut model = Model::default();
    for (at, file) in files.iter().enumerate() {
        let source = fs::read_to_string(Path::new(root).join(file)).map_err(|e| format!("{file}: {e}"))?;
        let tree = parse_clean(file, &source, &rules.language)?;
        read(&rules, at, &tree.root(), &mut model);
    }
    let mut maker = Maker { rules: &rules, model: &model, shared: BTreeSet::new() };
    // A type is kept shared when any function hands it out or takes it that way.
    for f in model.functions.iter().filter(|f| f.left_out.is_none()) {
        for written in f.parameters.iter().chain(&f.returns) {
            let inner = maker.returned(written).1;
            if let Some((name, true)) = maker.scope_type(inner, f.of.as_deref()) {
                maker.shared.insert(name);
            }
        }
    }

    let mut calls: Vec<Vec<String>> = vec![Vec::new(); files.len()];
    let mut builders: Vec<Vec<String>> = vec![Vec::new(); files.len()];
    let (mut called, mut not_called) = (BTreeSet::new(), Vec::new());
    let mut built_by: BTreeMap<String, String> = BTreeMap::new();
    let mut pending = Vec::new();
    for f in &model.functions {
        let outcome = match &f.left_out {
            Some(why) => Err(why.clone()),
            None => maker.calls(f),
        };
        match outcome {
            Ok((entries, builder, chained)) => {
                called.insert(f.label());
                called.extend(chained);
                calls[f.file].extend(entries.iter().map(row));
                if let Some(b) = builder {
                    let made = b.label.split(' ').nth(1).unwrap_or_default().to_owned();
                    built_by.entry(made).or_insert_with(|| "a function".to_owned());
                    builders[f.file].push(row(&b));
                }
            }
            Err(why) => pending.push((f.label(), why)),
        }
    }
    for (label, why) in pending {
        if !called.contains(&label) {
            not_called.push(json!({"function": label, "why": why}));
        }
    }

    // Values nothing returns: by default, or by writing the fields once.
    let mut first: Vec<Vec<String>> = vec![Vec::new(); files.len()];
    for (name, declared) in &model.structs {
        if maker.kept(name).is_none() || built_by.contains_key(name) {
            continue;
        }
        let kept_as = maker.kept_as(name, name);
        let wrap = |value: String| if maker.shared.contains(name) { format!("{}::new({value})", rules.shared_path) } else { value };
        let value = if declared.default {
            built_by.insert(name.clone(), "default".into());
            Some(format!("<{name}>::default()"))
        } else if declared.named {
            let fields: Result<Vec<String>, String> = declared
                .fields
                .iter()
                .map(|(field, of)| match maker.supply(of, Some(name))? {
                    Supply::Values(v) => Ok(format!("{field}: {}", v[1.min(v.len() - 1)])),
                    Supply::World { .. } => Err(String::new()),
                })
                .collect();
            fields.ok().map(|fields| {
                built_by.insert(name.clone(), "writing its fields (made up)".into());
                format!("{name} {{ {} }}", fields.join(", "))
            })
        } else {
            None
        };
        if let Some(value) = value {
            let code = format!("if w.has::<{kept_as}>() {{ return; }} w.put({});", wrap(value));
            first[declared.file].push(row(&Entry { label: format!("build {name} from nothing"), code }));
        }
    }

    // The modules.
    let mut written = Vec::new();
    let (mut all_holders, mut all_observe, mut all_builders, mut all_calls) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut holders_of: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (at, file) in files.iter().enumerate() {
        let (mut names, mut reads) = (Vec::new(), Vec::new());
        for (name, declared) in model.structs.iter().filter(|(n, s)| s.file == at && maker.kept(n).is_some()) {
            let mut holders = Vec::new();
            for (field, of) in &declared.fields {
                maker.observe(at, &format!("f.{field}"), of, field, 0, &mut holders);
            }
            let reach = if maker.shared.contains(name) { "&**x" } else { "x" };
            let pushes: String = holders.iter().map(|(_, e)| format!("s.push({e}); ")).collect();
            reads.push(format!(
                "    match w.get::<{}>() {{\n        Some(x) => {{ let f: &{name} = {reach}; s.push(1); {pushes}}}\n        None => s.extend([0u8; {}]),\n    }}",
                maker.kept_as(name, name),
                holders.len() + 1
            ));
            names.push(format!("    ({name:?}, \"(held)\"),"));
            names.extend(holders.iter().map(|(n, _)| format!("    ({name:?}, {n:?}),")));
            holders_of.insert(name.clone(), holders.into_iter().map(|(n, _)| n).collect());
        }
        let mut build = first[at].clone();
        build.extend(builders[at].clone());
        let module = PART
            .replace("@@FILE@@", file)
            .replace("@@ROOT@@", root_module)
            .replace("@@HOLDERS@@", &names.join("\n"))
            .replace("@@OBSERVE@@", &reads.join("\n"))
            .replace("@@BUILDERS@@", &build.join("\n"))
            .replace("@@CALLS@@", &calls[at].join("\n"));
        let stem = file.replace('\\', "/");
        let stem = stem.strip_suffix(".rs").unwrap_or(&stem).to_owned();
        let target = Path::new(root).join(&stem).join(format!("{PART_MODULE}.rs"));
        fs::create_dir_all(target.parent().expect("a directory")).map_err(|e| e.to_string())?;
        fs::write(&target, &module).map_err(|e| format!("{}: {e}", target.display()))?;
        let path = format!("super::{}::{PART_MODULE}", stem.replace('/', "::"));
        all_holders.push(format!("    h.extend({path}::HOLDERS);"));
        all_observe.push(format!("    {path}::observe(w, &mut s);"));
        all_builders.push(format!("    c.extend({path}::builders());"));
        all_calls.push(format!("    c.extend({path}::calls());"));
        written.push(json!({"file": file, "module": target.display().to_string(), "declare_in": file,
                            "declaration": format!("#[cfg(test)]\npub(crate) mod {PART_MODULE};"),
                            "calls": calls[at].len(), "builders": build.len(), "sha256": sha256_hex(module.as_bytes())}));
    }
    let central = WALK
        .replace("@@LENGTH@@", &rules.length.to_string())
        .replace("@@SECONDS@@", &rules.seconds.to_string())
        .replace("@@ROUNDS@@", &rules.rounds.to_string())
        .replace("@@BUDGET@@", &rules.budget.to_string())
        .replace("@@HOLDERS@@", &all_holders.join("\n"))
        .replace("@@OBSERVE@@", &all_observe.join("\n"))
        .replace("@@BUILDERS@@", &all_builders.join("\n"))
        .replace("@@CALLS@@", &all_calls.join("\n"));
    let target = Path::new(root).join(format!("{WALK_MODULE}.rs"));
    fs::write(&target, &central).map_err(|e| format!("{}: {e}", target.display()))?;

    let types: Vec<Value> = model
        .structs
        .iter()
        .map(|(name, s)| {
            json!({"type": name, "file": files[s.file], "kept": maker.kept(name).is_some(),
                   "not_kept_because": s.not_kept.clone().or_else(|| model.twice.contains(name).then(|| "declared twice".to_owned())),
                   "shared": maker.shared.contains(name), "built_by": built_by.get(name),
                   "holders": holders_of.get(name)})
        })
        .collect();
    Ok(json!({
        "receipt": "scope-walk-r169", "revision": 1,
        "rules_sha256": sha256_hex(rules_text.as_bytes()),
        "walk_module": {"module": target.display().to_string(), "declaration": format!("#[cfg(test)]\nmod {WALK_MODULE};"), "sha256": sha256_hex(central.as_bytes())},
        "parts": written, "types": types,
        "functions": {"read": model.functions.len(), "called": called.len(), "called_names": called, "not_called": not_called},
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
