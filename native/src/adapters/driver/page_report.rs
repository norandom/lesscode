//! The run page: what a run found about a change it judged
//! (`.kiro/specs/run-dashboard`, design: the snapshot, `report`; task 6.2).
//! One row per measure of a lens, before and after, with the direction that
//! was expected. A lens nobody measured is a row that says so. The page
//! shows these rows and judges nothing.

use serde::Serialize;
use serde_json::Value;

/// The lenses of a rework, as the page names them.
pub const SIZE: &str = "Size";
pub const STRUCTURE: &str = "Structure";
pub const STATE: &str = "State";
pub const LOCAL: &str = "Local complexity";
pub const BEHAVIOUR: &str = "Behaviour kept";
pub const PROTECTION: &str = "Protection";

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Report {
    pub lenses: Vec<Lens>,
    /// The planted faults of each function the change was judged on.
    pub functions: Vec<Protection>,
    pub refusal: Option<Refusal>,
}

/// Why a change was refused, in the command's own words.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Refusal {
    pub reasons: Vec<String>,
}

/// One measure of a lens.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Lens {
    pub lens: String,
    pub measure: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
    /// The direction a rework is expected to move the measure in.
    pub expected: String,
    /// Whether it moved that way; `None` where that cannot be said.
    pub moved: Option<bool>,
    pub measured: bool,
    /// The command that produced the numbers.
    pub command: Option<String>,
    /// The numbers were handed to the run and not measured by it.
    pub handed_in: bool,
    /// Why there is no number, or what the number leaves out.
    pub note: Option<String>,
}

impl Lens {
    /// A count the run took on both sides and that a rework should lower.
    pub fn falls(lens: &str, measure: &str, command: &str, [before, after]: [usize; 2]) -> Lens {
        Lens {
            lens: lens.to_owned(),
            measure: measure.to_owned(),
            before: Some(before.into()),
            after: Some(after.into()),
            expected: "falls".to_owned(),
            moved: Some(after < before),
            measured: true,
            command: Some(command.to_owned()),
            ..Lens::default()
        }
    }

    /// A lens without numbers, and why.
    pub fn not_measured(lens: &str, measure: &str, expected: &str, note: &str) -> Lens {
        Lens {
            lens: lens.to_owned(),
            measure: measure.to_owned(),
            expected: expected.to_owned(),
            note: Some(note.to_owned()),
            ..Lens::default()
        }
    }
}

/// Planted faults by what the tests made of them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Counts {
    pub noticed: u64,
    pub unnoticed: u64,
    /// Did not build, ran over the time limit or could not be run.
    pub neither: u64,
}

/// The faults of one function on both sides of a change. A side is absent
/// where the function does not exist on it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Protection {
    pub file: String,
    pub function: String,
    pub before: Option<Counts>,
    pub after: Option<Counts>,
    /// The reason the change gives for leaving the function unprotected.
    pub out_of_reach: Option<String>,
}
