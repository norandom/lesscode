//! Size governance for Rust sources: the `rust-syntax-units-v2` metric.
//!
//! Enforces the ceilings in a budgets file. Size is counted in tokens
//! outside comments, so that layout cannot change it: by default 2,800
//! tokens per file, 300 logical units and 560 tokens per function (the
//! earlier 400 and 80 lines at seven tokens a line). A file whose entry
//! still gives a ceiling in lines is held to that ceiling, in lines.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::adapters::driver::measure::token_starts;
use crate::adapters::driver::registry::embedded;
use crate::receipt::write_receipt;
use report::receipt;

mod report;

/// One governed file; omitted ceilings fall back to the metric defaults.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileBudget {
    pub path: String,
    /// Ceiling on production tokens; the default when no ceiling is given.
    pub max_tokens: Option<usize>,
    /// Earlier form of the size ceiling, in lines; used when `max_tokens` is absent.
    pub max_physical_lines: Option<usize>,
    pub max_logical_units: Option<usize>,
    /// Ceiling on the tokens of one function.
    pub max_fn_tokens: Option<usize>,
    /// Earlier form, in lines; used when `max_fn_tokens` is absent.
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
    pub production_tokens: usize,
    pub test_tokens: usize,
    pub longest_fn_tokens: usize,
    pub longest_fn_tokens_name: String,
    /// File size, logical units, function size.
    pub limit: [usize; 3],
    /// Whether the file and the function ceiling are still given in lines.
    pub in_lines: [bool; 2],
    pub owner: String,
    pub allocation: String,
    pub disposition: Option<String>,
}

impl FileReport {
    pub fn exceeded(&self) -> bool {
        let pick = |lines: bool, a: usize, b: usize| if lines { a } else { b };
        let size = pick(
            self.in_lines[0],
            self.production_lines,
            self.production_tokens,
        );
        let function = pick(
            self.in_lines[1],
            self.longest_fn_lines,
            self.longest_fn_tokens,
        );
        self.present
            && (size > self.limit[0]
                || self.logical_units > self.limit[1]
                || function > self.limit[2])
    }

    pub fn status(&self) -> &'static str {
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
    /// Start and end (line, column) of each `#[cfg(test)]` subtree and of
    /// each production function with its name, for counting tokens.
    test_spans: Vec<Range>,
    fn_spans: Vec<(String, Range)>,
}

type Range = ((usize, usize), (usize, usize));

fn range(span: proc_macro2::Span) -> Range {
    let (a, b) = (span.start(), span.end());
    ((a.line, a.column), (b.line, b.column))
}

fn tokens_in(starts: &[(usize, usize)], (from, to): Range) -> usize {
    starts.iter().filter(|s| **s >= from && **s < to).count()
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
                self.fn_spans.push((i.sig.ident.to_string(), range(span)));
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
            if self.test_depth == 1 {
                self.test_spans.push(range(span));
            }
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

fn new_report(b: &FileBudget) -> FileReport {
    FileReport {
        path: b.path.clone(),
        limit: [
            b.max_tokens.or(b.max_physical_lines).unwrap_or(2800),
            b.max_logical_units.unwrap_or(300),
            b.max_fn_tokens.or(b.max_fn_lines).unwrap_or(560),
        ],
        in_lines: [
            b.max_tokens.is_none() && b.max_physical_lines.is_some(),
            b.max_fn_tokens.is_none() && b.max_fn_lines.is_some(),
        ],
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
        test_spans: Vec::new(),
        fn_spans: Vec::new(),
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
    let pack = embedded("rust").map_err(|e| e.to_string())?;
    let starts = token_starts(pack, &path.to_string_lossy(), &text)?;
    r.test_tokens = c.test_spans.iter().map(|s| tokens_in(&starts, *s)).sum();
    r.production_tokens = starts.len().saturating_sub(r.test_tokens);
    for (name, span) in c.fn_spans {
        let tokens = tokens_in(&starts, span);
        if tokens > r.longest_fn_tokens {
            r.longest_fn_tokens = tokens;
            r.longest_fn_tokens_name = name;
        }
    }
    Ok(r)
}

fn discover_rs(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let list_err = |e| format!("cannot list {}: {e}", dir.display());
        for entry in fs::read_dir(&dir).map_err(list_err)? {
            let path = entry.map_err(list_err)?.path();
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

/// Returns per-file reports, unlisted paths and the exit code (1 on any
/// exceeded, unlisted or missing file).
pub fn check(
    root: &Path,
    config: &BudgetsConfig,
) -> Result<(Vec<FileReport>, Vec<String>, u8), String> {
    let listed: Vec<String> = config.files.iter().map(|f| f.path.clone()).collect();
    let unlisted: Vec<String> = discover_rs(root)?
        .iter()
        .filter_map(|p| {
            p.strip_prefix(root)
                .ok()
                .map(|r| r.to_string_lossy().replace('\\', "/"))
        })
        .filter(|rel| !listed.contains(rel))
        .collect();
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
    Ok((reports, unlisted, u8::from(failed)))
}

/// Exit 0 when every discovered file is listed and within its ceilings; 1 on
/// any exceeded, unlisted or missing file; errors surface as `Err` (exit 3).
pub fn cli_run(root: &Path, config: &Path, out: &Path) -> Result<u8, String> {
    let text =
        fs::read_to_string(config).map_err(|e| format!("cannot read {}: {e}", config.display()))?;
    let parsed: BudgetsConfig = serde_json::from_str(&text)
        .map_err(|e| format!("invalid budgets config in {}: {e}", config.display()))?;
    let (reports, unlisted, code) = check(root, &parsed)?;
    write_receipt(out, &receipt(root, config, &reports, &unlisted))
        .map_err(|e| format!("cannot write {}: {e}", out.display()))?;
    Ok(code)
}
