"""Optional evidence envelopes preserve provenance and existing nested records."""

from copy import deepcopy
from dataclasses import replace
import json
import unittest

from lesscode_baseline import contracts as c


HASH = "a" * 64


def artifact():
    return {
        "schema_version": 1, "artifact_id": "optional-evidence-1",
        "source_hash": HASH, "plan_hash": "b" * 64,
        "produced_at": "2026-10-31T07:30:00+01:00", "origin": "producer_supplied",
        "metrics": [{"key": "coverage", "status": "available", "unit": "covered lines",
                     "definition_id": "coverage-lines-v1", "producer": "original coverage producer",
                     "producer_version": "7.10", "scope_hash": "c" * 64,
                     "reason": None, "raw_artifact": "raw/coverage.json",
                     "value": {"kind": "population", "population_ids": ["main.py:1", "main.py:2"],
                               "population_hash": "d" * 64, "numerator": 1, "denominator": 2,
                               "explanation": "Second line added to population"}}],
        "runtime_benchmarks": [{"benchmark_id": "lookup", "workload_hash": "e" * 64,
                                "environment_hash": "f" * 64,
                                "measurement": {"status": "available", "value": 0.4, "unit": "seconds",
                                                "source": "original benchmark export",
                                                "observed_at": "2026-10-31T06:25:00Z", "reason": None}}],
    }


class OptionalContractTests(unittest.TestCase):
    def read(self, data):
        return c.loads(c.OptionalEvidenceArtifact, json.dumps(data).encode("utf-8"))

    def reject(self, data, field):
        with self.assertRaisesRegex(c.ContractError, field):
            self.read(data)

    def test_populated_envelopes_roundtrip_with_both_origins(self):
        for origin in ("generated", "producer_supplied"):
            data = artifact()
            data["origin"] = origin
            record = self.read(data)
            self.assertIsInstance(record.metrics, tuple)
            self.assertIsInstance(record.metrics[0], c.MetricEvidence)
            self.assertIsInstance(record.runtime_benchmarks, tuple)
            self.assertIsInstance(record.runtime_benchmarks[0], c.RuntimeBenchmark)
            self.assertEqual(json.loads(c.dumps(record)), data)
            self.assertEqual(record.to_dict(), data)
            self.assertEqual(c.loads(c.OptionalEvidenceArtifact, c.dumps(record)), record)
            self.assertEqual(record.metrics[0].to_dict(), data["metrics"][0])
            self.assertEqual(record.runtime_benchmarks[0].to_dict(), data["runtime_benchmarks"][0])

    def test_either_evidence_collection_can_be_empty_but_not_both(self):
        for empty in ("metrics", "runtime_benchmarks"):
            data = artifact()
            data[empty] = []
            self.assertEqual(self.read(data).to_dict(), data)
        data = artifact()
        data.update(metrics=[], runtime_benchmarks=[])
        self.reject(data, "evidence")

    def test_findings_and_unavailable_metrics_keep_their_own_identity(self):
        data = artifact()
        finding = deepcopy(data["metrics"][0])
        finding.update(key="nesting", producer="original nesting producer", unit="depth",
                       definition_id="nesting-v1", value={"kind": "findings", "findings": [
                           {"finding_id": "deep-branch", "location": {"path": "main.py", "start_line": 1,
                                                                     "end_line": 2},
                            "message": "Deeply nested branch", "value": 5}]})
        missing = deepcopy(data["metrics"][0])
        missing.update(key="mutation", status="unavailable", value=None, reason="producer unavailable")
        data["metrics"].extend([finding, missing])
        self.assertEqual(self.read(data).to_dict(), data)

    def test_every_envelope_field_is_required_and_unknown_fields_rejected(self):
        for key in artifact():
            data = artifact()
            del data[key]
            self.reject(data, key)
        data = artifact()
        data["producer"] = "must remain nested"
        self.reject(data, "producer")

    def test_versions_origins_and_ids_are_strict(self):
        for field, values in {
            "schema_version": [0, 2, True, 1.0, "1", None],
            "origin": ["imported", "Generated", True, None, ""],
            "artifact_id": ["", " ", "bad\x00id", 1, None],
        }.items():
            for value in values:
                with self.subTest(field=field, value=value):
                    data = artifact()
                    data[field] = value
                    self.reject(data, field)

    def test_source_and_plan_fingerprints_are_strict(self):
        for field in ("source_hash", "plan_hash"):
            for value in (None, 1, "", "A" * 64, "a" * 63, "g" * 64):
                data = artifact()
                data[field] = value
                self.reject(data, field)

    def test_produced_at_requires_valid_timezone_timestamp(self):
        for value in (None, 1, "", "yesterday", "2026-10-31", "2026-10-31T07:30:00",
                      "2026-02-30T00:00:00Z"):
            data = artifact()
            data["produced_at"] = value
            self.reject(data, "produced_at")

    def test_duplicate_metric_and_benchmark_identities_are_rejected(self):
        for field in ("metrics", "runtime_benchmarks"):
            data = artifact()
            data[field].append(deepcopy(data[field][0]))
            self.reject(data, field + r"\[1\]")

    def test_collections_and_nested_contracts_are_strict(self):
        for field in ("metrics", "runtime_benchmarks"):
            for value in (None, {}, "evidence", [None]):
                data = artifact()
                data[field] = value
                self.reject(data, field)
            data = artifact()
            data[field][0]["unknown"] = True
            self.reject(data, field + r"\[0\].unknown")
        data = artifact()
        data["metrics"][0]["value"]["numerator"] = 3
        self.reject(data, "numerator")
        data = artifact()
        data["runtime_benchmarks"][0]["environment_hash"] = "invalid"
        self.reject(data, "environment_hash")

    def test_serialization_revalidates_direct_construction(self):
        record = self.read(artifact())
        for changes, field in (({"origin": "invalid"}, "origin"),
                               ({"source_hash": "invalid"}, "source_hash"),
                               ({"metrics": (), "runtime_benchmarks": ()}, "evidence"),
                               ({"metrics": record.metrics * 2}, r"metrics\[1\]")):
            with self.assertRaisesRegex(c.ContractError, field):
                c.dumps(replace(record, **changes))

    def test_duplicate_json_fields_fail_closed(self):
        text = c.dumps(self.read(artifact()))
        with self.assertRaisesRegex(c.ContractError, "duplicate JSON object key"):
            c.loads(c.OptionalEvidenceArtifact, text.replace('"schema_version":1',
                                                           '"schema_version":1,"schema_version":1'))
