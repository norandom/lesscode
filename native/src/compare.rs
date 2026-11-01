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

/// A classified fact from scope, metric or evidence admission. All facts survive
/// selection of the final status, including lower-priority failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionFinding {
    pub status: ComparisonStatus,
    pub finding: Finding,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub status: ComparisonStatus,
    pub findings: Vec<DecisionFinding>,
}
/// Production-only measurements for one frozen capability and language. The
/// integration layer supplies snapshot-bound per-side scope identities; those
/// identities need not be equal when replacement source changes.
pub struct TargetMeasurements<'a> {
    pub scope_identity: &'a str,
    pub baseline_scope_identity: &'a str,
    pub candidate_scope_identity: &'a str,
    pub metrics: &'a Records<String, MetricPair>,
}

/// Pure decision over already admitted facts. This neither runs checks nor
/// measures source. Permitted structural tradeoffs remain frozen prose; only
/// explicit numeric count bounds are executable constraints.
pub fn decide_comparison(
    target: Option<&Target>,
    measurements: TargetMeasurements<'_>,
    evidence: &EvidenceAdmission,
    additional_findings: &[DecisionFinding],
) -> Decision {
    use ComparisonStatus::*;
    let mut findings = additional_findings.to_vec();
    findings.extend(evidence.findings.iter().map(|fact| DecisionFinding {
        status: match fact.issue {
            EvidenceIssue::Invalid => Invalid,
            EvidenceIssue::BaselineIneligible => BaselineIneligible,
            EvidenceIssue::RejectedBehavior => RejectedBehavior,
            EvidenceIssue::Incomplete => Incomplete,
        },
        finding: fact.finding.clone(),
    }));
    if !evidence
        .checks
        .0
        .values()
        .any(|pair| pair.baseline.is_some() && pair.candidate.is_some())
    {
        add_decision_finding(
            &mut findings,
            Incomplete,
            "checks",
            "no complete admitted behavioral evidence pair",
        );
    }
    if let Some(target) = target {
        if target.scope_identity != measurements.scope_identity || target.scope_identity.is_empty()
        {
            add_decision_finding(
                &mut findings,
                Invalid,
                "target",
                "frozen target capability scope does not match supplied production scope",
            );
        }
        let mut required: std::collections::BTreeSet<_> =
            target.definitions.0.keys().copied().collect();
        required.extend(target.count_bounds.0.keys().copied());
        required.insert(MetricName::LogicalStatements);
        for name in required {
            let subject = format!("production/{name:?}");
            let Some(definition) = target.definitions.0.get(&name) else {
                add_decision_finding(
                    &mut findings,
                    Incomplete,
                    &subject,
                    "required frozen metric definition missing",
                );
                continue;
            };
            let mut matching = measurements.metrics.0.values().filter(|pair| {
                pair.before.identity.name == name || pair.after.identity.name == name
            });
            let pair = matching.next();
            if pair.is_none() || matching.next().is_some() {
                add_decision_finding(
                    &mut findings,
                    Incomplete,
                    &subject,
                    "required production measurement missing or ambiguous",
                );
                continue;
            }
            let pair = pair.expect("exactly one matching measurement");
            if definition.name != name
                || definition.language != target.language
                || pair.before.identity != *definition
                || pair.after.identity != *definition
                || pair.before.scope_identity != measurements.baseline_scope_identity
                || pair.after.scope_identity != measurements.candidate_scope_identity
                || measurements.baseline_scope_identity.is_empty()
                || measurements.candidate_scope_identity.is_empty()
            {
                add_decision_finding(&mut findings, Incomplete, &subject, "required metric definitions, language or snapshot-bound scopes are incompatible");
                continue;
            }
            let (MetricValue::Available { count: before }, MetricValue::Available { count: after }) =
                (&pair.before.value, &pair.after.value)
            else {
                add_decision_finding(
                    &mut findings,
                    Incomplete,
                    &subject,
                    "required production metric unavailable; lower bounds are not totals",
                );
                continue;
            };
            let delta = after.checked_delta(*before);
            if delta.is_none() || pair.delta != delta {
                add_decision_finding(
                    &mut findings,
                    Incomplete,
                    &subject,
                    "required metric delta unavailable or inconsistent with checked counts",
                );
                continue;
            }
            if name == MetricName::LogicalStatements
                && (after >= before
                    || before.0.checked_sub(after.0).is_none_or(|reduction| {
                        reduction < target.minimum_production_statement_reduction.0
                    }))
            {
                add_decision_finding(&mut findings, RejectedTarget, &subject, "production statement reduction must be strictly positive and meet the frozen minimum");
            }
            if target
                .count_bounds
                .0
                .get(&name)
                .is_some_and(|bound| after > bound)
            {
                add_decision_finding(
                    &mut findings,
                    RejectedTarget,
                    &subject,
                    "candidate production count exceeds frozen absolute bound",
                );
            }
        }
    } else {
        add_decision_finding(
            &mut findings,
            NotDemonstrated,
            "target",
            "no frozen compression target; measurements remain descriptive",
        );
    }
    let status = [
        Invalid,
        BaselineIneligible,
        RejectedBehavior,
        Incomplete,
        RejectedTarget,
        NotDemonstrated,
    ]
    .into_iter()
    .find(|status| findings.iter().any(|finding| finding.status == *status))
    .unwrap_or(Accepted);
    Decision { status, findings }
}

fn add_decision_finding(
    findings: &mut Vec<DecisionFinding>,
    status: ComparisonStatus,
    subject: &str,
    explanation: &str,
) {
    findings.push(DecisionFinding {
        status,
        finding: Finding {
            schema_version: SchemaVersion::V1,
            subject: subject.into(),
            explanation: explanation.into(),
        },
    });
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

    fn decision_finding(status: ComparisonStatus, subject: &str) -> DecisionFinding {
        DecisionFinding {
            status,
            finding: Finding {
                schema_version: SchemaVersion::V1,
                subject: subject.into(),
                explanation: format!("{subject} finding"),
            },
        }
    }
    struct DecisionFixture {
        target: Target,
        metrics: Records<String, MetricPair>,
        admission: EvidenceAdmission,
    }
    impl DecisionFixture {
        fn new(before: u64, after: u64) -> Self {
            let id = json!({"schema_version":1,"name":"logical_statements","definition_revision":"v1",
                "language":"python","grammar_version":"grammar-v1","tool_version":"tool-v1"});
            let pair = json!({"schema_version":1,
                "before":{"schema_version":1,"identity":id,"scope_identity":"baseline-production",
                    "value":{"availability":"available","count":before}},
                "after":{"schema_version":1,"identity":id,"scope_identity":"candidate-production",
                    "value":{"availability":"available","count":after}},
                "delta":Count(after).checked_delta(Count(before))});
            Self {
                target: serde_json::from_value(
                    json!({"schema_version":1,"scope_identity":"production",
                    "language":"python","definitions":{"logical_statements":id},
                    "minimum_production_statement_reduction":3,"count_bounds":{},
                    "permitted_structural_tradeoffs":["one forwarding function may remain"]}),
                )
                .unwrap(),
                metrics: serde_json::from_value(json!({"statements":pair})).unwrap(),
                admission: EvidenceAdmission {
                    checks: decision_checks(),
                    findings: vec![],
                    limitations: vec!["finite checks".into()],
                },
            }
        }
        fn run(&self, findings: &[DecisionFinding]) -> Decision {
            decide_comparison(
                Some(&self.target),
                self.measurements(),
                &self.admission,
                findings,
            )
        }
        fn measurements(&self) -> TargetMeasurements<'_> {
            TargetMeasurements {
                scope_identity: "production",
                baseline_scope_identity: "baseline-production",
                candidate_scope_identity: "candidate-production",
                metrics: &self.metrics,
            }
        }
        fn pair(&mut self) -> &mut MetricPair {
            self.metrics.0.get_mut("statements").unwrap()
        }
    }

    fn decision_checks() -> Records<String, PairedCheck> {
        let result = json!({"schema_version":1,"source_identity":"already-admitted-source",
            "frozen_trial_digest":"already-admitted-trial","definition_digest":"already-admitted-check",
            "status":"passed","artifacts":{},"tested_domain":"reviewed API case","limitations":[],
            "summary":{"kind":"exit_status","code":0}});
        serde_json::from_value(
            json!({"api":{"schema_version":1,"baseline":result,"candidate":result}}),
        )
        .unwrap()
    }

    #[test]
    fn absent_behavioral_evidence_prevents_acceptance_without_erasing_metrics() {
        let mut fixture = DecisionFixture::new(10, 5);
        fixture.admission.checks.0.clear();
        let metrics = fixture.metrics.clone();
        assert_eq!(fixture.run(&[]).status, ComparisonStatus::Incomplete);
        assert_eq!(fixture.metrics, metrics);
    }

    #[test]
    fn decision_precedence_retains_every_conflicting_finding() {
        use ComparisonStatus::*;
        let fixture = DecisionFixture::new(10, 5);
        let ordered = [
            Invalid,
            BaselineIneligible,
            RejectedBehavior,
            Incomplete,
            RejectedTarget,
            NotDemonstrated,
        ];
        for first in 0..ordered.len() {
            let findings: Vec<_> = ordered[first..]
                .iter()
                .rev()
                .map(|status| decision_finding(*status, &format!("{status:?}")))
                .collect();
            let decision = fixture.run(&findings);
            assert_eq!(decision.status, ordered[first]);
            for finding in findings {
                assert!(decision.findings.contains(&finding));
            }
        }
        assert_eq!(fixture.run(&[]).status, Accepted);
    }

    #[test]
    fn evidence_admission_failures_are_never_offset_by_savings() {
        use ComparisonStatus::*;
        for (issue, status) in [
            (EvidenceIssue::Invalid, Invalid),
            (EvidenceIssue::BaselineIneligible, BaselineIneligible),
            (EvidenceIssue::RejectedBehavior, RejectedBehavior),
            (EvidenceIssue::Incomplete, Incomplete),
        ] {
            let mut fixture = DecisionFixture::new(100, 1);
            fixture
                .admission
                .finding(issue, "check", "raw behavioral finding");
            let decision = fixture.run(&[decision_finding(Incomplete, "scope")]);
            assert_eq!(decision.status, status);
            assert!(decision
                .findings
                .iter()
                .any(|f| f.finding.explanation == "raw behavioral finding"));
        }
    }

    #[test]
    fn frozen_statement_threshold_requires_positive_production_reduction() {
        use ComparisonStatus::*;
        for (before, after, minimum, expected) in [
            (10, 7, 3, Accepted),
            (10, 8, 3, RejectedTarget),
            (10, 10, 0, RejectedTarget),
            (0, 0, 0, RejectedTarget),
            (0, 1, 0, RejectedTarget),
            (1, 0, 0, Accepted),
            (1, u64::MAX, 1, Incomplete),
            (u64::MAX, 0, 1, Incomplete),
        ] {
            let mut fixture = DecisionFixture::new(before, after);
            fixture.target.minimum_production_statement_reduction = Count(minimum);
            assert_eq!(
                fixture.run(&[]).status,
                expected,
                "{before} -> {after}, minimum {minimum}"
            );
        }
    }

    #[test]
    fn no_target_is_descriptive_and_metrics_are_retained() {
        let fixture = DecisionFixture::new(10, 1);
        let original = fixture.metrics.clone();
        let result = decide_comparison(None, fixture.measurements(), &fixture.admission, &[]);
        assert_eq!(result.status, ComparisonStatus::NotDemonstrated);
        assert_eq!(fixture.metrics, original);
    }

    #[test]
    fn unavailable_missing_incompatible_and_ambiguous_metrics_remain_incomplete() {
        for case in 0..9 {
            let mut fixture = DecisionFixture::new(10, 5);
            match case {
                0 => fixture.metrics.0.clear(),
                1 => {
                    fixture.pair().after.value = MetricValue::Unavailable {
                        reason: "parse error".into(),
                        resolved_lower_bound: Some(Count(0)),
                    };
                    fixture.pair().delta = None;
                }
                2 => {
                    fixture.pair().after.identity.definition_revision = "changed".into();
                    fixture.pair().delta = None;
                }
                3 => fixture.pair().before.scope_identity = "unbound source".into(),
                4 => fixture.pair().after.scope_identity = "unbound source".into(),
                5 => {
                    fixture.pair().before.identity.language = Language::TypeScript;
                    fixture.pair().after.identity.language = Language::TypeScript;
                }
                6 => {
                    let pair = fixture.pair().clone();
                    fixture.metrics.0.insert("duplicate".into(), pair);
                }
                7 => fixture.target.definitions.0.clear(),
                8 => fixture.pair().delta = Some(-99),
                _ => unreachable!(),
            }
            let original = fixture.metrics.clone();
            assert_eq!(
                fixture.run(&[]).status,
                ComparisonStatus::Incomplete,
                "case {case}"
            );
            assert_eq!(fixture.metrics, original);
        }
    }

    #[test]
    fn frozen_scope_and_definitions_cannot_be_substituted() {
        let mut fixture = DecisionFixture::new(10, 5);
        fixture.target.scope_identity = "different capability".into();
        assert_eq!(fixture.run(&[]).status, ComparisonStatus::Invalid);
        fixture.target.scope_identity = "production".into();
        fixture
            .target
            .definitions
            .0
            .get_mut(&MetricName::LogicalStatements)
            .unwrap()
            .tool_version = "other tool".into();
        assert_eq!(fixture.run(&[]).status, ComparisonStatus::Incomplete);
    }

    #[test]
    fn numeric_bounds_are_absolute_and_tradeoffs_do_not_waive_them() {
        let mut fixture = DecisionFixture::new(10, 5);
        let mut functions = fixture.pair().clone();
        functions.before.identity.name = MetricName::Functions;
        functions.after.identity.name = MetricName::Functions;
        functions.before.value = MetricValue::Available { count: Count(2) };
        functions.after.value = MetricValue::Available { count: Count(3) };
        functions.delta = Some(1);
        fixture
            .target
            .definitions
            .0
            .insert(MetricName::Functions, functions.before.identity.clone());
        fixture.metrics.0.insert("functions".into(), functions);
        fixture
            .target
            .count_bounds
            .0
            .insert(MetricName::Functions, Count(3));
        assert_eq!(fixture.run(&[]).status, ComparisonStatus::Accepted);
        fixture
            .target
            .count_bounds
            .0
            .insert(MetricName::Functions, Count(2));
        assert_eq!(fixture.run(&[]).status, ComparisonStatus::RejectedTarget);
        fixture.metrics.0.remove("functions");
        assert_eq!(fixture.run(&[]).status, ComparisonStatus::Incomplete);
    }

    #[test]
    fn formatting_relocation_or_splitting_alone_cannot_be_accepted() {
        let mut fixture = DecisionFixture::new(10, 10);
        fixture.target.minimum_production_statement_reduction = Count(0);
        let mut lines = fixture.pair().clone();
        lines.before.identity.name = MetricName::PhysicalLines;
        lines.after.identity.name = MetricName::PhysicalLines;
        lines.before.value = MetricValue::Available { count: Count(100) };
        lines.after.value = MetricValue::Available { count: Count(1) };
        lines.delta = Some(-99);
        fixture.metrics.0.insert("lines".into(), lines);
        assert_eq!(fixture.run(&[]).status, ComparisonStatus::RejectedTarget);
    }

    #[test]
    fn behavior_missing_metrics_and_multiple_target_failures_are_all_retained() {
        let mut fixture = DecisionFixture::new(10, 9);
        fixture
            .target
            .count_bounds
            .0
            .insert(MetricName::LogicalStatements, Count(0));
        fixture
            .target
            .count_bounds
            .0
            .insert(MetricName::Functions, Count(0));
        fixture.admission.finding(
            EvidenceIssue::RejectedBehavior,
            "optional-check",
            "known regression",
        );
        fixture
            .admission
            .finding(EvidenceIssue::Incomplete, "required-check", "missing run");
        let decision = fixture.run(&[]);
        assert_eq!(decision.status, ComparisonStatus::RejectedBehavior);
        assert_eq!(
            decision
                .findings
                .iter()
                .filter(|fact| fact.status == ComparisonStatus::RejectedTarget)
                .count(),
            2
        );
        assert_eq!(
            decision
                .findings
                .iter()
                .filter(|fact| fact.status == ComparisonStatus::Incomplete)
                .count(),
            2
        );
        assert!(decision
            .findings
            .iter()
            .any(|fact| fact.finding.explanation == "known regression"));
        assert!(decision
            .findings
            .iter()
            .any(|fact| fact.finding.explanation == "missing run"));
    }
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
