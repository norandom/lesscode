"""Public evaluation JSON must preserve evidence and fail closed."""

from copy import deepcopy
import json
import math
import unittest

from lesscode_baseline import contracts as c


HASH = "a" * 64


def scope():
    return {"include": ["**/*"], "exclude": [".git/**"], "generated": ["generated/**"],
            "vendor": ["vendor/**"], "tests": ["tests/**"], "product": ["**/*.py"]}


def limits():
    return {"timeout_seconds": 30, "max_log_bytes": 10000}


def command():
    return {"command_id": "native", "argv": ["python", "-m", "unittest"], "cwd": ".",
            "environment": {}, "limits": limits()}


def check_spec():
    return {"check_id": "native", "kind": "native", "required": True,
            "command": command(), "result_path": "checks.json", "result_format": "normalized",
            "expected_ids": ["test.api"]}


def profile():
    return {"schema_version": 1, "scope": scope(),
            "tools": [{"name": "lizard", "version": "1.17.31", "options": {}}],
            "checks": [check_spec()], "required_metrics": ["complexity", "duplication", "source_bytes"],
            "thresholds": [{"metric_key": "source_bytes", "operator": "delta_maximum", "value": 0, "unit": "bytes"}],
            "limits": limits(), "evaluator_resources": ["checks/oracle.py"]}


def snapshot():
    return {"root": "C:/project", "content_hash": HASH, "scope_hash": HASH, "revision": None,
            "files": [{"path": "src/main.py", "category": "product", "size_bytes": 12,
                       "content_hash": HASH, "mode": 420}], "exclusions": [".git/**"]}


def metric():
    return {"key": "source_bytes", "status": "available", "value": 12, "unit": "bytes",
            "definition_id": "raw-source-bytes-v1", "producer": "lesscode", "producer_version": "0.1",
            "scope_hash": HASH, "reason": None, "raw_artifact": "baseline/size.json"}


def check():
    return {"check_id": "test.api", "status": "passed", "kind": "native", "required": True,
            "source_hash": HASH, "plan_hash": HASH, "diagnostic_artifact": "baseline/native.log"}


def observation():
    return {"snapshot": snapshot(), "metrics": [metric()], "checks": [check()], "executions": [], "failures": []}


def plan():
    return {"schema_version": 1, "profile": profile(), "profile_hash": HASH, "evaluator_hash": HASH,
            "baseline": snapshot(), "candidate": snapshot(),
            "resolved_commands": [{**command(), "argv": ["C:/Python/python.exe", "-m", "unittest"],
                                   "executable_version": "3.12.10"}],
            "protection": {"mode": "local_detection", "provider_evidence": None}}


def report():
    return {"schema_version": 1, "run_id": "run-1", "plan_hash": HASH, "baseline": observation(),
            "candidate": observation(), "integrity": {"status": "unchanged", "changed_resources": [],
            "protection": {"mode": "local_detection", "provider_evidence": None}},
            "decision": "accepted", "reasons": ["All declared checks passed"], "metric_deltas": [],
            "test_file_delta": {"added": [], "removed": [], "modified": []}, "limits": limits(),
            "elapsed_seconds": 0.1, "limitations": ["Unchecked behavior remains"],
            "started_at": "2026-10-30T18:00:00Z", "finished_at": "2026-10-30T18:00:01Z",
            "environment": {"os": "Windows", "python": "3.12.10", "secrets_supplied": False}}


class ContractTests(unittest.TestCase):
    def read(self, name, data):
        return c.loads(getattr(c, name, name), json.dumps(data))

    def reject(self, name, data, field):
        with self.assertRaisesRegex(ValueError, field):
            self.read(name, data)

    def test_all_evaluation_records_round_trip(self):
        for name, data in [("Profile", profile()), ("Plan", plan()), ("Snapshot", snapshot()),
                           ("MetricEvidence", metric()), ("CheckEvidence", check()),
                           ("Failure", {"code": "missing", "message": "No tool", "affected_path": None,
                            "check_id": None, "metric_key": "source_bytes", "retryable": False, "raw_artifact": None}),
                           ("RunReport", report())]:
            with self.subTest(record=name):
                record = self.read(name, data)
                self.assertEqual(json.loads(c.dumps(record)), data)
                self.assertEqual(self.read(name, data), record)

    def test_required_flags_and_unavailable_values_remain_explicit(self):
        data = metric()
        data.update(status="unavailable", value=None, reason="tool missing")
        evidence = self.read("MetricEvidence", data)
        self.assertIsNone(evidence.value)
        self.assertEqual(evidence.reason, "tool missing")
        data = check()
        data.update(status="missing", required=False)
        evidence = self.read("CheckEvidence", data)
        self.assertFalse(evidence.required)
        self.assertEqual(evidence.status, "missing")

    def test_unknown_fields_and_versions_fail_at_nested_field(self):
        data = profile()
        data["checks"][0]["requird"] = True
        self.reject("Profile", data, r"checks\[0\].requird")
        for version in [0, 2, "1", True, 1.1]:
            data = profile()
            data["schema_version"] = version
            self.reject("Profile", data, "schema_version")

    def test_missing_fields_and_wrong_types_are_field_diagnostics(self):
        data = check()
        del data["required"]
        self.reject("CheckEvidence", data, "required")
        data["required"] = 1
        self.reject("CheckEvidence", data, "required")
        data = snapshot()
        data["files"][0]["size_bytes"] = True
        self.reject("Snapshot", data, "size_bytes")

    def test_nonfinite_numbers_rejected_on_read_and_write(self):
        for value in [math.nan, math.inf, -math.inf]:
            data = metric()
            data["value"] = value
            self.reject("MetricEvidence", data, "value")
        evidence = self.read("MetricEvidence", metric())
        from dataclasses import replace
        with self.assertRaisesRegex(ValueError, "value"):
            c.dumps(replace(evidence, value=math.inf))

    def test_unavailable_requires_null_and_reason_available_requires_value(self):
        for status, value, reason in [("unavailable", 0, "missing"), ("unavailable", None, None),
                                      ("available", None, None), ("available", 12, "missing"),
                                      ("non_comparable", None, "")]:
            data = metric()
            data.update(status=status, value=value, reason=reason)
            self.reject("MetricEvidence", data, "value|reason")

    def test_duplicate_identities_rejected(self):
        cases = []
        data = profile(); data["checks"].append(deepcopy(data["checks"][0])); cases.append(("Profile", data, "checks"))
        data = profile(); data["tools"].append(deepcopy(data["tools"][0])); cases.append(("Profile", data, "tools"))
        data = profile(); data["checks"][0]["expected_ids"] *= 2; cases.append(("Profile", data, "expected_ids"))
        data = snapshot(); data["files"].append(deepcopy(data["files"][0])); cases.append(("Snapshot", data, "files"))
        data = report(); data["baseline"]["metrics"] *= 2; cases.append(("RunReport", data, "metrics"))
        data = report(); data["candidate"]["checks"] *= 2; cases.append(("RunReport", data, "checks"))
        for name, data, field in cases:
            with self.subTest(field=field): self.reject(name, data, field)

    def test_escaping_and_nonportable_artifact_paths_rejected(self):
        for path in ["../secret", "baseline/../../x", "/tmp/x", "C:/secret", "C:secret", "\\\\server\\x",
                     "baseline\\..\\secret", "a//b", "./a", "a/./b", "a\x00b", "a:stream", ""]:
            data = metric(); data["raw_artifact"] = path
            with self.subTest(path=path): self.reject("MetricEvidence", data, "raw_artifact")
            data = snapshot(); data["files"][0]["path"] = path
            with self.subTest(source=path): self.reject("Snapshot", data, "path")

    def test_case_colliding_source_paths_rejected(self):
        data = snapshot()
        second = deepcopy(data["files"][0]); second["path"] = "SRC/main.py"
        data["files"].append(second)
        self.reject("Snapshot", data, "files")

    def test_invalid_fingerprints_rejected(self):
        for fingerprint in ["", "abc", "g" * 64, 12]:
            data = metric(); data["scope_hash"] = fingerprint
            self.reject("MetricEvidence", data, "scope_hash")

    def test_limits_thresholds_and_commands_validated_before_execution(self):
        data = profile(); data["checks"][0]["command"]["argv"] = []
        self.reject("Profile", data, "argv")
        data = profile(); data["limits"]["timeout_seconds"] = 0
        self.reject("Profile", data, "timeout_seconds")
        data = profile(); data["thresholds"][0]["operator"] = "universal_score"
        self.reject("Profile", data, "operator")
        data = plan(); data["resolved_commands"][0]["argv"][0] = "python"
        self.reject("Plan", data, "argv")

    def test_profile_requires_behavior_and_structural_evidence(self):
        data = profile(); data["checks"][0]["required"] = False
        self.reject("Profile", data, "checks")
        data = profile(); data["required_metrics"] = []
        self.reject("Profile", data, "required_metrics")

    def test_structured_metric_payload_round_trip(self):
        data = metric()
        data.update(key="complexity", unit="ccn", definition_id="lizard-ccn-v1",
                    value={"kind": "complexity", "files": [{"path": "src/main.py", "nloc": 3}],
                    "functions": [{"path": "src/main.py", "name": "f", "long_name": "f()",
                    "start_line": 1, "end_line": 3, "ccn": 2, "nloc": 3, "token_count": 10}],
                    "count": 1, "minimum": 2, "maximum": 2, "median": 2, "p95": 2})
        self.assertEqual(json.loads(c.dumps(self.read("MetricEvidence", data))), data)
        data["value"]["functions"][0]["end_line"] = 0
        self.reject("MetricEvidence", data, "end_line")

    def test_untyped_vendor_objects_cannot_be_metric_payloads(self):
        data = metric(); data["value"] = {"vendor_magic": 123}
        self.reject("MetricEvidence", data, "kind")

    def test_json_duplicate_keys_and_nonobject_roots_rejected(self):
        for text in ['{"schema_version":1,"schema_version":2}', '[]', 'null', '{']:
            with self.subTest(text=text), self.assertRaises(ValueError):
                c.loads(getattr(c, "Profile", "Profile"), text)

    def test_run_retains_execution_provenance_and_partial_failures(self):
        data = report()
        data["candidate"]["executions"] = [{"command": {**command(), "argv": ["/usr/bin/python3"],
            "executable_version": "3.12.10"}, "exit_code": None, "status": "timeout", "elapsed_seconds": 30,
            "stdout_artifact": "candidate/stdout.log", "stderr_artifact": "candidate/stderr.log",
            "started_at": "2026-10-30T18:00:00+00:00", "environment_hash": HASH}]
        data["decision"] = "incomplete"
        self.assertEqual(json.loads(c.dumps(self.read("RunReport", data))), data)

    def test_remaining_payload_types_and_empty_complexity_round_trip(self):
        span = {"path": "src/main.py", "start_line": 1, "end_line": 3}
        payloads = [
            {"kind": "complexity", "files": [{"path": "empty.py", "nloc": 0}], "functions": [],
             "count": 0, "minimum": None, "maximum": None, "median": None, "p95": None},
            {"kind": "duplication", "duplicated_tokens": 50, "total_tokens": 100,
             "ratio": 0.5, "ratio_reason": None, "mode": "weak", "upstream_percentage": 50,
             "clones": [{"first": span, "second": {**span, "path": "src/copy.py"}}],
             "scanned_paths": ["src/main.py", "src/copy.py"], "zero_token_paths": []},
            {"kind": "population", "population_ids": ["test.api"], "population_hash": HASH,
             "numerator": 1, "denominator": 2, "explanation": None},
            {"kind": "findings", "findings": [{"finding_id": "nesting.f", "location": span,
             "message": "Nesting depth", "value": 3}]},
        ]
        for payload in payloads:
            with self.subTest(kind=payload["kind"]):
                data = metric(); data["value"] = payload
                self.assertEqual(json.loads(c.dumps(self.read("MetricEvidence", data))), data)
        data["value"] = payloads[2]
        data["value"]["numerator"] = 3
        self.reject("MetricEvidence", data, "numerator")

    def test_delta_zero_baseline_and_noncomparability_round_trip(self):
        data = {"key": "source_bytes", "status": "available", "before": 0, "after": 12,
                "absolute_delta": 12, "percentage_delta": None, "reason": None,
                "percentage_reason": "zero baseline", "unit": "bytes"}
        self.assertEqual(json.loads(c.dumps(self.read("MetricDelta", data))), data)
        data.update(status="non_comparable", absolute_delta=None, reason="definition changed")
        self.assertEqual(json.loads(c.dumps(self.read("MetricDelta", data))), data)
        data["percentage_reason"] = None
        self.reject("MetricDelta", data, "percentage_reason")

    def test_external_isolation_requires_evidence_and_finished_execution_requires_exit(self):
        self.reject("Protection", {"mode": "external_isolation_claimed", "provider_evidence": None}, "provider_evidence")
        data = {"command": {**command(), "argv": ["/usr/bin/python3"], "executable_version": "3.12.10"},
                "exit_code": None, "status": "completed", "elapsed_seconds": 1,
                "stdout_artifact": "out.log", "stderr_artifact": "err.log",
                "started_at": "2026-10-30T18:00:00Z", "environment_hash": HASH}
        self.reject("ExecutionResult", data, "exit_code")
        data["status"] = "error"; data["started_at"] = "2026-10-30T18:00:00"
        self.reject("ExecutionResult", data, "started_at")

    def test_utf16_and_windows_normalized_traversal_rejected(self):
        with self.assertRaisesRegex(ValueError, "UTF-8"):
            c.loads(getattr(c, "Profile", "Profile"), json.dumps(profile()).encode("utf-16"))
        for path in [".. /secret", "baseline/.../secret", "baseline/name. /out"]:
            data = metric(); data["raw_artifact"] = path
            self.reject("MetricEvidence", data, "raw_artifact")


if __name__ == "__main__":
    unittest.main()
