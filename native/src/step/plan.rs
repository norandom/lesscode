//! `step plan`: where a small step would help most. Per function of the
//! configured files: planted faults the tests notice and do not notice,
//! state writes, size, and for functions with no noticed fault what they
//! reach out to. Faults run in copies; the source tree is only read.

use super::functions::{FileFunctions, Tally, OUTSIDE};
use crate::adapter_traits::FrameSpec;
use crate::adapters::driver::analysis::{analyse, outside_frame};
use crate::adapters::driver::cli::emit_json;
use crate::adapters::driver::config::RunConfig;
use crate::adapters::driver::pack::Pack;
use crate::adapters::driver::registry::embedded;
use crate::adapters::driver::run_command::{run_run, RunArgs};
use crate::adapters::driver::source::SourceSet;
use crate::adapters::driver::Kind;
use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;

/// Arguments of `step plan`.
#[derive(clap::Args, Clone, Debug)]
pub struct PlanArgs {
    /// A run configuration as for `deconvolute run`: files, test paths, commands.
    #[arg(long)]
    pub config: PathBuf,
    /// Where the working copies and reusable outcomes are kept; outside the source root.
    #[arg(long)]
    pub work: PathBuf,
    /// A frame specification; state writes it does not allow are counted.
    #[arg(long)]
    pub frame_spec: Option<PathBuf>,
    /// Copies to run at once; the configuration's value when absent.
    #[arg(long)]
    pub workers: Option<usize>,
    /// Where to write the plan.
    #[arg(long)]
    pub out: PathBuf,
}

/// What a function with no noticed fault reaches out to.
#[derive(Serialize)]
struct ReachesOut {
    calls: Vec<String>,
    /// `line N: text` of each state write.
    state_writes: Vec<String>,
}

#[derive(Serialize)]
struct Row {
    file: String,
    function: String,
    #[serde(flatten)]
    faults: Tally,
    tokens: usize,
    state_writes: usize,
    /// State writes the frame specification does not allow; absent without one.
    #[serde(skip_serializing_if = "Option::is_none")]
    outside_frame: Option<usize>,
    /// Faults were planted in it and the tests noticed none.
    unprotected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    reaches_out: Option<ReachesOut>,
}

#[derive(Serialize)]
struct Plan {
    receipt: &'static str,
    run: String,
    incomplete: bool,
    /// Ranked by unnoticed faults, most first.
    functions: Vec<Row>,
}

/// The faults of a run receipt for one file, as position and class.
fn faults_of<'a>(run: &'a Value, file: &str) -> Vec<((usize, usize), &'a str)> {
    let number = |v: &Value| v.as_u64().unwrap_or(0) as usize;
    let mutants = run["mutants"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    let of_file = mutants.iter().filter(|m| m["site"]["file"] == file);
    of_file
        .map(|m| {
            let at = (
                number(&m["site"]["start_line"]),
                number(&m["site"]["start_column"]),
            );
            (at, m["class"].as_str().unwrap_or(""))
        })
        .collect()
}

fn rows_of(
    pack: &Pack,
    file: &str,
    text: &str,
    run: &Value,
    frame: Option<&FrameSpec>,
) -> Result<Vec<Row>, String> {
    let functions = FileFunctions::read(pack, file, text)?;
    let analysis = analyse(pack, file, text)?;
    let writes: Vec<_> = analysis
        .sites
        .iter()
        .filter(|s| s.kind == Kind::StateWrite)
        .map(|s| (functions.owner((s.start_line, s.start_column)), s))
        .collect();
    let table = functions.tally(faults_of(run, file));
    let mut sizes = functions.sizes();
    if table.contains_key(OUTSIDE) {
        sizes.insert(OUTSIDE, 0);
    }
    let row = |(name, tokens): (&str, usize)| {
        let faults = table.get(name).copied().unwrap_or_default();
        let own: Vec<_> = writes.iter().filter(|(owner, _)| *owner == name).collect();
        let labels: Vec<String> = own
            .iter()
            .map(|(_, s)| format!("{file}:{}", s.start_line))
            .collect();
        let planted = faults.noticed + faults.unnoticed + faults.other;
        let unprotected = faults.noticed == 0 && planted > 0;
        let line = |s: &str| s.lines().next().unwrap_or("").trim().to_owned();
        let reaches_out = unprotected.then(|| ReachesOut {
            calls: functions
                .calls_in(name)
                .into_iter()
                .map(str::to_owned)
                .collect(),
            state_writes: own
                .iter()
                .map(|(_, s)| format!("line {}: {}", s.start_line, line(&s.text)))
                .collect(),
        });
        Row {
            file: file.to_owned(),
            function: name.to_owned(),
            faults,
            tokens,
            state_writes: own.len(),
            outside_frame: frame.map(|spec| outside_frame(&labels, spec).len()),
            unprotected,
            reaches_out,
        }
    };
    Ok(sizes.into_iter().map(row).collect())
}

fn print(plan: &Plan) {
    println!(
        "{:<44} {:>8} {:>10} {:>7} {:>7}",
        "function", "noticed", "unnoticed", "tokens", "writes"
    );
    for row in &plan.functions {
        let planted = row.faults.noticed + row.faults.unnoticed + row.faults.other;
        let mark = match (row.unprotected, planted) {
            (true, _) => "  no fault noticed",
            (false, 0) => "  no fault planted",
            _ => "",
        };
        let name = format!("{}: {}", row.file, row.function);
        println!(
            "{:<44} {:>8} {:>10} {:>7} {:>7}{mark}",
            name, row.faults.noticed, row.faults.unnoticed, row.tokens, row.state_writes
        );
    }
}

/// Exit 0 when the plan is complete, 2 when faults of the run have no
/// class; a void run or bad input is an error (exit 3).
pub fn run_plan(args: PlanArgs) -> Result<u8, String> {
    let config = RunConfig::read(&args.config)?;
    let pack = embedded(&config.language).map_err(|e| e.to_string())?;
    let frame: Option<FrameSpec> = match &args.frame_spec {
        Some(path) => {
            let bytes = std::fs::read(path)
                .map_err(|e| format!("Cannot read frame spec {}: {e}", path.display()))?;
            Some(serde_json::from_slice(&bytes).map_err(|e| format!("Invalid frame spec: {e}"))?)
        }
        None => None,
    };
    let run_path = args.work.join("plan-run.json");
    let code = run_run(RunArgs {
        config: args.config.clone(),
        work: args.work.clone(),
        workers: args.workers,
        annotations: None,
        no_reuse: false,
        only: Vec::new(),
        out: run_path.clone(),
    })?;
    let bytes = std::fs::read(&run_path).map_err(|e| format!("{}: {e}", run_path.display()))?;
    let run: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let source = SourceSet {
        root: config.source_root.clone(),
        files: config.files.clone(),
        test_paths: config.test_paths.clone(),
    };
    let mut functions = Vec::new();
    for file in &config.files {
        let text = source.raw(file)?;
        functions.extend(rows_of(pack, file, &text, &run, frame.as_ref())?);
    }
    functions.sort_by(|a, b| {
        let key = |r: &Row| (std::cmp::Reverse(r.faults.unnoticed), r.faults.noticed);
        key(a).cmp(&key(b))
    });
    let plan = Plan {
        receipt: "step-plan-v1",
        run: run_path.to_string_lossy().replace('\\', "/"),
        incomplete: code != 0,
        functions,
    };
    print(&plan);
    emit_json(&plan, Some(&args.out))?;
    Ok(code)
}
