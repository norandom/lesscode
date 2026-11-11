//! Changed functions, their callers and the function a fault belongs to
//! (gradual-refactoring-steps, task 3.2).

use lesscode_core::adapters::driver::registry::embedded;
use lesscode_core::step::functions::{changed, FileFunctions, OUTSIDE};
use std::collections::BTreeSet;

fn set(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|n| (*n).to_owned()).collect()
}

const RUST_BASE: &str = "const LIMIT: i32 = 3;\n\nfn check(x: i32) -> bool {\n    if x > LIMIT {\n        return true;\n    }\n    false\n}\n\nfn caller(x: i32) -> i32 {\n    if check(x) { 1 } else { 0 }\n}\n\nfn bystander(x: i32) -> i32 {\n    let check = x + 1;\n    check\n}\n\nfn removed() {}\n";
// `check` is rewritten, `caller` only reformatted and commented, `removed`
// is gone and `added` is new.
const RUST_STEP: &str = "const LIMIT: i32 = 3;\n\nfn check(x: i32) -> bool {\n    x > LIMIT\n}\n\n// a comment\nfn caller(x: i32) -> i32 {\n    if check(x) {\n        1\n    } else {\n        0\n    }\n}\n\nfn bystander(x: i32) -> i32 {\n    let check = x + 1;\n    check\n}\n\nfn added() -> i32 {\n    caller(1)\n}\n";

#[test]
fn changed_functions_and_their_callers_are_found_by_name() {
    let pack = embedded("rust").unwrap();
    let base = FileFunctions::read(pack, "a.rs", RUST_BASE).unwrap();
    let step = FileFunctions::read(pack, "a.rs", RUST_STEP).unwrap();

    // Layout and comments do not make a function changed.
    let names = changed(&base, &step);
    assert_eq!(names, set(&["added", "check", "removed"]));
    assert!(changed(&base, &base).is_empty());

    // `caller` calls `check`; `bystander` only has a variable of that name;
    // a changed function is not listed as its own caller.
    assert_eq!(step.callers_of(&names), set(&["caller"]));
    assert_eq!(base.callers_of(&set(&["check"])), set(&["caller"]));
    assert_eq!(step.callers_of(&set(&["caller"])), set(&["added"]));
    assert!(step.callers_of(&set(&["nothing"])).is_empty());

    let sizes = step.sizes();
    assert!(sizes["check"] < base.sizes()["check"]);
    assert_eq!(sizes["bystander"], base.sizes()["bystander"]);
}

#[test]
fn faults_are_counted_for_the_function_around_them() {
    let pack = embedded("rust").unwrap();
    let base = FileFunctions::read(pack, "a.rs", RUST_BASE).unwrap();
    assert_eq!(base.owner((4, 7)), "check");
    assert_eq!(base.owner((11, 7)), "caller");
    assert_eq!(base.owner((1, 0)), OUTSIDE);
    let faults = [
        ((4, 7), "killed"),
        ((5, 8), "survived"),
        ((7, 4), "killed"),
        ((11, 7), "timeout"),
        ((1, 20), "survived"),
    ];
    let table = base.tally(faults);
    let row = |name: &str| {
        (
            table[name].noticed,
            table[name].unnoticed,
            table[name].other,
        )
    };
    assert_eq!(row("check"), (2, 1, 0));
    assert_eq!(row("caller"), (0, 0, 1));
    assert_eq!(row(OUTSIDE), (0, 1, 0));
    assert_eq!(table.len(), 3);
}

/// The same holds where functions are methods and where a function is a
/// value bound to a name.
#[test]
fn methods_and_bound_functions_have_names_in_other_languages() {
    let python = "class S:\n    def check(self, x):\n        return x > 3\n\n    def caller(self, x):\n        return 1 if self.check(x) else 0\n";
    let pack = embedded("python").unwrap();
    let file = FileFunctions::read(pack, "a.py", python).unwrap();
    assert_eq!(file.callers_of(&set(&["check"])), set(&["caller"]));
    assert_eq!(file.owner((3, 15)), "check");

    let typescript = "export const check = (x: number): boolean => {\n  return x > 3;\n};\n\nexport function caller(x: number): number {\n  return check(x) ? 1 : 0;\n}\n";
    let pack = embedded("typescript").unwrap();
    let file = FileFunctions::read(pack, "a.ts", typescript).unwrap();
    let names: Vec<&str> = file.functions.iter().map(|f| f.name.as_str()).collect();
    assert!(
        names.contains(&"check") && names.contains(&"caller"),
        "{names:?}"
    );
    assert_eq!(file.callers_of(&set(&["check"])), set(&["caller"]));

    let java = "class S {\n  boolean check(int x) {\n    return x > 3;\n  }\n\n  int caller(int x) {\n    return check(x) ? 1 : 0;\n  }\n}\n";
    let pack = embedded("java").unwrap();
    let file = FileFunctions::read(pack, "S.java", java).unwrap();
    assert_eq!(file.callers_of(&set(&["check"])), set(&["caller"]));
}
