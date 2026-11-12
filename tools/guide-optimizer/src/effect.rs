//! What an admitted pack that differs from the shipped pack does to a gate.
//! Every mutant of the shipped pack on a gate scope has a recorded class
//! (killed, survived, invalid, ...) in the driver's receipts. A pack that
//! lacks a survived mutant no longer reports that finding; what its extra
//! mutants would do is not recorded and is only counted.

use crate::task::{Language, mutant_keys, pack_text};
use lesscode_core::adapters::driver::pack::Pack;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;

/// Mutant key to class, from every `*-receipt.json` in a directory.
pub fn classes(dir: &Path) -> Result<BTreeMap<String, String>, String> {
    let mut found = BTreeMap::new();
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for path in entries.flatten().map(|e| e.path()) {
        if !path.to_string_lossy().ends_with("-receipt.json") {
            continue;
        }
        let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let receipt: Value =
            serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        for mutant in receipt["mutants"].as_array().into_iter().flatten() {
            let (site, text) = (&mutant["site"], |v: &Value| {
                v.as_str().unwrap_or("").to_owned()
            });
            let operator = text(&mutant["operator"]);
            let key = format!(
                "{}:{}:{}|{}|{}|{}",
                text(&site["file"]),
                site["start_line"],
                site["start_column"],
                text(&site["kind"]),
                operator.split('@').next().unwrap_or_default(),
                text(&mutant["sha256"])
            );
            found.insert(key, text(&mutant["class"]));
        }
    }
    Ok(found)
}

/// Per gate scope: the classes of the shipped pack's mutants, of those the
/// pack in `reply` keeps and of those it lacks, and how many it adds.
pub fn effect(
    language: &Language,
    reply: &str,
    classes: &BTreeMap<String, String>,
) -> Result<Value, String> {
    let pack = Pack::parse(&pack_text(reply)).map_err(|e| e.to_string())?;
    let tally = |keys: Vec<&String>| {
        // Counted per site kind and class, as "state_write:survived".
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for key in keys {
            let class = classes.get(key).map_or("not_recorded", String::as_str);
            let kind = key.split('|').nth(1).unwrap_or_default();
            *counts.entry(format!("{kind}:{class}")).or_default() += 1;
        }
        json!(counts)
    };
    let mut scopes = Vec::new();
    for scope in &language.scopes {
        let Some(shipped) = &scope.shipped else {
            continue;
        };
        let written = mutant_keys(&pack, &scope.source)?;
        let has = |key: &&String| written.contains(key);
        scopes.push(json!({
            "scope": scope.name,
            "shipped": tally(shipped.iter().collect()),
            "kept": tally(shipped.iter().filter(has).collect()),
            "missing": tally(shipped.iter().filter(|k| !has(k)).collect()),
            "extra": written.iter().filter(|k| !shipped.contains(k)).count(),
        }));
    }
    Ok(json!(scopes))
}
