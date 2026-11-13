//! The model seam of lesscode: the only place that talks to a model service.
//! Spec: `.kiro/specs/model-seam/`.

pub mod client;
pub mod endpoint;
pub mod error;
pub mod notice;
pub mod request;
pub mod structure;

pub use client::{host_of, send};
pub use endpoint::endpoint_from;
pub use error::CallError;
pub use notice::{Notice, Terminal};
pub use request::{CallRequest, Reply, Sent, Stop, Wait};
