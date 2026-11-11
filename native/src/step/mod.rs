//! Gradual refactoring in checked steps: one small change on one branch,
//! judged by the project's own tests and by planted faults
//! (`.kiro/specs/gradual-refactoring-steps`).
//!
//! Exit codes of every command: 0 accepted or done, 1 refused or rejected,
//! 2 not judged because a run is incomplete, 3 bad input or a void run.

pub mod check;
mod check_runs;
pub mod context;
pub mod declaration;
pub mod faults;
pub mod functions;
pub mod pin;
pub mod plan;
pub mod revisions;
pub mod trees;
pub mod verdict;

/// The commands of `lesscode-core step`.
#[derive(clap::Subcommand)]
pub enum StepCommand {
    /// Show per function where the tests notice planted faults and where not.
    Plan(plan::PlanArgs),
    /// Judge whether the tests of the step pin a named unnoticed fault.
    Pin(pin::PinArgs),
    /// Judge a step: tests on base and step, planted faults before and after.
    Check(check::CheckArgs),
}

pub fn run(command: StepCommand) -> Result<u8, String> {
    match command {
        StepCommand::Plan(args) => plan::run_plan(args),
        StepCommand::Pin(args) => pin::run_pin(args),
        StepCommand::Check(args) => check::run_check(args),
    }
}
