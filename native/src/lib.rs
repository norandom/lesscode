//! Native capability-compression package.
//!
//! Read-only preparation and comparison of supplied capability pairs.

pub use ast_grep_core::AstGrep;
pub use ast_grep_language::{Python, Tsx, TypeScript};

pub mod adapter_traits;
pub mod admission;
pub mod budgets;
pub mod compare;
pub mod jsonval;
pub mod measures;
pub mod metrics;
pub mod model;
pub mod receipt;
pub mod recording;
pub mod regenerator;
pub mod scope;
pub mod state_contract;
pub mod surface;
use model::*;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct InputError(pub String);
impl std::fmt::Display for InputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for InputError {}
impl From<scope::ScopeError> for InputError {
    fn from(error: scope::ScopeError) -> Self {
        Self(error.0)
    }
}

const EVALUATOR: &str = "lesscode-core@0.1.0;workflow.v1";

/// Freeze the baseline under current definitions, verifying any supplied
/// capability-inventory artifact against explicit external artifact directories.
pub fn prepare(
    manifest: Manifest,
    baseline: &Path,
    artifact_roots: &[PathBuf],
) -> Result<FrozenTrial, InputError> {
    let snapshot = scope::inventory(baseline, &manifest)?;
    scope::resolve_scope(baseline, &snapshot, &manifest, &Records(BTreeMap::new()))?;
    if let Some(reference) = &manifest.capability_inventory {
        let artifact = compare::resolve_artifact(&reference.artifact, artifact_roots)
            .map_err(|(_, e)| InputError(e))?;
        scope::verify_capability_inventory(&snapshot, reference, &artifact)?;
    }
    let trial = FrozenTrial {
        schema_version: SchemaVersion::V1,
        check_definition_digests: Records(
            manifest
                .checks
                .0
                .iter()
                .map(|(id, check)| (id.clone(), check.definition_digest.clone()))
                .collect(),
        ),
        manifest,
        baseline_files: snapshot.files.clone(),
        baseline_identity: snapshot.identity.clone(),
        evaluator_identity: EVALUATOR.into(),
        definitions: definitions(),
    };
    validate_frozen(&trial)?;
    recheck(baseline, &trial.manifest, &snapshot, "baseline")?;
    Ok(trial)
}
/// Measure both source roots and admit supplied evidence from explicit external
/// artifact directories. This function executes no project checks or writes.
/// `accounting.capability_metrics` keys are `all_categories/{Language}/{Metric}`
/// and `production/{Language}/{Metric}` (Rust enum names). The production subset
/// is the exact replacement scope used for targets. `wider_metrics` and
/// `category_totals` cover the wider context and use `{Language}/{Metric}` keys.
/// A supplied capability inventory remains bound to the frozen baseline; the
/// candidate uses the same classification rules without inheriting that binding.
pub fn compare(
    trial: &FrozenTrial,
    baseline: &Path,
    candidate: &Path,
    evidence: &EvidenceBundle,
    artifact_roots: &[PathBuf],
) -> Result<Comparison, InputError> {
    compare_inner(trial, baseline, candidate, evidence, artifact_roots, || {})
}

fn hash(value: &impl serde::Serialize) -> String {
    Sha256::digest(serde_json::to_vec(value).expect("closed records serialize"))
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn language(path: &str) -> Option<Language> {
    match Path::new(path).extension().and_then(|e| e.to_str()) {
        Some("py" | "pyi") => Some(Language::Python),
        Some("ts" | "mts" | "cts") => Some(Language::TypeScript),
        Some("tsx") => Some(Language::Tsx),
        _ => None,
    }
}
fn syntax(
    path: &str,
    bytes: &[u8],
    identity: &str,
    spans: &[scope::SourceSpan],
    lang: Language,
) -> Records<MetricName, MetricMeasurement> {
    match lang {
        Language::Python => metrics::measure_python(path, bytes, identity, spans),
        Language::TypeScript | Language::Tsx => {
            metrics::measure_typescript(path, bytes, identity, spans, lang == Language::Tsx)
        }
    }
}
fn key(language: Language, name: MetricName) -> String {
    format!("{language:?}/{name:?}")
}
fn definitions() -> Records<String, MetricIdentity> {
    let mut result = BTreeMap::new();
    for lang in [Language::Python, Language::TypeScript, Language::Tsx] {
        for (name, m) in syntax("", b"", "", &[], lang).0 {
            result.insert(key(lang, name), m.identity);
        }
        let call = metrics::measure_call_layers(&[], &[], &[], "", lang).measurement;
        result.insert(key(lang, MetricName::InternalCallLayers), call.identity);
    }
    Records(result)
}
fn validate_frozen(trial: &FrozenTrial) -> Result<(), InputError> {
    if trial.evaluator_identity != EVALUATOR || trial.definitions != definitions() {
        return Err(InputError(
            "frozen evaluator or metric definitions changed; prepare a new trial".into(),
        ));
    }
    let checks = Records(
        trial
            .manifest
            .checks
            .0
            .iter()
            .map(|(id, c)| (id.clone(), c.definition_digest.clone()))
            .collect(),
    );
    if trial.check_definition_digests != checks {
        return Err(InputError("frozen check inventory changed".into()));
    }
    if let Some(target) = &trial.manifest.target {
        if target.scope_identity.trim().is_empty() {
            return Err(InputError("target scope identity is empty".into()));
        }
        for (name, definition) in &target.definitions.0 {
            if definition.language != target.language
                || definition.name != *name
                || trial.definitions.0.get(&key(target.language, *name)) != Some(definition)
            {
                return Err(InputError(
                    "frozen target metric definitions changed".into(),
                ));
            }
        }
    }
    Ok(())
}
fn recheck(
    root: &Path,
    manifest: &Manifest,
    expected: &scope::Snapshot,
    side: &str,
) -> Result<(), InputError> {
    if scope::inventory(root, manifest)? != *expected {
        return Err(InputError(format!(
            "{side} source changed during comparison"
        )));
    }
    Ok(())
}
type Sources = BTreeMap<String, Vec<u8>>;
fn sources(root: &Path, snapshot: &scope::Snapshot) -> Result<Sources, InputError> {
    snapshot
        .files
        .0
        .keys()
        .map(|path| {
            Ok((
                path.clone(),
                std::fs::read(root.join(path)).map_err(|e| InputError(format!("{path}: {e}")))?,
            ))
        })
        .collect()
}
fn scope_id(snapshot: &scope::Snapshot, spans: &[scope::SourceSpan]) -> String {
    hash(&(
        snapshot.identity.as_str(),
        spans
            .iter()
            .map(|s| (&s.path, s.start, s.end))
            .collect::<Vec<_>>(),
    ))
}

fn measure(
    sources: &Sources,
    spans: &[scope::SourceSpan],
    entries: &[scope::SourceSpan],
    identity: &str,
    languages: &BTreeSet<Language>,
) -> (Records<String, MetricMeasurement>, Records<String, Finding>) {
    let mut result = BTreeMap::new();
    let mut unresolved = BTreeMap::new();
    let calls: Vec<_> = sources
        .iter()
        .filter_map(|(path, bytes)| {
            language(path).map(|language| metrics::CallSource {
                path,
                source: bytes,
                language,
            })
        })
        .collect();
    for lang in languages {
        let mut aggregate = syntax("", b"", identity, &[], *lang);
        for (path, bytes) in sources.iter().filter(|(path, _)| {
            language(path) == Some(*lang) && spans.iter().any(|s| s.path == **path)
        }) {
            for (name, value) in syntax(path, bytes, identity, spans, *lang).0 {
                let total = aggregate.0.get_mut(&name).expect("same definition set");
                total.value = match (&total.value, value.value) {
                    (MetricValue::Available { count: a }, MetricValue::Available { count: b }) => a
                        .checked_add(b)
                        .map(|count| MetricValue::Available { count })
                        .unwrap_or(MetricValue::Unavailable {
                            reason: "count overflow".into(),
                            resolved_lower_bound: None,
                        }),
                    (MetricValue::Unavailable { .. }, _) => total.value.clone(),
                    (_, unavailable) => unavailable,
                };
            }
        }
        for (name, m) in aggregate.0 {
            result.insert(key(*lang, name), m);
        }
        let layers = metrics::measure_call_layers(&calls, spans, entries, identity, *lang);
        result.insert(
            key(*lang, MetricName::InternalCallLayers),
            layers.measurement,
        );
        unresolved.extend(layers.unresolved.0);
    }
    (Records(result), Records(unresolved))
}
fn pairs(
    before: Records<String, MetricMeasurement>,
    mut after: Records<String, MetricMeasurement>,
) -> Records<String, MetricPair> {
    Records(
        before
            .0
            .into_iter()
            .map(|(key, before)| {
                let after = after.0.remove(&key).expect("same language definitions");
                let delta = match (&before.value, &after.value) {
                    (MetricValue::Available { count: a }, MetricValue::Available { count: b })
                        if before.identity == after.identity =>
                    {
                        b.checked_delta(*a)
                    }
                    _ => None,
                };
                (
                    key,
                    MetricPair {
                        schema_version: SchemaVersion::V1,
                        before,
                        after,
                        delta,
                    },
                )
            })
            .collect(),
    )
}
fn compare_inner(
    trial: &FrozenTrial,
    baseline: &Path,
    candidate: &Path,
    evidence: &EvidenceBundle,
    artifact_roots: &[PathBuf],
    before_recheck: impl FnOnce(),
) -> Result<Comparison, InputError> {
    validate_frozen(trial)?;
    let original = scope::inventory(baseline, &trial.manifest)?;
    if original.identity != trial.baseline_identity || original.files != trial.baseline_files {
        return Err(InputError(
            "baseline no longer matches frozen source".into(),
        ));
    }
    if let Some(reference) = &trial.manifest.capability_inventory {
        let artifact = compare::resolve_artifact(&reference.artifact, artifact_roots)
            .map_err(|(_, e)| InputError(e))?;
        scope::verify_capability_inventory(&original, reference, &artifact)?;
    }
    // The repository inventory describes the baseline snapshot, not a claim that
    // the edited candidate has the same identity. Only this reference is removed
    // from the candidate inventory input; frozen scope/classification stay intact.
    let mut candidate_manifest = trial.manifest.clone();
    candidate_manifest.capability_inventory = None;
    let replacement = scope::inventory(candidate, &candidate_manifest)?;
    let accounting = scope::account_replacement(
        scope::SourceSide {
            root: baseline,
            snapshot: &original,
        },
        scope::SourceSide {
            root: candidate,
            snapshot: &replacement,
        },
        &trial.manifest,
        evidence,
    )?;
    let original_sources = sources(baseline, &original)?;
    let replacement_sources = sources(candidate, &replacement)?;
    let entries: Vec<_> =
        scope::resolve_selections(baseline, &original, &trial.manifest.entry_points)?
            .0
            .into_values()
            .collect();
    // Replacement ownership is the declared candidate entry boundary: old paths
    // may be removed, so baseline selectors must never be resolved in candidate.
    let candidate_entries = &accounting.candidate.selected;
    let mut unresolved = BTreeMap::new();
    let mut required_relationships = BTreeSet::new();
    let mut paired =
        |before: &[scope::SourceSpan], after: &[scope::SourceSpan], review_required: bool| {
            let languages = before
                .iter()
                .chain(after)
                .filter_map(|s| language(&s.path))
                .collect();
            let (b, bu) = measure(
                &original_sources,
                before,
                &entries,
                &scope_id(&original, before),
                &languages,
            );
            let (a, au) = measure(
                &replacement_sources,
                after,
                candidate_entries,
                &scope_id(&replacement, after),
                &languages,
            );
            for (side, findings) in [("baseline", bu), ("candidate", au)] {
                for (id, finding) in findings.0 {
                    let id = format!("{side}/{id}");
                    if review_required {
                        required_relationships.insert(id.clone());
                    }
                    unresolved.insert(id, finding);
                }
            }
            pairs(b, a)
        };
    let all_capability_metrics = paired(
        &accounting.baseline.capability,
        &accounting.candidate.capability,
        true,
    );
    let wider_metrics = paired(
        &accounting.baseline.wider,
        &accounting.candidate.wider,
        false,
    );
    let mut categories = BTreeMap::new();
    for category in [
        Category::Production,
        Category::Tests,
        Category::Fixtures,
        Category::ResearchSupport,
        Category::Configuration,
        Category::Dependency,
    ] {
        let filter = |side: &scope::SideAccounting| {
            side.wider
                .iter()
                .filter(|s| side.categories.get(&s.path) == Some(&category))
                .cloned()
                .collect::<Vec<_>>()
        };
        categories.insert(
            category,
            paired(
                &filter(&accounting.baseline),
                &filter(&accounting.candidate),
                false,
            ),
        );
    }
    let filter = |side: &scope::SideAccounting| {
        side.capability
            .iter()
            .filter(|s| side.categories.get(&s.path) == Some(&Category::Production))
            .cloned()
            .collect::<Vec<_>>()
    };
    let bprod = filter(&accounting.baseline);
    let aprod = filter(&accounting.candidate);
    let production = paired(&bprod, &aprod, true);
    // Expose the exact production subset used for the target as well as totals
    // across capability categories; wider category totals include context.
    let capability_metrics = Records(
        all_capability_metrics
            .0
            .into_iter()
            .map(|(key, pair)| (format!("all_categories/{key}"), pair))
            .chain(
                production
                    .0
                    .iter()
                    .map(|(key, pair)| (format!("production/{key}"), pair.clone())),
            )
            .collect(),
    );
    let mut findings: Vec<_> = accounting
        .findings
        .iter()
        .cloned()
        .map(|finding| compare::DecisionFinding {
            status: ComparisonStatus::Incomplete,
            finding,
        })
        .collect();
    if let Some(target) = &trial.manifest.target {
        for path in bprod
            .iter()
            .chain(&aprod)
            .map(|span| &span.path)
            .collect::<BTreeSet<_>>()
        {
            if language(path).is_some_and(|language| language != target.language) {
                findings.push(compare::DecisionFinding {status:ComparisonStatus::Incomplete,finding:Finding {schema_version:SchemaVersion::V1,subject:path.clone(),explanation:"production replacement includes a language outside the frozen target; separate language counts cannot establish a whole-capability reduction".into()}});
            }
        }
    }
    let mut limitations = Vec::new();
    for (side, source, spans, categories) in [
        (
            "baseline",
            &original_sources,
            &accounting.baseline.wider,
            &accounting.baseline.categories,
        ),
        (
            "candidate",
            &replacement_sources,
            &accounting.candidate.wider,
            &accounting.candidate.categories,
        ),
    ] {
        for (path, bytes) in source
            .iter()
            .filter(|(path, _)| language(path).is_none() && spans.iter().any(|s| s.path == **path))
        {
            let mut start = 0;
            let count = bytes
                .split_inclusive(|b| *b == b'\n')
                .filter(|line| {
                    let end = start + line.len();
                    let included = spans
                        .iter()
                        .any(|s| s.path == *path && s.start < end && start < s.end);
                    start = end;
                    included
                })
                .count();
            limitations.push(format!("{side}: {path}; category={:?}; physical_lines.v1={count}; structural metrics unavailable: unsupported language; no language-neutral metric identity in schema v1",categories[path]));
            if categories[path] == Category::Production
                && (if side == "baseline" { &bprod } else { &aprod })
                    .iter()
                    .any(|s| s.path == *path)
            {
                findings.push(compare::DecisionFinding {status:ComparisonStatus::Incomplete,finding:Finding {schema_version:SchemaVersion::V1,subject:format!("{side}/{path}"),explanation:"production structural metrics unavailable: unsupported language".into()}});
            }
        }
    }
    for (id, finding) in &unresolved {
        if !required_relationships.contains(id) {
            continue;
        }
        let (side, relationship) = id.split_once('/').expect("side prefix");
        let review = if side == "baseline" {
            &trial.manifest.scope_review
        } else {
            &evidence.candidate_ownership.review
        };
        if !review.relationships.0.contains_key(relationship) {
            findings.push(compare::DecisionFinding {
                status: ComparisonStatus::Incomplete,
                finding: Finding {
                    schema_version: SchemaVersion::V1,
                    subject: id.clone(),
                    explanation: format!(
                        "unresolved relationship has no ownership review: {}",
                        finding.explanation
                    ),
                },
            });
        }
    }
    let mut admission =
        compare::admit_evidence(trial, &replacement.identity, evidence, artifact_roots);
    admission.limitations.extend(limitations);
    let target_language = trial.manifest.target.as_ref().map(|t| t.language);
    let target_metrics = Records(
        production
            .0
            .into_iter()
            .filter(|(_, p)| Some(p.before.identity.language) == target_language)
            .collect(),
    );
    let decision = compare::decide_comparison(
        trial.manifest.target.as_ref(),
        compare::TargetMeasurements {
            scope_identity: trial
                .manifest
                .target
                .as_ref()
                .map_or("production", |t| t.scope_identity.as_str()),
            baseline_scope_identity: &scope_id(&original, &bprod),
            candidate_scope_identity: &scope_id(&replacement, &aprod),
            metrics: &target_metrics,
        },
        &admission,
        &findings,
    );
    let paths: BTreeSet<_> = original
        .files
        .0
        .keys()
        .chain(replacement.files.0.keys())
        .cloned()
        .collect();
    before_recheck();
    recheck(baseline, &trial.manifest, &original, "baseline")?;
    recheck(candidate, &candidate_manifest, &replacement, "candidate")?;
    Ok(Comparison {
        schema_version: SchemaVersion::V1,
        trial_id: trial.manifest.trial_id.clone(),
        hypothesis: trial.manifest.hypothesis.clone(),
        baseline_identity: original.identity,
        candidate_identity: replacement.identity,
        frozen_trial_digest: compare::frozen_trial_digest(trial),
        status: decision.status,
        reasons: decision.findings.into_iter().map(|f| f.finding).collect(),
        accounting: ScopeAccounting {
            schema_version: SchemaVersion::V1,
            files: Records(
                paths
                    .into_iter()
                    .map(|p| {
                        let f = FileAccounting {
                            schema_version: SchemaVersion::V1,
                            baseline: original.files.0.get(&p).cloned(),
                            candidate: replacement.files.0.get(&p).cloned(),
                        };
                        (p, f)
                    })
                    .collect(),
            ),
            capability_metrics,
            wider_metrics,
            category_totals: Records(categories),
            unresolved: Records(unresolved),
        },
        checks: admission.checks,
        check_findings: admission.findings.into_iter().map(|f| f.finding).collect(),
        dependency_changes: Records(accounting.dependency_changes),
        limitations: admission.limitations,
        provenance: vec![
            "Supplied runner evidence is consistency-checked, not authenticated execution.".into(),
        ],
    })
}

#[cfg(test)]
mod workflow_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn mutation_after_measurement_is_detected_on_either_side() {
        // This private synchronous seam mutates real fixture bytes at the exact
        // boundary before final inventory. There is no public runtime hook.
        let root = std::env::temp_dir().join(format!("lesscode-recheck-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let baseline = root.join("baseline");
        let candidate = root.join("candidate");
        std::fs::create_dir(&baseline).unwrap();
        std::fs::create_dir(&candidate).unwrap();
        let manifest:Manifest=serde_json::from_value(json!({"schema_version":1,"trial_id":"recheck","hypothesis":"constant","entry_points":{"run":{"kind":"whole_file","path":"a.py"}},"owned":{"run":{"kind":"whole_file","path":"a.py"}},"wider_boundary":["."],"categories":{"a.py":"production"},"exclusions":{},"scope_review":{"schema_version":1,"rationale":"complete","relationships":{}},"checks":{},"target":null,"capability_inventory":null})).unwrap();
        let evidence:EvidenceBundle=serde_json::from_value(json!({"schema_version":1,"candidate_ownership":{"schema_version":1,"replacements":{"run":[{"kind":"whole_file","path":"a.py"}]},"review":{"schema_version":1,"rationale":"complete","relationships":{}}},"checks":{},"dependency_changes":{}})).unwrap();
        for (side, path) in [("baseline", &baseline), ("candidate", &candidate)] {
            for path in [&baseline, &candidate] {
                std::fs::write(path.join("a.py"), "def run():\n    return 1\n").unwrap();
            }
            let trial = prepare(manifest.clone(), &baseline, &[]).unwrap();
            let error = compare_inner(&trial, &baseline, &candidate, &evidence, &[], || {
                std::fs::write(path.join("a.py"), "pass\n").unwrap()
            })
            .unwrap_err();
            assert!(
                error.0.contains(side) && error.0.contains("source changed"),
                "{error}"
            );
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}
