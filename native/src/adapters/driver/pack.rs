//! The typed form of a language pack (format version 1).
//!
//! Deserialising is the shape check: an unknown key, a missing required key
//! or a value of the wrong type fails loading.

use super::Kind;
use crate::specimen_runner::sha256_hex_bytes;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackError {
    /// No pack exists for the language.
    NotSupported(String),
    /// A pack exists but cannot be read or does not have the pack shape.
    Invalid(String),
}

impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackError::NotSupported(language) => write!(f, "language not supported: {language}"),
            PackError::Invalid(reason) => write!(f, "pack is not usable: {reason}"),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Returned {
    pub of: Vec<String>,
    pub kinds: Vec<String>,
    pub tokens: Vec<String>,
}

/// How sites of one kind are recognised.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Rule {
    pub nodes: Vec<String>,
    pub nodes_with_prefix: BTreeMap<String, Vec<String>>,
    pub field_of: Vec<(String, String)>,
    pub requires_next_sibling: bool,
    pub call_statements: Vec<String>,
    pub returned: Option<Returned>,
}

/// A construct the analysis cannot follow, reported when it occurs.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Gap {
    pub reason: String,
    pub nodes: Vec<String>,
    pub nodes_with_prefix: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BranchOps {
    pub template: Option<String>,
    pub exclude_kinds: Vec<String>,
    pub token_swaps: Vec<(String, String)>,
    /// Token pairs whose swap says the same as negating the whole condition.
    pub negation_swaps: Vec<(String, String)>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FailureOps {
    pub by_kind: BTreeMap<String, String>,
    pub returned_literal_swaps: Vec<(String, String)>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WriteOps {
    pub statement: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OrderOps {
    pub swap_next: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Operators {
    pub branch_condition: BranchOps,
    pub failure_state: FailureOps,
    pub state_write: WriteOps,
    pub error_order: OrderOps,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pack {
    /// The grammar name the pack is written for.
    pub language: String,
    pub comment_kinds: Vec<String>,
    pub string_kinds: Vec<String>,
    pub unwrap_kinds: Vec<String>,
    pub test_attribute_markers: Vec<String>,
    /// Node kinds that can carry a test marker in front of an item.
    #[serde(default)]
    pub test_attribute_kinds: Vec<String>,
    pub kinds: BTreeMap<Kind, Rule>,
    pub noop_statement: String,
    pub statement_wrappers: Vec<String>,
    #[serde(default)]
    pub statement_terminator: Option<String>,
    pub operators: Operators,
    /// Constructs the analysis reports as gaps.
    #[serde(default)]
    pub gaps: Vec<Gap>,
    /// SHA-256 of the pack text; set on loading, never read from the pack.
    #[serde(skip)]
    pub digest: String,
}

impl Pack {
    pub fn parse(text: &str) -> Result<Pack, PackError> {
        let mut pack: Pack =
            serde_json::from_str(text).map_err(|e| PackError::Invalid(e.to_string()))?;
        pack.digest = sha256_hex_bytes(text.as_bytes());
        Ok(pack)
    }

    /// The pack `<dir>/<language>.json`, which must name that language.
    pub fn from_dir(dir: &Path, language: &str) -> Result<Pack, PackError> {
        let plain = !language.is_empty()
            && language
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
        let path = dir.join(format!("{language}.json"));
        if !plain || !path.is_file() {
            return Err(PackError::NotSupported(language.to_owned()));
        }
        let text = std::fs::read_to_string(&path)
            .map_err(|e| PackError::Invalid(format!("{}: {e}", path.display())))?;
        let pack = Pack::parse(&text)?;
        if pack.language != language {
            return Err(PackError::Invalid(format!(
                "{} is written for {}, not {language}",
                path.display(),
                pack.language
            )));
        }
        Ok(pack)
    }

    pub fn rule(&self, kind: Kind) -> Option<&Rule> {
        self.kinds.get(&kind)
    }

    pub fn terminator(&self) -> &str {
        self.statement_terminator.as_deref().unwrap_or("")
    }

    pub fn is_comment(&self, kind: &str) -> bool {
        has(&self.comment_kinds, kind)
    }

    /// Comments and strings: text in them is never a site.
    pub fn is_hidden(&self, kind: &str) -> bool {
        has(&self.comment_kinds, kind) || has(&self.string_kinds, kind)
    }
}

pub fn has(list: &[String], item: &str) -> bool {
    list.iter().any(|x| x == item)
}

/// Each token maps to its first partner, in both directions.
pub fn partners(pairs: &[(String, String)]) -> BTreeMap<String, String> {
    let mut partner = BTreeMap::new();
    for (a, b) in pairs {
        partner.entry(a.clone()).or_insert_with(|| b.clone());
        partner.entry(b.clone()).or_insert_with(|| a.clone());
    }
    partner
}

#[cfg(test)]
mod tests {
    use super::*;

    const SMALL: &str = r#"{"language": "x", "comment_kinds": ["c"], "string_kinds": [],
        "unwrap_kinds": [], "test_attribute_markers": [], "noop_statement": ";",
        "statement_wrappers": [], "operators": {},
        "kinds": {"branch_condition": {"nodes": ["n"], "requires_next_sibling": true}}}"#;

    #[test]
    fn a_pack_parses_with_defaults_and_a_digest() {
        let pack = Pack::parse(SMALL).unwrap();
        assert!(
            pack.rule(Kind::BranchCondition)
                .unwrap()
                .requires_next_sibling
        );
        assert!(pack.rule(Kind::StateWrite).is_none());
        assert_eq!(pack.terminator(), "");
        assert!(pack.is_hidden("c") && !pack.is_hidden("n"));
        assert_eq!(pack.digest.len(), 64);
    }

    #[test]
    fn wrong_shapes_are_rejected() {
        for (bad, why) in [
            (
                SMALL.replace("\"nodes\": [\"n\"]", "\"nodes\": \"n\""),
                "string for a list",
            ),
            (
                SMALL.replace("branch_condition", "branch"),
                "unknown site kind",
            ),
            (
                SMALL.replace("\"string_kinds\": [],", ""),
                "missing required key",
            ),
            (
                SMALL.replace("\"language\"", "\"extra\": 1, \"language\""),
                "unknown key",
            ),
            (SMALL.replace("true", "1"), "number for a flag"),
        ] {
            assert!(
                matches!(Pack::parse(&bad), Err(PackError::Invalid(_))),
                "{why}"
            );
        }
    }

    #[test]
    fn partners_keep_the_first_pairing() {
        let pairs = [
            ("<".to_owned(), "<=".to_owned()),
            ("<".to_owned(), ">".to_owned()),
        ];
        let map = partners(&pairs);
        assert_eq!(map["<"], "<=");
        assert_eq!(map["<="], "<");
        assert_eq!(map[">"], "<");
    }
}
