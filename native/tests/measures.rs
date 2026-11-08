//! ScopeMeasurer Rust-substrate suite (task 8.3): ports the offline
//! synthetic-tree cases of `tests/test_measures_scope.py` — as-written
//! measures, pure/shell structure validation, statement-level pure share,
//! separated-form v1, canonical unavailability, edit scope, and measure
//! gaps — against the tree-sitter-python seam. Port deltas from the Python
//! suite: the custom `EffectProfile` cases do not port (the vocabulary is
//! fixed to the DEFAULT sqlalchemy profile), and the monkeypatched version
//! mismatch is covered by the real-formatter version pin because process
//! interception is not portable here.

use lesscode_core::measures::{self, Structure};
use lesscode_core::receipt::J;
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("lc-measures-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    root
}

fn v(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn tree(name: &str) -> PathBuf {
    let root = scratch(name);
    fs::create_dir_all(root.join("src/pkg")).unwrap();
    fs::write(
        root.join("src/pkg/mod_a.py"),
        "import os\n\ndef pure_fn(a, b):\n    x = a + b\n    return x\n\ndef shell_fn(conn):\n    conn.execute(\"select 1\")\n    return 2\n",
    )
    .unwrap();
    fs::write(
        root.join("src/pkg/mod_b.py"),
        "from pkg import mod_a\n\ndef caller():\n    return mod_a.pure_fn(1, 2)\n",
    )
    .unwrap();
    root
}

fn scope() -> Vec<String> {
    v(&["src/pkg/mod_a.py", "src/pkg/mod_b.py"])
}

fn structure() -> Structure {
    Structure {
        pure: vec!["mod_a.py::pure_fn".into()],
        shell: vec!["mod_a.py::shell_fn".into(), "mod_b.py::caller".into()],
    }
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

fn num_of(value: &J) -> i64 {
    match value {
        J::Int(n) => *n,
        _ => panic!("not an int"),
    }
}

fn bool_of(value: &J) -> bool {
    match value {
        J::Bool(b) => *b,
        _ => panic!("not a bool"),
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

fn measure(root: &Path, scope: &[&str]) -> J {
    measures::measure_written(root, &v(scope), None).unwrap()
}

// --- TestMeasureWritten ---------------------------------------------------

#[test]
fn written_counts() {
    let root = tree("w1");
    let report = measure(&root, &["src/pkg/mod_a.py", "src/pkg/mod_b.py"]);
    assert_eq!(str_of(get(&report, "effect_profile")), "sqlalchemy");
    let mod_a = get(get(&report, "files"), "mod_a.py");
    assert_eq!(num_of(get(mod_a, "lines")), 9);
    assert_eq!(num_of(get(mod_a, "stmts")), 7);
    assert_eq!(num_of(get(mod_a, "funcs")), 2);
    assert_eq!(num_of(get(mod_a, "qualnames")), 2);
    assert_eq!(num_of(get(mod_a, "writes")), 1);
    // mod_b: ImportFrom + FunctionDef + Return.
    assert_eq!(num_of(get(get(&report, "total"), "stmts")), 7 + 3);
}

#[test]
fn written_edges_and_module_hops() {
    let root = tree("w2");
    let report = measure(&root, &["src/pkg/mod_a.py", "src/pkg/mod_b.py"]);
    assert_eq!(num_of(get(get(&report, "total"), "edges")), 1);
    assert_eq!(num_of(get(get(&report, "total"), "module_hops")), 1);
    assert_eq!(
        items(get(&report, "edges")),
        &[J::arr(vec![
            J::str("mod_b.py"),
            J::str("caller"),
            J::str("mod_a.py"),
            J::str("pure_fn"),
        ])]
    );
}

#[test]
fn written_write_site_attribution() {
    let root = tree("w3");
    let report = measure(&root, &["src/pkg/mod_a.py", "src/pkg/mod_b.py"]);
    let site = &items(get(&report, "write_sites"))[0];
    assert_eq!(str_of(get(site, "file")), "mod_a.py");
    assert_eq!(str_of(get(site, "match")), ".execute(");
    assert_eq!(str_of(get(site, "in")), "mod_a.py::shell_fn");
}

#[test]
fn written_structure_pass() {
    let root = tree("w4");
    let report = measures::measure_written(&root, &scope(), Some(&structure())).unwrap();
    let structure = get(&report, "structure");
    assert!(bool_of(get(structure, "pass")));
    assert_eq!(items(get(structure, "undeclared")), &[]);
    let pure = pairs(get(structure, "pure"));
    assert!(bool_of(get(&pure[0].1, "pass")));
}

#[test]
fn written_bad_param_in_pure() {
    let root = tree("w5");
    let structure = Structure {
        pure: vec!["mod_a.py::shell_fn".into()],
        shell: vec![],
    };
    let report = measures::measure_written(&root, &scope(), Some(&structure)).unwrap();
    let pure = pairs(get(get(&report, "structure"), "pure"));
    let why = items(get(&pure[0].1, "why"));
    assert!(why.iter().any(|w| str_of(w).contains("param conn")));
}

#[test]
fn written_sql_attr_call_in_pure() {
    let root = scratch("w6");
    fs::create_dir_all(root.join("src/pkg")).unwrap();
    fs::write(
        root.join("src/pkg/mod_c.py"),
        "def sneaky():\n    return db.commit()\n",
    )
    .unwrap();
    let structure = Structure {
        pure: vec!["mod_c.py::sneaky".into()],
        shell: vec![],
    };
    let report =
        measures::measure_written(&root, &v(&["src/pkg/mod_c.py"]), Some(&structure)).unwrap();
    let pure = pairs(get(get(&report, "structure"), "pure"));
    let why = items(get(&pure[0].1, "why"));
    assert!(why.iter().any(|w| str_of(w).contains("call .commit")));
    assert!(!bool_of(get(get(&report, "structure"), "pass")));
}

#[test]
fn written_write_outside_shell() {
    let root = tree("w7");
    let structure = Structure {
        pure: vec!["mod_a.py::pure_fn".into()],
        shell: vec![],
    };
    let report =
        measures::measure_written(&root, &v(&["src/pkg/mod_a.py"]), Some(&structure)).unwrap();
    assert!(!bool_of(get(get(&report, "structure"), "pass")));
    assert!(items(get(get(&report, "structure"), "write_sites"))
        .iter()
        .any(|s| !bool_of(get(s, "pass"))));
}

#[test]
fn written_undeclared_and_unknown() {
    let root = tree("w8");
    let structure = Structure {
        pure: vec!["mod_a.py::pure_fn".into()],
        shell: vec!["mod_a.py::ghost".into()],
    };
    let report =
        measures::measure_written(&root, &v(&["src/pkg/mod_a.py"]), Some(&structure)).unwrap();
    let structure = get(&report, "structure");
    assert_eq!(
        items(get(structure, "undeclared")),
        &[J::str("mod_a.py::shell_fn")]
    );
    assert_eq!(
        items(get(structure, "declared_unknown")),
        &[J::str("mod_a.py::ghost")]
    );
}

#[test]
fn written_structure_normalizes_scope_path_declarations() {
    // Scope-path-qualified declarations name the same (file, qualname) pair
    // as the basename form (G1-R live finding, 2026-11-08).
    let root = tree("w9");
    let structure = Structure {
        pure: vec!["src/pkg/mod_a.py::pure_fn".into()],
        shell: vec!["src/pkg/mod_a.py::shell_fn".into()],
    };
    let report =
        measures::measure_written(&root, &v(&["src/pkg/mod_a.py"]), Some(&structure)).unwrap();
    let pure = pairs(get(get(&report, "structure"), "pure"));
    assert_eq!(pure[0].0, "mod_a.py::pure_fn");
    assert!(bool_of(get(&pure[0].1, "pass")));
}

#[test]
fn written_write_forms_match_the_reference_regex() {
    // Attr form wins at the same '(', bare `\b(func)\(` matches inside
    // attribute text too, and prefix names without '(' do not match
    // (add_all is only a write in the attr form).
    let root = scratch("w10");
    fs::create_dir_all(root.join("src/pkg")).unwrap();
    fs::write(
        root.join("src/pkg/one.py"),
        "db.delete(x)\ns.update(y)\nreinsert(z)\ns.add_all([q])\n",
    )
    .unwrap();
    let report = measure(&root, &["src/pkg/one.py"]);
    let sites: Vec<&str> = items(get(&report, "write_sites"))
        .iter()
        .map(|s| str_of(get(s, "match")))
        .collect();
    assert_eq!(sites, vec![".delete(", "update(", ".add_all("]);
    assert_eq!(num_of(get(get(&report, "total"), "writes")), 3);
}

// --- TestMeasurePureShare -------------------------------------------------

#[test]
fn pure_share_statement_level_attribution() {
    let root = tree("p1");
    let share = measures::measure_pure_share(&root, &scope(), &structure()).unwrap();
    let stmts = get(&share, "stmts");
    assert_eq!(num_of(get(stmts, "pure")), 3); // FunctionDef, Assign, Return of pure_fn
    assert_eq!(num_of(get(stmts, "shell")), 5); // shell_fn x3 + caller FunctionDef + Return
    assert_eq!(num_of(get(stmts, "undeclared")), 0);
    assert_eq!(num_of(get(stmts, "outside_defs")), 2); // the two imports
    assert_eq!(num_of(get(&share, "stmts_in_defs")), 8);
    assert_eq!(get(&share, "pure_share_by_stmts_in_defs"), &J::Float(0.375));
    assert_eq!(get(&share, "pure_share_by_stmts_all"), &J::Float(0.3));
    assert_eq!(
        get(get(&share, "per_function"), "mod_a.py::pure_fn"),
        &J::int(3)
    );
}

#[test]
fn pure_share_undeclared_statements_counted() {
    let root = tree("p2");
    let structure = Structure {
        pure: vec!["mod_a.py::pure_fn".into()],
        shell: vec![],
    };
    let share = measures::measure_pure_share(&root, &v(&["src/pkg/mod_a.py"]), &structure).unwrap();
    assert_eq!(num_of(get(get(&share, "stmts"), "undeclared")), 3);
}

#[test]
fn pure_share_function_share() {
    let root = tree("p3");
    let share =
        measures::measure_pure_share(&root, &v(&["src/pkg/mod_a.py"]), &structure()).unwrap();
    assert_eq!(get(&share, "pure_share_by_funcs"), &J::Float(0.5));
}

#[test]
fn pure_share_rounding_is_half_even_like_python() {
    // 1 pure of 16 functions: round(1/16, 3) = round(0.0625, 3) = 0.062
    // under CPython's half-even decimal rounding.
    let root = scratch("p4");
    fs::create_dir_all(root.join("src")).unwrap();
    let mut text = String::new();
    for i in 0..16 {
        text.push_str(&format!("def f{i}():\n    pass\n"));
    }
    fs::write(root.join("src/one.py"), text).unwrap();
    let structure = Structure {
        pure: vec!["one.py::f0".into()],
        shell: vec![],
    };
    let share = measures::measure_pure_share(&root, &v(&["src/one.py"]), &structure).unwrap();
    assert_eq!(get(&share, "pure_share_by_funcs"), &J::Float(0.062));
}

// --- TestMeasureSeparated -------------------------------------------------

type RuleCase = (&'static str, &'static [(&'static str, i64)], i64);

const RULE_CASES: &[RuleCase] = &[
    ("import a, b, c\n", &[("import", 2)], 3), // rule 2
    ("from m import x, y\n", &[("from_import", 1)], 2), // rule 3
    ("from m import *\n", &[], 1),             // rule 3 star
    ("[a, (b, c)] = 1, (2, 3)\n", &[("tuple_assign", 2)], 3), // rule 4 recursive pairing
    ("a = b = v\n", &[("multi_target", 1)], 2), // rule 5
    ("ok = 0 < x < 1\n", &[("chained_compare", 1)], 2), // rule 6
    ("del a, b\n", &[("del", 1)], 2),          // rule 7
    ("global a, b\n", &[("global_nonlocal", 1)], 2), // rule 7
    (
        "with open(p) as f, closing(f) as c:\n    pass\n",
        &[("with_items", 1)],
        3,
    ), // rule 8: base 2 + extra 1
    ("a, b = f()\n", &[], 1),                  // rule 9: non-literal value not unpacked
    ("a, *b = xs\n", &[], 1),                  // rule 9: starred target not unpacked
    ("x += 1\n", &[], 1),                      // rule 9: augmented assignment
];

#[test]
fn separated_rule_table() {
    for (case, (text, extra, stmts)) in RULE_CASES.iter().enumerate() {
        let root = scratch(&format!("s{case}"));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/one.py"), text).unwrap();
        let report = measures::measure_separated(&root, &v(&["src/one.py"])).unwrap();
        let file_report = get(get(&report, "files"), "one.py");
        for (key, value) in pairs(get(file_report, "extra")) {
            let expected = extra
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| *v)
                .unwrap_or(0);
            assert_eq!(num_of(value), expected, "case {case} key {key}");
        }
        assert_eq!(num_of(get(file_report, "stmts")), *stmts, "case {case}");
    }
}

#[test]
fn separated_identity_against_written() {
    let root = tree("s13");
    let scope = ["src/pkg/mod_a.py", "src/pkg/mod_b.py"];
    let written = measure(&root, &scope);
    let separated = measures::measure_separated(&root, &v(&scope)).unwrap();
    for (name, file_report) in pairs(get(&separated, "files")) {
        let extra: i64 = pairs(get(file_report, "extra"))
            .iter()
            .map(|(_, v)| num_of(v))
            .sum();
        assert_eq!(
            num_of(get(file_report, "stmts")) - extra,
            num_of(get(get(get(&written, "files"), name), "stmts")),
            "{name}"
        );
    }
    let total: i64 = pairs(get(&separated, "files"))
        .iter()
        .map(|(_, v)| num_of(get(v, "stmts")))
        .sum();
    assert_eq!(num_of(get(get(&separated, "total"), "stmts")), total);
}

#[test]
fn separated_rule_version_recorded() {
    let root = tree("s14");
    let report = measures::measure_separated(&root, &scope()).unwrap();
    assert_eq!(str_of(get(get(&report, "rule"), "version")), "1");
}

// --- TestMeasureCanonical -------------------------------------------------

#[test]
fn canonical_unavailable_formatter_returns_none() {
    // cargo-nextest runs each test in its own process, so narrowing PATH
    // here cannot leak into the real-formatter test.
    let root = tree("c1");
    let empty = scratch("c1-empty");
    fs::create_dir_all(&empty).unwrap();
    let saved = std::env::var_os("PATH").unwrap();
    std::env::set_var("PATH", &empty);
    let report = measures::measure_canonical(&root, &scope(), &root.join("fmt")).unwrap();
    std::env::set_var("PATH", &saved);
    assert!(report.is_none());
}

fn uv_on_path() -> bool {
    std::env::var_os("PATH")
        .map(|paths| {
            std::env::split_paths(&paths)
                .any(|dir| ["uv.exe", "uv"].iter().any(|name| dir.join(name).is_file()))
        })
        .unwrap_or(false)
}

#[test]
fn canonical_real_formatter_pins_version() {
    let root = tree("c2");
    if !uv_on_path() {
        eprintln!("skipping: uv tool runner unavailable");
        return;
    }
    let fmt_dir = scratch("c2-fmt");
    let report = measures::measure_canonical(&root, &scope(), &fmt_dir)
        .unwrap()
        .expect("formatter available");
    assert_eq!(str_of(get(get(&report, "formatter"), "version")), "0.17.3");
    let written = measure(&root, &["src/pkg/mod_a.py", "src/pkg/mod_b.py"]);
    // Formatting never changes stmts; blank-line normalization only adds lines.
    assert_eq!(
        num_of(get(get(&report, "total"), "stmts")),
        num_of(get(get(&written, "total"), "stmts"))
    );
    assert!(
        num_of(get(get(&report, "total"), "lines")) >= num_of(get(get(&written, "total"), "lines"))
    );
}

// --- TestCheckEditScope ---------------------------------------------------

fn scope_trees(name: &str) -> (PathBuf, PathBuf) {
    let original = scratch(&format!("{name}-original"));
    let candidate = scratch(&format!("{name}-candidate"));
    for tree in [&original, &candidate] {
        fs::create_dir_all(tree.join("src")).unwrap();
        fs::write(tree.join("src/a.py"), "x = 1\n").unwrap();
        fs::write(tree.join("src/b.py"), "y = 2\n").unwrap();
    }
    fs::write(candidate.join("src/a.py"), "x = 3\n").unwrap();
    (original, candidate)
}

fn edit_scope_full(
    original: &Path,
    candidate: &Path,
    scope: &[&str],
    mentions: &[&str],
    extensions: &[&str],
) -> J {
    measures::check_edit_scope(
        original,
        candidate,
        &v(scope),
        &v(&["src", "tests", "packages"]),
        &mentions.iter().map(PathBuf::from).collect::<Vec<_>>(),
        &v(extensions),
        &v(&["node_modules", "target"]),
    )
    .unwrap()
}

fn edit_scope(original: &Path, candidate: &Path, scope: &[&str]) -> J {
    edit_scope_full(original, candidate, scope, &[], &[".py"])
}

fn log(sibling: &Path, text: &str) -> PathBuf {
    // Unique per test process: nextest runs one process per test, and a
    // fixed name would race across the mention tests.
    let path = sibling.with_extension("log");
    fs::write(&path, text).unwrap();
    path
}

#[test]
fn edit_scope_scope_only_change_passes() {
    let (original, candidate) = scope_trees("e1");
    let report = edit_scope(&original, &candidate, &["src/a.py"]);
    assert_eq!(str_of(get(&report, "edit_scope")), "pass");
    assert_eq!(str_of(get(&report, "verdict")), "pass");
    assert_eq!(items(get(&report, "differing")), &[J::str("src/a.py")]);
    assert_eq!(items(get(&report, "differing_outside_scope")), &[]);
}

#[test]
fn edit_scope_outside_scope_change_fails() {
    let (original, candidate) = scope_trees("e2");
    fs::write(candidate.join("src/b.py"), "y = 3\n").unwrap();
    let report = edit_scope(&original, &candidate, &["src/a.py"]);
    assert_eq!(str_of(get(&report, "edit_scope")), "fail");
    assert_eq!(
        items(get(&report, "differing_outside_scope")),
        &[J::str("src/b.py")]
    );
}

#[test]
fn edit_scope_added_file_fails() {
    let (original, candidate) = scope_trees("e3");
    fs::write(candidate.join("src/c.py"), "z = 9\n").unwrap();
    let report = edit_scope(&original, &candidate, &["src/a.py"]);
    assert_eq!(
        items(get(&report, "only_in_candidate")),
        &[J::str("src/c.py")]
    );
    assert_eq!(str_of(get(&report, "edit_scope")), "fail");
}

#[test]
fn edit_scope_scope_file_unchanged_is_reported() {
    let (original, candidate) = scope_trees("e4");
    fs::write(candidate.join("src/a.py"), "x = 1\n").unwrap();
    let report = edit_scope(&original, &candidate, &["src/a.py"]);
    assert_eq!(
        items(get(&report, "scope_unchanged")),
        &[J::str("src/a.py")]
    );
    assert_eq!(str_of(get(&report, "verdict")), "fail"); // nothing changed at all
}

#[test]
fn edit_scope_mentions_outside_scope() {
    let (original, candidate) = scope_trees("e5");
    let log = log(&original, "rewrote src/b.py and tools/helper.py today\n");
    let report = edit_scope_full(
        &original,
        &candidate,
        &["src/a.py"],
        &[log.to_string_lossy().as_ref()],
        &[".py"],
    );
    assert_eq!(
        items(get(get(&report, "mentions"), "outside_scope")),
        &[J::str("src/b.py"), J::str("tools/helper.py")]
    );
}

#[test]
fn edit_scope_mentions_in_scope_ignored() {
    let (original, candidate) = scope_trees("e6");
    let log = log(&original, "rewrote src/a.py\n");
    let report = edit_scope_full(
        &original,
        &candidate,
        &["src/a.py"],
        &[log.to_string_lossy().as_ref()],
        &[".py"],
    );
    assert_eq!(items(get(get(&report, "mentions"), "outside_scope")), &[]);
}

#[test]
fn edit_scope_source_extensions_parameterized() {
    let (original, candidate) = scope_trees("e7");
    fs::write(original.join("src/core.rs"), "fn main() {}\n").unwrap();
    fs::write(candidate.join("src/core.rs"), "fn main() { }\n").unwrap();
    let log = log(&original, "also touched src/other.rs\n");
    let report = edit_scope_full(
        &original,
        &candidate,
        &["src/a.py", "src/core.rs"],
        &[log.to_string_lossy().as_ref()],
        &[".py", ".rs"],
    );
    assert_eq!(str_of(get(&report, "edit_scope")), "pass");
    assert_eq!(
        items(get(get(&report, "mentions"), "outside_scope")),
        &[J::str("src/other.rs")]
    );
}

#[test]
fn edit_scope_ignore_dirs_skipped() {
    let (original, candidate) = scope_trees("e8");
    for tree in [&original, &candidate] {
        fs::create_dir_all(tree.join("node_modules")).unwrap();
        fs::write(tree.join("node_modules/junk.py"), "junk\n").unwrap();
    }
    fs::write(candidate.join("node_modules/junk.py"), "changed junk\n").unwrap();
    let report = edit_scope(&original, &candidate, &["src/a.py"]);
    assert_eq!(str_of(get(&report, "edit_scope")), "pass"); // differences are invisible
}

// --- TestComputeMeasureGaps -----------------------------------------------

fn readings(root: &Path) -> (J, Option<J>, J) {
    let written = measure(root, &["src/pkg/mod_a.py", "src/pkg/mod_b.py"]);
    let separated = measures::measure_separated(root, &scope()).unwrap();
    (written, None, separated)
}

#[test]
fn gaps_all_targets_met() {
    let root = tree("g1");
    let (written, canonical, separated) = readings(&root);
    let targets = vec![
        (
            "written_stmts_max".to_string(),
            num_of(get(get(&written, "total"), "stmts")) + 5,
        ),
        (
            "separated_stmts_max".to_string(),
            num_of(get(get(&separated, "total"), "stmts")) + 5,
        ),
        (
            "edges_max".to_string(),
            num_of(get(get(&written, "total"), "edges")) + 1,
        ),
    ];
    assert_eq!(
        measures::compute_measure_gaps(&written, canonical.as_ref(), &separated, &targets),
        J::Obj(vec![])
    );
}

#[test]
fn gaps_exceeded_target_reported() {
    let root = tree("g2");
    let (written, canonical, separated) = readings(&root);
    let targets = vec![(
        "written_stmts_max".to_string(),
        num_of(get(get(&written, "total"), "stmts")) - 1,
    )];
    let gaps = measures::compute_measure_gaps(&written, canonical.as_ref(), &separated, &targets);
    let gap = get(&gaps, "written_stmts_max");
    assert_eq!(num_of(get(gap, "excess")), 1);
    assert_eq!(
        num_of(get(gap, "actual")),
        num_of(get(get(&written, "total"), "stmts"))
    );
}

#[test]
fn gaps_canonical_unavailable_is_gap_never_zero() {
    let root = tree("g3");
    let (written, _, separated) = readings(&root);
    let gaps = measures::compute_measure_gaps(
        &written,
        None,
        &separated,
        &[("canonical_lines_max".to_string(), 100)],
    );
    let gap = get(&gaps, "canonical_lines_max");
    assert_eq!(get(gap, "actual"), &J::Null);
    assert_eq!(get(gap, "unavailable"), &J::Bool(true));
}

#[test]
fn gaps_exact_meets_target() {
    let root = tree("g4");
    let (written, canonical, separated) = readings(&root);
    let targets = vec![(
        "written_stmts_max".to_string(),
        num_of(get(get(&written, "total"), "stmts")),
    )];
    assert_eq!(
        measures::compute_measure_gaps(&written, canonical.as_ref(), &separated, &targets),
        J::Obj(vec![])
    );
}

// --- TestMeasureAll -------------------------------------------------------

#[test]
fn measure_all_composition() {
    let root = tree("m1");
    let fmt_dir = scratch("m1-fmt");
    let report = measures::measure_all(&root, &scope(), Some(&fmt_dir), None, &[]).unwrap();
    let written = measure(&root, &["src/pkg/mod_a.py", "src/pkg/mod_b.py"]);
    let separated = measures::measure_separated(&root, &scope()).unwrap();
    assert_eq!(
        get(get(&report, "as_written"), "total"),
        get(&written, "total")
    );
    assert_eq!(
        get(get(&report, "separated"), "total"),
        get(&separated, "total")
    );
    assert!(matches!(get(&report, "canonical"), J::Null | J::Obj(_)));
    assert_eq!(get(&report, "gaps"), &J::Obj(vec![]));
}

#[test]
fn measure_all_targets_flow_through() {
    let root = tree("m2");
    let fmt_dir = scratch("m2-fmt");
    let report = measures::measure_all(
        &root,
        &scope(),
        Some(&fmt_dir),
        None,
        &[("edges_max".to_string(), 0)],
    )
    .unwrap();
    assert!(
        matches!(get(&report, "gaps"), J::Obj(pairs) if pairs.iter().any(|(k, _)| k == "edges_max"))
    );
}
