//! A bounded comparison of versions of one piece of behaviour.
//!
//! A target file names integer symbols with their ranges, and per version
//! (an arm) an SMT expression over the symbols and a command that runs the
//! real code. The solver is asked whether two expressions can differ inside
//! the ranges. Every assignment of the ranges is then run through each arm's
//! command, and a solver witness is run again on its own. The expressions
//! and the commands come from the target file; nothing here knows a target.
//!
//! "No difference" means none inside the ranges. It is not a proof.

use super::bound_target::{integer, Arm, Symbol, Target};
use super::process;
use super::solver::{bindings, Solver, Term};
use crate::specimen_runner::sha256_hex_bytes;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Serialize)]
struct Witness {
    id: usize,
    values: BTreeMap<String, i64>,
    /// Run again on its own, the two arms differed as in the full run.
    reproduced: bool,
}

#[derive(Serialize)]
struct Comparison {
    left: String,
    right: String,
    solver: &'static str,
    /// Cases on which the two expressions differ.
    modeled_differences: usize,
    /// Cases on which the two commands returned different values.
    report_differences: usize,
    difference_ids: Vec<usize>,
    /// Cases where exactly one of the two kinds of difference shows.
    unexplained_ids: Vec<usize>,
    witness: Option<Witness>,
    /// `same`, `different` or `inconsistent`.
    verdict: &'static str,
}

#[derive(Serialize)]
struct Receipt<'a> {
    receipt: &'static str,
    target_sha256: String,
    solver: &'a Solver,
    symbols: &'a [Symbol],
    cases: usize,
    comparisons: Vec<Comparison>,
}

fn save(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    fs::write(path, bytes).map_err(|e| format!("Cannot write {}: {e}", path.display()))
}

fn pin(path: &Path, sha256: &str) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    match sha256_hex_bytes(&bytes) == sha256 {
        true => Ok(()),
        false => Err(format!("{} does not have SHA-256 {sha256}", path.display())),
    }
}

/// One run of the target: where its files go and how its commands run.
struct Run<'a> {
    target: &'a Target,
    out: &'a Path,
}

impl Run<'_> {
    fn loaded(&self, arm: &Arm, output: &Value) -> Result<(), String> {
        for (key, expected) in &arm.identities {
            let found = &output[key];
            let same_file = |wanted: &PathBuf| {
                let path = found["path"].as_str().map(Path::new);
                path.and_then(|p| p.canonicalize().ok()) == wanted.canonicalize().ok()
            };
            let right_file = expected.path.as_ref().is_none_or(same_file);
            if found["sha256"] != json!(expected.sha256) || !right_file {
                return Err(format!(
                    "{}: {key} is not the expected file: {found}",
                    arm.label
                ));
            }
        }
        Ok(())
    }

    /// Run the arm's command on the cases written as `<batch>-inputs.json`
    /// and give the value it reports per case, in the order of `ids`.
    fn reports(&self, arm: &Arm, batch: &str, ids: &[usize]) -> Result<Vec<Value>, String> {
        let name = format!("{batch}-{}", arm.label);
        let place = |file: String| self.out.join(file).to_string_lossy().into_owned();
        let (inputs, output) = (
            place(format!("{batch}-inputs.json")),
            place(format!("{name}.json")),
        );
        let fill = |word: &String| {
            let word = word.replace("{inputs}", &inputs);
            let word = word.replace("{output}", &output);
            word.replace("{out}", &self.out.to_string_lossy())
        };
        let argv: Vec<String> = arm.command.iter().map(fill).collect();
        let limit = Duration::from_secs(self.target.timeout_seconds);
        let done = process::run(&argv, self.out, &BTreeMap::new(), limit);
        let command = json!({"argv": argv, "exit": done.exit_code, "timed_out": done.timed_out,
            "seconds": done.seconds, "launch_error": done.launch_error, "output": done.output});
        save(&self.out.join(format!("{name}.command.json")), &command)?;
        if !done.passed() {
            return Err(format!("the command of {name} failed: {}", done.output));
        }
        let bytes = fs::read(&output).map_err(|e| format!("Cannot read {output}: {e}"))?;
        let mut result: Value =
            serde_json::from_slice(&bytes).map_err(|e| format!("{output} is not JSON: {e}"))?;
        self.loaded(arm, &result)?;
        let reports = result["reports"].take();
        let reports = reports.as_array().map(Vec::as_slice).unwrap_or_default();
        let told: Vec<Value> = reports.iter().map(|report| report["id"].clone()).collect();
        if told != ids.iter().map(|id| json!(id)).collect::<Vec<_>>() {
            return Err(format!(
                "{name} did not report exactly the cases it was given"
            ));
        }
        Ok(reports
            .iter()
            .map(|report| report["value"].clone())
            .collect())
    }

    fn batch(&self, batch: &str, cases: &[(usize, &Vec<i64>)]) -> Result<Vec<usize>, String> {
        let input = |(id, case): &(usize, &Vec<i64>)| self.target.input(*id, case);
        let inputs: Vec<Value> = cases.iter().map(input).collect();
        save(&self.out.join(format!("{batch}-inputs.json")), &inputs)?;
        Ok(cases.iter().map(|(id, _)| *id).collect())
    }

    /// Ask the solver for a case on which the pair differs and run it again.
    fn witness(
        &self,
        solver: &Solver,
        pair: (&str, &str),
        cases: &[Vec<i64>],
        census: &BTreeMap<&str, Vec<Value>>,
    ) -> Result<Witness, String> {
        let (left, right) = pair;
        let names = self.target.names();
        let ask = format!("(get-value ({}))\n", names.join(" "));
        let label = format!("witness-{left}-{right}");
        let answer = solver.ask(&label, &(self.target.query(left, right) + &ask))?;
        let [Term::Word(_), values] = answer.as_slice() else {
            return Err(format!("the solver gave no values for {label}"));
        };
        let values = bindings(values, &names)?;
        let value = |name: &String| integer(&values[name]);
        let case: Option<Vec<i64>> = names.iter().map(value).collect();
        let id = case.and_then(|case| cases.iter().position(|c| *c == case));
        let id = id.ok_or_else(|| format!("the values for {label} lie outside the ranges"))?;
        let ids = self.batch(&label, &[(id, &cases[id])])?;
        let again = |arm: &str| -> Result<Value, String> {
            let fresh = self.reports(self.target.arm(arm)?, &label, &ids)?.remove(0);
            match fresh == census[arm][id] {
                true => Ok(fresh),
                false => Err(format!(
                    "{arm} gave another value for case {id} when run again"
                )),
            }
        };
        Ok(Witness {
            id,
            values: names.into_iter().zip(cases[id].iter().copied()).collect(),
            reproduced: again(left)? != again(right)?,
        })
    }

    fn compare(
        &self,
        solver: &Solver,
        pair: (&str, &str),
        cases: &[Vec<i64>],
        modeled: &[BTreeMap<String, String>],
        census: &BTreeMap<&str, Vec<Value>>,
    ) -> Result<Comparison, String> {
        let (left, right) = pair;
        let label = format!("compare-{left}-{right}");
        let sat = solver.satisfiable(&label, &self.target.query(left, right))?;
        let ids = 0..cases.len();
        let modeled_differs = |id: &usize| {
            modeled[*id][&format!("arm_{left}")] != modeled[*id][&format!("arm_{right}")]
        };
        let reports_differ = |id: &usize| census[left][*id] != census[right][*id];
        let modeled_differences = ids.clone().filter(modeled_differs).count();
        let difference_ids: Vec<usize> = ids.clone().filter(reports_differ).collect();
        let unexplained_ids: Vec<usize> = ids
            .filter(|id| modeled_differs(id) != reports_differ(id))
            .collect();
        if sat != (modeled_differences > 0) {
            return Err(format!("the solver contradicts itself on {label}"));
        }
        let witness = match sat {
            true => Some(self.witness(solver, pair, cases, census)?),
            false => None,
        };
        let reproduced = witness.as_ref().is_none_or(|w| w.reproduced);
        let verdict = match (unexplained_ids.is_empty() && reproduced, sat) {
            (false, _) => "inconsistent",
            (true, true) => "different",
            (true, false) => "same",
        };
        Ok(Comparison {
            left: left.to_owned(),
            right: right.to_owned(),
            solver: if sat { "sat" } else { "unsat" },
            modeled_differences,
            report_differences: difference_ids.len(),
            difference_ids,
            unexplained_ids,
            witness,
            verdict,
        })
    }
}

/// `deconvolute bound-check`: exit 0 when no compared pair differs inside
/// the ranges, 1 when a pair differs and the difference was reproduced, 2
/// when the expressions and the commands disagree about where they differ.
pub fn run_bound_check(target: &Path, out: &Path) -> Result<u8, String> {
    let (target, target_sha256) = Target::read(target)?;
    fs::create_dir(out).map_err(|e| format!("{} must be a new directory: {e}", out.display()))?;
    let out = &out.canonicalize().map_err(|e| e.to_string())?;
    for file in &target.pins {
        pin(&file.path, &file.sha256)?;
    }
    let solver = Solver::pinned(&target.solver.path, &target.solver.sha256, out)?;
    let run = Run {
        target: &target,
        out,
    };
    let cases = target.cases();
    let modeled = target.modeled(&solver, &cases)?;
    let all: Vec<(usize, &Vec<i64>)> = cases.iter().enumerate().collect();
    let ids = run.batch("all", &all)?;
    let mut census = BTreeMap::new();
    for arm in &target.arms {
        census.insert(arm.label.as_str(), run.reports(arm, "all", &ids)?);
    }
    let mut comparisons = Vec::new();
    for (left, right) in &target.compare {
        comparisons.push(run.compare(&solver, (left, right), &cases, &modeled, &census)?);
    }
    let exit = comparisons.iter().map(|c| match c.verdict {
        "same" => 0,
        "different" => 1,
        _ => 2,
    });
    let exit = exit.max().unwrap_or(0);
    let receipt = Receipt {
        receipt: "bound-check-v1",
        target_sha256,
        solver: &solver,
        symbols: &target.symbols,
        cases: cases.len(),
        comparisons,
    };
    save(&out.join("result.json"), &receipt)?;
    super::cli::emit_json(&receipt, None)?;
    Ok(exit)
}
