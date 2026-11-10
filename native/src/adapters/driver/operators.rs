//! Operators: the mutants a pack gives for a site, and whether a mutated
//! text is a real, parseable change.

use super::pack::{has, partners, Pack};
use super::sites::{next_check, Site};
use super::tree::{chain, grammar, named_children, parse, position, walk, SgNode};
use super::Kind;

/// A mutated file: its full text and the replaced byte ranges of the
/// original.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mutant {
    pub operator: String,
    pub text: String,
    pub spans: Vec<(usize, usize)>,
}

type Edits<'r> = Vec<(SgNode<'r>, String)>;
type Labelled<'r> = (String, Edits<'r>);

/// Fill `{text}`, `{noop}` and `{child0}` in a template, left to right.
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
            "noop" => pack.noop_statement.clone(),
            "child0" => named_children(node, &pack.comment_kinds)
                .first()
                .map(|child| child.text().into_owned())
                .ok_or_else(|| format!("{{child0}} on a {} without named children", node.kind()))?,
            other => return Err(format!("unknown template placeholder {{{other}}}")),
        };
        out.push_str(&value);
        rest = &after[name_len + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// The pack's own operator for the site, if it has one.
fn first_edit<'r>(
    pack: &Pack,
    kind: Kind,
    node: &SgNode<'r>,
) -> Result<Option<Labelled<'r>>, String> {
    let ops = &pack.operators;
    let node_kind = node.kind();
    let one =
        |label: String, target: &SgNode<'r>, new: String| (label, vec![(target.clone(), new)]);
    match kind {
        Kind::BranchCondition => {
            let Some(template) = &ops.branch_condition.template else {
                return Ok(None);
            };
            Ok(Some(one(
                "negate".into(),
                node,
                fill(template, pack, node)?,
            )))
        }
        Kind::FailureState => {
            let Some(template) = ops.failure_state.by_kind.get(node_kind.as_ref()) else {
                return Ok(None);
            };
            let label = if template == "{noop}" {
                "drop"
            } else {
                "rewrite"
            };
            let new = fill(template, pack, node)?;
            Ok(Some(one(format!("{label}:{node_kind}"), node, new)))
        }
        Kind::StateWrite => {
            let parent = node
                .parent()
                .filter(|p| has(&pack.statement_wrappers, &p.kind()));
            let (Some(statement), Some(parent)) = (&ops.state_write.statement, parent) else {
                return Ok(None);
            };
            let new = fill(statement, pack, &parent)?;
            Ok(Some(one("drop_statement".into(), &parent, new)))
        }
        Kind::ErrorOrder => {
            let rule = pack.rule(kind).filter(|_| ops.error_order.swap_next);
            let first = chain(node).swap_remove(0);
            let sibling = rule.and_then(|rule| next_check(&first, rule, &pack.comment_kinds));
            Ok(sibling.map(|sibling| {
                let (a, b) = (first.text().into_owned(), sibling.text().into_owned());
                ("swap_next".into(), vec![(first, b), (sibling, a)])
            }))
        }
        Kind::PureShellBoundary => Ok(None),
    }
}

/// The first two operator tokens of a condition that have a partner, in
/// source order. Tokens inside comments and strings are not operators. When
/// the condition is also negated, a swap of its own operator that says the
/// same as the negation is left out.
fn token_swaps<'r>(pack: &Pack, node: &SgNode<'r>, negated: bool) -> Vec<Labelled<'r>> {
    let ops = &pack.operators.branch_condition;
    let partner = partners(&ops.token_swaps);
    let same_as_negation = |token: &SgNode<'r>| {
        let text = token.text();
        let listed = |(a, b): &(String, String)| *a == text || *b == text;
        negated
            && ops.negation_swaps.iter().any(listed)
            && token.parent().is_some_and(|p| p.range() == node.range())
    };
    if partner.is_empty() {
        return Vec::new();
    }
    let mut found: Vec<SgNode<'r>> = Vec::new();
    let mut stack = vec![node.clone()];
    while let Some(parent) = stack.pop() {
        for child in parent.children() {
            if child.is_named() {
                if !pack.is_hidden(&child.kind()) {
                    stack.push(child);
                }
            } else if partner.contains_key(child.text().as_ref()) {
                found.push(child);
            }
        }
    }
    found.sort_by_key(|t| t.range().start);
    found.truncate(2);
    found.retain(|token| !same_as_negation(token));
    let swap = |token: SgNode<'r>| {
        let old = token.text().into_owned();
        let new = partner[&old].clone();
        let (line, column, _, _) = position(&token);
        (
            format!("swap:{old}->{new}@{line}:{column}"),
            vec![(token, new)],
        )
    };
    found.into_iter().map(swap).collect()
}

/// A failure site without its own template whose one named child is a
/// listed literal.
fn literal_swap<'r>(pack: &Pack, node: &SgNode<'r>) -> Option<Labelled<'r>> {
    let ops = &pack.operators.failure_state;
    let named: Vec<_> = node.children().filter(|c| c.is_named()).collect();
    if ops.by_kind.contains_key(node.kind().as_ref()) || named.len() != 1 {
        return None;
    }
    let old = named[0].text().into_owned();
    let new = partners(&ops.returned_literal_swaps).get(&old)?.clone();
    Some((
        format!("literal:{old}->{new}"),
        vec![(named[0].clone(), new)],
    ))
}

fn all_edits<'r>(pack: &Pack, kind: Kind, node: &SgNode<'r>) -> Result<Vec<Labelled<'r>>, String> {
    let mut found: Vec<_> = first_edit(pack, kind, node)?.into_iter().collect();
    match kind {
        Kind::BranchCondition => found.extend(token_swaps(pack, node, !found.is_empty())),
        Kind::FailureState => found.extend(literal_swap(pack, node)),
        _ => {}
    }
    Ok(found)
}

/// Replace each node by exact offsets, last first; never by searching text.
fn apply(text: &str, operator: String, mut edits: Edits<'_>) -> Result<Mutant, String> {
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
    Ok(Mutant {
        operator,
        text: out,
        spans,
    })
}

/// Every mutant the pack gives for a site of `text`, in a stable order.
pub fn mutants(pack: &Pack, text: &str, site: &Site) -> Result<Vec<Mutant>, String> {
    let root = parse(text, grammar(&pack.language)?)?;
    let node = walk(&root.root())
        .into_iter()
        .find(|n| n.kind() == site.node_kind.as_str() && position(n) == site.span())
        .ok_or_else(|| format!("site no longer present in {}: {site:?}", site.file))?;
    all_edits(pack, site.kind, &node)?
        .into_iter()
        .map(|(operator, edits)| apply(text, operator, edits))
        .collect()
}

/// What a mutated text is, compared with the original.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Changed,
    NoOp,
    ParseError,
}

impl Change {
    pub fn name(self) -> &'static str {
        match self {
            Change::Changed => "changed",
            Change::NoOp => "no_op",
            Change::ParseError => "parse_error",
        }
    }
}

/// The text of every token outside comments, in order.
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

/// What `mutated` is, given the tokens of the original.
fn change(pack: &Pack, original_tokens: &[String], mutated: &str) -> Result<Change, String> {
    let mutated_root = parse(mutated, grammar(&pack.language)?)?;
    if walk(&mutated_root.root()).iter().any(|n| n.is_error()) {
        return Ok(Change::ParseError);
    }
    if original_tokens == tokens(&mutated_root.root(), &pack.comment_kinds) {
        return Ok(Change::NoOp);
    }
    Ok(Change::Changed)
}

/// `Changed` for a real, parseable change; otherwise why the mutant is
/// invalid.
pub fn classify(pack: &Pack, original: &str, mutated: &str) -> Result<Change, String> {
    let original_root = parse(original, grammar(&pack.language)?)?;
    change(
        pack,
        &tokens(&original_root.root(), &pack.comment_kinds),
        mutated,
    )
}

/// The mutants of one site: those that are a real change, and how many
/// the pack offered in all.
pub struct SiteMutants {
    pub valid: Vec<Mutant>,
    pub offered: usize,
}

/// The mutants of several sites of one text, which is parsed once.
pub fn mutants_of_sites(
    pack: &Pack,
    text: &str,
    sites: &[&Site],
) -> Result<Vec<SiteMutants>, String> {
    let root = parse(text, grammar(&pack.language)?)?;
    let nodes = walk(&root.root());
    let original_tokens = tokens(&root.root(), &pack.comment_kinds);
    let mut found = Vec::new();
    for site in sites {
        let node = nodes
            .iter()
            .find(|n| n.kind() == site.node_kind.as_str() && position(n) == site.span())
            .ok_or_else(|| format!("site no longer present in {}: {site:?}", site.file))?;
        let mut of_site = SiteMutants {
            valid: Vec::new(),
            offered: 0,
        };
        for (operator, edits) in all_edits(pack, site.kind, node)? {
            let mutant = apply(text, operator, edits)?;
            of_site.offered += 1;
            if change(pack, &original_tokens, &mutant.text)? == Change::Changed {
                of_site.valid.push(mutant);
            }
        }
        found.push(of_site);
    }
    Ok(found)
}
