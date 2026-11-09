//! Java runtime adapter implementation and profile module.
//!
//! Governed by:
//! - `.kiro/specs/java-deconvolution-unification/design.md`
//! - `native/src/adapter_traits.rs`

pub mod analysis;

pub use analysis::JavaEffectAnalyzer;
