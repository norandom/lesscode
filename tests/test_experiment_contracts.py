"""External experiment evidence preserves declared conditions and missing data."""

from copy import deepcopy
import json
import math
import unittest

from lesscode_baseline import contracts as c

HASH = "a" * 64
TIME = "2026-10-30T18:00:00Z"


def observed(unit="tokens", value=100):
    return {"status": "available", "value": value, "unit": unit,
            "source": "provider usage export", "observed_at": TIME, "reason": None}


def unavailable(unit):
    return {"status": "unavailable", "value": None, "unit": unit,
            "source": None, "observed_at": None, "reason": "not supplied"}


def telemetry():
    return {"token_usage": observed(), "provider_cost": unavailable("USD"),
            "agent_elapsed": observed("seconds", 12),
            "runtime_benchmarks": [{"benchmark_id": "lookup", "workload_hash": HASH,
                                    "environment_hash": HASH, "measurement": observed("seconds", 0.3)}]}


def budget():
    return {"time_seconds": 60, "tokens": 2000, "provider_cost": None, "cost_unit": None}


def conditions():
    return {"model": "recorded-model", "settings": {"temperature": 0}, "prompt_identity": HASH,
            "permissions": ["read_source"], "budget": budget(), "interventions": {"skill": "off"}}


def corpus():
    return {"corpus_id": "example", "origin": "https://example.org/repo", "revision": "v1",
            "content_hash": HASH, "spdx_license": "MIT", "license_artifact": None,
            "generation_provenance": {"status": "unknown", "description": None, "source": None}}


def manifest():
    return {"schema_version": 1, "experiment_id": "pilot", "corpus": [corpus()],
            "controls": conditions(), "allowed_interventions": ["skill"],
            "arms": [{"arm_id": "control", "interventions": {"skill": "off"}},
                     {"arm_id": "skill", "interventions": {"skill": "on"}}],
            "planned_trials": [{"trial_id": arm, "arm_id": arm, "corpus_id": "example",
                                "task_id": "simplify", "repetition": 1, "prompt_identity": HASH}
                               for arm in ("control", "skill")]}


def trial():
    return {"schema_version": 1, "trial_id": "control", "arm_id": "control", "corpus_id": "example",
            "task_id": "simplify", "repetition": 1, "starting_source_hash": HASH,
            "run_artifact": "runs/control/report.json", "conditions": conditions(),
            "execution_order": 1, "seed": None, "completion_state": "completed",
            "budget_exhausted": False, "telemetry": telemetry()}


def distribution(n=1):
    return {"key": "source_bytes", "unit": "bytes", "n": n, "mean": 10 if n else None,
            "median": 10 if n else None, "minimum": 10 if n else None,
            "maximum": 10 if n else None, "sample_stddev": 0 if n >= 2 else None}


def report():
    return {"schema_version": 1, "manifest": manifest(), "trials": [trial()], "extra_trials": [],
            "outcomes": [{"trial_id": arm, "decision": "incomplete", "baseline_status": "missing",
                          "required_behavior_status": "incomplete", "confounded_differences": [],
                          "reasons": ["candidate pending" if arm == "control" else "producer record absent"]}
                         for arm in ("control", "skill")],
            "arm_summaries": [{"arm_id": arm, "planned_attempts": 1,
                               "state_counts": {"accepted": 0, "rejected": 0, "incomplete": 1,
                                                "baseline_ineligible": 0, "invalid": 0},
                               "eligible_attempts": 0, "correct_attempts": 0, "budget_exhausted_attempts": 0,
                               "all_attempt_structural": [], "accepted_only_structural": []}
                              for arm in ("control", "skill")],
            "paired_summaries": [{"baseline_arm": "control", "candidate_arm": "skill",
                                  "differences": [distribution(0)]}],
            "limitations": ["producer evidence is supplied, not independently verified"]}


class ExperimentContractsTests(unittest.TestCase):
    def roundtrip(self, record_type, value):
        record = record_type.from_dict(value)
        self.assertEqual(json.loads(c.dumps(record)), value)
        self.assertEqual(c.loads(record_type, c.dumps(record)), record)
        return record

    def bad(self, record_type, value, field):
        with self.assertRaisesRegex(c.ContractError, field):
            record_type.from_dict(value)

    def test_multi_arm_manifest_and_producer_roundtrip(self):
        result = self.roundtrip(c.ExperimentManifest, manifest())
        self.assertEqual(len(result.arms), 2)
        self.roundtrip(c.TrialRecord, trial())

    def test_explicit_unavailable_telemetry_and_zero_observations(self):
        data = telemetry()
        data["token_usage"] = unavailable("tokens")
        data["provider_cost"] = observed("USD", 0)
        self.roundtrip(c.Telemetry, data)

    def test_corpus_identity_license_and_generation_provenance(self):
        data = corpus()
        data.update(revision=None, spdx_license=None, license_artifact="licenses/MIT.txt")
        data["generation_provenance"] = {"status": "known", "description": "human authored",
                                         "source": "maintainer declaration"}
        self.roundtrip(c.CorpusEntry, data)
        for key in ("revision", "content_hash", "spdx_license", "license_artifact", "generation_provenance"):
            broken = corpus()
            del broken[key]
            self.bad(c.CorpusEntry, broken, key)
        for fields, diagnostic in (({"revision": None, "content_hash": None}, "content_hash"),
                                   ({"spdx_license": None, "license_artifact": None}, "license"),
                                   ({"license_artifact": "../license"}, "license_artifact")):
            self.bad(c.CorpusEntry, corpus() | fields, diagnostic)

    def test_provenance_requires_evidence_for_known_and_explicit_unknown(self):
        for value in ({"status": "known", "description": "AI generated", "source": None},
                      {"status": "unknown", "description": "looks generated", "source": None}):
            self.bad(c.GenerationProvenance, value, "source|description")

    def test_duplicates_references_and_intervention_boundaries(self):
        for key in ("corpus", "arms", "planned_trials"):
            data = manifest()
            data[key].append(deepcopy(data[key][0]))
            self.bad(c.ExperimentManifest, data, key)
        for key in ("arm_id", "corpus_id"):
            data = manifest()
            data["planned_trials"][0][key] = "foreign"
            self.bad(c.ExperimentManifest, data, key)
        data = manifest()
        extra = deepcopy(data["planned_trials"][0])
        extra["trial_id"] = "another-id"
        data["planned_trials"].append(extra)
        self.bad(c.ExperimentManifest, data, "planned_trials")
        data = manifest()
        data["arms"][1]["interventions"]["permissions"] = "write"
        self.bad(c.ExperimentManifest, data, "interventions")

    def test_telemetry_rejects_nonfinite_negative_missing_and_false_values(self):
        for value in (math.nan, math.inf, -1, True):
            self.bad(c.ObservedMeasurement, observed(value=value), "value")
        for key in ("unit", "source", "observed_at"):
            data = observed()
            data[key] = None
            self.bad(c.ObservedMeasurement, data, key)
        self.bad(c.ObservedMeasurement, observed() | {"reason": "made up"}, "reason")
        self.bad(c.ObservedMeasurement, unavailable("USD") | {"value": 0}, "value")
        self.bad(c.ObservedMeasurement, unavailable("USD") | {"reason": None}, "reason")

    def test_runtime_workload_environment_and_units_are_separate(self):
        for key in ("workload_hash", "environment_hash"):
            data = telemetry()
            del data["runtime_benchmarks"][0][key]
            self.bad(c.Telemetry, data, key)
        for key, unit in (("token_usage", "bytes"), ("agent_elapsed", "tokens")):
            data = telemetry()
            data[key]["unit"] = unit
            self.bad(c.Telemetry, data, "unit")
        data = telemetry()
        data["token_usage"]["value"] = 0.5
        self.bad(c.Telemetry, data, "value")
        data = telemetry()
        data["runtime_benchmarks"] *= 2
        self.bad(c.Telemetry, data, "runtime_benchmarks")

    def test_trial_missing_seed_budget_exhaustion_and_provenance(self):
        data = trial()
        data.update(run_artifact=None, completion_state="budget_exhausted", budget_exhausted=True)
        self.roundtrip(c.TrialRecord, data)
        for key in ("starting_source_hash", "conditions", "execution_order", "seed", "telemetry"):
            data = trial()
            del data[key]
            self.bad(c.TrialRecord, data, key)
        for fields, diagnostic in (({"run_artifact": "../report.json"}, "run_artifact"),
                                   ({"execution_order": 0}, "execution_order"),
                                   ({"seed": True}, "seed"),
                                   ({"completion_state": "budget_exhausted"}, "budget_exhausted"),
                                   ({"run_artifact": None}, "run_artifact")):
            self.bad(c.TrialRecord, trial() | fields, diagnostic)

    def test_budget_units_and_positive_limits(self):
        for fields, diagnostic in (({"time_seconds": 0}, "time_seconds"),
                                   ({"tokens": -1}, "tokens"),
                                   ({"provider_cost": 1}, "cost_unit"),
                                   ({"cost_unit": "USD"}, "cost_unit"),
                                   ({"provider_cost": math.inf, "cost_unit": "USD"}, "provider_cost")):
            self.bad(c.Budget, budget() | fields, diagnostic)

    def test_versions_unknown_fields_and_direct_serialization_fail_closed(self):
        self.bad(c.ExperimentManifest, manifest() | {"schema_version": 2}, "schema_version")
        self.bad(c.TrialRecord, trial() | {"extra": 1}, "extra")
        record = c.ObservedMeasurement("available", math.nan, "tokens", "provider", TIME, None)
        with self.assertRaises(c.ContractError):
            c.dumps(record)

    def test_report_preserves_missing_extra_attempts_and_paired_populations(self):
        data = report()
        extra = trial() | {"trial_id": "unplanned", "execution_order": 3}
        data["extra_trials"] = [extra]
        self.roundtrip(c.ExperimentReport, data)
        for key in ("trials", "outcomes", "arm_summaries", "paired_summaries"):
            broken = deepcopy(data)
            broken[key].append(deepcopy(broken[key][0]))
            self.bad(c.ExperimentReport, broken, key)
        for fields, diagnostic in (({"outcomes": []}, "outcomes"),
                                   ({"arm_summaries": []}, "arm_summaries"),
                                   ({"extra_trials": [trial()]}, "extra_trials"),
                                   ({"trials": [extra]}, "trials")):
            self.bad(c.ExperimentReport, report() | fields, diagnostic)
        data["paired_summaries"][0]["candidate_arm"] = "foreign"
        self.bad(c.ExperimentReport, data, "paired_summaries")

    def test_descriptive_statistics_preserve_undefined_small_samples(self):
        for n in (0, 1, 2):
            self.roundtrip(c.Distribution, distribution(n))
        for fields, diagnostic in (({"sample_stddev": 0}, "sample_stddev"),
                                   ({"n": 0}, "n"), ({"mean": math.inf}, "mean"),
                                   ({"minimum": 20}, "minimum")):
            self.bad(c.Distribution, distribution() | fields, diagnostic)
        data = report()["arm_summaries"][0]
        self.bad(c.ArmSummary, data | {"correct_attempts": 1}, "eligible_attempts")
        self.bad(c.ArmSummary, data | {"planned_attempts": 2}, "state_counts")


if __name__ == "__main__":
    unittest.main()
