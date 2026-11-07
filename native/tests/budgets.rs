//! Fixtures for the budgets checker (`rust-syntax-units-v1`).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use lesscode_core::budgets::cli_run;
use serde_json::Value;

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn fixture(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "budgets-fixture-{}-{}-{name}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst),
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(path: &Path, text: &str) {
    fs::write(path, text).unwrap();
}

fn run(dir: &Path, config: &str) -> (u8, Value) {
    let cfg = dir.join("complexity-budgets.json");
    let out = dir.join("out.json");
    write(&cfg, config);
    let code = cli_run(dir, &cfg, &out).unwrap();
    let receipt = fs::read_to_string(&out).unwrap();
    (code, serde_json::from_str(&receipt).unwrap())
}

fn status_of(receipt: &Value, index: usize) -> &str {
    receipt["files"][index]["status"].as_str().unwrap()
}

#[test]
fn within_budget_reports_clean() {
    let dir = fixture("within");
    write(&dir.join("lib.rs"), "fn main() {}\n");
    let config = r#"{"files": [{"path": "lib.rs", "owner": "t", "allocation": "a"}]}"#;
    let (code, receipt) = run(&dir, config);
    assert_eq!(code, 0);
    assert_eq!(receipt["schema"], "rust-syntax-units-v1");
    assert_eq!(status_of(&receipt, 0), "within_budget");
    assert_eq!(receipt["files"][0]["logical_units"], 1);
    assert_eq!(receipt["files"][0]["longest_fn_name"], "main");
    assert_eq!(receipt["files"][0]["max_physical_lines"], 400);
    assert_eq!(receipt["files"][0]["max_logical_units"], 300);
    assert_eq!(receipt["files"][0]["max_fn_lines"], 80);
    assert_eq!(receipt["summary"]["total"], 1);
    assert_eq!(receipt["summary"]["within_budget"], 1);
    let sidecar = fs::read_to_string(dir.join("out.json.sha256")).unwrap();
    assert!(sidecar.ends_with("  out.json\n"));
}

#[test]
fn exceeded_physical_lines_fails() {
    let dir = fixture("physical");
    write(&dir.join("lib.rs"), "fn a() {}\nfn b() {}\nfn c() {}\n");
    let config = r#"{"files": [{"path": "lib.rs", "owner": "t", "allocation": "a",
        "max_physical_lines": 2}]}"#;
    let (code, receipt) = run(&dir, config);
    assert_eq!(code, 1);
    assert_eq!(status_of(&receipt, 0), "exceeded");
    assert_eq!(receipt["summary"]["exceeded"], 1);
}

#[test]
fn exceeded_logical_units_fails() {
    let dir = fixture("units");
    write(
        &dir.join("lib.rs"),
        "fn f() {\n    let a = 1;\n    let b = 2;\n}\n",
    );
    let config = r#"{"files": [{"path": "lib.rs", "owner": "t", "allocation": "a",
        "max_logical_units": 2}]}"#;
    let (code, receipt) = run(&dir, config);
    assert_eq!(code, 1);
    assert_eq!(receipt["files"][0]["logical_units"], 3);
    assert_eq!(status_of(&receipt, 0), "exceeded");
}

#[test]
fn exceeded_fn_lines_fails() {
    let dir = fixture("fnlines");
    write(&dir.join("lib.rs"), "fn f() {\n    let a = 1;\n}\n");
    let config = r#"{"files": [{"path": "lib.rs", "owner": "t", "allocation": "a",
        "max_fn_lines": 2}]}"#;
    let (code, receipt) = run(&dir, config);
    assert_eq!(code, 1);
    assert_eq!(receipt["files"][0]["longest_fn_lines"], 3);
    assert_eq!(receipt["files"][0]["longest_fn_name"], "f");
    assert_eq!(status_of(&receipt, 0), "exceeded");
}

#[test]
fn cfg_test_lines_and_units_are_separated() {
    let dir = fixture("cfgtest");
    let source = "fn prod() {}\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {\n        let x = 1;\n        let y = 2;\n        let z = 3;\n    }\n}\n";
    write(&dir.join("lib.rs"), source);
    let config = r#"{"files": [{"path": "lib.rs", "owner": "t", "allocation": "a",
        "max_logical_units": 1, "max_physical_lines": 10, "max_fn_lines": 1}]}"#;
    let (code, receipt) = run(&dir, config);
    assert_eq!(code, 0);
    let file = &receipt["files"][0];
    assert_eq!(file["total_lines"], 10);
    assert_eq!(file["production_lines"], 1);
    assert_eq!(file["test_lines"], 9);
    assert_eq!(file["logical_units"], 1);
    assert_eq!(file["longest_fn_name"], "prod");
    assert_eq!(status_of(&receipt, 0), "within_budget");
}

#[test]
fn disposition_reports_disposed() {
    let dir = fixture("disposition");
    write(&dir.join("lib.rs"), "fn main() {}\n");
    let config = r#"{"files": [{"path": "lib.rs", "owner": "t", "allocation": "a",
        "disposition": "keep-ceiling"}]}"#;
    let (code, receipt) = run(&dir, config);
    assert_eq!(code, 0);
    assert_eq!(status_of(&receipt, 0), "disposed");
    assert_eq!(receipt["files"][0]["disposition"], "keep-ceiling");
    assert_eq!(receipt["summary"]["disposed"], 1);
}

#[test]
fn unlisted_file_fails() {
    let dir = fixture("unlisted");
    write(&dir.join("lib.rs"), "fn main() {}\n");
    write(&dir.join("extra.rs"), "fn extra() {}\n");
    let config = r#"{"files": [{"path": "lib.rs", "owner": "t", "allocation": "a"}]}"#;
    let (code, receipt) = run(&dir, config);
    assert_eq!(code, 1);
    assert_eq!(receipt["unlisted"][0], "extra.rs");
    assert_eq!(receipt["summary"]["unlisted"], 1);
}

#[test]
fn missing_file_fails() {
    let dir = fixture("missing");
    let config = r#"{"files": [{"path": "ghost.rs", "owner": "t", "allocation": "a"}]}"#;
    let (code, receipt) = run(&dir, config);
    assert_eq!(code, 1);
    assert_eq!(status_of(&receipt, 0), "missing");
    assert_eq!(receipt["files"][0]["present"], false);
    assert_eq!(receipt["summary"]["missing"], 1);
}

#[test]
fn nested_paths_are_listed_with_forward_slashes() {
    let dir = fixture("nested");
    fs::create_dir_all(dir.join("sub")).unwrap();
    write(&dir.join("sub").join("inner.rs"), "fn inner() {}\n");
    let config = r#"{"files": [{"path": "sub/inner.rs", "owner": "t", "allocation": "a"}]}"#;
    let (code, receipt) = run(&dir, config);
    assert_eq!(code, 0);
    assert_eq!(receipt["files"][0]["path"], "sub/inner.rs");
}

#[test]
fn malformed_config_is_rejected() {
    let dir = fixture("malformed");
    write(&dir.join("lib.rs"), "fn main() {}\n");
    let cfg = dir.join("complexity-budgets.json");
    write(&cfg, r#"{"files": [{"path": "lib.rs"}]}"#);
    let out = dir.join("out.json");
    assert!(cli_run(&dir, &cfg, &out).is_err());
    write(
        &cfg,
        r#"{"files": [{"path": "lib.rs", "owner": "t", "allocation": "a",
        "bogus": 1}]}"#,
    );
    assert!(cli_run(&dir, &cfg, &out).is_err());
}
