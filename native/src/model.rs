//! Closed records shared by inventory, measurement and comparison.
//!
//! These records describe supplied facts; they never execute checks or decide
//! whether the evidence establishes compression.

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

/// The only supported native record version. Missing versions are invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct SchemaVersion(u8);

impl SchemaVersion {
    pub const V1: Self = Self(1);
}

impl<'de> Deserialize<'de> for SchemaVersion {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let version = u8::deserialize(deserializer)?;
        if version == 1 {
            Ok(Self::V1)
        } else {
            Err(serde::de::Error::custom("unsupported schema_version"))
        }
    }
}

/// Sorted keyed records; duplicate JSON keys are errors rather than last-write wins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Records<K: Ord, V>(pub BTreeMap<K, V>);

impl<'de, K, V> Deserialize<'de> for Records<K, V>
where
    K: Ord + Deserialize<'de>,
    V: Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor<K, V>(std::marker::PhantomData<(K, V)>);
        impl<'de, K: Ord + Deserialize<'de>, V: Deserialize<'de>> serde::de::Visitor<'de>
            for Visitor<K, V>
        {
            type Value = Records<K, V>;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a map of unique record identities")
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Self::Value, A::Error> {
                let mut records = BTreeMap::new();
                while let Some((key, value)) = map.next_entry()? {
                    if records.insert(key, value).is_some() {
                        return Err(serde::de::Error::custom("duplicate record identity"));
                    }
                }
                Ok(Records(records))
            }
        }
        deserializer.deserialize_map(Visitor(std::marker::PhantomData))
    }
}

/// Nonnegative integral count. Arithmetic must remain explicit and checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Count(pub u64);

impl Count {
    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    /// Candidate minus baseline; unavailable when the signed delta overflows.
    pub fn checked_delta(self, baseline: Self) -> Option<i64> {
        i64::try_from(i128::from(self.0) - i128::from(baseline.0)).ok()
    }
}

macro_rules! record {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name {
            pub schema_version: SchemaVersion,
            $(pub $field: $ty),*
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Python,
    TypeScript,
    Tsx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricName {
    PhysicalLines,
    LogicalStatements,
    Decisions,
    Functions,
    InternalCallLayers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Production,
    Tests,
    Fixtures,
    ResearchSupport,
    Configuration,
    Dependency,
}

record!(MetricIdentity {
    name: MetricName,
    definition_revision: String,
    language: Language,
    grammar_version: String,
    tool_version: String
});

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "availability", rename_all = "snake_case", deny_unknown_fields)]
pub enum MetricValue {
    Available {
        count: Count,
    },
    Unavailable {
        reason: String,
        resolved_lower_bound: Option<Count>,
    },
}

record!(MetricMeasurement {
    identity: MetricIdentity,
    scope_identity: String,
    value: MetricValue
});
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "MetricPairRecord")]
pub struct MetricPair {
    pub schema_version: SchemaVersion,
    pub before: MetricMeasurement,
    pub after: MetricMeasurement,
    pub delta: Option<i64>,
}

record!(MetricPairRecord { before: MetricMeasurement, after: MetricMeasurement, delta: Option<i64> });

impl TryFrom<MetricPairRecord> for MetricPair {
    type Error = &'static str;
    fn try_from(record: MetricPairRecord) -> Result<Self, Self::Error> {
        let expected = match (&record.before.value, &record.after.value) {
            (MetricValue::Available { count: before }, MetricValue::Available { count: after })
                if record.before.identity == record.after.identity =>
            {
                after.checked_delta(*before)
            }
            _ => None,
        };
        if record.delta != expected {
            return Err(
                "metric delta must match comparable available counts using checked arithmetic",
            );
        }
        Ok(Self {
            schema_version: record.schema_version,
            before: record.before,
            after: record.after,
            delta: record.delta,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Selector {
    WholeFile {
        path: String,
    },
    Symbol {
        path: String,
        qualified_name: String,
        occurrence: Count,
        source_digest: String,
    },
}

record!(ScopeReview { rationale: String, relationships: Records<String, RelationshipReview> });
record!(RelationshipReview { owned_targets: Vec<String>, external_contract: Option<String>, rationale: String });
record!(Exclusion {
    rationale: String,
    kind: ExclusionKind
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExclusionKind {
    Generated,
    ToolState,
    MaintainedSource,
}

record!(Manifest {
    trial_id: String, hypothesis: String, entry_points: Records<String, Selector>,
    owned: Records<String, Selector>, wider_boundary: Vec<String>,
    categories: Records<String, Category>, exclusions: Records<String, Exclusion>,
    scope_review: ScopeReview, checks: Records<String, CheckDefinition>,
    target: Option<Target>, capability_inventory: Option<InventoryReference>
});

record!(InventoryReference { source_identity: String, manifest_references: Vec<String>, artifact: Artifact });
record!(Artifact {
    path: String,
    sha256: String
});
record!(FileRecord {
    content_digest: String,
    category: Category
});
record!(FrozenTrial {
    manifest: Manifest, baseline_files: Records<String, FileRecord>, baseline_identity: String,
    evaluator_identity: String, definitions: Records<String, MetricIdentity>,
    check_definition_digests: Records<String, String>
});

record!(Target {
    scope_identity: String, language: Language, definitions: Records<MetricName, MetricIdentity>,
    minimum_production_statement_reduction: Count,
    count_bounds: Records<MetricName, Count>, permitted_structural_tradeoffs: Vec<String>
});

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Interpretation {
    ExitStatus,
    CaseCounts {
        expected: Count,
    },
    Differential {
        expected: Count,
        oracle_digest: String,
    },
}

record!(CheckDefinition {
    description: String, required: bool, definition_digest: String,
    artifact_expectations: Vec<String>, interpretation: Interpretation
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Passed,
    Failed,
    Missing,
    Error,
}

record!(CheckResult {
    source_identity: String, frozen_trial_digest: String, definition_digest: String,
    status: CheckStatus, artifacts: Records<String, Artifact>, tested_domain: String,
    limitations: Vec<String>, summary: CheckSummary
});

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CheckSummary {
    ExitStatus {
        code: Option<i32>,
    },
    CaseCounts {
        expected: Count,
        observed: Count,
        mismatches: Count,
    },
    Differential {
        baseline_identity: String,
        candidate_identity: String,
        oracle_digest: String,
        expected: Count,
        observed: Count,
        mismatches: Count,
    },
}

record!(PairedCheck { baseline: Option<CheckResult>, candidate: Option<CheckResult> });
record!(OwnershipMap { replacements: Records<String, Vec<Selector>>, review: ScopeReview });
record!(DependencyChange { description: String, repository_decision: Option<String> });
record!(EvidenceBundle {
    candidate_ownership: OwnershipMap, checks: Records<String, PairedCheck>,
    dependency_changes: Records<String, DependencyChange>
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ComparisonStatus {
    Invalid,
    BaselineIneligible,
    RejectedBehavior,
    Incomplete,
    RejectedTarget,
    NotDemonstrated,
    Accepted,
}

record!(Finding {
    subject: String,
    explanation: String
});
record!(FileAccounting { baseline: Option<FileRecord>, candidate: Option<FileRecord> });
record!(ScopeAccounting {
    files: Records<String, FileAccounting>, capability_metrics: Records<String, MetricPair>,
    wider_metrics: Records<String, MetricPair>, category_totals: Records<Category, Records<String, MetricPair>>,
    unresolved: Records<String, Finding>
});
record!(Comparison {
    trial_id: String, hypothesis: String, baseline_identity: String, candidate_identity: String,
    frozen_trial_digest: String, status: ComparisonStatus, reasons: Vec<Finding>,
    accounting: ScopeAccounting, checks: Records<String, PairedCheck>,
    check_findings: Vec<Finding>, dependency_changes: Records<String, DependencyChange>,
    limitations: Vec<String>, provenance: Vec<String>
});

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn manifest() -> Value {
        json!({"schema_version":1,"trial_id":"trial","hypothesis":"reuse library",
            "entry_points":{"entry":{"kind":"whole_file","path":"a.py"}},
            "owned":{"a":{"kind":"whole_file","path":"a.py"}},"wider_boundary":["."],
            "categories":{"a.py":"production"},"exclusions":{},
            "scope_review":{"schema_version":1,"rationale":"whole responsibility","relationships":{}},
            "checks":{"tests":{"schema_version":1,"description":"API checks","required":true,
                "definition_digest":"check-digest","artifact_expectations":["raw.log"],
                "interpretation":{"kind":"case_counts","expected":3}}},
            "target":null,"capability_inventory":null})
    }

    #[test]
    fn manifest_round_trip_and_frozen_record() {
        let manifest: Manifest = serde_json::from_value(manifest()).unwrap();
        let frozen = FrozenTrial {
            schema_version: SchemaVersion::V1,
            manifest,
            baseline_files: Records(BTreeMap::new()),
            baseline_identity: "source".into(),
            evaluator_identity: "evaluator".into(),
            definitions: Records(BTreeMap::new()),
            check_definition_digests: Records(BTreeMap::new()),
        };
        let encoded = serde_json::to_string(&frozen).unwrap();
        assert_eq!(
            serde_json::from_str::<FrozenTrial>(&encoded).unwrap(),
            frozen
        );
    }

    #[test]
    fn unknown_and_unsupported_manifest_fields_fail() {
        for key in ["command", "rule_expression", "bytes", "runtime_performance"] {
            let mut value = manifest();
            value[key] = json!(1);
            assert!(serde_json::from_value::<Manifest>(value).is_err());
        }
        let mut value = manifest();
        value["schema_version"] = json!(2);
        assert!(serde_json::from_value::<Manifest>(value).is_err());
    }

    #[test]
    fn duplicate_ids_are_not_overwritten_and_maps_are_sorted() {
        assert!(
            serde_json::from_str::<Records<String, Count>>(r#"{"check":1,"check":2}"#).is_err()
        );
        let records: Records<String, Count> = serde_json::from_str(r#"{"z":2,"a":1}"#).unwrap();
        assert_eq!(serde_json::to_string(&records).unwrap(), r#"{"a":1,"z":2}"#);
        let source = serde_json::to_string(&manifest())
            .unwrap()
            .replace("\"exclusions\":{}", "\"exclusions\":{},\"exclusions\":{}");
        assert!(serde_json::from_str::<Manifest>(&source).is_err());
    }

    #[test]
    fn unavailable_is_distinct_from_zero_and_closed() {
        let unavailable: MetricValue = serde_json::from_value(json!({
            "availability":"unavailable","reason":"parse error","resolved_lower_bound":null
        }))
        .unwrap();
        assert_ne!(unavailable, MetricValue::Available { count: Count(0) });
        for invalid in [
            json!({"availability":"available"}),
            json!({"availability":"available","count":0,"reason":"bad"}),
            json!({"availability":"unavailable","reason":"missing","count":0}),
        ] {
            assert!(serde_json::from_value::<MetricValue>(invalid).is_err());
        }
    }

    #[test]
    fn count_arithmetic_and_json_never_wrap_or_coerce() {
        assert_eq!(Count(u64::MAX).checked_add(Count(1)), None);
        assert_eq!(Count(2).checked_delta(Count(5)), Some(-3));
        assert_eq!(Count(u64::MAX).checked_delta(Count(0)), None);
        for invalid in ["-1", "1.5", "18446744073709551616", "null", "\"1\""] {
            assert!(serde_json::from_str::<Count>(invalid).is_err());
        }
    }

    fn identity(name: &str) -> Value {
        json!({"schema_version":1,"name":name,"definition_revision":"v1",
            "language":"python","grammar_version":"0.25","tool_version":"0.51.0"})
    }

    fn pair() -> Value {
        json!({"schema_version":1,
            "before":{"schema_version":1,"identity":identity("logical_statements"),
                "scope_identity":"baseline-scope","value":{"availability":"available","count":10}},
            "after":{"schema_version":1,"identity":identity("logical_statements"),
                "scope_identity":"candidate-scope","value":{"availability":"available","count":6}},
            "delta":-4})
    }

    #[test]
    fn metric_deltas_cannot_hide_unavailability_or_definition_changes() {
        let valid: MetricPair = serde_json::from_value(pair()).unwrap();
        assert_eq!(valid.delta, Some(-4));
        let mut wrong = pair();
        wrong["delta"] = json!(0);
        assert!(serde_json::from_value::<MetricPair>(wrong).is_err());
        let mut changed = pair();
        changed["after"]["identity"]["definition_revision"] = json!("v2");
        assert!(serde_json::from_value::<MetricPair>(changed.clone()).is_err());
        changed["delta"] = Value::Null;
        assert!(serde_json::from_value::<MetricPair>(changed).is_ok());
        let mut missing = pair();
        missing["after"]["value"] = json!({"availability":"unavailable","reason":"syntax",
            "resolved_lower_bound":null});
        assert!(serde_json::from_value::<MetricPair>(missing.clone()).is_err());
        missing["delta"] = Value::Null;
        assert!(serde_json::from_value::<MetricPair>(missing).is_ok());
    }

    #[test]
    fn target_is_closed_to_supported_counts() {
        let target = json!({"schema_version":1,"scope_identity":"production","language":"python",
            "definitions":{"logical_statements":identity("logical_statements")},
            "minimum_production_statement_reduction":1,"count_bounds":{"functions":5},
            "permitted_structural_tradeoffs":["one additional function"]});
        let parsed: Target = serde_json::from_value(target.clone()).unwrap();
        assert_eq!(serde_json::to_value(parsed).unwrap(), target);
        for unsupported in ["bytes", "gzip", "latency", "memory", "lizard_ccn"] {
            let mut invalid = target.clone();
            invalid["count_bounds"][unsupported] = json!(3);
            assert!(serde_json::from_value::<Target>(invalid).is_err());
        }
        let mut invalid = target;
        invalid["threshold_expression"] = json!("x < 3");
        assert!(serde_json::from_value::<Target>(invalid).is_err());
    }

    fn check_result(source: &str) -> Value {
        json!({"schema_version":1,"source_identity":source,"frozen_trial_digest":"trial-digest",
            "definition_digest":"check-digest","status":"passed",
            "artifacts":{"log":{"schema_version":1,"path":"raw.log","sha256":"artifact-digest"}},
            "tested_domain":"three API cases","limitations":["finite cases"],
            "summary":{"kind":"differential","baseline_identity":"baseline","candidate_identity":"candidate",
                "oracle_digest":"oracle","expected":3,"observed":3,"mismatches":0}})
    }

    #[test]
    fn evidence_round_trip_preserves_bindings_counts_and_missing_sides() {
        let value = json!({"schema_version":1,
            "candidate_ownership":{"schema_version":1,"replacements":{"a":[{"kind":"whole_file","path":"new.py"}]},
                "review":{"schema_version":1,"rationale":"replacement includes helper","relationships":{}}},
            "checks":{"tests":{"schema_version":1,"baseline":check_result("baseline"),"candidate":check_result("candidate")},
                "missing":{"schema_version":1,"baseline":null,"candidate":null}},"dependency_changes":{}});
        let parsed: EvidenceBundle = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(parsed).unwrap(), value);
        let mut invalid = check_result("candidate");
        invalid["summary"]["mismatches"] = json!(-1);
        assert!(serde_json::from_value::<CheckResult>(invalid).is_err());
        let mut invalid = check_result("candidate");
        invalid["command"] = json!("run tests");
        assert!(serde_json::from_value::<CheckResult>(invalid).is_err());
    }

    #[test]
    fn comparison_round_trip_retains_incomplete_and_failure_findings() {
        let finding =
            json!({"schema_version":1,"subject":"tests","explanation":"observed mismatch"});
        let value = json!({"schema_version":1,"trial_id":"trial","hypothesis":"reuse library",
            "baseline_identity":"baseline","candidate_identity":"candidate","frozen_trial_digest":"trial-digest",
            "status":"rejected-behavior","reasons":[finding.clone()],
            "accounting":{"schema_version":1,"files":{},"capability_metrics":{"statements":pair()},
                "wider_metrics":{},"category_totals":{},"unresolved":{"dynamic-call":finding.clone()}},
            "checks":{},"check_findings":[finding],"dependency_changes":{},
            "limitations":["missing optional run"],"provenance":["declared runner"]});
        let parsed: Comparison = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(parsed).unwrap(), value);
    }
}
