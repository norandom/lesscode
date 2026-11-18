//! Prepare a reviewer request using only source and the two runs before the edit.
use super::{ensure, predict, save, Result};
use lesscode_core::{adapters::driver::tree, receipt::sha256_hex};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, path::Path};

const QUESTION: &str = r#"The two observations below precede a planned project edit. A single worker tests mutations. Its test command reads src/app.py and tests/check.txt relative to its working directory. It succeeds when every nonempty line from check.txt occurs literally in app.py. The original check file contains the line "def step(". Replace its contents with the line "if a > 3:". Both files end in a newline. The selected mutation inverts the condition on line 2 of app.py. Its mutated text retains "def step(" and no longer contains the exact text "if a > 3:". No other input changes, no journal is pending, and there are no setup commands.

Predict two separate next runs after this edit. "reopened" keeps the work directory and stored outcomes from the unchanged run. "fresh" starts with an empty work directory and outcome store. For the selected mutation, predict which version of check.txt is visible to the test, whether the stored mutation result is reused, and the driver classification. "survived" means the mutation passed the test and "killed" means it failed.

Return one JSON object. Provide "reopened" and "fresh", each with "test_version" (original, edited, or unknown), "reused" (true, false, or null), and "classification" (killed, survived, or unknown). Also provide "basis", an object with three string fields "identity", "workspace", and "execution". For each use one exact module::function identifier from the supplied code that supports that link. In "explanation" give a brief causal account of both forecasts. Use only the supplied evidence."#;

#[derive(Deserialize)]
struct Site {
    file: String,
    function: String,
    start_line: usize,
    quote: String,
}

#[derive(Deserialize)]
struct Inventory {
    sites: Vec<Site>,
    sources: BTreeMap<String, String>,
}

fn functions(native: &Path, inventory: &Inventory) -> Result<Vec<Value>> {
    let mut selected = vec![
        ("run_command", "identity_with_tests"),
        ("store", "key"),
        ("runner", "open_copy"),
        ("workspace", "create"),
        ("workspace", "open"),
    ];
    if inventory
        .sources
        .contains_key("src/adapters/driver/sync.rs")
    {
        selected.push(("sync", "refresh"));
    }
    let mut excerpts = Vec::new();
    let language = tree::grammar("rust")?;
    for (module, name) in selected {
        let file = format!("src/adapters/driver/{module}.rs");
        let source = fs::read_to_string(native.join(&file))?;
        ensure(
            inventory.sources.get(&file) == Some(&sha256_hex(source.as_bytes())),
            "source hash mismatch",
        )?;
        let ast = tree::parse(&source, language)?;
        ensure(tree::first_error(&ast.root()).is_none(), "invalid source")?;
        let nodes: Vec<_> = tree::walk(&ast.root())
            .into_iter()
            .filter(|n| {
                n.kind() == "function_item"
                    && n.field("name").is_some_and(|id| id.text() == name)
                    && !tree::ancestors(n).iter().any(|a| a.kind() == "mod_item")
            })
            .collect();
        ensure(nodes.len() == 1, "expected one implementation")?;
        let node = &nodes[0];
        let code = node
            .text()
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        excerpts.push(
            json!({"id":format!("{module}::{name}"),"line":tree::position(node).0,"code":code}),
        );
    }
    for s in &inventory.sites {
        let module = Path::new(&s.file)
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("invalid source site")?;
        excerpts.push(json!({"id":format!("{}::{}",module,s.function),"line":s.start_line,"expression":s.quote}));
    }
    Ok(excerpts)
}

pub fn request(native: &Path, early: &Path, out: &Path) -> Result<()> {
    let sites = early.join("sites.json");
    let inventory: Inventory = serde_json::from_slice(&fs::read(&sites)?)?;
    let histories = predict::histories(
        &sites,
        early,
        Path::new("tests/check.txt"),
        &["warm", "unchanged"],
    )?;
    let mut observations = Vec::new();
    let original = sha256_hex(b"def step(\n");
    for phase in ["warm", "unchanged"] {
        let p = &histories[phase];
        let commands = p["commands"].as_array().ok_or("missing process events")?;
        let first = commands.first().ok_or("no test process")?;
        ensure(
            p["project_hash"] == original
                && commands.iter().all(|c| {
                    c["cwd"] == first["cwd"]
                        && c["current"] == true
                        && c["last_copy"] == first["last_copy"]
                }),
            "early observations disagree with the original check or its copy",
        )?;
        let cwd = first["cwd"].as_str().ok_or("missing cwd")?;
        ensure(
            Path::new(cwd).ends_with(Path::new("w1/tree")),
            "unexpected workspace path",
        )?;
        observations.push(json!({
            "phase":phase, "workspace_operation":p["worker"], "project_test_version":"original",
            "keys_looked_up":p["outcome_key_hits"].as_object().ok_or("missing keys")?.len(),
            "cache_hits":p["cache_hits"], "mutation_runs":p["mutant_executions"],
            "test_commands":commands.len(), "working_directory_under_work":"w1/tree",
            "relative_check":"tests/check.txt","last_observed_check_copy_in":first["last_copy"]["phase"]
        }));
    }
    let data = json!({"code":functions(native, &inventory)?, "observations":observations});
    save(
        out,
        &json!({"protocol":1,"purpose":"worker behavior forecast",
        "system":"Forecast the specified runs from the supplied facts. Return JSON only.",
        "user":format!("{QUESTION}\n\n{}",serde_json::to_string_pretty(&data)?),"max_output_tokens":8000}),
    )
}
