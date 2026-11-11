//! `step record`: put a step on the ledger. An accepted verdict that still
//! matches the committed content, an override with a written reason, or a
//! release. Earlier entries are never rewritten.

use super::context::Context;
use super::guard::{standing, Standing};
use super::ledger::{append, read, since_release, today, ACCEPTED, OVERRIDDEN, RELEASE};
use crate::specimen_runner::sha256_hex_bytes;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// Arguments of `step record`.
#[derive(clap::Args, Clone, Debug)]
pub struct RecordArgs {
    /// The step declaration, relative to the repository root.
    #[arg(long, required_unless_present = "release")]
    pub declaration: Option<String>,
    /// The ledger, relative to the repository root; needed for a release.
    #[arg(long, requires = "release")]
    pub ledger: Option<String>,
    /// The repository.
    #[arg(long, default_value = ".")]
    pub repo: PathBuf,
    /// Record a release of this name and list the steps since the one before.
    #[arg(long, conflicts_with = "overridden", requires = "ledger")]
    pub release: Option<String>,
    /// Record that the step goes ahead without an accepted verdict.
    #[arg(long = "override", requires = "reason")]
    pub overridden: bool,
    /// Why; required with --override.
    #[arg(long)]
    pub reason: Option<String>,
}

fn object(value: Value) -> Entry {
    value.as_object().cloned().unwrap_or_default()
}

/// The numbers of a verdict the ledger keeps per function.
fn rows(verdict: &Value) -> Vec<Value> {
    let functions = verdict["functions"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let row = |f: &Value| {
        json!({"file": f["file"], "function": f["function"],
               "before": f["before"], "after": f["after"]})
    };
    functions.iter().map(row).collect()
}

type Entry = serde_json::Map<String, Value>;

/// A release entry; the steps since the release before are listed.
fn release(ledger: &Path, entries: &[Value], name: &str) -> Result<u8, String> {
    let steps: Vec<&Value> = since_release(entries);
    let names: Vec<&Value> = steps.iter().map(|e| &e["step"]).collect();
    println!(
        "release {name}: {} steps since the release before",
        steps.len()
    );
    for step in &steps {
        let name = step["step"].as_str().unwrap_or("?");
        println!("  {name} ({})", step["result"]);
    }
    let entry = json!({"result": RELEASE, "release": name, "date": today(), "steps": names});
    append(ledger, object(entry)).map(|()| 0)
}

/// An entry for the verdict on disk, if it is accepted and still matches.
fn accepted(context: &Context, entries: &[Value], mut entry: Entry) -> Result<u8, String> {
    let declaration = &context.declaration;
    let revision = &context.revisions.step;
    let (found, _) = standing(&context.repo, &declaration.branch, revision)?;
    if found != Standing::Accepted {
        let why = match found {
            Standing::Refused(why) => why,
            other => format!("{other:?}"),
        };
        println!("step {}: not recorded: {why}", declaration.step);
        return Ok(1);
    }
    let path = context.repo.join(&declaration.outputs.verdict);
    let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let verdict: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let hash = sha256_hex_bytes(&bytes);
    if entries.iter().any(|e| e["verdict_sha256"] == hash.as_str()) {
        println!("step {}: recorded already", declaration.step);
        return Ok(0);
    }
    entry.insert("result".into(), ACCEPTED.into());
    entry.insert("verdict_sha256".into(), hash.into());
    entry.insert("functions".into(), rows(&verdict).into());
    entry.insert("size".into(), verdict["size"].clone());
    println!("step {}: recorded as accepted", declaration.step);
    let ledger = context.repo.join(&declaration.outputs.ledger);
    append(&ledger, entry).map(|()| 0)
}

/// Exit 0 when an entry was appended or was there already, 1 when the
/// verdict is not accepted or no longer matches.
pub fn run_record(args: RecordArgs) -> Result<u8, String> {
    if let (Some(name), Some(ledger)) = (&args.release, &args.ledger) {
        let ledger = args.repo.join(ledger);
        return release(&ledger, &read(&ledger)?, name);
    }
    let path = args.declaration.as_deref().unwrap_or_default();
    let context = Context::open(&args.repo, path)?;
    let declaration = &context.declaration;
    let ledger = args.repo.join(&declaration.outputs.ledger);
    let entries = read(&ledger)?;
    let mut entry = object(json!({
        "step": declaration.step, "date": today(),
        "base": context.revisions.base, "revision": context.revisions.step,
        "hashes": context.hashes_of(&context.step_tree),
    }));
    if !args.overridden {
        return accepted(&context, &entries, entry);
    }
    let reason = args.reason.as_deref().unwrap_or("").trim();
    if reason.is_empty() {
        return Err("an override needs a reason".into());
    }
    entry.insert("result".into(), OVERRIDDEN.into());
    entry.insert("reason".into(), reason.into());
    println!("step {}: overridden ({reason})", declaration.step);
    append(&ledger, entry).map(|()| 0)
}
