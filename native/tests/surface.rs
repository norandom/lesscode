//! SurfaceDeriver Rust-substrate suite (task 8.2): ports the offline
//! synthetic-tree cases of `tests/test_surface.py` — Contract A vs Contract B,
//! unresolved-vs-missing semantics, check_syntax line anchoring, candidate
//! verification violation classes — against the tree-sitter-python seam.

use lesscode_core::receipt::J;
use lesscode_core::surface;
use std::fs;
use std::path::{Path, PathBuf};

const MOD: &str = "pkg.mod";
const SCOPE: [&str; 1] = ["src/pkg/mod.py"];

fn scratch(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("lc-surface-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("src/pkg")).unwrap();
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::create_dir_all(root.join("tools")).unwrap();
    root
}

/// A minimal specimen: one scope module, one importer, one test, one recorder.
fn tree(name: &str) -> PathBuf {
    let root = scratch(name);
    fs::write(root.join("src/pkg/__init__.py"), "").unwrap();
    fs::write(
        root.join("src/pkg/mod.py"),
        "def used_fn() -> int:\n    return 1\n\nclass Used:\n    pass\n\nUSED_CONST = 7\n\ndef unused_fn() -> int:\n    return 2\n",
    )
    .unwrap();
    fs::write(
        root.join("src/pkg/other.py"),
        "from pkg.mod import used_fn, USED_CONST\nimport pkg.mod as pm\n\npm.Used()\npm.ghost_attr()\nsetattr(pm, \"patched_target\", 1)\n",
    )
    .unwrap();
    fs::write(
        root.join("tests/test_mod.py"),
        "from pkg.mod import used_fn\n\ndef test_it():\n    assert used_fn() == 1\n",
    )
    .unwrap();
    fs::write(
        root.join("tools/recorder.py"),
        "from pkg.mod import labeled_reason\n\ndef rec():\n    return labeled_reason\n",
    )
    .unwrap();
    root
}

fn get<'a>(value: &'a J, key: &str) -> &'a J {
    match value {
        J::Obj(pairs) => &pairs.iter().find(|(k, _)| k == key).expect("key").1,
        _ => panic!("not an object"),
    }
}

fn str_of(value: &J) -> &str {
    match value {
        J::Str(s) => s,
        _ => panic!("not a string"),
    }
}

fn items(value: &J) -> &[J] {
    match value {
        J::Arr(items) => items,
        _ => panic!("not an array"),
    }
}

fn pairs(value: &J) -> &[(String, J)] {
    match value {
        J::Obj(pairs) => pairs,
        _ => panic!("not an object"),
    }
}

fn module_names(report: &J) -> &[(String, J)] {
    pairs(get(get(get(report, "modules"), MOD), "names"))
}

fn derive(root: &Path, scope: &[&str], recorders: &[&str]) -> J {
    surface::derive_surface(
        root,
        &scope.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        &recorders.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        &[],
    )
    .unwrap()
}

#[test]
fn contract_a_kinds_and_tags() {
    let root = tree("a");
    let report = derive(&root, &SCOPE, &[]);
    let names = module_names(&report);
    let entry = |name: &str| &names.iter().find(|(n, _)| n == name).expect(name).1;
    assert_eq!(str_of(get(entry("used_fn"), "kind")), "function");
    assert_eq!(str_of(get(entry("Used"), "kind")), "class");
    assert_eq!(str_of(get(entry("USED_CONST"), "kind")), "constant");
    assert_eq!(
        items(get(entry("used_fn"), "tags")),
        &[J::str("prod"), J::str("test")]
    );
    assert_eq!(items(get(entry("used_fn"), "via")), &[J::str("import")]);
    assert_eq!(items(get(entry("Used"), "via")), &[J::str("attribute")]);
    assert_eq!(
        items(get(entry("patched_target"), "via")),
        &[J::str("patched")]
    );
}

#[test]
fn contract_a_files_scanned_and_importers() {
    let root = tree("a2");
    let report = derive(&root, &SCOPE, &[]);
    // __init__, mod, other (src) + test_mod (tests); tools/ is not a scan dir.
    assert!(matches!(get(&report, "files_scanned"), J::Int(n) if *n == 4));
    let importers = pairs(get(get(get(&report, "modules"), MOD), "importers"));
    let by_rel = |rel: &str| str_of(&importers.iter().find(|(r, _)| r == rel).expect(rel).1);
    assert_eq!(by_rel("src/pkg/other.py"), "prod");
    assert_eq!(by_rel("tests/test_mod.py"), "test");
}

#[test]
fn unresolved_records_missing_attribute_reads() {
    let root = tree("a3");
    let report = derive(&root, &SCOPE, &[]);
    let unresolved = pairs(get(get(&report, "unresolved"), MOD));
    let ghost = &unresolved
        .iter()
        .find(|(n, _)| n == "ghost_attr")
        .expect("ghost_attr")
        .1;
    assert_eq!(items(ghost), &[J::str("src/pkg/other.py")]);
}

#[test]
fn recorder_dir_not_scanned_by_default() {
    let root = tree("a4");
    let report = derive(&root, &SCOPE, &[]);
    assert!(module_names(&report)
        .iter()
        .all(|(n, _)| n != "labeled_reason"));
}

#[test]
fn extractor_version_recorded() {
    let root = tree("a5");
    let report = derive(&root, &SCOPE, &[]);
    assert_eq!(
        str_of(get(&report, "extractor_version")),
        surface::EXTRACTOR_VERSION
    );
}

#[test]
fn recorder_import_joins_contract() {
    let root = tree("b");
    let report = derive(&root, &SCOPE, &["tools/recorder.py"]);
    let entry = &module_names(&report)
        .iter()
        .find(|(n, _)| n == "labeled_reason")
        .expect("labeled_reason")
        .1;
    assert_eq!(str_of(get(entry, "kind")), "missing");
    assert!(items(get(entry, "tags"))
        .iter()
        .any(|t| str_of(t) == "recorder"));
    assert_eq!(
        items(get(entry, "importers")),
        &[J::str("tools/recorder.py")]
    );
}

#[test]
fn recorder_files_listed() {
    let root = tree("b2");
    let report = derive(&root, &SCOPE, &["tools/recorder.py"]);
    assert_eq!(
        items(get(&report, "recorder_files")),
        &[J::str("tools/recorder.py")]
    );
}

#[test]
fn intra_set_reads_among_scope_files() {
    let root = tree("b3");
    fs::write(
        root.join("src/pkg/mod2.py"),
        "from pkg.mod import used_fn\n",
    )
    .unwrap();
    let scope = ["src/pkg/mod.py", "src/pkg/mod2.py"];
    let report = derive(&root, &scope, &[]);
    let intra = pairs(get(get(&report, "intra_set"), MOD));
    assert!(intra.iter().any(|(n, _)| n == "used_fn"));
}

#[test]
fn check_syntax_clean_tree() {
    let root = tree("c");
    assert!(surface::check_syntax(&root, &["src/pkg/mod.py".to_string()], "python").is_empty());
}

#[test]
fn check_syntax_line_anchored_errors() {
    let root = tree("c2");
    fs::write(root.join("src/pkg/mod.py"), "def f(:\n    pass\n").unwrap();
    let errors = surface::check_syntax(&root, &["src/pkg/mod.py".to_string()], "python");
    assert_eq!(errors.len(), 1);
    assert!(errors[0].starts_with("src/pkg/mod.py:1:"), "{}", errors[0]);
}

#[test]
fn check_syntax_multiple_files() {
    let root = tree("c3");
    fs::write(root.join("src/pkg/mod.py"), "def f(:\n").unwrap();
    fs::write(root.join("src/pkg/mod2.py"), "def g(:\n").unwrap();
    let errors = surface::check_syntax(
        &root,
        &["src/pkg/mod.py".to_string(), "src/pkg/mod2.py".to_string()],
        "python",
    );
    assert_eq!(errors.len(), 2);
}

#[test]
fn check_syntax_unsupported_language_is_explicit() {
    let root = tree("c4");
    let errors = surface::check_syntax(&root, &["src/pkg/mod.py".to_string()], "typescript");
    assert_eq!(errors.len(), 1);
    assert!(errors[0].starts_with("check_syntax: unsupported language"));
    assert!(errors[0].contains("typescript"));
}

#[test]
fn check_syntax_missing_file_reported() {
    let root = tree("c5");
    let errors = surface::check_syntax(&root, &["src/pkg/absent.py".to_string()], "python");
    assert!(errors.iter().any(|e| e.contains("src/pkg/absent.py")));
}

#[test]
fn check_syntax_never_imports_candidate() {
    let root = tree("c6");
    fs::write(
        root.join("src/pkg/mod.py"),
        "import os\nraise SystemExit('must not execute')\n",
    )
    .unwrap();
    assert!(surface::check_syntax(&root, &["src/pkg/mod.py".to_string()], "python").is_empty());
}

/// The verification fixture: mod.py additionally defines patched_target.
fn report_with_patched(root: &Path) -> J {
    fs::write(
        root.join("src/pkg/mod.py"),
        "def used_fn() -> int:\n    return 1\n\nclass Used:\n    pass\n\nUSED_CONST = 7\n\npatched_target = 0\n\ndef unused_fn() -> int:\n    return 2\n",
    )
    .unwrap();
    derive(root, &SCOPE, &[])
}

fn candidate(root: &Path, mod_text: &str) -> PathBuf {
    fs::create_dir_all(root.join("cand/src/pkg")).unwrap();
    fs::write(root.join("cand/src/pkg/mod.py"), mod_text).unwrap();
    root.join("cand")
}

const FULL: &str = "def used_fn():\n    return 1\n\n\nclass Used:\n    pass\n\n\nUSED_CONST = 7\n\n\npatched_target = 0\n\n\ndef unused_fn():\n    return 2\n";

#[test]
fn verify_clean_candidate_passes() {
    let root = tree("v");
    let report = report_with_patched(&root);
    let cand = candidate(&root, FULL);
    let verification = surface::verify_candidate_surface(&cand, &report, &[]);
    assert!(matches!(get(&verification, "passed"), J::Bool(true)));
    assert!(items(get(&verification, "missing_symbols")).is_empty());
    assert!(items(get(&verification, "kind_mismatches")).is_empty());
    assert!(items(get(&verification, "local_rebindings")).is_empty());
}

#[test]
fn verify_missing_symbol_reported() {
    let root = tree("v2");
    let report = report_with_patched(&root);
    let cand = candidate(
        &root,
        "class Used:\n    pass\n\n\nUSED_CONST = 7\n\n\npatched_target = 0\n\n\ndef unused_fn():\n    return 2\n",
    );
    let verification = surface::verify_candidate_surface(&cand, &report, &[]);
    assert!(matches!(get(&verification, "passed"), J::Bool(false)));
    assert_eq!(
        items(get(&verification, "missing_symbols")),
        &[J::arr(vec![J::str(MOD), J::str("used_fn")])]
    );
    assert!(items(get(&verification, "kind_mismatches")).is_empty());
}

#[test]
fn verify_kind_mismatch_reported() {
    let root = tree("v3");
    let report = report_with_patched(&root);
    let cand = candidate(
        &root,
        "used_fn = 1\n\n\nclass Used:\n    pass\n\n\nUSED_CONST = 7\n\n\npatched_target = 0\n\n\ndef unused_fn():\n    return 2\n",
    );
    let verification = surface::verify_candidate_surface(&cand, &report, &[]);
    let mismatches = items(get(&verification, "kind_mismatches"));
    assert_eq!(mismatches.len(), 1);
    assert_eq!(str_of(get(&mismatches[0], "module")), MOD);
    assert_eq!(str_of(get(&mismatches[0], "name")), "used_fn");
    assert_eq!(str_of(get(&mismatches[0], "expected")), "function");
    assert_eq!(str_of(get(&mismatches[0], "actual")), "constant");
}

#[test]
fn verify_local_rebinding_by_parameter() {
    let root = tree("v4");
    let report = report_with_patched(&root);
    let cand = candidate(
        &root,
        "def used_fn(patched_target=1):\n    return patched_target\n\n\nclass Used:\n    pass\n\n\nUSED_CONST = 7\n\n\npatched_target = 0\n\n\ndef unused_fn():\n    return 2\n",
    );
    let verification = surface::verify_candidate_surface(&cand, &report, &[]);
    let findings = items(get(&verification, "local_rebindings"));
    assert_eq!(str_of(get(&findings[0], "name")), "patched_target");
    assert_eq!(str_of(get(&findings[0], "how")), "parameter");
}

#[test]
fn verify_local_rebinding_by_assignment() {
    let root = tree("v5");
    let report = report_with_patched(&root);
    let cand = candidate(
        &root,
        "def used_fn():\n    patched_target = 5\n    return patched_target\n\n\nclass Used:\n    pass\n\n\nUSED_CONST = 7\n\n\npatched_target = 0\n\n\ndef unused_fn():\n    return 2\n",
    );
    let verification = surface::verify_candidate_surface(&cand, &report, &[]);
    assert_eq!(
        str_of(get(
            &items(get(&verification, "local_rebindings"))[0],
            "how"
        )),
        "assignment"
    );
}

#[test]
fn verify_extra_patched_names_checked() {
    let root = tree("v6");
    let report = report_with_patched(&root);
    let cand = candidate(
        &root,
        "def used_fn(USED_CONST=3):\n    return USED_CONST\n\n\nclass Used:\n    pass\n\n\nUSED_CONST = 7\n\n\npatched_target = 0\n\n\ndef unused_fn():\n    return 2\n",
    );
    let verification =
        surface::verify_candidate_surface(&cand, &report, &["USED_CONST".to_string()]);
    assert_eq!(
        str_of(get(
            &items(get(&verification, "local_rebindings"))[0],
            "name"
        )),
        "USED_CONST"
    );
}

#[test]
fn verify_violations_carry_locations() {
    let root = tree("v7");
    let report = report_with_patched(&root);
    let cand = candidate(
        &root,
        "def used_fn(patched_target=1):\n    return patched_target\n\n\nclass Used:\n    pass\n\n\nUSED_CONST = 7\n\n\npatched_target = 0\n\n\ndef unused_fn():\n    return 2\n",
    );
    let verification = surface::verify_candidate_surface(&cand, &report, &[]);
    let finding = &items(get(&verification, "local_rebindings"))[0];
    assert!(matches!(get(finding, "line"), J::Int(n) if *n == 1));
    assert_eq!(str_of(get(finding, "function")), "used_fn");
}
