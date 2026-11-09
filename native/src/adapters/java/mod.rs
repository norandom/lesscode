//! Java runtime adapter implementation and profile module.
//!
//! Governed by:
//! - `.kiro/specs/java-deconvolution-unification/design.md`
//! - `native/src/adapter_traits.rs`

pub mod analysis;
pub mod metrics;
pub mod mutation;

pub use analysis::JavaEffectAnalyzer;
pub use mutation::JavaMutantGenerator;
