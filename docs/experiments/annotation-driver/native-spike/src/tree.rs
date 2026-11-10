//! Syntax-tree helpers shared by the validator and the engine.
//!
//! Spans are compared as byte ranges. Within one parsed text a byte offset
//! and a (line, character column) position determine each other, so this is
//! the same comparison as the prototype's `_span` tuples.

use ast_grep_core::matcher::KindMatcher;
use ast_grep_core::tree_sitter::{LanguageExt, StrDoc};
use ast_grep_core::{AstGrep, Node};
use ast_grep_language::SupportLang;
use std::panic;
use std::str::FromStr;

pub type Doc = StrDoc<SupportLang>;
pub type SgNode<'r> = Node<'r, Doc>;
pub type SgRoot = AstGrep<Doc>;

/// The grammar named `name`, if ast-grep knows it and it is compiled in.
///
/// ast-grep-language lists every language but panics when the grammar's
/// feature is off; the probe turns that panic into an error.
pub fn grammar(name: &str) -> Result<SupportLang, String> {
    let lang = SupportLang::from_str(name).map_err(|e| e.to_string())?;
    let hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let probe = panic::catch_unwind(|| lang.get_ts_language());
    panic::set_hook(hook);
    probe
        .map(|_| lang)
        .map_err(|_| format!("{name}: grammar not compiled in"))
}

pub fn parse(text: &str, lang: SupportLang) -> Result<SgRoot, String> {
    StrDoc::try_new(text, lang).map(AstGrep::doc)
}

pub fn kind_is_valid(lang: SupportLang, kind: &str) -> bool {
    KindMatcher::try_new(kind, lang).is_ok()
}

/// Every node in pre-order, unnamed tokens included (`validator._walk`).
pub fn walk<'r>(root: &SgNode<'r>) -> Vec<SgNode<'r>> {
    let mut out = Vec::new();
    let mut stack = vec![root.clone()];
    while let Some(node) = stack.pop() {
        let children: Vec<_> = node.children().collect();
        stack.extend(children.into_iter().rev());
        out.push(node);
    }
    out
}

/// Parent, grandparent, ... up to the root (`validator._ancestors`).
pub fn ancestors<'r>(node: &SgNode<'r>) -> Vec<SgNode<'r>> {
    let mut out = Vec::new();
    let mut parent = node.parent();
    while let Some(p) = parent {
        parent = p.parent();
        out.push(p);
    }
    out
}

/// The node plus the wrappers and wrapped nodes that cover the same text.
pub fn chain<'r>(node: &SgNode<'r>) -> Vec<SgNode<'r>> {
    let span = node.range();
    let mut out: Vec<SgNode<'r>> = Vec::new();
    let mut parent = node.parent();
    while let Some(p) = parent {
        if p.range() != span {
            break;
        }
        parent = p.parent();
        out.push(p);
    }
    out.reverse();
    out.push(node.clone());
    let mut inner = node.clone();
    loop {
        let same = inner.children().find(|c| c.range() == span);
        let Some(same) = same else { break };
        out.push(same.clone());
        inner = same;
    }
    out
}

/// Named children that are not comments (`engine._named_children`).
pub fn named_children<'r>(node: &SgNode<'r>, comment_kinds: &[String]) -> Vec<SgNode<'r>> {
    node.children()
        .filter(|c| c.is_named() && !comment_kinds.iter().any(|k| *k == c.kind()))
        .collect()
}

/// 1-based start line, 0-based character columns, as the prototype's `Site`.
pub fn position(node: &SgNode<'_>) -> (usize, usize, usize, usize) {
    let (start, end) = (node.start_pos(), node.end_pos());
    (
        start.line() + 1,
        start.column(node),
        end.line() + 1,
        end.column(node),
    )
}
