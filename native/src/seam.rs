//! The launcher of the spec `model-seam` (design: Launcher, Process
//! protocol). The core reaches a model only through the second executable
//! `lesscode-model`: one request as JSON on its standard input, one answer as
//! JSON on its standard output. Its standard error, the notices of a wait and
//! the reason of a failure, goes to the user's terminal as it is written.
//! Standard library and the core's JSON type only; no client library.

use crate::jsonval::{from_serde, j_get, j_str, J};
use crate::receipt;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The number both sides state; an answer with another number is refused.
const PROTOCOL: i64 = 1;
/// The mode of a call fact written through this protocol (design: Data
/// Models). Earlier receipts say `openai-json-http`.
const MODE: &str = "seam-1";
const NAME: &str = "lesscode-model";
/// Names the executable; when set, no other place is looked at.
pub const VARIABLE: &str = "LESSCODE_MODEL_BIN";
/// The purpose the seam writes into its record of calls.
const PURPOSE: &str = "regenerate";

/// An answered call: the text as the model wrote it, and the call fact for a
/// receipt. The fact holds the mode `seam-1`, what `lesscode-model` reported
/// (stop, tokens, seconds, waits, the settings sent) in the receipt's words,
/// and the digests of the system and user texts, of the request as written
/// and of the reply.
#[derive(Debug)]
pub struct SeamReply {
    pub text: String,
    pub meta: J,
}

/// The text without one code fence around the whole of it, as the seam's
/// check reads it. Models wrap a JSON reply in a fence on some calls and not
/// on others.
pub fn unfenced(text: &str) -> &str {
    let body = text.trim();
    body.strip_prefix("```")
        .and_then(|rest| rest.strip_suffix("```"))
        .map(|inner| {
            // An optional language word on the fence's first line.
            let after_word = inner.trim_start_matches(|c: char| c.is_ascii_alphanumeric());
            match after_word.starts_with(['\n', '\r']) {
                true => after_word.trim(),
                false => inner.trim(),
            }
        })
        .unwrap_or(body)
}

/// The request as it is written to the executable's standard input. Only a
/// stated temperature or structure is part of it.
fn request_text(
    system: &str,
    user: &str,
    max_output_tokens: i64,
    temperature: Option<f64>,
    structure: Option<&J>,
) -> String {
    let mut pairs = vec![
        ("protocol", J::int(PROTOCOL)),
        ("purpose", J::str(PURPOSE)),
        ("system", J::str(system)),
        ("user", J::str(user)),
        ("max_output_tokens", J::int(max_output_tokens)),
    ];
    if let Some(temperature) = temperature {
        pairs.push(("temperature", J::Float(temperature)));
    }
    if let Some(structure) = structure {
        pairs.push(("structure", structure.clone()));
    }
    receipt::dumps(&J::obj(pairs))
}

/// The executable: the path in `LESSCODE_MODEL_BIN`, else beside the running
/// program, else in a directory of the search path. The error says that the
/// model commands need it and where it was looked for.
fn find() -> Result<PathBuf, String> {
    if let Some(named) = std::env::var_os(VARIABLE) {
        let path = PathBuf::from(named);
        return match path.is_file() {
            true => Ok(path),
            false => Err(format!(
                "the model commands need {NAME}; {VARIABLE} names {}, which is not a file",
                path.display()
            )),
        };
    }
    let file = format!("{NAME}{}", std::env::consts::EXE_SUFFIX);
    let beside = std::env::current_exe()
        .ok()
        .and_then(|program| program.parent().map(Path::to_path_buf));
    let search = std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).collect::<Vec<_>>())
        .unwrap_or_default();
    let found = beside
        .iter()
        .chain(&search)
        .map(|dir| dir.join(&file))
        .find(|path| path.is_file());
    found.ok_or_else(|| {
        let at = beside.map_or(String::new(), |dir| format!(" in {}", dir.display()));
        format!(
            "the model commands need {NAME}, which was not found: {VARIABLE} is not set, \
             and {file} is neither beside this program{at} nor in a directory of the search path"
        )
    })
}

/// The call fact of a receipt: the mode, the reply's fields in the receipt's
/// words, and the digests of what went to the executable and came back. Of
/// the settings it holds only those the executable reported as sent; a
/// setting reported as null was not sent and is left out, as is a count the
/// endpoint did not report.
fn call_fact(reply: &J, crossed: [(&str, &str); 4]) -> J {
    let field = |key: &str| j_get(reply, key).cloned().unwrap_or(J::Null);
    let stated = |pairs: Vec<(String, J)>| {
        J::Obj(pairs.into_iter().filter(|(_, v)| *v != J::Null).collect())
    };
    let tokens = [
        "input_tokens",
        "output_tokens",
        "reasoning_tokens",
        "cached_input_tokens",
    ];
    let sent = match field("sent") {
        J::Obj(pairs) => stated(pairs),
        other => other,
    };
    let mut fact = vec![
        ("mode", J::str(MODE)),
        ("stop_reason", field("stop")),
        (
            "usage",
            stated(tokens.map(|key| (key.to_string(), field(key))).to_vec()),
        ),
        ("seconds", field("seconds")),
        ("waits", field("waits")),
        ("sent", sent),
    ];
    let digest = |text: &str| J::str(receipt::sha256_hex(text.as_bytes()));
    fact.extend(crossed.map(|(key, text)| (key, digest(text))));
    J::obj(fact)
}

/// The answer on standard output: the text and the reply of an answered
/// call, or the reason of a failed one. The outer error says why it cannot
/// be read.
fn answer(stdout: &[u8]) -> Result<Result<(String, J), String>, String> {
    let value: serde_json::Value =
        serde_json::from_slice(stdout).map_err(|problem| format!("not JSON: {problem}"))?;
    let answer = from_serde(&value);
    let protocol = j_get(&answer, "protocol").unwrap_or(&J::Null);
    if *protocol != J::Int(PROTOCOL) {
        return Err(format!(
            "it states protocol {}, this program speaks protocol {PROTOCOL}",
            receipt::dumps(protocol)
        ));
    }
    match (j_get(&answer, "ok"), j_get(&answer, "reply")) {
        (Some(J::Bool(true)), Some(reply)) => match j_get(reply, "text") {
            Some(J::Str(text)) => Ok(Ok((text.clone(), reply.clone()))),
            _ => Err("the reply has no text".into()),
        },
        (Some(J::Bool(false)), _) => {
            let error = j_get(&answer, "error").unwrap_or(&J::Null);
            let field = |key| j_str(j_get(error, key).unwrap_or(&J::Null));
            Ok(Err(format!(
                "{NAME}: the call failed ({}): {}",
                field("kind"),
                field("reason")
            )))
        }
        _ => Err("neither an answered call with a reply nor a failed one".into()),
    }
}

/// One call through `lesscode-model call`, with the endpoint file when one is
/// named (else the executable reads its default one).
pub fn call_through_seam(
    endpoint_file: Option<&Path>,
    system: &str,
    user: &str,
    max_output_tokens: i64,
    temperature: Option<f64>,
    structure: Option<&J>,
) -> Result<SeamReply, String> {
    let executable = find()?;
    let mut command = Command::new(&executable);
    command.arg("call");
    if let Some(file) = endpoint_file {
        command.arg("--endpoint-env").arg(file);
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| {
            format!(
                "{NAME} ({}) could not be started: {e}",
                executable.display()
            )
        })?;
    let request = request_text(system, user, max_output_tokens, temperature, structure);
    // An executable that ends before reading the request still answers, on
    // standard output; what it said decides, the failed write is only noted.
    let written = child
        .stdin
        .take()
        .expect("standard input is piped")
        .write_all(request.as_bytes());
    let output = child
        .wait_with_output()
        .map_err(|e| format!("{NAME} ({}): {e}", executable.display()))?;
    let (text, reply) = answer(&output.stdout).unwrap_or_else(|problem| {
        let unwritten = written.err().map_or(String::new(), |e| {
            format!("; the request could not be written: {e}")
        });
        Err(format!(
            "{NAME} gave an answer that cannot be read ({problem}); it ended with {}{unwritten}",
            output.status
        ))
    })?;
    let crossed = [
        ("system_sha256", system),
        ("user_sha256", user),
        ("request_sha256", &request),
        ("response_sha256", &text),
    ];
    let meta = call_fact(&reply, crossed);
    Ok(SeamReply { text, meta })
}
