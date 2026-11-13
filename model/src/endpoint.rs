//! Reading the endpoint file. The file's keys and the rules for a model or a
//! reasoning level set for one run are the core's (`load_endpoint`); this
//! adds what a user needs to see when the file is not usable.

use crate::error::CallError;
use lesscode_core::regenerator::{Endpoint, load_endpoint};
use std::path::Path;

/// The endpoint described by the file, or the default file when none is
/// named. A failure names the file and what is wrong with it; nothing is
/// called.
pub fn endpoint_from(file: Option<&Path>) -> Result<Endpoint, CallError> {
    load_endpoint(file).map_err(|problem| {
        let named = match file {
            Some(path) => format!("endpoint file {}", path.display()),
            None => "the default endpoint file (LESSCODE_MODEL_ENV, else .lesscode/model.env \
                     in the home directory)"
                .to_owned(),
        };
        let known = if problem.starts_with("unknown LESSCODE_MODEL_API_STYLE") {
            "; known styles: openai, anthropic"
        } else {
            ""
        };
        CallError::Config(format!("{named}: {problem}{known}"))
    })
}
