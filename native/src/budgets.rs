//! Size governance for the native sources: the `rust-syntax-units-v1` metric.
//!
//! Enforces the ceilings frozen in `complexity-budgets.json` (design.md,
//! Production Size Governance): a `.rs` file under `root` absent from the
//! config fails the run (`unlisted`); brownfield files keep their measured
//! baseline ceiling via a recorded `disposition` and report `disposed`.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::admission::{write_receipt, J};

pub const SCHEMA: &str = "rust-syntax-units-v1";
pub const DEFAULT_MAX_PHYSICAL_LINES: usize = 400;
pub const DEFAULT_MAX_LOGICAL_UNITS: usize = 300;
pub const DEFAULT_MAX_FN_LINES: usize = 80;

/// One governed file; omitted ceilings fall back to the metric defaults.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileBudget {
    pub path: String,
    pub max_physical_lines: Option<usize>,
    pub max_logical_units: Option<usize>,
    pub max_fn_lines: Option<usize>,
    pub owner: String,
    pub allocation: String,
    pub disposition: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetsConfig {
    pub files: Vec<FileBudget>,
}

/// Per-file measurement plus the verdict against its ceilings.
#[derive(Default)]
pub struct FileReport {
    pub path: String,
    pub present: bool,
    pub total_lines: usize,
    pub production_lines: usize,
    pub test_lines: usize,
    pub logical_units: usize,
    pub longest_fn_lines: usize,
    pub longest_fn_name: String,
    pub limit: [usize; 3],
    pub owner: String,
    pub allocation: String,
    pub disposition: Option<String>,
}

impl FileReport {
    fn exceeded(&self) -> bool {
        self.present
            && (self.production_lines > self.limit[0]
                || self.logical_units > self.limit[1]
                || self.longest_fn_lines > self.limit[2])
    }

    fn status(&self) -> &'static str {
        if !self.present {
            "missing"
        } else if self.exceeded() {
            "exceeded"
        } else if self.disposition.is_some() {
            "disposed"
        } else {
            "within_budget"
        }
    }
}

/// Counts logical units (one per counted item declaration plus one per
/// statement inside non-test function bodies), the longest non-test function
/// and the line count of `#[cfg(test)]` subtrees.
struct Counter {
    units: usize,
    longest: usize,
    longest_name: String,
    test_depth: usize,
    test_lines: usize,
}

fn is_cfg_test(a: &syn::Attribute) -> bool {
    matches!(&a.meta, syn::Meta::List(l)
        if l.path.is_ident("cfg") && l.tokens.to_string() == "test")
}

fn item_attrs(i: &syn::Item) -> &[syn::Attribute] {
    match i {
        syn::Item::Fn(f) => &f.attrs,
        syn::Item::Impl(f) => &f.attrs,
        syn::Item::Mod(f) => &f.attrs,
        syn::Item::Macro(f) => &f.attrs,
        _ => &[],
    }
}

fn is_counted_item(i: &syn::Item) -> bool {
    matches!(
        i,
        syn::Item::Struct(_)
            | syn::Item::Enum(_)
            | syn::Item::Type(_)
            | syn::Item::Const(_)
            | syn::Item::Static(_)
            | syn::Item::Trait(_)
            | syn::Item::Impl(_)
            | syn::Item::Mod(_)
            | syn::Item::Use(_)
            | syn::Item::Macro(_)
    )
}

macro_rules! visit_function {
    ($method:ident, $ty:ty, $walk:ident) => {
        fn $method(&mut self, i: &'ast $ty) {
            let span = i.span();
            if self.enter(&i.attrs, span) {
                self.units += 1;
                let lines = span.end().line - span.start().line + 1;
                if lines > self.longest {
                    self.longest = lines;
                    self.longest_name = i.sig.ident.to_string();
                }
            }
            syn::visit::$walk(self, i);
            self.exit(&i.attrs);
        }
    };
}

impl Counter {
    fn enter(&mut self, attrs: &[syn::Attribute], span: proc_macro2::Span) -> bool {
        if attrs.iter().any(is_cfg_test) {
            self.test_depth += 1;
            self.test_lines += span.end().line - span.start().line + 1;
            return false;
        }
        self.test_depth == 0
    }

    fn exit(&mut self, attrs: &[syn::Attribute]) {
        if attrs.iter().any(is_cfg_test) {
            self.test_depth -= 1;
        }
    }
}

impl<'ast> Visit<'ast> for Counter {
    visit_function!(visit_item_fn, syn::ItemFn, visit_item_fn);
    visit_function!(visit_impl_item_fn, syn::ImplItemFn, visit_impl_item_fn);
    visit_function!(visit_trait_item_fn, syn::TraitItemFn, visit_trait_item_fn);

    fn visit_item(&mut self, i: &'ast syn::Item) {
        let attrs = item_attrs(i);
        let production = self.enter(attrs, i.span());
        if production && is_counted_item(i) {
            self.units += 1;
        }
        syn::visit::visit_item(self, i);
        self.exit(attrs);
    }

    fn visit_stmt(&mut self, s: &'ast syn::Stmt) {
        if self.test_depth == 0 {
            self.units += 1;
        }
        syn::visit::visit_stmt(self, s);
    }
}

fn limits(b: &FileBudget) -> [usize; 3] {
    [
        b.max_physical_lines.unwrap_or(DEFAULT_MAX_PHYSICAL_LINES),
        b.max_logical_units.unwrap_or(DEFAULT_MAX_LOGICAL_UNITS),
        b.max_fn_lines.unwrap_or(DEFAULT_MAX_FN_LINES),
    ]
}

fn new_report(b: &FileBudget) -> FileReport {
    FileReport {
        path: b.path.clone(),
        limit: limits(b),
        owner: b.owner.clone(),
        allocation: b.allocation.clone(),
        disposition: b.disposition.clone(),
        ..FileReport::default()
    }
}

fn measure(path: &Path, b: &FileBudget) -> Result<FileReport, String> {
    let text =
        fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let file =
        syn::parse_file(&text).map_err(|e| format!("cannot parse {}: {e}", path.display()))?;
    let mut c = Counter {
        units: 0,
        longest: 0,
        longest_name: String::new(),
        test_depth: 0,
        test_lines: 0,
    };
    c.visit_file(&file);
    let mut r = new_report(b);
    r.present = true;
    r.total_lines = text.lines().count();
    r.test_lines = c.test_lines;
    r.production_lines = r.total_lines.saturating_sub(c.test_lines);
    r.logical_units = c.units;
    r.longest_fn_lines = c.longest;
    r.longest_fn_name = c.longest_name;
    Ok(r)
}

fn discover_rs(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in
            fs::read_dir(&dir).map_err(|e| format!("cannot list {}: {e}", dir.display()))?
        {
            let path = entry
                .map_err(|e| format!("cannot list {}: {e}", dir.display()))?
                .path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                found.push(path);
            }
        }
    }
    found.sort();
    Ok(found)
}

/// Exit 1 on any exceeded, unlisted or missing file.
pub struct CheckResult {
    pub reports: Vec<FileReport>,
    pub unlisted: Vec<String>,
    pub exit_code: u8,
}

pub fn check(root: &Path, config: &BudgetsConfig) -> Result<CheckResult, String> {
    let listed: Vec<String> = config.files.iter().map(|f| f.path.clone()).collect();
    let mut unlisted = Vec::new();
    for path in discover_rs(root)? {
        let rel = path
            .strip_prefix(root)
            .map_err(|e| format!("{} escapes {}: {e}", path.display(), root.display()))?
            .to_string_lossy()
            .replace('\\', "/");
        if !listed.contains(&rel) {
            unlisted.push(rel);
        }
    }
    let mut reports = Vec::new();
    for b in &config.files {
        let path = root.join(b.path.replace('\\', "/"));
        reports.push(if path.exists() {
            measure(&path, b)?
        } else {
            new_report(b)
        });
    }
    let failed = !unlisted.is_empty()
        || reports
            .iter()
            .any(|r| matches!(r.status(), "exceeded" | "missing"));
    Ok(CheckResult {
        reports,
        unlisted,
        exit_code: u8::from(failed),
    })
}

fn n(v: usize) -> J {
    J::int(v as i64)
}

fn file_j(r: &FileReport) -> J {
    J::obj(vec![
        ("path", J::str(r.path.clone())),
        ("present", J::Bool(r.present)),
        ("total_lines", n(r.total_lines)),
        ("production_lines", n(r.production_lines)),
        ("test_lines", n(r.test_lines)),
        ("logical_units", n(r.logical_units)),
        ("longest_fn_lines", n(r.longest_fn_lines)),
        ("longest_fn_name", J::str(r.longest_fn_name.clone())),
        ("max_physical_lines", n(r.limit[0])),
        ("max_logical_units", n(r.limit[1])),
        ("max_fn_lines", n(r.limit[2])),
        ("owner", J::str(r.owner.clone())),
        ("allocation", J::str(r.allocation.clone())),
        (
            "disposition",
            r.disposition.clone().map(J::str).unwrap_or(J::Null),
        ),
        ("status", J::str(r.status())),
    ])
}

fn count<'a>(reports: impl Iterator<Item = &'a FileReport>, status: &str) -> J {
    n(reports.filter(|r| r.status() == status).count())
}

fn receipt(root: &Path, config: &Path, result: &CheckResult) -> J {
    let reports = result.reports.iter();
    J::obj(vec![
        ("receipt", J::str("budgets")),
        ("schema", J::str(SCHEMA)),
        ("root", J::str(root.to_string_lossy().replace('\\', "/"))),
        (
            "config",
            J::str(config.to_string_lossy().replace('\\', "/")),
        ),
        ("files", J::Arr(result.reports.iter().map(file_j).collect())),
        (
            "unlisted",
            J::Arr(result.unlisted.iter().map(|p| J::str(p.clone())).collect()),
        ),
        (
            "summary",
            J::obj(vec![
                ("total", n(result.reports.len() + result.unlisted.len())),
                ("within_budget", count(reports.clone(), "within_budget")),
                ("disposed", count(reports.clone(), "disposed")),
                ("exceeded", count(reports.clone(), "exceeded")),
                ("unlisted", n(result.unlisted.len())),
                ("missing", count(reports, "missing")),
            ]),
        ),
    ])
}

/// Exit 0 when every discovered file is listed and within its ceilings; 1 on
/// any exceeded, unlisted or missing file; errors surface as `Err` (exit 3).
pub fn cli_run(root: &Path, config: &Path, out: &Path) -> Result<u8, String> {
    let text =
        fs::read_to_string(config).map_err(|e| format!("cannot read {}: {e}", config.display()))?;
    let parsed: BudgetsConfig = serde_json::from_str(&text)
        .map_err(|e| format!("invalid budgets config in {}: {e}", config.display()))?;
    let result = check(root, &parsed)?;
    write_receipt(out, &receipt(root, config, &result))
        .map_err(|e| format!("cannot write {}: {e}", out.display()))?;
    Ok(result.exit_code)
}
