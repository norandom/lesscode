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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::driver::registry::embedded;

    fn size(language: &str, text: &str) -> Size {
        measure(embedded(language).unwrap(), "sample", text).unwrap()
    }

    #[test]
    fn layout_and_comments_do_not_change_tokens_or_nodes() {
        let spread = "fn f(a: i32) -> i32 {\n    // why\n    if a > 0 {\n        return 1;\n    }\n    0\n}\n";
        let dense = "fn f(a: i32) -> i32 { if a > 0 { return 1; } 0 }\n";
        let (a, b) = (size("rust", spread), size("rust", dense));
        assert_eq!((a.tokens, a.nodes), (b.tokens, b.nodes));
        assert_eq!((a.lines, b.lines), (6, 1), "lines do depend on layout");
        let (a, b) = (
            size("python", "def f(a):\n    # note\n    return a + 1\n"),
            size("python", "def f(a): return a + 1\n"),
        );
        assert_eq!((a.tokens, a.nodes), (b.tokens, b.nodes));
    }

    #[test]
    fn removing_code_lowers_the_measure_and_a_broken_text_is_refused() {
        let more = size("java", "class A { int f() { int x = 1; return x; } }\n");
        let less = size("java", "class A { int f() { return 1; } }\n");
        assert!(less.tokens < more.tokens && less.nodes < more.nodes);
        let ts = size("typescript", "export const f = (a: number) => a + 1;\n");
        assert!(ts.tokens > 5);
        let error = measure(embedded("rust").unwrap(), "broken.rs", "fn f( {").unwrap_err();
        assert!(error.contains("does not parse"), "{error}");
    }
}
