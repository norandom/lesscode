from datetime import datetime, timedelta, timezone
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from lesscode_baseline.contracts import ContractError, OptionalEvidenceArtifact
from lesscode_baseline.process import read_optional_evidence


class OptionalEvidenceTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        self.logs = self.root / "logs"
        self.source, self.plan, self.scope = (x * 64 for x in "abc")
        self.started = datetime.now(timezone.utc) - timedelta(seconds=1)

    def document(self, origin="generated"):
        def metric(key, payload):
            return dict(key=key, status="available", value=payload, unit="items",
                        definition_id=key + ":v1", producer="provided-tool",
                        producer_version="1.2", scope_hash=self.scope,
                        reason=None, raw_artifact=None)
        population = dict(kind="population", population_ids=["line:1", "line:2"],
                          population_hash="d" * 64, numerator=1, denominator=2, explanation=None)
        finding = dict(finding_id="depth", location=dict(path="main.py", start_line=1,
                       end_line=2), message="declared nesting", value=2)
        return dict(schema_version=1, artifact_id="optional", source_hash=self.source,
                    plan_hash=self.plan, produced_at=datetime.now(timezone.utc).isoformat(),
                    origin=origin, metrics=[metric("coverage", population),
                    metric("mutation", population), metric("nesting", dict(kind="findings", findings=[finding])),
                    metric("architecture", dict(kind="findings", findings=[]))],
                    runtime_benchmarks=[dict(benchmark_id="lookup", workload_hash="e" * 64,
                    environment_hash="f" * 64, measurement=dict(status="available", value=1.5,
                    unit="seconds", source="benchmark-tool", observed_at=datetime.now(timezone.utc).isoformat(), reason=None))])

    def read(self, data=None, **kwargs):
        if data is not None:
            (self.root / "optional.json").write_bytes(json.dumps(data).encode())
        return read_optional_evidence("optional.json", self.root, self.logs,
            self.source, self.plan, self.scope, origin=kwargs.pop("origin", "generated"),
            generated_since=kwargs.pop("generated_since", self.started), **kwargs)

    def test_generated_preserves_typed_populations_findings_runtime_and_raw(self):
        data = self.document()
        evidence, raw, failures = self.read(data)
        self.assertFalse(failures)
        self.assertEqual(evidence, OptionalEvidenceArtifact.from_dict(data))
        self.assertEqual((self.logs / raw).read_bytes(), (self.root / "optional.json").read_bytes())
        self.assertEqual(evidence.metrics[0].value.denominator, 2)
        self.assertEqual(evidence.runtime_benchmarks[0].workload_hash, "e" * 64)

    def test_imported_retains_claims_and_old_timestamp(self):
        data = self.document("producer_supplied")
        data["produced_at"] = "2000-01-01T00:00:00Z"
        evidence, _, failures = self.read(data, origin="producer_supplied", generated_since=None)
        self.assertFalse(failures)
        self.assertEqual(evidence.origin, "producer_supplied")
        self.assertEqual(evidence.metrics[0].producer, "provided-tool")

    def test_population_changes_and_explanations_are_retained_without_adjudication(self):
        first, _, _ = self.read(self.document())
        data = self.document()
        data["metrics"][0]["value"].update(population_ids=["line:3"], population_hash="e" * 64,
                                         numerator=1, denominator=1, explanation="removed dead branch")
        second, _, failures = self.read(data)
        self.assertFalse(failures)
        self.assertEqual(second.metrics[0].status, "available")
        self.assertNotEqual(first.metrics[0].value.population_ids, second.metrics[0].value.population_ids)
        self.assertEqual(second.metrics[0].value.explanation, "removed dead branch")

    def test_foreign_bindings_scope_and_origin_rejected(self):
        for field in ("source_hash", "plan_hash", "origin", "scope_hash"):
            with self.subTest(field=field):
                data = self.document()
                if field == "scope_hash":
                    data["metrics"][0][field] = "d" * 64
                else:
                    data[field] = "producer_supplied" if field == "origin" else "d" * 64
                evidence, raw, failures = self.read(data)
                self.assertIsNone(evidence)
                self.assertTrue(raw)
                self.assertEqual(failures[0].code, "optional_evidence_invalid")

    def test_stale_generated_timestamp_or_file_rejected(self):
        data = self.document()
        data["produced_at"] = "2000-01-01T00:00:00Z"
        self.assertIsNone(self.read(data)[0])
        self.read(self.document())
        os.utime(self.root / "optional.json", (1, 1))
        self.assertIsNone(self.read()[0])

    def test_missing_invalid_duplicate_and_oversized_reports(self):
        evidence, raw, failures = self.read()
        self.assertIsNone(evidence)
        self.assertIsNone(raw)
        self.assertEqual(failures[0].code, "optional_evidence_missing")
        for content in (b"{", b'{"schema_version":1,"schema_version":1}', b"null"):
            (self.root / "optional.json").write_bytes(content)
            self.assertIsNone(self.read()[0])
        self.assertIsNone(self.read(self.document(), max_bytes=1)[0])

    def test_invalid_caller_policy_and_path_fail_before_read(self):
        for kwargs in (dict(generated_since=None), dict(max_bytes=0), dict(origin="other")):
            with self.subTest(kwargs=kwargs), self.assertRaises(ContractError):
                self.read(self.document(), **kwargs)
        with self.assertRaises(ContractError):
            read_optional_evidence("../outside", self.root, self.logs, self.source,
                                  self.plan, self.scope, origin="producer_supplied")

    def test_persistence_errors_propagate(self):
        self.read(self.document())
        real_open = Path.open
        def fail_write(path, mode="r", *args, **kwargs):
            if mode == "xb":
                raise OSError("disk unavailable")
            return real_open(path, mode, *args, **kwargs)
        with patch("pathlib.Path.open", fail_write):
            with self.assertRaises(OSError):
                self.read()

    def test_imported_foreign_bindings_and_scope_rejected(self):
        for field in ("source_hash", "plan_hash", "scope_hash"):
            data = self.document("producer_supplied")
            if field == "scope_hash":
                data["metrics"][0][field] = "d" * 64
            else:
                data[field] = "d" * 64
            self.assertIsNone(self.read(data, origin="producer_supplied")[0])

    def test_runtime_only_and_unavailable_metrics_remain_explicit(self):
        data = self.document()
        data["metrics"] = []
        evidence, _, failures = self.read(data)
        self.assertFalse(failures)
        self.assertEqual(len(evidence.runtime_benchmarks), 1)
        data = self.document()
        data["metrics"][0].update(status="unavailable", value=None, reason="no coverage supplied")
        evidence, _, failures = self.read(data)
        self.assertFalse(failures)
        self.assertIsNone(evidence.metrics[0].value)
        self.assertEqual(evidence.metrics[0].reason, "no coverage supplied")

    def test_future_timestamp_rejected(self):
        data = self.document()
        data["produced_at"] = (datetime.now(timezone.utc) + timedelta(days=1)).isoformat()
        self.assertIsNone(self.read(data)[0])
    def test_link_input_rejected_before_read(self):
        with patch("pathlib.Path.is_symlink", return_value=True), self.assertRaises(ContractError):
            read_optional_evidence("linked.json", self.root, self.logs, self.source,
                                  self.plan, self.scope, origin="producer_supplied")
