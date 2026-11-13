//! The model seam of lesscode: the only place that talks to a model service.
//! Spec: `.kiro/specs/model-seam/`.

pub mod client;
pub mod error;
pub mod request;

pub use client::{host_of, send};
pub use error::CallError;
pub use request::{CallRequest, Reply, Sent, Stop, Wait};
