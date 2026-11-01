//! Admission of supplied finite behavioral evidence; never executes project checks.
use crate::model::*;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceIssue {
    Invalid,
    BaselineIneligible,
    RejectedBehavior,
    Incomplete,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceFinding {
    pub issue: EvidenceIssue,
    pub finding: Finding,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceAdmission {
    pub checks: Records<String, PairedCheck>,
    pub findings: Vec<EvidenceFinding>,
    pub limitations: Vec<String>,
}
/// Stable SHA-256 of the closed, sorted frozen-trial JSON record.
pub fn frozen_trial_digest(trial: &FrozenTrial) -> String {
    hash(&serde_json::to_vec(trial).expect("typed frozen trial is JSON serializable"))
}
fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

impl EvidenceAdmission {
    fn finding(&mut self, issue: EvidenceIssue, subject: &str, explanation: impl Into<String>) {
        self.findings.push(EvidenceFinding {
            issue,
            finding: Finding {
                schema_version: SchemaVersion::V1,
                subject: subject.into(),
                explanation: explanation.into(),
            },
        });
    }
}
/// Validate supplied results under frozen interpretations. Roots are explicit external
/// evidence directories. Check artifact expectations are explicit relative paths,
/// matched against artifact records independently of their IDs. Paths are portable
/// relative paths; links and ambiguous
/// matches across approved roots are refused. Hashes establish consistency only,
/// not authenticity of a declared runner or universal behavioral equivalence.
pub fn admit_evidence(
    trial: &FrozenTrial,
    candidate_identity: &str,
    evidence: &EvidenceBundle,
    artifact_roots: &[PathBuf],
) -> EvidenceAdmission {
    let mut admission = EvidenceAdmission { checks: evidence.checks.clone(), findings: vec![],
        limitations: vec!["Finite supplied checks cover only their stated tested domains; unverified behavior remains outside this evidence.".into()] };
    if !evidence
        .checks
        .0
        .values()
        .any(|pair| pair.baseline.is_some() && pair.candidate.is_some())
    {
        admission.finding(
            EvidenceIssue::Incomplete,
            "checks",
            "no complete baseline/candidate behavioral evidence pair",
        );
    }
    let digest = frozen_trial_digest(trial);
    for id in trial.check_definition_digests.0.keys() {
        if !trial.manifest.checks.0.contains_key(id) {
            admission.finding(
                EvidenceIssue::Invalid,
                id,
                "frozen digest refers to unknown check",
            );
        }
    }
    for id in evidence.checks.0.keys() {
        if !trial.manifest.checks.0.contains_key(id) {
            admission.finding(
                EvidenceIssue::Invalid,
                id,
                "unknown supplied check identity",
            );
        }
    }
    if trial.manifest.checks.0.is_empty() {
        admission.finding(
            EvidenceIssue::Incomplete,
            "checks",
            "no frozen behavioral check inventory",
        );
    }
    for (id, definition) in &trial.manifest.checks.0 {
        if definition.definition_digest.is_empty()
            || trial.check_definition_digests.0.get(id) != Some(&definition.definition_digest)
        {
            admission.finding(
                EvidenceIssue::Invalid,
                id,
                "altered or missing frozen check definition digest",
            );
        }
        let pair = evidence.checks.0.get(id);
        for (side, result, source) in [
            (
                "baseline",
                pair.and_then(|p| p.baseline.as_ref()),
                trial.baseline_identity.as_str(),
            ),
            (
                "candidate",
                pair.and_then(|p| p.candidate.as_ref()),
                candidate_identity,
            ),
        ] {
            let subject = format!("{id}/{side}");
            let Some(result) = result else {
                if definition.required {
                    admission.finding(
                        EvidenceIssue::Incomplete,
                        &subject,
                        "required check result missing",
                    );
                }
                continue;
            };
            admission
                .limitations
                .extend(result.limitations.iter().cloned());
            if result.source_identity != source || source.is_empty() {
                admission.finding(EvidenceIssue::Invalid, &subject, "source identity mismatch");
            }
            if result.frozen_trial_digest != digest {
                admission.finding(
                    EvidenceIssue::Invalid,
                    &subject,
                    "frozen trial digest mismatch",
                );
            }
            if result.definition_digest != definition.definition_digest {
                admission.finding(
                    EvidenceIssue::Invalid,
                    &subject,
                    "check definition digest mismatch",
                );
            }
            if result.tested_domain.trim().is_empty() {
                admission.finding(
                    EvidenceIssue::Incomplete,
                    &subject,
                    "tested domain is missing",
                );
            }
            match result.status {
                CheckStatus::Failed => behavior_failure(
                    &mut admission,
                    side,
                    &subject,
                    "declared behavioral failure",
                ),
                CheckStatus::Missing | CheckStatus::Error => admission.finding(
                    EvidenceIssue::Incomplete,
                    &subject,
                    "missing result or collector error",
                ),
                CheckStatus::Passed => (),
            }
            if result.artifacts.0.is_empty() {
                admission.finding(
                    EvidenceIssue::Incomplete,
                    &subject,
                    "raw check artifacts missing",
                );
            }
            for expected in &definition.artifact_expectations {
                if !result
                    .artifacts
                    .0
                    .values()
                    .any(|artifact| &artifact.path == expected)
                {
                    admission.finding(
                        EvidenceIssue::Incomplete,
                        &subject,
                        format!("expected raw artifact missing: {expected}"),
                    );
                }
            }
            for (artifact_id, artifact) in &result.artifacts.0 {
                if let Err((issue, reason)) = validate_artifact(artifact, artifact_roots) {
                    admission.finding(issue, &format!("{subject}/{artifact_id}"), reason);
                }
            }
            let counts = match (&definition.interpretation, &result.summary) {
                (Interpretation::ExitStatus, CheckSummary::ExitStatus { code }) => {
                    match code {
                        Some(0) => (),
                        Some(_) => behavior_failure(
                            &mut admission,
                            side,
                            &subject,
                            "nonzero check exit status",
                        ),
                        None => admission.finding(
                            EvidenceIssue::Incomplete,
                            &subject,
                            "check exit status unavailable",
                        ),
                    }
                    None
                }
                (
                    Interpretation::CaseCounts { expected },
                    CheckSummary::CaseCounts {
                        expected: supplied,
                        observed,
                        mismatches,
                    },
                ) => Some((*expected, *supplied, *observed, *mismatches)),
                (
                    Interpretation::Differential {
                        expected,
                        oracle_digest,
                    },
                    CheckSummary::Differential {
                        baseline_identity,
                        candidate_identity: supplied_candidate,
                        oracle_digest: supplied_oracle,
                        expected: supplied,
                        observed,
                        mismatches,
                    },
                ) => {
                    if baseline_identity != &trial.baseline_identity
                        || supplied_candidate != candidate_identity
                        || supplied_oracle != oracle_digest
                        || oracle_digest.is_empty()
                    {
                        admission.finding(
                            EvidenceIssue::Invalid,
                            &subject,
                            "differential snapshots or frozen oracle mismatch",
                        );
                    }
                    Some((*expected, *supplied, *observed, *mismatches))
                }
                _ => {
                    admission.finding(EvidenceIssue::Invalid, &subject, "summary does not match frozen check interpretation; collector success alone is insufficient");
                    None
                }
            };
            if let Some((expected, supplied, observed, mismatches)) = counts {
                if supplied != expected {
                    admission.finding(
                        EvidenceIssue::Invalid,
                        &subject,
                        "expected case count differs from frozen definition",
                    );
                }
                if mismatches.0 > 0 {
                    behavior_failure(
                        &mut admission,
                        side,
                        &subject,
                        "observed behavioral mismatches",
                    );
                }
                if mismatches > observed {
                    admission.finding(
                        EvidenceIssue::Invalid,
                        &subject,
                        "mismatch count exceeds observed cases",
                    );
                }
                if observed > expected {
                    admission.finding(
                        EvidenceIssue::Invalid,
                        &subject,
                        "observed cases exceed frozen expected count",
                    );
                }
                if expected.0 == 0 || observed < expected {
                    admission.finding(
                        EvidenceIssue::Incomplete,
                        &subject,
                        "zero or incomplete observed case coverage",
                    );
                }
            }
        }
    }
    admission
}
fn behavior_failure(admission: &mut EvidenceAdmission, side: &str, subject: &str, reason: &str) {
    admission.finding(
        if side == "baseline" {
            EvidenceIssue::BaselineIneligible
        } else {
            EvidenceIssue::RejectedBehavior
        },
        subject,
        reason,
    );
}
fn validate_artifact(
    artifact: &Artifact,
    roots: &[PathBuf],
) -> Result<(), (EvidenceIssue, String)> {
    let invalid = |reason: &str| (EvidenceIssue::Invalid, reason.to_owned());
    let path = Path::new(&artifact.path);
    if artifact.path.is_empty()
        || artifact.path.contains(['\\', ':'])
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(invalid(
            "artifact path must be a portable relative path without traversal",
        ));
    }
    let mut found = None;
    for root in roots {
        let metadata = fs::symlink_metadata(root).map_err(|e| {
            (
                EvidenceIssue::Incomplete,
                format!("artifact root unavailable: {e}"),
            )
        })?;
        if is_link(&metadata) || !metadata.is_dir() {
            return Err(invalid("approved artifact root must be a real directory"));
        }
        let mut current = root.clone();
        let mut absent = false;
        for part in path.components() {
            current.push(part);
            match fs::symlink_metadata(&current) {
                Ok(metadata) if is_link(&metadata) => {
                    return Err(invalid(
                        "artifact path crosses a symbolic link or reparse point",
                    ))
                }
                Ok(_) => (),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    absent = true;
                    break;
                }
                Err(error) => {
                    return Err((
                        EvidenceIssue::Incomplete,
                        format!("artifact metadata unavailable: {error}"),
                    ))
                }
            }
        }
        if absent {
            continue;
        }
        if found.is_some() {
            return Err(invalid("artifact path is ambiguous across approved roots"));
        }
        found = Some(current);
    }
    let path = found.ok_or_else(|| {
        (
            EvidenceIssue::Incomplete,
            "raw artifact missing from approved roots".into(),
        )
    })?;
    if !path.is_file() {
        return Err(invalid("raw artifact is not a regular file"));
    }
    let bytes = fs::read(path).map_err(|e| {
        (
            EvidenceIssue::Incomplete,
            format!("raw artifact unreadable: {e}"),
        )
    })?;
    if hash(&bytes) != artifact.sha256 {
        return Err(invalid("raw artifact SHA-256 mismatch"));
    }
    Ok(())
}
fn is_link(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture {
        root: PathBuf,
        trial: FrozenTrial,
        evidence: EvidenceBundle,
        candidate: String,
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "lesscode-evidence-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&root).unwrap();
            fs::write(root.join("baseline.py"), "def api(): return 1\n").unwrap();
            fs::write(root.join("candidate.py"), "def api(): return 2\n").unwrap();
            fs::write(root.join("raw.log"), "3 cases, 0 mismatches\n").unwrap();
            let baseline = hash(&fs::read(root.join("baseline.py")).unwrap());
            let candidate = hash(&fs::read(root.join("candidate.py")).unwrap());
            let definition = json!({"schema_version":1,"description":"public API, types, errors and safeguards","required":true,"definition_digest":"check-v1","artifact_expectations":["raw.log"],"interpretation":{"kind":"differential","expected":3,"oracle_digest":"oracle-v1"}});
            let trial: FrozenTrial = serde_json::from_value(json!({"schema_version":1,"manifest":{"schema_version":1,"trial_id":"trial","hypothesis":"reuse","entry_points":{},"owned":{},"wider_boundary":["."],"categories":{},"exclusions":{},"scope_review":{"schema_version":1,"rationale":"reviewed","relationships":{}},"checks":{"api":definition},"target":null,"capability_inventory":null},"baseline_files":{},"baseline_identity":baseline,"evaluator_identity":"evaluator","definitions":{},"check_definition_digests":{"api":"check-v1"}})).unwrap();
            let result = |source: &str| json!({"schema_version":1,"source_identity":source,"frozen_trial_digest":frozen_trial_digest(&trial),"definition_digest":"check-v1","status":"passed","artifacts":{"log":{"schema_version":1,"path":"raw.log","sha256":hash(&fs::read(root.join("raw.log")).unwrap())}},"tested_domain":"three reviewed API cases","limitations":["finite cases only"],"summary":{"kind":"differential","baseline_identity":baseline,"candidate_identity":candidate,"oracle_digest":"oracle-v1","expected":3,"observed":3,"mismatches":0}});
            let evidence = serde_json::from_value(json!({"schema_version":1,"candidate_ownership":{"schema_version":1,"replacements":{},"review":{"schema_version":1,"rationale":"reviewed","relationships":{}}},"checks":{"api":{"schema_version":1,"baseline":result(&baseline),"candidate":result(&candidate)}},"dependency_changes":{}})).unwrap();
            Self {
                root,
                trial,
                evidence,
                candidate,
            }
        }
        fn run(&self) -> EvidenceAdmission {
            admit_evidence(
                &self.trial,
                &self.candidate,
                &self.evidence,
                std::slice::from_ref(&self.root),
            )
        }
        fn candidate(&mut self) -> &mut CheckResult {
            self.evidence
                .checks
                .0
                .get_mut("api")
                .unwrap()
                .candidate
                .as_mut()
                .unwrap()
        }
        fn has(&self, issue: EvidenceIssue) {
            assert!(
                self.run().findings.iter().any(|f| f.issue == issue),
                "{:#?}",
                self.run()
            );
        }
    }
    #[test]
    fn admits_bound_pair_and_retains_domains_and_limitations() {
        let f = Fixture::new();
        let admitted = f.run();
        assert!(admitted.findings.is_empty(), "{admitted:#?}");
        assert_eq!(admitted.checks, f.evidence.checks);
        assert!(admitted.limitations.iter().any(|s| s.contains("finite")));
    }
    #[test]
    fn stale_source_trial_or_check_binding_is_invalid() {
        for binding in ["source", "trial", "definition"] {
            let mut f = Fixture::new();
            let r = f.candidate();
            match binding {
                "source" => r.source_identity = "stale".into(),
                "trial" => r.frozen_trial_digest = "stale".into(),
                _ => r.definition_digest = "stale".into(),
            };
            f.has(EvidenceIssue::Invalid);
        }
    }
    #[test]
    fn unknown_check_and_altered_frozen_definition_are_invalid() {
        let mut f = Fixture::new();
        let pair = f.evidence.checks.0["api"].clone();
        f.evidence.checks.0.insert("unknown".into(), pair);
        f.has(EvidenceIssue::Invalid);
        let mut f = Fixture::new();
        f.trial
            .manifest
            .checks
            .0
            .get_mut("api")
            .unwrap()
            .description = "weakened".into();
        f.has(EvidenceIssue::Invalid);
        let mut f = Fixture::new();
        f.trial
            .check_definition_digests
            .0
            .insert("api".into(), "altered".into());
        f.has(EvidenceIssue::Invalid);
    }
    #[test]
    fn required_missing_checks_results_and_errors_are_incomplete() {
        let mut f = Fixture::new();
        f.evidence.checks.0.clear();
        f.has(EvidenceIssue::Incomplete);
        let mut f = Fixture::new();
        f.evidence.checks.0.get_mut("api").unwrap().baseline = None;
        f.has(EvidenceIssue::Incomplete);
        for status in [CheckStatus::Missing, CheckStatus::Error] {
            let mut f = Fixture::new();
            f.candidate().status = status;
            f.has(EvidenceIssue::Incomplete);
        }
    }
    #[test]
    fn artifacts_require_actual_bytes_paths_and_expected_ids() {
        let f = Fixture::new();
        fs::write(f.root.join("raw.log"), "tampered").unwrap();
        f.has(EvidenceIssue::Invalid);
        let mut f = Fixture::new();
        f.candidate().artifacts.0.clear();
        f.has(EvidenceIssue::Incomplete);
        let f = Fixture::new();
        fs::remove_file(f.root.join("raw.log")).unwrap();
        f.has(EvidenceIssue::Incomplete);
        for path in [
            "../raw.log",
            "/raw.log",
            "C:/raw.log",
            "nested/../raw.log",
            "raw\\log",
        ] {
            let mut f = Fixture::new();
            f.candidate().artifacts.0.get_mut("log").unwrap().path = path.into();
            f.has(EvidenceIssue::Invalid);
        }
    }
    #[test]
    fn differential_identity_oracle_and_expected_counts_are_bound() {
        for field in [
            "baseline_identity",
            "candidate_identity",
            "oracle_digest",
            "expected",
        ] {
            let mut f = Fixture::new();
            let mut value = serde_json::to_value(&*f.candidate()).unwrap();
            value["summary"][field] = if field == "expected" {
                json!(4)
            } else {
                json!("stale")
            };
            *f.candidate() = serde_json::from_value(value).unwrap();
            f.has(EvidenceIssue::Invalid);
        }
    }
    #[test]
    fn observed_mismatches_reject_even_with_passed_status_and_missing_check() {
        let mut f = Fixture::new();
        if let CheckSummary::Differential { mismatches, .. } = &mut f.candidate().summary {
            *mismatches = Count(1);
        }
        f.evidence.checks.0.get_mut("api").unwrap().baseline = None;
        f.has(EvidenceIssue::RejectedBehavior);
        f.has(EvidenceIssue::Incomplete);
    }
    #[test]
    fn short_or_zero_runs_are_incomplete_and_collector_exit_is_invalid() {
        for count in [0, 2] {
            let mut f = Fixture::new();
            if let CheckSummary::Differential { observed, .. } = &mut f.candidate().summary {
                *observed = Count(count)
            }
            f.has(EvidenceIssue::Incomplete);
        }
        let mut f = Fixture::new();
        f.candidate().summary = CheckSummary::ExitStatus { code: Some(0) };
        f.has(EvidenceIssue::Invalid);
    }
    #[test]
    fn baseline_failure_and_optional_candidate_regression_are_preserved() {
        let mut f = Fixture::new();
        f.evidence
            .checks
            .0
            .get_mut("api")
            .unwrap()
            .baseline
            .as_mut()
            .unwrap()
            .status = CheckStatus::Failed;
        f.candidate().status = CheckStatus::Failed;
        f.has(EvidenceIssue::BaselineIneligible);
        f.has(EvidenceIssue::RejectedBehavior);
        let mut f = Fixture::new();
        f.trial.manifest.checks.0.get_mut("api").unwrap().required = false;
        let digest = frozen_trial_digest(&f.trial);
        let p = f.evidence.checks.0.get_mut("api").unwrap();
        p.baseline.as_mut().unwrap().frozen_trial_digest = digest.clone();
        p.candidate.as_mut().unwrap().frozen_trial_digest = digest;
        f.candidate().status = CheckStatus::Failed;
        f.has(EvidenceIssue::RejectedBehavior);
    }
    #[test]
    fn exit_status_and_case_counts_follow_declared_interpretation() {
        for interpretation in [
            Interpretation::ExitStatus,
            Interpretation::CaseCounts { expected: Count(3) },
        ] {
            let mut f = Fixture::new();
            f.trial
                .manifest
                .checks
                .0
                .get_mut("api")
                .unwrap()
                .interpretation = interpretation.clone();
            let digest = frozen_trial_digest(&f.trial);
            let pair = f.evidence.checks.0.get_mut("api").unwrap();
            for result in [&mut pair.baseline, &mut pair.candidate] {
                let r = result.as_mut().unwrap();
                r.frozen_trial_digest = digest.clone();
                r.summary = match interpretation {
                    Interpretation::ExitStatus => CheckSummary::ExitStatus { code: Some(0) },
                    _ => CheckSummary::CaseCounts {
                        expected: Count(3),
                        observed: Count(3),
                        mismatches: Count(0),
                    },
                };
            }
            assert!(f.run().findings.is_empty());
            f.candidate().summary = match interpretation {
                Interpretation::ExitStatus => CheckSummary::ExitStatus { code: Some(1) },
                _ => CheckSummary::CaseCounts {
                    expected: Count(3),
                    observed: Count(3),
                    mismatches: Count(1),
                },
            };
            f.has(EvidenceIssue::RejectedBehavior);
        }
    }
    #[test]
    fn empty_optional_inventory_results_and_zero_expected_are_incomplete() {
        let mut f = Fixture::new();
        f.trial.manifest.checks.0.get_mut("api").unwrap().required = false;
        f.evidence.checks.0.clear();
        f.has(EvidenceIssue::Incomplete);
        let mut f = Fixture::new();
        f.trial
            .manifest
            .checks
            .0
            .get_mut("api")
            .unwrap()
            .interpretation = Interpretation::CaseCounts { expected: Count(0) };
        let digest = frozen_trial_digest(&f.trial);
        let pair = f.evidence.checks.0.get_mut("api").unwrap();
        for result in [&mut pair.baseline, &mut pair.candidate] {
            let r = result.as_mut().unwrap();
            r.frozen_trial_digest = digest.clone();
            r.summary = CheckSummary::CaseCounts {
                expected: Count(0),
                observed: Count(0),
                mismatches: Count(0),
            };
        }
        f.has(EvidenceIssue::Incomplete);
    }
    #[test]
    fn artifact_roots_cannot_ambiguously_bind_the_same_relative_path() {
        let f = Fixture::new();
        let other = Fixture::new();
        let admitted = admit_evidence(
            &f.trial,
            &f.candidate,
            &f.evidence,
            &[f.root.clone(), other.root.clone()],
        );
        assert!(admitted
            .findings
            .iter()
            .any(|finding| finding.issue == EvidenceIssue::Invalid));
    }
    #[cfg(unix)]
    #[test]
    fn artifact_symlink_cannot_escape_approved_root() {
        use std::os::unix::fs::symlink;
        let f = Fixture::new();
        let external = Fixture::new();
        fs::remove_file(f.root.join("raw.log")).unwrap();
        symlink(external.root.join("raw.log"), f.root.join("raw.log")).unwrap();
        f.has(EvidenceIssue::Invalid);
    }
    fn optional_pair_fixture() -> Fixture {
        let mut f = Fixture::new();
        f.trial.manifest.checks.0.get_mut("api").unwrap().required = false;
        let digest = frozen_trial_digest(&f.trial);
        let pair = f.evidence.checks.0.get_mut("api").unwrap();
        pair.baseline.as_mut().unwrap().frozen_trial_digest = digest.clone();
        pair.candidate.as_mut().unwrap().frozen_trial_digest = digest;
        f
    }
    #[test]
    fn baseline_only_optional_evidence_is_incomplete() {
        let mut f = optional_pair_fixture();
        f.evidence.checks.0.get_mut("api").unwrap().candidate = None;
        f.has(EvidenceIssue::Incomplete);
        f.evidence
            .checks
            .0
            .get_mut("api")
            .unwrap()
            .baseline
            .as_mut()
            .unwrap()
            .status = CheckStatus::Failed;
        f.has(EvidenceIssue::BaselineIneligible);
        f.has(EvidenceIssue::Incomplete);
    }
    #[test]
    fn candidate_only_optional_evidence_is_incomplete() {
        let mut f = optional_pair_fixture();
        f.evidence.checks.0.get_mut("api").unwrap().baseline = None;
        f.has(EvidenceIssue::Incomplete);
        f.candidate().status = CheckStatus::Failed;
        f.has(EvidenceIssue::RejectedBehavior);
        f.has(EvidenceIssue::Incomplete);
    }
    #[test]
    fn absent_optional_check_is_allowed_with_another_complete_pair() {
        let mut f = Fixture::new();
        let mut optional = f.trial.manifest.checks.0["api"].clone();
        optional.required = false;
        f.trial
            .manifest
            .checks
            .0
            .insert("optional".into(), optional);
        f.trial
            .check_definition_digests
            .0
            .insert("optional".into(), "check-v1".into());
        let digest = frozen_trial_digest(&f.trial);
        let pair = f.evidence.checks.0.get_mut("api").unwrap();
        pair.baseline.as_mut().unwrap().frozen_trial_digest = digest.clone();
        pair.candidate.as_mut().unwrap().frozen_trial_digest = digest;
        assert!(f.run().findings.is_empty());
    }
}
