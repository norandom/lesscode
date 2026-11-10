//! Control replay: recorded annotation cases that a pack must reproduce on a
//! source tree before it is admitted for it.
//!
//! A control file is `{"files": [...], "test_paths": [...], "cases":
//! [{"label", "annotations": [...], "expected": [...]}]}`. Each case is
//! validated on its own, in order; `expected` holds one outcome per
//! annotation (the reason, with `+anchor_completed` where it applies).

use super::annotations::{site_of, validate};
use super::operators::{classify, mutants, Change};
use super::pack::Pack;
use super::sites::OPERATOR_KINDS;
use super::source::SourceSet;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub label: String,
    pub annotations: Vec<Value>,
    pub expected: Vec<String>,
    /// When given, the exact operator labels the pack must offer at the site.
    #[serde(default)]
    pub operators: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlFile {
    /// Optional: the language the controls are written for.
    #[serde(default)]
    pub language: Option<String>,
    /// Optional: where the controls come from.
    #[serde(default)]
    pub note: Option<String>,
    pub files: Vec<String>,
    pub test_paths: Vec<String>,
    pub cases: Vec<Case>,
}

impl ControlFile {
    pub fn read(path: &Path) -> Result<ControlFile, String> {
        let bytes =
            std::fs::read(path).map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
        serde_json::from_slice(&bytes)
            .map_err(|e| format!("{} is not a control file: {e}", path.display()))
    }

    pub fn source(&self, root: &Path) -> SourceSet {
        SourceSet {
            root: root.to_path_buf(),
            files: self.files.clone(),
            test_paths: self.test_paths.clone(),
        }
    }
}

/// Pass or fail with the detail for the admission receipt.
pub type Outcome = Result<String, String>;

/// Every case must give its recorded outcomes.
pub fn replay(pack: &Pack, source: &SourceSet, cases: &[Case]) -> Outcome {
    let mut failing = Vec::new();
    for case in cases {
        let got: Vec<String> = match validate(source, pack, &case.annotations) {
            Ok(decisions) => decisions.iter().map(|d| d.outcome()).collect(),
            Err(e) => vec![format!("stopped: {e}")],
        };
        if got != case.expected {
            failing.push(format!(
                "{} expected {:?} got {got:?}",
                case.label, case.expected
            ));
        }
    }
    let detail = format!(
        "{} of {} controls behaved as expected",
        cases.len() - failing.len(),
        cases.len()
    );
    if cases.is_empty() {
        Err("the control file has no cases".into())
    } else if failing.is_empty() {
        Ok(detail)
    } else {
        Err(format!("{detail}; failing: {}", failing.join("; ")))
    }
}

/// What the pack's operators make of one accepted control annotation: the
/// outcome, and for a failing outcome what the author of the pack needs to
/// know.
fn seed_outcome(
    pack: &Pack,
    source: &SourceSet,
    case: &Case,
) -> Result<(&'static str, Option<String>), String> {
    let annotation = &case.annotations[0];
    let kind = annotation.get("kind").and_then(Value::as_str);
    if !OPERATOR_KINDS.iter().any(|k| Some(k.name()) == kind) {
        return Ok(("no_operator", None));
    }
    let Some(site) = site_of(source, pack, annotation)? else {
        let why = format!("{}: not a site the pack's operators can act on", case.label);
        return Ok(("no_site", Some(why)));
    };
    let text = source.raw(&site.file)?;
    let found = mutants(pack, &text, &site)?;
    if let Some(wanted) = &case.operators {
        let got: Vec<&String> = found.iter().map(|m| &m.operator).collect();
        if !got.iter().copied().eq(wanted) {
            let why = format!(
                "{}: expected operators {wanted:?} on a {}, got {got:?}",
                case.label, site.node_kind
            );
            return Ok(("operators_differ", Some(why)));
        }
    }
    let Some(mutant) = found.first() else {
        return Ok(("no_operator", None));
    };
    let change = classify(pack, &text, &mutant.text)?;
    let why = (change != Change::Changed).then(|| {
        format!(
            "{}: operator {} gives a text that is {}",
            case.label,
            mutant.operator,
            change.name()
        )
    });
    Ok((change.name(), why))
}

/// Every control that is one accepted annotation of a kind with operators
/// must yield a real, parseable change, or have no operator; a control that
/// lists operators must get exactly those.
pub fn seed_mutants(pack: &Pack, source: &SourceSet, cases: &[Case]) -> Outcome {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut failing = Vec::new();
    let accepted = |case: &&Case| {
        case.annotations.len() == 1 && case.expected.iter().all(|e| e.starts_with("ok"))
    };
    for case in cases.iter().filter(accepted) {
        let (outcome, why) = match seed_outcome(pack, source, case) {
            Ok((outcome, why)) => (outcome.to_owned(), why),
            Err(e) => ("error".to_owned(), Some(format!("{}: {e}", case.label))),
        };
        failing.extend(why);
        *counts.entry(outcome).or_default() += 1;
    }
    let total: usize = counts.values().sum();
    let listed: Vec<_> = counts.iter().map(|(k, n)| format!("{k} {n}")).collect();
    let detail = format!("{total} seeds: {}", listed.join(", "));
    if failing.is_empty() {
        Ok(detail)
    } else {
        Err(format!("{detail}; failing: {}", failing.join("; ")))
    }
}
