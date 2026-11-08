//! Contract-B public-surface derivation and candidate verification — the
//! Rust-substrate port of `src/lesscode_baseline/surface.py` (task 8.2, R36).
//!
//! All Python AST knowledge stays behind this seam (extractor version
//! `rust-tspython-v1`, tree-sitter-python): surface derivation, syntax
//! verification and candidate verification never execute candidate code, and
//! adding a target language means adding a sibling extractor module, not
//! rewriting callers. Report envelope field names are stable with the Python
//! stage-1 reference module; the parser differs, so syntax messages are
//! line-anchored tree-sitter reports rather than CPython messages.

use crate::receipt::J;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// A tree-sitter-python node over a string document (the language newtype's
/// document type is `StrDoc<Python>`, not `Python` itself).
type PyNode<'r> = ast_grep_core::Node<'r, ast_grep_core::tree_sitter::StrDoc<crate::Python>>;

pub const EXTRACTOR_VERSION: &str = "rust-tspython-v1";
const DEFAULT_SCAN_DIRS: [&str; 3] = ["src", "packages", "tests"];
const TEST_MARKERS: [&str; 3] = ["tests/", "/tests/", "facdrone_testing"];

#[derive(Debug)]
pub struct SurfaceError(pub String);
impl std::fmt::Display for SurfaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for SurfaceError {}

/// Map a scope file path to its dotted module name (frozen convention).
pub fn module_name(rel: &str) -> String {
    let body = rel.strip_suffix(".py").unwrap_or(rel);
    if let Some((first, rest)) = body.split_once('/') {
        if DEFAULT_SCAN_DIRS.contains(&first) && !rest.is_empty() {
            return rest.replace('/', ".");
        }
    }
    body.replace('/', ".")
}

pub(crate) fn line_starts(source: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (i, b) in source.bytes().enumerate() {
        if b == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}

pub(crate) fn line_of(starts: &[usize], byte: usize) -> usize {
    match starts.binary_search(&byte) {
        Ok(i) => i + 1,
        Err(i) => i,
    }
}

fn column_of(source: &str, starts: &[usize], byte: usize) -> usize {
    let line_start = match starts.binary_search(&byte) {
        Ok(i) => starts[i],
        Err(i) => starts[i.saturating_sub(1)],
    };
    source
        .get(line_start..byte)
        .map_or(1, |p| p.chars().count() + 1)
}

/// First ERROR/missing node in breadth-first source order with 1-based line
/// and character column. `None` means the file parsed cleanly.
fn syntax_error(source: &str, starts: &[usize]) -> Option<(usize, usize, String)> {
    let tree = crate::AstGrep::new(source, crate::Python);
    let root = tree.root();
    let root_node = root.dfs().next()?;
    let mut level = vec![root_node];
    while !level.is_empty() {
        let mut next = Vec::new();
        for node in &level {
            if node.is_error() {
                let byte = node.range().start;
                return Some((
                    line_of(starts, byte),
                    column_of(source, starts, byte),
                    "invalid syntax".into(),
                ));
            }
            if node.is_missing() {
                let byte = node.range().start;
                return Some((
                    line_of(starts, byte),
                    column_of(source, starts, byte),
                    format!("missing {}", node.kind()),
                ));
            }
            // This grammar version silently recovers `def f(:` into a
            // balanced-looking tree without ERROR/missing nodes; a parameter
            // list that opens but does not close is malformed.
            if node.kind() == "parameters" {
                let text = node.text();
                if text.starts_with('(') && !text.ends_with(')') {
                    let byte = node.range().start;
                    return Some((
                        line_of(starts, byte),
                        column_of(source, starts, byte),
                        "invalid syntax".into(),
                    ));
                }
            }
            next.extend(node.named_children());
        }
        level = next;
    }
    None
}

/// Line-anchored syntax errors for `scope_files` under `cand_root`; never
/// imports or executes candidate code. An empty result means every file
/// parsed cleanly.
pub fn check_syntax(cand_root: &Path, scope_files: &[String], language: &str) -> Vec<String> {
    if language != "python" {
        return vec![format!("check_syntax: unsupported language {language:?}")];
    }
    let mut errors = Vec::new();
    for rel in scope_files {
        match fs::read(cand_root.join(rel)) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                errors.push(format!("{rel}: file missing from candidate tree"));
            }
            Err(e) => errors.push(format!("{rel}: unreadable: {e}")),
            Ok(bytes) => match String::from_utf8(bytes) {
                Err(e) => errors.push(format!("{rel}: unreadable: {e}")),
                Ok(source) => {
                    let starts = line_starts(&source);
                    if let Some((line, column, message)) = syntax_error(&source, &starts) {
                        errors.push(format!("{rel}:{line}:{column}: {message}"));
                    }
                }
            },
        }
    }
    errors
}

/// Top-level callable/constant/re-export kinds of one scope module, mirroring
/// the Python reference: def/class (decorated definitions included), direct
/// identifier assignment targets, and `from m import x` re-exports.
fn top_level_kinds(source: &str) -> Vec<(String, String)> {
    let tree = crate::AstGrep::new(source, crate::Python);
    let root = tree.root();
    let mut out: Vec<(String, String)> = Vec::new();
    let Some(root_node) = root.dfs().next() else {
        return out;
    };
    for node in root_node.named_children() {
        // Decorated definitions contribute their inner definition; simple
        // assignments arrive wrapped in an expression_statement.
        let node = match node.kind().as_ref() {
            "decorated_definition" => match node.field("definition") {
                Some(inner) => inner,
                None => continue,
            },
            "expression_statement"
                if node.named_children().any(|c| {
                    matches!(c.kind().as_ref(), "assignment" | "annotated_assignment")
                }) =>
            {
                match node
                    .named_children()
                    .find(|c| matches!(c.kind().as_ref(), "assignment" | "annotated_assignment"))
                {
                    Some(inner) => inner,
                    None => continue,
                }
            }
            _ => node,
        };
        match node.kind().as_ref() {
            "function_definition" | "class_definition" => {
                if let Some(name) = node.field("name") {
                    let kind = if node.kind() == "class_definition" {
                        "class"
                    } else {
                        "function"
                    };
                    upsert(&mut out, name.text().into_owned(), kind.into());
                }
            }
            "assignment" | "annotated_assignment" => {
                // Only a direct identifier target declares a constant.
                if let Some(target) = node.field("left") {
                    if target.kind() == "identifier" {
                        upsert(&mut out, target.text().into_owned(), "constant".into());
                    }
                }
            }
            "import_from_statement" | "future_import_statement" => {
                let module = module_text(&node);
                let display = module.clone().unwrap_or_else(|| "None".into());
                for (_original, local) in imported_names(&node) {
                    upsert(&mut out, local, format!("reexport:{display}"));
                }
            }
            _ => {}
        }
    }
    out
}

fn upsert(map: &mut Vec<(String, String)>, key: String, value: String) {
    if let Some(entry) = map.iter_mut().find(|(k, _)| *k == key) {
        entry.1 = value;
    } else {
        map.push((key, value));
    }
}

fn omap_get<'a, V>(map: &'a [(String, V)], key: &str) -> Option<&'a V> {
    map.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

/// Absolute module of an import-from (leading relative dots stripped; `None`
/// for `from . import x`, whose Python `node.module` is None).
fn module_text(node: &PyNode) -> Option<String> {
    let text = node.field("module_name")?.text();
    let stripped = text.trim_start_matches('.');
    if stripped.is_empty() {
        None
    } else {
        Some(stripped.to_string())
    }
}

/// Imported names of an import_from_statement: (original, local) pairs; a
/// star import reports `*` for both, mirroring the Python alias model.
fn imported_names(node: &PyNode) -> Vec<(String, String)> {
    let module_range = node.field("module_name").map(|m| m.range());
    let mut out = Vec::new();
    for child in node.named_children() {
        if Some(child.range()) == module_range {
            continue;
        }
        match child.kind().as_ref() {
            "dotted_name" => {
                let name = child.text().into_owned();
                out.push((name.clone(), name));
            }
            "aliased_import" => {
                if let (Some(original), Some(local)) = (child.field("name"), child.field("alias")) {
                    out.push((original.text().into_owned(), local.text().into_owned()));
                }
            }
            "wildcard_import" => out.push(("*".into(), "*".into())),
            _ => {}
        }
    }
    out
}

/// Concatenated string content of a tree-sitter string node.
fn string_value(node: &PyNode) -> Option<String> {
    let joined: String = node
        .named_children()
        .filter(|c| c.kind() == "string_content")
        .map(|c| c.text().into_owned())
        .collect();
    Some(joined).filter(|s| !s.is_empty() || node.text().len() <= 2)
}

/// A use of a scope module discovered by scanning, in Python-reference
/// order: all import findings first, then attribute reads and setattr
/// patch targets.
enum ScanEvent {
    /// Symbol import from a scope module (via "import").
    Import(String, String),
    /// Whole-module import marker (name `<module>`, via "module").
    ModuleMarker(String),
    /// Attribute read on an aliased module name (via "attribute").
    Attribute(String, String),
    /// setattr patch target on a scope module (via "patched").
    Patched(String, String),
}

/// Pass 1: import findings plus the file-local alias map. Only modules in
/// `modules` produce events, mirroring the Python `in modules` branches.
fn scan_imports(
    source: &str,
    modules: &BTreeSet<String>,
) -> (Vec<ScanEvent>, Vec<(String, String)>) {
    let tree = crate::AstGrep::new(source, crate::Python);
    let root = tree.root();
    let mut events = Vec::new();
    let mut alias: Vec<(String, String)> = Vec::new();
    let Some(root_node) = root.dfs().next() else {
        return (events, alias);
    };
    let mut level: Vec<_> = root_node.named_children().collect();
    while !level.is_empty() {
        let mut next = Vec::new();
        for node in &level {
            next.extend(node.named_children());
            match node.kind().as_ref() {
                "import_from_statement" | "future_import_statement" => {
                    let module = module_text(node);
                    for (original, local) in imported_names(node) {
                        if let Some(m) = module.as_deref().filter(|m| modules.contains(*m)) {
                            events.push(ScanEvent::Import(m.to_string(), original));
                        } else if let (Some(m), false) = (module.as_deref(), original == "*") {
                            // Dotted candidate: `from pkg import mod` imports pkg.mod.
                            let candidate = format!("{m}.{original}");
                            if modules.contains(&candidate) {
                                events.push(ScanEvent::ModuleMarker(candidate.clone()));
                                upsert(&mut alias, local, candidate);
                            }
                        }
                    }
                }
                "import_statement" => {
                    for child in node.named_children() {
                        let (candidate, local) = match child.kind().as_ref() {
                            "dotted_name" => {
                                let text = child.text().into_owned();
                                (text.clone(), text)
                            }
                            "aliased_import" => match (child.field("name"), child.field("alias")) {
                                (Some(name), Some(as_name)) => {
                                    (name.text().into_owned(), as_name.text().into_owned())
                                }
                                _ => continue,
                            },
                            _ => continue,
                        };
                        if modules.contains(&candidate) {
                            events.push(ScanEvent::ModuleMarker(candidate.clone()));
                            upsert(&mut alias, local, candidate);
                        }
                    }
                }
                _ => {}
            }
        }
        level = next;
    }
    (events, alias)
}

/// Pass 2: attribute reads on aliased module names and setattr patch targets.
fn scan_uses(
    source: &str,
    alias: &[(String, String)],
    modules: &BTreeSet<String>,
) -> Vec<ScanEvent> {
    let tree = crate::AstGrep::new(source, crate::Python);
    let root = tree.root();
    let mut events = Vec::new();
    let Some(root_node) = root.dfs().next() else {
        return events;
    };
    let mut level: Vec<_> = root_node.named_children().collect();
    while !level.is_empty() {
        let mut next = Vec::new();
        for node in &level {
            next.extend(node.named_children());
            match node.kind().as_ref() {
                "attribute" => {
                    if let (Some(obj), Some(attr)) = (node.field("object"), node.field("attribute"))
                    {
                        if obj.kind() == "identifier" {
                            if let Some(target) = omap_get(alias, &obj.text()) {
                                events.push(ScanEvent::Attribute(
                                    target.clone(),
                                    attr.text().into_owned(),
                                ));
                            }
                        }
                    }
                }
                "call" => {
                    let is_setattr = node.field("function").is_some_and(|f| {
                        (f.kind() == "identifier" && f.text() == "setattr")
                            || (f.kind() == "attribute"
                                && f.field("attribute").is_some_and(|a| a.text() == "setattr"))
                    });
                    if !is_setattr {
                        continue;
                    }
                    let positional: Vec<_> = node
                        .field("arguments")
                        .map(|args| {
                            args.named_children()
                                .filter(|a| a.kind() != "keyword_argument")
                                .collect()
                        })
                        .unwrap_or_default();
                    if positional.len() <= 1 {
                        continue;
                    }
                    let (first, second) = (&positional[0], &positional[1]);
                    if first.kind() == "identifier" && second.kind() == "string" {
                        if let (Some(target), Some(name)) =
                            (omap_get(alias, &first.text()), string_value(second))
                        {
                            events.push(ScanEvent::Patched(target.clone(), name));
                        }
                    } else if first.kind() == "string" {
                        if let Some(dotted) = string_value(first) {
                            if let Some((head, _, tail)) =
                                dotted.rsplit_once('.').map(|(h, t)| (h, ".", t))
                            {
                                if modules.contains(head) {
                                    events.push(ScanEvent::Patched(
                                        head.to_string(),
                                        tail.to_string(),
                                    ));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        level = next;
    }
    events
}

struct NameEntry {
    kind: String,
    tags: Vec<String>,
    via: Vec<String>,
    importers: Vec<String>,
}

/// Insertion-ordered symbol → observing-files stratum (unresolved/intra-set).
type NameFiles = Vec<(String, Vec<(String, Vec<String>)>)>;

struct ModuleReport {
    file: String,
    names: Vec<(String, NameEntry)>,
    importers: Vec<(String, String)>,
}

/// Insertion-ordered report parts shared by the scan passes; converted to a
/// stable `J` envelope at the end.
struct SurfaceParts {
    modules: Vec<(String, ModuleReport)>,
    unresolved: NameFiles,
    intra_set: NameFiles,
}

impl SurfaceParts {
    fn new(scope_files: &[String]) -> Self {
        let mut modules = Vec::new();
        let mut unresolved = Vec::new();
        let mut intra_set = Vec::new();
        for rel in scope_files {
            modules.push((
                module_name(rel),
                ModuleReport {
                    file: rel.clone(),
                    names: Vec::new(),
                    importers: Vec::new(),
                },
            ));
            unresolved.push((module_name(rel), Vec::new()));
            intra_set.push((module_name(rel), Vec::new()));
        }
        SurfaceParts {
            modules,
            unresolved,
            intra_set,
        }
    }
}

/// One scanning file's view: who is importing and under which tag.
struct UseContext<'a> {
    kind_tables: &'a [(String, Vec<(String, String)>)],
    scope_set: &'a BTreeSet<String>,
    rel: &'a str,
    tag: &'a str,
}

impl UseContext<'_> {
    /// The Python `make_add` closure: one observed use of a scope-module
    /// symbol from this file.
    fn add_use(&self, parts: &mut SurfaceParts, module: &str, name: &str, via: &str) {
        let resolved: Option<String> = if name == "<module>" {
            Some("module".into())
        } else {
            match omap_get(self.kind_tables, module) {
                Some(table) => omap_get(table, name).cloned(),
                None => None,
            }
        };
        if self.scope_set.contains(self.rel) {
            if let Some(entries) = parts.intra_set.iter_mut().find(|(m, _)| m == module) {
                if let Some(list) = entries.1.iter_mut().find(|(n, _)| n == name) {
                    list.1.push(self.rel.to_string());
                } else {
                    entries
                        .1
                        .push((name.to_string(), vec![self.rel.to_string()]));
                }
            }
            return;
        }
        if resolved.is_none() && via == "attribute" {
            if let Some(entries) = parts.unresolved.iter_mut().find(|(m, _)| m == module) {
                if let Some(list) = entries.1.iter_mut().find(|(n, _)| n == name) {
                    if !list.1.iter().any(|r| r == self.rel) {
                        list.1.push(self.rel.to_string());
                    }
                } else {
                    entries
                        .1
                        .push((name.to_string(), vec![self.rel.to_string()]));
                }
            }
            return;
        }
        let kind = resolved.unwrap_or_else(|| "missing".into());
        if let Some((_, info)) = parts.modules.iter_mut().find(|(m, _)| m == module) {
            let entry = match info.names.iter_mut().find(|(n, _)| n == name) {
                Some((_, entry)) => entry,
                None => {
                    info.names.push((
                        name.to_string(),
                        NameEntry {
                            kind: kind.clone(),
                            tags: Vec::new(),
                            via: Vec::new(),
                            importers: Vec::new(),
                        },
                    ));
                    &mut info.names.last_mut().expect("just pushed").1
                }
            };
            if !entry.tags.iter().any(|t| t == self.tag) {
                entry.tags.push(self.tag.to_string());
            }
            if !entry.via.iter().any(|v| v == via) {
                entry.via.push(via.to_string());
            }
            if !entry.importers.iter().any(|i| i == self.rel) {
                entry.importers.push(self.rel.to_string());
            }
            upsert(
                &mut info.importers,
                self.rel.to_string(),
                self.tag.to_string(),
            );
        }
    }
}

pub(crate) fn read_source(path: &Path) -> Result<String, SurfaceError> {
    let bytes =
        fs::read(path).map_err(|e| SurfaceError(format!("cannot read {}: {e}", path.display())))?;
    String::from_utf8(bytes)
        .map_err(|e| SurfaceError(format!("invalid UTF-8 in {}: {e}", path.display())))
}

/// Recursively collect `*.py` paths under one scan directory, sorted by full
/// path string like the Python `sorted(rglob(...))` convention.
fn scan_dir_files(root: &Path, dir: &str) -> Result<Vec<String>, SurfaceError> {
    let start = root.join(dir);
    if !start.is_dir() {
        // Missing scan directories contribute no files (Python rglob yields
        // nothing for a missing root).
        return Ok(Vec::new());
    }
    let mut stack = vec![start];
    let mut found = Vec::new();
    while let Some(current) = stack.pop() {
        let entries = fs::read_dir(&current)
            .map_err(|e| SurfaceError(format!("cannot read {}: {e}", current.display())))?;
        for entry in entries {
            let entry = entry.map_err(|e| SurfaceError(e.to_string()))?;
            let file_type = entry.file_type().map_err(|e| SurfaceError(e.to_string()))?;
            if file_type.is_dir() {
                stack.push(entry.path());
            } else if entry.file_name().to_string_lossy().ends_with(".py") {
                found.push(entry.path());
            }
        }
    }
    let base = root.to_path_buf();
    let mut rels: Vec<String> = found
        .into_iter()
        .map(|p| {
            p.strip_prefix(&base)
                .map(|r| r.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| p.to_string_lossy().into_owned())
        })
        .collect();
    rels.sort();
    Ok(rels)
}

/// Derive the Contract-B public-surface report for `scope_files` under
/// `root`. Symbols imported or used only by the recorder oracle are part of
/// the contract (tagged `recorder`); attribute reads of names a scope module
/// lacks are recorded under `unresolved` — never silently absent.
pub fn derive_surface(
    root: &Path,
    scope_files: &[String],
    recorder_files: &[String],
    scan_dirs: &[String],
) -> Result<J, SurfaceError> {
    let scope: Vec<String> = scope_files.to_vec();
    let modules: BTreeSet<String> = scope.iter().map(|rel| module_name(rel)).collect();
    let scope_set: BTreeSet<String> = scope.iter().cloned().collect();
    let mut kind_tables: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for rel in &scope {
        let source = read_source(&root.join(rel))?;
        kind_tables.push((module_name(rel), top_level_kinds(&source)));
    }
    let dirs: Vec<String> = if scan_dirs.is_empty() {
        DEFAULT_SCAN_DIRS.iter().map(|d| d.to_string()).collect()
    } else {
        scan_dirs.to_vec()
    };
    let mut files = Vec::new();
    for dir in &dirs {
        files.extend(scan_dir_files(root, dir)?);
    }
    let mut recorders: Vec<String> = recorder_files.to_vec();
    recorders.sort();
    recorders.dedup();

    let mut parts = SurfaceParts::new(&scope);
    for rel in files.iter().chain(recorders.iter()) {
        let source = read_source(&root.join(rel))?;
        let tag = if recorders.iter().any(|r| r == rel) {
            "recorder"
        } else if TEST_MARKERS.iter().any(|m| rel.contains(m)) {
            "test"
        } else {
            "prod"
        };
        let context = UseContext {
            kind_tables: &kind_tables,
            scope_set: &scope_set,
            rel,
            tag,
        };
        let (import_events, alias) = scan_imports(&source, &modules);
        for event in import_events {
            let (module, name, via) = match event {
                ScanEvent::Import(m, n) => (m, n, "import"),
                ScanEvent::ModuleMarker(m) => (m, "<module>".into(), "module"),
                _ => continue,
            };
            context.add_use(&mut parts, &module, &name, via);
        }
        for event in scan_uses(&source, &alias, &modules) {
            let (module, name, via) = match event {
                ScanEvent::Attribute(m, n) => (m, n, "attribute"),
                ScanEvent::Patched(m, n) => (m, n, "patched"),
                _ => continue,
            };
            context.add_use(&mut parts, &module, &name, via);
        }
    }

    let report = J::obj(vec![
        ("extractor_version", J::str(EXTRACTOR_VERSION)),
        ("root", J::str(root.to_string_lossy().into_owned())),
        (
            "scope_files",
            J::arr(scope.iter().map(|f| J::str(f.clone())).collect()),
        ),
        (
            "recorder_files",
            J::arr(recorders.iter().map(|f| J::str(f.clone())).collect()),
        ),
        ("files_scanned", J::int(files.len() as i64)),
        (
            "modules",
            J::Obj(
                parts
                    .modules
                    .iter()
                    .map(|(m, info)| {
                        (
                            m.clone(),
                            J::obj(vec![
                                ("file", J::str(info.file.clone())),
                                (
                                    "names",
                                    J::Obj(
                                        info.names
                                            .iter()
                                            .map(|(name, entry)| {
                                                (
                                                    name.clone(),
                                                    J::obj(vec![
                                                        ("kind", J::str(entry.kind.clone())),
                                                        (
                                                            "tags",
                                                            J::arr(
                                                                entry
                                                                    .tags
                                                                    .iter()
                                                                    .map(|t| J::str(t.clone()))
                                                                    .collect(),
                                                            ),
                                                        ),
                                                        (
                                                            "via",
                                                            J::arr(
                                                                entry
                                                                    .via
                                                                    .iter()
                                                                    .map(|v| J::str(v.clone()))
                                                                    .collect(),
                                                            ),
                                                        ),
                                                        (
                                                            "importers",
                                                            J::arr(
                                                                entry
                                                                    .importers
                                                                    .iter()
                                                                    .map(|i| J::str(i.clone()))
                                                                    .collect(),
                                                            ),
                                                        ),
                                                    ]),
                                                )
                                            })
                                            .collect(),
                                    ),
                                ),
                                (
                                    "importers",
                                    J::Obj(
                                        info.importers
                                            .iter()
                                            .map(|(r, t)| (r.clone(), J::str(t.clone())))
                                            .collect(),
                                    ),
                                ),
                            ]),
                        )
                    })
                    .collect(),
            ),
        ),
        (
            "unresolved",
            J::Obj(
                parts
                    .unresolved
                    .iter()
                    .map(|(m, entries)| {
                        (
                            m.clone(),
                            J::Obj(
                                entries
                                    .iter()
                                    .map(|(n, rels)| {
                                        (
                                            n.clone(),
                                            J::arr(
                                                rels.iter().map(|r| J::str(r.clone())).collect(),
                                            ),
                                        )
                                    })
                                    .collect(),
                            ),
                        )
                    })
                    .collect(),
            ),
        ),
        (
            "intra_set",
            J::Obj(
                parts
                    .intra_set
                    .iter()
                    .map(|(m, entries)| {
                        (
                            m.clone(),
                            J::Obj(
                                entries
                                    .iter()
                                    .map(|(n, rels)| {
                                        (
                                            n.clone(),
                                            J::arr(
                                                rels.iter().map(|r| J::str(r.clone())).collect(),
                                            ),
                                        )
                                    })
                                    .collect(),
                            ),
                        )
                    })
                    .collect(),
            ),
        ),
    ]);
    Ok(report)
}

// ---------------------------------------------------------------------------
// CLI (task 8.2): the seam is drivable exactly where the Python module was.

use clap::Subcommand;
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub enum SurfaceCommand {
    /// Derive the Contract-B public-surface report for the scope files.
    Derive {
        #[arg(long)]
        root: PathBuf,
        #[arg(long = "scope-file")]
        scope: Vec<String>,
        #[arg(long = "recorder-file")]
        recorder: Vec<String>,
        #[arg(long = "scan-dir")]
        scan_dirs: Vec<String>,
    },
    /// Line-anchored syntax check for candidate scope files.
    CheckSyntax {
        #[arg(long)]
        root: PathBuf,
        #[arg(long = "scope-file")]
        scope: Vec<String>,
        #[arg(long, default_value = "python")]
        language: String,
    },
    /// Verify a candidate tree against a surface report JSON file.
    Verify {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        report: PathBuf,
        #[arg(long = "extra-patched")]
        extra_patched: Vec<String>,
    },
}

/// serde_json value to `J`. Object keys arrive sorted (serde_json's map is a
/// BTreeMap), so verification findings iterate in sorted-key order when a
/// report is loaded from a file; `derive_surface` output keeps insertion
/// order. Verification semantics are order-insensitive.
fn value_to_j(value: &serde_json::Value) -> J {
    match value {
        serde_json::Value::Null => J::Null,
        serde_json::Value::Bool(b) => J::Bool(*b),
        serde_json::Value::Number(n) => J::int(n.as_i64().unwrap_or(0)),
        serde_json::Value::String(s) => J::str(s.clone()),
        serde_json::Value::Array(items) => J::arr(items.iter().map(value_to_j).collect()),
        serde_json::Value::Object(map) => J::Obj(
            map.iter()
                .map(|(k, v)| (k.clone(), value_to_j(v)))
                .collect(),
        ),
    }
}

pub fn cli_run(cmd: SurfaceCommand) -> Result<u8, SurfaceError> {
    match cmd {
        SurfaceCommand::Derive {
            root,
            scope,
            recorder,
            scan_dirs,
        } => {
            let report = derive_surface(&root, &scope, &recorder, &scan_dirs)?;
            println!("{}", crate::receipt::dumps(&report));
            Ok(0)
        }
        SurfaceCommand::CheckSyntax {
            root,
            scope,
            language,
        } => {
            let errors = check_syntax(&root, &scope, &language);
            if errors.is_empty() {
                return Ok(0);
            }
            println!(
                "{}",
                crate::receipt::dumps(&J::arr(errors.iter().map(|e| J::str(e.clone())).collect()))
            );
            Ok(1)
        }
        SurfaceCommand::Verify {
            root,
            report,
            extra_patched,
        } => {
            let bytes = fs::read(&report)
                .map_err(|e| SurfaceError(format!("cannot read {}: {e}", report.display())))?;
            let parsed: serde_json::Value = serde_json::from_slice(&bytes)
                .map_err(|e| SurfaceError(format!("invalid JSON in {}: {e}", report.display())))?;
            let verification =
                verify_candidate_surface(&root, &value_to_j(&parsed), &extra_patched);
            println!("{}", crate::receipt::dumps(&verification));
            Ok(
                if matches!(j_get(&verification, "passed"), Some(J::Bool(true))) {
                    0
                } else {
                    1
                },
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Candidate verification

fn j_get<'a>(value: &'a J, key: &str) -> Option<&'a J> {
    match value {
        J::Obj(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
        _ => None,
    }
}

fn j_str(value: &J) -> Option<&str> {
    match value {
        J::Str(s) => Some(s),
        _ => None,
    }
}

fn j_pairs(value: &J) -> &[(String, J)] {
    match value {
        J::Obj(pairs) => pairs,
        _ => &[],
    }
}

fn j_items(value: &J) -> &[J] {
    match value {
        J::Arr(items) => items,
        _ => &[],
    }
}

/// Functions that shadow patched module globals locally (AST only): findings
/// carry name, how (parameter/default/assignment), function and line.
fn find_local_rebindings(
    source: &str,
    starts: &[usize],
    patched: &BTreeSet<String>,
) -> Vec<(String, String, String, usize)> {
    let tree = crate::AstGrep::new(source, crate::Python);
    let root = tree.root();
    let mut findings = Vec::new();
    let Some(root_node) = root.dfs().next() else {
        return findings;
    };
    let mut level: Vec<_> = root_node.named_children().collect();
    while !level.is_empty() {
        let mut next = Vec::new();
        for node in &level {
            next.extend(node.named_children());
            if node.kind() != "function_definition" {
                continue;
            }
            let function = node
                .field("name")
                .map(|n| n.text().into_owned())
                .unwrap_or_default();
            let line = line_of(starts, node.range().start);
            let mut default_names: Vec<String> = Vec::new();
            if let Some(params) = node.field("parameters") {
                for param in params.named_children() {
                    let name_node = match param.kind().as_ref() {
                        // *args/**kwargs are list_splat/dictionary_splat and
                        // are not part of the checked parameter classes.
                        "identifier" => Some(param.clone()),
                        "default_parameter" | "typed_parameter" | "typed_default_parameter" => {
                            param.field("name")
                        }
                        _ => None,
                    };
                    if let Some(name) = name_node {
                        if patched.contains(name.text().as_ref()) {
                            findings.push((
                                name.text().into_owned(),
                                "parameter".into(),
                                function.clone(),
                                line,
                            ));
                        }
                    }
                    // Name-valued defaults capture the module global.
                    if matches!(
                        param.kind().as_ref(),
                        "default_parameter" | "typed_default_parameter"
                    ) {
                        if let Some(value) = param.field("value") {
                            if value.kind() == "identifier"
                                && patched.contains(value.text().as_ref())
                            {
                                default_names.push(value.text().into_owned());
                            }
                        }
                    }
                }
            }
            default_names.sort();
            default_names.dedup();
            for name in default_names {
                findings.push((name, "default".into(), function.clone(), line));
            }
            // Assignments anywhere inside the function (nested functions
            // included) attribute to the enclosing function, like the
            // Python reference walk.
            let mut inner: Vec<_> = node.named_children().collect();
            while !inner.is_empty() {
                let mut inner_next = Vec::new();
                for stmt in &inner {
                    inner_next.extend(stmt.named_children());
                    if !matches!(
                        stmt.kind().as_ref(),
                        "assignment" | "annotated_assignment" | "augmented_assignment"
                    ) {
                        continue;
                    }
                    if let Some(target) = stmt.field("left") {
                        let mut bound: Vec<String> = Vec::new();
                        collect_target_names(&target, &mut bound);
                        bound.sort();
                        bound.dedup();
                        for name in bound {
                            if patched.contains(&name) {
                                findings.push((name, "assignment".into(), function.clone(), line));
                            }
                        }
                    }
                }
                inner = inner_next;
            }
        }
        level = next;
    }
    findings
}

/// All identifiers bound by an assignment target subtree, excluding the
/// attribute-name child of attribute targets (Python's `attr` is a plain
/// string, not a walked node).
fn collect_target_names(node: &PyNode, bound: &mut Vec<String>) {
    if node.kind() == "identifier" {
        bound.push(node.text().into_owned());
        return;
    }
    let skip_attr_name = node.kind() == "attribute";
    for child in node.named_children() {
        if skip_attr_name
            && child.range()
                == node
                    .field("attribute")
                    .map(|a| a.range())
                    .unwrap_or_default()
        {
            continue;
        }
        collect_target_names(&child, bound);
    }
}

/// Verify a candidate tree against the Contract-B surface report: missing
/// symbols, kind mismatches and local rebinding of `patched` module globals.
/// Never imports or executes candidate code.
pub fn verify_candidate_surface(cand_root: &Path, report: &J, extra_patched: &[String]) -> J {
    let mut missing_symbols: Vec<J> = Vec::new();
    let mut kind_mismatches: Vec<J> = Vec::new();
    let mut local_rebindings: Vec<J> = Vec::new();
    for (module, info) in j_pairs(j_get(report, "modules").unwrap_or(&J::Null)) {
        let rel = j_get(info, "file")
            .and_then(j_str)
            .unwrap_or_default()
            .to_string();
        let path = cand_root.join(&rel);
        let source = match read_source(&path) {
            Ok(source) => source,
            Err(_) => {
                missing_symbols.push(J::arr(vec![J::str(module.clone()), J::str("<module>")]));
                continue;
            }
        };
        let kinds = top_level_kinds(&source);
        let names = j_pairs(j_get(info, "names").unwrap_or(&J::Null));
        for (name, entry) in names {
            if name == "<module>" {
                continue; // module-import marker, not a definable symbol
            }
            let expected = j_get(entry, "kind").and_then(j_str).unwrap_or("missing");
            match omap_get(&kinds, name) {
                None => {
                    if expected != "missing" {
                        missing_symbols
                            .push(J::arr(vec![J::str(module.clone()), J::str(name.clone())]));
                    }
                }
                Some(actual) => {
                    if actual != expected {
                        kind_mismatches.push(J::obj(vec![
                            ("module", J::str(module.clone())),
                            ("name", J::str(name.clone())),
                            ("expected", J::str(expected)),
                            ("actual", J::str(actual.clone())),
                        ]));
                    }
                }
            }
        }
        let mut patched: BTreeSet<String> = names
            .iter()
            .filter(|(_, entry)| {
                j_get(entry, "via")
                    .map(|v| j_items(v).iter().any(|x| j_str(x) == Some("patched")))
                    .unwrap_or(false)
            })
            .map(|(name, _)| name.clone())
            .collect();
        patched.extend(extra_patched.iter().cloned());
        if !patched.is_empty() {
            let starts = line_starts(&source);
            for (name, how, function, line) in find_local_rebindings(&source, &starts, &patched) {
                local_rebindings.push(J::obj(vec![
                    ("name", J::str(name)),
                    ("how", J::str(how)),
                    ("function", J::str(function)),
                    ("line", J::int(line as i64)),
                ]));
            }
        }
    }
    let passed =
        missing_symbols.is_empty() && kind_mismatches.is_empty() && local_rebindings.is_empty();
    J::obj(vec![
        ("passed", J::Bool(passed)),
        ("missing_symbols", J::arr(missing_symbols)),
        ("kind_mismatches", J::arr(kind_mismatches)),
        ("local_rebindings", J::arr(local_rebindings)),
    ])
}
