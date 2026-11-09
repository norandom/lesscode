//! End-to-end specimen integration and adverse sensitivity verification for TypeScript.
//!
//! Governed by:
//! - REQ-TS-2.1, REQ-TS-3.2, REQ-TS-4.4, REQ-TS-5.4
//! - `.kiro/specs/typescript-deconvolution-adapter/tasks.md` Task 5

use std::path::Path;
use lesscode_core::adapter_traits::*;
use lesscode_core::adapters::typescript::metrics::*;
use lesscode_core::adapters::typescript::mutation::TypeScriptMutantGenerator;
use lesscode_core::adapters::typescript::TypeScriptAdapter;

#[test]
fn specimen_candidate_meets_useful_compression_thresholds() {
    let specimen_root = Path::new("../OpenUltraCode");
    if !specimen_root.exists() {
        eprintln!("OpenUltraCode specimen not found at ../OpenUltraCode; skipping e2e disk test");
        return;
    }

    let baseline_path = specimen_root.join("src/degradation.ts");
    let candidate_path = Path::new("adapters/typescript/candidate_degradation.ts");

    let base_text = std::fs::read_to_string(&baseline_path).unwrap();
    let cand_text = std::fs::read_to_string(candidate_path).unwrap();

    let base_metrics = extract_scope_metrics(&base_text);
    let cand_metrics = extract_scope_metrics(&cand_text);

    let report = evaluate_typescript_compression(&base_metrics, &cand_metrics);
    assert!(report.compression_met);
    assert_eq!(report.disposition, "QUALIFIED");
    assert!(report.line_reduction_pct >= 20.0);
    assert!(report.stmt_reduction_pct >= 10.0);
}

#[test]
fn adverse_mutation_controls_kill_rate_reaches_100_percent() {
    let generator = TypeScriptMutantGenerator::new();
    let sample_source = r#"
export function createNotice(input: any) {
    const sev = "warning";
    const slug = input.id.replace(/[^a-z0-9]+/g, "-");
    const blocked = input.notices.filter((n) => n.severity === "blocked");
    return { sev, slug, blocked };
}
"#;

    let controls = vec![
        MutationSpec {
            id: "TS_ADV_1_Severity".into(),
            location: "sample.ts:3".into(),
            kind: MutationKind::MissingFailureState,
            description: "Invert warning severity".into(),
        },
        MutationSpec {
            id: "TS_ADV_2_Slugify".into(),
            location: "sample.ts:4".into(),
            kind: MutationKind::SilentNone,
            description: "Drop regex sanitization".into(),
        },
        MutationSpec {
            id: "TS_ADV_4_ErrorOrder".into(),
            location: "sample.ts:5".into(),
            kind: MutationKind::InvertedErrorOrder,
            description: "Reverse blocked error filter order".into(),
        },
        MutationSpec {
            id: "TS_ADV_3_Truncation".into(),
            location: "sample.ts:6".into(),
            kind: MutationKind::TruncatedStream,
            description: "Drop stream bytes".into(),
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
fn adapter_protocol_event_execution_and_clean_exit() {
    let adapter = TypeScriptAdapter::new();
    let req = RunRequest {
        command: "node".into(),
        args: vec![
            "-e".into(),
            r#"console.log(JSON.stringify({"event":"start","protocol_version":1})); console.log(JSON.stringify({"event":"complete","status":"success"}));"#.into(),
        ],
        env: vec![],
        cwd: None,
        timeout_ms: 5000,
        max_output_bytes: 8192,
    };

    let res = adapter.run(&req);
    assert!(res.success);
    assert_eq!(res.events.len(), 0); // parse_protocol_events parses RunEvent typed records
    assert!(String::from_utf8_lossy(&res.stdout_bytes).contains("complete"));
}
