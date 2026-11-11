//! Fixtures for the budgets checker (`rust-syntax-units-v2`).

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
    assert_eq!(receipt["schema"], "rust-syntax-units-v2");
    assert_eq!(status_of(&receipt, 0), "within_budget");
    assert_eq!(receipt["files"][0]["logical_units"], 1);
    assert_eq!(receipt["files"][0]["longest_fn_name"], "main");
    // Without a ceiling in the entry the defaults apply, in tokens.
    assert_eq!(receipt["files"][0]["max_tokens"], 2800);
    assert_eq!(receipt["files"][0]["max_physical_lines"], Value::Null);
    assert_eq!(receipt["files"][0]["max_logical_units"], 300);
    assert_eq!(receipt["files"][0]["max_fn_tokens"], 560);
    assert_eq!(receipt["files"][0]["max_fn_lines"], Value::Null);
    // `fn main ( ) { }`: six tokens, all in the one function.
    assert_eq!(receipt["files"][0]["production_tokens"], 6);
    assert_eq!(receipt["files"][0]["longest_fn_tokens"], 6);
    assert_eq!(receipt["summary"]["total"], 1);
    assert_eq!(receipt["summary"]["within_budget"], 1);
    let sidecar = fs::read_to_string(dir.join("out.json.sha256")).unwrap();
    assert!(sidecar.ends_with("  out.json\n"));
}

/// Tokens do not depend on layout: the same code on one line or on many
/// has the same count, comments and test code are not counted, and a
/// ceiling in tokens cannot be met by squeezing lines together.
#[test]
fn token_ceilings_ignore_layout_comments_and_test_code() {
    let spread = "// a comment\nfn a(x: i32) -> i32 {\n    let y = x + 1;\n    y\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {\n        assert_eq!(super::a(1), 2);\n    }\n}\n";
    let dense = "fn a(x: i32) -> i32 { let y = x + 1; y }\n";
    let mut counts = Vec::new();
    for (name, text) in [("spread", spread), ("dense", dense)] {
        let dir = fixture(name);
        write(&dir.join("lib.rs"), text);
        let config = r#"{"files": [{"path": "lib.rs", "owner": "t", "allocation": "a"}]}"#;
        let (_, receipt) = run(&dir, config);
        let file = &receipt["files"][0];
        counts.push((
            file["production_tokens"].as_u64().unwrap(),
            file["longest_fn_tokens"].as_u64().unwrap(),
        ));
        assert_eq!(file["longest_fn_tokens_name"], "a");
    }
    assert_eq!(
        counts[0], counts[1],
        "layout, comments and tests must not count"
    );

    let (tokens, _) = counts[1];
    let dir = fixture("token-ceiling");
    write(&dir.join("lib.rs"), dense);
    let over = format!(
        r#"{{"files": [{{"path": "lib.rs", "owner": "t", "allocation": "a", "max_tokens": {}}}]}}"#,
        tokens - 1
    );
    let (code, receipt) = run(&dir, &over);
    assert_eq!((code, status_of(&receipt, 0)), (1, "exceeded"));
    let exact = over.replace(&format!("{}", tokens - 1), &tokens.to_string());
    let (code, _) = run(&dir, &exact);
    assert_eq!(code, 0);
    let function =
        r#"{"files": [{"path": "lib.rs", "owner": "t", "allocation": "a", "max_fn_tokens": 5}]}"#;
    let (code, receipt) = run(&dir, function);
    assert_eq!((code, status_of(&receipt, 0)), (1, "exceeded"));
}

/// An entry that still gives its ceiling in lines is held to it, in lines,
/// and a token ceiling in the same entry takes precedence.
#[test]
fn a_line_ceiling_in_an_entry_still_applies_until_it_is_replaced() {
    let dir = fixture("legacy");
    write(&dir.join("lib.rs"), "fn a() {}\nfn b() {}\nfn c() {}\n");
    let lines = r#"{"files": [{"path": "lib.rs", "owner": "t", "allocation": "a",
        "max_physical_lines": 2}]}"#;
    let (code, receipt) = run(&dir, lines);
    assert_eq!((code, status_of(&receipt, 0)), (1, "exceeded"));
    assert_eq!(receipt["files"][0]["max_physical_lines"], 2);
    assert_eq!(receipt["files"][0]["max_tokens"], Value::Null);
    let both = r#"{"files": [{"path": "lib.rs", "owner": "t", "allocation": "a",
        "max_physical_lines": 2, "max_tokens": 100}]}"#;
    let (code, receipt) = run(&dir, both);
    assert_eq!((code, status_of(&receipt, 0)), (0, "within_budget"));
    assert_eq!(receipt["files"][0]["max_tokens"], 100);
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
