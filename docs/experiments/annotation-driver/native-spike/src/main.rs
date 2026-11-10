//! CLI glue: one mode per invocation, JSON on stdout or to `--out`.
//!
//! ```text
//! spike sites      --packs DIR --scopes-dir DIR --manifest FILE --scope NAME
//! spike mutants    --packs DIR --scopes-dir DIR --manifest FILE --scope NAME
//! spike controls   --packs DIR --scopes-dir DIR --manifest FILE --controls FILE
//! spike check-pack FILE --language LANG
//! spike classify   --packs DIR --language LANG --original FILE --mutated FILE
//! spike tree       FILE --language LANG   (every node: kind, named, position)
//! ```
//! Every mode also takes `--out FILE`. Exit 0 on success, 1 when controls fail
//! or a pack is not admitted, 2 on a usage or input error.

use annotation_driver_spike::controls::{run_scope, ControlsFile};
use annotation_driver_spike::engine::{eligible_sites, mutants};
use annotation_driver_spike::pack::Pack;
use annotation_driver_spike::pack_check::check_pack;
use annotation_driver_spike::scope::{find_scope, load_scopes};
use annotation_driver_spike::tree::{grammar, parse, position, walk};
use annotation_driver_spike::validator::classify_mutant;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

struct Args {
    mode: String,
    positional: Option<String>,
    options: HashMap<String, String>,
}

impl Args {
    fn parse() -> Result<Args, String> {
        let mut raw = std::env::args().skip(1);
        let mode = raw.next().ok_or("no mode given")?;
        let (mut positional, mut options) = (None, HashMap::new());
        while let Some(arg) = raw.next() {
            match arg.strip_prefix("--") {
                Some(name) => {
                    let value = raw.next().ok_or(format!("--{name} needs a value"))?;
                    options.insert(name.to_owned(), value);
                }
                None if positional.is_none() => positional = Some(arg),
                None => return Err(format!("unexpected argument {arg}")),
            }
        }
        Ok(Args {
            mode,
            positional,
            options,
        })
    }

    fn get(&self, name: &str) -> Result<&str, String> {
        self.options
            .get(name)
            .map(String::as_str)
            .ok_or(format!("--{name} is required"))
    }

    fn path(&self, name: &str) -> Result<PathBuf, String> {
        self.get(name).map(PathBuf::from)
    }
}

fn read(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    String::from_utf8(bytes).map_err(|e| format!("{} is not UTF-8: {e}", path.display()))
}

fn sha256(text: &str) -> String {
    Sha256::digest(text.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn site_key(site: &annotation_driver_spike::engine::Site) -> String {
    let s = site;
    format!(
        "{}:{}:{}:{}-{}:{}",
        s.file, s.kind, s.start_line, s.start_column, s.end_line, s.end_column
    )
}

fn run(args: &Args) -> Result<(Value, bool), String> {
    match args.mode.as_str() {
        "check-pack" => {
            let file = PathBuf::from(
                args.positional
                    .as_deref()
                    .ok_or("check-pack needs a FILE")?,
            );
            let admission = check_pack(&read(&file)?, args.get("language")?);
            let ok = admission.admitted;
            Ok((json!(admission), ok))
        }
        "tree" => {
            let file = PathBuf::from(args.positional.as_deref().ok_or("tree needs a FILE")?);
            let lang = grammar(args.get("language")?)?;
            let root = parse(&read(&file)?, lang)?;
            let nodes: Vec<Value> = walk(&root.root())
                .iter()
                .map(|n| json!([n.kind(), n.is_named(), position(n)]))
                .collect();
            Ok((Value::Array(nodes), true))
        }
        "classify" => {
            let language = args.get("language")?;
            let pack = Pack::load(&args.path("packs")?, language)?;
            let (original, mutated) = (
                read(&args.path("original")?)?,
                read(&args.path("mutated")?)?,
            );
            Ok((
                json!(classify_mutant(&pack, language, &original, &mutated)?),
                true,
            ))
        }
        mode @ ("sites" | "mutants" | "controls") => {
            let scopes = load_scopes(&args.path("manifest")?, &args.path("scopes-dir")?)?;
            let packs = args.path("packs")?;
            if mode == "controls" {
                let controls: ControlsFile = serde_json::from_str(&read(&args.path("controls")?)?)
                    .map_err(|e| format!("controls file: {e}"))?;
                let mut results = Vec::new();
                for (name, spec) in &controls.scopes.0 {
                    let scope = find_scope(&scopes, name)?;
                    results.extend(run_scope(
                        scope,
                        &Pack::load(&packs, &scope.language)?,
                        spec,
                    )?);
                }
                let failed = results.iter().filter(|r| !r.pass).count();
                let report =
                    json!({"controls": results.len(), "failed": failed, "results": results});
                return Ok((report, failed == 0));
            }
            let scope = find_scope(&scopes, args.get("scope")?)?;
            let pack = Pack::load(&packs, &scope.language)?;
            let sites = eligible_sites(scope, &pack)?;
            if mode == "sites" {
                return Ok((json!(sites), true));
            }
            let mut out = Vec::new();
            for site in &sites {
                for m in mutants(scope, &pack, site)? {
                    out.push(json!({"site": site_key(site), "operator": m.operator, "sha256": sha256(&m.text)}));
                }
            }
            Ok((Value::Array(out), true))
        }
        other => Err(format!("unknown mode {other}")),
    }
}

fn main() -> ExitCode {
    let result = Args::parse().and_then(|args| {
        let (value, ok) = run(&args)?;
        let text = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())? + "\n";
        match args.options.get("out") {
            Some(out) => {
                std::fs::write(out, text).map_err(|e| format!("cannot write {out}: {e}"))?
            }
            None => print!("{text}"),
        }
        Ok(ok)
    });
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(2)
        }
    }
}
