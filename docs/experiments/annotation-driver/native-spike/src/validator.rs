//! Port of `validator.py`: site rules, annotation validation and mutant
//! classification. Everything language-specific comes from the pack.

use crate::pack::{has, truthy, Pack, Rule};
use crate::scope::{py_int, py_str, py_strip, universal_newlines, Scope};
use crate::tree::{ancestors, chain, grammar, parse, walk, SgNode, SgRoot};
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};

pub const KINDS: [&str; 5] = [
    "failure_state",
    "branch_condition",
    "error_order",
    "state_write",
    "pure_shell_boundary",
];

fn only_named_child(parent: &SgNode<'_>, node: &SgNode<'_>) -> bool {
    let named: Vec<_> = parent.children().filter(|c| c.is_named()).collect();
    named.len() == 1 && named[0].kind() == node.kind() && named[0].range() == node.range()
}

/// Pack format version 1: a returned expression of a listed kind with one of
/// its own operator tokens listed.
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

/// The first node of the chain (or its parenthesis wrapper) the rule allows.
fn kind_match<'r>(
    chain: &[SgNode<'r>],
    rule: &Rule,
    unwrap: &[String],
    wrappers: &[String],
) -> Option<SgNode<'r>> {
    let mut candidates = chain.to_vec();
    if let Some(wrapper) = chain[0].parent() {
        if has(unwrap, &wrapper.kind()) {
            candidates.push(wrapper);
        }
    }
    for node in candidates {
        let kind = node.kind();
        if has(&rule.nodes, &kind) {
            return Some(node.clone());
        }
        if let Some(prefixes) = rule.nodes_with_prefix.get(kind.as_ref()) {
            let text = node.text();
            if prefixes.iter().any(|p| text.starts_with(p.as_str())) {
                return Some(node.clone());
            }
        }
        let Some(parent) = node.parent() else {
            continue;
        };
        for (parent_kind, field) in &rule.field_of {
            if parent.kind() == parent_kind.as_str() {
                if let Some(target) = parent.field(field) {
                    if target.range() == node.range() {
                        return Some(node.clone());
                    }
                }
            }
        }
        if only_named_child(&parent, &node) {
            // A call that is a statement of its own; a returned yes-or-no expression.
            if has(&rule.call_statements, &kind) && has(wrappers, &parent.kind()) {
                return Some(node.clone());
            }
            if returned(&node, &parent, rule) {
                return Some(node.clone());
            }
        }
    }
    None
}

/// The following named sibling, if it is of a kind the rule names.
pub fn next_check<'r>(chain0: &SgNode<'r>, rule: &Rule, skip: &[String]) -> Option<SgNode<'r>> {
    let mut sibling = chain0.next();
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

fn in_test_region(node: &SgNode<'_>, markers: &[String]) -> bool {
    if markers.is_empty() {
        return false;
    }
    let mut candidate = Some(node.clone());
    while let Some(c) = candidate {
        let mut previous = c.prev();
        while let Some(p) = previous {
            if !p.kind().contains("attribute") {
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

/// Named nodes on lines `start`..`end` whose text (universal newlines) is `anchor`.
fn anchor_nodes<'r>(root: &SgNode<'r>, start: i64, end: i64, anchor: &str) -> Vec<SgNode<'r>> {
    walk(root)
        .into_iter()
        .filter(|n| {
            n.is_named()
                && n.start_pos().line() as i64 == start - 1
                && n.end_pos().line() as i64 == end - 1
                && universal_newlines(&n.text()) == anchor
        })
        .collect()
}

/// Rule 2: the matching nodes, and whether the statement terminator was appended.
pub fn match_anchor<'r>(
    pack: &Pack,
    root: &SgNode<'r>,
    start: i64,
    end: i64,
    anchor: &str,
) -> (Vec<SgNode<'r>>, bool) {
    let nodes = anchor_nodes(root, start, end, anchor);
    let terminator = pack.terminator();
    if !nodes.is_empty() || terminator.is_empty() {
        return (nodes, false);
    }
    let completed = anchor_nodes(root, start, end, &format!("{anchor}{terminator}"));
    let found = !completed.is_empty();
    (completed, found)
}

/// Rules 3 and 4 and the test-region rule for one node: (reason, allowed node).
pub fn site_check<'r>(
    pack: &Pack,
    node: &SgNode<'r>,
    kind: &str,
) -> (&'static str, Option<SgNode<'r>>) {
    let mut node = node.clone();
    // An anchor that includes the statement terminator names the whole statement;
    // what the statement wraps is the site (gate Amendment 12).
    if has(&pack.statement_wrappers, &node.kind()) {
        let inner = crate::tree::named_children(&node, &pack.comment_kinds);
        if inner.len() == 1 {
            node = inner[0].clone();
        }
    }
    let chain = chain(&node);
    let hidden = |n: &SgNode<'_>| pack.is_hidden(&n.kind());
    if chain.iter().any(hidden) || ancestors(&chain[0]).iter().any(hidden) {
        return ("in_comment_or_string", None);
    }
    if in_test_region(&chain[0], &pack.test_attribute_markers) {
        return ("test_region", None);
    }
    let empty = Rule::default();
    let rule = pack.kinds.get(kind).unwrap_or(&empty);
    let matched = kind_match(&chain, rule, &pack.unwrap_kinds, &pack.statement_wrappers);
    let Some(matched) = matched else {
        return ("kind_mismatch", None);
    };
    if truthy(&rule.requires_next_sibling)
        && next_check(&chain[0], rule, &pack.comment_kinds).is_none()
    {
        return ("kind_mismatch", None);
    }
    ("ok", Some(matched))
}

#[derive(Debug, Clone, Serialize)]
pub struct Decision {
    pub accepted: bool,
    pub reason: String,
    pub anchor_completed: bool,
}

struct Validation<'a> {
    scope: &'a Scope,
    pack: &'a Pack,
    roots: HashMap<String, SgRoot>,
    accepted: BTreeSet<(String, (usize, usize), String)>,
}

impl Validation<'_> {
    fn decide(&mut self, annotation: &Value) -> Result<(&'static str, bool), String> {
        let a = annotation
            .as_object()
            .ok_or("an annotation is not a JSON object")?;
        let kind = a
            .get("kind")
            .and_then(Value::as_str)
            .filter(|k| KINDS.contains(k));
        let anchor = a
            .get("anchor")
            .and_then(Value::as_str)
            .filter(|s| !py_strip(s).is_empty());
        let (start, end) = (py_int(a.get("start_line")), py_int(a.get("end_line")));
        let (Some(kind), Some(anchor), Some(start), Some(end)) = (kind, anchor, start, end) else {
            return Ok(("malformed", false));
        };
        if start < 1 || end < start {
            return Ok(("malformed", false));
        }
        let normalized = py_str(a.get("file")).replace('\\', "/");
        if self.scope.is_test_file(&normalized) {
            return Ok(("test_file", false));
        }
        if !self.scope.files.contains(&normalized) {
            return Ok(("outside_scope", false));
        }
        if !self.roots.contains_key(&normalized) {
            let lang = grammar(&self.scope.language)?;
            let root = parse(&self.scope.source(&normalized)?, lang)?;
            self.roots.insert(normalized.clone(), root);
        }
        let root = self.roots[&normalized].root();
        let (matches, completed) = match_anchor(self.pack, &root, start, end, anchor);
        let spans: BTreeSet<(usize, usize)> = matches
            .iter()
            .map(|n| (n.range().start, n.range().end))
            .collect();
        if spans.is_empty() {
            return Ok(("anchor_mismatch", completed));
        }
        if spans.len() > 1 {
            return Ok(("anchor_ambiguous", completed));
        }
        let (reason, _) = site_check(self.pack, &matches[0], kind);
        if reason != "ok" {
            return Ok((reason, completed));
        }
        let span = spans.into_iter().next().unwrap_or_default();
        if !self.accepted.insert((normalized, span, kind.to_owned())) {
            return Ok(("duplicate", completed));
        }
        Ok(("ok", completed))
    }
}

/// One decision per annotation (contract version 0, pack format version 1).
pub fn validate(
    scope: &Scope,
    pack: &Pack,
    annotations: &[Value],
) -> Result<Vec<Decision>, String> {
    let mut v = Validation {
        scope,
        pack,
        roots: HashMap::new(),
        accepted: BTreeSet::new(),
    };
    annotations
        .iter()
        .map(|a| {
            let (reason, completed) = v.decide(a)?;
            Ok(Decision {
                accepted: reason == "ok",
                reason: reason.to_owned(),
                anchor_completed: completed,
            })
        })
        .collect()
}

fn tokens(root: &SgNode<'_>, skip: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![root.clone()];
    while let Some(node) = stack.pop() {
        if has(skip, &node.kind()) {
            continue;
        }
        let children: Vec<_> = node.children().collect();
        if children.is_empty() {
            out.push(node.text().into_owned());
        }
        stack.extend(children.into_iter().rev());
    }
    out
}

/// `changed` for a real, parseable change; otherwise why it is invalid.
pub fn classify_mutant(
    pack: &Pack,
    language: &str,
    original: &str,
    mutated: &str,
) -> Result<&'static str, String> {
    let lang = grammar(language)?;
    let mutated_root = parse(mutated, lang)?;
    if walk(&mutated_root.root()).iter().any(|n| n.is_error()) {
        return Ok("parse_error");
    }
    let original_root = parse(original, lang)?;
    let skip = &pack.comment_kinds;
    if tokens(&original_root.root(), skip) == tokens(&mutated_root.root(), skip) {
        return Ok("no_op");
    }
    Ok("changed")
}
