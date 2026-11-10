//! Port of the model-free admission checks of `pack_check.py`: json, size,
//! shape, kinds_valid and templates, in that order, each always reported.

use crate::tree::{grammar, kind_is_valid};
use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::BTreeSet;

const MAX_LINES: usize = 40;
const MAX_CHARS: usize = 2500;
const REQUIRED: [(&str, &str); 9] = [
    ("language", "string"),
    ("comment_kinds", "strings"),
    ("string_kinds", "strings"),
    ("unwrap_kinds", "strings"),
    ("test_attribute_markers", "strings"),
    ("kinds", "object"),
    ("noop_statement", "string"),
    ("statement_wrappers", "strings"),
    ("operators", "object"),
];
const OPTIONAL: [(&str, &str); 1] = [("statement_terminator", "string")];
const KIND_ORDER: [&str; 5] = [
    "branch_condition",
    "failure_state",
    "error_order",
    "state_write",
    "pure_shell_boundary",
];
const RULE_KEYS: [(&str, &str); 6] = [
    ("field_of", "pairs"),
    ("nodes", "strings"),
    ("nodes_with_prefix", "prefixes"),
    ("requires_next_sibling", "boolean"),
    ("call_statements", "strings"),
    ("returned", "returned"),
];
const OPERATOR_ENTRIES: [&str; 4] = [
    "branch_condition",
    "failure_state",
    "state_write",
    "error_order",
];
const OPERATOR_KEYS: [(&str, &str, &str); 3] = [
    ("branch_condition", "exclude_kinds", "strings"),
    ("branch_condition", "token_swaps", "swaps"),
    ("failure_state", "returned_literal_swaps", "swaps"),
];
const RETURNED_KEYS: [&str; 3] = ["of", "kinds", "tokens"];
const PLACEHOLDERS: [&str; 3] = ["text", "noop", "child0"];

#[derive(Debug, Clone, Serialize)]
pub struct Check {
    pub name: String,
    pub pass: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Admission {
    pub admitted: bool,
    pub checks: Vec<Check>,
}

fn describe(expected: &str) -> &'static str {
    match expected {
        "string" => "a string",
        "object" => "an object",
        "strings" => "a list of strings",
        "boolean" => "true or false",
        "pairs" => "a list of [kind, field] string pairs",
        "prefixes" => "an object of lists of strings",
        "swaps" => "a list of [a, b] string pairs",
        _ => "an object with \"of\", \"kinds\" and \"tokens\", each a list of strings",
    }
}

/// Lines as `bytes.splitlines` counts them: LF, CRLF or CR; a final break adds none.
pub fn line_count(text: &str) -> usize {
    let bytes = text.as_bytes();
    let (mut lines, mut i, mut open) = (0, 0, false);
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => (lines, open) = (lines + 1, false),
            b'\r' => {
                (lines, open) = (lines + 1, false);
                if bytes.get(i + 1) == Some(&b'\n') {
                    i += 1;
                }
            }
            _ => open = true,
        }
        i += 1;
    }
    lines + usize::from(open)
}

/// `json.dumps` of a string with the default `ensure_ascii`.
fn py_dumps(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            c if (c as u32) < 0x20 || (c as u32) > 0x7e => {
                let mut units = [0u16; 2];
                for unit in c.encode_utf16(&mut units) {
                    out.push_str(&format!("\\u{unit:04x}"));
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn parse_pack(text: &str) -> Result<Map<String, Value>, String> {
    match serde_json::from_str::<Value>(text) {
        Err(e) => Err(format!("not valid JSON: {e}")),
        Ok(Value::Object(map)) => Ok(map),
        Ok(other) => {
            let name = match other {
                Value::Array(_) => "an array",
                Value::String(_) => "a string",
                Value::Bool(_) => "a boolean",
                Value::Null => "null",
                _ => "a number",
            };
            Err(format!("the JSON value is {name}, not an object"))
        }
    }
}

fn is_strings(value: Option<&Value>) -> bool {
    matches!(value, Some(Value::Array(a)) if a.iter().all(Value::is_string))
}

fn type_ok(value: &Value, expected: &str) -> bool {
    match expected {
        "string" => value.is_string(),
        "object" => value.is_object(),
        "strings" => is_strings(Some(value)),
        "boolean" => value.is_boolean(),
        "pairs" | "swaps" => matches!(value, Value::Array(a)
            if a.iter().all(|p| matches!(p, Value::Array(x) if x.len() == 2) && is_strings(Some(p)))),
        "returned" => matches!(value, Value::Object(o)
            if RETURNED_KEYS.iter().all(|k| is_strings(o.get(*k)))),
        _ => matches!(value, Value::Object(o) if o.values().all(|x| is_strings(Some(x)))),
    }
}

/// (problems that keep the validator and engine from reading the pack, other problems).
fn shape_problems(pack: &Map<String, Value>, language: &str) -> (Vec<String>, Vec<String>) {
    let (mut structural, mut other) = (Vec::new(), Vec::new());
    for (key, expected) in REQUIRED {
        match pack.get(key) {
            None => structural.push(format!("missing key {key}")),
            Some(v) if !type_ok(v, expected) => {
                structural.push(format!("{key} must be {}", describe(expected)))
            }
            _ => {}
        }
    }
    for (key, expected) in OPTIONAL {
        if pack.get(key).is_some_and(|v| !type_ok(v, expected)) {
            structural.push(format!("{key} must be {}", describe(expected)));
        }
    }
    if let Some(Value::String(found)) = pack.get("language") {
        if found != language {
            other.push(format!(
                "language is {}, expected {}",
                py_dumps(found),
                py_dumps(language)
            ));
        }
    }
    if let Some(Value::Object(kinds)) = pack.get("kinds") {
        for kind in KIND_ORDER {
            match kinds.get(kind) {
                None => structural.push(format!("missing kinds.{kind}")),
                Some(Value::Object(rule)) => {
                    for (key, expected) in RULE_KEYS {
                        if rule.get(key).is_some_and(|v| !type_ok(v, expected)) {
                            structural
                                .push(format!("kinds.{kind}.{key} must be {}", describe(expected)));
                        }
                    }
                }
                Some(_) => structural.push(format!("kinds.{kind} must be an object")),
            }
        }
    }
    if let Some(Value::Object(ops)) = pack.get("operators") {
        for name in OPERATOR_ENTRIES {
            if ops.get(name).is_some_and(|v| !v.is_object()) {
                structural.push(format!("operators.{name} must be an object"));
            }
        }
        for (name, key, expected) in OPERATOR_KEYS {
            if let Some(Value::Object(entry)) = ops.get(name) {
                if entry.get(key).is_some_and(|v| !type_ok(v, expected)) {
                    structural.push(format!(
                        "operators.{name}.{key} must be {}",
                        describe(expected)
                    ));
                }
            }
        }
        if let Some(Value::Object(failure)) = ops.get("failure_state") {
            if failure.get("by_kind").is_some_and(|v| !v.is_object()) {
                structural.push("operators.failure_state.by_kind must be an object".to_owned());
            }
        }
    }
    let known = |k: &String| REQUIRED.iter().chain(OPTIONAL.iter()).any(|(r, _)| r == k);
    let unknown: Vec<&str> = pack
        .keys()
        .filter(|k| !known(k))
        .map(String::as_str)
        .collect();
    if !unknown.is_empty() {
        other.push(format!("unknown top-level keys: {}", unknown.join(", ")));
    }
    (structural, other)
}

fn strings(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(|x| x.as_str().map(str::to_owned))
            .collect(),
        _ => Vec::new(),
    }
}

fn entry<'a>(mapping: Option<&'a Value>, key: &str) -> Option<&'a Map<String, Value>> {
    mapping?.as_object()?.get(key)?.as_object()
}

/// Every node kind name the pack uses, read defensively.
fn kind_names(pack: &Map<String, Value>) -> BTreeSet<String> {
    let mut names: BTreeSet<String> = [
        "comment_kinds",
        "string_kinds",
        "unwrap_kinds",
        "statement_wrappers",
    ]
    .iter()
    .flat_map(|key| strings(pack.get(*key)))
    .collect();
    if let Some(Value::Object(kinds)) = pack.get("kinds") {
        for rule in kinds.values().filter(|r| r.is_object()) {
            names.extend(strings(rule.get("nodes")));
            if let Some(Value::Array(pairs)) = rule.get("field_of") {
                let firsts = pairs.iter().filter_map(|p| p.as_array()?.first()?.as_str());
                names.extend(firsts.map(str::to_owned));
            }
            names.extend(
                entry(Some(rule), "nodes_with_prefix")
                    .into_iter()
                    .flat_map(|m| m.keys().cloned()),
            );
            names.extend(strings(rule.get("call_statements")));
            let returned = entry(Some(rule), "returned");
            names.extend(strings(returned.and_then(|r| r.get("of"))));
            names.extend(strings(returned.and_then(|r| r.get("kinds"))));
        }
    }
    let ops = pack.get("operators");
    names.extend(strings(
        entry(ops, "branch_condition").and_then(|b| b.get("exclude_kinds")),
    ));
    let failure = entry(ops, "failure_state").map(|f| Value::Object(f.clone()));
    names.extend(
        entry(failure.as_ref(), "by_kind")
            .into_iter()
            .flat_map(|m| m.keys().cloned()),
    );
    names
}

fn kinds_valid(pack: &Map<String, Value>, language: &str) -> (bool, String) {
    let names = kind_names(pack);
    let Ok(lang) = grammar(language) else {
        return (
            false,
            format!("ast-grep has no grammar named {}", py_dumps(language)),
        );
    };
    let invalid: Vec<&str> = names
        .iter()
        .filter(|n| !kind_is_valid(lang, n))
        .map(String::as_str)
        .collect();
    if !invalid.is_empty() {
        return (
            false,
            format!(
                "node kinds not in the {language} grammar: {}",
                invalid.join(", ")
            ),
        );
    }
    (
        true,
        format!("{} node kinds, all valid for {language}", names.len()),
    )
}

/// `{name}` placeholders with `\w+` names, as `engine._fill` substitutes them.
fn placeholders(template: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        let after = &rest[open + 1..];
        let len = after
            .find(|c: char| !(c.is_alphanumeric() || c == '_'))
            .unwrap_or(after.len());
        if len > 0 && after[len..].starts_with('}') {
            found.push(&after[..len]);
            rest = &after[len + 1..];
        } else {
            rest = after;
        }
    }
    found
}

fn templates(pack: &Map<String, Value>) -> (bool, String) {
    let ops = pack.get("operators");
    let branch = entry(ops, "branch_condition");
    let failure = entry(ops, "failure_state").map(|f| Value::Object(f.clone()));
    let write = entry(ops, "state_write");
    let mut found: Vec<(String, &Value)> = Vec::new();
    let mut problems = Vec::new();
    match branch.and_then(|b| b.get("template")) {
        None => problems.push("operators.branch_condition.template is missing".to_owned()),
        Some(t) => {
            if t.as_str().is_some_and(|s| !s.contains("{text}")) {
                problems
                    .push("operators.branch_condition.template does not contain {text}".to_owned());
            }
            found.push(("operators.branch_condition.template".to_owned(), t));
        }
    }
    for (kind, t) in entry(failure.as_ref(), "by_kind").into_iter().flatten() {
        found.push((format!("operators.failure_state.by_kind.{kind}"), t));
    }
    if let Some(t) = write.and_then(|w| w.get("statement")) {
        found.push(("operators.state_write.statement".to_owned(), t));
    }
    for (path, template) in &found {
        let Some(template) = template.as_str() else {
            problems.push(format!("{path} is not a string"));
            continue;
        };
        let unknown: BTreeSet<&str> = placeholders(template)
            .into_iter()
            .filter(|p| !PLACEHOLDERS.contains(p))
            .collect();
        if !unknown.is_empty() {
            let listed: Vec<String> = unknown.iter().map(|u| format!("{{{u}}}")).collect();
            problems.push(format!(
                "{path} uses unknown placeholders: {}",
                listed.join(", ")
            ));
        }
    }
    if !problems.is_empty() {
        return (false, problems.join("; "));
    }
    (
        true,
        format!(
            "{} templates, placeholders only {{text}}, {{noop}}, {{child0}}",
            found.len()
        ),
    )
}

/// `pack_check.check_pack` without a scope: the model-free checks.
pub fn check_pack(text: &str, language: &str) -> Admission {
    let mut checks = Vec::new();
    let mut add = |name: &str, pass: bool, detail: String| {
        checks.push(Check {
            name: name.to_owned(),
            pass,
            detail,
        })
    };
    let parsed = parse_pack(text);
    let json_detail = parsed
        .as_ref()
        .err()
        .cloned()
        .unwrap_or_else(|| "one JSON object".to_owned());
    add("json", parsed.is_ok(), json_detail);
    let (lines, chars) = (line_count(text), text.chars().count());
    add(
        "size",
        lines <= MAX_LINES && chars <= MAX_CHARS,
        format!("{lines} lines (at most {MAX_LINES}), {chars} characters (at most {MAX_CHARS})"),
    );
    match parsed {
        Err(_) => {
            for name in ["shape", "kinds_valid", "templates"] {
                add(
                    name,
                    false,
                    "not run: the text is not one JSON object".to_owned(),
                );
            }
        }
        Ok(pack) => {
            let (structural, other) = shape_problems(&pack, language);
            let problems = [structural, other].concat();
            let detail = if problems.is_empty() {
                "required keys present with the right types".to_owned()
            } else {
                problems.join("; ")
            };
            add("shape", problems.is_empty(), detail);
            let (ok, detail) = kinds_valid(&pack, language);
            add("kinds_valid", ok, detail);
            let (ok, detail) = templates(&pack);
            add("templates", ok, detail);
        }
    }
    let admitted = checks.iter().all(|c| c.pass);
    Admission { admitted, checks }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_count_matches_bytes_splitlines() {
        assert_eq!(line_count(""), 0);
        assert_eq!(line_count("a"), 1);
        assert_eq!(line_count("a\r\nb\rc\n"), 3);
        assert_eq!(line_count("\n\n"), 2);
    }

    #[test]
    fn placeholders_follow_the_regex() {
        assert_eq!(
            placeholders("{{text}} {noop} { x} {child0"),
            vec!["text", "noop"]
        );
    }
}
