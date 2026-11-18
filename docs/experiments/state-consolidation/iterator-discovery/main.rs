//! Select Java source reachable from a public iterator protocol.
mod citations;
mod defaults;
mod guard_trial;
use lesscode_core::{
    adapters::driver::tree::{self, SgNode},
    receipt::sha256_hex,
};
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    env,
    error::Error,
    fs,
    io::Write,
    path::Path,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Lines = BTreeMap<String, BTreeMap<usize, String>>;
const QUESTION: &str = include_str!("question.txt");
const DEPTH: usize = 4;
const MAX_METHOD_LINES: usize = 320;
const TOP_API: [&str; 3] = ["iterator", "close", "getRecordNumber"];
const ITER_API: [&str; 2] = ["hasNext", "next"];

fn field(node: &SgNode<'_>, key: &str) -> String {
    node.field(key)
        .map(|n| n.text().into_owned())
        .unwrap_or_default()
}
fn classes(node: &SgNode<'_>) -> Vec<String> {
    tree::ancestors(node)
        .iter()
        .filter(|n| n.kind() == "class_declaration")
        .map(|n| field(n, "name"))
        .collect()
}
fn public(node: &SgNode<'_>) -> bool {
    node.children()
        .any(|m| m.kind() == "modifiers" && m.text().split_whitespace().any(|w| w == "public"))
}
struct Method {
    file: String,
    owners: Vec<String>,
    name: String,
    calls: BTreeSet<(String, String)>,
    identifiers: BTreeSet<String>,
    span: (usize, usize),
    public: bool,
    static_no_args: bool,
}
struct Field {
    owner: String,
    scope: String,
    name: String,
    ty: String,
    file: String,
    line: usize,
    last_line: usize,
    initializer: Option<String>,
    static_field: bool,
    factories: Vec<(String, String)>,
}
#[derive(Default)]
struct Index {
    sources: BTreeMap<String, String>,
    methods: Vec<Method>,
    fields: Vec<Field>,
    iterable: BTreeSet<String>,
    iterators: BTreeSet<String>,
    init: Vec<(String, String, String, usize)>,
    types: BTreeSet<String>,
    reads: Vec<defaults::Read>,
}
impl Index {
    fn scan(&mut self, path: &Path) -> Result<()> {
        let file = path
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or("invalid filename")?
            .to_owned();
        let source = fs::read_to_string(path)?;
        let ast = tree::parse(&source, tree::grammar("java")?)?;
        if tree::first_error(&ast.root()).is_some() {
            return Err(format!("parse error in {file}").into());
        }
        for n in tree::walk(&ast.root()) {
            match n.kind().as_ref() {
                "class_declaration" => {
                    let interfaces = field(&n, "interfaces");
                    let name = field(&n, "name");
                    let mut enclosing = classes(&n);
                    enclosing.reverse();
                    enclosing.push(name.clone());
                    self.types.insert(enclosing.join("."));
                    if public(&n)
                        && classes(&n).is_empty()
                        && interfaces
                            .split(|c: char| !c.is_alphanumeric() && c != '_')
                            .any(|t| t == "Iterable")
                    {
                        self.iterable.insert(name.clone());
                    }
                    if interfaces
                        .split(|c: char| !c.is_alphanumeric() && c != '_')
                        .any(|t| t == "Iterator")
                    {
                        self.iterators.insert(name);
                    }
                }
                "method_declaration" => {
                    let owners = classes(&n);
                    if owners.is_empty() {
                        continue;
                    }
                    let mut calls = BTreeSet::new();
                    let mut identifiers = BTreeSet::new();
                    for child in tree::walk(&n) {
                        match child.kind().as_ref() {
                            "method_invocation" => {
                                calls.insert((field(&child, "object"), field(&child, "name")));
                            }
                            "method_reference" => {
                                if let Some((r, m)) = child.text().rsplit_once("::") {
                                    calls.insert((r.into(), m.into()));
                                }
                            }
                            "identifier" => {
                                identifiers.insert(child.text().into_owned());
                            }
                            _ => {}
                        }
                    }
                    let (line, _, end, _) = tree::position(&n);
                    self.methods.push(Method {
                        file: file.clone(),
                        owners,
                        name: field(&n, "name"),
                        calls,
                        identifiers,
                        span: (line, end),
                        public: public(&n),
                        static_no_args: defaults::modifier(&n, "static")
                            && defaults::empty_list(&n, "parameters"),
                    });
                }
                "field_declaration" => {
                    let owners = classes(&n);
                    if let Some(owner) = owners.first() {
                        for d in n.children().filter(|d| d.kind() == "variable_declarator") {
                            self.fields.push(Field {
                                owner: owner.clone(),
                                scope: owners.iter().rev().cloned().collect::<Vec<_>>().join("."),
                                name: field(&d, "name"),
                                ty: format!("{}{}", field(&n, "type"), field(&d, "dimensions")),
                                file: file.clone(),
                                line: tree::position(&d).0,
                                last_line: tree::position(&d).2,
                                initializer: d.field("value").map(|v| v.text().into_owned()),
                                static_field: defaults::modifier(&n, "static"),
                                factories: defaults::factories(&d),
                            });
                        }
                    }
                }
                "assignment_expression"
                    if tree::ancestors(&n)
                        .iter()
                        .any(|p| p.kind() == "constructor_declaration") =>
                {
                    if let Some(owner) = classes(&n).first() {
                        self.reads.extend(defaults::reads(&n, &file));
                        let lhs = field(&n, "left");
                        self.init.push((
                            owner.clone(),
                            lhs.trim_start_matches("this.").into(),
                            file.clone(),
                            tree::position(&n).0,
                        ));
                    }
                }
                _ => {}
            }
        }
        self.sources.insert(file, source);
        Ok(())
    }
    fn targets(&self, m: &Method, receiver: &str, name: &str) -> Vec<usize> {
        let owner = receiver.strip_suffix(".this").unwrap_or(receiver);
        let mut types = Vec::new();
        if owner.is_empty() || owner == "this" {
            types.extend(m.owners.iter().map(String::as_str));
        } else {
            let receiver_name = owner.strip_prefix("this.").unwrap_or(owner);
            types.push(receiver_name);
            for f in &self.fields {
                if m.owners.contains(&f.owner) && f.name == receiver_name {
                    types.push(&f.ty);
                }
            }
        }
        self.methods
            .iter()
            .enumerate()
            .filter(|(_, t)| t.name == name && types.contains(&t.owners[0].as_str()))
            .map(|(i, _)| i)
            .collect()
    }
}
fn save(path: &Path, value: &serde_json::Value) -> Result<()> {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    f.write_all(&serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn run() -> Result<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if let [cmd, left, right, z3, out] = args.as_slice() {
        if cmd == "guard" {
            return guard_trial::compare(left.as_ref(), right.as_ref(), z3.as_ref(), out.as_ref());
        }
    }
    if let [cmd, selection, answer, out] = args.as_slice() {
        if cmd == "citations" {
            return citations::run(selection.as_ref(), answer.as_ref(), out.as_ref());
        }
    }
    let [src, out] = args.as_slice() else {
        return Err("iterator-discovery PACKAGE_DIRECTORY NEW_OUTPUT_DIRECTORY | citations SELECTION ANSWER NEW_REPORT | guard LEFT_PARSER RIGHT_PARSER Z3 NEW_DIRECTORY".into());
    };
    let mut files = fs::read_dir(src)?
        .map(|e| e.map(|e| e.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    files.sort();
    let mut index = Index::default();
    for f in files
        .iter()
        .filter(|f| f.extension().is_some_and(|e| e == "java"))
    {
        index.scan(f)?;
    }
    let case = index
        .iterable
        .iter()
        .find(|owner| {
            TOP_API.iter().all(|name| {
                index
                    .methods
                    .iter()
                    .any(|m| m.public && &m.owners[0] == *owner && &m.name == name)
            })
        })
        .ok_or("no eligible public protocol")?;
    let mut queue = VecDeque::new();
    for (i, m) in index.methods.iter().enumerate() {
        if m.public
            && m.owners.last() == Some(case)
            && ((m.owners.len() == 1 && TOP_API.contains(&m.name.as_str()))
                || (index.iterators.contains(&m.owners[0]) && ITER_API.contains(&m.name.as_str())))
        {
            queue.push_back((i, 0));
        }
    }
    let mut seen = BTreeSet::new();
    let mut lines = Lines::new();
    let mut unresolved = BTreeSet::new();
    let mut ambiguous = BTreeSet::new();
    let mut omitted = BTreeSet::new();
    let mut fields = BTreeSet::new();
    let mut picked = Vec::new();
    let mut budget = MAX_METHOD_LINES;
    while let Some((i, depth)) = queue.pop_front() {
        if !seen.insert(i) {
            continue;
        }
        let m = &index.methods[i];
        let id = format!("{}:{}", m.file, m.span.0);
        let source = &index.sources[&m.file];
        let slice = source
            .lines()
            .enumerate()
            .skip(m.span.0 - 1)
            .take(m.span.1 - m.span.0 + 1)
            .filter_map(|(n, s)| {
                let s = s.trim();
                (!s.is_empty()
                    && !s.starts_with("//")
                    && !s.starts_with("/*")
                    && !s.starts_with('*'))
                .then(|| (n + 1, s.to_owned()))
            })
            .collect::<Vec<_>>();
        if slice.len() > budget {
            omitted.insert(id);
            continue;
        }
        budget -= slice.len();
        lines.entry(m.file.clone()).or_default().extend(slice);
        picked.push(json!({"id":id,"owner":m.owners[0],"method":m.name,"depth":depth}));
        for f in &index.fields {
            if m.owners.contains(&f.owner) && m.identifiers.contains(&f.name) {
                fields.insert((f.owner.clone(), f.name.clone()));
            }
        }
        for (r, n) in &m.calls {
            let targets = index.targets(m, r, n);
            let edge = format!("{id} -> {r}::{n}");
            if targets.is_empty() {
                unresolved.insert(edge);
                continue;
            }
            if targets.len() > 1 {
                ambiguous.insert(edge);
            }
            for j in targets {
                if depth < DEPTH {
                    queue.push_back((j, depth + 1));
                } else if !seen.contains(&j) {
                    omitted.insert(format!(
                        "{}:{}",
                        index.methods[j].file, index.methods[j].span.0
                    ));
                }
            }
        }
    }
    for (owner, name, file, line) in index
        .fields
        .iter()
        .map(|f| (&f.owner, &f.name, &f.file, &f.line))
        .chain(index.init.iter().map(|(o, n, f, l)| (o, n, f, l)))
    {
        if fields.contains(&(owner.clone(), name.clone())) {
            let text = index.sources[file]
                .lines()
                .nth(line - 1)
                .ok_or("invalid field location")?
                .trim();
            lines
                .entry(file.clone())
                .or_default()
                .insert(*line, text.into());
        }
    }
    let initializers = defaults::extend(&index, &mut lines)?;
    let mut evidence = String::new();
    for (file, body) in &lines {
        evidence.push_str(&format!("\n{file}\n"));
        for (line, text) in body {
            evidence.push_str(&format!("{line} {text}\n"));
        }
    }
    let dest = Path::new(out);
    fs::create_dir(dest)?;
    let hashes: BTreeMap<_, _> = index
        .sources
        .iter()
        .map(|(f, s)| (f, sha256_hex(s.as_bytes())))
        .collect();
    save(
        &dest.join("selection.json"),
        &json!({"case":case,"hashes":hashes,"selected":picked,"unresolved":unresolved,"ambiguous":ambiguous,"omitted_at_boundary":omitted,"initializers":initializers,"lines":lines}),
    )?;
    save(
        &dest.join("request.json"),
        &json!({"protocol":1,"purpose":"iterator protocol forecast","system":"Use the supplied production source as evidence. Return JSON only.","user":format!("{QUESTION}\n{evidence}"),"max_output_tokens":8000}),
    )
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
