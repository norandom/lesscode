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

#[test]
fn analyze_subcommand_reports_clean_pure_rust() {
    let dir = temp_dir("clean-rust");
    let src = dir.join("clean.rs");
    fs::write(
        &src,
        r#"
pub fn add(a: i32, b: i32) -> i32 {
    let c = a + b;
    c * 2
}
"#,
    )
    .unwrap();

    let out = dir.join("analysis.json");
    let (status, _, _) = run_cli(&[
        "deconvolute",
        "analyze",
        "--lang",
        "rust",
        "--source",
        src.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);

    assert_eq!(status.code(), Some(0));
    assert!(out.exists());
    let receipt: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&out).unwrap()).unwrap();
    assert_eq!(receipt["receipt"], "deconvolution-analysis-v1");
    assert_eq!(receipt["passed"], true);
    assert_eq!(receipt["gaps"].as_array().unwrap().len(), 0);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn analyze_subcommand_flags_dynamic_gaps() {
    let dir = temp_dir("gap-rust");
    let src = dir.join("unsafe.rs");
    fs::write(
        &src,
        r#"
pub fn dangerous() {
    unsafe {
        let ptr: *const i32 = std::ptr::null();
    }
}
"#,
    )
    .unwrap();

    let out = dir.join("analysis.json");
    let (status, _, _) = run_cli(&[
        "deconvolute",
        "analyze",
        "--lang",
        "rust",
        "--source",
        src.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);

    assert_eq!(status.code(), Some(1));
    assert!(out.exists());
    let receipt: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&out).unwrap()).unwrap();
    assert_eq!(receipt["passed"], false);
    assert!(!receipt["gaps"].as_array().unwrap().is_empty());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn mutate_subcommand_generates_adverse_mutants() {
    let dir = temp_dir("mutate-rust");
    let src = dir.join("source.rs");
    fs::write(
        &src,
        r#"
pub fn check(status: i32, level2_ops: i32) -> bool {
    if status != 0 {
        return false;
    }
    if level2_ops > 0 {
        return false;
    }
    true
}
"#,
    )
    .unwrap();

    let out_dir = dir.join("mutants");
    let out_receipt = dir.join("mutate-receipt.json");
    let (status, _, stderr) = run_cli(&[
        "deconvolute",
        "mutate",
        "--lang",
        "rust",
        "--source",
        src.to_str().unwrap(),
        "--out-dir",
        out_dir.to_str().unwrap(),
        "--out",
        out_receipt.to_str().unwrap(),
    ]);

    assert_eq!(status.code(), Some(0));
    assert!(out_dir.exists());
    assert!(!stderr.contains("No mutation operator matched"), "{stderr}");
    let receipt: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&out_receipt).unwrap()).unwrap();
    assert_eq!(receipt["receipt"], "deconvolution-mutation-v1");
    let count = receipt["mutants_generated"].as_u64().unwrap();
    assert!(count >= 1);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn mutate_without_matching_operator_exits_1_with_receipt() {
    let dir = temp_dir("mutate-nomatch");
    let src = dir.join("plain.rs");
    fs::write(&src, "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n").unwrap();

    let out_dir = dir.join("mutants");
    let out_receipt = dir.join("mutate-receipt.json");
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
        out_receipt.to_str().unwrap(),
    ]);

    assert_eq!(status.code(), Some(1), "{stderr}");
    assert!(stdout.is_empty(), "{stdout}");
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
    assert!(stderr.contains("No mutation operator matched"), "{stderr}");
    assert!(stderr.contains(src.to_str().unwrap()), "{stderr}");
    let receipt: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&out_receipt).unwrap()).unwrap();
    assert_eq!(receipt["receipt"], "deconvolution-mutation-v1");
    assert_eq!(receipt["mutants_generated"], 0);
    assert_eq!(receipt["passed"], false);
    let written = fs::read_dir(&out_dir).map(|d| d.count()).unwrap_or(0);
    assert_eq!(written, 0);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn analyze_python_is_not_implemented_and_writes_no_receipt() {
    let dir = temp_dir("analyze-python");
    let src = dir.join("module.py");
    fs::write(&src, "def add(a, b):\n    return a + b\n").unwrap();
    let out = dir.join("analysis.json");

    for lang in ["python", "py"] {
        let base = ["deconvolute", "analyze", "--lang", lang, "--source"];
        let mut to_stdout = base.to_vec();
        to_stdout.push(src.to_str().unwrap());
        let mut to_file = to_stdout.clone();
        to_file.extend(["--out", out.to_str().unwrap()]);

        for args in [to_stdout, to_file] {
            let (status, stdout, stderr) = run_cli(&args);
            assert_eq!(status.code(), Some(2), "{lang}: {stderr}");
            assert!(stdout.is_empty(), "{lang}: {stdout}");
            assert_eq!(stderr.lines().count(), 1, "{lang}: {stderr}");
            assert!(stderr.contains("not implemented"), "{lang}: {stderr}");
            assert!(!out.exists(), "{lang}: receipt was written");
        }
    }

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn mutate_python_is_not_implemented_and_writes_nothing() {
    let dir = temp_dir("mutate-python");
    let src = dir.join("module.py");
    fs::write(
        &src,
        "def check(status):\n    if status != 0:\n        return False\n    return True\n",
    )
    .unwrap();
    let out_dir = dir.join("mutants");
    let out_receipt = dir.join("mutate-receipt.json");

    for lang in ["python", "py"] {
        let mut args = vec!["deconvolute", "mutate", "--lang", lang, "--source"];
        args.extend([
            src.to_str().unwrap(),
            "--out-dir",
            out_dir.to_str().unwrap(),
        ]);
        let mut with_out = args.clone();
        with_out.extend(["--out", out_receipt.to_str().unwrap()]);

        for args in [args, with_out] {
            let (status, stdout, stderr) = run_cli(&args);
            assert_eq!(status.code(), Some(2), "{lang}: {stderr}");
            assert!(stdout.is_empty(), "{lang}: {stdout}");
            assert_eq!(stderr.lines().count(), 1, "{lang}: {stderr}");
            assert!(stderr.contains("not implemented"), "{lang}: {stderr}");
            let entries: Vec<_> = fs::read_dir(&dir)
                .unwrap()
                .map(|e| e.unwrap().file_name())
                .collect();
            assert_eq!(entries, vec!["module.py"], "{lang}: files were written");
        }
    }

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn analyze_java_and_typescript_still_print_receipts() {
    let adapters = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("adapters");
    for (lang, file) in [
        ("java", "java/ProbeMain.java"),
        ("typescript", "typescript/candidate_degradation.ts"),
    ] {
        let src = adapters.join(file);
        let (status, stdout, stderr) = run_cli(&[
            "deconvolute",
            "analyze",
            "--lang",
            lang,
            "--source",
            src.to_str().unwrap(),
        ]);
        assert_eq!(status.code(), Some(0), "{lang}: {stderr}");
        let receipt: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(receipt["receipt"], "deconvolution-analysis-v1");
        assert_eq!(receipt["lang"], lang);
        assert_eq!(receipt["passed"], true);
    }
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
