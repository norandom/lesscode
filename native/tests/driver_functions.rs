//! Functions of a file and the limit of a run to named ones
//! (gradual-refactoring-steps, task 1.1).

use lesscode_core::adapters::driver::functions::{functions_of, select};
use lesscode_core::adapters::driver::registry::embedded;
use lesscode_core::adapters::driver::sites::eligible_sites;
use lesscode_core::adapters::driver::source::SourceSet;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const RUST: &str = "fn first(x: i32) -> bool {\n    if x > 1 {\n        return true;\n    }\n    false\n}\n\nstruct S;\n\nimpl S {\n    fn second(&self, y: i32) -> i32 {\n        let inner = |z: i32| z + 1;\n        if y > 2 {\n            return inner(y);\n        }\n        y\n    }\n}\n";
const PYTHON: &str = "def first(x):\n    if x > 1:\n        return True\n    return False\n\n\nclass S:\n    def second(self, y):\n        if y > 2:\n            return y + 1\n        return y\n";

fn tree(name: &str, file: &str, text: &str) -> SourceSet {
    let root: PathBuf =
        std::env::temp_dir().join(format!("lesscode-functions-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join(file), text).unwrap();
    SourceSet {
        root,
        files: vec![file.to_owned()],
        test_paths: Vec::new(),
    }
}

#[test]
fn functions_are_named_in_two_languages() {
    for (language, text) in [("rust", RUST), ("python", PYTHON)] {
        let pack = embedded(language).unwrap();
        let functions = functions_of(pack, "f", text).unwrap();
        let names: Vec<&str> = functions.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"first"), "{language}: {names:?}");
        assert!(names.contains(&"second"), "{language}: {names:?}");
        let first = functions.iter().find(|f| f.name == "first").unwrap();
        assert_eq!(first.start_line, 1, "{language}");
    }
}

#[test]
fn a_run_can_be_limited_to_named_functions() {
    for (language, file, text) in [("rust", "a.rs", RUST), ("python", "a.py", PYTHON)] {
        let pack = embedded(language).unwrap();
        let source = tree(language, file, text);
        let all = eligible_sites(&source, pack).unwrap();
        let none: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let (kept, left_out) = select(all.clone(), &none, &source, pack).unwrap();
        assert_eq!((kept.len(), left_out), (all.len(), None), "{language}");

        let of = |names: &[&str]| {
            let wanted = BTreeMap::from([(
                file.to_owned(),
                names.iter().map(|n| (*n).to_owned()).collect(),
            )]);
            select(all.clone(), &wanted, &source, pack)
        };
        let (first, out_first) = of(&["first"]).unwrap();
        let (second, out_second) = of(&["second"]).unwrap();
        assert!(!first.is_empty() && !second.is_empty(), "{language}");
        // Each site of the file lies in exactly one of the two functions.
        assert_eq!(first.len() + second.len(), all.len(), "{language}");
        assert_eq!(out_first, Some(second.len()), "{language}");
        assert_eq!(out_second, Some(first.len()), "{language}");
        assert!(first.iter().all(|s| s.start_line <= 6), "{language}");
        assert!(second.iter().all(|s| s.start_line >= 8), "{language}");
        // The kept sites are the same sites, unchanged and in the same order.
        let both: Vec<_> = all
            .iter()
            .filter(|s| first.contains(s) || second.contains(s))
            .cloned()
            .collect();
        assert_eq!(both, all, "{language}");

        let (nothing, out) = of(&[]).unwrap();
        assert_eq!((nothing.len(), out), (0, Some(all.len())), "{language}");
        let error = of(&["third"]).unwrap_err();
        assert!(
            error.contains("no function third") && error.contains("first"),
            "{error}"
        );
        let wrong_file = BTreeMap::from([("b.x".to_owned(), vec!["first".to_owned()])]);
        assert!(select(all.clone(), &wrong_file, &source, pack)
            .unwrap_err()
            .contains("not one of the files"));
        let _ = fs::remove_dir_all(&source.root);
    }
}
