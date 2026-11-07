//! S10 turn checks (U6) against the frozen, admission-verified E4 turns. T4 turn
//! bytes are inadmissible (R28) and are deliberately absent from the descriptor.

use lesscode_core::turncheck::{check_turn, TurnSpec};
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("parent")
        .to_path_buf()
}

#[test]
fn frozen_e4_turns_match_preregistered_expectations() {
    let root = repo_root().join("docs/experiments/deconvolution/regenerator/results/turns");
    if !root.is_dir() {
        eprintln!("skip: frozen turns absent");
        return;
    }
    let specs = [
        (
            "e4-turn1",
            "failed-thinking-only",
            false,
            "request-e4-turn1.json",
        ),
        // Recorded protocol fact (R30 amd 2): a thinking-only failure yields no
        // candidate and no feedback; turn 2's request has feedback null.
        (
            "e4-turn2",
            "failed-collection",
            false,
            "request-e4-turn2.json",
        ),
        ("e4-turn3", "accepted", true, "request-e4-turn3.json"),
    ];
    for (dir, expected, feedback, request) in specs {
        let spec = TurnSpec {
            dir: dir.into(),
            expected: expected.into(),
            request: repo_root()
                .join("docs/experiments/deconvolution/e4-lifecycle/results")
                .join(request),
            feedback_required: feedback,
        };
        let checks = check_turn(&root, &spec);
        let failed: Vec<_> = checks.iter().filter(|(_, ok, _)| !ok).collect();
        assert!(
            failed.is_empty(),
            "{dir} failed checks: {failed:?} (all: {checks:?})"
        );
    }
}

#[test]
fn classification_rules() {
    let parse = |t: &str| lesscode_core::pyjson::parse(t).expect("parse");
    let thinking =
        parse(r#"{"stop_reason":"max_tokens","content":[{"type":"thinking","thinking":"..."}]}"#);
    assert_eq!(
        lesscode_core::turncheck::classify_for_test(&thinking),
        "thinking-only"
    );
    let produced =
        parse(r#"{"stop_reason":"end_turn","content":[{"type":"tool_use","name":"apply_patch"}]}"#);
    assert_eq!(
        lesscode_core::turncheck::classify_for_test(&produced),
        "produced-candidate"
    );
    let other = parse(r#"{"stop_reason":"refusal","content":[]}"#);
    assert_eq!(
        lesscode_core::turncheck::classify_for_test(&other),
        "unclassified:refusal"
    );
}
