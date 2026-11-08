//! Tier-1 structural checks and the three-reading scope measure for Python
//! specimen files — the Rust-substrate port of
//! `src/lesscode_baseline/measures.py` (task 8.3, R36).
//!
//! Field names, key order and verdict precedence are stable with the Python
//! reference module. Port deltas (recorded with the R36 limits): the effect
//! vocabulary is fixed to the reference DEFAULT sqlalchemy profile — the
//! Python `EffectProfile` regex pluggability is not carried over — and the
//! canonical reading shells out to the same pinned `uv tool run ruff`
//! command, returning `None` (never zeros) when the formatter is
//! unavailable or reports a different version. Python AST knowledge stays
//! behind this seam (tree-sitter-python, extractor `rust-tspython-v1`);
//! candidate code is parsed, never executed.

use crate::receipt::{sha256_hex, J};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

type PyNode<'r> = ast_grep_core::Node<'r, ast_grep_core::tree_sitter::StrDoc<crate::Python>>;

pub const MEASURE_VERSION: &str = "scope-measures-v1";

// DEFAULT sqlalchemy effect vocabulary (frozen; not pluggable in the port).
// Write sites match `\.(attr)\(` first — the attr form consumes through the
// '(' so `.delete(` never also matches the bare `\b(delete)\(` branch — and
// otherwise `\b(func)\(`, exactly like the reference regex alternation.
const WRITE_ATTRS: [&str; 7] = [
    "execute", "add", "add_all", "commit", "flush", "delete", "merge",
];
const WRITE_FUNCS: [&str; 3] = ["insert", "update", "delete"];
// Param names forbidden in pure functions: exact matches, or the reference
// suffixes (no `_bind` suffix — faithful to the reference pattern).
const BAD_PARAM_EXACT: [&str; 5] = ["engine", "conn", "connection", "session", "bind"];
const BAD_PARAM_SUFFIX: [&str; 4] = ["_engine", "_conn", "_connection", "_session"];
// Attribute calls forbidden inside pure functions.
const SQL_ATTRS: [&str; 8] = [
    "execute", "begin", "connect", "commit", "add", "flush", "merge", "delete",
];

const SEPARATED_KEYS: [&str; 8] = [
    "import",
    "from_import",
    "tuple_assign",
    "multi_target",
    "chained_compare",
    "del",
    "global_nonlocal",
    "with_items",
];
const X_IMPORT: usize = 0;
const X_FROM_IMPORT: usize = 1;
const X_TUPLE: usize = 2;
const X_MULTI: usize = 3;
const X_CHAINED: usize = 4;
const X_DEL: usize = 5;
const X_GLOBAL: usize = 6;
const X_WITH: usize = 7;

const SEPARATED_RULE_TEXT: &str = "1. Every ast.stmt node counts 1, exactly as the frozen `stmts`; rules 2-8 only add extras to it.\n2. `import a, b, c` counts one statement per name (extra len(names) - 1).\n3. `from m import a, b` counts one statement per name (extra len(names) - 1); `from m import *` counts 1.\n4. An assignment with one target that is a tuple or list, and a tuple or list value of the same length, neither holding a starred element, counts one statement per paired leaf, recursively.\n5. An assignment with several targets `a = b = v` counts len(targets); rule 4 is then not applied to its targets.\n6. A comparison with k comparators counts k, wherever it occurs.\n7. `del a, b` counts len(targets); `global a, b` and `nonlocal a, b` count len(names).\n8. A `with`/`async with` statement with n items counts n.\n9. Not unpacked (count only as rule 1): augmented and annotated assignments; a tuple target with a non-literal or starred value; boolean operators; conditional expressions; comprehensions; lambdas; walrus; for-loop targets; multi-type `except`; tuple returns, yields and call arguments; decorators; semicolon-joined lines.";

#[derive(Debug)]
pub struct MeasuresError(pub String);
impl std::fmt::Display for MeasuresError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for MeasuresError {}

/// Declared pure/shell structure under validation, keys `file::qualname`
/// (scope-path-qualified declarations are normalized during lookup).
pub struct Structure {
    pub pure: Vec<String>,
    pub shell: Vec<String>,
}

/// One counted `def` (classes extend qualnames but are not entries).
struct DefEntry {
    qualname: String,
    bare: String,
    start_line: usize,
    end_line: usize,
    params: Vec<String>,
}

/// One file's single-pass readings: written statements (with start lines for
/// attribution), defs, calls, forbidden attribute calls and rule-v1 extras.
#[derive(Default)]
struct FileFacts {
    stmts: usize,
    stmt_lines: Vec<usize>,
    defs: Vec<DefEntry>,
    calls: Vec<(String, usize)>,
    sql_calls: Vec<(String, usize)>,
    extra: [usize; 8],
}

fn upsert<V>(map: &mut Vec<(String, V)>, key: String, value: V) {
    if let Some(entry) = map.iter_mut().find(|(k, _)| *k == key) {
        entry.1 = value;
    } else {
        map.push((key, value));
    }
}

fn bump(map: &mut Vec<(String, i64)>, key: String, delta: i64) {
    if let Some(entry) = map.iter_mut().find(|(k, _)| *k == key) {
        entry.1 += delta;
    } else {
        map.push((key, delta));
    }
}

fn read_source(path: &Path) -> Result<String, MeasuresError> {
    crate::surface::read_source(path).map_err(|e| MeasuresError(e.0))
}

fn basename(rel: &str) -> String {
    Path::new(rel)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// `file::qualname` with a scope-path-qualified file head normalized to its
/// basename (G1-R live finding, 2026-11-08).
fn norm_key(key: &str) -> String {
    match key.rsplit_once("::") {
        Some((head, qual)) => format!("{}::{qual}", basename(head)),
        None => key.to_string(),
    }
}

fn is_bad_param(name: &str) -> bool {
    BAD_PARAM_EXACT.contains(&name) || BAD_PARAM_SUFFIX.iter().any(|s| name.ends_with(s))
}

/// Python `round(x, 3)`: correctly-rounded half-even at the third decimal.
/// Rust's fixed-precision float Display is correctly rounded with half-even
/// ties, so re-parsing reproduces CPython's dtoa-based result.
fn round3(x: f64) -> f64 {
    format!("{x:.3}").parse().unwrap_or(x)
}

/// CPython `str.splitlines` boundary set (not just `\n`).
fn splitlines_count(text: &str) -> usize {
    fn is_break(c: char) -> bool {
        matches!(
            c,
            '\n' | '\r'
                | '\u{b}'
                | '\u{c}'
                | '\u{1c}'
                | '\u{1d}'
                | '\u{1e}'
                | '\u{85}'
                | '\u{2028}'
                | '\u{2029}'
        )
    }
    let mut breaks = 0;
    let mut last_was_break = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if is_break(c) {
            if c == '\r' && chars.peek() == Some(&'\n') {
                chars.next();
            }
            breaks += 1;
            last_was_break = true;
        } else {
            last_was_break = false;
        }
    }
    breaks + usize::from(!text.is_empty() && !last_was_break)
}

// --- single-pass structural analysis -------------------------------------

/// Dotted qualname of a def: enclosing defs and classes prefix it, like the
/// reference `_defs` walker (decorator wrappers are not defs).
fn qualname_of(node: &PyNode) -> String {
    let mut parts = vec![node
        .field("name")
        .map(|n| n.text().into_owned())
        .unwrap_or_default()];
    let mut cur = node.parent();
    while let Some(p) = cur {
        if matches!(
            p.kind().as_ref(),
            "function_definition" | "class_definition"
        ) {
            if let Some(n) = p.field("name") {
                parts.push(n.text().into_owned());
            }
        }
        cur = p.parent();
    }
    parts.reverse();
    parts.join(".")
}

/// Parameter names including `*args`/`**kwargs` splat names (used by the
/// bad-param check, unlike the surface rebinding pass). `why`-list order
/// follows the reference: non-splat parameters in source order, then the
/// vararg/kwarg names.
fn param_names(def: &PyNode) -> Vec<String> {
    let Some(params) = def.field("parameters") else {
        return Vec::new();
    };
    let mut regular = Vec::new();
    let mut splats = Vec::new();
    for child in params.named_children() {
        match child.kind().as_ref() {
            "identifier" => regular.push(child.text().into_owned()),
            "default_parameter" | "typed_parameter" | "typed_default_parameter" => {
                if let Some(n) = child.field("name") {
                    if n.kind() == "identifier" {
                        regular.push(n.text().into_owned());
                    }
                }
            }
            "list_splat_pattern"
            | "list_splat"
            | "dictionary_splat_pattern"
            | "dictionary_splat" => {
                for inner in child.named_children() {
                    if inner.kind() == "identifier" {
                        splats.push(inner.text().into_owned());
                    }
                }
            }
            _ => {}
        }
    }
    regular.append(&mut splats);
    regular
}

fn is_stmt_kind(kind: &str) -> bool {
    // Remaining ast.stmt kinds after the special BFS arms; `elif_clause`
    // counts because Python represents `elif` as a nested `If` statement,
    // and `else`/`except`/`finally` clauses are not statements.
    matches!(
        kind,
        "augmented_assignment"
            | "annotated_assignment"
            | "class_definition"
            | "return_statement"
            | "if_statement"
            | "elif_clause"
            | "while_statement"
            | "for_statement"
            | "try_statement"
            | "raise_statement"
            | "assert_statement"
            | "pass_statement"
            | "break_statement"
            | "continue_statement"
            | "match_statement"
    )
}

/// Chained assignment `a = b = v` parses as assignment(right: assignment);
/// the inner node is not a statement and adds no extras (rule 5).
fn is_nested_right(node: &PyNode) -> bool {
    node.parent().is_some_and(|p| {
        p.kind() == "assignment" && p.field("right").is_some_and(|r| r.range() == node.range())
    })
}

/// Targets of a chained assignment: this node plus the assignments nested
/// as its `right` child (`a = b = v` → 2).
fn chain_len(node: &PyNode) -> usize {
    let mut n = 1;
    let mut cur = node.field("right");
    while let Some(r) = cur {
        if r.kind() != "assignment" {
            break;
        }
        n += 1;
        cur = r.field("right");
    }
    n
}

fn has_starred(node: &PyNode) -> bool {
    node.named_children().any(|c| {
        matches!(
            c.kind().as_ref(),
            "starred_expression"
                | "list_splat"
                | "list_splat_pattern"
                | "dictionary_splat"
                | "dictionary_splat_pattern"
        )
    })
}

/// Rule 4: paired leaves of a single-target sequence assignment, else 1.
/// Target sequences are tree-sitter pattern kinds; value sequences are
/// expression kinds (`1, 2` parses as expression_list, `(1, 2)` as tuple).
fn leaves(target: &PyNode, value: &PyNode) -> usize {
    let tseq = |kind: &str| matches!(kind, "pattern_list" | "list_pattern" | "tuple_pattern");
    let vseq = |kind: &str| matches!(kind, "expression_list" | "tuple" | "list");
    if tseq(target.kind().as_ref()) && vseq(value.kind().as_ref()) {
        let t: Vec<_> = target.named_children().collect();
        let v: Vec<_> = value.named_children().collect();
        if t.len() == v.len() && !has_starred(target) && !has_starred(value) {
            return t.iter().zip(&v).map(|(a, b)| leaves(a, b)).sum();
        }
    }
    1
}

fn analyze(source: &str) -> FileFacts {
    let tree = crate::AstGrep::new(source, crate::Python);
    let root = tree.root();
    let mut facts = FileFacts::default();
    let Some(root_node) = root.dfs().next() else {
        return facts;
    };
    let starts = crate::surface::line_starts(source);
    let mut level: Vec<_> = root_node.named_children().collect();
    while !level.is_empty() {
        let mut next = Vec::new();
        for node in &level {
            next.extend(node.named_children());
            let kind = node.kind();
            let line = crate::surface::line_of(&starts, node.range().start);
            let count_stmt = |facts: &mut FileFacts| {
                facts.stmts += 1;
                facts.stmt_lines.push(line);
            };
            match kind.as_ref() {
                "function_definition" => {
                    let range = node.range();
                    facts.defs.push(DefEntry {
                        qualname: qualname_of(node),
                        bare: node
                            .field("name")
                            .map(|n| n.text().into_owned())
                            .unwrap_or_default(),
                        start_line: line,
                        end_line: crate::surface::line_of(
                            &starts,
                            range.end.saturating_sub(1).max(range.start),
                        ),
                        params: param_names(node),
                    });
                    count_stmt(&mut facts);
                }
                // A bare expression statement; assignments (including
                // augmented/annotated) are counted by their own node.
                "expression_statement" => {
                    let wrapped = node.named_children().any(|c| {
                        matches!(
                            c.kind().as_ref(),
                            "assignment" | "annotated_assignment" | "augmented_assignment"
                        )
                    });
                    if !wrapped {
                        count_stmt(&mut facts);
                    }
                }
                "assignment" => {
                    if !is_nested_right(node) {
                        count_stmt(&mut facts);
                        let chain = chain_len(node);
                        if chain > 1 {
                            facts.extra[X_MULTI] += chain - 1;
                        } else if let (Some(target), Some(value)) =
                            (node.field("left"), node.field("right"))
                        {
                            facts.extra[X_TUPLE] += leaves(&target, &value).saturating_sub(1);
                        }
                    }
                }
                "import_statement" => {
                    count_stmt(&mut facts);
                    let names = node
                        .named_children()
                        .filter(|c| matches!(c.kind().as_ref(), "dotted_name" | "aliased_import"))
                        .count();
                    facts.extra[X_IMPORT] += names.saturating_sub(1);
                }
                "import_from_statement" | "future_import_statement" => {
                    count_stmt(&mut facts);
                    let module_range = node.field("module_name").map(|m| m.range());
                    let names = node
                        .named_children()
                        .filter(|c| {
                            Some(c.range()) != module_range
                                && matches!(
                                    c.kind().as_ref(),
                                    "dotted_name" | "aliased_import" | "wildcard_import"
                                )
                        })
                        .count();
                    facts.extra[X_FROM_IMPORT] += names.saturating_sub(1);
                }
                "delete_statement" => {
                    count_stmt(&mut facts);
                    // `del a, b` wraps its targets in an expression_list.
                    let mut targets = 0;
                    for child in node.named_children() {
                        if child.kind() == "expression_list" {
                            targets += child.named_children().count();
                        } else {
                            targets += 1;
                        }
                    }
                    facts.extra[X_DEL] += targets.saturating_sub(1);
                }
                "global_statement" | "nonlocal_statement" => {
                    count_stmt(&mut facts);
                    let names = node.named_children().count();
                    facts.extra[X_GLOBAL] += names.saturating_sub(1);
                }
                "with_statement" => {
                    count_stmt(&mut facts);
                    // Items live under a with_clause child.
                    let items: usize = node
                        .named_children()
                        .filter(|c| c.kind() == "with_clause")
                        .map(|c| {
                            c.named_children()
                                .filter(|i| i.kind() == "with_item")
                                .count()
                        })
                        .sum();
                    facts.extra[X_WITH] += items.saturating_sub(1);
                }
                "call" => {
                    if let Some(func) = node.field("function") {
                        match func.kind().as_ref() {
                            "identifier" => facts.calls.push((func.text().into_owned(), line)),
                            "attribute" => {
                                if let Some(attr) = func.field("attribute") {
                                    let name = attr.text().into_owned();
                                    facts.calls.push((name.clone(), line));
                                    if SQL_ATTRS.contains(&name.as_str()) {
                                        facts.sql_calls.push((name, line));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                // k comparators = named operands - 1, so the extra is
                // operands - 2 (rule 6, "wherever it occurs").
                "comparison_operator" => {
                    facts.extra[X_CHAINED] += node.named_children().count().saturating_sub(2);
                }
                other => {
                    if is_stmt_kind(other) {
                        count_stmt(&mut facts);
                    }
                }
            }
        }
        level = next;
    }
    facts
}

// --- write-site vocabulary matching --------------------------------------

fn is_word(b: u8) -> bool {
    // ASCII word chars; non-ASCII bytes count as word characters, matching
    // Python's Unicode \w for identifier text (non-word non-ASCII
    // punctuation could diverge; specimen sources are ASCII).
    b == b'_' || b.is_ascii_alphanumeric() || b >= 0x80
}

/// Matched span length when `buf` starts with one of `names` directly
/// followed by `(` — e.g. `.execute(` / `insert(`.
fn name_call(buf: &[u8], names: &[&str]) -> Option<usize> {
    for name in names {
        let bytes = name.as_bytes();
        if buf.len() > bytes.len() && &buf[..bytes.len()] == bytes && buf[bytes.len()] == b'(' {
            return Some(bytes.len() + 1);
        }
    }
    None
}

/// Text-order write-site matches with 1-based lines: attr form `.name(`
/// (consuming through the `(`, so it wins at the same parenthesis), else
/// bare `\bname(`.
fn write_matches(text: &str) -> Vec<(usize, String)> {
    let bytes = text.as_bytes();
    let starts = crate::surface::line_starts(text);
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'.' {
            if let Some(len) = name_call(&bytes[i + 1..], &WRITE_ATTRS) {
                let end = i + 1 + len;
                out.push((
                    crate::surface::line_of(&starts, i),
                    format!(".{}", &text[i + 1..end]),
                ));
                i = end;
                continue;
            }
        } else if is_word(bytes[i]) && (i == 0 || !is_word(bytes[i - 1])) {
            if let Some(len) = name_call(&bytes[i..], &WRITE_FUNCS) {
                let end = i + len;
                out.push((
                    crate::surface::line_of(&starts, i),
                    text[i..end].to_string(),
                ));
                i = end;
                continue;
            }
        }
        i += 1;
    }
    out
}

// --- as-written measure ---------------------------------------------------

/// One write site: (file basename, line, matched text, innermost def key).
type Site = (String, usize, String, Option<String>);

fn site_j(site: &Site, pass: Option<bool>) -> J {
    let (f, line, m, in_) = site;
    let mut pairs = vec![
        ("file", J::str(f.clone())),
        ("line", J::int(*line as i64)),
        ("match", J::str(m.clone())),
        ("in", in_.clone().map(J::Str).unwrap_or(J::Null)),
    ];
    if let Some(p) = pass {
        pairs.push(("pass", J::Bool(p)));
    }
    J::obj(pairs)
}

fn check_structure(facts: &[(String, FileFacts)], sites: &[Site], structure: &Structure) -> J {
    let mut nodes: Vec<(String, (usize, usize))> = Vec::new();
    for (fi, (f, facts_)) in facts.iter().enumerate() {
        for (di, d) in facts_.defs.iter().enumerate() {
            upsert(&mut nodes, format!("{f}::{}", d.qualname), (fi, di));
        }
    }
    let declared_pure: BTreeSet<String> = structure.pure.iter().map(|k| norm_key(k)).collect();
    let shell: BTreeSet<String> = structure.shell.iter().map(|k| norm_key(k)).collect();

    let mut pure: Vec<(String, J)> = Vec::new();
    for key in &declared_pure {
        let why: Vec<String> = match nodes.iter().find(|(k, _)| k == key) {
            None => vec!["not defined".into()],
            Some((_, (fi, di))) => {
                let facts_ = &facts[*fi].1;
                let d = &facts_.defs[*di];
                let mut w: Vec<String> = d
                    .params
                    .iter()
                    .filter(|p| is_bad_param(p))
                    .map(|p| format!("param {p}"))
                    .collect();
                for (attr, line) in &facts_.sql_calls {
                    if d.start_line <= *line && *line <= d.end_line {
                        w.push(format!("call .{attr} line {line}"));
                    }
                }
                w
            }
        };
        pure.push((
            key.clone(),
            J::obj(vec![
                ("pass", J::Bool(why.is_empty())),
                (
                    "why",
                    J::arr(why.iter().map(|w| J::str(w.clone())).collect()),
                ),
            ]),
        ));
    }
    let checked: Vec<(&Site, bool)> = sites
        .iter()
        .map(|s| (s, s.3.as_ref().is_some_and(|in_| shell.contains(in_))))
        .collect();
    // The reference builds `declared` from the RAW pure keys union the
    // NORMALIZED shell keys; preserved so undeclared/declared_unknown match
    // its verdicts exactly.
    let mut declared: BTreeSet<String> = structure.pure.iter().cloned().collect();
    declared.extend(shell.iter().cloned());
    let undeclared: BTreeSet<String> = nodes
        .iter()
        .map(|(k, _)| k)
        .filter(|k| !declared.contains(*k))
        .cloned()
        .collect();
    let unknown: Vec<String> = declared
        .iter()
        .filter(|k| !nodes.iter().any(|(n, _)| n == *k))
        .cloned()
        .collect();
    let pass = pure
        .iter()
        .all(|(_, j)| j_get(j, "pass") == Some(&J::Bool(true)))
        && checked.iter().all(|(_, p)| *p);
    let pure_count = pure.len();
    J::obj(vec![
        ("pure", J::Obj(pure)),
        (
            "write_sites",
            J::Arr(checked.iter().map(|(s, p)| site_j(s, Some(*p))).collect()),
        ),
        (
            "undeclared",
            J::Arr(undeclared.into_iter().map(J::str).collect()),
        ),
        (
            "declared_unknown",
            J::Arr(unknown.into_iter().map(J::str).collect()),
        ),
        (
            "pure_share",
            J::Float(round3(pure_count as f64 / nodes.len().max(1) as f64)),
        ),
        ("pass", J::Bool(pass)),
    ])
}

/// Count lines/stmts/funcs/qualnames/writes/edges/module_hops and validate
/// declared pure/shell structure. Output shape follows the frozen E3
/// measurer (files keyed by basename, `total`, sorted `edges`, `write_sites`
/// with innermost enclosing def, optional `structure` block).
pub fn measure_written(
    root: &Path,
    scope_files: &[String],
    structure: Option<&Structure>,
) -> Result<J, MeasuresError> {
    let mut src: Vec<(String, String)> = Vec::new();
    for rel in scope_files {
        let text = read_source(&root.join(rel))?;
        upsert(&mut src, basename(rel), text);
    }
    let facts: Vec<(String, FileFacts)> = src
        .iter()
        .map(|(f, text)| (f.clone(), analyze(text)))
        .collect();
    let facts_of = |f: &str| {
        &facts
            .iter()
            .find(|(name, _)| name == f)
            .expect("facts aligned with src")
            .1
    };

    let mut names: Vec<(String, BTreeSet<String>)> = Vec::new();
    for (f, facts_) in &facts {
        for d in &facts_.defs {
            match names.iter_mut().find(|(n, _)| *n == d.bare) {
                Some((_, files)) => {
                    files.insert(f.clone());
                }
                None => names.push((d.bare.clone(), BTreeSet::from([f.clone()]))),
            }
        }
    }
    let mut edges: BTreeSet<[String; 4]> = BTreeSet::new();
    for (f, facts_) in &facts {
        for d in &facts_.defs {
            for (callee, line) in &facts_.calls {
                if *line < d.start_line || *line > d.end_line || *callee == d.bare {
                    continue;
                }
                if let Some((_, files)) = names.iter().find(|(n, _)| n == callee) {
                    for g in files {
                        edges.insert([f.clone(), d.bare.clone(), g.clone(), callee.clone()]);
                    }
                }
            }
        }
    }
    let mut sites: Vec<(String, usize, String, Option<String>)> = Vec::new();
    for (f, text) in &src {
        for (line, m) in write_matches(text) {
            let inner = facts_of(f)
                .defs
                .iter()
                .filter(|d| d.start_line <= line && line <= d.end_line)
                .map(|d| (d.start_line, d.qualname.as_str()))
                .max();
            sites.push((f.clone(), line, m, inner.map(|(_, q)| format!("{f}::{q}"))));
        }
    }

    let mut files: Vec<(String, J)> = Vec::new();
    let mut sums = [0i64; 5];
    for (f, text) in &src {
        let facts_ = facts_of(f);
        let lines = splitlines_count(text) as i64;
        let funcs = facts_
            .defs
            .iter()
            .map(|d| d.bare.as_str())
            .collect::<BTreeSet<_>>()
            .len() as i64;
        let writes = sites.iter().filter(|(sf, ..)| sf == f).count() as i64;
        let row = [
            lines,
            facts_.stmts as i64,
            funcs,
            facts_.defs.len() as i64,
            writes,
        ];
        for (s, v) in sums.iter_mut().zip(row) {
            *s += v;
        }
        files.push((
            f.clone(),
            J::obj(vec![
                ("lines", J::int(row[0])),
                ("stmts", J::int(row[1])),
                ("funcs", J::int(row[2])),
                ("qualnames", J::int(row[3])),
                ("writes", J::int(row[4])),
            ]),
        ));
    }
    let hops: BTreeSet<(String, String)> = edges
        .iter()
        .filter(|e| e[0] != e[2])
        .map(|e| (e[0].clone(), e[2].clone()))
        .collect();
    let mut out = vec![
        ("measure_version", J::str(MEASURE_VERSION)),
        ("effect_profile", J::str("sqlalchemy")),
        ("root", J::str(root.to_string_lossy().into_owned())),
        ("files", J::Obj(files)),
        (
            "total",
            J::obj(vec![
                ("lines", J::int(sums[0])),
                ("stmts", J::int(sums[1])),
                ("funcs", J::int(sums[2])),
                ("qualnames", J::int(sums[3])),
                ("writes", J::int(sums[4])),
                ("edges", J::int(edges.len() as i64)),
                ("module_hops", J::int(hops.len() as i64)),
            ]),
        ),
        (
            "edges",
            J::Arr(
                edges
                    .iter()
                    .map(|e| {
                        J::arr(vec![
                            J::str(e[0].clone()),
                            J::str(e[1].clone()),
                            J::str(e[2].clone()),
                            J::str(e[3].clone()),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "write_sites",
            J::Arr(sites.iter().map(|s| site_j(s, None)).collect()),
        ),
    ];
    if let Some(structure) = structure {
        out.push(("structure", check_structure(&facts, &sites, structure)));
    }
    Ok(J::obj(out))
}

/// Attribute every counted statement to its innermost enclosing def (frozen
/// pureshare walker; declaration keys are used raw, not normalized).
pub fn measure_pure_share(
    root: &Path,
    scope_files: &[String],
    structure: &Structure,
) -> Result<J, MeasuresError> {
    let pure: BTreeSet<&str> = structure.pure.iter().map(String::as_str).collect();
    let shell: BTreeSet<&str> = structure.shell.iter().map(String::as_str).collect();
    let mut totals = [0i64; 4]; // pure, shell, undeclared, outside_defs
    let mut per_function: Vec<(String, i64)> = Vec::new(); // first-touch order
    for rel in scope_files {
        let name = basename(rel);
        let text = read_source(&root.join(rel))?;
        let facts = analyze(&text);
        for line in &facts.stmt_lines {
            let inner = facts
                .defs
                .iter()
                .filter(|d| d.start_line <= *line && *line <= d.end_line)
                .map(|d| (d.start_line, d.qualname.as_str()))
                .max();
            let (idx, key) = match inner {
                None => (3, None),
                Some((_, q)) => {
                    let key = format!("{name}::{q}");
                    let idx = if pure.contains(key.as_str()) {
                        0
                    } else if shell.contains(key.as_str()) {
                        1
                    } else {
                        2
                    };
                    (idx, Some(key))
                }
            };
            totals[idx] += 1;
            if let Some(key) = key {
                bump(&mut per_function, key, 1);
            }
        }
    }
    let in_defs = totals[0] + totals[1] + totals[2];
    let all = in_defs + totals[3];
    Ok(J::obj(vec![
        ("measure_version", J::str(MEASURE_VERSION)),
        ("root", J::str(root.to_string_lossy().into_owned())),
        (
            "stmts",
            J::obj(vec![
                ("pure", J::int(totals[0])),
                ("shell", J::int(totals[1])),
                ("undeclared", J::int(totals[2])),
                ("outside_defs", J::int(totals[3])),
            ]),
        ),
        ("stmts_in_defs", J::int(in_defs)),
        ("stmts_all", J::int(all)),
        (
            "pure_share_by_stmts_in_defs",
            J::Float(round3(totals[0] as f64 / in_defs.max(1) as f64)),
        ),
        (
            "pure_share_by_stmts_all",
            J::Float(round3(totals[0] as f64 / all.max(1) as f64)),
        ),
        (
            "pure_share_by_funcs",
            J::Float(round3(pure.len() as f64 / per_function.len().max(1) as f64)),
        ),
        (
            "per_function",
            J::Obj(
                per_function
                    .into_iter()
                    .map(|(k, v)| (k, J::int(v)))
                    .collect(),
            ),
        ),
    ]))
}

/// Separated-form statement counts under rule v1 (rules 1-9, frozen text).
pub fn measure_separated(root: &Path, scope_files: &[String]) -> Result<J, MeasuresError> {
    let mut files: Vec<(String, FileFacts)> = Vec::new();
    for rel in scope_files {
        let text = read_source(&root.join(rel))?;
        upsert(&mut files, basename(rel), analyze(&text));
    }
    let files_j: Vec<(String, J)> = files
        .iter()
        .map(|(f, facts_)| {
            let extra_total: usize = facts_.extra.iter().sum();
            (
                f.clone(),
                J::obj(vec![
                    ("stmts", J::int((facts_.stmts + extra_total) as i64)),
                    (
                        "extra",
                        J::Obj(
                            SEPARATED_KEYS
                                .iter()
                                .enumerate()
                                .map(|(i, k)| ((*k).to_string(), J::int(facts_.extra[i] as i64)))
                                .collect(),
                        ),
                    ),
                ]),
            )
        })
        .collect();
    let stmts_sum: i64 = files
        .iter()
        .map(|(_, f)| (f.stmts + f.extra.iter().sum::<usize>()) as i64)
        .sum();
    let extra_sums: [i64; 8] =
        std::array::from_fn(|i| files.iter().map(|(_, f)| f.extra[i] as i64).sum());
    Ok(J::obj(vec![
        ("measure_version", J::str(MEASURE_VERSION)),
        ("files", J::Obj(files_j)),
        (
            "total",
            J::obj(vec![
                ("stmts", J::int(stmts_sum)),
                (
                    "extra",
                    J::Obj(
                        SEPARATED_KEYS
                            .iter()
                            .enumerate()
                            .map(|(i, k)| ((*k).to_string(), J::int(extra_sums[i])))
                            .collect(),
                    ),
                ),
            ]),
        ),
        (
            "rule",
            J::obj(vec![
                ("version", J::str("1")),
                ("text", J::str(SEPARATED_RULE_TEXT)),
            ]),
        ),
    ]))
}

// --- canonical measure ----------------------------------------------------

fn formatter_j() -> J {
    J::obj(vec![
        ("name", J::str("ruff format")),
        ("version", J::str("0.17.3")),
        (
            "settings",
            J::obj(vec![
                ("isolated", J::Bool(true)),
                ("line-length", J::int(100)),
                ("target-version", J::str("py313")),
                (
                    "other",
                    J::str("ruff defaults (double quotes, 4-space indent, magic trailing comma respected)"),
                ),
            ]),
        ),
    ])
}

/// Format isolated copies with the pinned ruff and return canonical counts;
/// `None` (never zeros) when the formatter is unavailable, fails, or
/// reports a version other than the pinned one.
pub fn measure_canonical(
    root: &Path,
    scope_files: &[String],
    fmt_dir: &Path,
) -> Result<Option<J>, MeasuresError> {
    let _ = fs::remove_dir_all(fmt_dir);
    for rel in scope_files {
        let dest = fmt_dir.join(rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| MeasuresError(format!("cannot create {}: {e}", parent.display())))?;
        }
        fs::copy(root.join(rel), &dest)
            .map_err(|e| MeasuresError(format!("cannot copy to {}: {e}", dest.display())))?;
    }
    let uv_ok = |output: Option<std::process::Output>| {
        output
            .filter(|o| o.status.success())
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.split_whitespace().last().map(|v| v == "0.17.3"))
            .unwrap_or(false)
    };
    let version = std::process::Command::new("uv")
        .args([
            "tool",
            "run",
            "--offline",
            "--python",
            "3.13",
            "ruff==0.17.3",
            "--version",
        ])
        .output();
    if !uv_ok(version.ok()) {
        return Ok(None);
    }
    let mut format = std::process::Command::new("uv");
    format
        .args([
            "tool",
            "run",
            "--offline",
            "--python",
            "3.13",
            "ruff==0.17.3",
        ])
        .args([
            "format",
            "--isolated",
            "--line-length",
            "100",
            "--target-version",
            "py313",
            "--no-cache",
        ])
        .args(scope_files)
        .current_dir(fmt_dir);
    if !format.output().map(|o| o.status.success()).unwrap_or(false) {
        return Ok(None);
    }
    let measured = measure_written(fmt_dir, scope_files, None)?;
    let keep = |j: &J, keys: &[&str]| -> J {
        match j {
            J::Obj(pairs) => J::obj(
                keys.iter()
                    .filter_map(|k| {
                        pairs
                            .iter()
                            .find(|(n, _)| n == k)
                            .map(|(n, v)| (n.as_str(), v.clone()))
                    })
                    .collect(),
            ),
            _ => J::Null,
        }
    };
    let files = match j_get(&measured, "files") {
        Some(J::Obj(pairs)) => J::Obj(
            pairs
                .iter()
                .map(|(f, v)| (f.clone(), keep(v, &["lines", "stmts"])))
                .collect(),
        ),
        _ => J::Null,
    };
    let total = j_get(&measured, "total")
        .map(|t| keep(t, &["lines", "stmts"]))
        .unwrap_or(J::Null);
    Ok(Some(J::obj(vec![
        ("formatter", formatter_j()),
        ("files", files),
        ("total", total),
    ])))
}

// --- edit-scope confinement ----------------------------------------------

/// SHA-256 of every compared file: root-level files keyed by name, files
/// under `dirs` keyed by relative posix path; `.pyc` and `skip_dirs` are
/// invisible to the hash walk.
fn hash_tree(
    root: &Path,
    dirs: &[String],
    skip_dirs: &BTreeSet<String>,
) -> Result<Vec<(String, String)>, MeasuresError> {
    let read_err =
        |e: std::io::Error| MeasuresError(format!("cannot read {}: {e}", root.display()));
    let digest = |path: &Path| -> Result<String, MeasuresError> {
        let bytes = fs::read(path)
            .map_err(|e| MeasuresError(format!("cannot read {}: {e}", path.display())))?;
        Ok(sha256_hex(&bytes))
    };
    let mut out: Vec<(String, String)> = Vec::new();
    for entry in fs::read_dir(root).map_err(read_err)? {
        let path = entry.map_err(|e| MeasuresError(e.to_string()))?.path();
        if path.is_file() && path.extension().and_then(|e| e.to_str()) != Some("pyc") {
            if let Some(name) = path.file_name() {
                upsert(
                    &mut out,
                    name.to_string_lossy().into_owned(),
                    digest(&path)?,
                );
            }
        }
    }
    for dir in dirs {
        let base = root.join(dir);
        if !base.is_dir() {
            continue;
        }
        let mut stack = vec![base];
        while let Some(cur) = stack.pop() {
            for entry in fs::read_dir(&cur)
                .map_err(|e| MeasuresError(format!("cannot read {}: {e}", cur.display())))?
            {
                let path = entry.map_err(|e| MeasuresError(e.to_string()))?.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                if !path.is_file() || path.extension().and_then(|e| e.to_str()) == Some("pyc") {
                    continue;
                }
                let rel = path
                    .strip_prefix(root)
                    .map(|r| r.to_string_lossy().replace('\\', "/"))
                    .unwrap_or_else(|_| path.to_string_lossy().into_owned());
                if rel.split('/').any(|part| skip_dirs.contains(part)) {
                    continue;
                }
                upsert(&mut out, rel.clone(), digest(&path)?);
            }
        }
    }
    Ok(out)
}

/// Tokens matching `[\w./\\-]*\w(ext)\b` — file-like mentions of any of the
/// `source_extensions`; longest match per start position, like the regex.
fn mention_tokens(text: &str, source_extensions: &[String]) -> Vec<String> {
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    let allowed = |c: char| is_word(c) || matches!(c, '.' | '/' | '\\' | '-');
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let mut run_end = i;
        while run_end < chars.len() && allowed(chars[run_end]) {
            run_end += 1;
        }
        let mut matched = None;
        'splits: for p in (i..run_end).rev() {
            if !is_word(chars[p]) {
                continue;
            }
            for ext in source_extensions {
                let e: Vec<char> = ext.chars().collect();
                let after = p + 1 + e.len();
                if after <= chars.len()
                    && chars[p + 1..after] == e[..]
                    && (after == chars.len() || !is_word(chars[after]))
                {
                    matched = Some((i..after, after));
                    break 'splits;
                }
            }
        }
        match matched {
            Some((range, next)) => {
                out.push(chars[range].iter().collect());
                i = next;
            }
            None => i += 1,
        }
    }
    out
}

fn j_get<'a>(v: &'a J, key: &str) -> Option<&'a J> {
    match v {
        J::Obj(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
        _ => None,
    }
}

fn j_num(v: &J, key: &str) -> Option<i64> {
    match j_get(v, key) {
        Some(J::Int(n)) => Some(*n),
        _ => None,
    }
}

/// Verify SHA-256 identity outside `scope_files` and scan mentions for
/// out-of-scope tokens.
pub fn check_edit_scope(
    original: &Path,
    candidate: &Path,
    scope_files: &[String],
    dirs: &[String],
    mentions_files: &[PathBuf],
    source_extensions: &[String],
    ignore_dirs: &[String],
) -> Result<J, MeasuresError> {
    let mut skip_dirs: BTreeSet<String> = ["__pycache__", ".pytest_cache", ".git"]
        .into_iter()
        .map(String::from)
        .collect();
    skip_dirs.extend(ignore_dirs.iter().cloned());
    let a = hash_tree(original, dirs, &skip_dirs)?;
    let b = hash_tree(candidate, dirs, &skip_dirs)?;
    let keys = |m: &[(String, String)]| m.iter().map(|(k, _)| k.clone()).collect::<BTreeSet<_>>();
    let (ka, kb) = (keys(&a), keys(&b));
    let get =
        |m: &[(String, String)], k: &str| m.iter().find(|(n, _)| n == k).map(|(_, v)| v.clone());
    let mut scope = scope_files.to_vec();
    scope.sort();
    let differing: Vec<String> = ka
        .intersection(&kb)
        .filter(|k| get(&a, k) != get(&b, k))
        .cloned()
        .collect();
    let only_original: Vec<String> = ka.difference(&kb).cloned().collect();
    let only_candidate: Vec<String> = kb.difference(&ka).cloned().collect();
    let scope_set: BTreeSet<String> = scope.iter().cloned().collect();
    let differing_set: BTreeSet<String> = differing.iter().cloned().collect();
    let common: BTreeSet<String> = ka.intersection(&kb).cloned().collect();
    let outside: Vec<String> = differing
        .iter()
        .filter(|k| !scope_set.contains(*k))
        .cloned()
        .collect();
    let edit_scope_pass =
        only_original.is_empty() && only_candidate.is_empty() && outside.is_empty();
    let scope_unchanged: Vec<String> = scope_set.difference(&differing_set).cloned().collect();
    let scope_missing: Vec<String> = scope_set.difference(&common).cloned().collect();
    let hash_pairs = |m: &[(String, String)]| -> J {
        J::Obj(
            scope
                .iter()
                .map(|k| (k.clone(), get(m, k).map(J::Str).unwrap_or(J::Null)))
                .collect(),
        )
    };
    let mut report = vec![
        ("original", J::str(original.to_string_lossy().into_owned())),
        (
            "candidate",
            J::str(candidate.to_string_lossy().into_owned()),
        ),
        (
            "dirs",
            J::Arr(dirs.iter().map(|d| J::str(d.clone())).collect()),
        ),
        (
            "skip_dirs",
            J::Arr(skip_dirs.into_iter().map(J::str).collect()),
        ),
        ("skip_suffix", J::Arr(vec![J::str(".pyc")])),
        (
            "source_extensions",
            J::Arr(
                source_extensions
                    .iter()
                    .map(|e| J::str(e.clone()))
                    .collect(),
            ),
        ),
        ("files_original", J::int(a.len() as i64)),
        ("files_candidate", J::int(b.len() as i64)),
        ("files_compared", J::int(common.len() as i64)),
        (
            "only_in_original",
            J::Arr(only_original.iter().map(|k| J::str(k.clone())).collect()),
        ),
        (
            "only_in_candidate",
            J::Arr(only_candidate.iter().map(|k| J::str(k.clone())).collect()),
        ),
        (
            "differing",
            J::Arr(differing.iter().map(|k| J::str(k.clone())).collect()),
        ),
        (
            "scope",
            J::Arr(scope.iter().map(|k| J::str(k.clone())).collect()),
        ),
        (
            "differing_outside_scope",
            J::Arr(outside.iter().map(|k| J::str(k.clone())).collect()),
        ),
        (
            "scope_unchanged",
            J::Arr(scope_unchanged.iter().map(|k| J::str(k.clone())).collect()),
        ),
        (
            "scope_missing",
            J::Arr(scope_missing.iter().map(|k| J::str(k.clone())).collect()),
        ),
        ("original_hashes", hash_pairs(&a)),
        ("candidate_hashes", hash_pairs(&b)),
        (
            "edit_scope",
            J::str(if edit_scope_pass { "pass" } else { "fail" }),
        ),
        (
            "verdict",
            J::str(if edit_scope_pass && differing == scope {
                "pass"
            } else {
                "fail"
            }),
        ),
    ];
    if !mentions_files.is_empty() {
        let allowed: BTreeSet<String> = scope_set
            .iter()
            .cloned()
            .chain(scope_set.iter().map(|s| basename(s)))
            .collect();
        let mut found: BTreeSet<String> = BTreeSet::new();
        for f in mentions_files {
            let bytes = fs::read(f)
                .map_err(|e| MeasuresError(format!("cannot read {}: {e}", f.display())))?;
            for token in mention_tokens(&String::from_utf8_lossy(&bytes), source_extensions) {
                found.insert(token);
            }
        }
        let outside_scope: Vec<String> = found
            .into_iter()
            .filter(|t| !allowed.contains(t) && !allowed.contains(&basename(t)))
            .collect();
        report.push((
            "mentions",
            J::obj(vec![
                (
                    "files",
                    J::Arr(
                        mentions_files
                            .iter()
                            .map(|f| J::str(f.to_string_lossy().into_owned()))
                            .collect(),
                    ),
                ),
                (
                    "outside_scope",
                    J::Arr(outside_scope.iter().map(|t| J::str(t.clone())).collect()),
                ),
            ]),
        ));
    }
    Ok(J::obj(report))
}

// --- gaps and composition -------------------------------------------------

/// Unmet target deltas; empty object iff every preregistered target passes.
/// Canonical targets with a `None` reading stay unmet and are marked
/// unavailable — never zero.
pub fn compute_measure_gaps(
    written: &J,
    canonical: Option<&J>,
    separated: &J,
    targets: &[(String, i64)],
) -> J {
    let written_total = j_get(written, "total");
    let separated_total = j_get(separated, "total");
    let canonical_total = canonical.and_then(|c| j_get(c, "total"));
    let totals = |source: &str| match source {
        "separated" => separated_total,
        "canonical" => canonical_total,
        _ => written_total,
    };
    let mut gaps: Vec<(String, J)> = Vec::new();
    for (key, bound) in targets {
        let Some(stem) = key.strip_suffix("_max") else {
            gaps.push((
                key.clone(),
                J::obj(vec![
                    ("target", J::int(*bound)),
                    (
                        "error",
                        J::str("unknown target kind (only *_max supported)"),
                    ),
                ]),
            ));
            continue;
        };
        // Bare metrics (stmts_max, edges_max, ...) read as written.
        let (source, metric) = match ["written", "separated", "canonical"]
            .into_iter()
            .find(|s| stem.starts_with(&format!("{s}_")))
        {
            Some(s) => (s, &stem[s.len() + 1..]),
            None => ("written", stem),
        };
        if metric.is_empty() {
            gaps.push((
                key.clone(),
                J::obj(vec![
                    ("target", J::int(*bound)),
                    ("error", J::str("unknown target source")),
                ]),
            ));
            continue;
        }
        match totals(source).and_then(|t| j_num(t, metric)) {
            None => {
                gaps.push((
                    key.clone(),
                    J::obj(vec![
                        ("target", J::int(*bound)),
                        ("actual", J::Null),
                        ("unavailable", J::Bool(true)),
                    ]),
                ));
            }
            Some(actual) if actual > *bound => {
                gaps.push((
                    key.clone(),
                    J::obj(vec![
                        ("target", J::int(*bound)),
                        ("actual", J::int(actual)),
                        ("excess", J::int(actual - bound)),
                    ]),
                ));
            }
            Some(_) => {}
        }
    }
    J::Obj(gaps)
}

/// One call returning written, separated, canonical (`None` when the
/// formatter is unavailable), and the target gaps.
pub fn measure_all(
    root: &Path,
    scope_files: &[String],
    fmt_dir: Option<&Path>,
    structure: Option<&Structure>,
    targets: &[(String, i64)],
) -> Result<J, MeasuresError> {
    let written = measure_written(root, scope_files, structure)?;
    let separated = measure_separated(root, scope_files)?;
    let canonical = match fmt_dir {
        Some(dir) => measure_canonical(root, scope_files, dir)?,
        None => None,
    };
    let gaps = compute_measure_gaps(&written, canonical.as_ref(), &separated, targets);
    Ok(J::obj(vec![
        ("measure_version", J::str(MEASURE_VERSION)),
        ("root", J::str(root.to_string_lossy().into_owned())),
        ("as_written", written),
        ("separated", separated),
        ("canonical", canonical.unwrap_or(J::Null)),
        ("gaps", gaps),
    ]))
}
