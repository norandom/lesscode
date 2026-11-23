//! Reads the holders of a subsystem's types while the project's own tests
//! run (R170). `instrument` adds one statement, compiled in test builds
//! only, to each function that takes a struct of the subsystem by reference,
//! and writes the modules those statements call. `collect` gathers what the
//! tests noted. Which syntax means what and how a holder is read is in the
//! rules; nothing here names a project.

use lesscode_core::adapters::driver::tree::{ancestors, parse_clean, walk, SgNode};
use lesscode_core::receipt::sha256_hex;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::{env, fs, path::Path, process::ExitCode};

const PART: &str = include_str!("part_template.rs");
const WATCH: &str = include_str!("watch_template.rs");
const PART_MODULE: &str = "lesscode_watch_part";
const WATCH_MODULE: &str = "lesscode_watch";

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
    parameters: String,
    body: String,
    generics: String,
    modifiers: String,
    left_out: Vec<String>,
}
#[derive(Deserialize)]
struct ReceiverRule {
    node: String,
    shared: String,
    name: String,
}
#[derive(Deserialize)]
struct ParameterRule {
    node: String,
    name: String,
    r#type: String,
}
#[derive(Deserialize)]
struct ModuleRule {
    node: String,
    body: String,
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
    shared: String,
    observe: BTreeMap<String, String>,
    /// Wrappers that are looked through.
    enter: BTreeMap<String, String>,
    /// Wrappers that are looked into only when free; the template gives a
    /// `Result` whose `Ok` holds the guard.
    enter_if_free: BTreeMap<String, String>,
    /// The place behind a guard bound as `h`.
    inside: String,
    if_free: String,
    variant: String,
    pick: String,
    parent: String,
    statement: String,
    statement_for_receiver: String,
}

fn text(node: &SgNode<'_>) -> String {
    node.text().into_owned()
}
fn has(list: &[String], item: &str) -> bool {
    list.iter().any(|x| x == item)
}

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

/// Whether the type is taken by reference, and the rest.
fn without_reference(written: &str) -> (bool, &str) {
    let t = written.trim();
    let Some(mut rest) = t.strip_prefix('&') else {
        return (false, t);
    };
    rest = rest.trim_start();
    if rest.starts_with('\'') {
        rest = rest.split_once(' ').map_or("", |x| x.1).trim_start();
    }
    (true, rest.strip_prefix("mut ").unwrap_or(rest).trim_start())
}

fn head_and_argument(written: &str) -> (&str, Option<&str>) {
    let t = without_reference(written).1;
    let (path, argument) = match t.find('<') {
        Some(open) if t.ends_with('>') => {
            (&t[..open], Some(first_argument(&t[open + 1..t.len() - 1])))
        }
        _ => (t, None),
    };
    let argument = argument.filter(|a| !a.is_empty() && !a.starts_with('\''));
    (path.rsplit("::").next().unwrap_or_default().trim(), argument)
}

struct Struct {
    file: usize,
    fields: Vec<(String, String)>,
    watched: bool,
}

struct Function {
    file: usize,
    of: Option<String>,
    name: String,
    /// Where the body's first statement goes.
    at: usize,
    by_reference: bool,
    parameters: Vec<(String, String)>,
    left_out: Option<String>,
}

#[derive(Default)]
struct Model {
    structs: BTreeMap<String, Struct>,
    enums: BTreeMap<String, Vec<String>>,
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

/// `Some("")` for test code; otherwise why the item is not taken, if so.
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
    for node in walk(root) {
        let kind = node.kind();
        let is_struct = kind == rules.r#struct.node.as_str();
        let is_enum = kind == rules.r#enum.node.as_str();
        let is_function = kind == rules.function.node.as_str();
        if !(is_struct || is_enum || is_function) {
            continue;
        }
        let state = standing(rules, &node);
        if state.as_deref() == Some("") {
            continue;
        }
        if is_struct {
            let Some(name) = node.field(&rules.r#struct.name).map(|n| text(&n)) else {
                continue;
            };
            let fields = walk(&node)
                .iter()
                .filter(|f| f.kind() == rules.field.node.as_str())
                .filter_map(|f| Some((text(&f.field(&rules.field.name)?), text(&f.field(&rules.field.r#type)?))))
                .collect();
            let watched = state.is_none() && node.field(&rules.r#struct.generics).is_none();
            if model.structs.insert(name.clone(), Struct { file, fields, watched }).is_some() {
                model.twice.insert(name);
            }
        } else if is_enum {
            let Some(name) = node.field(&rules.r#enum.name).map(|n| text(&n)) else {
                continue;
            };
            let variants = walk(&node)
                .iter()
                .filter(|v| v.kind() == rules.r#enum.variant.as_str())
                .filter_map(|v| v.field(&rules.r#enum.variant_name).map(|n| text(&n)))
                .collect();
            model.enums.insert(name, variants);
        } else {
            let above = ancestors(&node);
            if above.iter().any(|a| a.kind() == rules.function.node.as_str()) {
                continue;
            }
            let (Some(name), Some(body)) = (node.field(&rules.function.name).map(|n| text(&n)), node.field(&rules.function.body)) else {
                continue;
            };
            let implemented = above.iter().find(|a| a.kind() == rules.r#impl.node.as_str());
            let of = implemented.and_then(|i| i.field(&rules.r#impl.r#type)).map(|t| head_and_argument(&text(&t)).0.to_owned());
            let modifiers = node.children().find(|c| c.kind() == rules.function.modifiers.as_str()).map(|m| text(&m)).unwrap_or_default();
            let left_out = state
                .or_else(|| implemented.and_then(|i| i.field(&rules.r#impl.r#trait)).map(|_| "implements a trait".to_owned()))
                .or_else(|| node.field(&rules.function.generics).map(|_| "it has parameters of its own".to_owned()))
                .or_else(|| rules.function.left_out.iter().find(|w| modifiers.split_whitespace().any(|m| m == w.as_str())).map(|w| format!("it is `{w}`")));
            let own: Vec<SgNode<'_>> = node.field(&rules.function.parameters).map(|p| p.children().collect()).unwrap_or_default();
            let by_reference = own.iter().find(|p| p.kind() == rules.receiver.node.as_str()).is_some_and(|r| r.text().starts_with(rules.receiver.shared.as_str()));
            let parameters = own
                .iter()
                .filter(|p| p.kind() == rules.parameter.node.as_str())
                .filter_map(|p| Some((text(&p.field(&rules.parameter.name)?), text(&p.field(&rules.parameter.r#type)?))))
                .collect();
            model.functions.push(Function { file, of, name, at: body.range().start + 1, by_reference, parameters, left_out });
        }
    }
}

struct Maker<'a> {
    rules: &'a Rules,
    model: &'a Model,
}

impl Maker<'_> {
    /// A written type as the module inside the file has to write it.
    fn fix(&self, written: &str) -> String {
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
        out
    }

    /// The holders below a place, each with the expression that reads it.
    /// `guarded` says the place already lies behind a lock.
    fn observe(&self, file: usize, place: &str, written: &str, name: &str, guarded: bool, depth: usize, out: &mut Vec<(String, String)>) {
        let (head, argument) = head_and_argument(written);
        if depth > 6 {
            return;
        }
        if let Some(how) = self.rules.observe.get(head) {
            out.push((name.to_owned(), format!("u8::from({})", how.replace("{x}", place))));
        } else if let Some(how) = self.rules.enter.get(head) {
            if let Some(inner) = argument {
                self.observe(file, &how.replace("{x}", place), inner, name, guarded, depth + 1, out);
            }
        } else if let Some(how) = self.rules.enter_if_free.get(head) {
            // One lock deep; what lies behind a second lock is not read.
            if let (Some(inner), false) = (argument, guarded) {
                let mut behind = Vec::new();
                self.observe(file, &self.rules.inside, inner, name, true, depth + 1, &mut behind);
                let attempt = how.replace("{x}", place);
                out.extend(behind.into_iter().map(|(n, read)| (n, self.rules.if_free.replace("{try}", &attempt).replace("{read}", &read))));
            }
        } else if let Some(declared) = self.model.structs.get(head).filter(|s| s.file == file && argument.is_none()) {
            for (field, of) in &declared.fields {
                self.observe(file, &format!("{place}.{field}"), of, &format!("{name}.{field}"), guarded, depth + 1, out);
            }
        } else if let (Some(variants), None) = (self.model.enums.get(head), argument) {
            let named = self.fix(without_reference(written).1);
            let arms: Vec<String> = variants
                .iter()
                .enumerate()
                .map(|(n, v)| self.rules.variant.replace("{E}", &named).replace("{V}", v).replace("{n}", &n.to_string()))
                .collect();
            out.push((name.to_owned(), self.rules.pick.replace("{x}", place).replace("{arms}", &arms.join(" "))));
        }
    }
}

fn simple_name(pattern: &str) -> Option<&str> {
    let name = pattern.strip_prefix("mut ").unwrap_or(pattern).trim();
    let plain = !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') && name != "_";
    plain.then_some(name)
}

fn instrument(args: &[String]) -> Result<Value, String> {
    let [rules_path, root, root_module, files @ ..] = args else {
        return Err("usage: scope-watch instrument RULES ROOT_DIRECTORY ROOT_MODULE FILE...".into());
    };
    let rules_text = fs::read_to_string(rules_path).map_err(|e| format!("{rules_path}: {e}"))?;
    let rules: Rules = serde_json::from_str(&rules_text).map_err(|e| e.to_string())?;
    let mut model = Model::default();
    let mut sources = Vec::new();
    for (at, file) in files.iter().enumerate() {
        let source = fs::read_to_string(Path::new(root).join(file)).map_err(|e| format!("{file}: {e}"))?;
        let tree = parse_clean(file, &source, &rules.language)?;
        read(&rules, at, &tree.root(), &mut model);
        sources.push(source);
    }
    let maker = Maker { rules: &rules, model: &model };
    let stems: Vec<String> = files.iter().map(|f| f.replace('\\', "/").trim_end_matches(".rs").to_owned()).collect();
    let part_path = |file: usize| format!("{root_module}::{}::{PART_MODULE}", stems[file].replace('/', "::"));

    // The types that are watched, with their holders.
    let mut holders_of: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for (name, declared) in model.structs.iter().filter(|(n, s)| s.watched && !model.twice.contains(*n)) {
        let mut holders = Vec::new();
        for (field, of) in &declared.fields {
            maker.observe(declared.file, &format!("f.{field}"), of, field, false, 0, &mut holders);
        }
        if !holders.is_empty() {
            holders_of.insert(name.clone(), holders);
        }
    }

    // The statements, per file, by where they go.
    let mut added: Vec<Vec<(usize, String)>> = vec![Vec::new(); files.len()];
    let (mut watched_at, mut not_watched) = (Vec::new(), Vec::new());
    for f in &model.functions {
        let label = match &f.of {
            Some(t) => format!("{t}::{}", f.name),
            None => f.name.clone(),
        };
        let mut statements = Vec::new();
        if let (true, Some(t)) = (f.by_reference, f.of.as_ref().filter(|t| holders_of.contains_key(*t))) {
            statements.push((t.clone(), rules.statement_for_receiver.clone(), rules.receiver.name.clone()));
        }
        for (pattern, written) in &f.parameters {
            let (by_reference, core) = without_reference(written);
            let (head, argument) = head_and_argument(core);
            let named = if head == rules.shared { argument.map(|a| head_and_argument(a).0) } else { Some(head).filter(|_| argument.is_none()) };
            let named = named.map(|n| if n == rules.r#impl.own_type { f.of.as_deref().unwrap_or(n) } else { n });
            let (Some(t), true, Some(value)) = (named.filter(|t| holders_of.contains_key(*t)), by_reference, simple_name(pattern)) else {
                continue;
            };
            statements.push((t.to_owned(), rules.statement.clone(), value.to_owned()));
        }
        if statements.is_empty() {
            continue;
        }
        if let Some(why) = &f.left_out {
            not_watched.push(json!({"function": label, "why": why}));
            continue;
        }
        let mut line = String::new();
        for (n, (t, statement, value)) in statements.iter().enumerate() {
            let written = statement
                .replace("{local}", &format!("_lesscode_watch_{n}"))
                .replace("{part}", &part_path(model.structs[t].file))
                .replace("{T}", t)
                .replace("{value}", value)
                .replace("{at:?}", &format!("{label:?}"));
            line.push_str(&format!(" {written}"));
        }
        added[f.file].push((f.at, line));
        watched_at.push(json!({"function": label, "types": statements.iter().map(|s| s.0.clone()).collect::<Vec<_>>()}));
    }

    // The files with the statements in, the module inside each, and the collector.
    let mut parts = Vec::new();
    for (at, file) in files.iter().enumerate() {
        let mut source = sources[at].clone();
        added[at].sort_by(|a, b| b.0.cmp(&a.0));
        for (offset, line) in &added[at] {
            source.insert_str(*offset, line);
        }
        let types: Vec<String> = holders_of
            .iter()
            .filter(|(name, _)| model.structs[*name].file == at)
            .map(|(name, holders)| {
                let pushes: String = holders.iter().map(|(_, read)| format!("    s.push({read});\n")).collect();
                format!(
                    "pub(crate) fn watch_{name}(f: &{name}, at: &'static str) -> Watch {{\n    Watch::new({name:?}, at, f as *const {name} as *const (), read_{name})\n}}\n\nfn read_{name}(p: *const ()) -> Vec<u8> {{\n    // The value was taken by reference by the function that is being watched.\n    let f: &{name} = unsafe {{ &*(p as *const {name}) }};\n    let mut s = Vec::new();\n{pushes}    s\n}}\n"
                )
            })
            .collect();
        let module = PART.replace("@@FILE@@", file).replace("@@ROOT@@", root_module).replace("@@TYPES@@", &types.join("\n"));
        let target = Path::new(root).join(&stems[at]).join(format!("{PART_MODULE}.rs"));
        fs::create_dir_all(target.parent().expect("a directory")).map_err(|e| e.to_string())?;
        fs::write(&target, &module).map_err(|e| format!("{}: {e}", target.display()))?;
        source.push_str(&format!("\n#[cfg(test)]\npub(crate) mod {PART_MODULE};\n"));
        fs::write(Path::new(root).join(file), &source).map_err(|e| format!("{file}: {e}"))?;
        parts.push(json!({"file": file, "statements": added[at].len(), "module": target.display().to_string(), "sha256": sha256_hex(source.as_bytes())}));
    }
    let target = Path::new(root).join(format!("{WATCH_MODULE}.rs"));
    fs::write(&target, WATCH).map_err(|e| format!("{}: {e}", target.display()))?;

    let types: Vec<Value> = holders_of.iter().map(|(name, holders)| json!({"type": name, "file": files[model.structs[name].file], "holders": holders.iter().map(|(n, _)| n).collect::<Vec<_>>()})).collect();
    Ok(json!({
        "receipt": "scope-watch-r170", "revision": 1,
        "rules_sha256": sha256_hex(rules_text.as_bytes()),
        "collector": {"module": target.display().to_string(), "declaration": format!("#[cfg(test)]\npub(crate) mod {WATCH_MODULE};")},
        "parts": parts, "types": types,
        "functions": {"read": model.functions.len(), "watched": watched_at.len(), "watched_at": watched_at, "not_watched": not_watched},
    }))
}

fn collect(args: &[String]) -> Result<Value, String> {
    let [made_path, directory] = args else {
        return Err("usage: scope-watch collect GEN_JSON DIRECTORY".into());
    };
    let made: Value = serde_json::from_str(&fs::read_to_string(made_path).map_err(|e| format!("{made_path}: {e}"))?).map_err(|e| e.to_string())?;
    #[derive(Default)]
    struct Shape {
        tests: BTreeSet<String>,
        at: BTreeSet<String>,
    }
    #[derive(Default)]
    struct Seen {
        shapes: BTreeMap<Vec<u8>, Shape>,
        /// Function, shape before, shape after, where they differ.
        changes: BTreeSet<(String, Vec<u8>, Vec<u8>)>,
        functions: BTreeSet<String>,
        not_read: usize,
        notes: usize,
    }
    let mut seen: BTreeMap<String, Seen> = BTreeMap::new();
    let (mut files, mut lines) = (0usize, 0usize);
    for entry in fs::read_dir(directory).map_err(|e| format!("{directory}: {e}"))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().is_none_or(|e| e != "tsv") {
            continue;
        }
        files += 1;
        for line in fs::read_to_string(&path).map_err(|e| e.to_string())?.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            let [of, at, before, after, test] = parts.as_slice() else {
                continue;
            };
            let (Ok(before), Ok(after)) = (serde_json::from_str::<Vec<u8>>(before), serde_json::from_str::<Vec<u8>>(after)) else {
                continue;
            };
            lines += 1;
            let entry = seen.entry((*of).to_owned()).or_default();
            entry.notes += 1;
            entry.functions.insert((*at).to_owned());
            let read = |values: &[u8]| !values.contains(&255);
            for values in [&before, &after] {
                if read(values) {
                    let shape = entry.shapes.entry(values.clone()).or_default();
                    shape.tests.insert((*test).to_owned());
                    shape.at.insert((*at).to_owned());
                } else {
                    entry.not_read += 1;
                }
            }
            if before != after && read(&before) && read(&after) {
                entry.changes.insert(((*at).to_owned(), before, after));
            }
        }
    }
    let types: Vec<Value> = made["types"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .map(|t| {
            let name = t["type"].as_str().unwrap_or_default();
            let Some(s) = seen.get(name) else {
                return json!({"type": name, "holders": t["holders"], "seen": false});
            };
            let shapes: Vec<Value> = s.shapes.iter().map(|(values, shape)| json!({"values": values, "tests": shape.tests.len(), "one_test": shape.tests.iter().next(), "at": shape.at})).collect();
            let changes: Vec<Value> = s.changes.iter().map(|(at, before, after)| json!({"at": at, "before": before, "after": after})).collect();
            json!({"type": name, "holders": t["holders"], "seen": true, "notes": s.notes, "readings_with_a_lock_held": s.not_read,
                   "functions": s.functions, "shapes": shapes, "changes": changes})
        })
        .collect();
    Ok(json!({"receipt": "scope-watch-collect-r170", "processes": files, "notes": lines, "types": types}))
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let outcome = match args.split_first() {
        Some((command, rest)) if command == "instrument" => instrument(rest),
        Some((command, rest)) if command == "collect" => collect(rest),
        _ => Err("usage: scope-watch instrument|collect ...".into()),
    };
    match outcome {
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
