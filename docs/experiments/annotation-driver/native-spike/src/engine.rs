//! Port of `engine.py` without selection: eligible sites and every mutant of
//! a site. All language knowledge comes from the pack.

use crate::pack::{has, partners, truthy, Pack, Rule};
use crate::scope::Scope;
use crate::tree::{chain, grammar, named_children, parse, position, walk, SgNode};
use crate::validator::{next_check, site_check};
use serde::Serialize;

pub const OPERATOR_KINDS: [&str; 4] = [
    "branch_condition",
    "failure_state",
    "state_write",
    "error_order",
];

/// One mutable location. Lines are 1-based, columns 0-based characters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Site {
    pub file: String,
    pub kind: String,
    pub node_kind: String,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub text: String,
}

impl Site {
    fn span(&self) -> (usize, usize, usize, usize) {
        (
            self.start_line,
            self.start_column,
            self.end_line,
            self.end_column,
        )
    }

    fn order(&self) -> (&str, usize, usize, &str, usize, usize) {
        let s = self;
        (
            &s.file,
            s.start_line,
            s.start_column,
            &s.kind,
            s.end_line,
            s.end_column,
        )
    }
}

/// A mutated file: its full text and the replaced byte ranges of the original.
#[derive(Debug, Clone)]
pub struct Mutant {
    pub operator: String,
    pub text: String,
    pub spans: Vec<(usize, usize)>,
}

type Edits<'r> = Vec<(SgNode<'r>, String)>;

/// The node an operator acts on; None for a condition the pack excludes.
fn normalise<'r>(pack: &Pack, kind: &str, node: SgNode<'r>) -> Option<SgNode<'r>> {
    if kind != "branch_condition" {
        return Some(node);
    }
    let mut node = node;
    if has(&pack.unwrap_kinds, &node.kind()) {
        let inner = named_children(&node, &pack.comment_kinds);
        if inner.len() == 1 {
            node = inner[0].clone();
        }
    }
    let excluded = &pack.operators.branch_condition.exclude_kinds;
    if chain(&node).iter().any(|n| has(excluded, &n.kind())) {
        return None;
    }
    Some(node)
}

fn site_at(pack: &Pack, file: &str, kind: &str, node: &SgNode<'_>) -> Option<Site> {
    let (reason, matched) = site_check(pack, node, kind);
    let target = normalise(pack, kind, matched.filter(|_| reason == "ok")?)?;
    let (start_line, start_column, end_line, end_column) = position(&target);
    Some(Site {
        file: file.to_owned(),
        kind: kind.to_owned(),
        node_kind: target.kind().into_owned(),
        start_line,
        start_column,
        end_line,
        end_column,
        text: target.text().into_owned(),
    })
}

/// Eligible sites of one file text, deduplicated by (span, kind), in order.
pub fn file_sites(
    pack: &Pack,
    language: &str,
    file: &str,
    text: &str,
) -> Result<Vec<Site>, String> {
    let root = parse(text, grammar(language)?)?;
    let mut found: Vec<Site> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for node in walk(&root.root()) {
        for kind in OPERATOR_KINDS {
            if let Some(site) = site_at(pack, file, kind, &node) {
                if seen.insert((site.span(), site.kind.clone())) {
                    found.push(site);
                }
            }
        }
    }
    found.sort_by(|a, b| a.order().cmp(&b.order()));
    Ok(found)
}

/// Every site the pack makes eligible, ordered by file, start position and kind.
pub fn eligible_sites(scope: &Scope, pack: &Pack) -> Result<Vec<Site>, String> {
    let mut sites = Vec::new();
    for file in scope.files.iter().filter(|f| !scope.is_test_file(f)) {
        sites.extend(file_sites(pack, &scope.language, file, &scope.raw(file)?)?);
    }
    sites.sort_by(|a, b| a.order().cmp(&b.order()));
    Ok(sites)
}

/// `engine._fill`: `{name}` placeholders (`\w+`) replaced left to right.
fn fill(template: &str, pack: &Pack, node: &SgNode<'_>) -> Result<String, String> {
    let mut out = String::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let name_len = after
            .find(|c: char| !(c.is_alphanumeric() || c == '_'))
            .unwrap_or(after.len());
        if name_len == 0 || !after[name_len..].starts_with('}') {
            out.push('{');
            rest = after;
            continue;
        }
        let value = match &after[..name_len] {
            "text" => node.text().into_owned(),
            "noop" => pack
                .noop_statement
                .clone()
                .ok_or("the pack has no noop_statement")?,
            "child0" => match named_children(node, &pack.comment_kinds).first() {
                Some(child) => child.text().into_owned(),
                None => {
                    return Err(format!(
                        "{{child0}} on a {} without named children",
                        node.kind()
                    ))
                }
            },
            other => return Err(format!("unknown template placeholder {{{other}}}")),
        };
        out.push_str(&value);
        rest = &after[name_len + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// The pack's own operator for the site: (label, edits), or None.
fn first_edit<'r>(
    pack: &Pack,
    kind: &str,
    node: &SgNode<'r>,
) -> Result<Option<(String, Edits<'r>)>, String> {
    let ops = &pack.operators;
    let node_kind = node.kind();
    if kind == "branch_condition" {
        if let Some(template) = &ops.branch_condition.template {
            return Ok(Some((
                "negate".into(),
                vec![(node.clone(), fill(template, pack, node)?)],
            )));
        }
    }
    if kind == "failure_state" {
        if let Some(template) = ops.failure_state.by_kind.get(node_kind.as_ref()) {
            let label = if template == "{noop}" {
                "drop"
            } else {
                "rewrite"
            };
            let edit = (node.clone(), fill(template, pack, node)?);
            return Ok(Some((format!("{label}:{node_kind}"), vec![edit])));
        }
    }
    if kind == "state_write" {
        let parent = node
            .parent()
            .filter(|p| has(&pack.statement_wrappers, &p.kind()));
        if let (Some(statement), Some(parent)) = (&ops.state_write.statement, parent) {
            let replacement = fill(statement, pack, &parent)?;
            return Ok(Some(("drop_statement".into(), vec![(parent, replacement)])));
        }
    }
    if kind == "error_order" && truthy(&ops.error_order.swap_next) {
        let empty = Rule::default();
        let rule = pack.kinds.get(kind).unwrap_or(&empty);
        let first = chain(node).swap_remove(0);
        if let Some(sibling) = next_check(&first, rule, &pack.comment_kinds) {
            let (a, b) = (first.text().into_owned(), sibling.text().into_owned());
            return Ok(Some(("swap_next".into(), vec![(first, b), (sibling, a)])));
        }
    }
    Ok(None)
}

/// The first two operator tokens of a condition that have a partner, in source order.
fn token_swaps<'r>(pack: &Pack, node: &SgNode<'r>) -> Vec<(String, Edits<'r>)> {
    let partner = partners(&pack.operators.branch_condition.token_swaps);
    if partner.is_empty() {
        return Vec::new();
    }
    let mut found: Vec<SgNode<'r>> = Vec::new();
    let mut stack = vec![node.clone()];
    while let Some(parent) = stack.pop() {
        let mut named = Vec::new();
        for child in parent.children() {
            if child.is_named() {
                if !pack.is_hidden(&child.kind()) {
                    named.push(child);
                }
            } else if partner.contains_key(child.text().as_ref()) {
                found.push(child);
            }
        }
        stack.extend(named.into_iter().rev());
    }
    found.sort_by_key(|t| t.range().start);
    found.truncate(2);
    found
        .into_iter()
        .map(|token| {
            let old = token.text().into_owned();
            let new = partner[&old].clone();
            let (line, column, _, _) = position(&token);
            (
                format!("swap:{old}->{new}@{line}:{column}"),
                vec![(token, new)],
            )
        })
        .collect()
}

/// A failure site without a by_kind template whose one named child is a listed literal.
fn literal_swap<'r>(pack: &Pack, node: &SgNode<'r>) -> Vec<(String, Edits<'r>)> {
    let ops = &pack.operators.failure_state;
    let named: Vec<_> = node.children().filter(|c| c.is_named()).collect();
    if ops.by_kind.contains_key(node.kind().as_ref()) || named.len() != 1 {
        return Vec::new();
    }
    let old = named[0].text().into_owned();
    match partners(&ops.returned_literal_swaps).get(&old) {
        Some(new) => vec![(
            format!("literal:{old}->{new}"),
            vec![(named[0].clone(), new.clone())],
        )],
        None => Vec::new(),
    }
}

fn all_edits<'r>(
    pack: &Pack,
    kind: &str,
    node: &SgNode<'r>,
) -> Result<Vec<(String, Edits<'r>)>, String> {
    let mut found: Vec<_> = first_edit(pack, kind, node)?.into_iter().collect();
    if kind == "branch_condition" {
        found.extend(token_swaps(pack, node));
    }
    if kind == "failure_state" {
        found.extend(literal_swap(pack, node));
    }
    Ok(found)
}

/// Replace each node by exact offsets, last first; never by searching text.
fn apply(text: &str, mut edits: Edits<'_>) -> Result<(String, Vec<(usize, usize)>), String> {
    edits.sort_by_key(|e| std::cmp::Reverse(e.0.range().start));
    let mut out = text.to_owned();
    let mut spans = Vec::new();
    for (node, new) in edits {
        let r = node.range();
        if out.get(r.clone()) != Some(node.text().as_ref()) {
            return Err(format!(
                "offsets {}:{} do not address the node text",
                r.start, r.end
            ));
        }
        out.replace_range(r.clone(), &new);
        spans.push((r.start, r.end));
    }
    spans.sort();
    Ok((out, spans))
}

/// Every mutant the pack gives for the site, in a stable order.
pub fn mutants(scope: &Scope, pack: &Pack, site: &Site) -> Result<Vec<Mutant>, String> {
    let text = scope.raw(&site.file)?;
    let root = parse(&text, grammar(&scope.language)?)?;
    let node = walk(&root.root())
        .into_iter()
        .find(|n| n.kind() == site.node_kind.as_str() && position(n) == site.span())
        .ok_or_else(|| format!("site no longer present in {}: {site:?}", site.file))?;
    all_edits(pack, &site.kind, &node)?
        .into_iter()
        .map(|(operator, edits)| {
            let (text, spans) = apply(&text, edits)?;
            Ok(Mutant {
                operator,
                text,
                spans,
            })
        })
        .collect()
}
