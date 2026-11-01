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
        let root = std::env::temp_dir().join(format!(
            "lesscode-workflow-{}-{}",
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

#[test]
fn prepared_pair_measures_real_production_and_separate_categories_deterministically() {
    let pair = Pair::new();
    let trial = pair.trial();
    let evidence = pair.evidence(&trial);
    let before = fs::read(pair.candidate.join("a.py")).unwrap();
    let result = pair.compare(&trial, &evidence).unwrap();
    assert_eq!(
        result.status,
        ComparisonStatus::Accepted,
        "{:?}",
        result.reasons
    );
    assert_eq!(result, pair.compare(&trial, &evidence).unwrap());
    assert!(result
        .accounting
        .capability_metrics
        .0
        .contains_key("production/Python/LogicalStatements"));
    assert!(result
        .accounting
        .capability_metrics
        .0
        .values()
        .any(|p| p.before.identity.name == MetricName::LogicalStatements && p.delta == Some(-1)));
    assert!(result
        .accounting
        .category_totals
        .0
        .contains_key(&Category::Tests));
    assert_ne!(
        result.accounting.capability_metrics,
        result.accounting.wider_metrics
    );
    assert_eq!(before, fs::read(pair.candidate.join("a.py")).unwrap());
}

#[test]
fn failures_missing_checks_and_stale_sources_cannot_accept() {
    let pair = Pair::new();
    let trial = pair.trial();
    let mut evidence = pair.evidence(&trial);
    evidence
        .checks
        .0
        .get_mut("api")
        .unwrap()
        .candidate
        .as_mut()
        .unwrap()
        .summary = CheckSummary::CaseCounts {
        expected: Count(1),
        observed: Count(1),
        mismatches: Count(1),
    };
    assert_eq!(
        pair.compare(&trial, &evidence).unwrap().status,
        ComparisonStatus::RejectedBehavior
    );
    evidence.checks.0.clear();
    assert_eq!(
        pair.compare(&trial, &evidence).unwrap().status,
        ComparisonStatus::Incomplete
    );
    fs::write(pair.baseline.join("a.py"), "pass\n").unwrap();
    assert!(pair
        .compare(&trial, &evidence)
        .unwrap_err()
        .0
        .contains("baseline"));
}

#[test]
fn frozen_definitions_and_evaluator_are_checked_without_rebaselining() {
    let pair = Pair::new();
    let trial = pair.trial();
    let evidence = pair.evidence(&trial);
    let mut changed = trial.clone();
    changed.evaluator_identity.push_str("changed");
    assert!(pair.compare(&changed, &evidence).is_err());
    let mut changed = trial;
    changed
        .definitions
        .0
        .values_mut()
        .next()
        .unwrap()
        .definition_revision = "v99".into();
    assert!(pair.compare(&changed, &evidence).is_err());
}

#[test]
fn unresolved_locations_join_each_sides_ownership_review() {
    let pair = Pair::new();
    fs::write(
        pair.baseline.join("a.py"),
        "def run():\n    external()\n    return 1\n",
    )
    .unwrap();
    fs::write(
        pair.candidate.join("a.py"),
        "def run():\n    return external()\n",
    )
    .unwrap();
    let trial = pair.trial();
    let evidence = pair.evidence(&trial);
    let result = pair.compare(&trial, &evidence).unwrap();
    assert_eq!(result.status, ComparisonStatus::Incomplete);
    assert_eq!(result.accounting.unresolved.0.len(), 2);
    let mut manifest = trial.manifest.clone();
    let mut evidence = evidence;
    for id in result.accounting.unresolved.0.keys() {
        let (side, id) = id.split_once('/').unwrap();
        let review = RelationshipReview {
            schema_version: SchemaVersion::V1,
            owned_targets: vec![],
            external_contract: Some("supplied external API".into()),
            rationale: "public contract checked externally".into(),
        };
        if side == "baseline" {
            manifest
                .scope_review
                .relationships
                .0
                .insert(id.into(), review);
        } else {
            evidence
                .candidate_ownership
                .review
                .relationships
                .0
                .insert(id.into(), review);
        }
    }
    let trial = prepare(manifest, &pair.baseline, &[]).unwrap();
    let checks = pair.evidence(&trial).checks;
    evidence.checks = checks;
    let result = pair.compare(&trial, &evidence).unwrap();
    assert_eq!(
        result.status,
        ComparisonStatus::Accepted,
        "{:?}",
        result.reasons
    );
    assert_eq!(result.accounting.unresolved.0.len(), 2);
    assert!(result
        .accounting
        .capability_metrics
        .0
        .values()
        .any(|p| p.before.identity.name == MetricName::InternalCallLayers && p.delta.is_none()));
}

#[test]
fn support_helpers_are_charged_and_configuration_lines_disclosed() {
    let pair = Pair::new();
    fs::write(
        pair.candidate.join("helper.py"),
        "def helper():\n    x = 1\n    return x\n",
    )
    .unwrap();
    fs::write(pair.candidate.join("config.json"), "{\n  \"value\": 1\n}\n").unwrap();
    let mut manifest = pair.manifest();
    manifest
        .categories
        .0
        .insert("helper.py".into(), Category::Production);
    manifest
        .categories
        .0
        .insert("config.json".into(), Category::Configuration);
    manifest.target = pair.trial().manifest.target;
    let trial = prepare(manifest, &pair.baseline, &[]).unwrap();
    let mut evidence = pair.evidence(&trial);
    evidence
        .candidate_ownership
        .replacements
        .0
        .get_mut("implementation")
        .unwrap()
        .push(Selector::WholeFile {
            path: "helper.py".into(),
        });
    let result = pair.compare(&trial, &evidence).unwrap();
    assert_eq!(
        result.status,
        ComparisonStatus::RejectedTarget,
        "{:?}",
        result.reasons
    );
    assert!(result
        .accounting
        .capability_metrics
        .0
        .values()
        .any(|p| p.before.identity.name == MetricName::LogicalStatements && p.delta == Some(2)));
    assert!(result.limitations.iter().any(|s| s.contains("config.json")
        && s.contains("physical_lines.v1=3")
        && s.contains("Configuration")));
}

#[test]
fn no_target_is_descriptive_and_missing_ownership_is_incomplete() {
    let pair = Pair::new();
    let trial = prepare(pair.manifest(), &pair.baseline, &[]).unwrap();
    let mut evidence = pair.evidence(&trial);
    assert_eq!(
        pair.compare(&trial, &evidence).unwrap().status,
        ComparisonStatus::NotDemonstrated
    );
    evidence.candidate_ownership.replacements.0.clear();
    let result = pair.compare(&trial, &evidence).unwrap();
    assert_eq!(result.status, ComparisonStatus::Incomplete);
    assert!(result
        .reasons
        .iter()
        .any(|f| f.explanation.contains("omitted")));
}

#[test]
fn preparation_verifies_external_capability_inventory_bytes() {
    let pair = Pair::new();
    let mut manifest = pair.manifest();
    let snapshot = lesscode_core::scope::inventory(&pair.baseline, &manifest).unwrap();
    fs::write(pair.artifacts.join("inventory.json"), "reviewed inventory").unwrap();
    let sha256: String = Sha256::digest(b"reviewed inventory")
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    manifest.capability_inventory = Some(InventoryReference {
        schema_version: SchemaVersion::V1,
        source_identity: snapshot.identity,
        manifest_references: vec!["a.py".into()],
        artifact: Artifact {
            schema_version: SchemaVersion::V1,
            path: "inventory.json".into(),
            sha256,
        },
    });
    assert!(prepare(
        manifest.clone(),
        &pair.baseline,
        std::slice::from_ref(&pair.artifacts)
    )
    .is_ok());
    fs::write(pair.artifacts.join("inventory.json"), "changed inventory").unwrap();
    assert!(prepare(
        manifest,
        &pair.baseline,
        std::slice::from_ref(&pair.artifacts)
    )
    .unwrap_err()
    .0
    .contains("digest"));
}

#[test]
fn wider_unrelated_calls_are_reported_without_expanding_owned_capability() {
    let pair = Pair::new();
    for root in [&pair.baseline, &pair.candidate] {
        fs::write(root.join("unrelated.py"), "def other():\n    external()\n").unwrap();
    }
    let trial = pair.trial();
    let evidence = pair.evidence(&trial);
    let result = pair.compare(&trial, &evidence).unwrap();
    assert_eq!(
        result.status,
        ComparisonStatus::Accepted,
        "{:?}",
        result.reasons
    );
    assert!(result
        .accounting
        .unresolved
        .0
        .keys()
        .any(|s| s.contains("unrelated.py:")));
    assert!(result
        .accounting
        .wider_metrics
        .0
        .values()
        .any(|p| p.before.identity.name == MetricName::InternalCallLayers && p.delta.is_none()));
}

#[test]
fn stale_candidate_evidence_and_changed_frozen_checks_are_detected() {
    let pair = Pair::new();
    let trial = pair.trial();
    let evidence = pair.evidence(&trial);
    fs::write(pair.candidate.join("a.py"), "def run():\n    return 2\n").unwrap();
    assert_eq!(
        pair.compare(&trial, &evidence).unwrap().status,
        ComparisonStatus::Invalid
    );
    let mut trial = trial;
    trial.check_definition_digests.0.clear();
    assert!(pair
        .compare(&trial, &evidence)
        .unwrap_err()
        .0
        .contains("check inventory"));
}

#[test]
fn changing_production_language_does_not_manufacture_a_zero_target_count() {
    let pair = Pair::new();
    let mut manifest = pair.manifest();
    manifest
        .categories
        .0
        .insert("replacement.ts".into(), Category::Production);
    manifest.target = pair.trial().manifest.target;
    fs::remove_file(pair.candidate.join("a.py")).unwrap();
    fs::write(
        pair.candidate.join("replacement.ts"),
        "export function run() { return 1; }\n",
    )
    .unwrap();
    let trial = prepare(manifest, &pair.baseline, &[]).unwrap();
    let mut evidence = pair.evidence(&trial);
    evidence.candidate_ownership.replacements.0.insert(
        "implementation".into(),
        vec![Selector::WholeFile {
            path: "replacement.ts".into(),
        }],
    );
    let result = pair.compare(&trial, &evidence).unwrap();
    assert_eq!(
        result.status,
        ComparisonStatus::Incomplete,
        "{:?}",
        result.reasons
    );
    assert!(result
        .reasons
        .iter()
        .any(|f| f.explanation.contains("language outside")));
    assert!(result
        .accounting
        .capability_metrics
        .0
        .values()
        .any(|p| p.after.identity.language == Language::TypeScript
            && p.after.identity.name == MetricName::LogicalStatements
            && matches!(p.after.value, MetricValue::Available { count: Count(2) })));
}
