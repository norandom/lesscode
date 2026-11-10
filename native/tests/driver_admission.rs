//! Pack admission and `deconvolute admit-pack`.
//!
//! Governed by `.kiro/specs/agnostic-annotation-driver/` Requirements 1.3 to
//! 1.6 and 8.4 (tasks 2.1 to 2.3). The model-written packs are the ones the
//! prototype gate admitted in attempt 2.

use lesscode_core::adapters::driver::admission::admit;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn pack_text(path: &Path) -> (String, String) {
    let text = fs::read_to_string(path).unwrap();
    let value: Value = serde_json::from_str(&text).unwrap();
    (text, value["language"].as_str().unwrap().to_owned())
}

fn packs_in(dir: &Path) -> Vec<PathBuf> {
    let mut found: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    found.sort();
    found
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lc-test-admit-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn run_cli(args: &[&str]) -> (Option<i32>, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_lesscode-core"))
        .args(args)
        .output()
        .expect("run lesscode-core binary");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

/// A shipped pack as a JSON value and its language, for building wrong packs.
fn base() -> (Value, String, String) {
    let path = &packs_in(&repo().join("native/packs"))[0];
    let (text, language) = pack_text(path);
    (serde_json::from_str(&text).unwrap(), language, text)
}

#[test]
fn shipped_and_model_written_packs_are_admitted() {
    let shipped = packs_in(&repo().join("native/packs"));
    let written = packs_in(&repo().join("native/tests/fixtures/driver/model-packs"));
    assert_eq!((shipped.len(), written.len()), (4, 2));
    for path in shipped.iter().chain(&written) {
        let (text, language) = pack_text(path);
        let result = admit(&text, &language).unwrap();
        assert!(
            result.admitted,
            "{}: {:?}",
            path.display(),
            result.feedback()
        );
    }
}

#[test]
fn wrong_packs_are_rejected_for_the_recorded_reasons() {
    let (pack, language, text) = base();
    let edit = |change: &dyn Fn(&mut Value)| {
        let mut wrong = pack.clone();
        change(&mut wrong);
        serde_json::to_string(&wrong).unwrap()
    };
    let cases = [
        (
            edit(&|p| {
                p["comment_kinds"]
                    .as_array_mut()
                    .unwrap()
                    .push("no_such_kind".into())
            }),
            "kinds_valid",
        ),
        (
            edit(&|p| drop(p.as_object_mut().unwrap().remove("string_kinds"))),
            "shape",
        ),
        (format!("{text}{}", " \n".repeat(41)), "size"),
        (
            edit(&|p| p["operators"]["branch_condition"]["template"] = "!({text}) {bogus}".into()),
            "templates",
        ),
    ];
    for (wrong, check) in cases {
        let result = admit(&wrong, &language).unwrap();
        assert!(!result.admitted, "{check}");
        assert_eq!(result.failed(), vec![check], "{:?}", result.feedback());
    }
}

#[test]
fn details_name_what_is_wrong() {
    let (mut pack, language, _) = base();
    pack["comment_kinds"]
        .as_array_mut()
        .unwrap()
        .push("no_such_kind".into());
    pack["operators"]["branch_condition"]["template"] = "{bogus}".into();
    pack["kinds"].as_object_mut().unwrap().remove("state_write");
    let result = admit(&serde_json::to_string(&pack).unwrap(), &language).unwrap();
    let feedback = result.feedback().join("\n");
    assert!(
        feedback.contains("not in the") && feedback.contains("no_such_kind"),
        "{feedback}"
    );
    assert!(feedback.contains("does not contain {text}"), "{feedback}");
    assert!(
        feedback.contains("unknown placeholders: {bogus}"),
        "{feedback}"
    );
    assert!(feedback.contains("missing kinds.state_write"), "{feedback}");
}

#[test]
fn admit_pack_exit_codes_are_0_1_and_3() {
    let dir = temp_dir("cli");
    let (pack, language, text) = base();
    let good = dir.join("good.json");
    fs::write(&good, &text).unwrap();
    let receipt = dir.join("out/receipt.json");
    let (code, _) = run_cli(&[
        "deconvolute",
        "admit-pack",
        "--pack",
        good.to_str().unwrap(),
        "--lang",
        &language,
        "--out",
        receipt.to_str().unwrap(),
    ]);
    assert_eq!(code, Some(0));
    let written: Value = serde_json::from_slice(&fs::read(&receipt).unwrap()).unwrap();
    assert_eq!(written["receipt"], "pack-admission-v1");
    assert_eq!(written["admitted"], true);
    assert_eq!(written["checks"].as_array().unwrap().len(), 5);

    let mut wrong = pack.clone();
    wrong["string_kinds"] = "oops".into();
    let bad = dir.join("bad.json");
    fs::write(&bad, serde_json::to_string(&wrong).unwrap()).unwrap();
    let rejected = dir.join("rejected.json");
    let (code, stderr) = run_cli(&[
        "deconvolute",
        "admit-pack",
        "--pack",
        bad.to_str().unwrap(),
        "--lang",
        &language,
        "--out",
        rejected.to_str().unwrap(),
    ]);
    assert_eq!(code, Some(1));
    assert!(stderr.contains("shape:"), "{stderr}");
    let written: Value = serde_json::from_slice(&fs::read(&rejected).unwrap()).unwrap();
    assert_eq!(written["admitted"], false);

    let nothing = dir.join("nothing.json");
    for args in [
        vec!["--pack", "no-such-file.json", "--lang", language.as_str()],
        vec!["--pack", good.to_str().unwrap(), "--lang", "klingon"],
        vec![
            "--pack",
            good.to_str().unwrap(),
            "--lang",
            language.as_str(),
            "--controls",
            "x",
        ],
    ] {
        let mut full = vec![
            "deconvolute",
            "admit-pack",
            "--out",
            nothing.to_str().unwrap(),
        ];
        full.extend(args);
        let (code, stderr) = run_cli(&full);
        assert_eq!(code, Some(3), "{full:?}: {stderr}");
        assert!(!nothing.exists(), "{full:?}");
    }
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_control_file_that_cannot_be_read_as_one_is_bad_input() {
    let dir = temp_dir("controls");
    let (_, language, text) = base();
    let good = dir.join("good.json");
    fs::write(&good, &text).unwrap();
    let controls = dir.join("controls.json");
    fs::write(&controls, "{}").unwrap();
    let receipt = dir.join("receipt.json");
    let (code, stderr) = run_cli(&[
        "deconvolute",
        "admit-pack",
        "--pack",
        good.to_str().unwrap(),
        "--lang",
        &language,
        "--controls",
        controls.to_str().unwrap(),
        "--source-root",
        dir.to_str().unwrap(),
        "--out",
        receipt.to_str().unwrap(),
    ]);
    assert_eq!(code, Some(3), "{stderr}");
    assert!(stderr.contains("not a control file"), "{stderr}");
    assert!(!receipt.exists());
    fs::remove_dir_all(&dir).unwrap();
}
