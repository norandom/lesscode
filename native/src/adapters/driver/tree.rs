//! Syntax-tree helpers shared by the site rules and the operators.
//!
//! Spans are compared as byte ranges. Positions shown to people are 1-based
//! lines and 0-based character columns.

use ast_grep_core::matcher::KindMatcher;
use ast_grep_core::tree_sitter::{LanguageExt, StrDoc};
use ast_grep_core::{AstGrep, Node};
use ast_grep_language::SupportLang;
use std::panic;
use std::str::FromStr;
use std::sync::Mutex;

pub type Doc = StrDoc<SupportLang>;
pub type SgNode<'r> = Node<'r, Doc>;
pub type SgRoot = AstGrep<Doc>;

/// The grammar named `name`, if the parser knows it and it is compiled in.
///
/// The parser crate lists every language but panics when a grammar's feature
/// is off. The probe turns that panic into an error; the lock keeps two
/// probes from interleaving their swaps of the process-wide panic hook.
pub fn grammar(name: &str) -> Result<SupportLang, String> {
    static PROBE: Mutex<()> = Mutex::new(());
    let lang = SupportLang::from_str(name).map_err(|e| e.to_string())?;
    let guard = PROBE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let probe = panic::catch_unwind(|| lang.get_ts_language());
    panic::set_hook(hook);
    drop(guard);
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

/// Every node in pre-order, unnamed tokens included.
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

/// Line and column of the first place the text does not parse, if any.
pub fn first_error(root: &SgNode<'_>) -> Option<(usize, usize)> {
    walk(root)
        .into_iter()
        .find(|n| n.is_error() || n.is_missing())
        .map(|n| {
            let (line, column, _, _) = position(&n);
            (line, column)
        })
}

/// Parent, grandparent and so on up to the root.
pub fn ancestors<'r>(node: &SgNode<'r>) -> Vec<SgNode<'r>> {
    let mut out = Vec::new();
    let mut parent = node.parent();
    while let Some(p) = parent {
        parent = p.parent();
        out.push(p);
    }
    out
}

/// The node plus the wrappers and wrapped nodes that cover the same text,
/// outermost first.
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

/// Named children that are not comments.
pub fn named_children<'r>(node: &SgNode<'r>, comment_kinds: &[String]) -> Vec<SgNode<'r>> {
    node.children()
        .filter(|c| c.is_named() && !comment_kinds.iter().any(|k| *k == c.kind()))
        .collect()
}

/// Start line, start column, end line, end column.
pub fn position(node: &SgNode<'_>) -> (usize, usize, usize, usize) {
    let (start, end) = (node.start_pos(), node.end_pos());
    (
        start.line() + 1,
        start.column(node),
        end.line() + 1,
        end.column(node),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::driver::registry::shipped;

    /// A grammar taken from the shipped packs, so no language is named here.
    fn any_grammar() -> SupportLang {
        grammar(&shipped().unwrap()[0].language).unwrap()
    }

    #[test]
    fn every_shipped_pack_has_a_compiled_grammar() {
        for pack in shipped().unwrap() {
            let lang = grammar(&pack.language).unwrap();
            for kind in pack.comment_kinds.iter().chain(&pack.string_kinds) {
                assert!(kind_is_valid(lang, kind), "{}: {kind}", pack.language);
            }
            assert!(!kind_is_valid(lang, "no_such_node_kind"));
        }
    }

    #[test]
    fn an_unknown_or_absent_grammar_is_an_error_not_a_panic() {
        assert!(grammar("klingon").is_err());
        assert!(grammar("haskell").unwrap_err().contains("not compiled in"));
    }

    #[test]
    fn columns_count_characters_and_lines_survive_crlf() {
        let text = "class A {\r\n  int f() { return \"\u{e9}\u{e9}\".length() + x; }\r\n}\r\n";
        let root = parse(text, any_grammar()).unwrap();
        assert_eq!(first_error(&root.root()), None);
        let x = walk(&root.root())
            .into_iter()
            .find(|n| n.is_named() && n.text() == "x")
            .unwrap();
        assert_eq!(position(&x), (2, 35, 2, 36));
        assert_eq!(&text[x.range()], "x");
    }

    #[test]
    fn broken_text_reports_where_it_stops_parsing() {
        let root = parse("class A {\n  int f() { return ((((; }\n}\n", any_grammar()).unwrap();
        let (line, _) = first_error(&root.root()).unwrap();
        assert_eq!(line, 2);
    }

    #[test]
    fn a_chain_covers_wrappers_with_the_same_text() {
        let root = parse("class A { void f() { g(); } }", any_grammar()).unwrap();
        let call = walk(&root.root())
            .into_iter()
            .find(|n| n.text() == "g()")
            .unwrap();
        let kinds: Vec<_> = chain(&call).iter().map(|n| n.kind().to_string()).collect();
        assert_eq!(kinds.len(), 1, "{kinds:?}");
        let statement = call.parent().unwrap();
        assert!(ancestors(&call).len() > 3);
        assert_eq!(named_children(&statement, &[]).len(), 1);
    }
}
