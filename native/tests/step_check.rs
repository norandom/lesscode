//! `step check` through the binary (gradual-refactoring-steps, tasks 4.2
//! to 4.4): one accepted step and a refusal by each check.

mod step_support;

use serde_json::{json, Value};
use step_support::{toy, Repo, APP};

const PINNED: &str = "if a > 3:\nstate.total = a\n";

/// Check a step and read its verdict.
fn check(repo: &Repo, step: &str) -> (Option<i32>, Value, String) {
    let (code, stdout, stderr) = repo.step("check", step, &[]);
    let path = repo.dir.join(format!(".kiro/specs/{step}/verdict.json"));
    let verdict = match path.is_file() {
        true => repo.json(&path),
        false => Value::Null,
    };
    (code, verdict, format!("{stdout}{stderr}"))
}

fn result_of<'a>(verdict: &'a Value, name: &str) -> &'a str {
    let checks = verdict["checks"].as_array().unwrap();
    let check = checks.iter().find(|c| c["name"] == name);
    check.map_or("absent", |c| c["result"].as_str().unwrap())
}

fn reasons(verdict: &Value) -> String {
    let reasons = verdict["reasons"].as_array().unwrap();
    let texts: Vec<&str> = reasons.iter().map(|r| r.as_str().unwrap()).collect();
    texts.join(" | ")
}

#[test]
fn a_step_with_pins_first_and_a_smaller_change_is_accepted() {
    let repo = Repo::new("accepted");
    let criterion = [
        toy(),
        "--file".into(),
        "src/app.py".into(),
        "--expect".into(),
        "state.total = a".into(),
    ];
    repo.begin(
        "smaller",
        json!({"criteria": [{"id": "1.1", "command": criterion}]}),
    );
    repo.commit(&[("tests/expect.txt", PINNED)], "pin the write");
    let simpler = APP.replace("raise ValueError(\"big\")", "raise ValueError");
    let revision = repo.commit(&[("src/app.py", &simpler)], "the change");
    let before = repo.state();

    let (code, verdict, said) = check(&repo, "smaller");
    assert_eq!(code, Some(0), "{said}\n{verdict}");
    assert_eq!(verdict["receipt"], "step-verdict-v1");
    assert_eq!(verdict["result"], "accepted");
    assert_eq!(verdict["revision"], revision.as_str());
    assert_eq!(verdict["base"], repo.git(&["rev-parse", "main"]).as_str());
    assert_eq!(verdict["reasons"], json!([]));
    for name in [
        "declaration and revisions",
        "changes stay in scope",
        "pins apart from the change",
        "size",
        "state frame",
        "tests and fault runs",
        "planted faults",
        "evidence for criteria",
    ] {
        assert_eq!(result_of(&verdict, name), "pass", "{name}");
    }
    // The hashes are those git has for the committed files.
    let blob = repo.git(&["rev-parse", "HEAD:src/app.py"]);
    assert_eq!(verdict["hashes"]["scope"]["src/app.py"], blob.as_str());
    let declaration = repo.git(&["rev-parse", "HEAD:.kiro/specs/smaller/step.json"]);
    assert_eq!(verdict["hashes"]["declaration"], declaration.as_str());
    assert_eq!(verdict["hashes"]["tests"].as_str().unwrap().len(), 64);

    // Faults ran in the changed function only, and fewer go unnoticed.
    assert_eq!(verdict["functions_run"], json!({"src/app.py": ["step"]}));
    let row = &verdict["functions"][0];
    assert_eq!(row["function"], "step");
    let (was, is) = (&row["before"], &row["after"]);
    assert!(
        is["unnoticed"].as_u64() < was["unnoticed"].as_u64(),
        "{row}"
    );
    assert!(
        is["noticed"].as_u64() > was["noticed"].as_u64(),
        "the pin: {row}"
    );
    assert_eq!(
        row["unnoticed_after"].as_array().unwrap().len() as u64,
        is["unnoticed"].as_u64().unwrap()
    );
    let tokens = verdict["size"]["tokens"].as_array().unwrap();
    assert!(tokens[1].as_u64() < tokens[0].as_u64(), "{tokens:?}");

    // Evidence for the tied criterion, marked as provisional.
    let evidence = repo.json(&repo.dir.join(".kiro/specs/smaller/evidence.json"));
    assert_eq!(evidence["receipt"], "step-evidence-provisional-v0");
    assert_eq!(evidence["provisional"], true);
    assert_eq!(evidence["revision"], revision.as_str());
    assert_eq!(evidence["hashes"], verdict["hashes"]);
    assert_eq!(evidence["criteria"][0]["id"], "1.1");
    assert_eq!(evidence["criteria"][0]["passed"], true);

    // Nothing in the repository changed but the two declared outputs.
    let after = repo.state();
    let new: Vec<&str> = after
        .iter()
        .filter(|entry| !before.contains(entry))
        .map(|(name, _)| name.as_str())
        .collect();
    assert_eq!(
        new,
        [
            ".kiro/specs/smaller/evidence.json",
            ".kiro/specs/smaller/verdict.json"
        ]
    );
    assert!(before.iter().all(|entry| after.contains(entry)));
}

#[test]
fn a_step_is_refused_for_what_it_leaves_unprotected_or_breaks() {
    let repo = Repo::new("refused");
    let more = APP.replace("    return a\n", "    state.seen = True\n    return a\n");

    // A new statement no test looks at: more faults go unnoticed.
    repo.begin("unprotected", json!({"max_token_growth": 50}));
    repo.commit(&[("src/app.py", &more)], "add a write");
    let (code, verdict, said) = check(&repo, "unprotected");
    assert_eq!(code, Some(1), "{said}");
    assert_eq!(verdict["result"], "refused");
    assert_eq!(result_of(&verdict, "planted faults"), "refused");
    let why = reasons(&verdict);
    assert!(
        why.contains("src/app.py: step: unnoticed faults rise from"),
        "{why}"
    );
    assert!(
        why.contains("line 5 drop_statement"),
        "the fault is named: {why}"
    );

    // The same step, with the function declared out of reach: reported
    // as unprotected, not refused.
    let reach = json!([{"function": "step", "reason": "no seam for the state object yet"}]);
    repo.begin(
        "declared",
        json!({"max_token_growth": 50, "out_of_reach": reach}),
    );
    repo.commit(&[("src/app.py", &more)], "add a write");
    let (code, verdict, said) = check(&repo, "declared");
    assert_eq!(code, Some(0), "{said}");
    assert_eq!(
        verdict["functions"][0]["out_of_reach"],
        "no seam for the state object yet"
    );
    assert!(!verdict["functions"][0]["unnoticed_after"]
        .as_array()
        .unwrap()
        .is_empty());

    // The project's tests fail after the step.
    repo.begin("broken", json!({}));
    repo.commit(
        &[("src/app.py", &APP.replace("a > 3", "a > 4"))],
        "change the limit",
    );
    let (code, verdict, said) = check(&repo, "broken");
    assert_eq!(code, Some(1), "{said}");
    assert_eq!(result_of(&verdict, "tests and fault runs"), "refused");
    assert!(reasons(&verdict).contains("tests do not pass on the step"));
    assert_eq!(result_of(&verdict, "planted faults"), "absent");

    // A test command that cannot run says nothing about the step: the
    // verdict is "not judged", which is neither accepted nor refused.
    repo.begin("void", json!({"test": ["no-such-command-for-this-test"]}));
    repo.commit(&[("tests/expect.txt", PINNED)], "a pin");
    let (code, verdict, said) = check(&repo, "void");
    assert_eq!(code, Some(2), "{said}");
    assert_eq!(verdict["result"], "not_judged");
    assert_eq!(result_of(&verdict, "tests and fault runs"), "not_judged");

    // A function that has no counterpart of its name: the file's totals
    // are compared and the verdict says so.
    repo.begin("renamed", json!({}));
    repo.commit(
        &[("src/app.py", &APP.replace("def step(", "def advance("))],
        "rename",
    );
    let (code, verdict, said) = check(&repo, "renamed");
    assert_eq!(code, Some(0), "{said}\n{verdict}");
    assert_eq!(
        verdict["unmatched_functions"],
        json!(["src/app.py: advance", "src/app.py: step"])
    );
    assert_eq!(
        verdict["functions_run"],
        json!({"src/app.py": ["advance", "step"]})
    );
}

#[test]
fn a_step_is_refused_before_any_run_for_what_its_commits_show() {
    let repo = Repo::new("static");
    let more = APP.replace("    return a\n", "    state.seen = True\n    return a\n");

    // Growth beyond what the declaration allows.
    repo.begin("grows", json!({}));
    repo.commit(&[("src/app.py", &more)], "add a write");
    let (code, verdict, said) = check(&repo, "grows");
    assert_eq!(code, Some(1), "{said}");
    assert_eq!(result_of(&verdict, "size"), "refused");
    assert!(reasons(&verdict).contains("the declaration allows 0 more"));
    assert_eq!(
        result_of(&verdict, "tests and fault runs"),
        "absent",
        "no run after a refusal"
    );
    assert!(!repo.work.join("grows").join("base").exists());

    // A file outside the scope.
    repo.begin("strays", json!({}));
    repo.commit(&[("README.md", "hello\n")], "something else too");
    let (code, verdict, _) = check(&repo, "strays");
    assert_eq!(code, Some(1));
    assert_eq!(result_of(&verdict, "changes stay in scope"), "refused");
    assert!(reasons(&verdict).contains("README.md is changed and is outside the scope"));

    // Pins and the change in one commit.
    repo.begin("mixed", json!({}));
    let simpler = APP.replace("raise ValueError(\"big\")", "raise ValueError");
    let commit = repo.commit(
        &[("tests/expect.txt", PINNED), ("src/app.py", &simpler)],
        "all at once",
    );
    let (code, verdict, _) = check(&repo, "mixed");
    assert_eq!(code, Some(1));
    assert_eq!(result_of(&verdict, "pins apart from the change"), "refused");
    let why = reasons(&verdict);
    assert!(
        why.contains(&commit[..7]) && why.contains("together"),
        "{why}"
    );

    // A state write outside the declared frame.
    let frame = json!({"allowed_write_sites": ["app.py:4", "app.py:9"],
                       "permitted_shell_locations": [], "failure_invariants": []});
    let spec = ".kiro/specs/framed/frame.json";
    repo.begin(
        "framed",
        json!({"max_token_growth": 50, "frame_spec": spec}),
    );
    repo.commit(
        &[(spec, &frame.to_string()), ("src/app.py", &more)],
        "add a write",
    );
    let (code, verdict, said) = check(&repo, "framed");
    assert_eq!(code, Some(1), "{said}");
    assert_eq!(result_of(&verdict, "state frame"), "refused");
    assert!(reasons(&verdict).contains("src/app.py:5 writes state outside the declared frame"));

    // Work that is not committed is not judged, and nothing is written.
    repo.begin("dirty", json!({}));
    repo.write("src/app.py", &more);
    let (code, verdict, said) = check(&repo, "dirty");
    assert_eq!(code, Some(3), "{said}");
    assert_eq!(verdict, Value::Null);
    assert!(
        said.contains("src/app.py on disk differs from what is committed"),
        "{said}"
    );
    repo.git(&["checkout", "-q", "--", "src/app.py"]);

    // A base that is not an ancestor of the branch is bad input.
    repo.begin("unrelated", json!({"base": "step/grows"}));
    let (code, _, said) = check(&repo, "unrelated");
    assert_eq!(code, Some(3), "{said}");
    assert!(said.contains("not an ancestor"), "{said}");
}
