//! Pure turn-input validation under the ratified S10 contract (U6): verify frozen
//! regenerator turns — preregistered parameters, permitted feedback, response
//! classification and retained failed attempts — against recorded expectations.
//! Evidence interpretation only; no turn is re-run and no model is called.

use crate::pyjson::{self, Fmt, J};
use std::fs;
use std::path::{Path, PathBuf};

pub struct TurnSpec {
    pub dir: PathBuf,
    pub expected: String,
    pub request: PathBuf,
    pub feedback_required: bool,
}

fn get<'a>(v: &'a J, key: &str) -> Option<&'a J> {
    match v {
        J::Obj(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
        _ => None,
    }
}
fn as_str(v: Option<&J>) -> Option<&str> {
    match v {
        Some(J::Str(s)) => Some(s),
        _ => None,
    }
}

/// Classify a recorded response: `thinking-only` (stop_reason max_tokens, no
/// non-thinking content), `produced-candidate` (end_turn with tool use) or the
/// raw stop reason when it matches neither recorded shape.
fn classify(response: &J) -> String {
    let stop = as_str(get(response, "stop_reason")).unwrap_or("");
    let thinking_only = match get(response, "content") {
        Some(J::Arr(items)) => {
            !items.is_empty()
                && items
                    .iter()
                    .all(|c| as_str(get(c, "type")) == Some("thinking"))
        }
        _ => false,
    };
    match (stop, thinking_only) {
        ("max_tokens", true) => "thinking-only".into(),
        ("end_turn", _) => "produced-candidate".into(),
        (other, _) => format!("unclassified:{other}"),
    }
}

/// Test-visible classification wrapper.
pub fn classify_for_test(response: &J) -> String {
    classify(response)
}

pub fn check_turn(root: &Path, spec: &TurnSpec) -> Vec<(String, bool, String)> {
    let mut results = Vec::new();
    let user = fs::read_to_string(root.join(&spec.dir).join("user.txt")).ok();
    let response = fs::read_to_string(root.join(&spec.dir).join("response.json"))
        .ok()
        .and_then(|t| pyjson::parse(&t).ok());
    let request = fs::read_to_string(&spec.request)
        .ok()
        .and_then(|t| pyjson::parse(&t).ok());
    // Retained failed attempts: every turn directory keeps its bytes (bytes
    // verified separately by admission; here presence + parseability).
    results.push((
        "bytes-present".into(),
        user.is_some() && response.is_some(),
        format!(
            "user.txt={} response.json={}",
            user.is_some(),
            response.is_some()
        ),
    ));
    let (Some(user), Some(response), Some(request)) = (&user, &response, &request) else {
        return results;
    };
    // Preregistered parameters: targets, contract and acceptance command exist
    // in the request; feedback is null exactly where no prior turn failed.
    for key in ["targets", "contract", "acceptance_command"] {
        results.push((
            format!("preregistered-{key}"),
            !matches!(get(request, key), None | Some(J::Null)),
            format!("{key} present in {}", spec.request.display()),
        ));
    }
    let feedback = get(request, "feedback");
    let feedback_ok = if spec.feedback_required {
        !matches!(feedback, None | Some(J::Null))
    } else {
        matches!(feedback, None | Some(J::Null))
    };
    results.push((
        "permitted-feedback".into(),
        feedback_ok,
        format!(
            "feedback_required={} and feedback is {}",
            spec.feedback_required,
            if matches!(feedback, None | Some(J::Null)) {
                "null"
            } else {
                "present"
            }
        ),
    ));
    // Pure turn-input transformation: the assembled prompt carries the frozen
    // contract head and the acceptance command from the request.
    for (label, needle) in [
        (
            "prompt-contract",
            as_str(get(request, "contract")).unwrap_or(""),
        ),
        (
            "prompt-acceptance-command",
            as_str(get(request, "acceptance_command")).unwrap_or(""),
        ),
    ] {
        let needle = needle.lines().next().unwrap_or("").trim();
        results.push((
            label.into(),
            !needle.is_empty() && user.contains(needle),
            format!("user.txt carries the frozen {}", label),
        ));
    }
    // Response classification against the preregistered expectation.
    let class = classify(response);
    let expected_class = match spec.expected.as_str() {
        "failed-thinking-only" => "thinking-only",
        "accepted" => "produced-candidate",
        "failed-collection" => "produced-candidate", // the model produced a candidate; the arm's collection failed
        other => other,
    };
    results.push((
        "response-classification".into(),
        class == expected_class,
        format!("expected {expected_class}, recorded {class}"),
    ));
    results
}

/// CLI entry: descriptor `{"turns_root": "...", "turns": [{"dir", "expected",
/// "request", "feedback_required"}]}`; writes a verdict receipt; exit 0 only if
/// every check passes.
pub fn cli_run(descriptor: &Path, out: &Path) -> Result<u8, String> {
    let text = fs::read_to_string(descriptor)
        .map_err(|e| format!("cannot read {}: {e}", descriptor.display()))?;
    let spec = pyjson::parse(&text).map_err(|e| format!("invalid descriptor: {e}"))?;
    let root = PathBuf::from(as_str(get(&spec, "turns_root")).ok_or("turns_root missing")?);
    let turns = match get(&spec, "turns") {
        Some(J::Arr(items)) => items.clone(),
        _ => return Err("turns must be an array".into()),
    };
    let mut per_turn = Vec::new();
    let mut failed = 0usize;
    for t in &turns {
        let dir = as_str(get(t, "dir")).ok_or("turn needs dir")?.to_string();
        let expected = as_str(get(t, "expected"))
            .ok_or("turn needs expected")?
            .to_string();
        let request = as_str(get(t, "request"))
            .ok_or("turn needs request")?
            .to_string();
        let feedback_required = matches!(get(t, "feedback_required"), Some(J::Bool(true)));
        let checks = check_turn(
            &root,
            &TurnSpec {
                dir: PathBuf::from(&dir),
                expected: expected.clone(),
                request: PathBuf::from(request),
                feedback_required,
            },
        );
        failed += checks.iter().filter(|(_, ok, _)| !ok).count();
        per_turn.push(J::obj(vec![
            ("turn", J::str(dir)),
            ("expected", J::str(expected)),
            (
                "checks",
                J::Arr(
                    checks
                        .iter()
                        .map(|(c, ok, note)| {
                            J::obj(vec![
                                ("check", J::str(c.clone())),
                                ("ok", J::Bool(*ok)),
                                ("note", J::str(note.clone())),
                            ])
                        })
                        .collect(),
                ),
            ),
        ]));
    }
    let receipt = J::obj(vec![
        ("receipt", J::str("s10-turncheck-1")),
        ("descriptor", J::str(descriptor.display().to_string())),
        ("turns", J::Arr(per_turn)),
        ("failed_checks", J::int(failed as i64)),
    ]);
    let mut body = String::new();
    pyjson::write(
        &receipt,
        Fmt::Indent {
            width: 1,
            sort_keys: false,
        },
        &mut body,
    );
    body.push('\n');
    fs::write(out, body.as_bytes()).map_err(|e| format!("cannot write {}: {e}", out.display()))?;
    println!("turn checks: {} failed", failed);
    Ok(if failed == 0 { 0 } else { 1 })
}
