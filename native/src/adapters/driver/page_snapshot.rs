//! The one value the run page shows (`.kiro/specs/run-dashboard`, design:
//! the snapshot). The page draws from it and from nothing else, so every
//! number and every sentence about the run is here and not in the page.

use serde::Serialize;

/// The version of this shape. A page reads only the version it was made for.
pub const SCHEMA: u32 = 1;

/// A moment as milliseconds since 1970, in UTC, written as browsers read it.
pub fn stamp(milliseconds: u64) -> String {
    let (seconds, part) = (milliseconds / 1_000, milliseconds % 1_000);
    let (day, rest) = (seconds / 86_400, seconds % 86_400);
    let (year, month, date) = civil(day as i64);
    let (hour, minute, second) = (rest / 3_600, rest % 3_600 / 60, rest % 60);
    format!("{year:04}-{month:02}-{date:02}T{hour:02}:{minute:02}:{second:02}.{part:03}Z")
}

/// Year, month and day of a day counted from 1970-01-01. The step ledger
/// has the same arithmetic for its dates; the driver may not reach up into
/// the step commands for it.
fn civil(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let (era, day_of_era) = (z.div_euclid(146_097), z.rem_euclid(146_097));
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted = (5 * day_of_year + 2) / 153;
    let month = if shifted < 10 {
        shifted + 3
    } else {
        shifted - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day_of_year - (153 * shifted + 2) / 5 + 1)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Snapshot {
    pub schema: u32,
    /// Rises with every change, so that a reader can tell two snapshots apart.
    pub sequence: u64,
    pub tool: Tool,
    pub command: String,
    pub started: String,
    /// The last sign of life of the tool.
    pub heartbeat: String,
    pub ended: bool,
    pub outcome: Option<String>,
    pub outcome_detail: Option<String>,
    pub exit_code: Option<u8>,
    /// How long the run has taken so far, or took.
    pub seconds: f64,
    pub subject: Subject,
    pub copies: Copies,
    /// The project's commands the run executes, in the copies only.
    pub commands: Vec<Vec<String>>,
    pub writes_in_repository: Vec<String>,
    pub tests: Tests,
    pub stages: Vec<Stage>,
    pub faults: Faults,
    /// What the person can do next, for this outcome.
    pub next: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Tool {
    pub name: String,
    pub version: String,
}

/// What the run reads. `None` where it is not known.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Subject {
    pub repository: Option<String>,
    pub branch: Option<String>,
    pub commit: Option<String>,
    /// Uncommitted changes to tracked files when the run began.
    pub dirty: Option<bool>,
    pub on_main_branch: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Copies {
    pub directory: Option<String>,
    pub planned: u64,
    pub working: u64,
    pub dropped: Vec<Dropped>,
    /// The space the copies take, once measured.
    pub bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Dropped {
    pub copy: u64,
    pub at: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Tests {
    pub command_given: bool,
    /// Whether the unchanged tests passed in at least one copy.
    pub passed_unchanged: Option<bool>,
    pub tests_run: Option<u64>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Status {
    #[serde(rename = "not begun")]
    NotBegun,
    #[serde(rename = "going on")]
    GoingOn,
    #[serde(rename = "ended")]
    Ended,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "left out")]
    LeftOut,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Stage {
    pub id: String,
    pub title: String,
    /// One or two sentences on what the stage does to the code.
    pub what: String,
    pub status: Status,
    /// Items done and in all, for a stage that counts.
    pub done: Option<u64>,
    pub total: Option<u64>,
    /// How long the stage took, once it has ended.
    pub seconds: Option<f64>,
    pub seconds_so_far: Option<f64>,
    /// Given only after `FEW` items are done.
    pub seconds_left: Option<f64>,
    pub reason: Option<String>,
}

/// Fewer items done than this say nothing about the rest.
pub const FEW: u64 = 5;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Faults {
    pub total: u64,
    pub done: u64,
    pub noticed: u64,
    pub unnoticed: u64,
    pub not_built: u64,
    pub timed_out: u64,
    /// Faults whose run failed twice and that have no class.
    pub not_judged: u64,
    /// Faults in code the platform does not compile; `None` while unknown.
    pub not_compiled_here: Option<u64>,
    pub example: Option<Example>,
    pub files: Vec<Tally>,
    pub functions: Vec<Tally>,
}

/// One fault of the run, shown with the explanation of what a fault is.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Example {
    pub file: String,
    pub line: u64,
    pub what_was_changed: String,
    pub text: String,
    pub outcome: String,
}

/// The faults of one file or one function.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Tally {
    pub file: String,
    /// Absent for a file.
    pub function: Option<String>,
    pub noticed: u64,
    pub unnoticed: u64,
    pub not_built: u64,
    pub timed_out: u64,
    pub not_judged: u64,
    /// Faults not yet run.
    pub left: u64,
    /// No fault was noticed, and none is left to run.
    pub unprotected: bool,
    /// What the function calls and writes outside itself, where known.
    pub reaches_out: Vec<String>,
}

impl Tally {
    pub fn done(&self) -> u64 {
        self.noticed + self.unnoticed + self.not_built + self.timed_out + self.not_judged
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_moment_is_written_as_a_browser_reads_it() {
        assert_eq!(stamp(0), "1970-01-01T00:00:00.000Z");
        // 2026-11-23 14:20:05.250 UTC.
        assert_eq!(stamp(1_795_443_605_250), "2026-11-23T14:20:05.250Z");
    }

    #[test]
    fn a_status_is_written_in_words() {
        let written = serde_json::to_string(&[Status::NotBegun, Status::GoingOn, Status::LeftOut]);
        assert_eq!(written.unwrap(), r#"["not begun","going on","left out"]"#);
    }
}
