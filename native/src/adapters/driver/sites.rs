//! Site rules: where a pack says behaviour is decided, and the eligible
//! sites of a file. Nothing here is specific to a language.

use super::pack::{has, Pack, Rule};
use super::source::SourceSet;
use super::tree::{ancestors, chain, grammar, named_children, parse, position, walk, SgNode};
use super::Kind;
use serde::Serialize;

/// The kinds that have operators, in the order sites are discovered.
pub const OPERATOR_KINDS: [Kind; 4] = [
    Kind::BranchCondition,
    Kind::FailureState,
    Kind::StateWrite,
    Kind::ErrorOrder,
];

/// Why a node is or is not a site of a kind.
pub const OK: &str = "ok";
pub const HIDDEN: &str = "in_comment_or_string";
pub const TEST_REGION: &str = "test_region";
pub const KIND_MISMATCH: &str = "kind_mismatch";

/// One place a mutant can be planted. Lines are 1-based, columns 0-based
/// characters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Site {
    pub file: String,
    pub kind: Kind,
    pub node_kind: String,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub text: String,
}

impl Site {
    pub fn span(&self) -> (usize, usize, usize, usize) {
        (
            self.start_line,
            self.start_column,
            self.end_line,
            self.end_column,
        )
    }

    /// File, start position, kind name, end position.
    pub fn order(&self) -> (&str, usize, usize, &str, usize, usize) {
        (
            &self.file,
            self.start_line,
            self.start_column,
            self.kind.name(),
            self.end_line,
            self.end_column,
        )
    }
}

fn only_named_child(parent: &SgNode<'_>, node: &SgNode<'_>) -> bool {
    let named: Vec<_> = parent.children().filter(|c| c.is_named()).collect();
    named.len() == 1 && named[0].kind() == node.kind() && named[0].range() == node.range()
}

/// A returned expression of a listed kind with one of its own operator
/// tokens listed.
fn returned(node: &SgNode<'_>, parent: &SgNode<'_>, rule: &Rule) -> bool {
    let Some(r) = &rule.returned else {
        return false;
    };
    has(&r.of, &parent.kind())
        && has(&r.kinds, &node.kind())
        && node
            .children()
            .any(|c| !c.is_named() && has(&r.tokens, &c.text()))
}

/// Whether the rule allows this node, taken together with its parent.
fn allowed(node: &SgNode<'_>, rule: &Rule, wrappers: &[String]) -> bool {
    let kind = node.kind();
    if has(&rule.nodes, &kind) {
        return true;
    }
    if let Some(prefixes) = rule.nodes_with_prefix.get(kind.as_ref()) {
        let text = node.text();
        if prefixes.iter().any(|p| text.starts_with(p.as_str())) {
            return true;
        }
    }
    let Some(parent) = node.parent() else {
        return false;
    };
    let is_field = |(parent_kind, field): &(String, String)| {
        parent.kind() == parent_kind.as_str()
            && parent
                .field(field)
                .is_some_and(|target| target.range() == node.range())
    };
    if rule.field_of.iter().any(is_field) {
        return true;
    }
    // A call that is a statement of its own; a returned yes-or-no expression.
    only_named_child(&parent, node)
        && ((has(&rule.call_statements, &kind) && has(wrappers, &parent.kind()))
            || returned(node, &parent, rule))
}

/// The first node of the chain (or its unwrap wrapper) the rule allows.
fn kind_match<'r>(chain: &[SgNode<'r>], rule: &Rule, pack: &Pack) -> Option<SgNode<'r>> {
    let wrapper = chain[0]
        .parent()
        .filter(|w| has(&pack.unwrap_kinds, &w.kind()));
    chain
        .iter()
        .chain(wrapper.iter())
        .find(|node| allowed(node, rule, &pack.statement_wrappers))
        .cloned()
}

/// The following named sibling, if it is of a kind the rule names.
pub fn next_check<'r>(first: &SgNode<'r>, rule: &Rule, skip: &[String]) -> Option<SgNode<'r>> {
    let mut sibling = first.next();
    while let Some(s) = &sibling {
        if s.is_named() && !has(skip, &s.kind()) {
            break;
        }
        sibling = s.next();
    }
    let sibling = sibling?;
    let matches = chain(&sibling).iter().any(|n| has(&rule.nodes, &n.kind()));
    matches.then_some(sibling)
}

/// Whether the node or one of its ancestors is preceded by a test marker.
fn in_test_region(node: &SgNode<'_>, pack: &Pack) -> bool {
    let markers = &pack.test_attribute_markers;
    if markers.is_empty() {
        return false;
    }
    let mut candidate = Some(node.clone());
    while let Some(c) = candidate {
        let mut previous = c.prev();
        while let Some(p) = previous {
            if !has(&pack.test_attribute_kinds, &p.kind()) {
                break;
            }
            let text = p.text();
            if markers.iter().any(|m| text.contains(m.as_str())) {
                return true;
            }
            previous = p.prev();
        }
        candidate = c.parent();
    }
    false
}

/// The reason a node is or is not a site of `kind`, and the allowed node.
pub fn site_check<'r>(
    pack: &Pack,
    node: &SgNode<'r>,
    kind: Kind,
) -> (&'static str, Option<SgNode<'r>>) {
    let mut node = node.clone();
    // A node that is a whole statement names what the statement wraps.
    if has(&pack.statement_wrappers, &node.kind()) {
        let inner = named_children(&node, &pack.comment_kinds);
        if inner.len() == 1 {
            node = inner[0].clone();
        }
    }
    let chain = chain(&node);
    let hidden = |n: &SgNode<'_>| pack.is_hidden(&n.kind());
    if chain.iter().any(hidden) || ancestors(&chain[0]).iter().any(hidden) {
        return (HIDDEN, None);
    }
    if in_test_region(&chain[0], pack) {
        return (TEST_REGION, None);
    }
    let Some(rule) = pack.rule(kind) else {
        return (KIND_MISMATCH, None);
    };
    let Some(matched) = kind_match(&chain, rule, pack) else {
        return (KIND_MISMATCH, None);
    };
    if rule.requires_next_sibling && next_check(&chain[0], rule, &pack.comment_kinds).is_none() {
        return (KIND_MISMATCH, None);
    }
    (OK, Some(matched))
}

/// The node an operator acts on; None for a condition the pack excludes.
fn normalise<'r>(pack: &Pack, kind: Kind, node: SgNode<'r>) -> Option<SgNode<'r>> {
    if kind != Kind::BranchCondition {
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

/// The site at a node, if the node is a site of `kind`.
pub fn site_at(pack: &Pack, file: &str, kind: Kind, node: &SgNode<'_>) -> Option<Site> {
    let (reason, matched) = site_check(pack, node, kind);
    let target = normalise(pack, kind, matched.filter(|_| reason == OK)?)?;
    let (start_line, start_column, end_line, end_column) = position(&target);
    Some(Site {
        file: file.to_owned(),
        kind,
        node_kind: target.kind().into_owned(),
        start_line,
        start_column,
        end_line,
        end_column,
        text: target.text().into_owned(),
    })
}

/// Eligible sites of one file text, without duplicates, in order.
pub fn file_sites(pack: &Pack, file: &str, text: &str) -> Result<Vec<Site>, String> {
    let root = parse(text, grammar(&pack.language)?)?;
    let mut found: Vec<Site> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for node in walk(&root.root()) {
        for kind in OPERATOR_KINDS {
            if let Some(site) = site_at(pack, file, kind, &node) {
                if seen.insert((site.span(), kind)) {
                    found.push(site);
                }
            }
        }
    }
    found.sort_by(|a, b| a.order().cmp(&b.order()));
    Ok(found)
}

/// Every site the pack makes eligible in the files that are not tests,
/// ordered by file, start position and kind.
pub fn eligible_sites(source: &SourceSet, pack: &Pack) -> Result<Vec<Site>, String> {
    let mut sites = Vec::new();
    for file in source.files.iter().filter(|f| !source.is_test_file(f)) {
        sites.extend(file_sites(pack, file, &source.raw(file)?)?);
    }
    sites.sort_by(|a, b| a.order().cmp(&b.order()));
    Ok(sites)
}
