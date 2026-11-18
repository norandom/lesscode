use super::{ensure, save, Result};
use lesscode_core::{adapters::driver::tree, receipt::sha256_hex};
use serde::Serialize;
use std::{collections::BTreeMap, fs, ops::Range, path::Path};

#[derive(Serialize)]
struct Site {
    id: String,
    operation: String,
    file: String,
    function: String,
    start_line: usize,
    end_line: usize,
    bytes: Range<usize>,
    quote: String,
}

// Only these observation boundaries are admitted in this known-case trial.
// For filesystem copies, discover all production call sites in the driver.
fn operation(module: &str, owner: &str, callee: &str) -> Option<&'static str> {
    match (module, owner, callee) {
        ("source", "test_files_digest", "std::fs::read") => Some("read_test"),
        ("run_command", "run_run", "test_files_digest") => Some("test_digest"),
        ("run_command", "run_run", "identity_with_tests") => Some("identity"),
        ("run_command", "planned", "Store::key") => Some("key"),
        ("runner", "run_batch", "s.get") => Some("lookup"),
        ("runner", "open_copy", "Workspace::open") => Some("open"),
        ("runner", "open_copy", "Workspace::create") => Some("create"),
        ("runner", "one", "run_steps") => Some("mutant_tree"),
        ("process", "run", name) if name.ends_with(".current_dir") => Some("command_dir"),
        (_, _, "fs::copy" | "std::fs::copy") => Some("copy"),
        _ => None,
    }
}

fn rewrite(
    op: &str,
    id: &str,
    original: &str,
    args: &[tree::SgNode<'_>],
    span: &Range<usize>,
) -> Result<(Range<usize>, String)> {
    let probe = "crate::__lc_trace";
    let arg = |i: usize| {
        args.get(i)
            .map(|n| n.text().into_owned())
            .ok_or("missing call argument")
    };
    let replacement = match op {
        "read_test" => format!("{probe}::read({id}, {})", arg(0)?),
        "copy" => format!("{probe}::copy({id}, {}, {})", arg(0)?, arg(1)?),
        "test_digest" => format!("{probe}::digest({id}, {original})"),
        "identity" => format!("{probe}::identity({id}, {original})"),
        "key" => {
            ensure(arg(2)? == "identity", "unsupported key identity expression")?;
            format!("{probe}::key({id}, identity, {original})")
        }
        "lookup" => {
            ensure(arg(0)? == "&job.key", "unsupported lookup key expression")?;
            format!("{probe}::lookup({id}, &job.key, {original})")
        }
        "open" | "create" => format!("{probe}::workspace({id}, {op:?}, {original})"),
        "mutant_tree" => {
            let value = arg(1)?;
            return Ok((args[1].range(), format!("{probe}::tree({id}, {value})")));
        }
        "command_dir" => {
            let value = arg(0)?;
            return Ok((
                args[0].range(),
                format!("{probe}::cwd({id}, command, {value})"),
            ));
        }
        _ => return Err("unsupported probe".into()),
    };
    Ok((span.clone(), replacement))
}
pub fn overlay(root: &Path, out: &Path) -> Result<()> {
    let driver = "src/adapters/driver";
    let mut paths = fs::read_dir(root.join(driver))?
        .map(|e| e.map(|p| p.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    paths.sort();
    let mut sites = Vec::new();
    let mut sources = BTreeMap::new();
    let mut outputs = BTreeMap::new();
    let mut counts = BTreeMap::new();
    let language = tree::grammar("rust")?;
    for path in paths
        .into_iter()
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"))
    {
        let module = path
            .file_stem()
            .and_then(|n| n.to_str())
            .ok_or("invalid module name")?;
        let file = format!("{driver}/{module}.rs");
        let mut source = fs::read_to_string(&path)?;
        let ast = tree::parse(&source, language)?;
        ensure(
            tree::first_error(&ast.root()).is_none(),
            "invalid Rust module",
        )?;
        sources.insert(file.clone(), sha256_hex(source.as_bytes()));
        let mut edits = Vec::new();
        for call in tree::walk(&ast.root())
            .into_iter()
            .filter(|n| n.kind() == "call_expression")
        {
            let ancestors = tree::ancestors(&call);
            // The historical test modules are nested; the driver functions are not.
            if ancestors.iter().any(|n| n.kind() == "mod_item") {
                continue;
            }
            let Some(owner) = ancestors
                .iter()
                .find(|n| n.kind() == "function_item")
                .and_then(|n| n.field("name"))
            else {
                continue;
            };
            let owner = owner.text();
            let callee = call.field("function").ok_or("missing call target")?;
            let Some(op) = operation(module, &owner, &callee.text()) else {
                continue;
            };
            let args = tree::named_children(
                &call.field("arguments").ok_or("missing arguments")?,
                &["line_comment".into(), "block_comment".into()],
            );
            let (line, column, end, _) = tree::position(&call);
            let id = format!("{file}:{line}:{column}");
            let quote = call.text().into_owned();
            let bytes = call.range();
            edits.push(rewrite(op, &format!("{id:?}"), &quote, &args, &bytes)?);
            sites.push(Site {
                id,
                operation: op.into(),
                file: file.clone(),
                function: owner.into_owned(),
                start_line: line,
                end_line: end,
                bytes,
                quote,
            });
            *counts.entry(op).or_insert(0usize) += 1;
        }
        if !edits.is_empty() {
            edits.sort_by_key(|(s, _)| s.start);
            ensure(
                edits.windows(2).all(|p| p[0].0.end <= p[1].0.start),
                "overlapping probe sites",
            )?;
            for (range, replacement) in edits.into_iter().rev() {
                source.replace_range(range, &replacement);
            }
            outputs.insert(file, source);
        }
    }
    for op in [
        "read_test",
        "test_digest",
        "identity",
        "key",
        "lookup",
        "create",
        "open",
        "mutant_tree",
        "command_dir",
    ] {
        ensure(
            counts.get(op) == Some(&1),
            &format!("expected one {op} boundary"),
        )?;
    }
    ensure(counts.contains_key("copy"), "missing copy boundary")?;
    let lib = fs::read_to_string(root.join("src/lib.rs"))?;
    ensure(!lib.contains("__lc_trace"), "input already instrumented")?;
    outputs.insert("src/lib.rs".into(), format!("{lib}\nmod __lc_trace;\n"));
    outputs.insert("src/__lc_trace.rs".into(), include_str!("probes.rs").into());
    fs::create_dir(out)?;
    for (name, source) in outputs {
        let file = out.join(name);
        fs::create_dir_all(file.parent().ok_or("missing parent")?)?;
        fs::write(file, source)?;
    }
    save(
        &out.join("sites.json"),
        &serde_json::json!({"sources":sources,"sites":sites}),
    )
}
