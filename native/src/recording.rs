//! Shared recording/replay/acceptance engine — the Rust-substrate port of
//! `src/lesscode_baseline/recording.py` (task 8.4, R36).
//!
//! Division of labor (preregistered): this module owns the engine semantics
//! that can leave Python — golden body bytes (`json.dumps` sort_keys/
//! indent=0 parity) and LF-body digests with the `.sha256` sidecar, replay
//! diffing (`first_diff` over canonical JSON), the L2 round report,
//! signature sets, deadline-killed subprocess arms (`taskkill` tree kill),
//! serial pytest groups, arm environments and the acceptance verdict.
//! Declared normalization executes inside the frozen oracle envs (their own
//! `Norm`), reached through the embedded counted shim
//! (`native/shim/recording_shim.py`) running in the pinned specimen venv;
//! JSON files under a scratch dir are the interface.

use crate::receipt::{sha256_hex, J};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Frozen LF-body digests of the double-record gate (R30; task 8.4 done-when).
pub const FROZEN_E3_LF_BODY: &str =
    "92a48f77f66f2c9d0d0ebaf6e9e4b37f747816d2831dd577b72e183b7bf07671";
pub const FROZEN_E4_LF_BODY: &str =
    "0b2e383dd6fbbc243215e24a7844674404331308b3a547239594461c5bea5584";

/// Scope adapter selection: meta variant and harness-suspect error classes.
const SUSPECT_ERRORS: [(&str, [&str; 5]); 2] = [
    (
        "e3",
        [
            "AttributeError",
            "KeyError",
            "NameError",
            "UnboundLocalError",
            "",
        ],
    ),
    (
        "e4",
        [
            "AttributeError",
            "KeyError",
            "NameError",
            "UnboundLocalError",
            "TypeError",
        ],
    ),
];

#[derive(Debug)]
pub struct RecordingError(pub String);
impl std::fmt::Display for RecordingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for RecordingError {}

fn err<T>(msg: String) -> Result<T, RecordingError> {
    Err(RecordingError(msg))
}

/// Everything the engine needs to drive one frozen scope.
pub struct ScopePlan {
    /// `e3` or `e4` — selects the frozen adapter and the meta variant.
    pub scope: String,
    /// The embedded shim (`native/shim/recording_shim.py`).
    pub shim: PathBuf,
    /// Pinned specimen venv python (runs the shim).
    pub venv: PathBuf,
    /// Scratch dir for the shim's JSON interface files.
    pub work_dir: PathBuf,
}

// --- JSON writers (Python json.dumps parity) ------------------------------

fn write_str_py(s: &str, out: &mut String) {
    // The receipt writer's string escaping is exactly json.dumps.
    crate::receipt::write_json_str(s, out);
}

/// Python `repr(x)` for a finite double: shortest round-trip digits with
/// the CPython fixed/scientific switch (scientific when the decimal point
/// sits before -3 or after 16) and two-digit exponents ("1e-05", "1e+16").
fn py_float_repr(x: f64) -> String {
    if !x.is_finite() {
        return "NaN".into(); // unreachable on the frozen material
    }
    let sci = format!("{x:e}");
    let (mantissa, exp) = sci.split_once('e').expect("LowerExp always has e");
    let exp: i32 = exp.parse().expect("valid exponent");
    let digits: String = mantissa
        .chars()
        .filter(|c| !matches!(c, '.' | '-'))
        .collect();
    let decpt = exp + 1;
    let body = if decpt <= -4 || decpt >= 17 {
        let mut s = String::from(&digits[..1]);
        if digits.len() > 1 {
            s.push('.');
            s.push_str(&digits[1..]);
        }
        let sign = if exp < 0 { '-' } else { '+' };
        if exp.abs() < 10 {
            let _ = write!(s, "e{sign}0{}", exp.abs());
        } else {
            let _ = write!(s, "e{sign}{}", exp.abs());
        }
        s
    } else if decpt <= 0 {
        format!("0.{}{}", "0".repeat((-decpt) as usize), digits)
    } else if decpt as usize >= digits.len() {
        format!("{}{}.0", digits, "0".repeat(decpt as usize - digits.len()))
    } else {
        format!(
            "{}.{}",
            &digits[..decpt as usize],
            &digits[decpt as usize..]
        )
    };
    if x.is_sign_negative() {
        format!("-{body}")
    } else {
        body
    }
}

fn write_scalar(value: &J, out: &mut String) {
    match value {
        J::Null => out.push_str("null"),
        J::Bool(true) => out.push_str("true"),
        J::Bool(false) => out.push_str("false"),
        J::Int(n) => out.push_str(&n.to_string()),
        // Python repr parity; non-finite never occurs in the goldens
        // (allow_nan output verified absent on the frozen set).
        J::Float(x) if x.is_finite() => out.push_str(&py_float_repr(*x)),
        J::Float(_) => out.push_str("null"),
        J::Str(s) => write_str_py(s, out),
        _ => unreachable!("scalar"),
    }
}

/// `json.dumps(value, sort_keys=True, separators=(",", ":"))` — canonical
/// compact form used for equality and diff messages.
pub fn canon_dumps(value: &J, out: &mut String) {
    match value {
        J::Arr(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                canon_dumps(item, out);
            }
            out.push(']');
        }
        J::Obj(pairs) => {
            let mut keys: Vec<&(String, J)> = pairs.iter().collect();
            keys.sort_by(|a, b| a.0.cmp(&b.0));
            out.push('{');
            for (i, (key, val)) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_str_py(key, out);
                out.push(':');
                canon_dumps(val, out);
            }
            out.push('}');
        }
        scalar => write_scalar(scalar, out),
    }
}

/// `json.dumps(value, sort_keys=True, indent=0, allow_nan=True)` — the frozen
/// golden body writer: one item per line, no indent spaces, `": "` keys.
fn golden_write(value: &J, out: &mut String) {
    match value {
        J::Arr(items) if !items.is_empty() => {
            out.push_str("[\n");
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push_str(",\n");
                }
                golden_write(item, out);
            }
            out.push_str("\n]");
        }
        J::Obj(pairs) if !pairs.is_empty() => {
            let mut keys: Vec<&(String, J)> = pairs.iter().collect();
            keys.sort_by(|a, b| a.0.cmp(&b.0));
            out.push_str("{\n");
            for (i, (key, val)) in keys.iter().enumerate() {
                if i > 0 {
                    out.push_str(",\n");
                }
                write_str_py(key, out);
                out.push_str(": ");
                golden_write(val, out);
            }
            out.push_str("\n}");
        }
        J::Arr(_) => out.push_str("[]"),
        J::Obj(_) => out.push_str("{}"),
        scalar => write_scalar(scalar, out),
    }
}

/// Thousands-separated count, like Python `f"{n:,}"`.
pub fn comma_count(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// The golden body for a value: `json.dumps(value, sort_keys=True,
/// indent=0, allow_nan=True)` bytes.
pub fn golden_dumps(value: &J) -> String {
    let mut out = String::new();
    golden_write(value, &mut out);
    out
}

/// Frozen recorder output: golden body (LF as stored) plus the `.sha256`
/// sidecar; returns the LF-body digest and body length.
pub fn write_golden(out: &Path, meta: &J, seqs: &J) -> Result<(String, usize), RecordingError> {
    let mut body = String::new();
    golden_write(
        &J::obj(vec![("meta", meta.clone()), ("sequences", seqs.clone())]),
        &mut body,
    );
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| RecordingError(format!("cannot create {}: {e}", parent.display())))?;
    }
    fs::write(out, body.as_bytes())
        .map_err(|e| RecordingError(format!("cannot write {}: {e}", out.display())))?;
    let digest = sha256_hex(body.as_bytes());
    let name = out
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let sidecar = format!(
        "{digest}  {name}\n# lf-body sha256 {digest} {} bytes (proto_core writer, LF as stored)\n",
        comma_count(body.len())
    );
    fs::write(
        PathBuf::from(format!("{}.sha256", out.display())),
        sidecar.as_bytes(),
    )
    .map_err(|e| RecordingError(format!("cannot write sidecar: {e}")))?;
    Ok((digest, body.len()))
}

// --- shim interface --------------------------------------------------------

fn value_to_j(value: &serde_json::Value) -> J {
    match value {
        serde_json::Value::Null => J::Null,
        serde_json::Value::Bool(b) => J::Bool(*b),
        serde_json::Value::Number(n) => match n.as_i64() {
            Some(i) => J::Int(i),
            None => J::Float(n.as_f64().unwrap_or(0.0)),
        },
        serde_json::Value::String(s) => J::Str(s.clone()),
        serde_json::Value::Array(items) => J::Arr(items.iter().map(value_to_j).collect()),
        serde_json::Value::Object(map) => J::Obj(
            map.iter()
                .map(|(k, v)| (k.clone(), value_to_j(v)))
                .collect(),
        ),
    }
}

fn read_json(path: &Path) -> Result<J, RecordingError> {
    let text = fs::read_to_string(path)
        .map_err(|e| RecordingError(format!("cannot read {}: {e}", path.display())))?;
    serde_json::from_str::<serde_json::Value>(&text)
        .map(|v| value_to_j(&v))
        .map_err(|e| RecordingError(format!("cannot parse {}: {e}", path.display())))
}

fn run_shim(plan: &ScopePlan, args: &[&str]) -> Result<(), RecordingError> {
    let output = Command::new(&plan.venv)
        .arg(&plan.shim)
        .args(args)
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .output()
        .map_err(|e| RecordingError(format!("cannot run shim: {e}")))?;
    if !output.status.success() {
        return err(format!(
            "shim {} failed ({}): {}{}",
            args.first().unwrap_or(&""),
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

fn j_get<'a>(v: &'a J, key: &str) -> Option<&'a J> {
    match v {
        J::Obj(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
        _ => None,
    }
}

fn j_str(v: &J) -> &str {
    match v {
        J::Str(s) => s,
        _ => "",
    }
}

fn j_arr(v: &J) -> &[J] {
    match v {
        J::Arr(items) => items,
        _ => &[],
    }
}

fn j_obj(v: &J) -> &[(String, J)] {
    match v {
        J::Obj(pairs) => pairs,
        _ => &[],
    }
}

fn canon(value: &J) -> String {
    let mut out = String::new();
    canon_dumps(value, &mut out);
    out
}

/// Python `str(v)` of an error-detail value: `None` for null, verbatim
/// strings (no quotes), numbers as themselves; containers fall back to the
/// canonical form (details on the frozen material are scalars).
fn py_str_of(value: &J) -> String {
    match value {
        J::Null => "None".into(),
        J::Bool(true) => "True".into(),
        J::Bool(false) => "False".into(),
        J::Int(n) => n.to_string(),
        J::Float(x) => py_float_repr(*x),
        J::Str(s) => s.clone(),
        other => canon(other),
    }
}

/// `op:{ok|error[0]<error[2]}` for every (op, observation) pair of a sequence.
pub fn signatures(seqs: &[J]) -> Vec<String> {
    let mut out = std::collections::BTreeSet::new();
    for seq in seqs {
        let ops = j_arr(j_get(seq, "ops").unwrap_or(&J::Null));
        let observations = j_arr(j_get(seq, "observations").unwrap_or(&J::Null));
        for (o, ob) in ops.iter().zip(observations) {
            let sig = match j_get(ob, "result") {
                Some(_) => "ok".to_string(),
                None => {
                    let error = j_arr(j_get(ob, "error").unwrap_or(&J::Null));
                    let kind = error
                        .first()
                        .map(|e| j_str(e).to_string())
                        .unwrap_or_default();
                    let detail = error.get(2).map(py_str_of).unwrap_or_else(|| "None".into());
                    // The frozen format opens '<' after the error class and
                    // never closes it (faithful to the reference f-string).
                    format!("{kind}<{detail}")
                }
            };
            out.insert(format!(
                "{}:{sig}",
                j_str(j_get(o, "op").unwrap_or(&J::Null))
            ));
        }
    }
    out.into_iter().collect()
}

/// First structural divergence between recorded and replayed values, with
/// the frozen path messages (missing keys, list indices, leaf canon text).
pub fn first_diff(a: &J, b: &J, path: &str) -> Option<String> {
    if a == b {
        return None;
    }
    match (a, b) {
        (J::Obj(_), J::Obj(_)) => {
            let mut keys: std::collections::BTreeSet<&String> =
                j_obj(a).iter().map(|(k, _)| k).collect();
            keys.extend(j_obj(b).iter().map(|(k, _)| k));
            for k in keys {
                let (va, vb) = (j_get(a, k), j_get(b, k));
                let d = match (va, vb) {
                    (Some(x), Some(y)) => first_diff(x, y, &format!("{path}/{k}")),
                    _ => Some(format!("{path}/{k} (present on one side only)")),
                };
                if d.is_some() {
                    return d;
                }
            }
            None
        }
        (J::Arr(items_a), J::Arr(items_b)) => {
            for (i, (x, y)) in items_a.iter().zip(items_b.iter()).enumerate() {
                if let Some(d) = first_diff(x, y, &format!("{path}[{i}]")) {
                    return Some(d);
                }
            }
            Some(format!(
                "{path} (length {} recorded vs {} replayed)",
                items_a.len(),
                items_b.len()
            ))
        }
        _ => Some(format!(
            "{path}: recorded {} vs replayed {}",
            truncate_chars(&canon(a), 300),
            truncate_chars(&canon(b), 300)
        )),
    }
}

fn truncate_chars(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

// --- record ----------------------------------------------------------------

/// Assemble the frozen golden meta from the shim's live inputs and the
/// normalized sequences (signatures are engine semantics, computed here).
fn assemble_meta(scope: &str, inputs: &J, seqs: &[J]) -> J {
    let seed: Vec<String> = seqs
        .iter()
        .filter(|s| j_str(j_get(s, "origin").unwrap_or(&J::Null)) == "seed")
        .flat_map(|s| signatures(std::slice::from_ref(s)))
        .collect();
    let seed_sigs: std::collections::BTreeSet<String> = seed.into_iter().collect();
    let gen: Vec<String> = seqs
        .iter()
        .filter(|s| j_str(j_get(s, "origin").unwrap_or(&J::Null)) == "generated")
        .flat_map(|s| signatures(std::slice::from_ref(s)))
        .collect();
    let gen_sigs: std::collections::BTreeSet<String> = gen.into_iter().collect();
    let suspects = SUSPECT_ERRORS
        .iter()
        .find(|(s, _)| *s == scope)
        .map(|(_, errs)| errs)
        .unwrap_or(&SUSPECT_ERRORS[0].1);
    let all_sigs: std::collections::BTreeSet<String> =
        seed_sigs.union(&gen_sigs).cloned().collect();
    let mut fields: Vec<(&str, J)> = vec![
        (
            "hypothesis_seed",
            j_get(inputs, "hypothesis_seed").cloned().unwrap_or(J::Null),
        ),
        (
            "generated_requested",
            j_get(inputs, "generated_requested")
                .cloned()
                .unwrap_or(J::Null),
        ),
        (
            "python",
            j_get(inputs, "python").cloned().unwrap_or(J::Null),
        ),
        (
            "hypothesis",
            j_get(inputs, "hypothesis").cloned().unwrap_or(J::Null),
        ),
        (
            "sqlalchemy",
            j_get(inputs, "sqlalchemy").cloned().unwrap_or(J::Null),
        ),
    ];
    if scope == "e3" {
        fields.push(("scope", j_get(inputs, "scope").cloned().unwrap_or(J::Null)));
    } else {
        fields.push(("root", j_get(inputs, "root").cloned().unwrap_or(J::Null)));
        fields.push((
            "scope_sha256",
            j_get(inputs, "scope_sha256").cloned().unwrap_or(J::Null),
        ));
        fields.push((
            "alphabet",
            j_get(inputs, "alphabet").cloned().unwrap_or(J::Null),
        ));
    }
    fields.push(("sequences", J::int(seqs.len() as i64)));
    fields.push((
        "operations",
        J::int(
            seqs.iter()
                .map(|s| j_arr(j_get(s, "ops").unwrap_or(&J::Null)).len() as i64)
                .sum(),
        ),
    ));
    fields.push((
        "seed_signatures",
        J::Arr(seed_sigs.iter().map(|s| J::str(s.clone())).collect()),
    ));
    fields.push((
        "generated_only_signatures",
        J::Arr(
            gen_sigs
                .difference(&seed_sigs)
                .map(|s| J::str(s.clone()))
                .collect(),
        ),
    ));
    fields.push((
        "harness_suspects",
        J::Arr(
            all_sigs
                .iter()
                .filter(|s| suspects.iter().any(|e| !e.is_empty() && s.contains(e)))
                .map(|s| J::str(s.clone()))
                .collect(),
        ),
    ));
    J::obj(fields)
}

/// Record outcome: the three summary fields the adapters returned.
pub struct RecordOutcome {
    pub sequences: i64,
    pub operations: i64,
    pub sha256_lf_body: String,
}

/// Double-record gate entry: run the shim, assemble meta, write the golden
/// body bytes and LF-body digest sidecar.
pub fn record_golden(plan: &ScopePlan, out: &Path) -> Result<RecordOutcome, RecordingError> {
    run_shim(
        plan,
        &[
            "record",
            "--scope",
            &plan.scope,
            "--out-dir",
            &plan.work_dir.to_string_lossy(),
        ],
    )?;
    let seqs_doc = read_json(&plan.work_dir.join("seqs.json"))?;
    let inputs = read_json(&plan.work_dir.join("meta-input.json"))?;
    let seqs: Vec<J> = j_arr(j_get(&seqs_doc, "sequences").unwrap_or(&J::Null)).to_vec();
    if seqs.is_empty() {
        return err("shim produced no sequences".into());
    }
    let meta = assemble_meta(&plan.scope, &inputs, &seqs);
    let (digest, _) = write_golden(out, &meta, &J::Arr(seqs))?;
    let j_int = |key: &str| -> i64 {
        match j_get(&meta, key) {
            Some(J::Int(n)) => *n,
            _ => 0,
        }
    };
    Ok(RecordOutcome {
        sequences: j_int("sequences"),
        operations: j_int("operations"),
        sha256_lf_body: digest,
    })
}

// --- replay -----------------------------------------------------------------

fn shim_replay(plan: &ScopePlan, goldens: &[PathBuf]) -> Result<Vec<J>, RecordingError> {
    let golden_args: Vec<String> = goldens
        .iter()
        .map(|g| g.to_string_lossy().into_owned())
        .collect();
    let mut args: Vec<String> = vec![
        "replay".into(),
        "--scope".into(),
        plan.scope.clone(),
        "--out-dir".into(),
        plan.work_dir.to_string_lossy().into_owned(),
        "--goldens".into(),
    ];
    args.extend(golden_args);
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    run_shim(plan, &arg_refs)?;
    let got = read_json(&plan.work_dir.join("got.json"))?;
    Ok(j_arr(j_get(&got, "sequences").unwrap_or(&J::Null)).to_vec())
}

/// Re-run every recorded operation sequence; the first L1 divergence or
/// `None`.
pub fn replay_golden(plan: &ScopePlan, golden: &Path) -> Result<Option<String>, RecordingError> {
    let trace = read_json(golden)?;
    let got = shim_replay(plan, &[golden.to_path_buf()])?;
    for seq in j_arr(j_get(&trace, "sequences").unwrap_or(&J::Null)) {
        let name = j_str(j_get(seq, "name").unwrap_or(&J::Null));
        let replayed = got
            .iter()
            .find(|g| j_str(j_get(g, "name").unwrap_or(&J::Null)) == name);
        let Some(replayed) = replayed else {
            return err(format!("shim did not replay sequence {name}"));
        };
        if let Some(d) = first_diff(
            j_get(seq, "start_tables").unwrap_or(&J::Null),
            j_get(replayed, "start_tables").unwrap_or(&J::Null),
            "",
        ) {
            return Ok(Some(format!("{name} start state: {d}")));
        }
        let ops = j_arr(j_get(seq, "ops").unwrap_or(&J::Null));
        let recorded = j_arr(j_get(seq, "observations").unwrap_or(&J::Null));
        let replayed_obs = j_arr(j_get(replayed, "observations").unwrap_or(&J::Null));
        for (i, ((o, a), b)) in ops.iter().zip(recorded).zip(replayed_obs).enumerate() {
            if let Some(d) = first_diff(a, b, "") {
                return Ok(Some(format!(
                    "{name} op {i} {}: {d}",
                    truncate_chars(&canon(o), 160)
                )));
            }
        }
    }
    Ok(None)
}

/// The replay target the frozen arms ran inside pytest: L1 asserts on first
/// divergence, L2 collected per operation without failing; writes the round
/// report and returns its summary fields.
pub fn replay_to_l2(
    plan: &ScopePlan,
    root: &Path,
    goldens: &[PathBuf],
    out: &Path,
) -> Result<J, RecordingError> {
    let got = shim_replay(plan, goldens)?;
    let mut l2: Vec<J> = Vec::new();
    let (mut sequences, mut operations) = (0i64, 0i64);
    for golden in goldens {
        let trace = read_json(golden)?;
        for seq in j_arr(j_get(&trace, "sequences").unwrap_or(&J::Null)) {
            let name = j_str(j_get(seq, "name").unwrap_or(&J::Null));
            let replayed = got.iter().find(|g| {
                j_str(j_get(g, "name").unwrap_or(&J::Null)) == name
                    && j_str(j_get(g, "golden").unwrap_or(&J::Null))
                        == golden
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_default()
                            .as_str()
            });
            let Some(replayed) = replayed else {
                return err(format!("shim did not replay sequence {name}"));
            };
            if let Some(d) = first_diff(
                j_get(seq, "start_tables").unwrap_or(&J::Null),
                j_get(replayed, "start_tables").unwrap_or(&J::Null),
                "",
            ) {
                return err(format!("{} {name} start state: {d}", golden.display()));
            }
            let ops = j_arr(j_get(seq, "ops").unwrap_or(&J::Null));
            let recorded = j_arr(j_get(seq, "observations").unwrap_or(&J::Null));
            let replayed_obs = j_arr(j_get(replayed, "observations").unwrap_or(&J::Null));
            for (i, ((o, a), b)) in ops.iter().zip(recorded).zip(replayed_obs).enumerate() {
                let strip_sql = |v: &J| -> J {
                    let mut v = v.clone();
                    if let J::Obj(pairs) = &mut v {
                        pairs.retain(|(k, _)| k != "sql");
                    }
                    v
                };
                let (sa, sb) = (
                    j_get(a, "sql").cloned().unwrap_or(J::Null),
                    j_get(b, "sql").cloned().unwrap_or(J::Null),
                );
                let (a_rest, b_rest) = (strip_sql(a), strip_sql(b));
                if let Some(d) = first_diff(&a_rest, &b_rest, "") {
                    return err(format!(
                        "{} {name} op {i} {}: {d}",
                        golden.display(),
                        truncate_chars(&canon(o), 160)
                    ));
                }
                if canon(&sa) != canon(&sb) {
                    l2.push(J::obj(vec![
                        (
                            "golden",
                            J::str(
                                golden
                                    .file_name()
                                    .map(|n| n.to_string_lossy().into_owned())
                                    .unwrap_or_default(),
                            ),
                        ),
                        ("sequence", J::str(name)),
                        ("op_index", J::int(i as i64)),
                        ("op", o.clone()),
                        ("recorded", sa),
                        ("replayed", sb),
                    ]));
                }
                operations += 1;
            }
            sequences += 1;
        }
    }
    let written = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    let report = J::obj(vec![
        ("written", J::Float(written)),
        ("root", J::str(root.to_string_lossy().into_owned())),
        (
            "golden",
            J::str(
                goldens
                    .first()
                    .unwrap_or(&PathBuf::new())
                    .to_string_lossy()
                    .into_owned(),
            ),
        ),
        ("sequences", J::int(sequences)),
        ("operations", J::int(operations)),
        ("level2_ops", J::int(l2.len() as i64)),
        ("divergences", J::Arr(l2.clone())),
    ]);
    fs::write(out, crate::receipt::dumps(&report).as_bytes())
        .map_err(|e| RecordingError(format!("cannot write {}: {e}", out.display())))?;
    Ok(J::obj(vec![
        ("sequences", J::int(sequences)),
        ("operations", J::int(operations)),
        ("level2_ops", J::int(l2.len() as i64)),
    ]))
}

// --- subprocess arms ---------------------------------------------------------

/// Run cmd; past the deadline kill its whole tree (Windows has no process
/// group kill; Python used blocking wait+timeout, Rust polls `try_wait` at
/// the same deadline semantics). Returns the arm exit code, 124 on deadline.
pub fn call(
    program: &str,
    args: &[&str],
    cwd: &Path,
    envs: &[(String, String)],
    deadline: Option<Instant>,
) -> i32 {
    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(cwd)
        .envs(envs.iter().cloned());
    let Ok(mut child) = command.spawn() else {
        return 127;
    };
    let poll = Duration::from_millis(50);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.code().unwrap_or(-1),
            Ok(None) => {}
            Err(_) => return -1,
        }
        if let Some(deadline) = deadline {
            if Instant::now() >= deadline {
                let _ = Command::new("taskkill")
                    .args(["/F", "/T", "/PID"])
                    .arg(child.id().to_string())
                    .output();
                println!("proto: deadline exceeded (infrastructure, not a verdict)");
                return 124;
            }
            std::thread::sleep(poll.min(deadline.saturating_duration_since(Instant::now())));
        } else {
            std::thread::sleep(poll);
        }
    }
}

/// Fail closed when the scope module is not imported from `root`.
pub fn probe(
    venv: &Path,
    root: &Path,
    module: &str,
    envs: &[(String, String)],
) -> Result<(), RecordingError> {
    let code = format!("import {module} as m; print(m.__file__)");
    let output = Command::new(venv)
        .arg("-c")
        .arg(&code)
        .envs(envs.iter().cloned())
        .output()
        .map_err(|e| RecordingError(format!("probe failed: {e}")))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let imported_from = PathBuf::from(if stdout.is_empty() {
        "/".to_string()
    } else {
        stdout.clone()
    });
    if !same_file_prefix(&imported_from, root) {
        return err(format!(
            "INVALID scope not imported from {}: {stdout}{stderr}",
            root.display()
        ));
    }
    Ok(())
}

fn same_file_prefix(candidate: &Path, root: &Path) -> bool {
    let (c, r) = (candidate.canonicalize().ok(), root.canonicalize().ok());
    match (c, r) {
        (Some(c), Some(r)) => c.starts_with(r),
        _ => candidate.starts_with(root),
    }
}

/// Serial per-group pytest with the frozen flags; exit 0 only if every group
/// passes (5 = every test deselected also passes).
pub fn run_groups(
    venv: &Path,
    root: &Path,
    groups: &[(String, Vec<String>)],
    marks: &str,
    envs: &[(String, String)],
    deadline: Option<Instant>,
    extra_first: &[String],
) -> i32 {
    for (cwd, args) in groups {
        if args.is_empty() {
            continue;
        }
        let mut cmd: Vec<String> = [
            "-m",
            "pytest",
            "-q",
            "-c",
            "pyproject.toml",
            "--rootdir",
            ".",
            "-p",
            "no:cacheprovider",
            "-p",
            "e2_winshim",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        cmd.extend(extra_first.iter().cloned());
        cmd.extend(
            ["-x", "--timeout=600", "-m", marks]
                .iter()
                .map(|s| s.to_string()),
        );
        cmd.extend(args.iter().cloned());
        let arg_refs: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
        let t0 = Instant::now();
        let code = call(
            venv.to_string_lossy().as_ref(),
            &arg_refs,
            &root.join(cwd),
            envs,
            deadline,
        );
        println!(
            "proto {cwd}: exit {code} in {:.1}s",
            t0.elapsed().as_secs_f64()
        );
        if !matches!(code, 0 | 5) {
            return 1;
        }
    }
    0
}

/// Frozen acceptance refusal rules: the arm must pass; a stale or short
/// replay is incomplete; L2 differences without bound query-differential
/// evidence refuse acceptance rather than silently falling back to L1.
pub fn verdict(
    arm_exit: i32,
    fresh: bool,
    level2_ops: i64,
    query_evidence: Option<&Path>,
) -> (i32, Vec<String>) {
    let mut reasons = if arm_exit == 0 {
        Vec::new()
    } else {
        vec![format!("arm exit {arm_exit}")]
    };
    if !fresh {
        reasons.push("replay not fresh or short of golden coverage".into());
    }
    if level2_ops > 0 && !matches!(query_evidence, Some(p) if p.is_file()) {
        reasons.push(format!(
            "{level2_ops} L2 differences without query-differential evidence"
        ));
    }
    if reasons.is_empty() {
        return (0, reasons);
    }
    (if arm_exit == 2 { 2 } else { 1 }, reasons)
}

/// The arm environment: thread caps, warning silencers, module path.
pub fn arm_env(
    root: &Path,
    paths: &[PathBuf],
    extra: &[(String, String)],
) -> Vec<(String, String)> {
    let mut pythonpath = root.join("src").to_string_lossy().into_owned();
    for p in paths {
        pythonpath.push(';');
        pythonpath.push_str(&p.to_string_lossy());
    }
    let mut env: Vec<(String, String)> = [
        ("OPENBLAS_NUM_THREADS", "1"),
        ("OMP_NUM_THREADS", "1"),
        ("MKL_NUM_THREADS", "1"),
        ("PYTHONWARNINGS", "ignore"),
        ("PYTHONDONTWRITEBYTECODE", "1"),
        ("PYTHONPATH", pythonpath.as_str()),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();
    for (k, v) in extra {
        if let Some(entry) = env.iter_mut().find(|(ek, _)| ek == k) {
            entry.1 = v.clone();
        } else {
            env.push((k.clone(), v.clone()));
        }
    }
    env
}

/// (cwd relative to root, pytest args): root-suite seed files, then satellite
/// packages; baseline deselections applied; the replay target rides the root
/// group via `-p` fixtures.
pub fn build_groups(
    files: &[String],
    bad: &[String],
    packages: &[&str],
    replay_target: Option<&Path>,
) -> Vec<(String, Vec<String>)> {
    let pick = |prefix: &str, xs: &[String]| -> Vec<String> {
        xs.iter()
            .filter_map(|x| x.strip_prefix(prefix))
            .map(|x| x.to_string())
            .collect()
    };
    let mut out: Vec<(String, Vec<String>)> = vec![(
        ".".into(),
        pick(
            "",
            &files
                .iter()
                .filter(|f| f.starts_with("tests/"))
                .cloned()
                .collect::<Vec<_>>(),
        ),
    )];
    for p in packages {
        let prefix = format!("packages/facdrone-{p}/");
        out.push((format!("packages/facdrone-{p}"), pick(&prefix, files)));
    }
    out = out
        .into_iter()
        .map(|(cwd, args)| {
            if args.is_empty() {
                (cwd, Vec::new())
            } else {
                let prefix = if cwd == "." { "" } else { &format!("{cwd}/") };
                let mut args = args;
                for n in pick(prefix, bad) {
                    if cwd != "." || n.starts_with("tests/") {
                        args.push(format!("--deselect={n}"));
                    }
                }
                (cwd, args)
            }
        })
        .collect();
    if let Some(target) = replay_target {
        if let Some(first) = out.first_mut() {
            first.1.extend([
                "-p".to_string(),
                "facdrone_testing.fixtures".to_string(),
                target.to_string_lossy().into_owned(),
            ]);
        }
    }
    out.into_iter()
        .filter(|(_, args)| !args.is_empty())
        .collect()
}
