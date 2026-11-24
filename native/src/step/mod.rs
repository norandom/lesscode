//! Gradual refactoring in checked steps: one small change on one branch,
//! judged by the project's own tests and by planted faults
//! (`.kiro/specs/gradual-refactoring-steps`).
//!
//! Exit codes of every command: 0 accepted or done, 1 refused or rejected,
//! 2 not judged because a run is incomplete, 3 bad input or a void run.

pub mod check;
mod check_handed;
pub mod check_page;
mod check_report;
mod check_runs;
mod check_show;
pub mod context;
pub mod declaration;
pub mod faults;
pub mod functions;
pub mod guard;
pub mod hook;
pub mod ledger;
pub mod pin;
pub mod plan;
pub mod plan_page;
pub mod plan_show;
pub mod protect;
pub mod protect_begin;
pub mod record;
pub mod reports;
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
    /// Protect named functions with tests before they are made less: begin.
    Protect {
        #[command(subcommand)]
        cmd: protect::ProtectCommand,
    },
    /// Accept a step branch only with a verdict for exactly its content.
    Guard(guard::GuardArgs),
    /// Put an accepted step, an override or a release on the ledger.
    Record(record::RecordArgs),
    /// Write one page that lists several reports and links to each.
    Reports(reports::ReportsArgs),
    /// Install a pre-push hook that runs the budgets and the step guard.
    InstallHook(hook::InstallHookArgs),
    /// What the installed hook runs: budgets, step guard, spec audit.
    PrePush(hook::StepPrePushArgs),
}

pub fn run(command: StepCommand) -> Result<u8, String> {
    match command {
        StepCommand::Plan(args) => plan::run_plan(args),
        StepCommand::Pin(args) => pin::run_pin(args),
        StepCommand::Check(args) => check::run_check(args),
        StepCommand::Protect { cmd } => protect::run(cmd),
        StepCommand::Guard(args) => guard::run_guard(args),
        StepCommand::Record(args) => record::run_record(args),
        StepCommand::Reports(args) => reports::run_reports(args),
        StepCommand::InstallHook(args) => hook::run_install_hook(args),
        StepCommand::PrePush(args) => hook::run_pre_push_hook(args),
    }
}
