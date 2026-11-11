//! `step pin` through the binary (gradual-refactoring-steps, task 4.1).

mod step_support;

use serde_json::json;
use step_support::Repo;

/// The unnoticed faults of the base in `step`, from a first call that names
/// a fault that does not exist.
fn unnoticed(repo: &Repo, step: &str) -> Vec<(String, String)> {
    let (code, _, stderr) = repo.step("pin", step, &["--fault", "9999"]);
    assert_eq!(code, Some(3), "{stderr}");
    let listed = stderr.split("unnoticed are: ").nth(1).expect("a list");
    let faults = listed.trim().split(", ");
    faults
        .map(|label| {
            let number = label.trim_start_matches('#').split(' ').next().unwrap();
            (number.to_owned(), label.to_owned())
        })
        .collect()
}

#[test]
fn a_pin_is_admitted_only_if_it_fails_on_the_fault_and_passes_on_the_code() {
    let repo = Repo::new("pin");

    // A real pin: the tests now expect the write.
    repo.begin("good", json!({}));
    repo.commit(
        &[("tests/expect.txt", "if a > 3:\nstate.total = a\n")],
        "pin the write",
    );
    let before = repo.state();
    let faults = unnoticed(&repo, "good");
    let (number, label) = faults
        .iter()
        .find(|(_, label)| label.contains("line 4 "))
        .unwrap_or_else(|| panic!("no unnoticed fault on the write: {faults:?}"));
    // Only faults of the function in focus are run.
    assert!(
        faults.iter().all(|(_, l)| !l.contains("line 9 ")),
        "{faults:?}"
    );

    let out = repo.work.join("good-pin.json");
    let args = ["--fault", number.as_str(), "--out", out.to_str().unwrap()];
    let (code, stdout, stderr) = repo.step("pin", "good", &args);
    assert_eq!(code, Some(0), "{stdout}{stderr}");
    let receipt = repo.json(&out);
    assert_eq!(receipt["receipt"], "step-pin-v1");
    assert_eq!(receipt["result"], "admitted");
    assert_eq!(receipt["fault"], label.as_str());
    let remaining = receipt["remaining_unnoticed"].as_array().unwrap();
    assert!(!remaining.iter().any(|f| f == label.as_str()));
    // Faults may remain; that is reported, not required to be empty.
    assert_eq!(remaining.len(), faults.len() - 1, "{receipt}");
    assert_eq!(repo.state(), before, "the repository changed");

    // A test that passes whatever the fault does.
    repo.begin("useless", json!({}));
    repo.commit(
        &[("tests/expect.txt", "if a > 3:\ndef step\n")],
        "a pin that pins nothing",
    );
    let (code, stdout, _) = repo.step("pin", "useless", &["--fault", number]);
    assert_eq!(code, Some(1), "{stdout}");
    assert!(
        stdout.contains("rejected") && stdout.contains("pass on the fault"),
        "{stdout}"
    );

    // A test that fails on the code as it is.
    repo.begin("wrong", json!({}));
    repo.commit(
        &[("tests/expect.txt", "if a > 3:\nNOT IN THE CODE\n")],
        "a pin that is wrong",
    );
    let (code, stdout, _) = repo.step("pin", "wrong", &["--fault", number]);
    assert_eq!(code, Some(1), "{stdout}");
    assert!(stdout.contains("fail on the unchanged code"), "{stdout}");

    // A pin is judged against the code of the base, whatever the branch
    // did to the code since.
    repo.begin("both", json!({}));
    repo.commit(
        &[
            ("tests/expect.txt", "if a > 3:\nstate.total = a\n"),
            ("src/app.py", "def step(state, a):\n    return a\n"),
        ],
        "pin and change together",
    );
    let (code, stdout, stderr) = repo.step("pin", "both", &["--fault", number]);
    assert_eq!(code, Some(0), "{stdout}{stderr}");
}
