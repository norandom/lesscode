//! Gradual refactoring in checked steps: one small change on one branch,
//! judged by the project's own tests and by planted faults
//! (`.kiro/specs/gradual-refactoring-steps`).

pub mod declaration;
pub mod functions;
pub mod revisions;
