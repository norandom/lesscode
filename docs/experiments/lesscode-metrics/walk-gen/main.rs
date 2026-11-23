//! Writes a walk over a type's methods (R167): from a source file, the name
//! of a type and a rules file, a test module that builds a value, calls its
//! methods in every order up to a length and writes down the shapes it saw.
//! What the module looks like is in `walk_template.rs`; which syntax means
//! what, how a holder is read and which values are tried is in the rules.

use lesscode_core::adapters::driver::tree::{ancestors, parse_clean, walk, SgNode};
use lesscode_core::receipt::sha256_hex;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::{env, fs, process::ExitCode};

const TEMPLATE: &str = include_str!("walk_template.rs");

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
    returns: String,
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
struct TestScope {
    node: String,
    marker: String,
}
#[derive(Deserialize)]
struct Construct {
    names: Vec<String>,
    default: String,
    call: String,
}
#[derive(Deserialize)]
struct Rules {
    language: String,
    r#struct: Named,
    field: FieldRule,
    r#enum: EnumRule,
    r#impl: ImplRule,
    function: FunctionRule,
    receiver: ReceiverRule,
    parameter: ParameterRule,
    attribute: String,
    comments: Vec<String>,
    test_scope: TestScope,
    /// Texts an attribute holds when the type can be built by default.
    derive_default: Vec<String>,
    length: usize,
    /// How a holder of this type is read as true or false; `{x}` is the place.
    observe: BTreeMap<String, String>,
    /// How to get at what a wrapper holds.
    enter: BTreeMap<String, String>,
    /// One arm for a variant, and the expression that picks among arms.
    variant: String,
    pick: String,
    /// Wrappers behind which a value cannot be changed through `&mut`.
    shared: Vec<String>,
    /// Wrappers a returned type is looked through.
    wrappers: Vec<String>,
    construct: Construct,
    call: String,
    chain: String,
    /// Values to try for a parameter, by its written type.
    values: BTreeMap<String, Vec<String>>,
    /// The same, by the last name of the type's path.
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

#[derive(Clone, Copy, PartialEq)]
enum Receiver {
    None,
    Shared,
    Changing,
    Owned,
}

struct Method {
    name: String,
    receiver: Receiver,
    /// Written types of the parameters after the receiver.
    parameters: Vec<String>,
    returns: Option<String>,
    test_only: bool,
}

#[derive(Default)]
struct Model {
    structs: BTreeMap<String, Vec<(String, String)>>,
    enums: BTreeMap<String, Vec<String>>,
    methods: BTreeMap<String, Vec<Method>>,
    built_by_default: BTreeSet<String>,
}

/// The attributes written directly before a node.
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

fn in_tests(rules: &Rules, node: &SgNode<'_>) -> bool {
    ancestors(node).iter().any(|a| {
        a.kind() == rules.test_scope.node.as_str()
            && attributes(rules, a).iter().any(|t| t.contains(&rules.test_scope.marker))
    })
}

fn read(rules: &Rules, root: &SgNode<'_>) -> Model {
    let mut model = Model::default();
    for node in walk(root) {
        let kind = node.kind();
        if in_tests(rules, &node) {
            continue;
        }
        if kind == rules.r#struct.node.as_str() {
            let Some(name) = node.field(&rules.r#struct.name).map(|n| text(&n)) else {
                continue;
            };
            let fields = walk(&node)
                .iter()
                .filter(|f| f.kind() == rules.field.node.as_str())
                .filter_map(|f| Some((text(&f.field(&rules.field.name)?), text(&f.field(&rules.field.r#type)?))))
                .collect();
            let marks = attributes(rules, &node);
            if marks.iter().any(|a| rules.derive_default.iter().all(|part| a.contains(part.as_str()))) {
                model.built_by_default.insert(name.clone());
            }
            model.structs.insert(name, fields);
        } else if kind == rules.r#enum.node.as_str() {
            let Some(name) = node.field(&rules.r#enum.name).map(|n| text(&n)) else {
                continue;
            };
            let variants = walk(&node)
                .iter()
                .filter(|v| v.kind() == rules.r#enum.variant.as_str())
                .filter_map(|v| v.field(&rules.r#enum.variant_name).map(|n| text(&n)))
                .collect();
            model.enums.insert(name, variants);
        } else if kind == rules.function.node.as_str() {
            let implemented = ancestors(&node).into_iter().find(|a| a.kind() == rules.r#impl.node.as_str());
            let Some(implemented) = implemented.filter(|i| i.field(&rules.r#impl.r#trait).is_none()) else {
                continue;
            };
            let (Some(of), Some(name)) = (implemented.field(&rules.r#impl.r#type), node.field(&rules.function.name)) else {
                continue;
            };
            let below = walk(&node);
            let receiver = match below.iter().find(|p| p.kind() == rules.receiver.node.as_str()).map(text) {
                None => Receiver::None,
                Some(written) if written.starts_with(&rules.receiver.changing) || (written.starts_with(&rules.receiver.shared) && written.contains("mut ")) => Receiver::Changing,
                Some(written) if written.starts_with(&rules.receiver.shared) => Receiver::Shared,
                Some(_) => Receiver::Owned,
            };
            // Only the function's own parameters, not those of a closure inside it.
            let own = |p: &SgNode<'_>| {
                ancestors(p)
                    .iter()
                    .find(|a| a.kind() == rules.function.node.as_str() || a.kind().ends_with("closure_expression"))
                    .is_some_and(|a| a.range() == node.range())
            };
            let parameters = below
                .iter()
                .filter(|p| p.kind() == rules.parameter.node.as_str() && own(p))
                .filter_map(|p| p.field(&rules.parameter.r#type).map(|t| text(&t)))
                .collect();
            let test_only = attributes(rules, &node).iter().any(|t| t.contains(&rules.test_scope.marker));
            model.methods.entry(head_and_argument(&text(&of)).0.to_owned()).or_default().push(Method {
                name: text(&name),
                receiver,
                parameters,
                returns: node.field(&rules.function.returns).map(|t| text(&t)),
                test_only,
            });
        }
    }
    model
}

struct Maker<'a> {
    rules: &'a Rules,
    model: &'a Model,
}

impl Maker<'_> {
    /// The type of the file a written type holds, through wrappers.
    fn held(&self, written: &str, own: &str) -> Option<String> {
        let mut at = written;
        for _ in 0..8 {
            let (head, argument) = head_and_argument(at);
            if head == self.rules.r#impl.own_type {
                return Some(own.to_owned());
            }
            if self.model.structs.contains_key(head) {
                return Some(head.to_owned());
            }
            let wraps = has(&self.rules.wrappers, head) || self.rules.enter.contains_key(head);
            at = argument.filter(|_| wraps)?;
        }
        None
    }

    /// The holders below a place, each with the expression that reads it.
    fn observe(&self, place: &str, written: &str, name: &str, depth: usize, out: &mut Vec<(String, String)>) {
        let (head, argument) = head_and_argument(written);
        if depth > 6 {
            return;
        }
        if let Some(how) = self.rules.observe.get(head) {
            out.push((name.to_owned(), format!("u8::from({})", how.replace("{x}", place))));
        } else if let Some(how) = self.rules.enter.get(head) {
            if let Some(inner) = argument {
                self.observe(&how.replace("{x}", place), inner, name, depth + 1, out);
            }
        } else if let Some(fields) = self.model.structs.get(head) {
            for (field, of) in fields {
                self.observe(&format!("{place}.{field}"), of, &format!("{name}.{field}"), depth + 1, out);
            }
        } else if let Some(variants) = self.model.enums.get(head) {
            let arms: Vec<String> = variants
                .iter()
                .enumerate()
                .map(|(n, v)| self.rules.variant.replace("{E}", head).replace("{V}", v).replace("{n}", &n.to_string()))
                .collect();
            out.push((name.to_owned(), self.rules.pick.replace("{x}", place).replace("{arms}", &arms.join(" "))));
        }
    }

    fn values(&self, written: &str) -> Option<&Vec<String>> {
        let plain: String = written.split_whitespace().collect::<Vec<_>>().join(" ");
        self.rules.values.get(&plain).or_else(|| self.rules.values_by_head.get(head_and_argument(written).0))
    }

    /// Argument lists to try: the middle value everywhere, then one parameter
    /// changed at a time. `Err` names a parameter type without values.
    fn arguments(&self, parameters: &[String]) -> Result<Vec<String>, String> {
        let tried: Vec<&Vec<String>> = parameters
            .iter()
            .map(|p| self.values(p).ok_or_else(|| p.clone()))
            .collect::<Result<_, _>>()?;
        let middle: Vec<usize> = tried.iter().map(|v| 1.min(v.len() - 1)).collect();
        let written = |pick: &[usize]| pick.iter().zip(&tried).map(|(&i, v)| v[i].clone()).collect::<Vec<_>>().join(", ");
        let mut out = vec![written(&middle)];
        for (at, values) in tried.iter().enumerate() {
            for other in (0..values.len()).filter(|&i| i != middle[at]) {
                let mut pick = middle.clone();
                pick[at] = other;
                out.push(written(&pick));
            }
        }
        Ok(out)
    }

    /// The largest value for every parameter: a value is built roomy.
    fn largest(&self, parameters: &[String]) -> Result<String, String> {
        let picked: Vec<String> = parameters
            .iter()
            .map(|p| self.values(p).and_then(|v| v.last().cloned()).ok_or_else(|| p.clone()))
            .collect::<Result<_, _>>()?;
        Ok(picked.join(", "))
    }
}

fn entry(label: &str, code: &str) -> String {
    format!("    ({label:?}, |f| {{ {code} }}),")
}

fn run() -> Result<Value, String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let [rules_path, source_path, subject, out_path] = args.as_slice() else {
        return Err("usage: walk-gen RULES SOURCE TYPE OUT".into());
    };
    let rules_text = fs::read_to_string(rules_path).map_err(|e| format!("{rules_path}: {e}"))?;
    let rules: Rules = serde_json::from_str(&rules_text).map_err(|e| e.to_string())?;
    let source = fs::read_to_string(source_path).map_err(|e| format!("{source_path}: {e}"))?;
    let tree = parse_clean(source_path, &source, &rules.language)?;
    let model = read(&rules, &tree.root());
    let maker = Maker { rules: &rules, model: &model };
    let fields = model.structs.get(subject).ok_or_else(|| format!("no struct `{subject}` outside test code"))?;
    let none = Vec::new();
    let methods = model.methods.get(subject).unwrap_or(&none);

    // How a value is built.
    let returns_it = |m: &&Method| {
        m.receiver == Receiver::None
            && !m.test_only
            && m.returns.as_deref().and_then(|r| maker.held(r, subject)).as_deref() == Some(subject.as_str())
    };
    let builders: Vec<&Method> = methods.iter().filter(returns_it).collect();
    let builder = builders.iter().find(|m| has(&rules.construct.names, &m.name)).or(builders.first());
    let (subject_type, make, built_by) = match builder {
        Some(m) => {
            let arguments = maker.largest(&m.parameters).map_err(|p| format!("no value for `{p}` to build a `{subject}`"))?;
            let call = rules.construct.call.replace("{T}", subject).replace("{f}", &m.name).replace("{args}", &arguments);
            let returned = m.returns.clone().unwrap_or_default().replace(&rules.r#impl.own_type, subject);
            (returned, call.clone(), call)
        }
        None if model.built_by_default.contains(subject) => {
            let call = rules.construct.default.replace("{T}", subject);
            (subject.clone(), call.clone(), call)
        }
        None => return Err(format!("no way found to build a `{subject}`")),
    };
    let behind_shared = has(&rules.shared, head_and_argument(&subject_type).0);

    let mut holders = Vec::new();
    for (field, of) in fields {
        maker.observe(&format!("f.{field}"), of, field, 0, &mut holders);
    }

    let (mut direct, mut chained, mut test_only) = (Vec::new(), Vec::new(), Vec::new());
    let (mut called, mut not_called) = (BTreeSet::new(), Vec::new());
    let callable = |m: &Method| match m.receiver {
        Receiver::Shared => Ok(()),
        Receiver::Changing if !behind_shared => Ok(()),
        Receiver::Changing => Err("needs `&mut`, and the value is shared".to_owned()),
        Receiver::Owned => Err("takes the value itself".to_owned()),
        Receiver::None => Err("has no receiver".to_owned()),
    };
    for m in methods {
        if builder.is_some_and(|b| b.name == m.name) {
            continue;
        }
        let lists = callable(m).and_then(|()| maker.arguments(&m.parameters).map_err(|p| format!("no value for a parameter of type `{p}`")));
        let lists = match lists {
            Ok(lists) => lists,
            Err(why) => {
                not_called.push(json!({"method": format!("{subject}::{}", m.name), "why": why}));
                continue;
            }
        };
        called.insert(format!("{subject}::{}", m.name));
        for arguments in &lists {
            let code = rules.call.replace("{m}", &m.name).replace("{args}", arguments);
            let row = entry(&format!("{}({arguments})", m.name), &code);
            if m.test_only { test_only.push(row) } else { direct.push(row) }
        }
        // Methods of another type of the file, on what this one returns.
        let other = m.returns.as_deref().and_then(|r| maker.held(r, subject)).filter(|t| t != subject);
        let (Some(other), false) = (other, m.test_only) else {
            continue;
        };
        let first = &lists[0];
        for n in model.methods.get(&other).unwrap_or(&none) {
            let more = match n.receiver {
                Receiver::Shared | Receiver::Changing => maker.arguments(&n.parameters),
                _ => Err("it does not take its receiver by reference".to_owned()),
            };
            match more {
                Ok(more) => {
                    called.insert(format!("{other}::{}", n.name));
                    for arguments in more {
                        let code = rules.chain.replace("{m}", &m.name).replace("{args}", first).replace("{n}", &n.name).replace("{more}", &arguments);
                        chained.push(entry(&format!("{}({first}) > {}({arguments})", m.name, n.name), &code));
                    }
                }
                Err(why) => not_called.push(json!({"method": format!("{other}::{}", n.name), "why": why})),
            }
        }
    }

    let names: Vec<String> = holders.iter().map(|(n, _)| format!("{n:?}")).collect();
    let reads: Vec<String> = holders.iter().map(|(_, e)| format!("    s.push({e});")).collect();
    let module = TEMPLATE
        .replace("@@TYPE@@", subject)
        .replace("@@SUBJECT@@", &subject_type)
        .replace("@@LENGTH@@", &rules.length.to_string())
        .replace("@@NAMES@@", &names.join(", "))
        .replace("@@MAKE@@", &make)
        .replace("@@OBSERVE@@", &reads.join("\n"))
        .replace("@@DIRECT@@", &direct.join("\n"))
        .replace("@@CHAINED@@", &chained.join("\n"))
        .replace("@@TEST_ONLY@@", &test_only.join("\n"));
    fs::write(out_path, &module).map_err(|e| format!("{out_path}: {e}"))?;
    Ok(json!({
        "receipt": "walk-gen-r167", "revision": 1,
        "rules_sha256": sha256_hex(rules_text.as_bytes()),
        "source_sha256": sha256_hex(source.as_bytes()),
        "module_sha256": sha256_hex(module.as_bytes()),
        "type": subject, "built_by": built_by, "value_type": subject_type,
        "holders": holders.iter().map(|(n, _)| n).collect::<Vec<_>>(),
        "calls": {"own": direct.len(), "of_what_they_return": chained.len(), "test_only": test_only.len()},
        "methods_called": called, "methods_not_called": not_called,
        "length": rules.length,
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
