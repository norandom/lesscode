//! The data of one call: what a caller asks, what comes back, and what was
//! actually sent.

use serde::{Deserialize, Serialize};

/// One request to a model. Only what is stated here, in the endpoint file or
/// in the settings of the run is sent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallRequest {
    /// Named by the caller; goes into the record of the call.
    pub purpose: String,
    pub system: String,
    pub user: String,
    pub max_output_tokens: u32,
    /// Sent only when given.
    #[serde(default)]
    pub temperature: Option<f64>,
    /// A JSON schema the reply should satisfy. It is not sent; the reply is
    /// checked against it (`Reply::structured`).
    #[serde(default)]
    pub structure: Option<serde_json::Value>,
}

/// Why the model stopped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stop {
    Complete,
    /// The reply ended at the output limit.
    CutOff,
    Refused,
    /// The endpoint's own word for anything else.
    Other(String),
}

/// What was sent, for receipts and records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sent {
    pub host: String,
    pub model: String,
    pub max_output_tokens: u32,
    pub temperature: Option<f64>,
    pub reasoning_level: Option<String>,
}

/// Time spent before an endpoint accepted a call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Wait {
    pub seconds: f64,
    pub stated_by_endpoint: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reply {
    /// The text parts joined, without the model's reasoning.
    pub text: String,
    pub stop: Stop,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub reasoning_tokens: Option<u64>,
    pub cached_input_tokens: Option<u64>,
    /// Present only when a structure was asked for.
    pub structured: Option<bool>,
    pub seconds: f64,
    pub waits: Vec<Wait>,
    pub sent: Sent,
}
