//! A data pack (`tables/<language>.json`) as the validator and engine read it.
//!
//! Keys the prototype reads with `table[...]` are required; keys it reads
//! with `.get(..., default)` default. A value of the wrong type fails loading
//! instead of being interpreted the way Python would (see the report).

use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Returned {
    pub of: Vec<String>,
    pub kinds: Vec<String>,
    pub tokens: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Rule {
    pub nodes: Vec<String>,
    pub nodes_with_prefix: BTreeMap<String, Vec<String>>,
    pub field_of: Vec<(String, String)>,
    pub requires_next_sibling: Value,
    pub call_statements: Vec<String>,
    pub returned: Option<Returned>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct BranchOps {
    pub template: Option<String>,
    pub exclude_kinds: Vec<String>,
    pub token_swaps: Vec<(String, String)>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct FailureOps {
    pub by_kind: BTreeMap<String, String>,
    pub returned_literal_swaps: Vec<(String, String)>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct WriteOps {
    pub statement: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct OrderOps {
    pub swap_next: Value,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Operators {
    pub branch_condition: BranchOps,
    pub failure_state: FailureOps,
    pub state_write: WriteOps,
    pub error_order: OrderOps,
}

#[derive(Debug, Deserialize)]
pub struct Pack {
    pub comment_kinds: Vec<String>,
    pub string_kinds: Vec<String>,
    #[serde(default)]
    pub unwrap_kinds: Vec<String>,
    #[serde(default)]
    pub test_attribute_markers: Vec<String>,
    pub kinds: BTreeMap<String, Rule>,
    #[serde(default)]
    pub noop_statement: Option<String>,
    #[serde(default)]
    pub statement_wrappers: Vec<String>,
    #[serde(default)]
    pub statement_terminator: Option<String>,
    #[serde(default)]
    pub operators: Operators,
}

impl Pack {
    /// `validator.load_table`: `<dir>/<language>.json`.
    pub fn load(dir: &Path, language: &str) -> Result<Pack, String> {
        let path = dir.join(format!("{language}.json"));
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read pack {}: {e}", path.display()))?;
        serde_json::from_str(&text)
            .map_err(|e| format!("pack {} is not usable: {e}", path.display()))
    }

    pub fn terminator(&self) -> &str {
        self.statement_terminator.as_deref().unwrap_or("")
    }

    pub fn is_comment(&self, kind: &str) -> bool {
        has(&self.comment_kinds, kind)
    }

    pub fn is_hidden(&self, kind: &str) -> bool {
        has(&self.comment_kinds, kind) || has(&self.string_kinds, kind)
    }
}

pub fn has(list: &[String], item: &str) -> bool {
    list.iter().any(|x| x == item)
}

/// Python truthiness of a JSON value.
pub fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64() != Some(0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

/// `engine._partners`: each token maps to its first partner, both ways.
pub fn partners(pairs: &[(String, String)]) -> BTreeMap<String, String> {
    let mut partner = BTreeMap::new();
    for (a, b) in pairs {
        partner.entry(a.clone()).or_insert_with(|| b.clone());
        partner.entry(b.clone()).or_insert_with(|| a.clone());
    }
    partner
}
