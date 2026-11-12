//! Prototype gate P4 of the spec `model-seam`: the core reaches a model
//! through a second executable in place of its own HTTP request.
//!
//! When `LESSCODE_MODEL_BIN` names an executable, a model request is handed
//! to it as one JSON object on standard input and its one JSON reply is read
//! from standard output; what it writes to standard error (waiting notices,
//! the reason of a failure) goes to the user's terminal. The endpoint is
//! named by its file (`LESSCODE_MODEL_ENV`), never by its key. Without the
//! variable the core's own request is used, as before.
//!
//! This prototype sits at the transport seam of the regenerator, so it
//! receives the request the hand-written client built and returns a reply in
//! the form that client expects. The final launcher replaces that client.

use super::ureq_post;
use serde_json::{json, Value};
use std::io::Write;
use std::process::{Command, Stdio};

const PREFIX: &str = "openai transport: model executable";

/// The reply without one code fence around the whole of it.
fn unfenced(text: &str) -> &str {
    let body = text.trim();
    body.strip_prefix("```")
        .and_then(|rest| rest.strip_suffix("```"))
        .map(|inner| inner.strip_prefix("json").unwrap_or(inner).trim())
        .unwrap_or(body)
}

/// A transport for the regenerator: through the executable when one is
/// configured, else the core's own request.
pub fn post(url: &str, headers: &[(String, String)], body: &[u8]) -> Result<Vec<u8>, String> {
    let Some(executable) = std::env::var_os("LESSCODE_MODEL_BIN") else {
        return ureq_post(url, headers, body);
    };
    let endpoint_file = std::env::var_os("LESSCODE_MODEL_ENV")
        .ok_or_else(|| format!("{PREFIX}: LESSCODE_MODEL_ENV must name the endpoint file"))?;
    let built: Value = serde_json::from_slice(body).map_err(|e| format!("{PREFIX}: {e}"))?;
    let message = |index: usize| built["messages"][index]["content"].as_str().unwrap_or("");
    // The temperature of the built request is not passed on: it is 0 whenever
    // none was given, and two of three vendors refuse that value.
    let request = json!({
        "protocol": 1, "system": message(0), "user": message(1),
        "max_output_tokens": built["max_tokens"],
    });
    let mut child = Command::new(&executable)
        .arg("call")
        .arg(&endpoint_file)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("{PREFIX} {}: {e}", executable.to_string_lossy()))?;
    child
        .stdin
        .take()
        .expect("standard input was piped")
        .write_all(request.to_string().as_bytes())
        .map_err(|e| format!("{PREFIX}: {e}"))?;
    let output = child
        .wait_with_output()
        .map_err(|e| format!("{PREFIX}: {e}"))?;
    let reply: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("{PREFIX}: unreadable reply ({e}), exit {}", output.status))?;
    if reply["protocol"] != 1 {
        return Err(format!("{PREFIX}: protocol 1 expected"));
    }
    if reply["ok"] != true {
        let reason = reply["error"]["reason"]
            .as_str()
            .unwrap_or("no reason given");
        return Err(format!("{PREFIX}: {reason}"));
    }
    let answer = &reply["reply"];
    let shaped = json!({
        "choices": [{
            "message": {"content": unfenced(answer["text"].as_str().unwrap_or(""))},
            "finish_reason": answer["stop"],
        }],
        "usage": {
            "prompt_tokens": answer["input_tokens"], "completion_tokens": answer["output_tokens"],
            "seconds": answer["seconds"], "waits": answer["waits"], "sent": answer["sent"],
        },
    });
    Ok(shaped.to_string().into_bytes())
}

#[cfg(test)]
mod tests {
    use super::unfenced;

    #[test]
    fn one_fence_around_the_whole_reply_is_removed() {
        assert_eq!(unfenced("```json\n{\"a\": 1}\n```"), "{\"a\": 1}");
        assert_eq!(unfenced("```\n{}\n```\n"), "{}");
        assert_eq!(unfenced(" {\"a\": \"```\"} "), "{\"a\": \"```\"}");
    }
}
