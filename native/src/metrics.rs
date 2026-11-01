//! Versioned native syntax measurements; these are not Radon LLOC or Lizard CCN.

use crate::model::{
    Count, Language, MetricIdentity, MetricMeasurement, MetricName, MetricValue, Records,
    SchemaVersion,
};
use crate::scope::SourceSpan;
use std::collections::BTreeMap;

/// Frozen, caller-supplied source. No filesystem/module search or execution occurs.
pub struct CallSource<'a> {
    pub path: &'a str,
    pub source: &'a [u8],
    pub language: Language,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallEdge {
    pub caller: String,
    pub target: String,
    pub location: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallLayers {
    pub measurement: MetricMeasurement,
    pub resolved: Vec<CallEdge>,
    /// Keys are stable `path:byte_offset` IDs, accepted by Scope's ownership review.
    pub unresolved: Records<String, crate::model::Finding>,
    pub cycles: Vec<String>,
}

#[derive(Default)]
struct CallFile {
    functions: Vec<CallFunction>,
    calls: Vec<(SourceSpan, Option<String>, String)>,
    bindings: BTreeMap<String, Vec<String>>,
    imports: Vec<(String, String, String)>,
    unsafe_names: std::collections::BTreeSet<String>,
    wildcard_binding: bool,
    errors: Vec<String>,
}

struct CallFunction {
    id: String,
    span: SourceSpan,
    name: Option<String>,
    exported: bool,
}

fn in_scope(span: &SourceSpan, spans: &[SourceSpan]) -> bool {
    spans.iter().any(|s| {
        s.path == span.path && s.start <= span.start && span.end <= s.end && s.start < s.end
    })
}

// Intentionally small grammar surface: top-level undecorated Python def and TS
// function declarations (optionally export-wrapped); only identifier callees.
// Every other call remains located. A name with *any* non-call use outside these
// declarations/imports is unsafe file-wide, including shadowing and rebinding.
fn call_file<L: ast_grep_language::LanguageExt>(
    path: &str,
    text: &str,
    spans: &[SourceSpan],
    language: L,
    python: bool,
) -> CallFile {
    let tree = crate::AstGrep::new(text, language);
    let root = tree.root();
    let mut file = CallFile::default();
    let mut allowed_names = Vec::new();
    let mut import_ranges = Vec::new();
    for node in root.dfs() {
        if node.is_error() || node.is_missing() {
            file.errors.push(format!(
                "{path}:{}: syntax error or missing node",
                node.range().start
            ));
        }
        let kind = node.kind();
        let function = if python {
            PYTHON_FUNCTION_KINDS
        } else {
            TYPESCRIPT_FUNCTION_KINDS
        };
        if function.contains(&kind.as_ref()) {
            let range = node.range();
            let span = SourceSpan {
                path: path.into(),
                start: range.start,
                end: range.end,
            };
            let top = node.parent().is_some_and(|p| {
                p.range() == root.range() && p.kind() == root.kind()
                    || !python
                        && p.kind() == "export_statement"
                        && p.parent().is_some_and(|gp| gp.kind() == root.kind())
            });
            let supported = top
                && matches!(
                    kind.as_ref(),
                    "function_definition" | "function_declaration"
                );
            let name = if supported {
                node.field("name").map(|n| {
                    allowed_names.push(n.range());
                    n.text().to_string()
                })
            } else {
                None
            };
            let id = format!("{path}:{}", range.start);
            if let Some(name) = &name {
                file.bindings
                    .entry(name.clone())
                    .or_default()
                    .push(id.clone());
            }
            if in_scope(&span, spans) {
                let exported = python
                    || node.parent().is_some_and(|p| {
                        p.kind() == "export_statement"
                            && !p.children().any(|child| child.kind() == "default")
                    });
                file.functions.push(CallFunction {
                    id,
                    span,
                    name,
                    exported,
                });
            }
        }
        let top = node.parent().is_some_and(|p| p.kind() == root.kind());
        if top && python && kind == "import_from_statement" {
            // A star import may replace any module binding. No execution or
            // export-list analysis is available to prove a particular name safe.
            file.wildcard_binding |= node.named_children().any(|n| n.kind() == "wildcard_import");
            if let Some(module) = node.field("module_name") {
                for name in node
                    .named_children()
                    .filter(|n| n.range() != module.range())
                {
                    let (original, local) = if name.kind() == "aliased_import" {
                        (name.field("name"), name.field("alias"))
                    } else {
                        (Some(name.clone()), Some(name.clone()))
                    };
                    if let (Some(original), Some(local)) = (original, local) {
                        if original.kind() == "dotted_name"
                            && !original.text().contains('.')
                            && !local.text().contains('.')
                        {
                            file.imports.push((
                                local.text().to_string(),
                                module.text().to_string(),
                                original.text().to_string(),
                            ));
                            import_ranges.push(name.range());
                        }
                    }
                }
            }
        }
        if top && !python && kind == "import_statement" && !node.text().starts_with("import type") {
            if let Some(module) = node.field("source") {
                let raw = module.text();
                if raw.len() >= 2 && !raw.contains('\\') {
                    for spec in node.dfs().filter(|n| {
                        n.kind() == "import_specifier" && !n.text().starts_with("type ")
                    }) {
                        if let Some(original) = spec.field("name") {
                            let local = spec.field("alias").unwrap_or_else(|| original.clone());
                            file.imports.push((
                                local.text().to_string(),
                                raw[1..raw.len() - 1].into(),
                                original.text().to_string(),
                            ));
                            import_ranges.push(spec.range());
                        }
                    }
                }
            }
        }
    }
    for node in root.dfs() {
        let kind = node.kind();
        if kind == "identifier" {
            let range = node.range();
            let callee = node.parent().is_some_and(|p| {
                matches!(p.kind().as_ref(), "call" | "call_expression")
                    && p.field("function").is_some_and(|f| f.range() == range)
            });
            if !callee
                && !allowed_names.contains(&range)
                && !import_ranges
                    .iter()
                    .any(|r| r.start <= range.start && range.end <= r.end)
            {
                file.unsafe_names.insert(node.text().to_string());
            }
        }
        let bare_decorator =
            kind == "decorator" && !node.named_children().any(|n| n.kind() == "call");
        if !matches!(kind.as_ref(), "call" | "call_expression" | "new_expression")
            && !bare_decorator
        {
            continue;
        }
        let range = node.range();
        let location = SourceSpan {
            path: path.into(),
            start: range.start,
            end: range.end,
        };
        if !in_scope(&location, spans) {
            continue;
        }
        let caller = file
            .functions
            .iter()
            .filter(|f| f.span.start <= range.start && range.end <= f.span.end)
            .min_by_key(|f| f.span.end - f.span.start)
            .map(|f| f.id.clone())
            .unwrap_or_else(|| format!("{path}:module"));
        // Python defaults/annotations execute while the definition is created,
        // not as part of invoking its body. Keep such calls located/unresolved
        // rather than inventing an edge from that function.
        let definition_context = python
            && node.ancestors().any(|ancestor| {
                PYTHON_FUNCTION_KINDS.contains(&ancestor.kind().as_ref())
                    && ancestor.field("body").is_none_or(|body| {
                        let body = body.range();
                        !(body.start <= range.start && range.end <= body.end)
                    })
            });
        let name = node
            .field("function")
            .filter(|n| {
                !definition_context
                    && n.kind() == "identifier"
                    && !node.named_children().any(|c| c.kind() == "optional_chain")
            })
            .map(|n| n.text().to_string());
        file.calls.push((location, name, caller));
    }
    file
}

/// Resolve a module name only against the explicitly supplied descriptor paths.
/// Python absolute/from-relative imports and TS relative named imports are the
/// supported forms; package __init__, indexes, aliases and reexports are omitted.
fn import_paths(path: &str, module: &str, language: Language) -> Vec<String> {
    let mut parts: Vec<&str> = path.split('/').collect();
    parts.pop();
    let mut base = if language == Language::Python {
        if module.starts_with('.') {
            for _ in 1..module.bytes().take_while(|b| *b == b'.').count() {
                if parts.pop().is_none() {
                    return vec![];
                }
            }
            parts.into_iter().map(str::to_string).collect::<Vec<_>>()
        } else {
            vec![]
        }
    } else {
        if !module.starts_with("./") && !module.starts_with("../") {
            return vec![];
        }
        parts.into_iter().map(str::to_string).collect::<Vec<_>>()
    };
    let module = if language == Language::Python {
        module.trim_start_matches('.').replace('.', "/")
    } else {
        module.into()
    };
    for part in module.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if base.pop().is_none() {
                    return vec![];
                }
            }
            _ => base.push(part.into()),
        }
    }
    let joined = base.join("/");
    if language == Language::Python {
        // A supplied package competes with a same-named module. Packages remain
        // unsupported targets; their presence must still prevent a false choice.
        vec![format!("{joined}.py"), format!("{joined}/__init__.py")]
    } else if joined.ends_with(".ts") || joined.ends_with(".tsx") {
        vec![joined]
    } else {
        vec![format!("{joined}.ts"), format!("{joined}.tsx")]
    }
}

pub fn measure_call_layers(
    sources: &[CallSource<'_>],
    spans: &[SourceSpan],
    entry_points: &[SourceSpan],
    scope_identity: &str,
    language: Language,
) -> CallLayers {
    let mut files = BTreeMap::new();
    let mut errors = Vec::new();
    for source in sources.iter().filter(|s| {
        s.language == language || language != Language::Python && s.language != Language::Python
    }) {
        if files.contains_key(source.path) {
            errors.push(format!("{}: duplicate source descriptor", source.path));
            continue;
        }
        let Ok(text) = std::str::from_utf8(source.source) else {
            errors.push(format!("{}: source is not UTF-8", source.path));
            continue;
        };
        if spans.iter().filter(|s| s.path == source.path).any(|s| {
            s.start > s.end
                || s.end > text.len()
                || !text.is_char_boundary(s.start)
                || !text.is_char_boundary(s.end)
        }) {
            errors.push(format!("{}: invalid selected byte range", source.path));
            continue;
        }
        let file = match source.language {
            Language::Python => call_file(source.path, text, spans, crate::Python, true),
            Language::TypeScript => call_file(source.path, text, spans, crate::TypeScript, false),
            Language::Tsx => call_file(source.path, text, spans, crate::Tsx, false),
        };
        errors.extend(file.errors.iter().cloned());
        files.insert(source.path, file);
    }
    for span in spans {
        if !files.contains_key(span.path.as_str()) {
            errors.push(format!(
                "{}: selected source descriptor is missing",
                span.path
            ));
        }
    }
    for entry in entry_points {
        if !in_scope(entry, spans) {
            errors.push(format!(
                "{}:{}: entry point is outside selected scope",
                entry.path, entry.start
            ));
        }
    }
    let mut resolved = Vec::new();
    let mut unresolved = BTreeMap::new();
    let mut graph: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut roots = std::collections::BTreeSet::new();
    for (path, file) in &files {
        let mut bindings = file.bindings.clone();
        for (local, module, original) in &file.imports {
            let candidates: Vec<_> = import_paths(path, module, language)
                .into_iter()
                .filter_map(|p| files.get(p.as_str()).map(|file| (p, file)))
                .collect();
            let targets = if candidates.len() == 1 {
                let (target_path, target) = &candidates[0];
                if !target_path.ends_with("/__init__.py")
                    && !target.wildcard_binding
                    && target.errors.is_empty()
                    && !target.unsafe_names.contains(original)
                    && target
                        .bindings
                        .get(original)
                        .is_some_and(|ids| ids.len() == 1)
                {
                    target
                        .functions
                        .iter()
                        .filter(|f| f.name.as_ref() == Some(original) && f.exported)
                        .map(|f| f.id.clone())
                        .collect()
                } else {
                    vec![]
                }
            } else {
                vec![]
            };
            // Invalid or duplicate imports must not leave a local definition usable.
            let ids = bindings.entry(local.clone()).or_default();
            if targets.len() == 1 {
                ids.extend(targets);
            } else {
                ids.push(String::new());
            }
        }
        for function in &file.functions {
            graph.entry(function.id.clone()).or_default();
            if in_scope(&function.span, entry_points) {
                roots.insert(function.id.clone());
            }
        }
        for (location, name, caller) in &file.calls {
            if caller.ends_with(":module") {
                graph.entry(caller.clone()).or_default();
                if in_scope(location, entry_points) {
                    roots.insert(caller.clone());
                }
            }
            let target = name
                .as_ref()
                .filter(|n| {
                    !file.wildcard_binding
                        && !file.unsafe_names.contains(*n)
                        && file.errors.is_empty()
                })
                .and_then(|n| bindings.get(n))
                .filter(|ids| ids.len() == 1)
                .and_then(|ids| ids.first())
                .filter(|id| {
                    files
                        .values()
                        .any(|f| f.functions.iter().any(|fun| &fun.id == *id))
                });
            if let Some(target) = target {
                graph
                    .entry(caller.clone())
                    .or_default()
                    .push(target.clone());
                resolved.push(CallEdge {
                    caller: caller.clone(),
                    target: target.clone(),
                    location: location.clone(),
                });
            } else {
                let id = format!("{}:{}", location.path, location.start);
                unresolved.insert(id.clone(), crate::model::Finding { schema_version: SchemaVersion::V1, subject: id, explanation: format!("unresolved call at bytes {}..{}; dynamic, external, ambiguous or unsupported binding; ownership review required", location.start, location.end) });
            }
        }
    }
    fn depth(
        id: &str,
        graph: &BTreeMap<String, Vec<String>>,
        active: &mut Vec<String>,
        memo: &mut BTreeMap<String, u64>,
        cycles: &mut std::collections::BTreeSet<String>,
    ) -> u64 {
        if active.iter().any(|s| s == id) {
            cycles.insert(id.into());
            return 0;
        }
        if let Some(depth) = memo.get(id) {
            return *depth;
        }
        active.push(id.into());
        let child = graph[id]
            .iter()
            .map(|next| depth(next, graph, active, memo, cycles))
            .max()
            .unwrap_or(0);
        active.pop();
        let result = child
            .checked_add(1)
            .expect("path depth is bounded by allocated vertices");
        // Back edges terminate at zero. Memoization retains a witnessed simple
        // path lower bound on cycles, and exact longest paths on acyclic graphs.
        memo.insert(id.into(), result);
        result
    }
    let mut cycles = std::collections::BTreeSet::new();
    let mut memo = BTreeMap::new();
    let mut lower = 0;
    for root in roots {
        lower = lower.max(depth(
            &root,
            &graph,
            &mut Vec::new(),
            &mut memo,
            &mut cycles,
        ));
    }
    for vertex in graph.keys() {
        depth(vertex, &graph, &mut Vec::new(), &mut memo, &mut cycles);
    }
    let value = if errors.is_empty() && unresolved.is_empty() && cycles.is_empty() {
        available(lower)
    } else {
        MetricValue::Unavailable {
            reason: format!(
                "{} unresolved calls; {} cycle vertices; {}",
                unresolved.len(),
                cycles.len(),
                errors.join("; ")
            ),
            resolved_lower_bound: Some(Count(lower)),
        }
    };
    CallLayers {
        measurement: MetricMeasurement {
            schema_version: SchemaVersion::V1,
            identity: MetricIdentity {
                schema_version: SchemaVersion::V1,
                name: MetricName::InternalCallLayers,
                definition_revision: "v1".into(),
                language,
                grammar_version: if language == Language::Python {
                    "tree-sitter-python@0.25.0"
                } else {
                    "tree-sitter-typescript@0.23.2"
                }
                .into(),
                tool_version: "lesscode-core@0.1.0;ast-grep@0.51.0".into(),
            },
            scope_identity: scope_identity.into(),
            value,
        },
        resolved,
        unresolved: Records(unresolved),
        cycles: cycles.into_iter().collect(),
    }
}

#[cfg(test)]
mod call_tests {
    use super::*;

    #[test]
    fn review_wildcard_import_can_replace_local_bindings() {
        let result = graph(
            &[(
                "a.py",
                "def target():\n    pass\nfrom other import *\ndef entry():\n    target()\n",
            )],
            Language::Python,
        );
        assert!(result.resolved.is_empty(), "{result:?}");
        assert_eq!(result.unresolved.0.len(), 1);
    }

    #[test]
    fn review_default_export_is_not_named_export_with_trivia() {
        for helper in [
            "export  default function helper() {}",
            "export /*comment*/ default function helper() {}",
        ] {
            let result = graph(
                &[
                    (
                        "a.ts",
                        "import {helper} from './helper'; function entry() { helper(); }",
                    ),
                    ("helper.ts", helper),
                ],
                Language::TypeScript,
            );
            assert!(result.resolved.is_empty(), "{result:?}");
            assert_eq!(result.unresolved.0.len(), 1);
        }
    }

    #[test]
    fn review_python_default_call_is_not_entry_body_edge() {
        let result = graph(
            &[(
                "a.py",
                "def tail():\n    pass\ndef mid():\n    tail()\ndef entry(x=mid()):\n    pass\n",
            )],
            Language::Python,
        );
        assert_eq!(result.unresolved.0.len(), 1, "{result:?}");
        assert_eq!(result.resolved.len(), 1);
        assert!(matches!(
            result.measurement.value,
            MetricValue::Unavailable {
                resolved_lower_bound: Some(Count(2)),
                ..
            }
        ));
    }

    #[test]
    fn review_python_module_package_collision_is_ambiguous() {
        let result = graph(
            &[
                (
                    "a.py",
                    "from helper import tail\ndef entry():\n    tail()\n",
                ),
                ("helper.py", "def tail():\n    pass\n"),
                ("helper/__init__.py", "def tail():\n    pass\n"),
            ],
            Language::Python,
        );
        assert!(result.resolved.is_empty(), "{result:?}");
        assert_eq!(result.unresolved.0.len(), 1);
    }

    fn graph(files: &[(&str, &str)], language: Language) -> CallLayers {
        let sources: Vec<_> = files
            .iter()
            .map(|(path, text)| CallSource {
                path,
                source: text.as_bytes(),
                language,
            })
            .collect();
        let spans: Vec<_> = files
            .iter()
            .map(|(path, text)| SourceSpan {
                path: (*path).into(),
                start: 0,
                end: text.len(),
            })
            .collect();
        measure_call_layers(&sources, &spans, &spans, "frozen-scope", language)
    }

    #[test]
    fn call_layers_direct_imported_and_entry_vertex() {
        for (language, files) in [
            (Language::Python, vec![("main.py", "from helper import tail as end\ndef entry():\n    middle()\ndef middle():\n    end()\n"), ("helper.py", "def tail():\n    pass\n")]),
            (Language::TypeScript, vec![("main.ts", "import {tail as end} from './helper';\nfunction entry() { middle(); }\nfunction middle() { end(); }"), ("helper.ts", "export function tail() {}")]),
            (Language::Tsx, vec![("main.tsx", "import {tail as end} from './helper';\nfunction entry() { middle(); }\nfunction middle() { end(); }"), ("helper.tsx", "export function tail() { return <div/>; }")]),
        ] {
            let result = graph(&files, language);
            assert_eq!(result.measurement.value, available(3), "{language:?}: {result:?}");
            assert_eq!(result.resolved.len(), 2);
            assert!(result.unresolved.0.is_empty());
        }
        assert_eq!(
            graph(&[("a.py", "def leaf():\n    pass\n")], Language::Python)
                .measurement
                .value,
            available(1)
        );
    }

    #[test]
    fn call_layers_ambiguous_shadowed_nested_and_dynamic_are_located() {
        for (language, text) in [
            (Language::Python, "def target():\n    pass\ndef entry(target):\n    target()\n    obj.method()\n"),
            (Language::Python, "def target():\n    pass\ntarget = other\ndef entry():\n    target()\n"),
            (Language::Python, "def entry():\n    def nested():\n        pass\n    nested()\n"),
            (Language::TypeScript, "function target() {} function entry(target: () => void) { target(); obj.method(); }"),
            (Language::TypeScript, "function target() {} target = other; function entry() { target(); }"),
            (Language::TypeScript, "function entry() { function nested() {} nested(); }"),
        ] {
            let path = if language == Language::Python { "a.py" } else { "a.ts" };
            let result = graph(&[(path, text)], language);
            assert!(result.resolved.is_empty(), "{text}: {result:?}");
            assert!(!result.unresolved.0.is_empty(), "{text}");
            assert!(matches!(result.measurement.value, MetricValue::Unavailable { resolved_lower_bound: Some(_), .. }));
            for (id, finding) in result.unresolved.0 { assert_eq!(id, finding.subject); assert!(id.starts_with(path)); }
        }
    }

    #[test]
    fn call_layers_cycles_and_lower_bound_are_distinct() {
        let result = graph(
            &[("a.py", "def a():\n    b()\ndef b():\n    a()\n")],
            Language::Python,
        );
        assert!(!result.cycles.is_empty());
        assert_eq!(result.resolved.len(), 2);
        assert!(matches!(
            result.measurement.value,
            MetricValue::Unavailable {
                resolved_lower_bound: Some(Count(2)),
                ..
            }
        ));
        let result = graph(
            &[("a.ts", "function a() { a(); missing(); }")],
            Language::TypeScript,
        );
        assert_eq!(result.unresolved.0.len(), 1);
        assert!(!result.cycles.is_empty());
    }

    #[test]
    fn call_layers_do_not_search_for_modules_or_assume_exports() {
        for (language, files) in [
            (
                Language::Python,
                vec![(
                    "main.py",
                    "from absent import helper\ndef entry():\n    helper()\n",
                )],
            ),
            (
                Language::TypeScript,
                vec![
                    (
                        "main.ts",
                        "import {helper} from './helper'; function entry() { helper(); }",
                    ),
                    ("helper.ts", "function helper() {}"),
                ],
            ),
            (
                Language::TypeScript,
                vec![
                    (
                        "main.ts",
                        "import {helper} from './helper'; function entry() { helper(); }",
                    ),
                    ("helper.ts", "export function helper() {}"),
                    ("helper.tsx", "export function helper() {}"),
                ],
            ),
        ] {
            let result = graph(&files, language);
            assert_eq!(result.unresolved.0.len(), 1);
            assert!(result.resolved.is_empty());
        }
        // A TSX file can directly import a declared TS helper; no runtime loader.
        let a = "import {helper} from './helper'; function entry() { helper(); }";
        let b = "export function helper() {}";
        let spans = [
            SourceSpan {
                path: "a.tsx".into(),
                start: 0,
                end: a.len(),
            },
            SourceSpan {
                path: "helper.ts".into(),
                start: 0,
                end: b.len(),
            },
        ];
        let result = measure_call_layers(
            &[
                CallSource {
                    path: "a.tsx",
                    source: a.as_bytes(),
                    language: Language::Tsx,
                },
                CallSource {
                    path: "helper.ts",
                    source: b.as_bytes(),
                    language: Language::TypeScript,
                },
            ],
            &spans,
            &spans,
            "scope",
            Language::Tsx,
        );
        assert_eq!(result.measurement.value, available(2));
    }

    #[test]
    fn call_layers_selection_and_unavailable_source_are_preserved() {
        let text = "def entry():\n    helper()\ndef helper():\n    pass\n";
        let selected = SourceSpan {
            path: "a.py".into(),
            start: 0,
            end: text.find("def helper").unwrap() - 1,
        };
        let result = measure_call_layers(
            &[CallSource {
                path: "a.py",
                source: text.as_bytes(),
                language: Language::Python,
            }],
            std::slice::from_ref(&selected),
            std::slice::from_ref(&selected),
            "scope",
            Language::Python,
        );
        assert_eq!(result.unresolved.0.len(), 1);
        assert!(result.resolved.is_empty());
        for bytes in [&b"\xff"[..], &b"def broken(:\n"[..]] {
            let span = SourceSpan {
                path: "a.py".into(),
                start: 0,
                end: bytes.len(),
            };
            let result = measure_call_layers(
                &[CallSource {
                    path: "a.py",
                    source: bytes,
                    language: Language::Python,
                }],
                std::slice::from_ref(&span),
                std::slice::from_ref(&span),
                "scope",
                Language::Python,
            );
            assert!(matches!(
                result.measurement.value,
                MetricValue::Unavailable { .. }
            ));
        }
    }
}

/// logical_statements.v1: Python 0.25.0 `_simple_statement` and
/// `_compound_statement` alternatives. Suites, comments, decorators and clauses
/// (elif/else/except/finally/case) add no statements. A decorated definition
/// replaces its wrapped declaration, but its body still contributes statements.
/// Async definitions/loops/with share these node kinds.
pub const PYTHON_STATEMENT_KINDS: &[&str] = &[
    "assert_statement",
    "break_statement",
    "continue_statement",
    "delete_statement",
    "exec_statement",
    "expression_statement",
    "future_import_statement",
    "global_statement",
    "import_from_statement",
    "import_statement",
    "nonlocal_statement",
    "pass_statement",
    "print_statement",
    "raise_statement",
    "return_statement",
    "type_alias_statement",
    "class_definition",
    "decorated_definition",
    "for_statement",
    "function_definition",
    "if_statement",
    "match_statement",
    "try_statement",
    "while_statement",
    "with_statement",
];

/// decisions.v1: one per listed node. Boolean operators are binary grammar
/// nodes: one per `and`/`or`; `not`, assert and try add none. `if_clause`
/// covers comprehension filters and match guards. Cases count except a sole
/// bare `_` without a guard; capture/union patterns count. Each generator's
/// `for_in_clause` counts once, including async. No implicit function baseline.
pub const PYTHON_DECISION_KINDS: &[&str] = &[
    "if_statement",
    "elif_clause",
    "conditional_expression",
    "for_statement",
    "while_statement",
    "except_clause",
    "boolean_operator",
    "for_in_clause",
    "if_clause",
];

/// functions.v1 includes methods, async/nested functions and lambdas.
pub const PYTHON_FUNCTION_KINDS: &[&str] = &["function_definition", "lambda"];

fn unavailable(reason: impl Into<String>) -> MetricValue {
    MetricValue::Unavailable {
        reason: reason.into(),
        resolved_lower_bound: None,
    }
}

fn available(count: u64) -> MetricValue {
    MetricValue::Available {
        count: Count(count),
    }
}

/// logical_statements.v1 for TypeScript/TSX grammar 0.23.2: statement and
/// declaration alternatives, excluding blocks. Export/ambient wrappers replace
/// their direct declarations (including nested wrappers). The grammar's bare
/// namespace expression_statement/internal_module pair also counts once. Class members, enum
/// members, interface signatures and type expressions are not statements.
/// A for initializer declaration counts; empty header separators do not.
pub const TYPESCRIPT_STATEMENT_KINDS: &[&str] = &[
    "export_statement",
    "import_statement",
    "debugger_statement",
    "expression_statement",
    "if_statement",
    "switch_statement",
    "for_statement",
    "for_in_statement",
    "while_statement",
    "do_statement",
    "try_statement",
    "with_statement",
    "break_statement",
    "continue_statement",
    "return_statement",
    "throw_statement",
    "empty_statement",
    "labeled_statement",
    "function_declaration",
    "generator_function_declaration",
    "class_declaration",
    "lexical_declaration",
    "variable_declaration",
    "function_signature",
    "abstract_class_declaration",
    "module",
    "internal_module",
    "type_alias_declaration",
    "enum_declaration",
    "interface_declaration",
    "import_alias",
    "ambient_declaration",
];

/// decisions.v1: else-if is another if; switch_case excludes switch_default.
/// for-in/of/await-of share one node. Catch counts once, finally/try do not.
/// Binary &&/||/?? each count once; bitwise and logical assignments do not.
/// JSX containers add no decision; expressions inside them follow these rules.
pub const TYPESCRIPT_DECISION_KINDS: &[&str] = &[
    "if_statement",
    "ternary_expression",
    "for_statement",
    "for_in_statement",
    "while_statement",
    "do_statement",
    "catch_clause",
    "switch_case",
];

/// functions.v1 counts executable definitions, including async/generators,
/// constructors, accessors, object/class methods and nested arrows. Overloads,
/// ambient declarations, interface methods and abstract signatures do not count.
pub const TYPESCRIPT_FUNCTION_KINDS: &[&str] = &[
    "function_declaration",
    "generator_function_declaration",
    "function_expression",
    "generator_function",
    "method_definition",
    "arrow_function",
];

/// Measure TypeScript or TSX with distinct grammar identities and no source execution.
pub fn measure_typescript(
    path: &str,
    source: &[u8],
    scope_identity: &str,
    spans: &[SourceSpan],
    tsx: bool,
) -> Records<MetricName, MetricMeasurement> {
    let language = if tsx {
        Language::Tsx
    } else {
        Language::TypeScript
    };
    let measured = if tsx {
        syntax_values(path, source, spans, crate::Tsx, false)
    } else {
        syntax_values(path, source, spans, crate::TypeScript, false)
    };
    measurements(
        measured,
        scope_identity,
        language,
        "tree-sitter-typescript@0.23.2",
    )
}

pub fn measure_python(
    path: &str,
    source: &[u8],
    scope_identity: &str,
    spans: &[SourceSpan],
) -> Records<MetricName, MetricMeasurement> {
    let measured = syntax_values(path, source, spans, crate::Python, true);
    measurements(
        measured,
        scope_identity,
        Language::Python,
        "tree-sitter-python@0.25.0",
    )
}

fn measurements(
    measured: [MetricValue; 4],
    scope_identity: &str,
    language: Language,
    grammar_version: &str,
) -> Records<MetricName, MetricMeasurement> {
    Records(
        [
            MetricName::PhysicalLines,
            MetricName::LogicalStatements,
            MetricName::Decisions,
            MetricName::Functions,
        ]
        .into_iter()
        .zip(measured)
        .map(|(name, value)| {
            (
                name,
                MetricMeasurement {
                    schema_version: SchemaVersion::V1,
                    identity: MetricIdentity {
                        schema_version: SchemaVersion::V1,
                        name,
                        definition_revision: "v1".into(),
                        language,
                        grammar_version: grammar_version.into(),
                        tool_version: "lesscode-core@0.1.0;ast-grep@0.51.0".into(),
                    },
                    scope_identity: scope_identity.into(),
                    value,
                },
            )
        })
        .collect::<BTreeMap<_, _>>(),
    )
}

/// Measure supplied bytes without execution or IO. Spans must be complete nodes
/// or files from scope resolution; other paths are ignored. physical_lines.v1
/// counts LF-separated byte lines (CRLF is one separator), including blanks and
/// comments. Empty input and trailing LF add no line. Line spans are unioned.
fn syntax_values<L: ast_grep_language::LanguageExt>(
    path: &str,
    source: &[u8],
    spans: &[SourceSpan],
    language: L,
    python: bool,
) -> [MetricValue; 4] {
    let spans: Vec<_> = spans
        .iter()
        .filter(|span| span.path == path)
        .cloned()
        .collect();
    if spans
        .iter()
        .any(|s| s.start > s.end || s.end > source.len())
    {
        return std::array::from_fn(|_| unavailable("invalid selected byte range"));
    }
    let spans = crate::scope::union_spans(spans);
    let selected = |start: usize, end: usize| {
        spans
            .iter()
            .any(|s| s.start <= start && end <= s.end && s.start < s.end)
    };
    let intersects = |start: usize, end: usize| {
        spans
            .iter()
            .any(|s| s.start < s.end && start < s.end && s.start < end)
    };
    let mut lines = 0_u64;
    let mut line_start = 0;
    for (index, byte) in source.iter().enumerate() {
        if *byte == b'\n' {
            if intersects(line_start, index + 1) {
                lines += 1;
            }
            line_start = index + 1;
        }
    }
    if line_start < source.len() && intersects(line_start, source.len()) {
        lines += 1;
    }
    let mut result = [available(lines), available(0), available(0), available(0)];
    let fail = |reason: &str, result: &mut [MetricValue; 4]| {
        for value in &mut result[1..] {
            *value = unavailable(reason);
        }
    };
    let Ok(text) = std::str::from_utf8(source) else {
        fail("source is not UTF-8", &mut result);
        return result;
    };
    if spans
        .iter()
        .any(|s| !text.is_char_boundary(s.start) || !text.is_char_boundary(s.end))
    {
        fail("selected byte range splits a UTF-8 character", &mut result);
        return result;
    }
    let tree = crate::AstGrep::new(text, language);
    let root = tree.root();
    if root.dfs().any(|node| {
        let range = node.range();
        (node.is_error() || node.is_missing())
            && (intersects(range.start, range.end)
                || range.is_empty()
                    && spans
                        .iter()
                        .any(|s| s.start < s.end && s.start <= range.start && range.start <= s.end))
    }) {
        fail(
            "syntax error or missing node in selected scope",
            &mut result,
        );
        return result;
    }
    let mut counts = [0_u64; 3];
    for node in root.dfs() {
        let range = node.range();
        if !node.is_named() || !selected(range.start, range.end) {
            continue;
        }
        let kind = node.kind();
        let wrapped = python
            && matches!(kind.as_ref(), "function_definition" | "class_definition")
            && node.parent().is_some_and(|p| {
                p.kind() == "decorated_definition" && selected(p.range().start, p.range().end)
            });
        let typed_wrapped = !python
            && node.parent().is_some_and(|p| {
                (matches!(
                    p.kind().as_ref(),
                    "export_statement" | "ambient_declaration"
                ) || kind == "internal_module" && p.kind() == "expression_statement")
                    && selected(p.range().start, p.range().end)
            });
        let empty_header = !python
            && kind == "empty_statement"
            && node.parent().is_some_and(|p| {
                p.kind() == "for_statement"
                    && p.field("body").is_none_or(|body| body.range() != range)
            });
        let statement = if python {
            PYTHON_STATEMENT_KINDS.contains(&kind.as_ref()) && !wrapped
        } else {
            TYPESCRIPT_STATEMENT_KINDS.contains(&kind.as_ref()) && !typed_wrapped && !empty_header
        };
        let default_case = kind == "case_clause"
            && node.field("guard").is_none()
            && node
                .named_children()
                .filter(|c| c.kind() == "case_pattern")
                .count()
                == 1
            && node
                .named_children()
                .any(|c| c.kind() == "case_pattern" && c.text() == "_");
        let decision = if python {
            PYTHON_DECISION_KINDS.contains(&kind.as_ref())
                || (kind == "case_clause" && !default_case)
        } else {
            TYPESCRIPT_DECISION_KINDS.contains(&kind.as_ref())
                || kind == "binary_expression"
                    && node
                        .field("operator")
                        .is_some_and(|op| matches!(op.text().as_ref(), "&&" | "||" | "??"))
        };
        for (count, increment) in counts.iter_mut().zip([
            statement,
            decision,
            if python {
                PYTHON_FUNCTION_KINDS.contains(&kind.as_ref())
            } else {
                TYPESCRIPT_FUNCTION_KINDS.contains(&kind.as_ref())
            },
        ]) {
            let Some(next) = count.checked_add(u64::from(increment)) else {
                fail("metric count overflow", &mut result);
                return result;
            };
            *count = next;
        }
    }
    for (value, count) in result[1..].iter_mut().zip(counts) {
        *value = available(count);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typed_counts(source: &str, tsx: bool, expected: [u64; 4]) {
        let result = measure_typescript(
            "a.ts",
            source.as_bytes(),
            "typed-scope",
            &[SourceSpan {
                path: "a.ts".into(),
                start: 0,
                end: source.len(),
            }],
            tsx,
        );
        for (name, count) in [
            MetricName::PhysicalLines,
            MetricName::LogicalStatements,
            MetricName::Decisions,
            MetricName::Functions,
        ]
        .into_iter()
        .zip(expected)
        {
            assert_eq!(
                result.0[&name].value,
                available(count),
                "{name:?}: {source}"
            );
            assert_eq!(result.0[&name].scope_identity, "typed-scope");
            assert_eq!(
                result.0[&name].identity.language,
                if tsx {
                    Language::Tsx
                } else {
                    Language::TypeScript
                }
            );
            assert_eq!(
                result.0[&name].identity.grammar_version,
                "tree-sitter-typescript@0.23.2"
            );
            assert_eq!(result.0[&name].identity.definition_revision, "v1");
        }
    }

    #[test]
    fn typescript_typed_declarations_wrappers_and_functions() {
        // Interface/type/ambient/exported const/exported function/class are six
        // statements; return + nested lexical declaration + return add three.
        // Interface signatures, overloads and abstract signatures are not definitions.
        typed_counts("interface I { f(x: number): number; p: string }\ntype T = I | null;\ndeclare function ambient(x: T): void;\nexport const arrow = (x: number): number => x;\nexport function outer(): number { const inner = function () { return 1; }; return inner(); }\nclass C { method() {} abstract absent(): void; }\n", false, [6, 9, 0, 4]);
    }

    #[test]
    fn typescript_control_flow_hand_counted() {
        // function/if/return/for/expression/while/break/do/expression/try/throw/
        // expression/switch/three breaks/return + for initializer = 18.
        // switch cases and catch are clauses, not statements. Decisions are if,
        // three short circuits, ternary, three loops, catch, two cases = 11.
        typed_counts("function f(a: number, xs: number[]) {\nif (a && a || a ?? 0) return a ? 1 : 2;\nfor (let i = 0; i < a; i++) a++;\nwhile (a) break;\ndo { a--; } while (a);\ntry { throw a; } catch (e) { a++; } finally {}\nswitch (a) { case 1: break; case 2: break; default: break; }\nreturn 0;\n}\n", false, [9, 18, 11, 1]);
        typed_counts(
            "for (const x of xs) { continue; }\nfor (const k in obj) { debugger; }\n",
            false,
            [2, 4, 2, 0],
        );
    }

    #[test]
    fn tsx_jsx_expressions_have_independent_counts() {
        typed_counts("export const View = ({ok}: {ok: boolean}) => <div>{ok && (ok ? <span/> : null)}</div>;\n", true, [1, 1, 2, 1]);
        typed_counts("const identity = <T,>(x: T): T => x;\n", true, [1, 1, 0, 1]);
        typed_counts("const n = <number>value;\n", false, [1, 1, 0, 0]);
    }

    #[test]
    fn typed_errors_encoding_and_grammar_difference_keep_physical() {
        for (source, tsx) in [
            (b"const x: = ;\n".as_slice(), false),
            (b"const v = <div/>;\n", false),
            (b"const n = <number>value;\n", true),
            (b"const v = <div>{</div>;\n", true),
            (b"\xff\n", false),
        ] {
            let result = measure_typescript(
                "a.ts",
                source,
                "scope",
                &[SourceSpan {
                    path: "a.ts".into(),
                    start: 0,
                    end: source.len(),
                }],
                tsx,
            );
            assert_eq!(result.0[&MetricName::PhysicalLines].value, available(1));
            for name in [
                MetricName::LogicalStatements,
                MetricName::Decisions,
                MetricName::Functions,
            ] {
                assert!(
                    matches!(&result.0[&name].value, MetricValue::Unavailable{reason, resolved_lower_bound:None} if !reason.is_empty())
                );
            }
        }
    }

    #[test]
    fn typed_union_selection_lines_and_scope_errors() {
        let source = "// α\r\nexport function f() { return 1; }\r\nconst broken: = ;\r\n";
        let start = source.find("export").unwrap();
        let end = source.find("\r\nconst").unwrap();
        let selected = SourceSpan {
            path: "a.ts".into(),
            start,
            end,
        };
        for tsx in [false, true] {
            let result = measure_typescript(
                "a.ts",
                source.as_bytes(),
                "scope",
                &[
                    selected.clone(),
                    selected.clone(),
                    SourceSpan {
                        path: "a.ts".into(),
                        start: source.find("return").unwrap(),
                        end: end - 2,
                    },
                    SourceSpan {
                        path: "other.ts".into(),
                        start: 0,
                        end: usize::MAX,
                    },
                ],
                tsx,
            );
            for (name, count) in [
                (MetricName::PhysicalLines, 1),
                (MetricName::LogicalStatements, 2),
                (MetricName::Decisions, 0),
                (MetricName::Functions, 1),
            ] {
                assert_eq!(result.0[&name].value, available(count));
            }
            let empty = measure_typescript("a.ts", b"const x = 1;", "scope", &[], tsx);
            assert!(empty.0.values().all(|m| m.value == available(0)));
            let invalid = measure_typescript(
                "a.ts",
                b"const x = 1;",
                "scope",
                &[SourceSpan {
                    path: "a.ts".into(),
                    start: 0,
                    end: 99,
                }],
                tsx,
            );
            assert!(invalid
                .0
                .values()
                .all(|m| matches!(m.value, MetricValue::Unavailable { .. })));
            let split = measure_typescript(
                "a.ts",
                source.as_bytes(),
                "scope",
                &[SourceSpan {
                    path: "a.ts".into(),
                    start: 4,
                    end: 5,
                }],
                tsx,
            );
            assert_eq!(split.0[&MetricName::PhysicalLines].value, available(1));
            assert!(matches!(
                split.0[&MetricName::Functions].value,
                MetricValue::Unavailable { .. }
            ));
        }
        typed_counts("", false, [0, 0, 0, 0]);
        typed_counts("// α\r\n\r\nconst x = '💡';\r\n", true, [3, 1, 0, 0]);
    }

    #[test]
    fn typed_declaration_families_and_header_rules() {
        // import/import alias/type/enum/namespace/interface/overload/definition/
        // class/export list/export ambient = 11 module statements; namespace
        // const, function return, class return, ambient namespace const add four.
        let source = "import { x } from 'x';\nimport Alias = NS.X;\ntype F = (x: number) => void;\nenum E { A, B }\nnamespace N { export const x = 1; }\ninterface I { new(): I; (): void; method(): void; }\nfunction f(x: number): number;\nfunction f(x: number) { return x; }\nabstract class C { abstract absent(): void; constructor() {} get x() { return 1; } set x(v: number) {} field = () => {}; }\nexport { f };\nexport declare namespace D { const v: number; }\n";
        for tsx in [false, true] {
            typed_counts(source, tsx, [11, 15, 0, 5]);
        }
        // Empty for separators add none; empty body and standalone ; each count.
        typed_counts("for (;;) ;\n;\n", false, [2, 3, 1, 0]);
        typed_counts("async function* f(xs: any) { for await (const x of xs) { yield x; } }\nconst obj = { async method() {}, *gen() {} };\n", false, [2,4,1,3]);
        // Bitwise / logical assignment do not add decisions. Else-if counts as if.
        typed_counts(
            "if (a) {} else if (b) {} else {}\na &= b; a &&= b; a ||= b; a ??= b;\n",
            true,
            [2, 6, 2, 0],
        );
    }

    fn span(start: usize, end: usize) -> SourceSpan {
        SourceSpan {
            path: "a.py".into(),
            start,
            end,
        }
    }
    fn values(source: &[u8], spans: &[SourceSpan]) -> Records<MetricName, MetricMeasurement> {
        measure_python("a.py", source, "selected-scope", spans)
    }
    fn counts(source: &str, expected: [u64; 4]) {
        let result = values(source.as_bytes(), &[span(0, source.len())]);
        for (name, count) in [
            MetricName::PhysicalLines,
            MetricName::LogicalStatements,
            MetricName::Decisions,
            MetricName::Functions,
        ]
        .into_iter()
        .zip(expected)
        {
            assert_eq!(
                result.0[&name].value,
                MetricValue::Available {
                    count: Count(count)
                },
                "{name:?}: {source}"
            );
            assert_eq!(result.0[&name].scope_identity, "selected-scope");
            assert_eq!(result.0[&name].identity.definition_revision, "v1");
            assert_eq!(
                result.0[&name].identity.grammar_version,
                "tree-sitter-python@0.25.0"
            );
        }
    }

    #[test]
    fn physical_newlines_empty_crlf_and_unicode() {
        counts("", [0, 0, 0, 0]);
        counts("\n", [1, 0, 0, 0]);
        counts("# α\r\n\r\nx = '💡'\r\n", [3, 1, 0, 0]);
        counts("x = 1", [1, 1, 0, 0]);
        counts("x = 1\n\n", [2, 1, 0, 0]);
    }

    #[test]
    fn compound_statements_and_wrappers_hand_counted() {
        // class, decorated def (once), if, return, return, expression assignment.
        counts("class A:\n    @decorate\n    def f(self, x):\n        if x:\n            return 1\n        else:\n            return 2\nx = 3; y = 4\n", [8, 7, 1, 1]);
        counts(
            "async def f(xs):\n    async for x in xs:\n        async with x:\n            pass\n",
            [4, 4, 1, 1],
        );
    }

    #[test]
    fn nested_functions_lambda_no_implicit_decision() {
        counts(
            "def outer():\n    def inner():\n        return lambda x: x\n    return inner\n",
            [4, 4, 0, 3],
        );
    }

    #[test]
    fn decisions_branches_handlers_comprehensions_and_cases() {
        counts("if a and b or c:\n    x = 1 if y else 2\nelif d:\n    pass\nwhile z:\n    break\nfor x in xs:\n    continue\ntry:\n    f()\nexcept ValueError:\n    raise\nexcept Exception:\n    pass\n", [14, 11, 9, 0]);
        counts(
            "xs = [x for x in a if x for y in b if y]\nys = (x for x in a if x)\n",
            [2, 2, 6, 0],
        );
        counts("match x:\n    case 1:\n        pass\n    case 2 | 3:\n        pass\n    case _:\n        pass\n", [7, 4, 2, 0]);
    }

    #[test]
    fn selected_spans_union_nodes_and_shared_lines() {
        let source = "α = 1; β = 2\r\ndef f():\r\n    return 3\r\n";
        let start = source.find("def").unwrap();
        let end = source.trim_end().len();
        let result = values(
            source.as_bytes(),
            &[
                span(start, end),
                span(start, end),
                span(source.find("return").unwrap(), end),
            ],
        );
        for (name, count) in [
            (MetricName::PhysicalLines, 2),
            (MetricName::LogicalStatements, 2),
            (MetricName::Functions, 1),
        ] {
            assert_eq!(
                result.0[&name].value,
                MetricValue::Available {
                    count: Count(count)
                }
            );
        }
        let result = values(source.as_bytes(), &[span(0, 6), span(8, 14)]);
        assert_eq!(
            result.0[&MetricName::PhysicalLines].value,
            MetricValue::Available { count: Count(1) }
        );
    }

    #[test]
    fn parse_and_encoding_errors_are_unavailable_not_zero() {
        for source in [b"def f(:\n  pass\n".as_slice(), b"x = (\n", b"\xff\n"] {
            let result = values(source, &[span(0, source.len())]);
            assert!(matches!(
                result.0[&MetricName::PhysicalLines].value,
                MetricValue::Available { .. }
            ));
            for name in [
                MetricName::LogicalStatements,
                MetricName::Decisions,
                MetricName::Functions,
            ] {
                assert!(
                    matches!(&result.0[&name].value, MetricValue::Unavailable { reason, resolved_lower_bound: None } if !reason.is_empty())
                );
            }
        }
        let result = values(b"x = 1", &[span(0, 99)]);
        assert!(result
            .0
            .values()
            .all(|m| matches!(m.value, MetricValue::Unavailable { .. })));
    }

    #[test]
    fn grammar_statement_table_covers_imports_bindings_and_suites() {
        // Six module statements, function + six body statements; clauses add none.
        counts("from __future__ import annotations\nimport os\nfrom x import y\nx: int = 1\nx += 1\ntype T = int\ndef f():\n    global x\n    assert x\n    del x\n    with open('x'):\n        pass\n    return None\n", [13, 13, 0, 1]);
        counts("def f():\n    x = 1\n    def g():\n        nonlocal x\n        return x\n    return g\n", [6, 6, 0, 2]);
        counts("try:\n    pass\nfinally:\n    pass\n", [4, 3, 0, 0]);
    }

    #[test]
    fn guarded_cases_and_boolean_chains_are_explicit() {
        // match + two passes; two non-default cases, one guard, three operators.
        counts("match x:\n    case _ if a and b and c or d:\n        pass\n    case captured:\n        pass\n", [5, 3, 6, 0]);
        counts(
            "x = {a: b for a, b in xs if a}\ny = {x for x in xs}\n",
            [2, 2, 3, 0],
        );
    }

    #[test]
    fn errors_outside_selected_scope_and_empty_selection() {
        let source = b"x = 1\ndef broken(:\n    pass\n";
        let result = values(source, &[span(0, 5)]);
        assert_eq!(result.0[&MetricName::LogicalStatements].value, available(1));
        let result = values(b"x = 1", &[]);
        assert!(result.0.values().all(|m| m.value == available(0)));
        let result = values("α = 1".as_bytes(), &[span(1, 6)]);
        assert_eq!(result.0[&MetricName::PhysicalLines].value, available(1));
        assert!(matches!(
            result.0[&MetricName::LogicalStatements].value,
            MetricValue::Unavailable { .. }
        ));
    }
}
