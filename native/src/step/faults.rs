//! Fault runs of a step: the driver's run on one copy, limited to named
//! functions, with its result read back.

use super::declaration::Declaration;
use crate::adapters::driver::run_command::{run_run, RunArgs};
use crate::adapters::driver::runner::TESTS_FAIL;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

/// One planted fault and what the tests made of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fault {
    pub number: usize,
    pub file: String,
    pub at: (usize, usize),
    pub operator: String,
    pub class: String,
}

impl Fault {
    pub fn label(&self) -> String {
        format!("#{} line {} {}", self.number, self.at.0, self.operator)
    }
}

/// How a run ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ended {
    /// Every fault has a class.
    Complete,
    /// Some faults could not be classed.
    Incomplete,
    /// The project's tests fail without any fault.
    TestsFail,
    /// The run says nothing, for the reason given.
    Void(String),
}

pub struct FaultRun {
    pub ended: Ended,
    pub faults: Vec<Fault>,
    /// The run's receipt, relative to the work directory.
    pub receipt: String,
}

/// What to run: a copy, the functions per file, and optionally single faults.
pub struct Request<'a> {
    pub name: &'a str,
    pub copy: &'a Path,
    pub functions: &'a BTreeMap<String, Vec<String>>,
    pub only: Vec<usize>,
    pub workers: Option<usize>,
}

fn fault(m: &Value) -> Fault {
    let number = |v: &Value| v.as_u64().unwrap_or(0) as usize;
    let text = |v: &Value| v.as_str().unwrap_or("").to_owned();
    Fault {
        number: number(&m["number"]),
        file: text(&m["site"]["file"]),
        at: (
            number(&m["site"]["start_line"]),
            number(&m["site"]["start_column"]),
        ),
        operator: text(&m["operator"]),
        class: text(&m["class"]),
    }
}

/// Run the faults of `request` in copies under `work/<name>-run`. Bad input
/// is an error; a void run is a result.
pub fn run(declaration: &Declaration, work: &Path, request: Request) -> Result<FaultRun, String> {
    let config = declaration.run_config_json(request.copy, request.functions)?;
    let config_path = work.join(format!("{}-config.json", request.name));
    let receipt = format!("{}-run.json", request.name);
    let out = work.join(&receipt);
    let io = |e: std::io::Error| format!("{}: {e}", work.display());
    std::fs::create_dir_all(work).map_err(io)?;
    std::fs::write(&config_path, config.to_string()).map_err(io)?;
    if out.exists() {
        std::fs::remove_file(&out).map_err(io)?;
    }
    let outcome = run_run(RunArgs {
        config: config_path,
        work: work.join(format!("{}-run", request.name)),
        workers: request.workers,
        annotations: None,
        no_reuse: false,
        only: request.only,
        out: out.clone(),
    });
    // A void run has written its receipt before it reports the error.
    let Ok(bytes) = std::fs::read(&out) else {
        return Err(outcome
            .err()
            .unwrap_or_else(|| "the run wrote no receipt".into()));
    };
    let run: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let ended = match (run["void"].as_str(), outcome) {
        (Some(why), _) if why.contains(TESTS_FAIL) => Ended::TestsFail,
        (Some(why), _) => Ended::Void(why.to_owned()),
        (None, Ok(0)) => Ended::Complete,
        (None, Ok(_)) => Ended::Incomplete,
        (None, Err(why)) => return Err(why),
    };
    let mutants = run["mutants"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    Ok(FaultRun {
        ended,
        faults: mutants.iter().map(fault).collect(),
        receipt,
    })
}
