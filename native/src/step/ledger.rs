//! The ledger of steps: one JSON object per line, only ever appended. Each
//! entry carries the hash of the line before it, so that a rewritten
//! earlier entry is noticed by every later reader.

use crate::specimen_runner::sha256_hex_bytes;
use serde_json::{Map, Value};
use std::io::Write;
use std::path::Path;

pub const ACCEPTED: &str = "accepted";
pub const OVERRIDDEN: &str = "overridden";
pub const RELEASE: &str = "release";

/// The entries of a ledger text, with the chain checked. A text that is
/// empty has no entries.
pub fn parse(text: &str) -> Result<Vec<Value>, String> {
    let mut entries = Vec::new();
    let mut previous = String::new();
    let lines = text.lines().map(|l| l.trim_end_matches('\r'));
    for (index, line) in lines.filter(|l| !l.trim().is_empty()).enumerate() {
        let number = index + 1;
        let entry: Value = serde_json::from_str(line)
            .map_err(|e| format!("ledger entry {number} is not JSON: {e}"))?;
        if entry["previous"].as_str() != Some(previous.as_str()) {
            return Err(format!(
                "the ledger was rewritten: entry {number} does not follow the entry before it"
            ));
        }
        previous = sha256_hex_bytes(line.as_bytes());
        entries.push(entry);
    }
    Ok(entries)
}

/// The entries of the ledger file; none when it does not exist.
pub fn read(path: &Path) -> Result<Vec<Value>, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => parse(&text).map_err(|e| format!("{}: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

/// Append an entry, chained to the last line. Earlier lines are read to
/// check the chain and are never written.
pub fn append(path: &Path, mut entry: Map<String, Value>) -> Result<(), String> {
    let failed = |e: std::io::Error| format!("{}: {e}", path.display());
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(failed(e)),
    };
    parse(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut lines = text.lines().map(|l| l.trim_end_matches('\r'));
    let last = lines.rfind(|l| !l.trim().is_empty());
    let previous = last.map_or(String::new(), |l| sha256_hex_bytes(l.as_bytes()));
    entry.insert("previous".into(), Value::String(previous));
    let line = Value::Object(entry).to_string();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(failed)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(failed)?;
    let gap = if text.is_empty() || text.ends_with('\n') {
        ""
    } else {
        "\n"
    };
    writeln!(file, "{gap}{line}").map_err(failed)
}

/// The number of unnoticed faults of a function after the last accepted
/// step that ran faults in it.
pub fn last_accepted(entries: &[Value], file: &str, function: &str) -> Option<usize> {
    let of_function = |entry: &Value| -> Option<usize> {
        let rows = entry["functions"].as_array()?;
        let row = rows
            .iter()
            .find(|r| r["file"] == file && r["function"] == function)?;
        row["after"]["unnoticed"].as_u64().map(|n| n as usize)
    };
    let accepted = entries.iter().filter(|e| e["result"] == ACCEPTED);
    accepted.filter_map(of_function).next_back()
}

/// The steps recorded after the last release entry.
pub fn since_release(entries: &[Value]) -> Vec<&Value> {
    let after_release = entries
        .iter()
        .rposition(|e| e["result"] == RELEASE)
        .map_or(0, |at| at + 1);
    let steps = entries[after_release..].iter();
    steps.filter(|e| e["result"] != RELEASE).collect()
}

/// Today's date in UTC as `YYYY-MM-DD`.
pub fn today() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    civil(seconds / 86_400)
}

/// The date of a day counted from 1970-01-01.
fn civil(days: u64) -> String {
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn entry(value: Value) -> Map<String, Value> {
        value.as_object().unwrap().clone()
    }

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("lesscode-ledger-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir.join("steps").join("ledger.jsonl")
    }

    #[test]
    fn dates_are_civil_dates() {
        assert_eq!(civil(0), "1970-01-01");
        assert_eq!(civil(59), "1970-03-01");
        assert_eq!(civil(11_016), "2000-02-29");
        assert_eq!(civil(20_768), "2026-11-11");
    }

    #[test]
    fn entries_are_chained_and_a_rewrite_is_noticed() {
        let path = scratch("chain");
        assert!(read(&path).unwrap().is_empty());
        let row = |unnoticed: u64| json!([{"file": "a", "function": "f", "after": {"unnoticed": unnoticed}}]);
        append(
            &path,
            entry(json!({"step": "one", "result": ACCEPTED, "functions": row(3)})),
        )
        .unwrap();
        append(
            &path,
            entry(json!({"step": "two", "result": OVERRIDDEN, "functions": row(9)})),
        )
        .unwrap();
        append(
            &path,
            entry(json!({"step": "three", "result": ACCEPTED, "functions": row(1)})),
        )
        .unwrap();
        let entries = read(&path).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0]["previous"], "");
        assert_ne!(entries[1]["previous"], entries[2]["previous"]);
        // The ratchet reads accepted steps only, the last one.
        assert_eq!(last_accepted(&entries, "a", "f"), Some(1));
        assert_eq!(last_accepted(&entries[..2], "a", "f"), Some(3));
        assert_eq!(last_accepted(&entries, "a", "g"), None);

        // A checkout that turns line ends into CRLF does not break the chain.
        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(parse(&text.replace('\n', "\r\n")).unwrap().len(), 3);

        // An earlier entry that was edited, or removed, is noticed.
        let edited = text.replacen("\"one\"", "\"uno\"", 1);
        assert!(parse(&edited).unwrap_err().contains("rewritten"));
        let cut: String = text.lines().skip(1).collect::<Vec<_>>().join("\n");
        assert!(parse(&cut).unwrap_err().contains("rewritten"));
        std::fs::write(&path, edited).unwrap();
        let refused = append(&path, entry(json!({"step": "four", "result": ACCEPTED})));
        assert!(refused.unwrap_err().contains("rewritten"));
        let _ = std::fs::remove_dir_all(path.parent().unwrap().parent().unwrap());
    }

    #[test]
    fn a_release_lists_the_steps_since_the_one_before() {
        let steps = |names: &[&str], result: &str| -> Vec<Value> {
            names
                .iter()
                .map(|n| json!({"step": n, "result": result}))
                .collect()
        };
        let mut entries = steps(&["a", "b"], ACCEPTED);
        entries.extend(steps(&["1.0"], RELEASE));
        entries.extend(steps(&["c"], ACCEPTED));
        entries.extend(steps(&["d"], OVERRIDDEN));
        let names = |found: Vec<&Value>| -> Vec<String> {
            found
                .iter()
                .map(|e| e["step"].as_str().unwrap().to_owned())
                .collect()
        };
        assert_eq!(names(since_release(&entries)), ["c", "d"]);
        assert_eq!(names(since_release(&entries[..2])), ["a", "b"]);
        assert!(since_release(&entries[..3]).is_empty());
    }
}
