//! The functions of a source text, as the pack's boundary kind finds them,
//! and the sites that lie in named ones.
//!
//! A function is named by the `name` field of its syntax node, or of the
//! node it is bound to; one without either is called `line N` after the
//! line it starts on. Two functions of one file may share a name (methods
//! of different types); naming it then means both.

use super::pack::Pack;
use super::sites::{sites_of, Site};
use super::source::SourceSet;
use super::tree::{grammar, parse, position, walk};
use super::Kind;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

/// Lines are 1-based, columns 0-based characters, as for a site.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Function {
    pub name: String,
    /// The syntax gives it no name; it is called after its line.
    pub anonymous: bool,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

impl Function {
    /// Whether the site starts inside this function.
    pub fn contains(&self, site: &Site) -> bool {
        let at = (site.start_line, site.start_column);
        (self.start_line, self.start_column) <= at && at < (self.end_line, self.end_column)
    }
}

/// The functions of `text`, in source order. Nested functions are listed
/// as well as the function around them.
pub fn functions_of(pack: &Pack, label: &str, text: &str) -> Result<Vec<Function>, String> {
    let boundaries = sites_of(pack, label, text, &[Kind::PureShellBoundary])?;
    let root = parse(text, grammar(&pack.language)?)?;
    let wanted: BTreeSet<_> = boundaries
        .iter()
        .map(|b| (b.span(), b.node_kind.clone()))
        .collect();
    let mut names = BTreeMap::new();
    for node in walk(&root.root()) {
        let key = (position(&node), node.kind().into_owned());
        if !wanted.contains(&key) {
            continue;
        }
        // Its own name, or the name of what it is bound to.
        let name = node
            .field("name")
            .or_else(|| node.parent().and_then(|p| p.field("name")));
        if let Some(name) = name {
            names.entry(key).or_insert_with(|| name.text().into_owned());
        }
    }
    let function = |b: &Site| {
        let name = names.get(&(b.span(), b.node_kind.clone()));
        Function {
            name: name
                .cloned()
                .unwrap_or_else(|| format!("line {}", b.start_line)),
            anonymous: name.is_none(),
            start_line: b.start_line,
            start_column: b.start_column,
            end_line: b.end_line,
            end_column: b.end_column,
        }
    };
    Ok(boundaries.iter().map(function).collect())
}

/// Keep the sites that lie in the functions named for their file. A file
/// without an entry keeps all its sites; an empty list keeps none. Returns
/// the sites kept and how many were left out, or `None` when nothing is
/// named at all. A name that no function of the file has is an error.
pub fn select(
    sites: Vec<Site>,
    wanted: &BTreeMap<String, Vec<String>>,
    source: &SourceSet,
    pack: &Pack,
) -> Result<(Vec<Site>, Option<usize>), String> {
    if wanted.is_empty() {
        return Ok((sites, None));
    }
    let mut chosen: BTreeMap<&str, Vec<Function>> = BTreeMap::new();
    for (file, names) in wanted {
        if !source.files.contains(file) {
            return Err(format!("functions: {file} is not one of the files"));
        }
        let all = functions_of(pack, file, &source.raw(file)?)?;
        if let Some(missing) = names.iter().find(|n| !all.iter().any(|f| f.name == **n)) {
            let known: Vec<&str> = all.iter().map(|f| f.name.as_str()).collect();
            return Err(format!(
                "functions: {file} has no function {missing}; it has {}",
                known.join(", ")
            ));
        }
        let named = all.into_iter().filter(|f| names.contains(&f.name));
        chosen.insert(file, named.collect());
    }
    let total = sites.len();
    let inside = |site: &Site| match chosen.get(site.file.as_str()) {
        Some(functions) => functions.iter().any(|f| f.contains(site)),
        None => true,
    };
    let kept: Vec<Site> = sites.into_iter().filter(inside).collect();
    let left_out = total - kept.len();
    Ok((kept, Some(left_out)))
}
