use super::{save, Lines, Result};
use serde::Deserialize;
use serde_json::json;
use std::{fs, path::Path};

#[derive(Deserialize)]
struct Selection {
    lines: Lines,
}
#[derive(Deserialize)]
struct Answer {
    evidence: Vec<Citation>,
}
#[derive(Deserialize)]
struct Citation {
    file: String,
    line: usize,
    quote: String,
}

fn resolve(lines: &Lines, c: &Citation) -> (&'static str, Option<usize>, Vec<usize>) {
    let Some(file) = lines.get(&c.file) else {
        return ("unknown_file", None, vec![]);
    };
    let quote = c.quote.trim();
    let matches: Vec<_> = file
        .iter()
        .filter(|(_, s)| !quote.is_empty() && s.trim() == quote)
        .map(|(line, _)| *line)
        .collect();
    if matches.contains(&c.line) {
        return ("exact", Some(c.line), matches);
    }
    match matches.as_slice() {
        [] => ("missing", None, matches),
        [only] => ("relocated", Some(*only), matches),
        _ => ("ambiguous", None, matches),
    }
}

pub(super) fn run(selection: &Path, answer: &Path, out: &Path) -> Result<()> {
    let source: Selection = serde_json::from_slice(&fs::read(selection)?)?;
    let envelope: serde_json::Value = serde_json::from_slice(&fs::read(answer)?)?;
    if envelope["ok"] != true || envelope["reply"]["stop"] != "complete" {
        return Err("incomplete model answer".into());
    }
    let text = envelope["reply"]["text"]
        .as_str()
        .ok_or("missing answer text")?
        .trim();
    let text = text
        .strip_prefix("```json")
        .and_then(|s| s.strip_suffix("```"))
        .unwrap_or(text);
    let answer: Answer = serde_json::from_str(text)?;
    if answer.evidence.is_empty() {
        return Err("no citations supplied".into());
    }
    let mut rows = Vec::new();
    let mut complete = true;
    for c in &answer.evidence {
        let (status, line, matches) = resolve(&source.lines, c);
        complete &= line.is_some();
        rows.push(
            json!({"file":c.file,"claimed_line":c.line,"quote":c.quote,"status":status,
            "resolved_line":line,"matching_lines":matches}),
        );
    }
    save(out, &json!({"all_resolved":complete,"citations":rows}))?;
    if !complete {
        return Err("unresolved citations (see report)".into());
    }
    Ok(())
}
