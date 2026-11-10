//! Integration tests for deconvolution CLI porcelain commands.
//!
//! Governed by:
//! - REQ-DOG-2.1 through REQ-DOG-2.5, REQ-DOG-6.1 through REQ-DOG-6.3
//! - `.kiro/specs/dogfood-self-application/tasks.md` Task 4

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn run_cli(args: &[&str]) -> (std::process::ExitStatus, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_lesscode-core"))
        .args(args)
        .output()
        .expect("run lesscode-core binary");
    (
        output.status,
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lc-test-deconv-{name}-{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    dir
}

#[test]
fn deconvolute_help_lists_all_subcommands() {
    let (status, stdout, _) = run_cli(&["deconvolute", "--help"]);
    assert!(status.success());
    assert!(stdout.contains("analyze"));
    assert!(stdout.contains("mutate"));
    assert!(stdout.contains("density"));
}

fn receipt_of(path: &std::path::Path) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

/// One small source per language with a condition, a state write and a
/// failure signal.
const SAMPLES: [(&str, &str, &str); 4] = [
    (
        "rust",
        "sample.rs",
        "pub fn step(total: &mut i32, a: i32) -> Result<i32, String> {\n    if a > 3 {\n        return Err(\"big\".to_owned());\n    }\n    *total = a;\n    Ok(a)\n}\n",
    ),
    (
        "python",
        "sample.py",
        "def step(state, a):\n    if a > 3:\n        raise ValueError(\"big\")\n    state.total = a\n    return a\n",
    ),
    (
        "typescript",
        "sample.ts",
        "export function step(state: { total: number }, a: number): number {\n  if (a > 3) {\n    throw new Error(\"big\");\n  }\n  state.total = a;\n  return a;\n}\n",
    ),
    (
        "java",
        "Sample.java",
        "class Sample {\n  int total;\n  int step(int a) {\n    if (a > 3) {\n      throw new IllegalArgumentException(\"big\");\n    }\n    total = a;\n    return a;\n  }\n}\n",
    ),
];

#[test]
fn analyze_reports_sites_writes_and_boundaries_for_four_languages() {
    for (lang, name, source) in SAMPLES {
        let dir = temp_dir(&format!("analyze-{lang}"));
        let src = dir.join(name);
        fs::write(&src, source).unwrap();
        let out = dir.join("analysis.json");
        let (status, _, stderr) = run_cli(&[
            "deconvolute",
            "analyze",
            "--lang",
            lang,
            "--source",
            src.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
        ]);
        assert_eq!(status.code(), Some(0), "{lang}: {stderr}");
        let receipt = receipt_of(&out);
        assert_eq!(receipt["receipt"], "deconvolution-analysis-v2");
        assert_eq!(receipt["lang"], lang);
        assert_eq!(receipt["passed"], true, "{lang}");
        assert_eq!(receipt["gaps"].as_array().unwrap().len(), 0, "{lang}");
        assert_eq!(
            receipt["write_sites"].as_array().unwrap().len(),
            1,
            "{lang}"
        );
        assert_eq!(
            receipt["function_boundaries"].as_array().unwrap().len(),
            1,
            "{lang}"
        );
        let kinds: Vec<&str> = receipt["sites"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["kind"].as_str().unwrap())
            .collect();
        for kind in [
            "branch_condition",
            "failure_state",
            "state_write",
            "pure_shell_boundary",
        ] {
            assert!(kinds.contains(&kind), "{lang}: no {kind} in {kinds:?}");
        }
        assert_eq!(receipt["pack_sha256"].as_str().unwrap().len(), 64);
        let _ = fs::remove_dir_all(&dir);
    }
}

#[test]
fn analyze_flags_declared_gaps_and_frame_violations() {
    let dir = temp_dir("gap-rust");
    let src = dir.join("unsafe.rs");
    fs::write(
        &src,
        "pub fn dangerous(total: &mut i32) {\n    unsafe {\n        let ptr: *const i32 = std::ptr::null();\n        let _ = ptr;\n    }\n    *total = 1;\n}\n",
    )
    .unwrap();
    let out = dir.join("analysis.json");
    let args = [
        "deconvolute",
        "analyze",
        "--lang",
        "rust",
        "--source",
        src.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ];
    let (status, _, _) = run_cli(&args);
    assert_eq!(status.code(), Some(1));
    let receipt = receipt_of(&out);
    assert_eq!(receipt["passed"], false);
    let gaps = receipt["gaps"].as_array().unwrap();
    assert!(gaps.len() >= 2, "{gaps:?}");
    assert!(
        gaps[0]["location"]
            .as_str()
            .unwrap()
            .ends_with("unsafe.rs:2"),
        "{gaps:?}"
    );

    // A write outside the frame specification is a violation; inside it is not.
    let clean = dir.join("clean.rs");
    fs::write(
        &clean,
        "pub fn set(total: &mut i32) {\n    *total = 1;\n}\n",
    )
    .unwrap();
    for (allowed, code, violations) in [("other.rs", 1, 1), ("clean.rs", 0, 0)] {
        let spec = dir.join("frame.json");
        fs::write(
            &spec,
            format!(r#"{{"allowed_write_sites": ["{allowed}"], "permitted_shell_locations": [], "failure_invariants": []}}"#),
        )
        .unwrap();
        let (status, _, stderr) = run_cli(&[
            "deconvolute",
            "analyze",
            "--lang",
            "rust",
            "--source",
            clean.to_str().unwrap(),
            "--frame-spec",
            spec.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
        ]);
        assert_eq!(status.code(), Some(code), "{allowed}: {stderr}");
        assert_eq!(
            receipt_of(&out)["violations"].as_array().unwrap().len(),
            violations
        );
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn analyze_and_mutate_refuse_a_source_that_does_not_parse() {
    let dir = temp_dir("broken");
    let src = dir.join("broken.py");
    fs::write(&src, "def step(a):\n    return ((((\n").unwrap();
    let out = dir.join("receipt.json");
    let out_dir = dir.join("mutants");
    for command in ["analyze", "mutate"] {
        let mut args = vec![
            "deconvolute",
            command,
            "--lang",
            "python",
            "--source",
            src.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
        ];
        if command == "mutate" {
            args.extend(["--out-dir", out_dir.to_str().unwrap()]);
        }
        let (status, stdout, stderr) = run_cli(&args);
        assert_eq!(status.code(), Some(3), "{command}: {stderr}");
        assert!(
            stderr.contains("does not parse") && stderr.contains("broken.py:1:0"),
            "{stderr}"
        );
        assert!(
            stdout.is_empty() && !out.exists() && !out_dir.exists(),
            "{command}"
        );
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn mutate_writes_mutants_for_four_languages_and_leaves_the_source_alone() {
    for (lang, name, source) in SAMPLES {
        let dir = temp_dir(&format!("mutate-{lang}"));
        let src = dir.join(name);
        fs::write(&src, source).unwrap();
        let out_dir = dir.join("mutants");
        let out = dir.join("mutation.json");
        let (status, _, stderr) = run_cli(&[
            "deconvolute",
            "mutate",
            "--lang",
            lang,
            "--source",
            src.to_str().unwrap(),
            "--out-dir",
            out_dir.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
        ]);
        assert_eq!(status.code(), Some(0), "{lang}: {stderr}");
        assert!(!stderr.contains("No mutation operator matched"), "{stderr}");
        assert_eq!(
            fs::read_to_string(&src).unwrap(),
            source,
            "{lang}: the source changed"
        );
        let receipt = receipt_of(&out);
        assert_eq!(receipt["receipt"], "deconvolution-mutation-v2");
        let mutants = receipt["mutants"].as_array().unwrap();
        assert!(mutants.len() >= 3, "{lang}: {mutants:?}");
        assert_eq!(receipt["mutants_generated"], mutants.len());
        let extension = src.extension().unwrap().to_str().unwrap();
        for mutant in mutants {
            let file = out_dir.join(mutant["file"].as_str().unwrap());
            assert_eq!(file.extension().unwrap().to_str().unwrap(), extension);
            let text = fs::read_to_string(&file).unwrap();
            assert_ne!(text, source, "{lang}: {mutant}");
            assert_eq!(mutant["sha256"].as_str().unwrap().len(), 64);
        }
        assert_eq!(fs::read_dir(&out_dir).unwrap().count(), mutants.len());
        let operators: Vec<&str> = mutants
            .iter()
            .map(|m| m["operator"].as_str().unwrap())
            .collect();
        assert!(
            operators.contains(&"negate") && operators.contains(&"drop_statement"),
            "{lang}: {operators:?}"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}

#[test]
fn mutate_limit_and_seed_select_the_same_mutants_every_time() {
    let dir = temp_dir("mutate-seed");
    let (_, name, source) = SAMPLES[3];
    let src = dir.join(name);
    fs::write(&src, source).unwrap();
    let mut picks = Vec::new();
    for (round, seed) in ["7", "7", "8"].iter().enumerate() {
        let out_dir = dir.join(format!("mutants-{round}"));
        let out = dir.join(format!("mutation-{round}.json"));
        let (status, _, stderr) = run_cli(&[
            "deconvolute",
            "mutate",
            "--lang",
            "java",
            "--source",
            src.to_str().unwrap(),
            "--out-dir",
            out_dir.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
            "--limit",
            "2",
            "--seed",
            seed,
        ]);
        assert_eq!(status.code(), Some(0), "{stderr}");
        let receipt = receipt_of(&out);
        assert_eq!(receipt["mutants_generated"], 2);
        assert!(receipt["mutants_available"].as_u64().unwrap() > 2);
        assert_eq!(fs::read_dir(&out_dir).unwrap().count(), 2);
        let hashes: Vec<String> = receipt["mutants"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| m["sha256"].as_str().unwrap().to_owned())
            .collect();
        picks.push(hashes);
    }
    assert_eq!(picks[0], picks[1]);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn mutate_without_matching_operator_exits_1_with_receipt() {
    let dir = temp_dir("mutate-none");
    let src = dir.join("pure.rs");
    fs::write(&src, "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n").unwrap();
    let out_dir = dir.join("mutants");
    let out = dir.join("mutation.json");
    let (status, stdout, stderr) = run_cli(&[
        "deconvolute",
        "mutate",
        "--lang",
        "rust",
        "--source",
        src.to_str().unwrap(),
        "--out-dir",
        out_dir.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(status.code(), Some(1), "{stderr}");
    assert!(stdout.is_empty(), "{stdout}");
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
    assert!(stderr.contains("No mutation operator matched"), "{stderr}");
    assert!(stderr.contains(src.to_str().unwrap()), "{stderr}");
    let receipt = receipt_of(&out);
    assert_eq!(receipt["receipt"], "deconvolution-mutation-v2");
    assert_eq!(receipt["mutants_generated"], 0);
    assert_eq!(receipt["passed"], false);
    assert_eq!(fs::read_dir(&out_dir).unwrap().count(), 0);
    let _ = fs::remove_dir_all(&dir);
}
#[test]
fn density_subcommand_evaluates_structural_reduction() {
    let dir = temp_dir("density-rust");
    let base = dir.join("base.rs");
    let cand = dir.join("cand.rs");

    let mut base_text = String::new();
    for i in 0..100 {
        base_text.push_str(&format!("fn fn_{i}() {{\n    let _ = {i};\n}}\n"));
    }
    fs::write(&base, &base_text).unwrap();

    let mut cand_text = String::new();
    for i in 0..30 {
        cand_text.push_str(&format!("fn fn_{i}() {{\n    let _ = {i};\n}}\n"));
    }
    fs::write(&cand, &cand_text).unwrap();

    let out = dir.join("density.json");
    let (status, _, _) = run_cli(&[
        "deconvolute",
        "density",
        "--lang",
        "rust",
        "--baseline",
        base.to_str().unwrap(),
        "--candidate",
        cand.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);

    assert_eq!(status.code(), Some(0));
    let receipt: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&out).unwrap()).unwrap();
    assert_eq!(receipt["receipt"], "deconvolution-density-v1");
    assert_eq!(receipt["passed"], true);
    assert_eq!(receipt["verdict"], "pass");
    assert!(receipt["line_reduction_pct"].as_f64().unwrap() >= 20.0);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn unsupported_language_fails_with_exit_code_3() {
    let dir = temp_dir("unsupported-lang");
    let src = dir.join("code.rb");
    fs::write(&src, "puts 'hello'").unwrap();

    let (status, _, stderr) = run_cli(&[
        "deconvolute",
        "analyze",
        "--lang",
        "ruby",
        "--source",
        src.to_str().unwrap(),
    ]);

    assert_eq!(status.code(), Some(3));
    assert!(stderr.contains("Unsupported language"));

    let _ = fs::remove_dir_all(&dir);
}
