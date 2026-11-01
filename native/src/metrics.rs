//! Versioned native syntax measurements; these are not Radon LLOC or Lizard CCN.

use crate::model::{
    Count, Language, MetricIdentity, MetricMeasurement, MetricName, MetricValue, Records,
    SchemaVersion,
};
use crate::scope::SourceSpan;
use std::collections::BTreeMap;

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

pub fn measure_python(
    path: &str,
    source: &[u8],
    scope_identity: &str,
    spans: &[SourceSpan],
) -> Records<MetricName, MetricMeasurement> {
    let measured = python_values(path, source, spans);
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
                        language: Language::Python,
                        grammar_version: "tree-sitter-python@0.25.0".into(),
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
fn python_values(path: &str, source: &[u8], spans: &[SourceSpan]) -> [MetricValue; 4] {
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
    let tree = crate::AstGrep::new(text, crate::Python);
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
        let wrapped = matches!(kind.as_ref(), "function_definition" | "class_definition")
            && node.parent().is_some_and(|p| {
                p.kind() == "decorated_definition" && selected(p.range().start, p.range().end)
            });
        let statement = PYTHON_STATEMENT_KINDS.contains(&kind.as_ref()) && !wrapped;
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
        let decision = PYTHON_DECISION_KINDS.contains(&kind.as_ref())
            || (kind == "case_clause" && !default_case);
        for (count, increment) in counts.iter_mut().zip([
            statement,
            decision,
            PYTHON_FUNCTION_KINDS.contains(&kind.as_ref()),
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
