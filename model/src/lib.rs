//! The model seam of lesscode: the only place that talks to a model service.
//! Spec: `.kiro/specs/model-seam/`.

pub mod client;
pub mod endpoint;
pub mod error;
pub mod notice;
pub mod record;
pub mod repair;
pub mod request;
pub mod structure;
pub mod wait;

pub use client::{Attempt, begin, host_of, send};
pub use endpoint::endpoint_from;
pub use error::CallError;
pub use notice::{Notice, Terminal};
pub use request::{CallRequest, Reply, Sent, Stop, Wait};
pub use wait::{Clock, Limits, Time, WaitPolicy, call, call_with};
