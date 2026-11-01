//! Native capability-compression package.
//!
//! The parser substrate is exposed directly; metric definitions and comparison
//! behavior are implemented in subsequent tasks. Parsing never executes source.

pub use ast_grep_core::AstGrep;
pub use ast_grep_language::{Python, Tsx, TypeScript};

pub mod metrics;
pub mod model;
pub mod scope;
