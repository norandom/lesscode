//! Agnostic driver: one engine for every language, driven by data packs.
//!
//! Governed by `.kiro/specs/agnostic-annotation-driver/design.md`. Nothing in
//! this module tree names a language or a project; that knowledge is in the
//! packs under `native/packs/`, listed once in `registry.rs`.

pub mod pack;
pub mod registry;

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
}
