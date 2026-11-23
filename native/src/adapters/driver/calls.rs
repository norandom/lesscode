//! Calls by name: how often a file calls the functions it declares, and
//! which functions of a set of files call which.
//!
//! The pack says which nodes are calls and which field leads from a node to
//! the name the call is made under. Matching is by name alone, so a count or
//! an edge says that such a call is written there, not that it runs, and a
//! name declared in several files gives an edge to each of them.

use super::functions::{functions_of, Function};
use super::pack::{has, Pack};
use super::tree::{named_children, parse_clean, position, walk, SgNode};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

/// The name a call is made under and where the call starts.
struct Call {
    name: String,
    at: (usize, usize),
}

/// Follow the pack's fields from a call to its name. A call of what another
/// call returns has no name.
fn name_of(pack: &Pack, call: &SgNode<'_>) -> Option<String> {
    let mut node = call.clone();
    while let Some(field) = pack.calls.via.get(node.kind().as_ref()) {
        node = node.field(field)?;
        while has(&pack.unwrap_kinds, &node.kind()) {
            node = named_children(&node, &pack.comment_kinds)
                .into_iter()
                .next()?;
        }
        if has(&pack.calls.nodes, &node.kind()) {
            return None;
        }
    }
    Some(node.text().into_owned())
}

/// The named functions of one file and the calls written in it.
pub struct Named {
    functions: Vec<Function>,
    calls: Vec<Call>,
}

/// `None` when the pack does not say what a call is.
pub fn named(pack: &Pack, label: &str, text: &str) -> Result<Option<Named>, String> {
    if pack.calls.nodes.is_empty() {
        return Ok(None);
    }
    let root = parse_clean(label, text, &pack.language)?;
    let call = |node: SgNode<'_>| {
        let (line, column, _, _) = position(&node);
        Some(Call {
            name: name_of(pack, &node)?,
            at: (line, column),
        })
    };
    let calls = walk(&root.root())
        .into_iter()
        .filter(|node| has(&pack.calls.nodes, &node.kind()))
        .filter_map(call)
        .collect();
    let mut functions = functions_of(pack, label, text)?;
    functions.retain(|f| !f.anonymous);
    Ok(Some(Named { functions, calls }))
}

impl Named {
    fn names(&self) -> BTreeSet<&str> {
        self.functions.iter().map(|f| f.name.as_str()).collect()
    }

    /// Calls made under the name of a function this file declares. A call
    /// that is written twice counts twice.
    pub fn occurrences(&self) -> usize {
        let names = self.names();
        let declared = |call: &&Call| names.contains(call.name.as_str());
        self.calls.iter().filter(declared).count()
    }
}

#[derive(Serialize)]
pub struct FileCalls {
    /// Distinct names of the functions the file declares.
    pub functions: usize,
    pub call_occurrences: usize,
}

#[derive(Serialize)]
pub struct ScopeTotal {
    pub functions: usize,
    pub call_occurrences: usize,
    pub edges: usize,
    /// Distinct pairs of different files with an edge between them.
    pub module_hops: usize,
}

#[derive(Serialize)]
pub struct ScopeCalls {
    pub receipt: &'static str,
    pub lang: String,
    pub pack_sha256: String,
    pub root: String,
    pub files: BTreeMap<String, FileCalls>,
    pub total: ScopeTotal,
    /// Caller file, caller, callee file, callee. A function calling its own
    /// name is not an edge.
    pub edges: BTreeSet<[String; 4]>,
}

fn read(root: &Path, file: &str) -> Result<String, String> {
    let inside = Path::new(file)
        .components()
        .all(|part| matches!(part, Component::Normal(_) | Component::CurDir));
    if !inside {
        return Err(format!("{file} is not a path below the root"));
    }
    let path = root.join(file);
    std::fs::read_to_string(&path).map_err(|e| format!("Cannot read {}: {e}", path.display()))
}

/// The edges from one function of `file`: a call written in it, under
/// another name than its own, to each file that declares that name.
fn edges_from<'a>(
    file: &'a str,
    caller: &'a Function,
    calls: &'a [Call],
    declared: &'a BTreeMap<&str, Vec<&str>>,
) -> impl Iterator<Item = [String; 4]> + 'a {
    let inside = move |call: &&Call| caller.covers(call.at) && call.name != caller.name;
    calls.iter().filter(inside).flat_map(move |call| {
        let targets = declared.get(call.name.as_str()).into_iter().flatten();
        targets.map(move |target| [file, &caller.name, *target, &call.name].map(str::to_owned))
    })
}

/// Count the calls among `files`, which are named relative to `root`.
pub fn scope_calls(pack: &Pack, root: &Path, files: &[String]) -> Result<ScopeCalls, String> {
    let mut texts = Vec::new();
    for file in files {
        texts.push((file.clone(), read(root, file)?));
    }
    calls_among(pack, &root.to_string_lossy(), &texts)
}

/// Count the calls among files given as name and text; `root` says where
/// they are from.
pub fn calls_among(
    pack: &Pack,
    root: &str,
    texts: &[(String, String)],
) -> Result<ScopeCalls, String> {
    let mut facts = BTreeMap::new();
    for (file, text) in texts {
        let named = named(pack, file, text)?
            .ok_or_else(|| format!("the {} pack does not say what a call is", pack.language))?;
        facts.insert(file.replace('\\', "/"), named);
    }
    let mut declared: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (file, named) in &facts {
        for name in named.names() {
            declared.entry(name).or_default().push(file);
        }
    }
    let mut edges = BTreeSet::new();
    for (file, named) in &facts {
        for caller in &named.functions {
            edges.extend(edges_from(file, caller, &named.calls, &declared));
        }
    }
    let files: BTreeMap<String, FileCalls> = facts
        .iter()
        .map(|(file, named)| {
            let counts = FileCalls {
                functions: named.names().len(),
                call_occurrences: named.occurrences(),
            };
            (file.clone(), counts)
        })
        .collect();
    let apart = edges.iter().filter(|e| e[0] != e[2]);
    let hops: BTreeSet<_> = apart.map(|e| (&e[0], &e[2])).collect();
    let total = ScopeTotal {
        functions: files.values().map(|f| f.functions).sum(),
        call_occurrences: files.values().map(|f| f.call_occurrences).sum(),
        edges: edges.len(),
        module_hops: hops.len(),
    };
    Ok(ScopeCalls {
        receipt: "scope-calls-v1",
        lang: pack.language.clone(),
        pack_sha256: pack.digest.clone(),
        root: root.to_owned(),
        files,
        total,
        edges,
    })
}
