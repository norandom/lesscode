//! Agnostic driver: one engine for every language, driven by data packs.
//!
//! Governed by `.kiro/specs/agnostic-annotation-driver/design.md`. Nothing in
//! this module tree names a language or a project; that knowledge is in the
//! packs under `native/packs/`, listed once in `registry.rs`.

pub mod admission;
pub mod analysis;
pub mod annotations;
pub mod classes;
pub mod cli;
pub mod config;
pub mod contract;
pub mod controls;
pub mod interrupt;
pub mod measure;
pub mod merge;
pub mod operators;
pub mod pack;
pub mod process;
pub mod registry;
pub mod run_command;
pub mod runner;
pub mod sites;
pub mod source;
pub mod store;
pub mod tree;
pub mod workspace;

use serde::{Deserialize, Serialize};

/// The kinds of place where behaviour is decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    BranchCondition,
    FailureState,
    ErrorOrder,
    StateWrite,
    PureShellBoundary,
    /// The argument list of a call. A pack may leave this kind out.
    CallArgument,
}

impl Kind {
    /// The kinds every pack must have a rule for.
    pub const REQUIRED: [Kind; 5] = [
        Kind::BranchCondition,
        Kind::FailureState,
        Kind::ErrorOrder,
        Kind::StateWrite,
        Kind::PureShellBoundary,
    ];

    pub const ALL: [Kind; 6] = [
        Kind::BranchCondition,
        Kind::FailureState,
        Kind::ErrorOrder,
        Kind::StateWrite,
        Kind::PureShellBoundary,
        Kind::CallArgument,
    ];

    /// The name used in packs, annotations and receipts.
    pub fn name(self) -> &'static str {
        match self {
            Kind::BranchCondition => "branch_condition",
            Kind::FailureState => "failure_state",
            Kind::ErrorOrder => "error_order",
            Kind::StateWrite => "state_write",
            Kind::PureShellBoundary => "pure_shell_boundary",
            Kind::CallArgument => "call_argument",
        }
    }
}
