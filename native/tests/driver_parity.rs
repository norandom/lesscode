//! Parity of the native driver with the retained prototype results.
//!
//! Governed by `.kiro/specs/agnostic-annotation-driver/` Requirements 1.7,
//! 9.1 to 9.3 and 9.5 (tasks 3.1 to 3.4). The references under
//! `tests/fixtures/driver/reference/` were written by the frozen Python
//! prototype from the scope files under `tests/fixtures/driver/scopes/`
//! (`docs/experiments/annotation-driver/native-spike/make_reference.py`).
//! A missing or changed reference fails with "not run"; it never passes.

use lesscode_core::adapters::driver::annotations::{site_of, validate};
use lesscode_core::adapters::driver::operators::{classify, mutants};
use lesscode_core::adapters::driver::pack::Pack;
use lesscode_core::adapters::driver::registry::{embedded, shipped_texts};
use lesscode_core::adapters::driver::sites::eligible_sites;
use lesscode_core::adapters::driver::source::SourceSet;
use lesscode_core::specimen_runner::sha256_hex_bytes;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/driver")
}

struct Reference {
    name: String,
    data: Value,
    source: SourceSet,
}

fn strings(value: &Value) -> Vec<String> {
    let items = value.as_array().expect("a list");
    items
        .iter()
        .map(|s| s.as_str().expect("a string").to_owned())
        .collect()
}

/// The five references with their scope files checked against the hashes
/// the prototype recorded.
fn references() -> Vec<Reference> {
    let dir = fixtures().join("reference");
    let mut paths: Vec<_> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("not run: no references in {}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .collect();
    paths.sort();
    assert_eq!(paths.len(), 5, "not run: expected five references");
    let load = |path: &PathBuf| {
        let data: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        let root = fixtures()
            .join("scopes")
            .join(data["repository"].as_str().unwrap());
        for (file, sha) in data["scope_files_sha256"].as_object().unwrap() {
            let bytes = fs::read(root.join(file))
                .unwrap_or_else(|e| panic!("not run: scope file {file} missing: {e}"));
            assert_eq!(
                sha256_hex_bytes(&bytes),
                sha.as_str().unwrap(),
                "not run: {file} changed"
            );
        }
        let source = SourceSet {
            root,
            files: strings(&data["files"]),
            test_paths: strings(&data["test_paths"]),
        };
        Reference {
            name: data["scope"].as_str().unwrap().to_owned(),
            data,
            source,
        }
    };
    paths.iter().map(load).collect()
}

/// The shipped pack as the gate had it: without the pairs that suppress a
/// swap saying the same as the negation.
fn gate_pack(language: &str) -> Pack {
    let text = shipped_texts()
        .iter()
        .find(|t| serde_json::from_str::<Value>(t).unwrap()["language"] == language)
        .unwrap();
    let mut value: Value = serde_json::from_str(text).unwrap();
    let branch = value["operators"]["branch_condition"]
        .as_object_mut()
        .unwrap();
    assert!(branch.remove("negation_swaps").is_some());
    Pack::parse(&value.to_string()).unwrap()
}

/// Site, operator and hash of every mutant, as the reference lists them.
fn native_mutants(reference: &Reference, pack: &Pack) -> Vec<Value> {
    let mut found = Vec::new();
    for site in eligible_sites(&reference.source, pack).unwrap() {
        let text = reference.source.raw(&site.file).unwrap();
        for mutant in mutants(pack, &text, &site).unwrap() {
            found.push(json!({
                "site": site,
                "operator": mutant.operator,
                "sha256": sha256_hex_bytes(mutant.text.as_bytes()),
            }));
        }
    }
    found
}

#[test]
fn eligible_sites_equal_the_prototype() {
    for reference in references() {
        let pack = embedded(reference.data["language"].as_str().unwrap()).unwrap();
        let sites = eligible_sites(&reference.source, pack).unwrap();
        let expected = reference.data["sites"].as_array().unwrap();
        assert_eq!(sites.len(), expected.len(), "{}", reference.name);
        for (site, want) in sites.iter().zip(expected) {
            assert_eq!(
                &serde_json::to_value(site).unwrap(),
                want,
                "{}",
                reference.name
            );
        }
    }
}

#[test]
fn mutants_equal_the_prototype_under_the_gate_packs() {
    for reference in references() {
        let pack = gate_pack(reference.data["language"].as_str().unwrap());
        let found = native_mutants(&reference, &pack);
        let expected = reference.data["mutants"].as_array().unwrap();
        assert_eq!(found.len(), expected.len(), "{}", reference.name);
        for (mutant, want) in found.iter().zip(expected) {
            assert_eq!(mutant, want, "{}", reference.name);
        }
    }
}

/// Requirement 3.4 is the one intended difference from the prototype: with
/// the shipped packs, a swap of a condition's own equality operator is left
/// out because the negation at the same site says the same.
#[test]
fn shipped_packs_drop_only_swaps_that_repeat_the_negation() {
    let mut dropped_per_scope = Vec::new();
    for reference in references() {
        let pack = embedded(reference.data["language"].as_str().unwrap()).unwrap();
        let found = native_mutants(&reference, pack);
        let expected = reference.data["mutants"].as_array().unwrap();
        let pairs = &pack.operators.branch_condition.negation_swaps;
        let mut kept = found.iter().peekable();
        let mut dropped = 0;
        for want in expected {
            if kept.peek() == Some(&want) {
                kept.next();
                continue;
            }
            let operator = want["operator"].as_str().unwrap();
            let repeats = pairs.iter().any(|(a, b)| {
                operator.starts_with(&format!("swap:{a}->{b}@"))
                    || operator.starts_with(&format!("swap:{b}->{a}@"))
            });
            assert!(
                repeats,
                "{}: unexpected difference at {want}",
                reference.name
            );
            let negated = expected
                .iter()
                .any(|m| m["site"] == want["site"] && m["operator"] == "negate");
            assert!(negated, "{}: {want} has no negation", reference.name);
            dropped += 1;
        }
        assert!(
            kept.next().is_none(),
            "{}: extra native mutants",
            reference.name
        );
        dropped_per_scope.push((reference.name.clone(), expected.len(), dropped));
    }
    println!("mutants in the reference and dropped as repeats: {dropped_per_scope:?}");
    assert!(dropped_per_scope.iter().any(|(_, _, dropped)| *dropped > 0));
}

#[test]
fn every_control_gives_the_recorded_outcome() {
    let mut total = 0;
    for reference in references() {
        let pack = embedded(reference.data["language"].as_str().unwrap()).unwrap();
        for control in reference.data["controls"].as_array().unwrap() {
            let annotations = control["annotations"].as_array().unwrap();
            let decisions = validate(&reference.source, pack, annotations).unwrap();
            let got: Vec<String> = decisions.iter().map(|d| d.outcome()).collect();
            let label = format!("{} {}", reference.name, control["label"]);
            assert_eq!(got, strings(&control["got"]), "{label}");
            assert_eq!(got, strings(&control["expected"]), "{label}");
            total += 1;
        }
        let spec = &reference.data["controls_spec"];
        let source = reference
            .source
            .text(spec["file"].as_str().unwrap())
            .unwrap();
        let (old, new) = (
            spec["real_change"][0].as_str().unwrap(),
            spec["real_change"][1].as_str().unwrap(),
        );
        let comment = spec["comment_text"].as_str().unwrap();
        let texts = [
            ("identical", source.clone()),
            (
                "whitespace_only",
                format!("{}\n\n\n", source.trim_end_matches('\n')),
            ),
            ("comment_only", format!("{source}\n{comment}\n")),
            ("real_change", source.replacen(old, new, 1)),
            (
                "unparseable",
                source.replacen(old, &format!("{old} (((("), 1),
            ),
        ];
        let recorded = reference.data["mutant_controls"].as_array().unwrap();
        assert_eq!(recorded.len(), texts.len());
        for ((label, text), want) in texts.iter().zip(recorded) {
            let name = format!("{} mutant:{label}", reference.name);
            assert_eq!(want["label"], *label, "{name}");
            assert_eq!(want["sha256"], sha256_hex_bytes(text.as_bytes()), "{name}");
            let got = classify(pack, &source, text).unwrap().name();
            assert_eq!(want["got"], got, "{name}");
            assert_eq!(want["expected"], got, "{name}");
            total += 1;
        }
    }
    assert_eq!(total, 98);
}

#[test]
fn accepted_seeds_are_sites_with_a_real_change() {
    for reference in references() {
        let pack = embedded(reference.data["language"].as_str().unwrap()).unwrap();
        for control in reference.data["controls"].as_array().unwrap() {
            let label = control["label"].as_str().unwrap();
            let Some(kind) = label.strip_prefix("seed:") else {
                continue;
            };
            let site = site_of(&reference.source, pack, &control["annotations"][0]).unwrap();
            if kind.starts_with("pure_shell_boundary") {
                assert!(site.is_none(), "{} {label}", reference.name);
                continue;
            }
            let site = site.unwrap_or_else(|| panic!("{} {label}: no site", reference.name));
            let text = reference.source.raw(&site.file).unwrap();
            for mutant in mutants(pack, &text, &site).unwrap() {
                let change = classify(pack, &text, &mutant.text).unwrap();
                assert_eq!(
                    change.name(),
                    "changed",
                    "{} {label} {}",
                    reference.name,
                    mutant.operator
                );
            }
        }
    }
}

fn control_file(reference: &Reference, dir: &Path) -> PathBuf {
    let cases: Vec<Value> = reference.data["controls"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| json!({"label": c["label"], "annotations": c["annotations"], "expected": c["expected"]}))
        .collect();
    let file = json!({"files": reference.data["files"], "test_paths": reference.data["test_paths"], "cases": cases});
    let path = dir.join(format!("{}.controls.json", reference.name));
    fs::write(&path, file.to_string()).unwrap();
    path
}

fn admit_pack(
    pack: &Path,
    language: &str,
    controls: &Path,
    root: &Path,
    out: &Path,
) -> Option<i32> {
    let status = Command::new(env!("CARGO_BIN_EXE_lesscode-core"))
        .args(["deconvolute", "admit-pack", "--lang", language])
        .arg("--pack")
        .arg(pack)
        .arg("--controls")
        .arg(controls)
        .arg("--source-root")
        .arg(root)
        .arg("--out")
        .arg(out)
        .output()
        .expect("run lesscode-core binary")
        .status;
    status.code()
}

#[test]
fn admit_pack_replays_the_controls_of_every_scope() {
    let dir = std::env::temp_dir().join(format!("lc-test-parity-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let packs = Path::new(env!("CARGO_MANIFEST_DIR")).join("packs");
    for reference in references() {
        let language = reference.data["language"].as_str().unwrap();
        let controls = control_file(&reference, &dir);
        let pack = packs.join(format!("{language}.json"));
        let out = dir.join(format!("{}.receipt.json", reference.name));
        let code = admit_pack(&pack, language, &controls, &reference.source.root, &out);
        let receipt: Value = serde_json::from_slice(&fs::read(&out).unwrap()).unwrap();
        assert_eq!(code, Some(0), "{}: {receipt}", reference.name);
        assert_eq!(receipt["checks"].as_array().unwrap().len(), 7);

        // A pack that no longer recognises conditions fails the replay.
        let mut weak: Value = serde_json::from_slice(&fs::read(&pack).unwrap()).unwrap();
        weak["kinds"]["branch_condition"] = json!({});
        let weak_path = dir.join(format!("{language}-weak.json"));
        fs::write(&weak_path, weak.to_string()).unwrap();
        let code = admit_pack(
            &weak_path,
            language,
            &controls,
            &reference.source.root,
            &out,
        );
        let receipt: Value = serde_json::from_slice(&fs::read(&out).unwrap()).unwrap();
        assert_eq!(code, Some(1), "{}", reference.name);
        let failed: Vec<_> = receipt["checks"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["pass"] == false)
            .map(|c| c["name"].as_str().unwrap())
            .collect();
        assert!(
            failed.contains(&"controls"),
            "{}: {failed:?}",
            reference.name
        );
    }
    fs::remove_dir_all(&dir).unwrap();
}

/// The engine names no language, repository or gate file; only the registry
/// lists the shipped packs.
#[test]
fn the_engine_names_no_language_or_project() {
    let mut banned = vec![
        "mmhmem",
        "mvibe",
        "openultracode",
        "commons",
        "lexer",
        "ratelimit",
    ];
    let languages: Vec<String> = shipped_texts()
        .iter()
        .map(|t| {
            serde_json::from_str::<Value>(t).unwrap()["language"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    banned.extend(languages.iter().map(String::as_str));
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/adapters/driver");
    let mut scanned = 0;
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.file_name().is_some_and(|n| n == "registry.rs") {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap().to_lowercase();
        for word in text.split(|c: char| !c.is_alphanumeric()) {
            assert!(!banned.contains(&word), "{} names {word}", path.display());
        }
        scanned += 1;
    }
    assert!(scanned >= 10, "scanned {scanned} files");
    for text in shipped_texts() {
        let lower = text.to_lowercase();
        for word in &banned[..6] {
            assert!(!lower.contains(word), "a pack names {word}");
        }
    }
}
