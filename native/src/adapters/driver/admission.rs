//! Admission of a language pack: deterministic checks before a pack is used.
//!
//! Checks, in this order: `json`, `size`, `shape`, `kinds_valid`, `templates`.
//! Every check is reported; one that cannot run because an earlier one failed
//! is reported as not run and keeps the pack from being admitted.

use super::pack::{Pack, PackError};
use super::tree::{grammar, kind_is_valid};
use super::Kind;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

pub const MAX_LINES: usize = 40;
pub const MAX_CHARS: usize = 2500;
const PLACEHOLDERS: [&str; 3] = ["text", "noop", "child0"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Check {
    pub name: &'static str,
    pub pass: bool,
    /// False when an earlier failure kept the check from running.
    pub ran: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Admission {
    pub receipt: &'static str,
    pub language: String,
    pub pack_sha256: String,
    pub admitted: bool,
    pub checks: Vec<Check>,
}

impl Admission {
    /// Names of the checks that ran and failed.
    pub fn failed(&self) -> Vec<&'static str> {
        let failed = self.checks.iter().filter(|c| c.ran && !c.pass);
        failed.map(|c| c.name).collect()
    }

    /// Lines for the author of the pack: every check that did not pass.
    pub fn feedback(&self) -> Vec<String> {
        let open = self.checks.iter().filter(|c| !c.pass);
        open.map(|c| format!("{}: {}", c.name, c.detail)).collect()
    }

    /// Record a check that ran: Ok with its detail when it passed.
    pub fn add(&mut self, name: &'static str, result: Result<String, String>) {
        let (pass, detail) = match result {
            Ok(detail) => (true, detail),
            Err(detail) => (false, detail),
        };
        self.checks.push(Check {
            name,
            pass,
            ran: true,
            detail,
        });
        self.admitted = self.checks.iter().all(|c| c.pass);
    }

    /// Record a check that could not run; the pack is then not admitted.
    pub fn not_run(&mut self, name: &'static str, why: &str) {
        self.checks.push(Check {
            name,
            pass: false,
            ran: false,
            detail: format!("not run: {why}"),
        });
        self.admitted = false;
    }
}

/// Check one pack text for `language`. An error means the language has no
/// grammar in this build, so nothing can be said about the pack.
pub fn admit(text: &str, language: &str) -> Result<Admission, PackError> {
    static SEEN: Mutex<BTreeMap<(String, String), Admission>> = Mutex::new(BTreeMap::new());
    let lang = grammar(language).map_err(|_| PackError::NotSupported(language.to_owned()))?;
    let digest = crate::specimen_runner::sha256_hex_bytes(text.as_bytes());
    let key = (digest.clone(), language.to_owned());
    let mut seen = SEEN.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(known) = seen.get(&key) {
        return Ok(known.clone());
    }
    let mut result = Admission {
        receipt: "pack-admission-v1",
        language: language.to_owned(),
        pack_sha256: digest,
        admitted: true,
        checks: Vec::new(),
    };
    let object = json_object(text);
    result.add("json", object.clone());
    result.add("size", size(text));
    let pack = match (object, Pack::parse(text)) {
        (Err(_), _) => Err("the text is not one JSON object"),
        (Ok(_), Err(e)) => {
            result.add("shape", Err(e.to_string()));
            Err("the pack does not have the pack shape")
        }
        (Ok(_), Ok(pack)) => {
            result.add("shape", shape(&pack, language));
            Ok(pack)
        }
    };
    match pack {
        Ok(pack) => {
            let valid = |kind: &str| kind_is_valid(lang, kind);
            result.add("kinds_valid", kinds_valid(&pack, language, valid));
            result.add("templates", templates(&pack));
        }
        Err(why) => {
            let done: BTreeSet<_> = result.checks.iter().map(|c| c.name).collect();
            for name in ["shape", "kinds_valid", "templates"] {
                if !done.contains(name) {
                    result.not_run(name, why);
                }
            }
        }
    }
    seen.insert(key, result.clone());
    Ok(result)
}

fn json_object(text: &str) -> Result<String, String> {
    match serde_json::from_str::<serde_json::Value>(text) {
        Ok(serde_json::Value::Object(_)) => Ok("one JSON object".into()),
        Ok(_) => Err("the JSON value is not an object".into()),
        Err(e) => Err(format!("not valid JSON: {e}")),
    }
}

/// Lines end at LF, CRLF or CR; a final line ending adds no line.
pub fn line_count(text: &str) -> usize {
    let breaks =
        text.matches('\n').count() + text.matches('\r').count() - text.matches("\r\n").count();
    let open_last_line = !text.is_empty() && !text.ends_with(['\n', '\r']);
    breaks + usize::from(open_last_line)
}

fn size(text: &str) -> Result<String, String> {
    let (lines, chars) = (line_count(text), text.chars().count());
    let detail =
        format!("{lines} lines (at most {MAX_LINES}), {chars} characters (at most {MAX_CHARS})");
    if lines <= MAX_LINES && chars <= MAX_CHARS {
        Ok(detail)
    } else {
        Err(detail)
    }
}

/// What typed loading does not already check.
fn shape(pack: &Pack, language: &str) -> Result<String, String> {
    let mut problems = Vec::new();
    if pack.language != language {
        problems.push(format!(
            "language is {:?}, expected {language:?}",
            pack.language
        ));
    }
    for kind in Kind::ALL {
        if pack.rule(kind).is_none() {
            problems.push(format!("missing kinds.{}", kind.name()));
        }
    }
    if problems.is_empty() {
        Ok("required keys present with the right types".into())
    } else {
        Err(problems.join("; "))
    }
}

/// Every node kind name the pack uses.
pub fn kind_names(pack: &Pack) -> BTreeSet<&str> {
    let lists = [
        &pack.comment_kinds,
        &pack.string_kinds,
        &pack.unwrap_kinds,
        &pack.statement_wrappers,
        &pack.test_attribute_kinds,
        &pack.operators.branch_condition.exclude_kinds,
    ];
    let mut names: BTreeSet<&str> = lists.into_iter().flatten().map(String::as_str).collect();
    for rule in pack.kinds.values() {
        names.extend(rule.nodes.iter().map(String::as_str));
        names.extend(rule.call_statements.iter().map(String::as_str));
        names.extend(rule.field_of.iter().map(|(kind, _)| kind.as_str()));
        names.extend(rule.nodes_with_prefix.keys().map(String::as_str));
        if let Some(returned) = &rule.returned {
            names.extend(returned.of.iter().map(String::as_str));
            names.extend(returned.kinds.iter().map(String::as_str));
        }
    }
    names.extend(
        pack.operators
            .failure_state
            .by_kind
            .keys()
            .map(String::as_str),
    );
    names
}

fn kinds_valid(
    pack: &Pack,
    language: &str,
    valid: impl Fn(&str) -> bool,
) -> Result<String, String> {
    let names = kind_names(pack);
    let invalid: Vec<_> = names.iter().copied().filter(|n| !valid(n)).collect();
    if invalid.is_empty() {
        Ok(format!(
            "{} node kinds, all valid for {language}",
            names.len()
        ))
    } else {
        Err(format!(
            "node kinds not in the {language} grammar: {}",
            invalid.join(", ")
        ))
    }
}

/// The names inside `{...}` in a template.
fn placeholders(template: &str) -> BTreeSet<&str> {
    let mut found = BTreeSet::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        rest = &rest[open + 1..];
        let word = rest
            .find(|c: char| !(c.is_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
        if word > 0 && rest[word..].starts_with('}') {
            found.insert(&rest[..word]);
        }
    }
    found
}

fn templates(pack: &Pack) -> Result<String, String> {
    let ops = &pack.operators;
    let mut problems = Vec::new();
    let mut found: Vec<(String, &str)> = Vec::new();
    match &ops.branch_condition.template {
        None => problems.push("operators.branch_condition.template is missing".to_owned()),
        Some(template) => {
            if !template.contains("{text}") {
                problems
                    .push("operators.branch_condition.template does not contain {text}".to_owned());
            }
            found.push(("operators.branch_condition.template".into(), template));
        }
    }
    for (kind, template) in &ops.failure_state.by_kind {
        found.push((format!("operators.failure_state.by_kind.{kind}"), template));
    }
    if let Some(statement) = &ops.state_write.statement {
        found.push(("operators.state_write.statement".into(), statement));
    }
    for (path, template) in &found {
        let unknown: Vec<_> = placeholders(template)
            .into_iter()
            .filter(|name| !PLACEHOLDERS.contains(name))
            .map(|name| format!("{{{name}}}"))
            .collect();
        if !unknown.is_empty() {
            problems.push(format!(
                "{path} uses unknown placeholders: {}",
                unknown.join(", ")
            ));
        }
    }
    if problems.is_empty() {
        Ok(format!(
            "{} templates, placeholders only {{text}}, {{noop}}, {{child0}}",
            found.len()
        ))
    } else {
        Err(problems.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::driver::registry::{shipped, shipped_texts};

    #[test]
    fn every_shipped_pack_is_admitted_against_the_compiled_grammars() {
        for (text, pack) in shipped_texts().iter().zip(shipped().unwrap()) {
            let result = admit(text, &pack.language).unwrap();
            assert!(
                result.admitted,
                "{}: {:?}",
                pack.language,
                result.feedback()
            );
            assert_eq!(result.checks.len(), 5);
            assert_eq!(result.pack_sha256, pack.digest);
        }
    }

    #[test]
    fn a_pack_for_another_language_fails_shape_and_kinds() {
        let packs = shipped().unwrap();
        let (text, other) = (shipped_texts()[0], &packs[1].language);
        let result = admit(text, other).unwrap();
        assert!(!result.admitted);
        assert!(
            result.failed().contains(&"shape"),
            "{:?}",
            result.feedback()
        );
    }

    #[test]
    fn text_that_is_not_an_object_stops_the_later_checks() {
        let language = &shipped().unwrap()[0].language;
        for text in ["[1, 2]", "{\"language\": "] {
            let result = admit(text, language).unwrap();
            assert_eq!(result.failed(), vec!["json"]);
            assert_eq!(result.checks.iter().filter(|c| !c.ran).count(), 3);
            assert!(!result.admitted);
        }
    }

    #[test]
    fn a_language_without_a_grammar_is_not_supported() {
        assert_eq!(
            admit("{}", "klingon").unwrap_err(),
            PackError::NotSupported("klingon".into())
        );
    }

    #[test]
    fn lines_and_placeholders_are_counted_as_documented() {
        assert_eq!(line_count(""), 0);
        assert_eq!(line_count("a"), 1);
        assert_eq!(line_count("a\r\nb\rc\n"), 3);
        assert_eq!(line_count("a\n\nb"), 3);
        let names = placeholders("!({text}) {} {a b} {child0}{x_1} {open");
        assert_eq!(
            names.into_iter().collect::<Vec<_>>(),
            ["child0", "text", "x_1"]
        );
    }
}
