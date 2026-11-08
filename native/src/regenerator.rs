//! Two-tier refine controller port (task 8.5, R36): `TurnRequest` and the
//! prompt builder, the Tier-1 static ladder (stop_reason, schema, editscope,
//! syntax, Contract-B surface, pure/shell write-sites, static targets), the
//! Tier-2 pinned-formula runtime verdict, BestOfN refine control with
//! preregistered retry directives, receipts with `.sha256` sidecars and
//! `bindings.json`, and the openai-compatible HTTP adapter on pinned `ureq`.
//! Ports `src/lesscode_baseline/regenerator.py` (frozen `regen_turn.py`
//! contracts: exit codes 0/2/3/4/5/6, R26 amendments 1-4).
//!
//! Port deltas (documented, no silent narrowing): only the openai-compatible
//! adapter ships here (R36 dependency decision); the anthropic streaming arm
//! and its `count_tokens` guard — the only exit-5 trigger — stay on the
//! Python substrate. `run_refine_loop` evaluators drop Python's always-empty
//! `results` argument. Receipt objects arrive through `serde_json` maps, so
//! keys load sorted where the Python dicts preserved insertion order (same
//! class of divergence as the 8.2 surface report loader; semantics are
//! order-insensitive).

use crate::jsonval::{
    from_serde as j_from_value, j_get, j_str, to_serde as j_to_value,
    write_json_str_py as write_str_py,
};
use crate::measures::{self, Structure};
use crate::receipt::{self, J};
use crate::recording;
use crate::surface;
use std::collections::BTreeSet;
use std::fs;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// Exit codes preserved from the frozen regen_turn.py contract.
pub const EXIT_OK: u8 = 0;
pub const EXIT_USAGE: u8 = 2;
pub const EXIT_FAILED_TURN: u8 = 3;
pub const EXIT_COPY_MISMATCH: u8 = 4;
pub const EXIT_INPUT_TOO_LARGE: u8 = 5;
pub const EXIT_OUT_OF_SCOPE: u8 = 6;

pub const MAX_INPUT_TOKENS: i64 = 120_000;
const FEEDBACK_HEADING: &str = "## Feedback from the previous round";
const FEEDBACK_FIELDS: [&str; 4] = [
    "### First divergence",
    "### Measure gaps against the targets",
    "### Surviving faults",
    "### Realised counterexamples",
];
const SEED_NOTE: &str = "endpoint exposes no seed; determinism not claimed";

/// Controller errors mapped onto the frozen exit codes.
#[derive(Debug)]
pub enum RegenError {
    /// Duplicate receipt labels or holdout overlap (exit 2).
    Usage(String),
    /// Failed turn, malformed input or unexpected IO (exit 3).
    FailedTurn(String),
    /// The candidate copy's scope files differ from the frozen ones (exit 4).
    CopyMismatch(String),
    /// The counted input exceeds `MAX_INPUT_TOKENS` (exit 5; unreachable on
    /// the openai arm, preserved for contract parity).
    InputTooLarge(String),
    /// A returned path is outside the scope list (exit 6).
    OutOfScope(String),
    /// Feedback payload carries held-out sequence names (refuse to dispatch).
    FeedbackBoundary(String),
}

impl RegenError {
    pub fn exit_code(&self) -> u8 {
        match self {
            RegenError::Usage(_) => EXIT_USAGE,
            RegenError::FailedTurn(_) => EXIT_FAILED_TURN,
            RegenError::CopyMismatch(_) => EXIT_COPY_MISMATCH,
            RegenError::InputTooLarge(_) => EXIT_INPUT_TOO_LARGE,
            RegenError::OutOfScope(_) => EXIT_OUT_OF_SCOPE,
            // Not in the frozen CLI contract (Python lets it escape as a
            // traceback); the controller surface still maps it somewhere.
            RegenError::FeedbackBoundary(_) => EXIT_FAILED_TURN,
        }
    }
    pub fn message(&self) -> &str {
        match self {
            RegenError::Usage(m)
            | RegenError::FailedTurn(m)
            | RegenError::CopyMismatch(m)
            | RegenError::InputTooLarge(m)
            | RegenError::OutOfScope(m)
            | RegenError::FeedbackBoundary(m) => m,
        }
    }
}

// --- JSON helpers ------------------------------------------------------------

/// Python truthiness for the receipt domain (missing key counts falsy).
fn j_truthy(v: Option<&J>) -> bool {
    match v {
        None | Some(J::Null) => false,
        Some(J::Bool(b)) => *b,
        Some(J::Int(n)) => *n != 0,
        Some(J::Float(x)) => *x != 0.0,
        Some(J::Str(s)) => !s.is_empty(),
        Some(J::Arr(a)) => !a.is_empty(),
        Some(J::Obj(o)) => !o.is_empty(),
    }
}

fn py_float(x: f64) -> String {
    recording::py_float_repr(x)
}

/// `json.dumps(value)` parity: compact `, `/`: ` separators, Python float
/// repr, `ensure_ascii` and optional `sort_keys` (byte-order key sort, which
/// matches Python code-point ordering for UTF-8).
fn compact_dumps(value: &J, sort_keys: bool, ascii: bool) -> String {
    fn write(v: &J, out: &mut String, sort_keys: bool, ascii: bool) {
        match v {
            J::Null => out.push_str("null"),
            J::Bool(true) => out.push_str("true"),
            J::Bool(false) => out.push_str("false"),
            J::Int(n) => out.push_str(&n.to_string()),
            J::Float(x) if x.is_finite() => out.push_str(&py_float(*x)),
            J::Float(_) => out.push_str("null"),
            J::Str(s) => write_str_py(s, out, ascii),
            J::Arr(items) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    write(item, out, sort_keys, ascii);
                }
                out.push(']');
            }
            J::Obj(pairs) => {
                let mut keys: Vec<&(String, J)> = pairs.iter().collect();
                if sort_keys {
                    keys.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
                }
                out.push('{');
                for (i, (k, val)) in keys.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    write_str_py(k, out, ascii);
                    out.push_str(": ");
                    write(val, out, sort_keys, ascii);
                }
                out.push('}');
            }
        }
    }
    let mut out = String::new();
    write(value, &mut out, sort_keys, ascii);
    out
}

/// Python `str()` for scalar receipt values (top-level strings verbatim).
fn py_str(v: &J) -> String {
    match v {
        J::Str(s) => s.clone(),
        J::Null => "None".into(),
        J::Bool(true) => "True".into(),
        J::Bool(false) => "False".into(),
        J::Int(n) => n.to_string(),
        J::Float(x) => py_float(*x),
        other => py_repr(other),
    }
}

/// Python `repr()` for nested diagnostic values (prompt feedback only).
fn py_repr(v: &J) -> String {
    match v {
        J::Str(s) => {
            // Python repr prefers single quotes, switching to double quotes
            // only when the text holds a single quote and no double quote.
            let quote = if s.contains('\'') && !s.contains('"') {
                '"'
            } else {
                '\''
            };
            let mut out = String::from(quote);
            for c in s.chars() {
                match c {
                    '\n' => out.push_str("\\n"),
                    '\t' => out.push_str("\\t"),
                    '\r' => out.push_str("\\r"),
                    c if c == quote || c == '\\' => {
                        out.push('\\');
                        out.push(c);
                    }
                    c => out.push(c),
                }
            }
            out.push(quote);
            out
        }
        J::Null => "None".into(),
        J::Bool(b) => if *b { "True" } else { "False" }.into(),
        J::Int(n) => n.to_string(),
        J::Float(x) => py_float(*x),
        J::Arr(items) => {
            let parts: Vec<String> = items.iter().map(py_repr).collect();
            format!("[{}]", parts.join(", "))
        }
        J::Obj(pairs) => {
            let parts: Vec<String> = pairs
                .iter()
                .map(|(k, v)| format!("{}: {}", py_repr(&J::str(k.clone())), py_repr(v)))
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
    }
}

pub fn sha256_text(text: &str) -> String {
    receipt::sha256_hex(text.as_bytes())
}

pub fn sha256_file(path: &Path) -> Result<String, RegenError> {
    fs::read(path)
        .map(|bytes| receipt::sha256_hex(&bytes))
        .map_err(|e| RegenError::FailedTurn(format!("{}: {e}", path.display())))
}

/// Truncate on a char boundary (Python `s[:n]` semantics).
fn truncate_chars(s: &str, n: usize) -> String {
    s.char_indices()
        .take_while(|(i, _)| *i < n)
        .map(|(_, c)| c)
        .collect()
}

// --- TurnRequest and prompt ---------------------------------------------------

/// One request behind a fixed signature; `language` drives prompt labels and
/// fences (Req 1.1).
#[derive(Clone)]
pub struct TurnRequest {
    pub scope_root: String,
    pub scope_files: Vec<String>,
    pub surface_path: String,
    pub definitions: String,
    /// Targets arrive through a sorted `serde_json` map (insertion order of
    /// the request file is not preserved; measures read them order-free).
    pub targets: Vec<(String, i64)>,
    pub contract: String,
    pub acceptance_command: String,
    pub feedback: Option<J>,
    pub language: String,
}

impl TurnRequest {
    pub fn load(path: &Path) -> Result<Self, RegenError> {
        let bytes = fs::read(path)
            .map_err(|e| RegenError::FailedTurn(format!("cannot read {}: {e}", path.display())))?;
        let parsed: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| {
            RegenError::FailedTurn(format!("invalid JSON in {}: {e}", path.display()))
        })?;
        let obj = parsed
            .as_object()
            .ok_or_else(|| RegenError::FailedTurn("request must be a JSON object".into()))?;
        let str_field = |key: &str| -> Result<String, RegenError> {
            obj.get(key)
                .and_then(|v| v.as_str())
                .map(str::to_string)
                .ok_or_else(|| RegenError::FailedTurn(format!("request field {key} missing")))
        };
        let files = obj
            .get("scope_files")
            .and_then(|v| v.as_array())
            .ok_or_else(|| RegenError::FailedTurn("request field scope_files missing".into()))?
            .iter()
            .map(|v| {
                v.as_str().map(str::to_string).ok_or_else(|| {
                    RegenError::FailedTurn("scope_files entries must be strings".into())
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let targets = obj
            .get("targets")
            .and_then(|v| v.as_object())
            .ok_or_else(|| RegenError::FailedTurn("request field targets missing".into()))?
            .iter()
            .map(|(k, v)| {
                v.as_i64()
                    .map(|n| (k.clone(), n))
                    .ok_or_else(|| RegenError::FailedTurn(format!("target {k} must be an integer")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(TurnRequest {
            scope_root: str_field("scope_root")?,
            scope_files: files,
            surface_path: str_field("surface_path")?,
            definitions: str_field("definitions")?,
            targets,
            contract: str_field("contract")?,
            acceptance_command: str_field("acceptance_command")?,
            feedback: obj
                .get("feedback")
                .filter(|v| !v.is_null())
                .map(j_from_value),
            language: obj
                .get("language")
                .and_then(|v| v.as_str())
                .unwrap_or("python")
                .to_string(),
        })
    }

    pub fn path(&self, p: &str) -> PathBuf {
        let path = Path::new(p);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            Path::new(&self.scope_root).join(path)
        }
    }

    pub fn targets_j(&self) -> J {
        J::Obj(
            self.targets
                .iter()
                .map(|(k, v)| (k.clone(), J::Int(*v)))
                .collect(),
        )
    }
}

/// One parsed model response (schema of `SCHEMA`).
pub struct TurnResult {
    pub files: Vec<(String, String)>,
    pub structure: Structure,
    pub change_log: String,
}

fn language_label(language: &str) -> String {
    match language {
        "python" => "Python".into(),
        "rust" => "Rust".into(),
        "typescript" => "TypeScript".into(),
        other => {
            let mut chars = other.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().to_string() + &chars.as_str().to_lowercase(),
                None => String::new(),
            }
        }
    }
}

fn system_prompt(language: &str) -> String {
    let label = language_label(language);
    // The frozen regen_turn.py system brief; `{LABEL}` marks the two label
    // substitutions, every other brace is literal JSON punctuation.
    concat!(
        "You are the regenerator in a preregistered software experiment. You rewrite a small set of {LABEL}\n",
        "files behind their public API so that observable behaviour is unchanged and the code is smaller and less\n",
        "entangled. An acceptance arm (recorded traces, a codec harness and the original test suite) is the only\n",
        "authority on behaviour; you cannot run it, so reason from the sources.\n",
        "\n",
        "Rules, all binding:\n",
        "1. Only the scope files listed in the user message may change. Return every scope file you change with its\n",
        "   complete new content; omit files you leave unchanged. Never return any other path.\n",
        "2. Preserve every name in the derived public surface with the same kind (function, class, constant, module)\n",
        "   and the same signature and observable behaviour, including private names that tests import. Names that\n",
        "   tests patch or override (kind \"patch\") must stay module globals that the module's own functions look up\n",
        "   by name at call time.\n",
        "3. Keep SQL statement text, parameters, check order and error messages unchanged unless the surface shows\n",
        "   nothing observes them; the acceptance arm compares statement text and parameters.\n",
        "4. Obey the structural contract in the user message: declare every function of every scope file (changed\n",
        "   or not) as pure core or effect shell in \"structure\", using \"<basename>::<qualname>\" keys such as\n",
        "   \"example.py::function_name\" or \"example.py::Class.method\" — the basename alone (for scope file\n",
        "   \"src/example.py\" the key is \"example.py::function_name\"), two colons, qualified function name.\n",
        "5. Meet the measure targets in the user message.\n",
        "6. If feedback from a previous round is present, treat it as the only information about that round.\n",
        "Answer with exactly one JSON object and nothing else: {\"files\": [{\"path\": \"<scope path>\", \"content\":\n",
        "\"<full file text>\"}], \"structure\": {\"pure\": [...], \"shell\": [...]}, \"change_log\": \"<what changed, per\n",
        "file, and what is deliberately not preserved>\"}.\n",
    )
    .replace("{LABEL}", &label)
}

fn fence(text: &str, lang: &str) -> String {
    let mut ticks = "```".to_string();
    while text.contains(&ticks) {
        ticks.push('`');
    }
    format!("{ticks}{lang}\n{text}\n{ticks}")
}

fn feedback_keys(kind: &str) -> Option<Vec<&'static str>> {
    match kind {
        "failed" => Some(vec!["first_divergence", "measure_gaps"]),
        "accepted_scored" => Some(vec![
            "first_divergence",
            "measure_gaps",
            "surviving_faults",
            "counterexamples",
        ]),
        _ => None,
    }
}

fn feedback_section(fb: &J) -> Result<String, RegenError> {
    let kind = j_str(j_get(fb, "kind").unwrap_or(&J::Null));
    let Some(keys) = feedback_keys(kind) else {
        let sorted: Vec<String> = match fb {
            J::Obj(pairs) => {
                let mut names: Vec<String> = pairs.iter().map(|(k, _)| k.clone()).collect();
                names.sort();
                names
            }
            _ => vec![],
        };
        return Err(RegenError::FailedTurn(format!(
            "feedback must be one of [failed, accepted_scored] with exactly its fields: {}",
            py_repr(&J::arr(sorted.into_iter().map(J::str).collect()))
        )));
    };
    if let J::Obj(pairs) = fb {
        for (k, _) in pairs {
            if k != "kind" && !keys.contains(&k.as_str()) {
                let mut names: Vec<String> = pairs.iter().map(|(k, _)| k.clone()).collect();
                names.sort();
                return Err(RegenError::FailedTurn(format!(
                    "feedback must be one of [failed, accepted_scored] with exactly its fields: {}",
                    py_repr(&J::arr(names.into_iter().map(J::str).collect()))
                )));
            }
        }
    }
    let mut out = vec![
        FEEDBACK_HEADING.to_string(),
        format!("Previous round outcome: {kind}."),
    ];
    for (heading, key) in FEEDBACK_FIELDS.iter().zip(keys.iter()) {
        let v = j_get(fb, key);
        let body = match v {
            Some(J::Str(s)) => s.clone(),
            None | Some(J::Null) => "none".into(),
            Some(other) => {
                // json.dumps(v, indent=1, ensure_ascii=False)
                indent1_dumps(other, false)
            }
        };
        out.push(heading.to_string());
        out.push(body);
    }
    Ok(out.join("\n"))
}

/// `json.dumps(value, indent=1)` parity with selectable `ensure_ascii`
/// (one space per level, `": "`, `",\n"`, empty containers inline).
fn indent1_dumps(value: &J, ascii: bool) -> String {
    fn write(v: &J, depth: usize, out: &mut String, ascii: bool) {
        match v {
            J::Arr(items) if !items.is_empty() => {
                out.push_str("[\n");
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push_str(",\n");
                    }
                    for _ in 0..=depth {
                        out.push(' ');
                    }
                    write(item, depth + 1, out, ascii);
                }
                out.push('\n');
                for _ in 0..depth {
                    out.push(' ');
                }
                out.push(']');
            }
            J::Obj(pairs) if !pairs.is_empty() => {
                out.push_str("{\n");
                for (i, (k, val)) in pairs.iter().enumerate() {
                    if i > 0 {
                        out.push_str(",\n");
                    }
                    for _ in 0..=depth {
                        out.push(' ');
                    }
                    write_str_py(k, out, ascii);
                    out.push_str(": ");
                    write(val, depth + 1, out, ascii);
                }
                out.push('\n');
                for _ in 0..depth {
                    out.push(' ');
                }
                out.push('}');
            }
            J::Arr(_) => out.push_str("[]"),
            J::Obj(_) => out.push_str("{}"),
            other => compact_scalar(other, out, ascii),
        }
    }
    fn compact_scalar(v: &J, out: &mut String, ascii: bool) {
        match v {
            J::Null => out.push_str("null"),
            J::Bool(true) => out.push_str("true"),
            J::Bool(false) => out.push_str("false"),
            J::Int(n) => out.push_str(&n.to_string()),
            J::Float(x) if x.is_finite() => out.push_str(&py_float(*x)),
            J::Float(_) => out.push_str("null"),
            J::Str(s) => write_str_py(s, out, ascii),
            // Scalars only in this branch; containers handled above.
            _ => unreachable!("containers routed above"),
        }
    }
    let mut out = String::new();
    write(value, 0, &mut out, ascii);
    out
}

/// Fixed system brief plus a user message holding the inputs verbatim;
/// fences follow `req.language`.
pub fn build_prompt(req: &TurnRequest) -> Result<(String, String), RegenError> {
    let system = system_prompt(&req.language);
    let root = Path::new(&req.scope_root);
    let read = |p: &Path| -> Result<String, RegenError> {
        fs::read_to_string(p).map_err(|e| RegenError::FailedTurn(format!("{}: {e}", p.display())))
    };
    let mut parts = vec!["## Scope files (the only files that may change)".to_string()];
    for rel in &req.scope_files {
        parts.push(format!("### {rel}"));
        parts.push(fence(&read(&root.join(rel))?, &req.language));
    }
    parts.push("## Derived public surface (public-surface.json)".into());
    parts.push(fence(read(&req.path(&req.surface_path))?.trim(), "json"));
    parts.push("## Definitions".into());
    parts.push(req.definitions.clone());
    parts.push("## Measure targets".into());
    parts.push(fence(&indent1_dumps(&req.targets_j(), true), "json"));
    parts.push("## Structural contract".into());
    parts.push(req.contract.clone());
    parts.push("## Acceptance command (run by the loop, not by you)".into());
    parts.push(fence(&req.acceptance_command, ""));
    if let Some(fb) = &req.feedback {
        parts.push(feedback_section(fb)?);
    }
    Ok((system, format!("{}\n", parts.join("\n\n"))))
}

// --- Endpoint ------------------------------------------------------------------

/// Endpoint configuration; the key is reachable only through `api_key()`.
pub struct Endpoint {
    pub style: String,
    pub base_url: String,
    pub model: String,
    pub host: String,
    key: String,
}

impl std::fmt::Display for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Endpoint(style='{}', host='{}', model='{}', key=<redacted>)",
            self.style, self.host, self.model
        )
    }
}

impl Endpoint {
    pub fn new(style: &str, base_url: &str, model: &str, key: &str) -> Self {
        // urlsplit(base_url).hostname or base_url, minus a port.
        let host = base_url
            .split_once("://")
            .map(|(_, rest)| rest)
            .unwrap_or(base_url)
            .split(['/', '?', '#'])
            .next()
            .unwrap_or(base_url)
            .split(':')
            .next()
            .unwrap_or(base_url)
            .to_lowercase();
        Endpoint {
            style: style.into(),
            base_url: base_url.into(),
            model: model.into(),
            host: if host.is_empty() {
                base_url.to_string()
            } else {
                host
            },
            key: key.into(),
        }
    }

    pub fn api_key(&self) -> &str {
        &self.key
    }

    pub fn public(&self) -> J {
        J::obj(vec![
            ("style", J::str(self.style.clone())),
            ("host", J::str(self.host.clone())),
            ("model", J::str(self.model.clone())),
            ("model_sha256", J::str(sha256_text(&self.model))),
        ])
    }
}

pub const ENV_KEYS: [&str; 4] = [
    "LESSCODE_MODEL_API_STYLE",
    "LESSCODE_MODEL_BASE_URL",
    "LESSCODE_MODEL_NAME",
    "LESSCODE_MODEL_API_KEY",
];

pub fn load_endpoint(path: Option<&Path>) -> Result<Endpoint, String> {
    let home_env = |()| -> PathBuf {
        let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME"));
        match home {
            Ok(h) => Path::new(&h).join(".lesscode").join("model.env"),
            Err(_) => PathBuf::from(".lesscode/model.env"),
        }
    };
    let path = match path {
        Some(p) => p.to_path_buf(),
        None => match std::env::var("LESSCODE_MODEL_ENV") {
            Ok(p) => PathBuf::from(p),
            Err(_) => home_env(()),
        },
    };
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut vals: Vec<(String, String)> = Vec::new();
    for line in text.lines() {
        let body = line.split('#').next().unwrap_or("").trim();
        if body.is_empty() || !body.contains('=') {
            continue;
        }
        let (k, v) = body.split_once('=').unwrap_or((body, ""));
        let mut entry = |k: &str, v: &str| {
            if let Some(e) = vals.iter_mut().find(|(ek, _)| ek == k) {
                e.1 = v.trim().to_string();
            } else {
                vals.push((k.trim().to_string(), v.trim().to_string()));
            }
        };
        entry(k.trim(), v);
    }
    let get = |k: &str| vals.iter().find(|(ek, _)| ek == k).map(|(_, v)| v.clone());
    let missing: Vec<&str> = ENV_KEYS
        .iter()
        .filter(|k| get(k).filter(|v| !v.is_empty()).is_none())
        .copied()
        .collect();
    if !missing.is_empty() {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        return Err(format!(
            "{name}: missing {}",
            py_repr(&J::arr(missing.iter().map(|k| J::str(*k)).collect()))
        ));
    }
    let style = get(ENV_KEYS[0]).unwrap_or_default().to_lowercase();
    if style != "anthropic" && style != "openai" {
        return Err(format!(
            "unknown LESSCODE_MODEL_API_STYLE {}",
            py_repr(&J::str(style.clone()))
        ));
    }
    Ok(Endpoint::new(
        &style,
        &get(ENV_KEYS[1]).unwrap_or_default(),
        &get(ENV_KEYS[2]).unwrap_or_default(),
        &get(ENV_KEYS[3]).unwrap_or_default(),
    ))
}

// --- Request kwargs, schema and the receipt hash set ----------------------------

/// The response JSON schema (frozen `regen_turn.py` SCHEMA).
pub fn schema() -> J {
    let string = || J::obj(vec![("type", J::str("string"))]);
    let strings = || {
        J::obj(vec![
            ("type", J::str("array")),
            ("items", J::obj(vec![("type", J::str("string"))])),
        ])
    };
    let obj = |props: Vec<(&str, J)>| {
        J::obj(vec![
            ("type", J::str("object")),
            ("additionalProperties", J::Bool(false)),
            (
                "required",
                J::arr(props.iter().map(|(k, _)| J::str(*k)).collect()),
            ),
            (
                "properties",
                J::Obj(props.into_iter().map(|(k, v)| (k.to_string(), v)).collect()),
            ),
        ])
    };
    J::obj(vec![
        ("type", J::str("object")),
        ("additionalProperties", J::Bool(false)),
        (
            "required",
            J::arr(vec![
                J::str("files"),
                J::str("structure"),
                J::str("change_log"),
            ]),
        ),
        (
            "properties",
            J::obj(vec![
                (
                    "files",
                    J::obj(vec![
                        ("type", J::str("array")),
                        (
                            "items",
                            obj(vec![("path", string()), ("content", string())]),
                        ),
                    ]),
                ),
                (
                    "structure",
                    obj(vec![("pure", strings()), ("shell", strings())]),
                ),
                ("change_log", string()),
            ]),
        ),
    ])
}

/// The request shape hashed into receipts. `effort: None` omits the knob
/// entirely: weaker models reject `output_config.effort` (G1-R live finding,
/// 2026-11-08); the json_schema output format always stays.
pub fn request_kwargs(
    ep: &Endpoint,
    system: &str,
    user: &str,
    max_tokens: i64,
    effort: Option<&str>,
    temperature: Option<f64>,
    display: Option<&str>,
) -> J {
    let mut output_config = vec![(
        "format".to_string(),
        J::obj(vec![("type", J::str("json_schema")), ("schema", schema())]),
    )];
    if let Some(effort) = effort {
        output_config.push(("effort".to_string(), J::str(effort)));
    }
    let mut pairs = vec![
        ("model".to_string(), J::str(ep.model.clone())),
        ("max_tokens".to_string(), J::int(max_tokens)),
        ("system".to_string(), J::str(system)),
        (
            "messages".to_string(),
            J::arr(vec![J::obj(vec![
                ("role", J::str("user")),
                ("content", J::str(user)),
            ])]),
        ),
        ("output_config".to_string(), J::Obj(output_config)),
    ];
    if let Some(t) = temperature {
        pairs.push(("temperature".to_string(), J::Float(t)));
    }
    if let Some(d) = display {
        pairs.push((
            "thinking".to_string(),
            J::obj(vec![("type", J::str("adaptive")), ("display", J::str(d))]),
        ));
    }
    J::Obj(pairs)
}

/// The receipt hash set (req 6.8); kwargs and response never contain the key.
pub fn hash_set(kwargs: &J, response_json: &str, model_returned: &str) -> J {
    let system = j_str(j_get(kwargs, "system").unwrap_or(&J::Null));
    let user_content = j_get(kwargs, "messages")
        .and_then(|m| match m {
            J::Arr(items) => items.first(),
            _ => None,
        })
        .and_then(|m| j_get(m, "content"))
        .map(j_str)
        .unwrap_or("");
    J::obj(vec![
        ("system_sha256", J::str(sha256_text(system))),
        ("user_sha256", J::str(sha256_text(user_content))),
        (
            "request_sha256",
            J::str(sha256_text(&compact_dumps(kwargs, true, false))),
        ),
        ("response_sha256", J::str(sha256_text(response_json))),
        (
            "model_sent",
            j_get(kwargs, "model").cloned().unwrap_or(J::Null),
        ),
        ("model_returned", J::str(model_returned)),
        ("seed", J::Null),
        ("seed_note", J::str(SEED_NOTE)),
    ])
}

// --- OpenAI-compatible transport (pinned ureq) -----------------------------------

/// One POST; transport-level failures carry the frozen message prefix.
pub type HttpTransport<'a> =
    &'a dyn Fn(&str, &[(String, String)], &[u8]) -> Result<Vec<u8>, String>;

pub fn ureq_post(url: &str, headers: &[(String, String)], body: &[u8]) -> Result<Vec<u8>, String> {
    let agent = ureq::config::Config::builder()
        .timeout_global(Some(Duration::from_secs(3600)))
        .http_status_as_error(false)
        .build()
        .new_agent();
    let mut request = agent.post(url);
    for (name, value) in headers {
        request = request.header(name, value);
    }
    let mut response = request
        .send(body)
        .map_err(|e| format!("openai transport: {e}"))?;
    let status = response.status().as_u16();
    if status >= 400 {
        return Err(format!("openai transport: HTTP {status}"));
    }
    let mut buf = Vec::new();
    response
        .body_mut()
        .as_reader()
        .read_to_end(&mut buf)
        .map_err(|e| format!("openai transport: {e}"))?;
    Ok(buf)
}

/// One OpenAI-compatible chat completion via the transport seam.
fn openai_call(
    ep: &Endpoint,
    system: &str,
    user: &str,
    max_tokens: i64,
    effort: Option<&str>,
    temperature: Option<f64>,
    transport: HttpTransport,
) -> Result<(String, J), String> {
    let body = serde_json::json!({
        "model": ep.model,
        "max_tokens": max_tokens,
        "temperature": temperature.unwrap_or(0.0),
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user},
        ],
        "response_format": {
            "type": "json_schema",
            "json_schema": {"name": "regenerate", "schema": j_to_value(&schema())},
        },
    });
    let url = format!("{}/chat/completions", ep.base_url.trim_end_matches('/'));
    let headers = vec![
        (
            "Authorization".to_string(),
            format!("Bearer {}", ep.api_key()),
        ),
        ("Content-Type".to_string(), "application/json".to_string()),
    ];
    let t0 = Instant::now();
    let bytes = transport(
        &url,
        &headers,
        &serde_json::to_vec(&body).expect("closed body serializes"),
    )?;
    let payload: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|e| format!("openai transport: {e}"))?;
    let choice = payload
        .get("choices")
        .and_then(|c| c.get(0))
        .ok_or_else(|| "KeyError: 'choices'".to_string())?;
    let text = choice
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .ok_or_else(|| "KeyError: 'message'".to_string())?
        .to_string();
    let stop_reason = match choice.get("finish_reason") {
        Some(serde_json::Value::String(s)) => J::str(s.clone()),
        _ => J::Null,
    };
    let meta = J::obj(vec![
        ("mode", J::str("openai-json-http")),
        ("stop_reason", stop_reason),
        (
            "elapsed_s",
            J::Float((t0.elapsed().as_secs_f64() * 100.0).round() / 100.0),
        ),
        (
            "usage",
            payload.get("usage").map(j_from_value).unwrap_or(J::Null),
        ),
        ("max_tokens", J::int(max_tokens)),
        ("effort", effort.map(J::str).unwrap_or(J::Null)),
        ("temperature", J::Float(temperature.unwrap_or(0.0))),
    ]);
    Ok((text, meta))
}

/// One request; returns the response text plus meta carrying the receipt
/// hash set. Only the openai-compatible arm ships here (R36); the
/// anthropic arm's token-count guard stays on the Python substrate.
#[allow(clippy::too_many_arguments)]
pub fn call(
    ep: &Endpoint,
    system: &str,
    user: &str,
    max_tokens: i64,
    effort: Option<&str>,
    temperature: Option<f64>,
    display: Option<&str>,
    transport: HttpTransport,
) -> Result<(String, J), String> {
    let (text, mut meta) =
        openai_call(ep, system, user, max_tokens, effort, temperature, transport)?;
    let kwargs = request_kwargs(ep, system, user, max_tokens, effort, temperature, display);
    if let J::Obj(pairs) = &mut meta {
        pairs.extend(match hash_set(&kwargs, &text, &ep.model) {
            J::Obj(hs) => hs,
            _ => unreachable!("hash_set builds an object"),
        });
    }
    Ok((text, meta))
}

// --- Response parsing and candidate preparation ----------------------------------

pub fn parse_result(text: &str) -> Result<TurnResult, String> {
    let data: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let files = data
        .get("files")
        .and_then(|f| f.as_array())
        .ok_or("response JSON does not match the result shape")?;
    let mut parsed_files = Vec::new();
    for f in files {
        let path = f.get("path").and_then(|p| p.as_str());
        let content = f.get("content").and_then(|c| c.as_str());
        match (path, content) {
            (Some(p), Some(c)) => parsed_files.push((p.to_string(), c.to_string())),
            _ => return Err("response JSON does not match the result shape".into()),
        }
    }
    let structure = data
        .get("structure")
        .ok_or("response JSON does not match the result shape")?;
    let list = |key: &str| -> Result<Vec<String>, String> {
        structure
            .get(key)
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|i| i.as_str().map(str::to_string))
                    .collect()
            })
            .ok_or_else(|| "response JSON does not match the result shape".to_string())
    };
    let change_log = data
        .get("change_log")
        .and_then(|c| c.as_str())
        .ok_or("response JSON does not match the result shape")?;
    Ok(TurnResult {
        files: parsed_files,
        structure: Structure {
            pure: list("pure")?,
            shell: list("shell")?,
        },
        change_log: change_log.to_string(),
    })
}

const COPY_IGNORE: [&str; 4] = [".git", "__pycache__", ".pytest_cache", ".venv"];

fn copy_tree(src: &Path, dst: &Path) -> Result<(), RegenError> {
    fs::create_dir_all(dst)
        .map_err(|e| RegenError::FailedTurn(format!("{}: {e}", dst.display())))?;
    for entry in
        fs::read_dir(src).map_err(|e| RegenError::FailedTurn(format!("{}: {e}", src.display())))?
    {
        let entry = entry.map_err(|e| RegenError::FailedTurn(e.to_string()))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let from = entry.path();
        let to = dst.join(&name);
        if entry.file_type().map(|t| t.is_symlink()).unwrap_or(false) {
            continue; // symlinks are skipped (Windows specimen trees carry none)
        }
        if from.is_dir() {
            if COPY_IGNORE.contains(&name.as_str()) {
                continue;
            }
            copy_tree(&from, &to)?;
        } else {
            fs::copy(&from, &to)
                .map_err(|e| RegenError::FailedTurn(format!("{}: {e}", from.display())))?;
        }
    }
    Ok(())
}

/// Fresh isolated copy of the frozen tree if absent; the scope files must
/// still equal the frozen ones (exit 4).
pub fn prepare_candidate(
    frozen: &Path,
    cand: &Path,
    scope_files: &[String],
) -> Result<J, RegenError> {
    if !cand.exists() {
        copy_tree(frozen, cand)?;
    }
    let mut hashes: Vec<(String, J)> = Vec::new();
    let mut bad: Vec<String> = Vec::new();
    for rel in scope_files {
        let frozen_hash = sha256_file(&frozen.join(rel))?;
        if sha256_file(&cand.join(rel))? != frozen_hash {
            bad.push(rel.clone());
        }
        hashes.push((rel.clone(), J::str(frozen_hash)));
    }
    if !bad.is_empty() {
        return Err(RegenError::CopyMismatch(format!(
            "candidate scope files differ from frozen before writing: {}",
            py_repr(&J::arr(bad.iter().map(J::str).collect()))
        )));
    }
    Ok(J::Obj(hashes))
}

/// `Path(s).as_posix()` for response-borne paths: backslashes fold to `/`
/// and leading `./` segments drop.
fn to_posix(p: &str) -> String {
    let folded = p.replace('\\', "/");
    let stripped = folded
        .split('/')
        .collect::<Vec<_>>()
        .iter()
        .cloned()
        .skip_while(|part| *part == "." || part.is_empty())
        .collect::<Vec<_>>()
        .join("/");
    if stripped.is_empty() && folded == "./" {
        return ".".into();
    }
    stripped
}

/// Write returned files into the candidate; refuse any path outside the
/// scope before writing anything (exit 6).
pub fn write_candidate(
    result: &TurnResult,
    cand: &Path,
    scope_files: &[String],
) -> Result<Vec<J>, RegenError> {
    let allowed: BTreeSet<String> = scope_files.iter().map(|s| to_posix(s)).collect();
    let outside: Vec<String> = result
        .files
        .iter()
        .map(|(p, _)| to_posix(p))
        .filter(|p| !allowed.contains(p))
        .collect();
    if !outside.is_empty() {
        return Err(RegenError::OutOfScope(format!(
            "paths outside scope: {}",
            py_repr(&J::arr(outside.iter().map(J::str).collect()))
        )));
    }
    let mut written = Vec::new();
    for (path, content) in &result.files {
        let rel = to_posix(path);
        let target = cand.join(&rel);
        // keep the replaced file's line-ending convention
        let crlf = target.exists()
            && fs::read(&target)
                .map(|bytes| bytes.windows(2).any(|w| w == b"\r\n"))
                .unwrap_or(false);
        let content_lf = content.replace("\r\n", "\n");
        let data = if crlf {
            content_lf.replace('\n', "\r\n").into_bytes()
        } else {
            content_lf.into_bytes()
        };
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| RegenError::FailedTurn(format!("{}: {e}", parent.display())))?;
        }
        fs::write(&target, &data)
            .map_err(|e| RegenError::FailedTurn(format!("{}: {e}", target.display())))?;
        written.push(J::obj(vec![
            ("path", J::str(rel)),
            ("bytes", J::int(data.len() as i64)),
            ("sha256", J::str(receipt::sha256_hex(&data))),
            ("crlf", J::Bool(crlf)),
        ]));
    }
    Ok(written)
}

/// Write text as UTF-8 LF bytes with a sha256 sidecar; the digest is over
/// the stored bytes (R26 amendment 3). Public for turn-artifact workflows.
pub fn save(dir: &Path, name: &str, text: &str) -> Result<String, RegenError> {
    fs::create_dir_all(dir)
        .map_err(|e| RegenError::FailedTurn(format!("{}: {e}", dir.display())))?;
    fs::write(dir.join(name), text.as_bytes())
        .map_err(|e| RegenError::FailedTurn(format!("{}/{}: {e}", dir.display(), name)))?;
    let digest = sha256_text(text);
    let sidecar = format!("{name}.sha256");
    fs::write(dir.join(&sidecar), format!("{digest}  {name}\n").as_bytes())
        .map_err(|e| RegenError::FailedTurn(format!("{}/{}: {e}", dir.display(), sidecar)))?;
    Ok(digest)
}

// --- Receipt writer (atomic .lock protocol, admission bindings) -------------------

fn utc_now_iso() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let micros = now.subsec_micros();
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}.{micros:06}+00:00",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

/// Howard Hinnant's civil-from-days for the UTC timestamp.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Internalized atomic receipt writer (.lock sidecar protocol) with
/// admission bindings. Every update rewrites the receipt, refreshes
/// `receipt.json.sha256` and `bindings.json` so `lesscode-core admission`
/// verifies the receipt and every bound turn artifact byte-exactly. Labels
/// are validated case-insensitively unique before execution (R26 amend 4).
pub struct ReceiptWriter {
    path: PathBuf,
    base: PathBuf,
    labels: BTreeSet<String>,
}

impl ReceiptWriter {
    pub fn new(path: &Path) -> Result<Self, RegenError> {
        let mut labels = BTreeSet::new();
        if path.exists() {
            let data = Self::read(path)?;
            if let Some(J::Arr(notes)) = j_get(&data, "notes").cloned() {
                for note in &notes {
                    labels.insert(j_str(j_get(note, "key").unwrap_or(&J::Null)).to_lowercase());
                }
            }
        }
        Ok(ReceiptWriter {
            path: path.to_path_buf(),
            base: path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from(".")),
            labels,
        })
    }

    fn empty() -> J {
        J::obj(vec![("facts", J::arr(vec![])), ("notes", J::arr(vec![]))])
    }

    fn read(path: &Path) -> Result<J, RegenError> {
        let bytes = fs::read(path)
            .map_err(|e| RegenError::FailedTurn(format!("{}: {e}", path.display())))?;
        serde_json::from_slice::<serde_json::Value>(&bytes)
            .map(|v| j_from_value(&v))
            .map_err(|e| RegenError::FailedTurn(format!("{}: {e}", path.display())))
    }

    fn load(&self) -> J {
        Self::read(&self.path).unwrap_or_else(|_| Self::empty())
    }

    fn lock_path(&self) -> PathBuf {
        let mut name = self
            .path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        name.push_str(".lock");
        self.path.with_file_name(name)
    }

    fn acquire(&self, lock: &Path) -> Result<(), RegenError> {
        let start = Instant::now();
        loop {
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(lock)
            {
                Ok(_) => return Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    let stale = fs::metadata(lock)
                        .and_then(|m| m.modified())
                        .ok()
                        .and_then(|t| t.elapsed().ok())
                        .map(|age| age.as_secs_f64() > 120.0)
                        .unwrap_or(false);
                    if stale {
                        let _ = fs::remove_file(lock); // stale holder: take over
                        continue;
                    }
                    if start.elapsed() > Duration::from_secs(300) {
                        return Err(RegenError::FailedTurn(format!(
                            "receipt lock held beyond 300s: {}",
                            lock.display()
                        )));
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    return Err(RegenError::FailedTurn(format!("{}: {e}", lock.display())));
                }
            }
        }
    }

    fn write(&self, data: &J, bound: &[J]) -> Result<(), RegenError> {
        // The native writer ends receipt.json with '\n' where the Python
        // writer did not; each sidecar digests the bytes as stored, so both
        // verify under admission.
        receipt::write_receipt(&self.path, data)
            .map_err(|e| RegenError::FailedTurn(format!("{}: {e}", self.path.display())))?;
        fs::create_dir_all(&self.base)
            .map_err(|e| RegenError::FailedTurn(format!("{}: {e}", self.base.display())))?;
        let mut bindings = vec![J::obj(vec![
            ("artifact", J::str("receipt.json")),
            ("sidecars", J::arr(vec![J::str("receipt.json.sha256")])),
        ])];
        bindings.extend(bound.iter().cloned());
        let bindings = J::obj(vec![("bindings", J::arr(bindings))]);
        fs::write(
            self.base.join("bindings.json"),
            indent1_dumps(&bindings, true).as_bytes(),
        )
        .map_err(|e| RegenError::FailedTurn(format!("{}: {e}", self.base.display())))?;
        Ok(())
    }

    fn current_bindings(&self) -> Vec<J> {
        let bindings = self.base.join("bindings.json");
        if !bindings.exists() {
            return Vec::new();
        }
        match Self::read(&bindings) {
            Ok(doc) => match j_get(&doc, "bindings") {
                Some(J::Arr(items)) => items
                    .iter()
                    .filter(|b| j_str(j_get(b, "artifact").unwrap_or(&J::Null)) != "receipt.json")
                    .cloned()
                    .collect(),
                _ => Vec::new(),
            },
            Err(_) => Vec::new(),
        }
    }

    fn append(&self, collection: &str, entry: J, bound: Option<&[J]>) -> Result<(), RegenError> {
        fs::create_dir_all(&self.base)
            .map_err(|e| RegenError::FailedTurn(format!("{}: {e}", self.base.display())))?;
        let lock = self.lock_path();
        self.acquire(&lock)?;
        let result = self.append_locked(collection, entry, bound);
        let _ = fs::remove_file(&lock);
        result
    }

    fn append_locked(
        &self,
        collection: &str,
        entry: J,
        bound: Option<&[J]>,
    ) -> Result<(), RegenError> {
        let mut data = self.load();
        if let J::Obj(pairs) = &mut data {
            let list = pairs
                .iter_mut()
                .find(|(k, _)| k == collection)
                .map(|(_, v)| v);
            match list {
                Some(J::Arr(items)) => items.push(entry),
                _ => pairs.push((collection.to_string(), J::arr(vec![entry]))),
            }
        }
        let bound = bound
            .map(Vec::from)
            .unwrap_or_else(|| self.current_bindings());
        self.write(&data, &bound)
    }

    pub fn add_fact(&self, fact: J, bound: &[J]) -> Result<(), RegenError> {
        let mut pairs = match &fact {
            J::Obj(pairs) => pairs.clone(),
            _ => vec![],
        };
        pairs.insert(0, ("time_utc".to_string(), J::str(utc_now_iso())));
        self.append("facts", J::Obj(pairs), Some(bound))
    }

    pub fn add_note(&mut self, key: &str, value: J, bound: &[J]) -> Result<(), RegenError> {
        if self.labels.contains(&key.to_lowercase()) {
            return Err(RegenError::Usage(format!(
                "receipt label collision (case-insensitive): {}",
                py_repr(&J::str(key.to_string()))
            )));
        }
        self.labels.insert(key.to_lowercase());
        let entry = J::obj(vec![
            ("time_utc", J::str(utc_now_iso())),
            ("key", J::str(key)),
            ("value", value),
        ]);
        self.append("notes", entry, Some(bound))
    }

    pub fn add_bound_artifact(&self, artifact: &Path) -> Result<(), RegenError> {
        let resolved = artifact
            .canonicalize()
            .unwrap_or_else(|_| artifact.to_path_buf());
        let base = self
            .base
            .canonicalize()
            .unwrap_or_else(|_| self.base.clone());
        let rel = resolved
            .strip_prefix(&base)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| resolved.to_string_lossy().into_owned());
        let name = Path::new(&rel)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut bindings: Vec<J> = self
            .current_bindings()
            .into_iter()
            .filter(|b| j_str(j_get(b, "artifact").unwrap_or(&J::Null)) != rel)
            .collect();
        bindings.push(J::obj(vec![
            ("artifact", J::str(rel)),
            ("sidecars", J::arr(vec![J::str(format!("{name}.sha256"))])),
        ]));
        let lock = self.lock_path();
        self.acquire(&lock)?;
        let result = self.write(&self.load(), &bindings);
        let _ = fs::remove_file(&lock);
        result
    }
}

// --- Two-tier reward -------------------------------------------------------------

/// Two-tier reward: tier 1 is static and offline, tier 2 wraps the runtime
/// oracle.
#[derive(Debug, Clone)]
pub struct RewardEvaluation {
    pub score: f64,
    pub tier: i64,
    pub passed_tier1: bool,
    pub failure_kind: Option<String>,
    pub diagnostics: J,
}

impl RewardEvaluation {
    pub fn as_receipt_value(&self) -> J {
        J::obj(vec![
            ("score", J::Float(self.score)),
            ("tier", J::int(self.tier)),
            ("passed_tier1", J::Bool(self.passed_tier1)),
            ("accepted", J::Bool(self.score >= 1.0)),
            (
                "failure_kind",
                self.failure_kind.clone().map(J::str).unwrap_or(J::Null),
            ),
        ])
    }
}

/// Exact decimal `Decimal(str(x))` parse from the shortest float repr.
struct Dec {
    mantissa: i128,
    scale: u32,
}

fn dec_from_f64(x: f64) -> Dec {
    let s = py_float(x);
    let (mantissa_str, exp) = match s.split_once(['e', 'E']) {
        Some((m, e)) => (m.to_string(), e.parse::<i32>().unwrap_or(0)),
        None => (s, 0),
    };
    let (int_part, frac_part) = mantissa_str.split_once('.').unwrap_or((&mantissa_str, ""));
    let digits: String = format!("{int_part}{frac_part}");
    let neg = digits.starts_with('-');
    let digits = digits.trim_start_matches(['-', '+']);
    let mut mantissa: i128 = digits.parse::<i128>().unwrap_or(0);
    let mut scale = frac_part.len() as i32 - exp;
    if neg {
        mantissa = -mantissa;
    }
    while scale > 0 && mantissa % 10 == 0 && mantissa != 0 {
        mantissa /= 10;
        scale -= 1;
    }
    Dec {
        mantissa,
        scale: scale.max(0) as u32,
    }
}

fn pow10(n: u32) -> i128 {
    10i128.checked_pow(n).expect("fraction scale bounded")
}

/// `R = 0.60 + 0.25 * (0.5*g + 0.5*s)` in exact decimal arithmetic, half-up
/// to two decimals (f64 would misround 0.675 to 0.67).
fn tier2_score(g: f64, s: f64) -> f64 {
    let clamp01 = |mut d: Dec| {
        let full = pow10(d.scale);
        d.mantissa = d.mantissa.clamp(0, full);
        d
    };
    let gd = clamp01(dec_from_f64(g));
    let sd = clamp01(dec_from_f64(s));
    let scale = gd.scale.max(sd.scale);
    let gm = gd.mantissa * pow10(scale - gd.scale);
    let sm = sd.mantissa * pow10(scale - sd.scale);
    // value = (60*10^(scale+1) + (gm+sm)*125) / 10^(scale+3);
    // cents = floor(value*100 + 0.5) = (200N + D) / (2D)
    let n = 60 * pow10(scale + 1) + (gm + sm) * 125;
    let d = pow10(scale + 3);
    let cents = (200 * n + d) / (2 * d);
    cents as f64 / 100.0
}

/// Tier-2 RuntimeOracleGate over the recording arm's three-part outcome.
/// `exit_code` is the arm verdict (0 accepted, 1 kill/divergence, 2 invalid
/// arm, 124 timeout). The 0.60-0.85 partial-credit band applies the pinned
/// formula, rounded half-up to two decimals; a missing fraction key counts
/// as 0, never as a pass. Infrastructure outcomes stay
/// `runtime_infrastructure` and are never classified as a kill or an
/// equivalence pass (Req 5.4, 5.6).
pub fn evaluate_tier2_runtime(
    exit_code: i32,
    first_divergence: Option<&str>,
    results: &J,
) -> RewardEvaluation {
    let divergence_j = first_divergence.map(J::str).unwrap_or(J::Null);
    if exit_code != 0 && exit_code != 1 {
        return RewardEvaluation {
            score: 0.50,
            tier: 2,
            passed_tier1: true,
            failure_kind: Some("runtime_infrastructure".into()),
            diagnostics: J::obj(vec![
                ("exit_code", J::int(exit_code as i64)),
                ("first_divergence", divergence_j),
            ]),
        };
    }
    if exit_code == 1 {
        let fraction = |key: &str| -> f64 {
            match j_get(results, key) {
                Some(J::Float(x)) => *x,
                Some(J::Int(n)) => *n as f64,
                _ => 0.0, // missing key counts as 0, never as a pass
            }
        };
        let (g, s) = (
            fraction("seed_group_pass_fraction"),
            fraction("sequence_pass_fraction"),
        );
        return RewardEvaluation {
            score: tier2_score(g, s),
            tier: 2,
            passed_tier1: true,
            failure_kind: Some("kill".into()),
            diagnostics: J::obj(vec![
                ("first_divergence", divergence_j),
                (
                    "formula",
                    J::obj(vec![("g", J::Float(g)), ("s", J::Float(s))]),
                ),
            ]),
        };
    }
    if j_truthy(j_get(results, "measure_gaps")) {
        return RewardEvaluation {
            score: 0.90,
            tier: 2,
            passed_tier1: true,
            failure_kind: Some("gaps".into()),
            diagnostics: J::obj(vec![(
                "measure_gaps",
                j_get(results, "measure_gaps").cloned().unwrap_or(J::Null),
            )]),
        };
    }
    if j_truthy(j_get(results, "surviving_faults")) || j_truthy(j_get(results, "counterexamples")) {
        return RewardEvaluation {
            score: 0.95,
            tier: 2,
            passed_tier1: true,
            failure_kind: Some("scored_faults".into()),
            diagnostics: J::obj(vec![
                (
                    "surviving_faults",
                    j_get(results, "surviving_faults")
                        .cloned()
                        .unwrap_or(J::arr(vec![])),
                ),
                (
                    "counterexamples",
                    j_get(results, "counterexamples")
                        .cloned()
                        .unwrap_or(J::arr(vec![])),
                ),
            ]),
        };
    }
    RewardEvaluation {
        score: 1.00,
        tier: 2,
        passed_tier1: true,
        failure_kind: None,
        diagnostics: J::obj(vec![]),
    }
}

/// One turn's outcome: a failed call/parse/write (`kind` one of transport,
/// stop_reason, invalid_json, out_of_scope) or a written candidate.
pub struct TurnOutcome {
    pub ok: bool,
    pub kind: String,
    pub detail: Option<J>,
    pub result: Option<TurnResult>,
    pub written: Vec<J>,
}

const TIER1_FAILURE_SCORES: [(&str, f64); 4] = [
    ("transport", 0.00),
    ("stop_reason", 0.00),
    ("invalid_json", 0.10),
    ("out_of_scope", 0.20),
];

/// The Tier-1 static ladder: score rungs 0.00-0.50, offline, zero runtime
/// calls. Syntax delegates to `surface::check_syntax`, the surface rung to
/// `surface::verify_candidate_surface`, the structure rung to
/// `measures::measure_written` with the candidate's declared structure, and
/// the static-targets rung to `measures::compute_measure_gaps`.
pub fn evaluate_tier1_static(
    req: &TurnRequest,
    turn_outcome: &TurnOutcome,
    cand_root: &Path,
    surface_report: &J,
    static_targets: &[(String, i64)],
) -> Result<RewardEvaluation, RegenError> {
    if !turn_outcome.ok {
        let kind = if turn_outcome.kind.is_empty() {
            "transport"
        } else {
            turn_outcome.kind.as_str()
        };
        let score = TIER1_FAILURE_SCORES
            .iter()
            .find(|(k, _)| *k == kind)
            .map(|(_, s)| *s)
            .unwrap_or(0.00);
        return Ok(RewardEvaluation {
            score,
            tier: 1,
            passed_tier1: false,
            failure_kind: Some(kind.to_string()),
            diagnostics: J::obj(vec![(
                "detail",
                turn_outcome.detail.clone().unwrap_or(J::Null),
            )]),
        });
    }
    let errors = surface::check_syntax(cand_root, &req.scope_files, &req.language);
    if !errors.is_empty() {
        return Ok(RewardEvaluation {
            score: 0.30,
            tier: 1,
            passed_tier1: false,
            failure_kind: Some("syntax_error".into()),
            diagnostics: J::obj(vec![(
                "errors",
                J::arr(errors.into_iter().map(J::str).collect()),
            )]),
        });
    }
    let verification = surface::verify_candidate_surface(cand_root, surface_report, &[]);
    if !matches!(j_get(&verification, "passed"), Some(J::Bool(true))) {
        let pick = |key: &str| j_get(&verification, key).cloned().unwrap_or(J::Null);
        return Ok(RewardEvaluation {
            score: 0.40,
            tier: 1,
            passed_tier1: false,
            failure_kind: Some("surface".into()),
            diagnostics: J::obj(vec![
                ("missing_symbols", pick("missing_symbols")),
                ("kind_mismatches", pick("kind_mismatches")),
                ("local_rebindings", pick("local_rebindings")),
            ]),
        });
    }
    let written = measures::measure_written(
        cand_root,
        &req.scope_files,
        turn_outcome.result.as_ref().map(|r| &r.structure),
    )
    .map_err(|e| RegenError::FailedTurn(e.0))?;
    if turn_outcome.result.is_some() {
        if let Some(block) = j_get(&written, "structure") {
            if !matches!(j_get(block, "pass"), Some(J::Bool(true))) {
                let mut why: Vec<J> = Vec::new();
                if let Some(J::Obj(pure)) = j_get(block, "pure") {
                    for (_, v) in pure {
                        if !matches!(j_get(v, "pass"), Some(J::Bool(true))) {
                            if let Some(J::Arr(reasons)) = j_get(v, "why") {
                                why.extend(reasons.iter().cloned());
                            }
                        }
                    }
                }
                let sites: Vec<J> = match j_get(block, "write_sites") {
                    Some(J::Arr(sites)) => sites
                        .iter()
                        .filter(|s| !matches!(j_get(s, "pass"), Some(J::Bool(true))))
                        .cloned()
                        .collect(),
                    _ => vec![],
                };
                return Ok(RewardEvaluation {
                    score: 0.45,
                    tier: 1,
                    passed_tier1: false,
                    failure_kind: Some("structure".into()),
                    diagnostics: J::obj(vec![("why", J::arr(why)), ("write_sites", J::arr(sites))]),
                });
            }
        }
    }
    let gaps = measures::compute_measure_gaps(
        &written,
        None,
        &J::obj(vec![("total", J::obj(vec![]))]),
        static_targets,
    );
    if j_truthy(Some(&gaps)) {
        return Ok(RewardEvaluation {
            score: 0.49,
            tier: 1,
            passed_tier1: false,
            failure_kind: Some("static_targets".into()),
            diagnostics: J::obj(vec![("gaps", gaps)]),
        });
    }
    Ok(RewardEvaluation {
        score: 0.50,
        tier: 1,
        passed_tier1: true,
        failure_kind: None,
        diagnostics: J::obj(vec![]),
    })
}

// --- Feedback and boundary rules ---------------------------------------------------

/// Feedback carries divergence identities and measure gaps only — never
/// held-out sequence names (Req 7.5 separation).
pub fn enforce_feedback_boundary(
    feedback: &J,
    held_out_names: &[String],
) -> Result<(), RegenError> {
    let blob = compact_dumps(feedback, false, false);
    for name in held_out_names {
        if !name.is_empty() && blob.contains(name.as_str()) {
            return Err(RegenError::FeedbackBoundary(format!(
                "feedback payload carries held-out sequence name {name}; refusing to dispatch"
            )));
        }
    }
    Ok(())
}

/// Training and held-out evaluation scopes must not intersect (Req 7.5).
pub fn verify_holdout_disjoint(
    train_scopes: &BTreeSet<String>,
    eval_scopes: &BTreeSet<String>,
) -> Result<(), RegenError> {
    let overlap: Vec<&String> = train_scopes.intersection(eval_scopes).collect();
    if !overlap.is_empty() {
        return Err(RegenError::Usage(format!(
            "optimizer training scope intersects held-out evaluation scope: {}",
            py_repr(&J::arr(
                overlap.iter().map(|s| J::str(s.as_str())).collect()
            ))
        )));
    }
    Ok(())
}

/// Static failure feedback for the next inner rollout; the rung's own
/// diagnostics ride along so the model learns the concrete violation.
fn static_feedback(reward: &RewardEvaluation) -> J {
    let mut parts = vec![format!(
        "static:{}",
        reward.failure_kind.clone().unwrap_or_default()
    )];
    for key in ["detail", "errors", "why", "write_sites"] {
        if let Some(v) = j_get(&reward.diagnostics, key) {
            if j_truthy(Some(v)) {
                let items: Vec<&J> = match v {
                    J::Arr(items) => items.iter().take(3).collect(),
                    other => vec![other],
                };
                parts.extend(items.iter().map(|x| py_str(x)));
            }
        }
    }
    let divergence = truncate_chars(&parts.join(" "), 2000);
    J::obj(vec![
        ("kind", J::str("failed")),
        ("first_divergence", J::str(divergence)),
        (
            "measure_gaps",
            j_get(&reward.diagnostics, "gaps")
                .cloned()
                .unwrap_or_else(|| J::obj(vec![])),
        ),
    ])
}

fn turn_feedback_from(record: &RolloutRecord) -> Option<J> {
    if record.failure_kind.is_none() && !j_truthy(Some(&record.diagnostics)) {
        return None;
    }
    let divergence = j_get(&record.diagnostics, "first_divergence")
        .map(py_str)
        .filter(|s| !s.is_empty())
        .or_else(|| record.failure_kind.clone())
        .unwrap_or_else(|| record.label.clone());
    Some(J::obj(vec![
        ("kind", J::str("failed")),
        (
            "first_divergence",
            J::str(truncate_chars(&divergence, 2000)),
        ),
        (
            "measure_gaps",
            j_get(&record.diagnostics, "measure_gaps")
                .cloned()
                .unwrap_or_else(|| J::obj(vec![])),
        ),
    ]))
}

fn known_obligations(record: &RolloutRecord) -> BTreeSet<String> {
    let mut obligations = BTreeSet::new();
    if let Some(kind) = &record.failure_kind {
        obligations.insert(kind.clone());
    }
    if let Some(divergence) = j_get(&record.diagnostics, "first_divergence") {
        let d = py_str(divergence);
        if !d.is_empty() {
            obligations.insert(d);
        }
    }
    if let Some(J::Arr(missing)) = j_get(&record.diagnostics, "missing_symbols") {
        for pair in missing {
            if let J::Arr(items) = pair {
                if let Some(name) = items.get(1) {
                    obligations.insert(j_str(name).to_string());
                }
            }
        }
    }
    if let Some(J::Obj(gaps)) = j_get(&record.diagnostics, "gaps") {
        for (key, _) in gaps {
            obligations.insert(key.clone());
        }
    }
    obligations
}

fn discharged_obligations(record: &RolloutRecord, _resolved: &BTreeSet<String>) -> BTreeSet<String> {
    if record.score < 1.0 {
        return BTreeSet::new();
    }
    known_obligations(record)
}

// --- Adaptive schedule -------------------------------------------------------------

const TEMPERATURE_STEPS: [f64; 3] = [0.0, 0.2, 0.4];

/// Adaptive schedule for the next inner rollout; empty when nothing needs
/// adapting.
#[derive(Default)]
pub struct Adaptation {
    pub effort: Option<String>,
    pub directive: Option<String>,
    pub temperature: Option<f64>,
    pub inject_missing_symbols: Option<Vec<String>>,
}

pub fn next_rollout_params(prev: &RewardEvaluation, attempt: i64) -> Adaptation {
    if prev.passed_tier1 && prev.score >= 0.50 {
        return Adaptation::default();
    }
    let mut params = Adaptation::default();
    if prev.score == 0.00 {
        // transport or thinking-budget stop: cheaper effort, concise output
        params.effort = Some("medium".into());
        params.directive = Some(
            "Be concise: emit the JSON object directly, without extended internal monologue, within the budget."
                .into(),
        );
    }
    if (0.10..=0.49).contains(&prev.score) {
        let idx = (attempt as usize).min(TEMPERATURE_STEPS.len() - 1);
        params.temperature = Some(TEMPERATURE_STEPS[idx]);
    }
    if prev.failure_kind.as_deref() == Some("surface") {
        if let Some(J::Arr(missing)) = j_get(&prev.diagnostics, "missing_symbols") {
            let mut names: BTreeSet<String> = BTreeSet::new();
            for pair in missing {
                if let J::Arr(items) = pair {
                    if let Some(name) = items.get(1) {
                        names.insert(j_str(name).to_string());
                    }
                }
            }
            if !names.is_empty() {
                params.inject_missing_symbols = Some(names.into_iter().collect());
            }
        }
    }
    params
}

/// Preregistered per-attempt knobs; the directive rides the prompt, not the
/// API surface (dispatch fix, 2026-11-08).
#[derive(Clone)]
pub struct AttemptParams {
    pub max_tokens: i64,
    pub effort: String,
    pub temperature: Option<f64>,
    pub display: Option<String>,
    pub directive: Option<String>,
}

impl AttemptParams {
    fn base() -> Self {
        AttemptParams {
            max_tokens: 64_000,
            effort: "high".into(),
            temperature: Some(0.0),
            display: None,
            directive: None,
        }
    }

    fn as_j(&self) -> J {
        let mut pairs = vec![
            ("max_tokens", J::int(self.max_tokens)),
            ("effort", J::str(self.effort.clone())),
            (
                "temperature",
                self.temperature.map(J::Float).unwrap_or(J::Null),
            ),
            (
                "display",
                self.display.clone().map(J::str).unwrap_or(J::Null),
            ),
        ];
        if let Some(d) = &self.directive {
            pairs.push(("directive", J::str(d.clone())));
        }
        J::obj(pairs)
    }
}

// --- Refine/BestOfN controller -------------------------------------------------------

/// Bounded turn/rollout/retry budget; every field is preregistered in the
/// receipt.
#[derive(Clone)]
pub struct RefineBudget {
    pub max_turns: i64,
    pub max_static_retries: i64,
    pub n_rollouts_per_turn: i64,
    pub max_input_tokens_per_call: i64,
    pub max_total_tokens: i64,
    pub no_progress_limit: i64,
    pub threshold: f64,
}

impl Default for RefineBudget {
    fn default() -> Self {
        RefineBudget {
            max_turns: 4,
            max_static_retries: 3,
            n_rollouts_per_turn: 1,
            max_input_tokens_per_call: MAX_INPUT_TOKENS,
            max_total_tokens: 600_000,
            no_progress_limit: 2,
            threshold: 1.0,
        }
    }
}

/// One recorded rollout (BestOfN element); hashes and diagnostics come from
/// the turn.
#[derive(Debug)]
pub struct RolloutRecord {
    pub label: String,
    pub turn: i64,
    pub rollout_id: i64,
    pub attempt: i64,
    pub score: f64,
    pub tier: i64,
    pub failure_kind: Option<String>,
    pub hashes: J,
    pub diagnostics: J,
}

impl RolloutRecord {
    fn as_dict(&self) -> J {
        J::obj(vec![
            ("label", J::str(self.label.clone())),
            ("turn", J::int(self.turn)),
            ("rollout_id", J::int(self.rollout_id)),
            ("attempt", J::int(self.attempt)),
            ("score", J::Float(self.score)),
            ("tier", J::int(self.tier)),
            (
                "failure_kind",
                self.failure_kind.clone().map(J::str).unwrap_or(J::Null),
            ),
            ("hashes", self.hashes.clone()),
            ("diagnostics", self.diagnostics.clone()),
        ])
    }
}

/// Validity, compression and adoption stay separate; adoption is never
/// claimed (Req 6.7).
#[derive(Debug)]
pub struct RefineOutcome {
    pub status: String,
    pub best: RolloutRecord,
    pub validity: bool,
    pub compression_met: bool,
    pub adopted: bool,
    pub unresolved_obligations: Vec<String>,
    pub rollout_history: Vec<J>,
    pub usage: J,
}

/// Tier-2 default: one oracle-script execution inside the candidate tree
/// through the recording driver's deadline-killed `call`. The oracle writes
/// `oracle-verdict.json` (`{"ok", "passed", "total", "failures"}`) under the
/// candidate root — the g1r_live stdout protocol moves to a verdict file so
/// the engine never scrapes process output (port delta, task 8.5).
pub struct RuntimeOracle {
    pub python: PathBuf,
    pub script_rel: String,
    pub timeout_s: u64,
}

impl RuntimeOracle {
    pub fn evaluate(&self, cand_root: &Path) -> (i32, Option<String>, J) {
        if !cand_root.join(&self.script_rel).is_file() {
            return (
                2,
                Some(format!("oracle script absent: {}", self.script_rel)),
                J::obj(vec![]),
            );
        }
        let verdict_path = cand_root.join("oracle-verdict.json");
        let _ = fs::remove_file(&verdict_path);
        let deadline = Instant::now() + Duration::from_secs(self.timeout_s);
        let code = recording::call(
            self.python.to_string_lossy().as_ref(),
            &[self.script_rel.as_str()],
            cand_root,
            &[],
            Some(deadline),
        );
        if code != 0 && code != 1 {
            return (
                code,
                Some(format!("oracle process failed or timed out (exit {code})")),
                J::obj(vec![]),
            );
        }
        let payload = fs::read(&verdict_path)
            .ok()
            .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
        let Some(payload) = payload else {
            return (
                2,
                Some(format!(
                    "oracle output unparsable: no verdict at oracle-verdict.json (exit {code})"
                )),
                J::obj(vec![]),
            );
        };
        let ok = payload.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
        let failures = payload
            .get("failures")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let has_failures = match &failures {
            serde_json::Value::Array(a) => !a.is_empty(),
            serde_json::Value::Null => false,
            _ => true,
        };
        let total = payload
            .get("total")
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
            .max(1);
        let passed = payload.get("passed").and_then(|v| v.as_i64()).unwrap_or(0);
        if code == 0 && ok && !has_failures && passed >= total {
            return (0, None, J::obj(vec![]));
        }
        let field = |key: &str| -> String {
            failures
                .get(0)
                .and_then(|f| f.get(key))
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .unwrap_or_else(|| "None".into())
        };
        let divergence = truncate_chars(
            &format!(
                "case {} ({}): {}",
                field("case"),
                field("name"),
                field("error")
            ),
            400,
        );
        let fraction = ((passed as f64 / total as f64) * 10_000.0).round() / 10_000.0;
        (
            1,
            Some(divergence),
            J::obj(vec![
                ("seed_group_pass_fraction", J::Float(fraction)),
                ("sequence_pass_fraction", J::Float(fraction)),
            ]),
        )
    }
}

/// Tier-2 evaluator seam: returns the arm verdict triple
/// (exit code, first divergence, results map).
pub type RuntimeEvaluator<'a> = &'a mut dyn FnMut(&Path) -> (i32, Option<String>, J);
/// Model-call seam: returns (response text, receipt meta).
pub type ModelCall<'a> =
    &'a mut dyn FnMut(&str, &str, &AttemptParams) -> Result<(String, J), String>;

/// Stdlib Refine/BestOfN controller: outer turns x BestOfN rollouts x inner
/// static retries. The first attempt of the run resumes an existing
/// candidate tree by hash identity (exit 4 on mismatch); every later
/// attempt restores the candidate from the frozen tree before the next
/// write. Static failures never launch the runtime oracle; tier 2 runs only
/// after a tier-1 pass. Turn feedback is boundary-checked against
/// `held_out_names` before the next prompt is built. No-progress and token
/// budgets stop on outer turns; exhaustion retains the best trajectory.
#[allow(clippy::too_many_arguments)]
pub fn run_refine_loop(
    req: &TurnRequest,
    cand_root: &Path,
    frozen_root: &Path,
    receipt_path: &Path,
    base_label: &str,
    budget: &RefineBudget,
    mut runtime_evaluator: Option<RuntimeEvaluator>,
    call_fn: ModelCall,
    held_out_names: &[String],
    static_targets: Option<&[(String, i64)]>,
) -> Result<RefineOutcome, RegenError> {
    let surface_bytes = fs::read(req.path(&req.surface_path)).map_err(|e| {
        RegenError::FailedTurn(format!("{}: {e}", req.path(&req.surface_path).display()))
    })?;
    let surface_report: J = serde_json::from_slice::<serde_json::Value>(&surface_bytes)
        .map(|v| j_from_value(&v))
        .map_err(|e| RegenError::FailedTurn(format!("invalid surface report: {e}")))?;
    let mut writer = ReceiptWriter::new(receipt_path)?;
    let mut history: Vec<J> = Vec::new();
    let mut best: Option<RolloutRecord> = None;
    let mut best_files: Vec<(String, Option<Vec<u8>>)> = Vec::new();
    let (mut usage_in, mut usage_out, mut usage_calls) = (0i64, 0i64, 0i64);
    let mut no_progress: i64 = 0;
    let mut resolved: BTreeSet<String> = BTreeSet::new();
    let mut all_obligations: BTreeSet<String> = BTreeSet::new();
    let mut turn_feedback: Option<J> = None;
    let mut status = "exhausted";
    let mut first_attempt = true;
    let targets: &[(String, i64)] = static_targets.unwrap_or(&[]);
    for turn in 1..=budget.max_turns {
        let mut turn_best: Option<RolloutRecord> = None;
        let mut turn_best_files: Vec<(String, Option<Vec<u8>>)> = Vec::new();
        for rollout in 1..=budget.n_rollouts_per_turn {
            let mut attempt_params = AttemptParams::base();
            let mut record: Option<RolloutRecord> = None;
            let mut attempts: Vec<J> = Vec::new();
            let label = format!("{base_label}-t{turn}-r{rollout}");
            for attempt in 1..=budget.max_static_retries {
                if first_attempt {
                    prepare_candidate(frozen_root, cand_root, &req.scope_files)?; // resume by hash identity
                    first_attempt = false;
                } else {
                    let _ = fs::remove_dir_all(cand_root);
                    prepare_candidate(frozen_root, cand_root, &req.scope_files)?;
                }
                let mut working = req.clone();
                working.feedback = turn_feedback.clone();
                let (system, built_user) = build_prompt(&working)?;
                let mut user = built_user;
                if let Some(directive) = &attempt_params.directive {
                    // concise-output recovery reaches the model in the user
                    // prompt; API adapters consume only base knobs
                    user.push_str(directive);
                    user.push('\n');
                }
                let mut meta: Option<J> = None;
                let outcome: TurnOutcome = match call_fn(&system, &user, &attempt_params) {
                    Ok((text, call_meta)) => {
                        meta = Some(call_meta);
                        let stop = j_get(meta.as_ref().expect("just set"), "stop_reason").cloned();
                        let stop_ok =
                            matches!(&stop, Some(J::Str(s)) if s == "end_turn" || s == "stop");
                        if !stop_ok {
                            TurnOutcome {
                                ok: false,
                                kind: "stop_reason".into(),
                                detail: Some(stop.unwrap_or(J::Null)),
                                result: None,
                                written: vec![],
                            }
                        } else {
                            match parse_result(&text)
                                .map_err(RegenError::FailedTurn)
                                .and_then(|result| {
                                    write_candidate(&result, cand_root, &req.scope_files)
                                        .map(|written| (result, written))
                                }) {
                                Ok((result, written)) => TurnOutcome {
                                    ok: true,
                                    kind: String::new(),
                                    detail: None,
                                    result: Some(result),
                                    written,
                                },
                                Err(RegenError::OutOfScope(e)) => TurnOutcome {
                                    ok: false,
                                    kind: "out_of_scope".into(),
                                    detail: Some(J::str(e)),
                                    result: None,
                                    written: vec![],
                                },
                                Err(RegenError::FailedTurn(e)) => TurnOutcome {
                                    ok: false,
                                    kind: "invalid_json".into(),
                                    detail: Some(J::str(e)),
                                    result: None,
                                    written: vec![],
                                },
                                Err(other) => return Err(other),
                            }
                        }
                    }
                    Err(e) => TurnOutcome {
                        ok: false,
                        kind: "transport".into(),
                        detail: Some(J::str(e)),
                        result: None,
                        written: vec![],
                    },
                };
                if let Some(meta) = &meta {
                    if let Some(usage) = j_get(meta, "usage") {
                        let token = |key: &str| -> i64 {
                            match j_get(usage, key) {
                                Some(J::Int(n)) => *n,
                                Some(J::Float(x)) => *x as i64,
                                _ => 0,
                            }
                        };
                        usage_in += token("input_tokens");
                        usage_out += token("output_tokens");
                    }
                }
                usage_calls += 1;
                let mut reward =
                    evaluate_tier1_static(req, &outcome, cand_root, &surface_report, targets)?;
                if reward.passed_tier1 && reward.score >= 0.50 {
                    if let Some(evaluator) = runtime_evaluator.as_mut() {
                        let (exit_code, divergence, results) = evaluator(cand_root);
                        reward = evaluate_tier2_runtime(exit_code, divergence.as_deref(), &results);
                    }
                }
                let hashes = match &meta {
                    Some(J::Obj(pairs)) => J::Obj(
                        pairs
                            .iter()
                            .filter(|(k, _)| k.ends_with("_sha256"))
                            .cloned()
                            .collect(),
                    ),
                    _ => J::obj(vec![]),
                };
                let rec = RolloutRecord {
                    label: label.clone(),
                    turn,
                    rollout_id: rollout,
                    attempt,
                    score: reward.score,
                    tier: reward.tier,
                    failure_kind: reward.failure_kind.clone(),
                    hashes,
                    diagnostics: reward.diagnostics.clone(),
                };
                let mut attempt_note = vec![
                    ("attempt", J::int(attempt)),
                    ("preregistered_params", attempt_params.as_j()),
                    ("reward", reward.as_receipt_value()),
                    ("diagnostics", reward.diagnostics.clone()),
                ];
                if let Some(meta) = &meta {
                    for key in ["stop_reason", "usage"] {
                        if let Some(v) = j_get(meta, key) {
                            attempt_note.push((key, v.clone()));
                        }
                    }
                }
                attempts.push(J::obj(attempt_note));
                history.push(rec.as_dict());
                all_obligations.extend(known_obligations(&rec));
                record = Some(rec);
                if reward.score >= 0.50 {
                    break; // the static repair chain ends on the first tier-1 pass
                }
                turn_feedback = Some(static_feedback(&reward));
                let adaptation = next_rollout_params(&reward, attempt - 1);
                if let Some(effort) = &adaptation.effort {
                    attempt_params.effort = effort.clone();
                }
                if let Some(temperature) = adaptation.temperature {
                    attempt_params.temperature = Some(temperature);
                }
                if let Some(directive) = &adaptation.directive {
                    attempt_params.directive = Some(directive.clone());
                }
                if let Some(names) = &adaptation.inject_missing_symbols {
                    if let Some(J::Obj(pairs)) = turn_feedback.as_mut() {
                        for (k, v) in pairs.iter_mut() {
                            if k != "first_divergence" {
                                continue;
                            }
                            if let J::Str(d) = v {
                                let appended = format!("{d} missing: {}", names.join(", "));
                                *v = J::str(appended);
                            }
                        }
                    }
                }
            }
            if !attempts.is_empty() {
                writer.add_note(
                    &format!("{label}-turn"),
                    J::obj(vec![("attempts", J::arr(attempts))]),
                    &[],
                )?;
            }
            if let Some(rec) = record {
                let improve = turn_best
                    .as_ref()
                    .map(|b| rec.score > b.score)
                    .unwrap_or(true);
                if improve {
                    turn_best = Some(rec);
                    turn_best_files = req
                        .scope_files
                        .iter()
                        .map(|f| (f.clone(), fs::read(cand_root.join(f)).ok()))
                        .collect();
                }
            }
        }
        let Some(turn_best) = turn_best else {
            continue;
        };
        let improved = best
            .as_ref()
            .map(|b| turn_best.score > b.score)
            .unwrap_or(true);
        let discharged = discharged_obligations(&turn_best, &resolved);
        no_progress = if improved || !discharged.is_empty() {
            0
        } else {
            no_progress + 1
        };
        if improved {
            best = Some(turn_best);
            best_files = turn_best_files;
        }
        resolved.extend(discharged);
        let best_score = best.as_ref().expect("set above").score;
        if best_score >= budget.threshold {
            status = "accepted";
            break;
        }
        if no_progress >= budget.no_progress_limit {
            break;
        }
        if usage_in + usage_out >= budget.max_total_tokens {
            break;
        }
        if runtime_evaluator.is_some() {
            if let Some(fb) = best.as_ref().and_then(turn_feedback_from) {
                enforce_feedback_boundary(&fb, held_out_names)?;
                turn_feedback = Some(fb);
            }
        }
    }
    let best =
        best.ok_or_else(|| RegenError::FailedTurn("no rollout recorded; budget too small".into()))?;
    for (rel, data) in &best_files {
        let path = cand_root.join(rel);
        if let Some(bytes) = data {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&path, bytes);
        } else if path.exists() {
            let _ = fs::remove_file(&path);
        }
    }
    let validity = best.score >= budget.threshold;
    let unresolved: Vec<String> = all_obligations
        .difference(&resolved)
        .cloned()
        .collect();
    Ok(RefineOutcome {
        status: status.into(),
        best,
        validity,
        compression_met: static_targets.is_none(),
        adopted: false, // adoption is never claimed by the loop (Req 6.7)
        unresolved_obligations: unresolved,
        rollout_history: history,
        usage: J::obj(vec![
            ("input_tokens", J::int(usage_in)),
            ("output_tokens", J::int(usage_out)),
            ("calls", J::int(usage_calls)),
        ]),
    })
}

/// Live endpoint call with the pinned transport (the loop's default `call_fn`).
pub fn default_call_fn(
    system: &str,
    user: &str,
    params: &AttemptParams,
) -> Result<(String, J), String> {
    let ep = load_endpoint(None)?;
    call(
        &ep,
        system,
        user,
        params.max_tokens,
        Some(params.effort.as_str()),
        params.temperature,
        params.display.as_deref(),
        &ureq_post,
    )
}

// --- CLI (frozen prompt/run contract) -------------------------------------------------

use clap::Subcommand;

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum RegenerateCommand {
    /// Build and hash the prompt for a request.
    Prompt {
        #[arg(long)]
        request: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    /// One recorded turn: prompt, call, write, receipt.
    Run {
        #[arg(long)]
        request: PathBuf,
        #[arg(long)]
        cand: PathBuf,
        #[arg(long)]
        receipt: PathBuf,
        #[arg(long)]
        label: String,
        #[arg(long)]
        frozen: PathBuf,
        #[arg(long)]
        turns: PathBuf,
        #[arg(long = "model-env")]
        model_env: Option<PathBuf>,
        #[arg(long, default_value_t = 64_000)]
        max_tokens: i64,
        #[arg(long, default_value = "high")]
        effort: String,
        #[arg(long)]
        temperature: Option<f64>,
        #[arg(long, default_value_t = 1)]
        rollout_id: i64,
        #[arg(long)]
        thinking_display: Option<String>,
    },
    /// Multi-turn Refine/BestOfN controller (driven by the spike, not CLI).
    Refine,
}

fn rel_under(turn_dir: &Path, receipt: &Path, name: &str) -> String {
    let resolved_turn = turn_dir
        .canonicalize()
        .unwrap_or_else(|_| turn_dir.to_path_buf());
    let resolved_base = receipt
        .parent()
        .map(|p| p.canonicalize().unwrap_or_else(|_| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    resolved_turn
        .strip_prefix(&resolved_base)
        .map(|p| format!("{}/{}", p.to_string_lossy().replace('\\', "/"), name))
        .unwrap_or_else(|_| name.to_string())
}

fn cmd_prompt(request: &Path, out: &Path) -> Result<u8, RegenError> {
    let req = TurnRequest::load(request)?;
    let (system, user) = build_prompt(&req)?;
    fs::create_dir_all(out)
        .map_err(|e| RegenError::FailedTurn(format!("{}: {e}", out.display())))?;
    let hashes = J::obj(vec![
        ("system_sha256", J::str(save(out, "system.txt", &system)?)),
        ("user_sha256", J::str(save(out, "user.txt", &user)?)),
        ("system_chars", J::int(system.chars().count() as i64)),
        ("user_chars", J::int(user.chars().count() as i64)),
    ]);
    fs::write(
        out.join("prompt-hashes.json"),
        indent1_dumps(&hashes, true).as_bytes(),
    )
    .map_err(|e| RegenError::FailedTurn(format!("{}: {e}", out.display())))?;
    println!("{}", receipt::dumps(&hashes));
    Ok(EXIT_OK)
}

fn cmd_run(a: &RegenerateCommand) -> Result<u8, RegenError> {
    let RegenerateCommand::Run {
        request,
        cand,
        receipt: receipt_path,
        label,
        frozen,
        turns,
        model_env,
        max_tokens,
        effort,
        temperature,
        rollout_id,
        thinking_display,
    } = a
    else {
        return Err(RegenError::Usage(
            "run requires the run subcommand arguments".into(),
        ));
    };
    let mut writer = ReceiptWriter::new(receipt_path)?;
    let req = TurnRequest::load(request)?;
    let turn_dir = turns.join(label);
    let (system, user) = build_prompt(&req)?;
    let prompts = J::obj(vec![
        (
            "system.txt",
            J::str(save(&turn_dir, "system.txt", &system)?),
        ),
        ("user.txt", J::str(save(&turn_dir, "user.txt", &user)?)),
    ]);
    let ep = load_endpoint(model_env.as_deref()).map_err(RegenError::FailedTurn)?;
    writer.add_fact(
        J::obj(vec![
            ("key", J::str(format!("{label}-inputs"))),
            (
                "apparatus",
                J::obj(vec![(
                    "regenerator.rs",
                    J::str(receipt::sha256_hex(include_bytes!("regenerator.rs"))),
                )]),
            ),
            (
                "request",
                J::Obj(vec![(
                    request.to_string_lossy().into_owned(),
                    J::str(sha256_file(request)?),
                )]),
            ),
            (
                "surface",
                J::str(sha256_file(&req.path(&req.surface_path))?),
            ),
            (
                "frozen_scope",
                prepare_candidate(frozen, cand, &req.scope_files)?,
            ),
            (
                "candidate_root",
                J::str(cand.to_string_lossy().into_owned()),
            ),
            ("endpoint", ep.public()),
            (
                "feedback_kind",
                req.feedback
                    .as_ref()
                    .and_then(|fb| j_get(fb, "kind").cloned())
                    .unwrap_or(J::Null),
            ),
            ("language", J::str(req.language.clone())),
            (
                "preregistered_params",
                J::obj(vec![
                    ("max_tokens", J::int(*max_tokens)),
                    ("effort", J::str(effort.clone())),
                    ("temperature", temperature.map(J::Float).unwrap_or(J::Null)),
                    ("rollout_id", J::int(*rollout_id)),
                    ("seed", J::Null),
                ]),
            ),
            ("prompts", prompts),
        ]),
        &[
            J::obj(vec![
                (
                    "artifact",
                    J::str(rel_under(&turn_dir, receipt_path, "system.txt")),
                ),
                ("sidecars", J::arr(vec![J::str("system.txt.sha256")])),
            ]),
            J::obj(vec![
                (
                    "artifact",
                    J::str(rel_under(&turn_dir, receipt_path, "user.txt")),
                ),
                ("sidecars", J::arr(vec![J::str("user.txt.sha256")])),
            ]),
        ],
    )?;

    // failed turns stay in the receipt; no retry here
    let failed = |writer: &mut ReceiptWriter, reason: &str, code: u8| -> Result<u8, RegenError> {
        writer.add_note(
            &format!("{label}-failed"),
            J::obj(vec![
                ("reason", J::str(truncate_chars(reason, 2000))),
                ("outcome", J::str("failed")),
                ("exit", J::int(code as i64)),
            ]),
            &[],
        )?;
        Ok(code)
    };

    let call_result = call(
        &ep,
        &system,
        &user,
        *max_tokens,
        Some(effort.as_str()),
        *temperature,
        thinking_display.as_deref(),
        &ureq_post,
    );
    let (text, mut meta) = match call_result {
        Ok(ok) => ok,
        Err(e) => return failed(&mut writer, &e, EXIT_FAILED_TURN),
    };
    let saved = save(&turn_dir, "response.json", &text)?;
    if let J::Obj(pairs) = &mut meta {
        pairs.push(("response_sha256_saved".to_string(), J::str(saved)));
    }
    writer.add_note(
        &format!("{label}-turn"),
        meta.clone(),
        &[J::obj(vec![
            (
                "artifact",
                J::str(rel_under(&turn_dir, receipt_path, "response.json")),
            ),
            ("sidecars", J::arr(vec![J::str("response.json.sha256")])),
        ])],
    )?;
    match j_get(&meta, "stop_reason") {
        Some(J::Str(s)) if s == "end_turn" || s == "stop" => {}
        _ => {
            return failed(
                &mut writer,
                &format!(
                    "stop_reason {}",
                    py_str(j_get(&meta, "stop_reason").unwrap_or(&J::Null))
                ),
                EXIT_FAILED_TURN,
            )
        }
    }
    let parsed = parse_result(&text).map_err(RegenError::FailedTurn);
    let result = match parsed {
        Ok(result) => result,
        Err(e) => return failed(&mut writer, e.message(), EXIT_FAILED_TURN),
    };
    let written = match write_candidate(&result, cand, &req.scope_files) {
        Ok(written) => written,
        Err(e @ RegenError::OutOfScope(_)) => {
            return failed(&mut writer, e.message(), EXIT_OUT_OF_SCOPE)
        }
        Err(e) => return failed(&mut writer, e.message(), EXIT_FAILED_TURN),
    };
    let summary = J::obj(vec![
        ("files", J::arr(written.clone())),
        (
            "structure",
            J::obj(vec![
                (
                    "pure",
                    J::arr(
                        result
                            .structure
                            .pure
                            .iter()
                            .map(|s| J::str(s.clone()))
                            .collect(),
                    ),
                ),
                (
                    "shell",
                    J::arr(
                        result
                            .structure
                            .shell
                            .iter()
                            .map(|s| J::str(s.clone()))
                            .collect(),
                    ),
                ),
            ]),
        ),
        ("change_log", J::str(result.change_log.clone())),
        ("written_utc", J::str(utc_now_iso())),
    ]);
    let digest = save(&turn_dir, "result.json", &indent1_dumps(&summary, false))?;
    writer.add_note(
        &format!("{label}-candidate"),
        J::obj(vec![
            ("root", J::str(cand.to_string_lossy().into_owned())),
            ("files", J::arr(written.clone())),
            ("result_sha256", J::str(digest)),
            (
                "structure_counts",
                J::obj(vec![
                    ("pure", J::int(result.structure.pure.len() as i64)),
                    ("shell", J::int(result.structure.shell.len() as i64)),
                ]),
            ),
        ]),
        &[J::obj(vec![
            (
                "artifact",
                J::str(rel_under(&turn_dir, receipt_path, "result.json")),
            ),
            ("sidecars", J::arr(vec![J::str("result.json.sha256")])),
        ])],
    )?;
    println!(
        "{label}: end_turn, {} files written to {}",
        written.len(),
        cand.display()
    );
    Ok(EXIT_OK)
}

pub fn cli_run(cmd: RegenerateCommand) -> Result<u8, String> {
    let result = match &cmd {
        RegenerateCommand::Prompt { request, out } => cmd_prompt(request, out),
        RegenerateCommand::Run { .. } => cmd_run(&cmd),
        RegenerateCommand::Refine => {
            eprintln!("refine: implemented by task 5.4");
            return Ok(EXIT_USAGE);
        }
    };
    match result {
        Ok(code) => Ok(code),
        Err(e) => {
            eprintln!("{}", e.message());
            Ok(e.exit_code())
        }
    }
}
