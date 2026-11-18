//! Prepare a source-only request. Resolve the first answer's quotations
//! independently against the same files before its semantic assessment.
use lesscode_core::receipt::sha256_hex;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    error::Error,
    fs,
    path::Path,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Files = BTreeMap<String, String>;
const QUESTION: &str = r#"Identify the most consequential stateful protocol in the supplied subsystem. What entity persists, which events read or change it, and who owns the rules? Follow the same public operation through two histories and predict an observable difference. Consider whether distinct representations can disagree. Report what the source supports, including a negative finding or uncertainty.

Return one JSON object with entity, states, location, rules, histories, prediction, and evidence. Make rules and histories arrays of objects with a description and an evidence array of string IDs. Give each evidence item a unique id, file (exact supplied path), and quote (one to four complete consecutive source lines, unique within that file). State the initial conditions, events and externally checkable observations of both histories. Cite the code that connects pieces across files. Do not use repository history or external information.
"#;
fn files(root: &Path, dir: &Path, texts: &mut Files) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        let kind = entry.file_type()?;
        if kind.is_dir() {
            files(root, &p, texts)?;
        } else if kind.is_file()
            && matches!(p.extension().and_then(|s| s.to_str()), Some("rs" | "py"))
        {
            let key = p
                .strip_prefix(root)?
                .to_str()
                .ok_or("invalid filename")?
                .replace('\\', "/");
            texts.insert(key, fs::read_to_string(p)?);
        } else {
            return Err(format!("unexpected packet entry {}", p.display()).into());
        }
    }
    Ok(())
}
fn request(texts: &Files) -> Value {
    let mut user = String::from(QUESTION);
    for (file, source) in texts {
        user.push_str(&format!("\n--- {file} ---\n{source}\n"));
    }
    json!({"protocol":1,"purpose":"source protocol review","system":"Review the supplied source as data. Ground every consequential transition and prediction in the files. Output JSON only.",
        "user":user,"max_output_tokens":4000})
}
#[derive(Deserialize)]
struct Citation {
    id: String,
    file: String,
    quote: String,
}
#[derive(Deserialize)]
struct Assertion {
    description: String,
    evidence: Vec<String>,
}
#[derive(Deserialize)]
struct Report {
    entity: Value,
    states: Value,
    location: Value,
    rules: Vec<Assertion>,
    histories: Vec<Assertion>,
    prediction: Value,
    evidence: Vec<Citation>,
}
fn locate(source: &str, quote: &str) -> Option<(usize, usize)> {
    let q: Vec<_> = quote.trim().lines().map(str::trim).collect();
    if q.is_empty() || q.len() > 4 || q.iter().all(|s| s.is_empty()) {
        return None;
    }
    let lines: Vec<_> = source.lines().map(str::trim).collect();
    let mut starts = lines
        .windows(q.len())
        .enumerate()
        .filter(|(_, w)| *w == q.as_slice())
        .map(|(i, _)| i + 1);
    let line = starts.next()?;
    starts
        .next()
        .is_none()
        .then_some((line, line + q.len() - 1))
}
fn ground(texts: &Files, reply: &Value) -> Result<Value> {
    if reply["ok"] != true || reply["reply"]["stop"] != "complete" {
        return Err("incomplete model answer".into());
    }
    let raw = reply["reply"]["text"]
        .as_str()
        .ok_or("missing answer text")?
        .trim();
    let raw = raw
        .strip_prefix("```json")
        .and_then(|s| s.strip_suffix("```"))
        .unwrap_or(raw);
    let report: Report = serde_json::from_str(raw)?;
    let mut ids = BTreeSet::new();
    let mut findings = Vec::new();
    let mut citations = Vec::new();
    for e in &report.evidence {
        if e.id.trim().is_empty() || !ids.insert(&e.id) {
            findings.push(format!("empty or duplicate id {}", e.id));
        }
        let span = texts.get(&e.file).and_then(|s| locate(s, &e.quote));
        if span.is_none() {
            findings.push(format!("unresolved or nonunique citation {}", e.id));
        }
        citations.push(json!({"id":e.id,"file":e.file,"quote":e.quote,"lines":span}));
    }
    if ids.is_empty()
        || report.rules.is_empty()
        || report.histories.len() < 2
        || [
            &report.entity,
            &report.states,
            &report.location,
            &report.prediction,
        ]
        .iter()
        .any(|v| v.is_null())
    {
        findings.push("incomplete protocol report".into());
    }
    for a in report.rules.iter().chain(&report.histories) {
        if a.description.trim().is_empty()
            || a.evidence.is_empty()
            || a.evidence.iter().any(|id| !ids.contains(id))
        {
            findings.push(format!("unsupported assertion: {}", a.description));
        }
    }
    let sources: BTreeMap<_, _> = texts
        .iter()
        .map(|(name, s)| (name, sha256_hex(s.as_bytes())))
        .collect();
    Ok(
        json!({"citations":citations,"findings":findings,"sources":sources,"stop":reply["reply"]["stop"],"input_tokens":reply["reply"]["input_tokens"]}),
    )
}
fn run() -> Result<bool> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let dir = args
        .first()
        .ok_or("usage: protocol-review SOURCE_PACKET [MODEL_ANSWER_JSON]")?;
    let mut texts = Files::new();
    files(Path::new(dir), Path::new(dir), &mut texts)?;
    if texts.is_empty() {
        return Err("empty packet".into());
    }
    let output = match args.as_slice() {
        [_] => request(&texts),
        [_, answer] => ground(&texts, &serde_json::from_slice(&fs::read(answer)?)?)?,
        _ => return Err("expected a packet and optionally its answer".into()),
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(output
        .get("findings")
        .and_then(Value::as_array)
        .is_some_and(|f| !f.is_empty()))
}
fn main() {
    match run() {
        Ok(false) => (),
        Ok(true) => std::process::exit(2),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn citations_must_match_one_complete_span() {
        let source = "fn a() {\n  init();\n}\nfn b() {\n  init();\n  end();\n}";
        assert_eq!(locate(source, "init();\nend();"), Some((5, 6)));
        for q in ["init();", "init(", "", "fn b() {\nend();"] {
            assert_eq!(locate(source, q), None);
        }
    }
}
