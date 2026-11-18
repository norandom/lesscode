mod guard;

use guard::{Case, Guard, Result, SYMBOLS};
use lesscode_core::{adapters::driver::process, receipt::sha256_hex};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    time::Duration,
};

const SUBJECT: &str = "src/facdrone/shells/lifecycle_projection.py";
const KERNEL: &str = "packages/facdrone-kernel/facdrone_kernel/domain/signal_lifecycle.py";
const BASE: &str = "adb6075c061d48058a56b69655d0486bc6ea1f6a98e927774e0c3e596d94cea7";
const CLEAN: &str = "4eac76fa9b31d9f3953e0a1de8a79b49139ba64451dd67dafae6fd23973615d4";
const DOMAIN: &str = "c8d7311f4495cf22b49f48f5b3d2078bb69d253a49f25d6f755b4eb41181f0c9";
const Z3: &str = "2bffeee10ba02a510d07be9c3fdf9c22f10ccc9724dc84201c5a40196f1a1e8f";
const PYTHON: &str = "602f83da0874772f7a9480e15029b8ff57e752f9e07f398a743df042a6b3ca09";
const INTERPRETER: &str = "932bef4080ec12fff46a85debc0efe91403add7249423b5c2b60a8eb1c30bb99";
const RUNTIME: &str = "da67ce13731766f5a9ed0295adb606834d2d64c1363064ba8b7f777ad19fb647";
const FILTER: &str = r#"if (r.get("baseline_digest"), r.get("rules_revision")) != basis:"#;
const PLANTED: &str = r#"if (r.get("baseline_digest"), newest.get("rules_revision")) != basis:"#;
const ADAPTER: &str = include_str!("../invoke.py");
const RADIX: u8 = 3;
const WINDOW: [&str; 2] = ["2026-09-05", "2026-09-08"];
const MONITOR: [&str; 2] = ["traded_horizon_ic", "survival"];
const NOW: &str = "2026-09-17T12:00:00+00:00";

fn require(ok: bool, reason: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(reason.to_owned().into())
    }
}

fn hash(path: &Path) -> Result<String> {
    Ok(sha256_hex(&fs::read(path)?))
}

fn pin(path: &Path, expected: &str) -> Result<()> {
    require(
        hash(path)? == expected,
        &format!("SHA-256 mismatch at {}", path.display()),
    )
}

fn save(path: &Path, value: &impl serde::Serialize) -> Result<()> {
    Ok(fs::write(path, serde_json::to_vec_pretty(value)?)?)
}

fn case_id(c: Case) -> usize {
    c.into_iter()
        .fold(0, |id, digit| id * usize::from(RADIX) + usize::from(digit))
}

fn fixture(c: Case) -> Value {
    let row = |index: usize, digest, revision| {
        json!({
            "window_id": WINDOW[index], "monitor": MONITOR[index],
            "disposition": "within", "value": 0.03, "boundary": 0.025, "comparator": "<",
            "units": "IC", "known_at": format!("{}T12:00:00+00:00", WINDOW[index]), "payload": {},
            "baseline_digest": format!("d{digest}"), "rules_revision": format!("r{revision}")
        })
    };
    json!({"id": case_id(c), "standing": {"signal": "cot", "version": 1}, "days": 14, "now": NOW,
        "rows": [row(0, c[0], c[1]), row(1, c[2], c[3])]})
}

fn query(base: &Guard, other: &Guard) -> String {
    let mut s = String::from("(set-logic QF_LIA)\n");
    for name in SYMBOLS {
        s.push_str(&format!(
            "(declare-const {name} Int)\n(assert (and (<= 0 {name}) (< {name} {RADIX})))\n"
        ));
    }
    s.push_str(&format!(
        "(assert (distinct {} {}))\n(check-sat)\n",
        base.expression(),
        other.expression()
    ));
    s
}

fn run(out: &Path, label: &str, argv: Vec<String>, seconds: u64) -> Result<String> {
    let cwd = env::current_dir()?;
    let result = process::run(&argv, &cwd, &BTreeMap::new(), Duration::from_secs(seconds));
    fs::write(out.join(format!("{label}.txt")), &result.output)?;
    save(
        &out.join(format!("{label}.command.json")),
        &json!({
            "argv":argv, "cwd":cwd, "exit":result.exit_code, "timeout":result.timed_out,
            "elapsed_seconds":result.seconds, "launch_error":result.launch_error
        }),
    )?;
    require(
        result.passed(),
        &format!(
            "{label} failed: {} {:?}",
            result.output, result.launch_error
        ),
    )?;
    Ok(result.output.trim().to_owned())
}

fn arg(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn solve(out: &Path, z3: &Path, label: &str, smt: &str) -> Result<String> {
    let file = out.join(format!("{label}.smt2"));
    fs::write(&file, smt)?;
    run(out, label, vec![arg(z3), "-smt2".into(), arg(&file)], 10)
}

// Accept exactly the requested four nonnegative integer bindings, once each.
fn witness(output: &str) -> Result<Case> {
    let (verdict, bindings) = output.split_once('\n').ok_or("missing Z3 bindings")?;
    require(verdict.trim() == "sat", "model response lacks SAT")?;
    let bindings = bindings
        .trim()
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
        .ok_or("invalid binding list")?;
    let mut rest = bindings.trim();
    let mut values = [None; 4];
    while !rest.is_empty() {
        let (pair, tail) = rest
            .strip_prefix('(')
            .and_then(|s| s.split_once(')'))
            .ok_or("invalid binding")?;
        let fields: Vec<_> = pair.split_whitespace().collect();
        require(fields.len() == 2, "invalid binding arity")?;
        let i = SYMBOLS
            .iter()
            .position(|&name| name == fields[0])
            .ok_or("unexpected binding")?;
        let digit = fields[1].parse::<u8>()?;
        require(
            digit < RADIX && values[i].replace(digit).is_none(),
            "invalid or duplicate digit",
        )?;
        rest = tail.trim();
    }
    require(values.iter().all(Option::is_some), "incomplete model")?;
    Ok(values.map(Option::unwrap))
}

struct Arm {
    label: &'static str,
    root: PathBuf,
    source: PathBuf,
    digest: String,
    guard: Guard,
}

#[derive(Deserialize)]
struct Identity {
    path: PathBuf,
    sha256: String,
}
#[derive(Deserialize)]
struct Observation {
    id: usize,
    value: Value,
}
#[derive(Deserialize)]
struct Receipt {
    source: Identity,
    kernel: Identity,
    interpreter: Identity,
    runtime: Identity,
    reports: Vec<Observation>,
}

fn invoke(
    out: &Path,
    python: &Path,
    arm: &Arm,
    batch: &str,
    inputs: &[Case],
) -> Result<Vec<Value>> {
    let label = format!("{batch}-{}", arm.label);
    let file = out.join(format!("{label}.json"));
    let input = out.join(format!("{batch}-inputs.json"));
    run(
        out,
        &label,
        vec![
            arg(python),
            "-I".into(),
            "-S".into(),
            "-B".into(),
            "-X".into(),
            format!("pycache_prefix={}", out.join("unused-pycache").display()),
            arg(&out.join("invoke.py")),
            arg(&arm.root),
            arg(&arm.source),
            arg(&input),
            arg(&file),
        ],
        60,
    )?;
    let receipt: Receipt = serde_json::from_slice(&fs::read(file)?)?;
    for (loaded, expected, digest) in [
        (&receipt.source, arm.source.clone(), arm.digest.as_str()),
        (&receipt.kernel, arm.root.join(KERNEL), DOMAIN),
    ] {
        require(
            loaded.path.canonicalize()? == expected.canonicalize()? && loaded.sha256 == digest,
            "specimen import identity mismatch",
        )?;
    }
    require(
        receipt.interpreter.sha256 == INTERPRETER && receipt.runtime.sha256 == RUNTIME,
        "CPython runtime mismatch",
    )?;
    require(
        receipt.reports.len() == inputs.len(),
        "missing or extra reports",
    )?;
    let mut values = Vec::new();
    for (obs, &case) in receipt.reports.into_iter().zip(inputs) {
        require(obs.id == case_id(case), "unexpected case ID")?;
        let rows = obs
            .value
            .get("readings")
            .and_then(Value::as_array)
            .ok_or("invalid readings")?;
        let windows: Vec<_> = rows
            .iter()
            .map(|r| r.get("window").and_then(Value::as_str))
            .collect();
        let expected = if arm.guard.keep(case) {
            vec![Some(WINDOW[0]), Some(WINDOW[1])]
        } else {
            vec![Some(WINDOW[1])]
        };
        require(
            windows == expected,
            &format!("model disagrees with {label} on case {}", obs.id),
        )?;
        values.push(obs.value);
    }
    Ok(values)
}

fn experiment(repo: &Path, out: &Path) -> Result<()> {
    let base = repo.join(".runs/e2-facdrone/source");
    let clean = repo.join(".runs/e5-state-boundary/cand-e5");
    let python = repo.join(".runs/e2-facdrone/venv/Scripts/python.exe");
    let z3 = repo.join(".runs/e1c-facdrone/extras/bin/z3.exe");
    for (file, expected) in [
        (base.join(SUBJECT), BASE),
        (clean.join(SUBJECT), CLEAN),
        (base.join(KERNEL), DOMAIN),
        (clean.join(KERNEL), DOMAIN),
        (z3.clone(), Z3),
        (python.clone(), PYTHON),
    ] {
        pin(&file, expected)?;
    }
    let clean_src = fs::read_to_string(clean.join(SUBJECT))?;
    require(
        clean_src.matches(FILTER).count() == 1,
        "fault site must be unique",
    )?;
    let unsupported = clean_src.replace(FILTER, &FILTER.replace("rules_revision", "other_field"));
    let rejection = guard::lower(&unsupported, false)
        .err()
        .ok_or("unsupported guard was admitted")?;
    require(
        rejection.to_string().contains("unsupported row access"),
        "unexpected cause of rejection",
    )?;
    fs::write(out.join("unsupported.txt"), rejection.to_string())?;
    let planted = out.join("planted.py");
    fs::write(&planted, clean_src.replace(FILTER, PLANTED))?;
    fs::write(out.join("invoke.py"), ADAPTER)?;
    let mut arms = Vec::new();
    for (label, root, source) in [
        ("baseline", &base, base.join(SUBJECT)),
        ("candidate", &clean, clean.join(SUBJECT)),
        ("planted", &clean, planted),
    ] {
        let guard = guard::lower(&fs::read_to_string(&source)?, label == "baseline")?;
        let digest = hash(&source)?;
        save(
            &out.join(format!("guard-{label}.json")),
            &json!({
                "source":source, "sha256":digest, "symbols":SYMBOLS, "guard":guard, "smt":guard.expression()
            }),
        )?;
        arms.push(Arm {
            label,
            root: root.clone(),
            source,
            digest,
            guard,
        });
    }

    let z3_version = run(out, "z3-version", vec![arg(&z3), "-version".into()], 10)?;
    let python_version = run(out, "python-version", vec![arg(&python), "-VV".into()], 60)?;
    let clean_smt = query(&arms[0].guard, &arms[1].guard);
    let fault_smt = query(&arms[0].guard, &arms[2].guard);
    let clean_verdict = solve(out, &z3, "clean", &clean_smt)?;
    let fault_verdict = solve(out, &z3, "fault", &fault_smt)?;
    require(
        clean_verdict == "unsat" && fault_verdict == "sat",
        "unexpected solver result",
    )?;
    let model_query = format!("{fault_smt}(get-value ({}))\n", SYMBOLS.join(" "));
    let model = witness(&solve(out, &z3, "model", &model_query)?)?;
    require(
        arms[0].guard.keep(model) != arms[2].guard.keep(model),
        "model does not witness the query",
    )?;

    let mut cases = Vec::new();
    for a in 0..RADIX {
        for b in 0..RADIX {
            for c in 0..RADIX {
                for d in 0..RADIX {
                    cases.push([a, b, c, d]);
                }
            }
        }
    }
    let execute = |label: &str, cs: &[Case]| -> Result<Vec<Vec<Value>>> {
        save(
            &out.join(format!("{label}-inputs.json")),
            &cs.iter().copied().map(fixture).collect::<Vec<_>>(),
        )?;
        arms.iter()
            .map(|arm| invoke(out, &python, arm, label, cs))
            .collect()
    };
    let batch = execute("all", &cases)?;
    let clean_differences = batch[0]
        .iter()
        .zip(&batch[1])
        .filter(|(a, b)| a != b)
        .count();
    require(clean_differences == 0, "clean full reports differ")?;
    let mut differences = Vec::new();
    for &case in &cases {
        let id = case_id(case);
        let different = batch[0][id] != batch[2][id];
        require(
            different == (case[0] == case[2] && case[1] != case[3]),
            "unexpected fault population",
        )?;
        if different {
            differences.push(id);
        }
    }
    let replay = execute("fresh", &[model])?;
    for (a, r) in batch.iter().zip(&replay) {
        require(
            a[case_id(model)] == r[0],
            "fresh replay disagrees with census",
        )?;
    }
    require(
        replay[0] == replay[1] && replay[0] != replay[2],
        "witness did not reproduce",
    )?;
    save(
        &out.join("result.json"),
        &json!({
            "clean_guard":clean_verdict, "planted_guard":fault_verdict, "whole_reports_per_arm":cases.len(),
            "clean_report_differences":clean_differences, "planted_difference_ids":differences, "witness":model,
            "witness_id":case_id(model), "fresh_replays":replay.len(), "symbol_order":SYMBOLS,
            "z3":z3_version, "python":python_version, "z3_sha256":Z3, "launcher_sha256":PYTHON,
            "adapter_sha256":sha256_hex(ADAPTER.as_bytes()), "adapter_physical_lines":ADAPTER.lines().count(),
            "controller_sha256":hash(&env::current_exe()?)?
        }),
    )
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    require(
        args.len() == 2,
        "usage: e5-bound LESSCODE_REPO NEW_EVIDENCE_DIRECTORY",
    )?;
    let repo = Path::new(&args[0]).canonicalize()?;
    let out = Path::new(&args[1]);
    fs::create_dir(out)?;
    experiment(&repo, &out.canonicalize()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_is_bound_by_name_and_checked_against_domain() {
        let bindings = "((new_revision 2) (old_revision 0) (old_digest 1) (new_digest 1))";
        assert_eq!(
            witness(&format!("sat\r\n{bindings}")).unwrap(),
            [1, 0, 1, 2]
        );
        for invalid in [
            bindings.replace("new_digest", "old_digest"),
            bindings.replace("old_revision 0", "old_revision 3"),
            bindings.replace("old_revision 0", "old_revision -1"),
            bindings.replace("(new_revision 2)", ""),
            format!("{bindings} (error \"extra output\")"),
        ] {
            assert!(witness(&format!("sat\n{invalid}")).is_err(), "{invalid}");
        }
        assert!(witness(&format!("unknown\n{bindings}")).is_err());
    }
}
