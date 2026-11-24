//! `step protect`: the brownfield test stage
//! (`.kiro/specs/brownfield-test-stage`). Before a function is made less,
//! tests are written where the plan shows that none would notice a mistake.
//! `begin` turns a plan and the functions a person names into the start of
//! a stage; `result` (group 2 of the spec) says whether they are protected
//! enough.
//!
//! Neither command plants or runs a fault, opens a page or asks anything:
//! what they need is in their arguments and the files those name, and what
//! they found is in the exit code. They run the same under a person and in
//! a pipeline.

use super::protect_begin::{run_begin, BeginArgs};
use super::protect_result::{run_result, ResultArgs};

/// The commands of `lesscode-core step protect`.
#[derive(clap::Subcommand)]
pub enum ProtectCommand {
    /// Begin a stage: check a plan and the named functions against the code as it is.
    Begin(BeginArgs),
    /// Say whether the chosen functions are protected enough, from the verdict of `step check`.
    Result(ResultArgs),
}

pub fn run(command: ProtectCommand) -> Result<u8, String> {
    match command {
        ProtectCommand::Begin(args) => run_begin(args),
        ProtectCommand::Result(args) => run_result(args),
    }
}
