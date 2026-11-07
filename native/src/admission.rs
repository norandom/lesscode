//! Artifact admission for the deconvolution prototype (S2): bind artifact
//! bytes to sidecar digests and emit receipts. Fail-closed: missing bytes or
//! missing sidecars stay unavailable evidence, never zero, and a sidecar whose
//! artifact bytes are absent verifies nothing.
//!
//! Attempt-2 descope (prototype-plan.md §9): self-contained. Python owns
//! canonical digest emission; admission hashes bytes as stored (`sha2`),
//! parses its bindings via `serde_json` (input, order irrelevant) and emits
//! its fixed receipt schema in Python `json.dumps(indent=1)` form:
//! ASCII-escaped strings, integers, booleans, null, arrays and
//! insertion-ordered objects (no floats by construction).

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub use crate::receipt::{sha256_hex, write_receipt, J};

fn joined(root: &Path, rel: &str) -> PathBuf {
    root.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR))
}

/// Parsed `.sha256` sidecar: stored-bytes digest plus the declared LF-body
/// identity where the apparatus recorded one (write_text CRLF correction).
#[derive(Debug, Default, Clone)]
pub struct Sidecar {
    pub stored: Option<String>,
    pub declared_lf_body: Option<String>,
}

pub fn parse_sidecar(text: &str) -> Sidecar {
    let mut out = Sidecar::default();
    for line in text.lines() {
        let line = line.trim_end_matches('\r');
        if let Some(rest) = line.strip_prefix("# lf-body sha256 ") {
            let hex = &rest[..rest.find(' ').unwrap_or(rest.len())];
            if hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_hexdigit()) {
                out.declared_lf_body = Some(hex.to_ascii_lowercase());
            }
        } else if let Some(hex) = line.get(..64) {
            if hex.bytes().all(|b| b.is_ascii_hexdigit()) && line[64..].starts_with("  ") {
                out.stored = Some(hex.to_ascii_lowercase());
            }
        }
    }
    out
}

/// Binding status. `NoBoundSidecar` records present bytes with no sidecar;
/// such artifacts remain locally hashed but are not committed-sidecar evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Verified,
    Mismatch,
    Missing,
    NoBoundSidecar,
}

#[derive(Debug, Clone)]
pub struct SidecarCheck {
    pub sidecar: PathBuf,
    pub present: bool,
    pub expected_stored: Option<String>,
    pub declared_lf_body: Option<String>,
    pub matches_stored: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct BindingReport {
    pub artifact: String,
    pub present: bool,
    pub bytes: Option<u64>,
    pub sha256_stored: Option<String>,
    pub sidecars: Vec<SidecarCheck>,
    pub status: Status,
}

pub fn verify(root: &Path, artifact: &str, sidecars: &[String]) -> BindingReport {
    let path = joined(root, artifact);
    let mut report = BindingReport {
        artifact: artifact.to_string(),
        present: path.is_file(),
        bytes: None,
        sha256_stored: None,
        sidecars: Vec::new(),
        status: Status::Missing,
    };
    if !report.present {
        for sc in sidecars {
            report.sidecars.push(SidecarCheck {
                sidecar: PathBuf::from(sc),
                present: joined(root, sc).is_file(),
                expected_stored: None,
                declared_lf_body: None,
                matches_stored: None,
            });
        }
        return report;
    }
    let bytes = fs::read(&path).unwrap_or_default();
    report.bytes = Some(bytes.len() as u64);
    let actual = sha256_hex(&bytes);
    report.sha256_stored = Some(actual.clone());
    report.status = Status::NoBoundSidecar;
    for sc in sidecars {
        let sc_path = joined(root, sc);
        let mut check = SidecarCheck {
            sidecar: PathBuf::from(sc),
            present: sc_path.is_file(),
            expected_stored: None,
            declared_lf_body: None,
            matches_stored: None,
        };
        if check.present {
            let parsed = parse_sidecar(&fs::read_to_string(&sc_path).unwrap_or_default());
            check.expected_stored = parsed.stored.clone();
            check.declared_lf_body = parsed.declared_lf_body.clone();
            check.matches_stored = parsed.stored.as_ref().map(|expected| *expected == actual);
            match check.matches_stored {
                Some(true) if report.status != Status::Mismatch => report.status = Status::Verified,
                Some(false) => report.status = Status::Mismatch,
                _ => {}
            }
        }
        report.sidecars.push(check);
    }
    report
}

/// Builds the admission receipt as an order-preserving value (receipt writer:
/// `indent=1`, insertion order preserved).
pub fn receipt(command: &str, reports: &[BindingReport]) -> J {
    let bindings = reports
        .iter()
        .map(|r| {
            let sidecars = r
                .sidecars
                .iter()
                .map(|c| {
                    J::obj(vec![
                        ("sidecar", J::str(c.sidecar.to_string_lossy().to_string())),
                        ("present", J::Bool(c.present)),
                        (
                            "expected_stored",
                            c.expected_stored.clone().map(J::Str).unwrap_or(J::Null),
                        ),
                        (
                            "declared_lf_body",
                            c.declared_lf_body.clone().map(J::Str).unwrap_or(J::Null),
                        ),
                        (
                            "matches_stored",
                            c.matches_stored.map(J::Bool).unwrap_or(J::Null),
                        ),
                    ])
                })
                .collect::<Vec<_>>();
            J::obj(vec![
                ("artifact", J::str(r.artifact.clone())),
                ("present", J::Bool(r.present)),
                (
                    "bytes",
                    r.bytes.map(|b| J::int(b as i64)).unwrap_or(J::Null),
                ),
                (
                    "sha256_stored",
                    r.sha256_stored.clone().map(J::Str).unwrap_or(J::Null),
                ),
                ("sidecars", J::Arr(sidecars)),
                ("status", J::str(status_name(r.status))),
            ])
        })
        .collect::<Vec<_>>();
    let count = |s: Status| reports.iter().filter(|r| r.status == s).count();
    J::obj(vec![
        ("receipt", J::str("s2-artifact-admission-rust-1")),
        (
            "purpose",
            J::str("bind artifact bytes to sidecar digests (prototype-plan.md U1)"),
        ),
        (
            "generated_unix",
            J::int(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0),
            ),
        ),
        ("command", J::str(command)),
        (
            "hashing",
            J::str("SHA256 over bytes as stored; declared LF-body hashes recorded separately"),
        ),
        ("bindings", J::Arr(bindings)),
        (
            "summary",
            J::obj(vec![
                ("total", J::int(reports.len() as i64)),
                ("verified", J::int(count(Status::Verified) as i64)),
                ("mismatch", J::int(count(Status::Mismatch) as i64)),
                ("missing", J::int(count(Status::Missing) as i64)),
                (
                    "no_bound_sidecar",
                    J::int(count(Status::NoBoundSidecar) as i64),
                ),
            ]),
        ),
    ])
}

pub fn status_name(status: Status) -> &'static str {
    match status {
        Status::Verified => "verified",
        Status::Mismatch => "mismatch",
        Status::Missing => "missing",
        Status::NoBoundSidecar => "no_bound_sidecar",
    }
}

/// CLI entry: `bindings` points to a JSON file
/// `{"bindings": [{"artifact": "...", "sidecars": ["...", ...]}, ...]}` resolved
/// against `root`; writes the receipt to `out`. Exit 0 only when no bound
/// artifact mismatches or is missing.
pub fn cli_run(root: &Path, bindings: &Path, out: &Path) -> Result<u8, String> {
    let text = fs::read_to_string(bindings)
        .map_err(|e| format!("cannot read {}: {e}", bindings.display()))?;
    let spec: Value = serde_json::from_str(&text)
        .map_err(|e| format!("invalid bindings {}: {e}", bindings.display()))?;
    let items = spec
        .get("bindings")
        .and_then(Value::as_array)
        .ok_or("bindings file needs a \"bindings\" array")?;
    let mut reports = Vec::new();
    for item in items {
        let artifact = item
            .get("artifact")
            .and_then(Value::as_str)
            .ok_or("each binding needs an \"artifact\" string")?
            .to_string();
        let sidecars = item
            .get("sidecars")
            .and_then(Value::as_array)
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        reports.push(verify(root, &artifact, &sidecars));
    }
    for r in &reports {
        println!("{:<72} {}", r.artifact, status_name(r.status));
    }
    let value = receipt(
        &format!(
            "lesscode-core admission --root {} --bindings {} --out {}",
            root.display(),
            bindings.display(),
            out.display()
        ),
        &reports,
    );
    write_receipt(out, &value).map_err(|e| format!("cannot write {}: {e}", out.display()))?;
    let failed = reports
        .iter()
        .any(|r| matches!(r.status, Status::Mismatch | Status::Missing));
    Ok(if failed { 1 } else { 0 })
}
