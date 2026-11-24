//! What `step protect begin` writes once its checks have passed: the
//! declaration of the stage (`step.json`) and `stage.json`, into the
//! directory of the stage spec and nowhere else.

use super::declaration::Declaration;
use super::protect_begin::{json, BeginArgs, Begun};
use super::protect_stage::{
    chosen, run_again, unnoticed, with_unnoticed, LeftOut, Named, PlanRef, Reported, Stage,
    NOT_BUILT, RECEIPT,
};
use crate::specimen_runner::sha256_hex_bytes;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::path::Path;

/// Where accepted steps of a repository are recorded.
const LEDGER: &str = ".kiro/steps/ledger.jsonl";

/// `stage.json` of what the checks left, starting from the commit `base`.
pub fn stage_of(args: &BeginArgs, begun: &Begun, base: &str) -> Result<Stage, String> {
    let bytes = std::fs::read(&args.plan).map_err(|e| format!("{}: {e}", args.plan.display()))?;
    let rows = begun.plan["functions"].as_array().map(Vec::as_slice);
    let rows = rows.unwrap_or(&[]);
    let is =
        |row: &Value, one: &Named| row["file"] == *one.file && row["function"] == *one.function;
    let row_of = |one: &Named| rows.iter().find(|row| is(row, one)).unwrap_or(&Value::Null);
    let faults = |one: &Named| begun.planted.get(one).map_or(&[][..], Vec::as_slice);
    let not_taken = |one: &&Named| !begun.chosen.contains(one) && !begun.left_out.contains(one);
    let report = |one: &Named| Reported {
        file: one.file.clone(),
        function: one.function.clone(),
        unnoticed: unnoticed(row_of(one)),
    };
    let out = |one: &Named| LeftOut {
        file: one.file.clone(),
        function: one.function.clone(),
        why: NOT_BUILT.to_owned(),
    };
    let rows_of_chosen = begun.chosen.iter();
    Ok(Stage {
        receipt: RECEIPT.to_owned(),
        spec: args.spec.clone(),
        base: base.to_owned(),
        plan: PlanRef {
            file: args.plan.to_string_lossy().replace('\\', "/"),
            sha256: sha256_hex_bytes(&bytes),
        },
        test: begun.config.test.clone(),
        chosen: rows_of_chosen
            .map(|one| chosen(row_of(one), one, faults(one)))
            .collect(),
        reported: with_unnoticed(&begun.plan)
            .iter()
            .filter(not_taken)
            .map(report)
            .collect(),
        left_out: begun.left_out.iter().map(out).collect(),
        to_run_again: run_again(begun.planted.values()),
    })
}

/// The fields of a run configuration that a declaration sets itself; the
/// others are the run settings it carries along.
const OWN: [&str; 9] = [
    "language",
    "source_root",
    "files",
    "functions",
    "test_paths",
    "setup",
    "build",
    "test",
    "env",
];

/// `step.json`: the declaration of the stage as a step that changes test
/// files only. The scope is the files of the chosen functions, the focus
/// the functions, and the scope may not grow.
pub fn declaration_of(args: &BeginArgs, begun: &Begun, base: &str) -> Result<Value, String> {
    let config = &begun.config;
    let mut run = match json(&args.config, "the run configuration")? {
        Value::Object(fields) => fields,
        _ => Map::new(),
    };
    run.retain(|key, _| !OWN.contains(&key.as_str()));
    let mut focus: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for one in &begun.chosen {
        focus.entry(&one.file).or_default().push(&one.function);
    }
    let beside = |name: &str| format!("{}/{}/{name}", args.specs_dir, args.spec);
    let named = args.branch.clone();
    let branch = named.unwrap_or(format!("protect/{}", args.spec));
    let mut declaration = json!({
        "step": args.spec, "spec": args.spec, "branch": branch, "base": base,
        "language": config.language, "scope": focus.keys().collect::<Vec<_>>(),
        "test_paths": config.test_paths, "test": config.test, "env": config.env,
        "run": run, "focus": focus, "max_token_growth": 0,
        "outputs": {"verdict": beside("verdict.json"), "evidence": beside("evidence.json"),
                    "ledger": LEDGER},
    });
    if !config.setup.is_empty() {
        declaration["setup"] = json!(config.setup);
    }
    if let Some(build) = &config.build {
        declaration["build"] = json!(build);
    }
    Ok(declaration)
}

/// Writes the two files of the stage into its spec's directory, and
/// nothing else. The declaration is read back as `step check` reads it
/// before anything is written.
pub fn write(dir: &Path, declaration: &Value, stage: &Stage) -> Result<(), String> {
    let pretty = |value: &Value| serde_json::to_string_pretty(value).map(|text| text + "\n");
    let step = pretty(declaration).map_err(|e| e.to_string())?;
    Declaration::parse(step.as_bytes())
        .map_err(|e| format!("the declaration of the stage: {e}"))?;
    let stage = serde_json::to_value(stage).and_then(|v| pretty(&v));
    let stage = stage.map_err(|e| e.to_string())?;
    let said = |e: std::io::Error| format!("{}: {e}", dir.display());
    std::fs::create_dir_all(dir).map_err(said)?;
    std::fs::write(dir.join("step.json"), step).map_err(said)?;
    std::fs::write(dir.join("stage.json"), stage).map_err(said)
}
