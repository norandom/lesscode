//! The target file of a bounded comparison: what it names, what is refused,
//! the cases of its ranges and the SMT-LIB text asked of the solver.

use super::solver::{bindings, Solver, Term};
use crate::specimen_runner::sha256_hex_bytes;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Pinned {
    pub(super) path: PathBuf,
    pub(super) sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Identity {
    /// The file that must have been loaded; any file when absent.
    pub(super) path: Option<PathBuf>,
    pub(super) sha256: String,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub(super) struct Symbol {
    pub(super) name: String,
    pub(super) min: i64,
    pub(super) max: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Arm {
    pub(super) label: String,
    /// SMT-LIB term over the symbols: what this version does, as modelled.
    pub(super) expression: String,
    /// The command that runs the real code on `{inputs}` and writes
    /// `{output}`; `{out}` is the output directory.
    pub(super) command: Vec<String>,
    /// Keys of the command's output that name a loaded file and its hash.
    #[serde(default)]
    pub(super) identities: BTreeMap<String, Identity>,
}

fn default_sort() -> String {
    "Bool".into()
}

fn default_timeout() -> u64 {
    60
}

fn default_max_cases() -> usize {
    1024
}

/// The target file. Relative paths are relative to its directory.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Target {
    pub(super) solver: Pinned,
    /// Files that must have these hashes before anything runs.
    #[serde(default)]
    pub(super) pins: Vec<Pinned>,
    pub(super) symbols: Vec<Symbol>,
    /// The sort of the arms' expressions.
    #[serde(default = "default_sort")]
    pub(super) sort: String,
    pub(super) arms: Vec<Arm>,
    /// Pairs of arm labels to compare.
    pub(super) compare: Vec<(String, String)>,
    /// One input case. In its strings `{name}` stands for the value of a
    /// symbol and `{id}` for the number of the case; a string that is
    /// nothing else becomes that number.
    pub(super) case: Value,
    #[serde(default = "default_timeout")]
    pub(super) timeout_seconds: u64,
    #[serde(default = "default_max_cases")]
    pub(super) max_cases: usize,
}

pub(super) fn plain(word: &str) -> bool {
    let mut chars = word.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

pub(super) fn literal(value: i64) -> String {
    match value {
        v if v < 0 => format!("(- {})", v.unsigned_abs()),
        v => v.to_string(),
    }
}

pub(super) fn integer(text: &str) -> Option<i64> {
    let negative = text.strip_prefix("(- ").and_then(|t| t.strip_suffix(')'));
    match negative {
        Some(digits) => digits.parse::<i64>().ok().map(|v| -v),
        None => text.parse().ok(),
    }
}

pub(super) fn fill(template: &Value, values: &BTreeMap<&str, i64>) -> Value {
    match template {
        Value::String(text) => {
            let whole = values
                .iter()
                .find(|(name, _)| *text == format!("{{{name}}}"));
            if let Some((_, value)) = whole {
                return json!(value);
            }
            let replace = |text: String, (name, value): (&&str, &i64)| {
                text.replace(&format!("{{{name}}}"), &value.to_string())
            };
            Value::String(values.iter().fold(text.clone(), replace))
        }
        Value::Array(items) => items.iter().map(|item| fill(item, values)).collect(),
        Value::Object(map) => {
            let filled = map.iter().map(|(k, v)| (k.clone(), fill(v, values)));
            Value::Object(filled.collect())
        }
        other => other.clone(),
    }
}

impl Target {
    pub(super) fn read(path: &Path) -> Result<(Target, String), String> {
        let bytes = fs::read(path).map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
        let mut target: Target = serde_json::from_slice(&bytes)
            .map_err(|e| format!("{} is not a target file: {e}", path.display()))?;
        let base = path.parent().unwrap_or(Path::new("."));
        target.solver.path = base.join(&target.solver.path);
        for file in &mut target.pins {
            file.path = base.join(&file.path);
        }
        let identities = target
            .arms
            .iter_mut()
            .flat_map(|a| a.identities.values_mut());
        for path in identities.filter_map(|identity| identity.path.as_mut()) {
            *path = base.join(&*path);
        }
        target.check()?;
        Ok((target, sha256_hex_bytes(&bytes)))
    }

    pub(super) fn arm(&self, label: &str) -> Result<&Arm, String> {
        let found = self.arms.iter().find(|arm| arm.label == label);
        found.ok_or_else(|| format!("no arm is called {label}"))
    }

    pub(super) fn check(&self) -> Result<(), String> {
        let words = self.symbols.iter().map(|s| &s.name);
        let words = words.chain(self.arms.iter().map(|a| &a.label));
        let mut seen = std::collections::BTreeSet::new();
        for word in words {
            if !plain(word) || word == "id" || !seen.insert(word) {
                return Err(format!("{word}: not a plain name, or used twice"));
            }
        }
        for (left, right) in &self.compare {
            self.arm(left).and(self.arm(right))?;
        }
        let mut sizes = self.symbols.iter().map(|s| s.max.checked_sub(s.min));
        let count = sizes.try_fold(1usize, |count, size| {
            let size = usize::try_from(size?).ok()?.checked_add(1)?;
            count.checked_mul(size).filter(|n| *n <= self.max_cases)
        });
        match count {
            Some(_) if !self.compare.is_empty() => Ok(()),
            Some(_) => Err("compare names no pair of arms".into()),
            None => Err(format!(
                "a range is empty or the ranges give more than {} cases",
                self.max_cases
            )),
        }
    }

    /// Every assignment of the ranges. The first symbol changes slowest.
    pub(super) fn cases(&self) -> Vec<Vec<i64>> {
        let mut cases = vec![Vec::new()];
        for symbol in &self.symbols {
            let extend = |case: &Vec<i64>| {
                let with = |value| [case.as_slice(), &[value]].concat();
                (symbol.min..=symbol.max).map(with).collect::<Vec<_>>()
            };
            cases = cases.iter().flat_map(extend).collect();
        }
        cases
    }

    pub(super) fn names(&self) -> Vec<String> {
        self.symbols.iter().map(|s| s.name.clone()).collect()
    }

    pub(super) fn functions(&self) -> Vec<String> {
        let name = |arm: &Arm| format!("arm_{}", arm.label);
        self.arms.iter().map(name).collect()
    }

    /// Declarations of the symbols and one function per arm.
    pub(super) fn preamble(&self) -> String {
        let mut text = String::from("(set-logic QF_LIA)\n");
        for symbol in &self.symbols {
            text += &format!("(declare-const {} Int)\n", symbol.name);
        }
        for arm in &self.arms {
            let (label, sort) = (&arm.label, &self.sort);
            text += &format!("(define-fun arm_{label} () {sort} {})\n", arm.expression);
        }
        text
    }

    /// Can the two arms differ inside the ranges?
    pub(super) fn query(&self, left: &str, right: &str) -> String {
        let mut text = self.preamble();
        for Symbol { name, min, max } in &self.symbols {
            let (min, max) = (literal(*min), literal(*max));
            text += &format!("(assert (and (<= {min} {name}) (<= {name} {max})))\n");
        }
        text + &format!("(assert (distinct arm_{left} arm_{right}))\n(check-sat)\n")
    }

    /// What each arm's expression gives on each case, by arm function.
    pub(super) fn modeled(
        &self,
        solver: &Solver,
        cases: &[Vec<i64>],
    ) -> Result<Vec<BTreeMap<String, String>>, String> {
        let functions = self.functions();
        let mut text = self.preamble();
        for case in cases {
            text += "(push)\n";
            for (symbol, value) in self.symbols.iter().zip(case) {
                text += &format!("(assert (= {} {}))\n", symbol.name, literal(*value));
            }
            text += &format!(
                "(check-sat)\n(get-value ({}))\n(pop)\n",
                functions.join(" ")
            );
        }
        let answer = solver.ask("census", &text)?;
        let per_case = answer.chunks(2).map(|pair| match pair {
            [Term::Word(sat), values] if sat == "sat" => bindings(values, &functions),
            _ => Err("the solver did not evaluate a case".to_owned()),
        });
        let values: Vec<_> = per_case.collect::<Result<_, _>>()?;
        match values.len() == cases.len() {
            true => Ok(values),
            false => Err("the solver evaluated fewer cases than asked".into()),
        }
    }

    pub(super) fn input(&self, id: usize, case: &[i64]) -> Value {
        let symbols = self.symbols.iter().map(|s| s.name.as_str());
        let mut values: BTreeMap<&str, i64> = symbols.zip(case.iter().copied()).collect();
        values.insert("id", id as i64);
        fill(&self.case, &values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TARGET: &str = r#"{"solver": {"path": "z3", "sha256": "0"},
        "symbols": [{"name": "a", "min": -1, "max": 0}, {"name": "b", "min": 0, "max": 2}],
        "arms": [{"label": "old", "expression": "(= a b)", "command": []},
                 {"label": "new", "expression": "(not (distinct a b))", "command": []}],
        "compare": [["old", "new"]], "case": {}}"#;

    fn target(text: &str) -> Result<Target, String> {
        let target: Target = serde_json::from_str(text).map_err(|e| e.to_string())?;
        target.check().map(|()| target)
    }

    #[test]
    fn cases_cover_the_ranges_and_the_query_bounds_every_symbol() {
        let target = target(TARGET).unwrap();
        let cases = target.cases();
        assert_eq!(cases.len(), 6);
        assert_eq!(cases[0], [-1, 0]);
        assert_eq!(cases[5], [0, 2]);
        let query = target.query("old", "new");
        assert!(query.contains("(define-fun arm_new () Bool (not (distinct a b)))"));
        assert!(query.contains("(assert (and (<= (- 1) a) (<= a 0)))"));
        assert!(query.ends_with("(assert (distinct arm_old arm_new))\n(check-sat)\n"));
    }

    #[test]
    fn a_target_with_wrong_names_pairs_or_ranges_is_refused() {
        for (from, to) in [
            ("\"name\": \"b\"", "\"name\": \"a\""),
            ("\"name\": \"b\"", "\"name\": \"id\""),
            ("\"label\": \"new\"", "\"label\": \"new arm\""),
            ("[\"old\", \"new\"]", "[\"old\", \"other\"]"),
            ("[[\"old\", \"new\"]]", "[]"),
            ("\"max\": 2", "\"max\": -1"),
            ("\"max\": 2", "\"max\": 2000"),
            ("\"case\": {}", "\"case\": {}, \"extra\": 1"),
        ] {
            assert!(target(&TARGET.replace(from, to)).is_err(), "{to}");
        }
    }

    #[test]
    fn a_case_template_takes_numbers_and_text_from_the_symbols() {
        let values = BTreeMap::from([("a", -1), ("id", 7)]);
        let filled = fill(&json!({"id": "{id}", "row": ["d{a}", 2, "{a}"]}), &values);
        assert_eq!(filled, json!({"id": 7, "row": ["d-1", 2, -1]}));
    }

    #[test]
    fn negative_numbers_are_written_and_read_as_the_solver_does() {
        assert_eq!(literal(-3), "(- 3)");
        assert_eq!(integer("(- 3)"), Some(-3));
        assert_eq!(integer("12"), Some(12));
        assert_eq!(integer("true"), None);
    }
}
