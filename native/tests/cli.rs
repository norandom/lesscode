use lesscode_core::{compare, model::*, prepare};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

struct Pair {
    root: PathBuf,
    baseline: PathBuf,
    candidate: PathBuf,
    artifacts: PathBuf,
}
impl Pair {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = fs::canonicalize(std::env::temp_dir())
            .unwrap()
            .join(format!(
                "lesscode-cli-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        let pair = Self {
            baseline: root.join("baseline"),
            candidate: root.join("candidate"),
            artifacts: root.join("artifacts"),
            root,
        };
        for path in [&pair.baseline, &pair.candidate, &pair.artifacts] {
            fs::create_dir_all(path).unwrap();
        }
        fs::write(
            pair.baseline.join("a.py"),
            "def run():\n    x = 1\n    return x\n",
        )
        .unwrap();
        fs::write(pair.candidate.join("a.py"), "def run():\n    return 1\n").unwrap();
        for root in [&pair.baseline, &pair.candidate] {
            fs::write(root.join("test_a.py"), "assert True\n").unwrap();
            fs::write(root.join("unrelated.py"), "x = 1\ny = 2\n").unwrap();
        }
        fs::write(pair.artifacts.join("raw.log"), "one checked case").unwrap();
        pair
    }
    fn manifest(&self) -> Manifest {
        serde_json::from_value(json!({"schema_version":1,"trial_id":"workflow","hypothesis":"reuse constant return",
            "entry_points":{"run":{"kind":"whole_file","path":"a.py"}},"owned":{"implementation":{"kind":"whole_file","path":"a.py"}},
            "wider_boundary":["."],"categories":{"a.py":"production","unrelated.py":"production","test_a.py":"tests"},"exclusions":{},
            "scope_review":{"schema_version":1,"rationale":"complete implementation","relationships":{}},
            "checks":{"api":{"schema_version":1,"description":"return contract","required":true,"definition_digest":"check-v1","artifact_expectations":["raw.log"],"interpretation":{"kind":"case_counts","expected":1}}},
            "target":null,"capability_inventory":null})).unwrap()
    }
    fn trial(&self) -> FrozenTrial {
        let mut manifest = self.manifest();
        let definitions = lesscode_core::metrics::measure_python("a.py", b"", "", &[]);
        manifest.target = Some(Target {
            schema_version: SchemaVersion::V1,
            scope_identity: "production".into(),
            language: Language::Python,
            definitions: Records(
                definitions
                    .0
                    .into_iter()
                    .map(|(name, m)| (name, m.identity))
                    .collect(),
            ),
            minimum_production_statement_reduction: Count(1),
            count_bounds: Records(Default::default()),
            permitted_structural_tradeoffs: vec![],
        });
        prepare(
            manifest,
            &self.baseline,
            std::slice::from_ref(&self.artifacts),
        )
        .unwrap()
    }
    fn evidence(&self, trial: &FrozenTrial) -> EvidenceBundle {
        let candidate = lesscode_core::scope::inventory(&self.candidate, &trial.manifest).unwrap();
        let raw: String = Sha256::digest(fs::read(self.artifacts.join("raw.log")).unwrap())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        let result = |source: &str| json!({"schema_version":1,"source_identity":source,"frozen_trial_digest":lesscode_core::compare::frozen_trial_digest(trial),"definition_digest":"check-v1","status":"passed","artifacts":{"raw":{"schema_version":1,"path":"raw.log","sha256":raw}},"tested_domain":"one return case","limitations":["finite"],"summary":{"kind":"case_counts","expected":1,"observed":1,"mismatches":0}});
        serde_json::from_value(json!({"schema_version":1,"candidate_ownership":{"schema_version":1,"replacements":{"implementation":[{"kind":"whole_file","path":"a.py"}]},"review":{"schema_version":1,"rationale":"complete replacement","relationships":{}}},"checks":{"api":{"schema_version":1,"baseline":result(&trial.baseline_identity),"candidate":result(&candidate.identity)}},"dependency_changes":{}})).unwrap()
    }
    fn compare(
        &self,
        trial: &FrozenTrial,
        evidence: &EvidenceBundle,
    ) -> Result<Comparison, lesscode_core::InputError> {
        compare(
            trial,
            &self.baseline,
            &self.candidate,
            evidence,
            std::slice::from_ref(&self.artifacts),
        )
    }
}
impl Drop for Pair {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

fn cli(args: &[&std::ffi::OsStr]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_lesscode-core"))
        .args(args)
        .output()
        .unwrap()
}
fn write_json(path: &std::path::Path, value: &impl serde::Serialize) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}
fn run_compare(
    pair: &Pair,
    trial: &FrozenTrial,
    evidence: &EvidenceBundle,
    out: &std::path::Path,
) -> std::process::Output {
    let frozen = pair.root.join("frozen.json");
    let checks = pair.root.join("checks.json");
    write_json(&frozen, trial);
    write_json(&checks, evidence);
    cli(&[
        "compare".as_ref(),
        "--frozen".as_ref(),
        frozen.as_os_str(),
        "--baseline".as_ref(),
        pair.baseline.as_os_str(),
        "--candidate".as_ref(),
        pair.candidate.as_os_str(),
        "--evidence".as_ref(),
        checks.as_os_str(),
        "--artifact-root".as_ref(),
        pair.artifacts.as_os_str(),
        "--out".as_ref(),
        out.as_os_str(),
    ])
}
#[test]
fn every_status_is_rendered_with_its_exit_code_and_sources_unchanged() {
    for (status, code) in [
        (ComparisonStatus::Accepted, 0),
        (ComparisonStatus::RejectedBehavior, 1),
        (ComparisonStatus::RejectedTarget, 1),
        (ComparisonStatus::NotDemonstrated, 1),
        (ComparisonStatus::Incomplete, 2),
        (ComparisonStatus::BaselineIneligible, 3),
        (ComparisonStatus::Invalid, 3),
    ] {
        let pair = Pair::new();
        let trial = if status == ComparisonStatus::NotDemonstrated {
            prepare(pair.manifest(), &pair.baseline, &[]).unwrap()
        } else {
            pair.trial()
        };
        let mut evidence = pair.evidence(&trial);
        match status {
            ComparisonStatus::RejectedBehavior | ComparisonStatus::BaselineIneligible => {
                let check = evidence.checks.0.get_mut("api").unwrap();
                let side = if status == ComparisonStatus::BaselineIneligible {
                    &mut check.baseline
                } else {
                    &mut check.candidate
                };
                side.as_mut().unwrap().summary = CheckSummary::CaseCounts {
                    expected: Count(1),
                    observed: Count(1),
                    mismatches: Count(1),
                };
            }
            ComparisonStatus::RejectedTarget => {
                fs::copy(pair.baseline.join("a.py"), pair.candidate.join("a.py")).unwrap();
                evidence = pair.evidence(&trial);
            }
            ComparisonStatus::Incomplete => evidence.checks.0.clear(),
            ComparisonStatus::Invalid => {
                evidence
                    .checks
                    .0
                    .get_mut("api")
                    .unwrap()
                    .candidate
                    .as_mut()
                    .unwrap()
                    .source_identity = "stale".into()
            }
            _ => {}
        }
        assert_eq!(pair.compare(&trial, &evidence).unwrap().status, status);
        let before = [
            lesscode_core::scope::inventory(&pair.baseline, &trial.manifest).unwrap(),
            lesscode_core::scope::inventory(&pair.candidate, &trial.manifest).unwrap(),
        ];
        let out = pair.root.join("new-parent/report");
        let result = run_compare(&pair, &trial, &evidence, &out);
        assert_eq!(
            result.status.code(),
            Some(code),
            "{status:?}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(out.join("result.json")).unwrap()).unwrap();
        assert_eq!(
            report["comparison"]["status"],
            serde_json::to_value(status).unwrap()
        );
        assert!(report["comparison"]["accounting"]["capability_metrics"].is_object());
        assert!(fs::read_to_string(out.join("report.md"))
            .unwrap()
            .contains("Status:"));
        assert_eq!(
            before,
            [
                lesscode_core::scope::inventory(&pair.baseline, &trial.manifest).unwrap(),
                lesscode_core::scope::inventory(&pair.candidate, &trial.manifest).unwrap()
            ]
        );
    }
}
#[test]
fn prepare_freezes_once_and_output_refusals_do_not_write_sources() {
    let pair = Pair::new();
    let manifest = pair.root.join("manifest.json");
    write_json(&manifest, &pair.manifest());
    let frozen = pair.root.join("new-parent/frozen.json");
    let run = |out: &std::path::Path| {
        cli(&[
            "prepare".as_ref(),
            "--manifest".as_ref(),
            manifest.as_os_str(),
            "--baseline".as_ref(),
            pair.baseline.as_os_str(),
            "--out".as_ref(),
            out.as_os_str(),
        ])
    };
    assert_eq!(run(&frozen).status.code(), Some(0));
    let bytes = fs::read(&frozen).unwrap();
    let trial: FrozenTrial = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(run(&frozen).status.code(), Some(3));
    assert_eq!(fs::read(&frozen).unwrap(), bytes);
    assert_eq!(
        run(&pair.baseline.join("forbidden/frozen.json"))
            .status
            .code(),
        Some(3)
    );
    assert!(!pair.baseline.join("forbidden").exists());
    let evidence = pair.evidence(&trial);
    for out in [
        &pair.baseline.join("no/report"),
        &pair.candidate.join("no/report"),
        &pair.artifacts,
    ] {
        let result = run_compare(&pair, &trial, &evidence, out);
        assert_eq!(result.status.code(), Some(3));
        assert!(!result.stderr.is_empty());
    }
    assert!(!pair.baseline.join("no").exists());
    assert!(!pair.candidate.join("no").exists());
    let separator = std::path::MAIN_SEPARATOR;
    let traversal = PathBuf::from(format!(
        "{}{separator}missing{separator}..{separator}..{separator}escaped",
        pair.baseline.display()
    ));
    assert_eq!(run(&traversal).status.code(), Some(3));
    assert!(!pair.baseline.join("missing").exists());
}
#[test]
fn invalid_inputs_leave_no_output_and_help_explains_contract() {
    let pair = Pair::new();
    let frozen = pair.root.join("malformed.json");
    fs::write(&frozen, "{bad").unwrap();
    let out = pair.root.join("missing/new.json");
    let result = cli(&[
        "prepare".as_ref(),
        "--manifest".as_ref(),
        frozen.as_os_str(),
        "--baseline".as_ref(),
        pair.baseline.as_os_str(),
        "--out".as_ref(),
        out.as_os_str(),
    ]);
    assert_eq!(result.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&result.stderr).contains("malformed.json"));
    assert!(!out.parent().unwrap().exists());
    let help = cli(&["--help".as_ref()]);
    assert!(help.status.success());
    let text = String::from_utf8(help.stdout).unwrap();
    for word in ["prepare", "compare", "incomplete", "baseline-ineligible"] {
        assert!(text.contains(word), "{word}: {text}");
    }
    assert_eq!(cli(&["compare".as_ref()]).status.code(), Some(3));
}
#[test]
fn omitted_artifact_roots_report_incomplete_and_repeated_reports_are_identical() {
    let pair = Pair::new();
    let trial = pair.trial();
    let evidence = pair.evidence(&trial);
    let first = pair.root.join("first");
    let second = pair.root.join("second");
    assert_eq!(
        run_compare(&pair, &trial, &evidence, &first).status.code(),
        Some(0)
    );
    assert_eq!(
        run_compare(&pair, &trial, &evidence, &second).status.code(),
        Some(0)
    );
    for file in ["result.json", "report.md"] {
        assert_eq!(
            fs::read(first.join(file)).unwrap(),
            fs::read(second.join(file)).unwrap()
        );
    }
    let third = pair.root.join("without-roots");
    let result = cli(&[
        "compare".as_ref(),
        "--frozen".as_ref(),
        pair.root.join("frozen.json").as_os_str(),
        "--baseline".as_ref(),
        pair.baseline.as_os_str(),
        "--candidate".as_ref(),
        pair.candidate.as_os_str(),
        "--evidence".as_ref(),
        pair.root.join("checks.json").as_os_str(),
        "--out".as_ref(),
        third.as_os_str(),
    ]);
    assert_eq!(result.status.code(), Some(2));
    assert!(third.join("result.json").is_file());
    assert!(String::from_utf8_lossy(&result.stderr).contains("artifact"));
    fs::write(pair.baseline.join("a.py"), "pass\n").unwrap();
    let invalid = pair.root.join("invalid-parent/report");
    let result = run_compare(&pair, &trial, &evidence, &invalid);
    assert_eq!(result.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&result.stderr).contains("baseline"));
    assert!(!invalid.parent().unwrap().exists());
}
#[test]
fn output_links_and_reparse_ancestors_are_rejected_before_writes() {
    let pair = Pair::new();
    let alias = pair.root.join("linked-source");
    #[cfg(windows)]
    {
        let result = std::process::Command::new("cmd")
            .args(["/c", "mklink", "/J"])
            .arg(&alias)
            .arg(&pair.candidate)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "junction fixture: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(&pair.candidate, &alias).unwrap();
    let trial = pair.trial();
    let evidence = pair.evidence(&trial);
    for out in [&alias.join("new/report"), &alias] {
        let result = run_compare(&pair, &trial, &evidence, out);
        assert_eq!(result.status.code(), Some(3));
        assert!(String::from_utf8_lossy(&result.stderr).contains("symlink/reparse"));
    }
    assert!(!pair.candidate.join("new").exists());
    fs::remove_dir(&alias)
        .or_else(|_| fs::remove_file(&alias))
        .unwrap();
}

#[test]
fn budgets_subcommand_enforces_frozen_ceilings_through_the_binary() {
    let pair = Pair::new();
    let native = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = pair.root.join("budgets-receipt.json");
    let result = cli(&[
        "budgets".as_ref(),
        "--config".as_ref(),
        native.join("complexity-budgets.json").as_os_str(),
        "--root".as_ref(),
        native.join("src").as_os_str(),
        "--out".as_ref(),
        out.as_os_str(),
    ]);
    // The live native/src tree is within every frozen ceiling.
    assert_eq!(result.status.code(), Some(0));
    let receipt: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&out).unwrap()).unwrap();
    assert_eq!(receipt["receipt"], "budgets");
    assert_eq!(receipt["schema"], "rust-syntax-units-v1");
    assert_eq!(receipt["summary"]["exceeded"], 0);
    assert_eq!(receipt["summary"]["unlisted"], 0);
    assert_eq!(receipt["summary"]["missing"], 0);
    assert!(receipt["unlisted"].as_array().unwrap().is_empty());

    // An unlisted .rs file fails closed with exit code 1.
    let viol_root = pair.root.join("viol");
    fs::create_dir_all(&viol_root).unwrap();
    fs::write(viol_root.join("lib.rs"), "fn main() {}\n").unwrap();
    let empty_cfg = pair.root.join("empty-budgets.json");
    fs::write(&empty_cfg, r#"{"files": []}"#).unwrap();
    let viol_out = pair.root.join("viol-receipt.json");
    let result = cli(&[
        "budgets".as_ref(),
        "--config".as_ref(),
        empty_cfg.as_os_str(),
        "--root".as_ref(),
        viol_root.as_os_str(),
        "--out".as_ref(),
        viol_out.as_os_str(),
    ]);
    assert_eq!(result.status.code(), Some(1));
    let receipt: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&viol_out).unwrap()).unwrap();
    assert_eq!(receipt["unlisted"][0], "lib.rs");
    assert_eq!(receipt["summary"]["unlisted"], 1);

    // An invalid config surfaces as exit code 3.
    let bad_cfg = pair.root.join("bad-budgets.json");
    fs::write(&bad_cfg, "{bad").unwrap();
    let result = cli(&[
        "budgets".as_ref(),
        "--config".as_ref(),
        bad_cfg.as_os_str(),
        "--root".as_ref(),
        viol_root.as_os_str(),
        "--out".as_ref(),
        pair.root.join("bad-receipt.json").as_os_str(),
    ]);
    assert_eq!(result.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&result.stderr).contains("bad-budgets.json"));
}

#[test]
fn admission_subcommand_reports_binding_statuses_through_the_binary() {
    let pair = Pair::new();
    fs::write(pair.root.join("a.json"), b"[1]\n").unwrap();
    let digest: String = Sha256::digest(b"[1]\n")
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    fs::write(
        pair.root.join("a.json.sha256"),
        format!("{digest}  a.json\n"),
    )
    .unwrap();
    fs::write(pair.root.join("b.json"), b"[]\n").unwrap();
    let bindings = pair.root.join("bindings.json");
    fs::write(
        &bindings,
        br#"{"bindings": [{"artifact": "a.json", "sidecars": ["a.json.sha256"]}, {"artifact": "b.json"}]}"#,
    )
    .unwrap();
    let run_admission = |bindings: &std::path::Path, out: &std::path::Path| {
        cli(&[
            "admission".as_ref(),
            "--root".as_ref(),
            pair.root.as_os_str(),
            "--bindings".as_ref(),
            bindings.as_os_str(),
            "--out".as_ref(),
            out.as_os_str(),
        ])
    };

    // Verified plus unbound-only bindings pass under default admission.
    let out = pair.root.join("adm-receipt.json");
    let result = run_admission(&bindings, &out);
    assert_eq!(result.status.code(), Some(0));
    let receipt: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&out).unwrap()).unwrap();
    assert_eq!(receipt["summary"]["verified"], 1);
    assert_eq!(receipt["summary"]["no_bound_sidecar"], 1);
    assert!(fs::read_to_string(pair.root.join("adm-receipt.json.sha256")).is_ok());

    // A mismatched sidecar digest fails closed with exit code 1.
    let bad = pair.root.join("bindings-bad.json");
    fs::write(
        &bad,
        br#"{"bindings": [{"artifact": "b.json", "sidecars": ["a.json.sha256"]}]}"#,
    )
    .unwrap();
    let out_bad = pair.root.join("adm-bad.json");
    let result = run_admission(&bad, &out_bad);
    assert_eq!(result.status.code(), Some(1));
    let receipt: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&out_bad).unwrap()).unwrap();
    assert_eq!(receipt["summary"]["mismatch"], 1);

    // Unreadable bindings surface as exit code 3.
    let result = run_admission(
        &pair.root.join("ghost-bindings.json"),
        &pair.root.join("adm-ghost.json"),
    );
    assert_eq!(result.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&result.stderr).contains("ghost-bindings.json"));

    // No --require-sidecar mode exists: exit codes are 0/1/3 only.
    let help = cli(&["admission".as_ref(), "--help".as_ref()]);
    assert!(help.status.success());
    let text = String::from_utf8(help.stdout).unwrap();
    assert!(text.contains("--bindings"));
    assert!(!text.to_lowercase().contains("require-sidecar"));
}
