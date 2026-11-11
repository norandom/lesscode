//! `step plan` through the binary (gradual-refactoring-steps, task 3.3).

use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::process::Command;

const APP: &str = "def step(state, a):\n    if a > 3:\n        raise ValueError(\"big\")\n    state.total = a\n    return a\n\n\ndef helper(state, b):\n    if b > 5:\n        log(b)\n    state.count = b\n    return b\n";

/// The test stand-in, built beside this test binary.
fn toy() -> String {
    let test_binary = std::env::current_exe().unwrap();
    let profile = test_binary.parent().unwrap().parent().unwrap();
    let name = format!("toy_check{}", std::env::consts::EXE_SUFFIX);
    let path = profile.join("examples").join(name);
    assert!(path.is_file(), "not run: {} was not built", path.display());
    path.to_string_lossy().into_owned()
}

fn files_of(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut found = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in fs::read_dir(&d).unwrap() {
            let path = entry.unwrap().path();
            match path.is_dir() {
                true => stack.push(path),
                false => found.push((
                    path.to_string_lossy().into_owned(),
                    fs::read(&path).unwrap(),
                )),
            }
        }
    }
    found.sort();
    found
}

#[test]
fn a_plan_ranks_functions_and_shows_what_unprotected_ones_reach() {
    let dir = std::env::temp_dir().join(format!("lc-test-plan-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("project/src")).unwrap();
    fs::write(dir.join("project/src/app.py"), APP).unwrap();
    // The stand-in test only looks at the first function.
    let config = json!({
        "language": "python", "source_root": "project", "files": ["src/app.py"],
        "test": [toy(), "--file", "src/app.py", "--print", "TOY RAN",
                 "--expect", "if a > 3:", "--expect", "state.total = a"],
        "timeout_floor_s": 10, "timeout_ceiling_s": 60,
    });
    fs::write(dir.join("run.json"), config.to_string()).unwrap();
    fs::write(
        dir.join("frame.json"),
        json!({"allowed_write_sites": ["app.py:4"], "permitted_shell_locations": [], "failure_invariants": []}).to_string(),
    )
    .unwrap();
    let before = files_of(&dir.join("project"));

    let output = Command::new(env!("CARGO_BIN_EXE_lesscode-core"))
        .args(["step", "plan", "--config"])
        .arg(dir.join("run.json"))
        .arg("--work")
        .arg(dir.join("work"))
        .arg("--frame-spec")
        .arg(dir.join("frame.json"))
        .arg("--out")
        .arg(dir.join("plan.json"))
        .output()
        .expect("run lesscode-core binary");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(0), "{stderr}");
    let plan: Value = serde_json::from_slice(&fs::read(dir.join("plan.json")).unwrap()).unwrap();
    assert_eq!(plan["receipt"], "step-plan-v1");
    assert_eq!(plan["incomplete"], false);

    let rows = plan["functions"].as_array().unwrap();
    let names: Vec<&str> = rows
        .iter()
        .map(|r| r["function"].as_str().unwrap())
        .collect();
    // Ranked by unnoticed faults: the function the test ignores comes first.
    assert_eq!(names, ["helper", "step"], "{plan}");
    let (helper, step) = (&rows[0], &rows[1]);

    assert_eq!(helper["noticed"], 0);
    assert!(helper["unnoticed"].as_u64().unwrap() >= 2);
    assert_eq!(helper["unprotected"], true);
    // The pack counts a call that is a statement of its own as a write.
    assert_eq!(helper["state_writes"], 2);
    assert_eq!(helper["outside_frame"], 2);
    assert_eq!(helper["reaches_out"]["calls"], json!(["log"]));
    assert_eq!(
        helper["reaches_out"]["state_writes"],
        json!(["line 10: log(b)", "line 11: state.count = b"])
    );

    assert!(step["noticed"].as_u64().unwrap() >= 2);
    assert_eq!(step["unprotected"], false);
    assert_eq!(step["outside_frame"], 0);
    assert_eq!(step.get("reaches_out"), None);
    assert!(step["tokens"].as_u64().unwrap() > 10);

    // Every fault of the run is counted once.
    let run: Value =
        serde_json::from_slice(&fs::read(dir.join("work/plan-run.json")).unwrap()).unwrap();
    let counted: u64 = rows
        .iter()
        .map(|r| {
            ["noticed", "unnoticed", "other"]
                .iter()
                .map(|k| r[*k].as_u64().unwrap())
                .sum::<u64>()
        })
        .sum();
    assert_eq!(counted, run["mutants"].as_array().unwrap().len() as u64);

    let table = String::from_utf8_lossy(&output.stdout);
    assert!(
        table.contains("src/app.py: helper") && table.contains("no fault noticed"),
        "{table}"
    );
    assert_eq!(
        files_of(&dir.join("project")),
        before,
        "the source tree changed"
    );
    let _ = fs::remove_dir_all(&dir);
}
