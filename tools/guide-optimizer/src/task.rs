//! The task being optimised: write the pack for a language. A reply is
//! judged by the driver's admission with every control set of the language
//! (the minimum barrier) and, when admitted, by whether its mutants on the
//! gate scopes equal the shipped pack's.

use lesscode_core::adapters::driver::admission::admit;
use lesscode_core::adapters::driver::controls::{Case, ControlFile, replay, seed_mutants};
use lesscode_core::adapters::driver::operators::mutants;
use lesscode_core::adapters::driver::pack::Pack;
use lesscode_core::adapters::driver::registry::embedded;
use lesscode_core::adapters::driver::sites::eligible_sites;
use lesscode_core::adapters::driver::source::SourceSet;
use lesscode_core::specimen_runner::sha256_hex_bytes;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;

/// One control set with its source tree; `shipped` holds the shipped pack's
/// mutants for a gate scope and is absent for a hand-written set.
pub struct Scope {
    pub name: String,
    pub source: SourceSet,
    pub cases: Vec<Case>,
    pub shipped: Option<Vec<String>>,
}

pub struct Language {
    pub name: String,
    /// The user message of an authoring request, with the example packs.
    pub request: String,
    pub scopes: Vec<Scope>,
}

pub struct Verdict {
    /// 0 not admitted, 0.5 admitted, 1 admitted with the shipped pack's mutants.
    pub score: f64,
    pub admitted: bool,
    pub same: bool,
    pub feedback: Vec<String>,
    /// A hash of the mutants on the gate scopes; equal hashes, equal behaviour.
    pub behaviour: String,
}

/// The example packs shown for each language (R66): never its own.
fn examples(language: &str) -> [&'static str; 2] {
    match language {
        "rust" => ["python", "java"],
        "python" => ["rust", "java"],
        _ => ["rust", "python"],
    }
}

fn mutant_keys(pack: &Pack, source: &SourceSet) -> Result<Vec<String>, String> {
    let mut keys = Vec::new();
    for site in eligible_sites(source, pack)? {
        let text = source.raw(&site.file)?;
        for mutant in mutants(pack, &text, &site)? {
            let label = mutant
                .operator
                .split('@')
                .next()
                .unwrap_or_default()
                .to_owned();
            let sha = sha256_hex_bytes(mutant.text.as_bytes());
            keys.push(format!(
                "{}:{}:{}|{}|{label}|{sha}",
                site.file,
                site.start_line,
                site.start_column,
                site.kind.name()
            ));
        }
    }
    Ok(keys)
}

fn read_json(path: &Path) -> Result<Value, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))
}

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|s| s.as_str().map(str::to_owned))
        .collect()
}

fn sorted_files(dir: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    let mut found: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .flatten()
        .map(|e| e.path())
        .collect();
    found.sort();
    Ok(found)
}

/// The four languages with their gate scopes and hand-written control sets.
pub fn load_languages(repo: &Path) -> Result<BTreeMap<String, Language>, String> {
    let fixtures = repo.join("native/tests/fixtures/driver");
    let packs = repo.join("native/packs");
    let mut languages: BTreeMap<String, Language> = BTreeMap::new();
    let mut add = |language: &str, scope: Scope| -> Result<(), String> {
        if !languages.contains_key(language) {
            let shown: Result<Vec<String>, String> = examples(language)
                .iter()
                .map(|name| {
                    let path = packs.join(format!("{name}.json"));
                    let text = std::fs::read_to_string(&path)
                        .map_err(|e| format!("{}: {e}", path.display()))?;
                    Ok(format!("{name}.json:\n{text}"))
                })
                .collect();
            let request = format!(
                "Example packs:\n\n{}\n\nWrite the pack for: {language}",
                shown?.join("\n\n")
            );
            languages.insert(
                language.to_owned(),
                Language {
                    name: language.to_owned(),
                    request,
                    scopes: Vec::new(),
                },
            );
        }
        languages
            .get_mut(language)
            .expect("just inserted")
            .scopes
            .push(scope);
        Ok(())
    };
    for path in sorted_files(&fixtures.join("reference"))? {
        let reference = read_json(&path)?;
        let language = reference["language"]
            .as_str()
            .ok_or("reference without a language")?
            .to_owned();
        let source = SourceSet {
            root: fixtures
                .join("scopes")
                .join(reference["repository"].as_str().unwrap_or_default()),
            files: strings(&reference["files"]),
            test_paths: strings(&reference["test_paths"]),
        };
        let cases: Result<Vec<Case>, _> = reference["controls"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|c| serde_json::from_value(json!({"label": c["label"], "annotations": c["annotations"], "expected": c["expected"]})))
            .collect();
        let shipped = mutant_keys(embedded(&language).map_err(|e| e.to_string())?, &source)?;
        let name = reference["scope"].as_str().unwrap_or_default().to_owned();
        add(
            &language,
            Scope {
                name,
                source,
                cases: cases.map_err(|e| e.to_string())?,
                shipped: Some(shipped),
            },
        )?;
    }
    for dir in sorted_files(&fixtures.join("controls"))? {
        let file = ControlFile::read(&dir.join("controls.json"))?;
        let language = file
            .language
            .clone()
            .ok_or("a hand-written control set without a language")?;
        let name = format!(
            "controls-{}",
            dir.file_name().unwrap_or_default().to_string_lossy()
        );
        let source = file.source(&dir);
        add(
            &language,
            Scope {
                name,
                source,
                cases: file.cases,
                shipped: None,
            },
        )?;
    }
    Ok(languages)
}

/// The reply without surrounding white space and without one code fence
/// around the whole of it.
pub fn pack_text(reply: &str) -> String {
    let body = reply.trim();
    let inner = body
        .strip_prefix("```")
        .and_then(|rest| rest.strip_suffix("```"))
        .map(|fenced| {
            let fenced = fenced.strip_prefix("json").unwrap_or(fenced);
            fenced.trim().to_owned()
        });
    inner.unwrap_or_else(|| body.to_owned())
}

fn operators_of(keys: &[&String]) -> String {
    let mut labels: Vec<&str> = keys.iter().filter_map(|k| k.split('|').nth(2)).collect();
    labels.sort();
    labels.dedup();
    labels.join(", ")
}

/// Judge one pack text for a language.
pub fn judge(language: &Language, text: &str) -> Verdict {
    let rejected = |feedback: Vec<String>| Verdict {
        score: 0.0,
        admitted: false,
        same: false,
        feedback,
        behaviour: String::new(),
    };
    let admission = match admit(text, &language.name) {
        Ok(admission) => admission,
        Err(e) => return rejected(vec![e.to_string()]),
    };
    let Ok(pack) = Pack::parse(text) else {
        return rejected(admission.feedback());
    };
    let mut feedback = admission.feedback();
    let mut differences = Vec::new();
    let mut behaviour = String::new();
    for scope in &language.scopes {
        for (check, outcome) in [
            ("controls", replay(&pack, &scope.source, &scope.cases)),
            (
                "seed_mutants",
                seed_mutants(&pack, &scope.source, &scope.cases),
            ),
        ] {
            if let Err(detail) = outcome {
                feedback.push(format!("{check} ({}): {detail}", scope.name));
            }
        }
        let Some(shipped) = &scope.shipped else {
            continue;
        };
        match mutant_keys(&pack, &scope.source) {
            Err(e) => feedback.push(format!("mutants ({}): {e}", scope.name)),
            Ok(written) => {
                let missing: Vec<&String> =
                    shipped.iter().filter(|k| !written.contains(k)).collect();
                let extra: Vec<&String> = written.iter().filter(|k| !shipped.contains(k)).collect();
                if !missing.is_empty() || !extra.is_empty() {
                    differences.push(format!(
                        "on {}: {} changes of the reference pack are missing (operators: {}), {} are extra (operators: {})",
                        scope.name, missing.len(), operators_of(&missing), extra.len(), operators_of(&extra)
                    ));
                }
                behaviour.push_str(&written.join("\n"));
            }
        }
    }
    if !feedback.is_empty() {
        return rejected(feedback);
    }
    let same = differences.is_empty();
    let behaviour = sha256_hex_bytes(behaviour.as_bytes())[..12].to_owned();
    Verdict {
        score: if same { 1.0 } else { 0.5 },
        admitted: true,
        same,
        feedback: differences,
        behaviour,
    }
}
