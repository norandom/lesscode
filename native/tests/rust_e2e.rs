//! End-to-end specimen integration and adverse sensitivity verification for Rust.
//!
//! Governed by:
//! - REQ-RUST-2.1, REQ-RUST-3.2, REQ-RUST-4.4, REQ-RUST-5.4
//! - `.kiro/specs/rust-deconvolution-unification/tasks.md` Task 5

use std::path::Path;
use lesscode_core::adapter_traits::*;
use lesscode_core::adapters::rust::metrics::*;
use lesscode_core::adapters::rust::mutation::RustMutantGenerator;
use lesscode_core::adapters::rust::RustAdapter;

#[test]
fn specimen_candidate_meets_useful_compression_thresholds() {
    let baseline_path = if Path::new("adapters/rust/baseline_lifecycle.rs").exists() {
        Path::new("adapters/rust/baseline_lifecycle.rs").to_path_buf()
    } else {
        Path::new("native/adapters/rust/baseline_lifecycle.rs").to_path_buf()
    };
    let candidate_path = if Path::new("adapters/rust/candidate_lifecycle.rs").exists() {
        Path::new("adapters/rust/candidate_lifecycle.rs").to_path_buf()
    } else {
        Path::new("native/adapters/rust/candidate_lifecycle.rs").to_path_buf()
    };

    assert!(baseline_path.exists(), "Baseline specimen must exist");
    assert!(candidate_path.exists(), "Candidate specimen must exist");

    let base_text = std::fs::read_to_string(&baseline_path).unwrap();
    let cand_text = std::fs::read_to_string(&candidate_path).unwrap();

    let base_metrics = extract_scope_metrics(&base_text);
    let cand_metrics = extract_scope_metrics(&cand_text);

    let report = evaluate_rust_compression(&base_metrics, &cand_metrics);
    assert!(report.compression_met);
    assert_eq!(report.disposition, "QUALIFIED");
    assert_eq!(report.baseline_lines, 248);
    assert_eq!(report.candidate_lines, 148);
    assert_eq!(report.baseline_stmts, 83);
    assert_eq!(report.candidate_stmts, 70);
    assert!(report.line_reduction_pct >= 20.0);
    assert!(report.stmt_reduction_pct >= 10.0);
}

#[test]
fn adverse_mutation_controls_kill_rate_reaches_100_percent() {
    let generator = RustMutantGenerator::new();
    let sample_source = r#"
pub fn run_lifecycle(arm_exit: i32, level2_ops: usize, env_key: &str) -> i32 {
    if Instant::now() >= deadline {
        let _ = Command::new("taskkill").args(["/F", "/T", "/PID"]).output();
        return 124;
    }
    if arm_exit != 0 {
        return 1;
    }
    if level2_ops > 0 {
        return 2;
    }
    let env_val = std::env::var("PYTHONPATH").unwrap_or_default();
    0
}
"#;

    let controls = vec![
        MutationSpec {
            id: "RUST_ADV_1".into(),
            location: "lifecycle.rs:4".into(),
            kind: MutationKind::MissingFailureState,
            description: "Omit taskkill deadline kill".into(),
        },
        MutationSpec {
            id: "RUST_ADV_2".into(),
            location: "lifecycle.rs:7".into(),
            kind: MutationKind::InvertedCondition,
            description: "Inverted refusal condition in verdict".into(),
        },
        MutationSpec {
            id: "RUST_ADV_3".into(),
            location: "lifecycle.rs:10".into(),
            kind: MutationKind::InvertedErrorOrder,
            description: "Bypassed level2 differences check".into(),
        },
        MutationSpec {
            id: "RUST_ADV_4".into(),
            location: "lifecycle.rs:13".into(),
            kind: MutationKind::SilentNone,
            description: "Dropped environment isolation PYTHONPATH".into(),
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
    let adapter = RustAdapter::new();
    let domain = DomainSpec {
        param_names: vec![
            "exit_code".into(),
            "is_fresh".into(),
            "l2_divergences".into(),
            "arm_env_set".into(),
        ],
        finite_values: vec![
            (
                "exit_code".into(),
                vec![
                    TaggedValue::Int64(0),
                    TaggedValue::Int64(1),
                    TaggedValue::Int64(2),
                    TaggedValue::Int64(137),
                    TaggedValue::Int64(255),
                ],
            ),
            (
                "is_fresh".into(),
                vec![TaggedValue::Bool(true), TaggedValue::Bool(false)],
            ),
            (
                "l2_divergences".into(),
                vec![
                    TaggedValue::Int64(0),
                    TaggedValue::Int64(1),
                    TaggedValue::Int64(2),
                    TaggedValue::Int64(5),
                    TaggedValue::Int64(10),
                ],
            ),
            (
                "arm_env_set".into(),
                vec![TaggedValue::Bool(true), TaggedValue::Bool(false)],
            ),
        ],
        sample_limit: None,
    };

    let req = ComparisonRequest {
        entrypoint_baseline: "baseline_lifecycle.rs:verdict".into(),
        entrypoint_candidate: "candidate_lifecycle.rs:verdict".into(),
        domain,
        timeout_ms: 10000,
    };

    let res = adapter.compare(&req);
    match res {
        VerificationResult::Verified { evidence, domain } => {
            assert!(evidence.contains("100 cases verified without divergence"));
            assert_eq!(domain, "exit_code, is_fresh, l2_divergences, arm_env_set");
        }
        _ => panic!("Expected Verified outcome with 100 test cases"),
    }
}

#[test]
fn adapter_protocol_event_execution_and_clean_exit() {
    let adapter = RustAdapter::new();
    let req = RunRequest {
        command: "rustc".into(),
        args: vec!["--version".into()],
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
    assert!(out.contains("rustc"));
}
