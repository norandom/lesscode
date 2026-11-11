//! The size of a source text by its syntax, so that layout cannot change
//! it: tokens and named nodes outside comments. Lines are counted too, for
//! information only; they do depend on layout.

use super::pack::Pack;
use super::tree::{ancestors, first_error, grammar, parse, position, walk};
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Size {
    /// Leaves of the syntax tree outside comments: names, literals,
    /// operators, punctuation.
    pub tokens: usize,
    /// Named syntax nodes outside comments.
    pub nodes: usize,
    /// Lines on which at least one token starts. Depends on layout.
    pub lines: usize,
}

/// A token: where it starts (1-based line, 0-based character column) and
/// its text.
pub type Token = ((usize, usize), String);

/// Where each token outside comments starts: 1-based line, 0-based
/// character column, in source order. For measures of a part of a file.
pub fn token_starts(pack: &Pack, label: &str, text: &str) -> Result<Vec<(usize, usize)>, String> {
    let tokens = tokens(pack, label, text)?;
    Ok(tokens.into_iter().map(|(start, _)| start).collect())
}

/// Each token outside comments with where it starts, in source order.
pub fn tokens(pack: &Pack, label: &str, text: &str) -> Result<Vec<Token>, String> {
    let root = parse(text, grammar(&pack.language)?)?;
    if let Some((line, column)) = first_error(&root.root()) {
        return Err(format!(
            "{label}:{line}:{column}: the source does not parse"
        ));
    }
    let is_comment = |kind: &str| pack.comment_kinds.iter().any(|c| c == kind);
    let mut starts = Vec::new();
    for node in walk(&root.root()) {
        let leaf = node.children().next().is_none() && !node.text().is_empty();
        if leaf
            && !is_comment(&node.kind())
            && !ancestors(&node).iter().any(|a| is_comment(&a.kind()))
        {
            let (line, column, _, _) = position(&node);
            starts.push(((line, column), node.text().into_owned()));
        }
    }
    starts.sort();
    Ok(starts)
}

/// Measure `text` with the grammar and comment kinds of `pack`. A text that
/// does not parse is an error that says where parsing stopped.
pub fn measure(pack: &Pack, label: &str, text: &str) -> Result<Size, String> {
    let root = parse(text, grammar(&pack.language)?)?;
    if let Some((line, column)) = first_error(&root.root()) {
        return Err(format!(
            "{label}:{line}:{column}: the source does not parse"
        ));
    }
    let is_comment = |kind: &str| pack.comment_kinds.iter().any(|c| c == kind);
    let (mut tokens, mut nodes, mut lines) = (0, 0, BTreeSet::new());
    for node in walk(&root.root()) {
        if is_comment(&node.kind()) || ancestors(&node).iter().any(|a| is_comment(&a.kind())) {
            continue;
        }
        if node.is_named() {
            nodes += 1;
        }
        if node.children().next().is_none() && !node.text().is_empty() {
            tokens += 1;
            lines.insert(position(&node).0);
        }
    }
    Ok(Size {
        tokens,
        nodes,
        lines: lines.len(),
    })
}
