//! `stage.json`: what a stage of `step protect` began from. The plan and
//! its hash, the commit, the test command, and per chosen function its
//! counts and every fault the tests did not notice, each with an id that
//! does not depend on the run. `step protect begin` writes it; the result
//! of the stage and the stage spec are read against it.

use super::functions::FileFunctions;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const RECEIPT: &str = "step-stage-v1";

/// A function of a file, as the plan names it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Named {
    pub file: String,
    pub function: String,
}

impl std::fmt::Display for Named {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.file, self.function)
    }
}

/// One fault the plan's run planted, and what the tests made of it.
#[derive(Debug, Clone)]
pub struct Planted {
    pub at: (usize, usize),
    pub operator: String,
    pub text: String,
    pub class: String,
    pub seconds: f64,
}

impl Planted {
    /// `<operator>@<line>:<column>` within the file. An operator that names
    /// the place it changed already is the id as it stands.
    pub fn id(&self) -> String {
        match self.operator.contains('@') {
            true => self.operator.clone(),
            false => format!("{}@{}:{}", self.operator, self.at.0, self.at.1),
        }
    }
}

/// The faults of a plan's run in one file, per function they belong to.
pub fn planted(
    functions: &FileFunctions,
    file: &str,
    run: &Value,
) -> BTreeMap<String, Vec<Planted>> {
    let number = |v: &Value| v.as_u64().unwrap_or(0) as usize;
    let text = |v: &Value| v.as_str().unwrap_or_default().to_owned();
    let mutants = run["mutants"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    let mut table: BTreeMap<String, Vec<Planted>> = BTreeMap::new();
    for m in mutants.iter().filter(|m| m["site"]["file"] == file) {
        let site = &m["site"];
        let at = (number(&site["start_line"]), number(&site["start_column"]));
        let owner = functions.owner(at).to_owned();
        table.entry(owner).or_default().push(Planted {
            at,
            operator: text(&m["operator"]),
            text: text(&site["text"]),
            class: text(&m["class"]),
            seconds: m["seconds"].as_f64().unwrap_or(0.0),
        });
    }
    table
}

/// Seconds to one decimal place.
fn tenths(seconds: f64) -> f64 {
    (seconds * 10.0).round() / 10.0
}

/// A fault the tests of the plan did not notice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fault {
    pub id: String,
    pub line: usize,
    pub column: usize,
    pub operator: String,
    /// The code the fault was planted in.
    pub text: String,
    /// How long its run took in the plan.
    pub seconds: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReachesOut {
    pub calls: Vec<String>,
    pub state_writes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chosen {
    pub file: String,
    pub function: String,
    pub noticed: usize,
    pub unnoticed: usize,
    /// Faults that did not build, ran over their time or have no class.
    pub other: usize,
    /// What the plan says the function reaches out to, where it says so.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reaches_out: Option<ReachesOut>,
    pub faults: Vec<Fault>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeftOut {
    pub file: String,
    pub function: String,
    pub why: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanRef {
    pub file: String,
    pub sha256: String,
}

/// What the base run and the check each run again.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunAgain {
    pub faults: usize,
    /// The seconds those faults took in the plan, one after another.
    pub seconds: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stage {
    pub receipt: String,
    pub spec: String,
    /// The commit the stage starts from.
    pub base: String,
    pub plan: PlanRef,
    pub test: Vec<String>,
    pub chosen: Vec<Chosen>,
    /// Unprotected functions of the plan that were not chosen.
    pub left_alone: Vec<Named>,
    pub left_out: Vec<LeftOut>,
    pub to_run_again: RunAgain,
}

pub const NOT_BUILT: &str = "not built on this system";

/// The row of a chosen function: the plan's counts, and its unnoticed
/// faults from the run in the order of the code.
pub fn chosen(row: &Value, one: &Named, planted: &[Planted]) -> Chosen {
    let count = |key: &str| row[key].as_u64().unwrap_or(0) as usize;
    let mut unnoticed: Vec<&Planted> = planted.iter().filter(|p| p.class == "survived").collect();
    unnoticed.sort_by_key(|p| p.at);
    let fault = |p: &Planted| Fault {
        id: p.id(),
        line: p.at.0,
        column: p.at.1,
        operator: p.operator.clone(),
        text: p.text.clone(),
        seconds: tenths(p.seconds),
    };
    Chosen {
        file: one.file.clone(),
        function: one.function.clone(),
        noticed: count("noticed"),
        unnoticed: count("unnoticed"),
        other: count("other"),
        reaches_out: serde_json::from_value(row["reaches_out"].clone()).ok(),
        faults: unnoticed.into_iter().map(fault).collect(),
    }
}

/// What every fault of the chosen functions took in the plan.
pub fn run_again<'a>(planted: impl IntoIterator<Item = &'a Vec<Planted>>) -> RunAgain {
    let all: Vec<&Planted> = planted.into_iter().flatten().collect();
    RunAgain {
        faults: all.len(),
        seconds: tenths(all.iter().map(|p| p.seconds).sum()),
    }
}
