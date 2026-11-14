//! The repair loop around a caller's check: propose, check, and on a
//! rejection ask afresh with the request, the rejected reply and the
//! reasons. The check decides alone what is accepted, and it is handed every
//! reply before the caller gets it back. The loop makes its calls through
//! the caller's `call` and writes no file.
//!
//! The check is a required argument:
//!
//! ```
//! use lesscode_model::repair::{REPAIRS, Verdict, with_repairs};
//! use lesscode_model::{CallError, CallRequest};
//!
//! let request = CallRequest {
//!     purpose: "example".into(),
//!     system: String::new(),
//!     user: "Say ok.".into(),
//!     max_output_tokens: 100,
//!     temperature: None,
//!     structure: None,
//! };
//! let mut call = |_: &CallRequest| Err(CallError::Config("no endpoint in an example".into()));
//! let mut check = |text: &str| Verdict { passed: text == "ok", reasons: vec![] };
//! with_repairs(&mut call, &request, &mut check, REPAIRS);
//! ```
//!
//! The same without a check is not a program:
//!
//! ```compile_fail
//! use lesscode_model::repair::{REPAIRS, with_repairs};
//! use lesscode_model::{CallError, CallRequest};
//!
//! let request = CallRequest {
//!     purpose: "example".into(),
//!     system: String::new(),
//!     user: "Say ok.".into(),
//!     max_output_tokens: 100,
//!     temperature: None,
//!     structure: None,
//! };
//! let mut call = |_: &CallRequest| Err(CallError::Config("no endpoint in an example".into()));
//! with_repairs(&mut call, &request, REPAIRS);
//! ```

use crate::error::CallError;
use crate::request::{CallRequest, Reply, Stop};

/// The repairs allowed for one request unless the caller states another
/// number (measured in R99).
pub const REPAIRS: usize = 2;

/// What the caller's check says about one reply.
#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    pub passed: bool,
    /// Why not, in the check's words.
    pub reasons: Vec<String>,
}

/// One reply as the check saw it.
#[derive(Debug, Clone, PartialEq)]
pub struct Attempt {
    pub passed: bool,
    pub reasons: Vec<String>,
    pub stop: Stop,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Debug, Clone)]
pub enum Outcome {
    /// The reply passed the check.
    Accepted {
        reply: Reply,
        attempts: Vec<Attempt>,
    },
    /// No reply passed within the repairs allowed. An answer the check
    /// rejects is not a failure of the endpoint.
    Rejected { last: Reply, attempts: Vec<Attempt> },
    /// A call failed; the attempts are those made before it.
    Failed {
        error: CallError,
        attempts: Vec<Attempt>,
    },
}

/// The request of a repair: the request as it was, the rejected reply and
/// why it was rejected. Everything but the request text stays as stated.
pub fn repair_request(request: &CallRequest, rejected: &str, reasons: &[String]) -> CallRequest {
    CallRequest {
        user: format!(
            "{}\n\nA previous answer to this request was rejected.\n\nThe rejected answer:\n\
             {rejected}\n\nWhy it was rejected:\n- {}\n\nWrite the corrected answer.",
            request.user,
            reasons.join("\n- ")
        ),
        ..request.clone()
    }
}

/// A first call and, while the check rejects the reply, at most `repairs`
/// more. A repair is a fresh call that shows the rejected reply and the
/// reasons; where no reply arrived or it was not complete, the request goes
/// out again as it was. `Accepted` is made in one place, after a verdict
/// that passed.
pub fn with_repairs(
    call: &mut dyn FnMut(&CallRequest) -> Result<Reply, CallError>,
    request: &CallRequest,
    check: &mut dyn FnMut(&str) -> Verdict,
    repairs: usize,
) -> Outcome {
    let mut attempts = Vec::new();
    let mut asked = request.clone();
    loop {
        let reply = match call(&asked) {
            Ok(reply) => reply,
            Err(error) => return Outcome::Failed { error, attempts },
        };
        let Verdict { passed, reasons } = check(&reply.text);
        attempts.push(Attempt {
            passed,
            reasons,
            stop: reply.stop.clone(),
            input_tokens: reply.input_tokens,
            output_tokens: reply.output_tokens,
        });
        if passed {
            return Outcome::Accepted { reply, attempts };
        }
        if attempts.len() > repairs {
            return Outcome::Rejected {
                last: reply,
                attempts,
            };
        }
        let whole = reply.stop == Stop::Complete && !reply.text.trim().is_empty();
        asked = if whole {
            let reasons = &attempts[attempts.len() - 1].reasons;
            repair_request(request, &reply.text, reasons)
        } else {
            request.clone()
        };
    }
}
