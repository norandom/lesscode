//! Rust runtime adapter implementation and profile module.
//!
//! Governed by:
//! - `.kiro/specs/rust-deconvolution-unification/design.md`
//! - `native/src/adapter_traits.rs`

pub mod analysis;

pub use analysis::RustEffectAnalyzer;
