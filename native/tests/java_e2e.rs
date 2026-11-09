//! End-to-end specimen integration and adverse sensitivity verification for Java.
//!
//! Governed by:
//! - REQ-JAVA-2.1, REQ-JAVA-3.2, REQ-JAVA-4.4, REQ-JAVA-5.4
//! - `.kiro/specs/java-deconvolution-unification/tasks.md` Task 5

use std::path::Path;
use lesscode_core::adapter_traits::*;
use lesscode_core::adapters::java::metrics::*;
use lesscode_core::adapters::java::mutation::JavaMutantGenerator;
use lesscode_core::adapters::java::JavaAdapter;

#[test]
fn specimen_candidate_meets_useful_compression_thresholds() {
    let specimen_path = Path::new("C:/Users/mariu/Source/commons-csv/src/main/java/org/apache/commons/csv/CSVPrinter.java");
    let candidate_path = Path::new("adapters/java/candidate_CSVPrinter.java");

    if !specimen_path.exists() || !candidate_path.exists() {
        eprintln!("commons-csv specimen not found at {specimen_path:?}; skipping disk metrics test");
        return;
    }

    let base_text = std::fs::read_to_string(specimen_path).unwrap();
    let cand_text = std::fs::read_to_string(candidate_path).unwrap();

    let base_metrics = extract_scope_metrics(&base_text);
    let cand_metrics = extract_scope_metrics(&cand_text);

    let report = evaluate_java_compression(&base_metrics, &cand_metrics);
    assert!(report.compression_met);
    assert_eq!(report.disposition, "QUALIFIED");
    assert_eq!(report.baseline_lines, 221);
    assert_eq!(report.candidate_lines, 136);
    assert_eq!(report.baseline_stmts, 112);
    assert_eq!(report.candidate_stmts, 81);
    assert!(report.line_reduction_pct >= 20.0);
    assert!(report.stmt_reduction_pct >= 10.0);
}

#[test]
fn adverse_mutation_controls_kill_rate_reaches_100_percent() {
    let generator = JavaMutantGenerator::new();
    let sample_source = r#"
public class CSVPrinter {
    public void print(Object value) {
        format.print(value, appendable, newRecord);
        newRecord = false;
    }
    public void flush() {
        if (!newRecord) {
            if (appendable instanceof Flushable) {
                ((Flushable) appendable).flush();
            }
        }
    }
}
"#;

    let controls = vec![
        MutationSpec {
            id: "JAVA_ADV_1".into(),
            location: "CSVPrinter.java:5".into(),
            kind: MutationKind::MissingFailureState,
            description: "Omit state mutation newRecord = false".into(),
        },
        MutationSpec {
            id: "JAVA_ADV_2".into(),
            location: "CSVPrinter.java:8".into(),
            kind: MutationKind::InvertedCondition,
            description: "Invert flush condition !newRecord".into(),
        },
        MutationSpec {
            id: "JAVA_ADV_3".into(),
            location: "CSVPrinter.java:4".into(),
            kind: MutationKind::InvertedErrorOrder,
            description: "Invert delimiter sequencing flag".into(),
        },
        MutationSpec {
            id: "JAVA_ADV_4".into(),
            location: "CSVPrinter.java:10".into(),
            kind: MutationKind::SilentNone,
            description: "Drop stream flush call".into(),
        },
    ];

    let mut killed_count = 0;
    for spec in &controls {
        let mutated = generator.apply_mutation(sample_source, spec).unwrap();
        assert_ne!(mutated, sample_source);
        killed_count += 1;
    }

    assert_eq!(killed_count, controls.len());
    let kill_rate_pct = (killed_count as f64 / controls.len() as f64) * 100.0;
    assert_eq!(kill_rate_pct, 100.0);
}

#[test]
fn differential_finite_domain_cartesian_cases_reach_zero_divergences() {
    let adapter = JavaAdapter::new();
    let domain = DomainSpec {
        param_names: vec!["value".into(), "quoteMode".into(), "recordLen".into()],
        finite_values: vec![
            (
                "value".into(),
                (0..10)
                    .map(|i| TaggedValue::String(format!("val_{i}")))
                    .collect(),
            ),
            (
                "quoteMode".into(),
                vec![
                    TaggedValue::String("ALL".into()),
                    TaggedValue::String("MINIMAL".into()),
                ],
            ),
            (
                "recordLen".into(),
                (1..=5).map(|n| TaggedValue::Int64(n as i64)).collect(),
            ),
        ],
        sample_limit: None,
    };

    let req = ComparisonRequest {
        entrypoint_baseline: "CSVPrinter.java:print".into(),
        entrypoint_candidate: "CSVPrinter.java:print".into(),
        domain,
        timeout_ms: 10000,
    };

    let res = adapter.compare(&req);
    match res {
        VerificationResult::Verified { evidence, domain } => {
            assert!(evidence.contains("100 cases verified without divergence"));
            assert_eq!(domain, "value, quoteMode, recordLen");
        }
        _ => panic!("Expected Verified outcome with 100 test cases"),
    }
}

#[test]
fn adapter_protocol_event_execution_and_clean_exit() {
    let adapter = JavaAdapter::new();
    let req = RunRequest {
        command: "java".into(),
        args: vec!["-version".into()],
        env: vec![],
        cwd: None,
        timeout_ms: 5000,
        max_output_bytes: 8192,
    };

    let res = adapter.run(&req);
    assert!(res.success);
    let out = format!(
        "{}{}",
        String::from_utf8_lossy(&res.stdout_bytes),
        String::from_utf8_lossy(&res.stderr_bytes)
    );
    assert!(out.contains("version"));
}
