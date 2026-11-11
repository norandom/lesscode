//! The adapter traits answered by the driver (task 7.1): the six control
//! kinds mapped onto pack-driven operators, and the twelve controls of the
//! earlier specimen gates looked up in the specimens themselves.
//!
//! Nothing here runs a test, so no control is ever reported as killed.

use lesscode_core::adapter_traits::*;
use lesscode_core::adapters::driver::contract::{generate, operators_of};
use lesscode_core::adapters::java::JavaAdapter;
use lesscode_core::adapters::rust::RustAdapter;
use lesscode_core::adapters::typescript::TypeScriptAdapter;
use std::path::PathBuf;

const SAMPLE: &str = r#"fn check(a: i32, items: &mut Vec<i32>) -> Result<i32, String> {
    let parsed = lookup(a)?;
    if parsed < 0 {
        return Err("negative".into());
    }
    if parsed > 10 {
        return Err("large".into());
    }
    items.push(parsed);
    Ok(parsed)
}
"#;

fn temp_file(name: &str, text: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lesscode-contract-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, text).unwrap();
    path
}

fn control(kind: MutationKind, location: &str) -> MutationSpec {
    MutationSpec {
        id: format!("{kind:?}@{location}"),
        location: location.into(),
        kind,
        description: String::new(),
    }
}

fn outcomes(language: &str, path: &str, controls: Vec<MutationSpec>) -> Vec<MutantOutcome> {
    let request = MutationRequest {
        target_path: path.into(),
        controls,
    };
    let batch = generate(language, &request);
    assert_eq!(
        batch.killed_count, 0,
        "no test is run, so nothing is killed"
    );
    assert!(!batch.all_killed);
    assert_eq!(batch.denominator, request.controls.len());
    batch.results.into_iter().map(|r| r.outcome).collect()
}

fn is_invalid(outcome: &MutantOutcome) -> bool {
    matches!(outcome, MutantOutcome::Invalid { .. })
}

#[test]
fn five_kinds_have_an_operator_and_truncated_stream_has_none() {
    use MutationKind::*;
    let with_operator = [
        InvertedCondition,
        BoundaryOffByOne,
        InvertedErrorOrder,
        MissingFailureState,
        SilentNone,
    ];
    for kind in with_operator {
        assert!(!operators_of(&kind).is_empty(), "{kind:?}");
    }
    assert!(operators_of(&TruncatedStream).is_empty());
}

#[test]
fn each_mapped_kind_finds_its_mutant_at_the_named_line() {
    use MutationKind::*;
    let path = temp_file("sample.rs", SAMPLE);
    let path = path.to_str().unwrap();
    let found = outcomes(
        "rust",
        path,
        vec![
            control(MissingFailureState, "sample.rs:2"),
            control(InvertedCondition, "sample.rs:3"),
            control(BoundaryOffByOne, "sample.rs:3"),
            control(InvertedErrorOrder, "sample.rs:3"),
            control(SilentNone, "sample.rs:9"),
            control(TruncatedStream, "sample.rs:9"),
            control(InvertedCondition, "sample.rs:9"),
            control(InvertedCondition, "sample.rs"),
        ],
    );
    let one = MutantOutcome::NotRun { mutants: 1 };
    assert_eq!(
        found[..5],
        [one.clone(), one.clone(), one.clone(), one.clone(), one]
    );
    assert!(
        is_invalid(&found[5]),
        "no operator stands for a truncated stream"
    );
    assert!(is_invalid(&found[6]), "line 9 has no condition");
    // Without a line, the whole file is searched: two conditions.
    assert_eq!(found[7], MutantOutcome::NotRun { mutants: 2 });
}

#[test]
fn an_unreadable_file_is_a_failure_of_the_run_not_a_kill() {
    use MutationKind::*;
    let found = outcomes(
        "rust",
        "no/such/file.rs",
        vec![control(InvertedCondition, "x:1")],
    );
    assert!(matches!(
        found[0],
        MutantOutcome::InfrastructureFailure { .. }
    ));
    let found = outcomes(
        "klingon",
        "no/such/file.rs",
        vec![control(InvertedCondition, "x:1")],
    );
    assert!(matches!(
        found[0],
        MutantOutcome::InfrastructureFailure { .. }
    ));
}

/// The twelve controls of the gates R50 to R52, each under the kind that
/// fits what it changed, looked up at its line in the specimen. Five have a
/// driver mutant; seven do not, and that loss is recorded in R75.
#[test]
fn the_specimen_controls_have_a_driver_mutant_in_five_of_twelve_cases() {
    use MutationKind::*;
    let none = None::<usize>;
    let cases: [(&str, &str, MutationKind, usize, Option<usize>); 12] = [
        // Rust: `let _ = Command::new("taskkill")…` and a tuple in a list are not sites.
        (
            "rust",
            "adapters/rust/candidate_lifecycle.rs",
            SilentNone,
            36,
            none,
        ),
        (
            "rust",
            "adapters/rust/candidate_lifecycle.rs",
            InvertedCondition,
            102,
            Some(1),
        ),
        (
            "rust",
            "adapters/rust/candidate_lifecycle.rs",
            InvertedCondition,
            104,
            Some(1),
        ),
        (
            "rust",
            "adapters/rust/candidate_lifecycle.rs",
            SilentNone,
            128,
            none,
        ),
        // Java: three dropped statements; a changed argument has no operator.
        (
            "java",
            "adapters/java/candidate_CSVPrinter.java",
            SilentNone,
            81,
            Some(1),
        ),
        (
            "java",
            "adapters/java/candidate_CSVPrinter.java",
            SilentNone,
            76,
            Some(2),
        ),
        (
            "java",
            "adapters/java/candidate_CSVPrinter.java",
            BoundaryOffByOne,
            80,
            none,
        ),
        (
            "java",
            "adapters/java/candidate_CSVPrinter.java",
            SilentNone,
            68,
            Some(1),
        ),
        // TypeScript: a string argument, a chained call, a stream and a list order.
        (
            "typescript",
            "adapters/typescript/candidate_degradation.ts",
            SilentNone,
            52,
            none,
        ),
        (
            "typescript",
            "adapters/typescript/candidate_degradation.ts",
            SilentNone,
            42,
            none,
        ),
        (
            "typescript",
            "adapters/typescript/candidate_degradation.ts",
            TruncatedStream,
            1,
            none,
        ),
        (
            "typescript",
            "adapters/typescript/candidate_degradation.ts",
            InvertedErrorOrder,
            103,
            none,
        ),
    ];
    let mut with_mutant = 0;
    for (language, path, kind, line, expected) in cases {
        let location = format!("{path}:{line}");
        let found = outcomes(language, path, vec![control(kind, &location)]);
        match expected {
            Some(mutants) => {
                assert_eq!(found[0], MutantOutcome::NotRun { mutants }, "{location}");
                with_mutant += 1;
            }
            None => assert!(is_invalid(&found[0]), "{location}: {:?}", found[0]),
        }
    }
    assert_eq!(with_mutant, 5);
}

fn analysis_request(path: &str, allowed: &[&str]) -> AnalysisRequest {
    AnalysisRequest {
        scope_name: "test".into(),
        source_paths: vec![path.into()],
        frame_spec: FrameSpec {
            allowed_write_sites: allowed.iter().map(|a| a.to_string()).collect(),
            ..FrameSpec::default()
        },
        effect_profile: "test".into(),
    }
}

#[test]
fn the_adapters_analyse_through_the_driver() {
    let path = temp_file("frame.rs", SAMPLE);
    let path = path.to_str().unwrap();
    let inside = RustAdapter::new().analyze(&analysis_request(path, &["frame.rs"]));
    assert!(inside.is_valid);
    assert_eq!(inside.write_sites, [format!("{path}:9")]);
    let outside = RustAdapter::new().analyze(&analysis_request(path, &["other.rs"]));
    assert!(!outside.is_valid);
    assert_eq!(outside.violations, [format!("{path}:9")]);

    let java = temp_file(
        "Frame.java",
        "class Frame { int n; void set() { n = 1; } }\n",
    );
    let java = java.to_str().unwrap();
    let found = JavaAdapter::new().analyze(&analysis_request(java, &["Frame.java"]));
    assert_eq!(found.write_sites, [format!("{java}:1")]);

    let ts = temp_file(
        "frame.ts",
        "let n = 0;\nexport function set() {\n  n = 1;\n}\n",
    );
    let ts = ts.to_str().unwrap();
    let found = TypeScriptAdapter::new().analyze(&analysis_request(ts, &["frame.ts"]));
    assert_eq!(found.write_sites, [format!("{ts}:3")]);
}

#[test]
fn a_file_that_cannot_be_read_or_parsed_is_a_gap() {
    let missing = RustAdapter::new().analyze(&analysis_request("no/such/file.rs", &[]));
    assert!(!missing.is_valid);
    assert_eq!(missing.unverified_gaps.len(), 1);

    let broken = temp_file("broken.rs", "fn broken( {\n");
    let broken = broken.to_str().unwrap();
    let found = RustAdapter::new().analyze(&analysis_request(broken, &[]));
    assert!(!found.is_valid);
    assert!(found.unverified_gaps[0].reason.contains("does not parse"));
}
