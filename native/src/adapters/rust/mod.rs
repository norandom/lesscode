//! Rust runtime adapter implementation and profile module.
//!
//! Governed by:
//! - `.kiro/specs/rust-deconvolution-unification/design.md`
//! - `native/src/adapter_traits.rs`

pub mod analysis;
pub mod metrics;
pub mod mutation;

pub use analysis::RustEffectAnalyzer;
pub use mutation::RustMutantGenerator;
