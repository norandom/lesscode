//! `step protect begin`: from a plan and the functions a person names to
//! the start of a stage (`.kiro/specs/brownfield-test-stage`, group 1).
//!
//! Everything is checked before anything is written. A refusal is an error
//! (exit 3) that names what is wrong, and leaves the repository as it was.
//! The command reads the plan, the receipt of its run, the run
//! configuration and the code; it runs nothing.

use super::functions::{FileFunctions, OUTSIDE};
use super::protect_says::{run_of, says};
use super::protect_stage::{planted, Named, Planted};
use super::protect_write::{declaration_of, stage_of, write};
use super::revisions::resolve;
use crate::adapters::driver::config::RunConfig;
use crate::adapters::driver::pack::Pack;
use crate::adapters::driver::registry::embedded;
use crate::adapters::driver::sites::{not_built_spans, within};
use crate::adapters::driver::source::SourceSet;
use crate::specimen_runner::sha256_hex_bytes;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

/// Arguments of `step protect begin`.
#[derive(clap::Args, Clone, Debug)]
pub struct BeginArgs {
    /// The plan that `step plan` wrote; the functions are chosen from it.
    #[arg(long)]
    pub plan: PathBuf,
    /// The run configuration the plan was made with; its source root is the repository.
    #[arg(long)]
    pub config: PathBuf,
    /// A function to protect, as <file>:<name> with the file as the plan names it. Repeat for several.
    #[arg(long = "function", required = true, value_name = "FILE:NAME")]
    pub functions: Vec<String>,
    /// The name of the stage spec; its directory is <specs-dir>/<name>.
    #[arg(long, value_name = "NAME")]
    pub spec: String,
    /// The branch of the stage; protect/<name> when absent.
    #[arg(long)]
    pub branch: Option<String>,
    /// Where the repository keeps its specs, relative to its root.
    #[arg(long, default_value = ".kiro/specs")]
    pub specs_dir: String,
    /// The repository.
    #[arg(long, default_value = ".")]
    pub repo: PathBuf,
}

/// What the checks leave to begin a stage with.
pub struct Begun {
    pub config: RunConfig,
    pub plan: Value,
    /// The receipt of the plan's fault run.
    pub run: Value,
    pub chosen: Vec<Named>,
    /// Per chosen function, every fault the plan's run planted in it.
    pub planted: BTreeMap<Named, Vec<Planted>>,
    /// Named functions that lie wholly in code not built on this system.
    pub left_out: Vec<Named>,
    /// The directory of the stage spec.
    pub dir: PathBuf,
}

const PLAN_FIRST: &str = "A plan with a result is needed first: run `step plan` again.";

pub fn json(path: &Path, what: &str) -> Result<Value, String> {
    let said = |e: &dyn std::fmt::Display| format!("Cannot read {what} {}: {e}", path.display());
    let bytes = std::fs::read(path).map_err(|e| said(&e))?;
    serde_json::from_slice(&bytes).map_err(|e| said(&e))
}

/// The plan and the receipt of its run, if the run judged the faults.
fn plan_with_result(path: &Path) -> Result<(Value, Value), String> {
    let plan = json(path, "the plan")?;
    if plan["receipt"] != "step-plan-v1" {
        return Err(format!("{} is not a plan of `step plan`.", path.display()));
    }
    if plan["incomplete"] != false {
        return Err(format!(
            "The plan {} is incomplete: faults of its run have no class. {PLAN_FIRST}",
            path.display()
        ));
    }
    let run = json(&run_of(path, &plan), "the run receipt of the plan")?;
    if let Some(why) = run["void"].as_str() {
        return Err(format!(
            "The run of the plan {} was void ({why}). {PLAN_FIRST}",
            path.display()
        ));
    }
    Ok((plan, run))
}

/// The functions the plan has, in its order.
pub fn functions_of(plan: &Value) -> Vec<Named> {
    let rows = plan["functions"].as_array().map(Vec::as_slice);
    let text = |v: &Value| v.as_str().unwrap_or_default().to_owned();
    let named = rows.unwrap_or(&[]).iter().map(|row| Named {
        file: text(&row["file"]),
        function: text(&row["function"]),
    });
    named.filter(|n| n.function != OUTSIDE).collect()
}

/// The functions the person named, each once, each one of the plan's.
fn named(args: &BeginArgs, plan: &Value) -> Result<Vec<Named>, String> {
    let has = functions_of(plan);
    let mut chosen: Vec<Named> = Vec::new();
    for text in &args.functions {
        let Some((file, function)) = text
            .split_once(':')
            .filter(|(f, n)| !f.is_empty() && !n.is_empty())
        else {
            return Err(format!(
                "--function {text}: write it as <file>:<name>, the file as the plan names it."
            ));
        };
        let one = Named {
            file: file.replace('\\', "/"),
            function: function.to_owned(),
        };
        if !has.contains(&one) {
            return Err(not_in_plan(&one, &has));
        }
        if !chosen.contains(&one) {
            chosen.push(one);
        }
    }
    Ok(chosen)
}

/// The refusal for a function the plan does not have: the functions it has
/// in that file, or all of them where the file is not one of the plan's.
fn not_in_plan(missing: &Named, has: &[Named]) -> String {
    let of_file: Vec<&Named> = has.iter().filter(|n| n.file == missing.file).collect();
    let (where_, listed) = match of_file.is_empty() {
        true => ("The plan has", has.iter().collect()),
        false => ("In that file the plan has", of_file),
    };
    let lines: Vec<String> = listed
        .iter()
        .map(|n| format!("  {}:{}", n.file, n.function))
        .collect();
    format!(
        "{}:{} is not a function of the plan. {where_}:\n{}",
        missing.file,
        missing.function,
        lines.join("\n")
    )
}

/// The directory of the stage spec; refused where a stage was begun there.
fn stage_dir(args: &BeginArgs) -> Result<PathBuf, String> {
    let normal = |p: &str| {
        let parts = Path::new(p).components();
        !p.is_empty()
            && !p.contains('\\')
            && parts.into_iter().all(|c| matches!(c, Component::Normal(_)))
    };
    if !normal(&args.spec) || args.spec.contains('/') {
        return Err(format!(
            "--spec {}: not the name of one directory.",
            args.spec
        ));
    }
    if !normal(&args.specs_dir) {
        return Err(format!(
            "--specs-dir {}: not a path inside the repository, with forward slashes.",
            args.specs_dir
        ));
    }
    let dir = args.repo.join(&args.specs_dir).join(&args.spec);
    match dir.join("stage.json").is_file() {
        true => Err(format!(
            "{} already holds a stage.json: a stage was begun there. Name another spec with --spec, or remove the directory if that stage is given up.",
            dir.display()
        )),
        false => Ok(dir),
    }
}

/// The configuration, whose source root must be the repository: a stage is
/// declared with paths from the repository's root.
fn config_of(args: &BeginArgs) -> Result<RunConfig, String> {
    let config = RunConfig::read(&args.config)?;
    let place = |p: &Path| {
        p.canonicalize()
            .map_err(|e| format!("{}: {e}", p.display()))
    };
    match place(&config.source_root)? == place(&args.repo)? {
        true => Ok(config),
        false => Err(format!(
            "The source root of the run configuration, {}, is not the repository {}. A stage is declared with paths from the repository's root; make the plan with a configuration whose source root is the repository.",
            config.source_root.display(),
            args.repo.display()
        )),
    }
}

/// The text of a file, if it is the text the plan's run had.
fn unchanged(source: &SourceSet, run: &Value, file: &str) -> Result<String, String> {
    let text = source.raw(file)?;
    let now = sha256_hex_bytes(text.as_bytes());
    match run["source_files_sha256"][file].as_str() {
        Some(then) if then == now => Ok(text),
        Some(_) => Err(format!(
            "{file} has changed since the plan was made, so the plan no longer describes the code. Make a new plan with `step plan` before the stage begins."
        )),
        None => Err(format!(
            "The plan planted no fault in {file}, so it shows nothing to protect there."
        )),
    }
}

/// Every file of the tree, relative with forward slashes; version control's
/// own directory and the configuration's exclusions are left out.
fn files_of(root: &Path, exclusions: &[String]) -> Result<Vec<String>, String> {
    let (mut found, mut stack) = (Vec::new(), vec![PathBuf::new()]);
    while let Some(relative) = stack.pop() {
        let dir = root.join(&relative);
        let failed = |e: std::io::Error| format!("{}: {e}", dir.display());
        for entry in std::fs::read_dir(&dir).map_err(failed)? {
            let entry = entry.map_err(failed)?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let below = relative.join(&name);
            let slashed = below.to_string_lossy().replace('\\', "/");
            if name == ".git" || exclusions.iter().any(|e| *e == name || *e == slashed) {
                continue;
            }
            match entry.file_type().map_err(failed)?.is_dir() {
                true => stack.push(below),
                false => found.push(slashed),
            }
        }
    }
    Ok(found)
}

/// A stage adds tests and nothing else, so every test path must already
/// hold a file: where tests live inside the files of the code, a test file
/// of the stage's own has to be part of the base.
fn tests_have_a_place(config: &RunConfig) -> Result<(), String> {
    if config.test_paths.is_empty() {
        return Err("The run configuration names no test path. A stage adds tests and nothing else, and has to know where tests are kept: name the test paths in the configuration and make the plan again.".into());
    }
    let files = files_of(&config.source_root, &config.exclusions)?;
    let holds = |path: &&String| {
        let only = SourceSet {
            root: config.source_root.clone(),
            files: Vec::new(),
            test_paths: vec![(*path).clone()],
        };
        files.iter().any(|f| only.is_test_file(f))
    };
    match config.test_paths.iter().find(|p| !holds(p)) {
        None => Ok(()),
        Some(empty) => Err(format!(
            "The test path {empty} holds no file. The tests of a stage are added to test files that are part of the base: commit a test file there (where tests live inside the files of the code, a test file of its own and the line that joins it), then make the plan again."
        )),
    }
}

/// A chosen function with the faults planted in it, or `None`.
type OfFile = (Named, Option<Vec<Planted>>);

/// The faults of the functions chosen in one file. A function whose faults
/// all lie in code that is not built on this system comes back with `None`:
/// nothing is known about it here.
fn faults_of_file(
    pack: &Pack,
    file: &str,
    text: &str,
    run: &Value,
    chosen: &[Named],
) -> Result<Vec<OfFile>, String> {
    let functions = FileFunctions::read(pack, file, text)?;
    let spans = not_built_spans(pack, text, std::env::consts::OS)?;
    let mut table = planted(&functions, file, run);
    let own = |one: &Named| {
        let faults = table.remove(&one.function).unwrap_or_default();
        let elsewhere = !faults.is_empty() && faults.iter().all(|p| within(&spans, p.at));
        (one.clone(), (!elsewhere).then_some(faults))
    };
    let of_file = chosen.iter().filter(|n| n.file == file);
    Ok(of_file.map(own).collect())
}

/// Every check of `step protect begin`, in the order of what is cheapest to
/// put right. Nothing is written.
pub fn begin(args: &BeginArgs) -> Result<Begun, String> {
    let (plan, run) = plan_with_result(&args.plan)?;
    let mut chosen = named(args, &plan)?;
    let dir = stage_dir(args)?;
    let config = config_of(args)?;
    let pack = embedded(&config.language).map_err(|e| e.to_string())?;
    let source = SourceSet {
        root: config.source_root.clone(),
        files: config.files.clone(),
        test_paths: config.test_paths.clone(),
    };
    let files: BTreeSet<&str> = chosen.iter().map(|n| n.file.as_str()).collect();
    let (mut left_out, mut planted) = (Vec::new(), BTreeMap::new());
    for file in files {
        let text = unchanged(&source, &run, file)?;
        for (one, faults) in faults_of_file(pack, file, &text, &run, &chosen)? {
            match faults {
                Some(faults) => drop(planted.insert(one, faults)),
                None => left_out.push(one),
            }
        }
    }
    tests_have_a_place(&config)?;
    chosen.retain(|n| !left_out.contains(n));
    if chosen.is_empty() {
        let names: Vec<String> = left_out.iter().map(Named::to_string).collect();
        return Err(format!(
            "Every named function lies wholly in code that is not built on this system ({}). Nothing is known about them here, so there is nothing to protect.",
            names.join("; ")
        ));
    }
    Ok(Begun {
        config,
        plan,
        run,
        chosen,
        planted,
        left_out,
        dir,
    })
}

pub fn run_begin(args: BeginArgs) -> Result<u8, String> {
    let begun = begin(&args)?;
    let base = resolve(&args.repo, "HEAD")?;
    let stage = stage_of(&args, &begun, &base)?;
    let declaration = declaration_of(&args, &begun, &base)?;
    write(&begun.dir, &declaration, &stage)?;
    let branch = declaration["branch"].as_str().unwrap_or_default();
    let receipt = run_of(&args.plan, &begun.plan);
    println!("{}", says(&stage, branch, &begun.run, &receipt));
    println!(
        "\nWritten: step.json and stage.json in {}",
        begun.dir.display()
    );
    Ok(0)
}
