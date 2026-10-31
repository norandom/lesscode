"""Baseline-backed fixture infrastructure; no policy evaluator is implemented here."""

import hashlib
import json
from pathlib import Path
import shutil
import sys
import tempfile
import unittest

from lesscode_baseline import contracts as c
from lesscode_baseline.decision import compare
from lesscode_baseline.report import persist, recover
from lesscode_baseline.snapshot import capture_scope

FIXTURES = Path(__file__).parent / "fixtures/agent-quality-rules"


class BaselinePolicyFixtureTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve()

    def test_complete_storage_preserves_every_evaluation_outcome(self):
        for name in ("accepted", "rejected", "incomplete", "invalid"):
            with self.subTest(name=name):
                fixture = materialize(self.root / name, name)
                recovered = recover(fixture["run"])
                self.assertEqual(recovered.state, "complete")
                self.assertEqual(recovered.report.decision, name)
                self.assertEqual(recovered.report, fixture["report"])
                expected = fixture["expected"]
                by_id = {rule.rule_id: rule for rule in fixture["policy"].rules}
                self.assertEqual(set(by_id), {finding["rule_id"] for finding in expected["findings"]})
                metrics = {metric.key: metric for metric in recovered.report.candidate.metrics}
                for finding in expected["findings"]:
                    if finding["rule_id"] == "api":
                        self.assertEqual(finding["candidate"], recovered.report.candidate.checks[0].status)
                    else:
                        key = "source_bytes" if finding["rule_id"] == "size" else "duplication"
                        self.assertEqual(finding["candidate"], metrics[key].value)
                self.assertEqual(by_id["size"].absolute_maximum, 100)
                self.assertEqual(expected["baseline_decision"], name)
                self.assertEqual(expected["accepted"], name == "accepted")
                comparison = compare(recovered.report.baseline, recovered.report.candidate,
                                     fixture["prepared"].plan.profile, recovered.report.integrity)
                self.assertEqual(comparison.decision, name)

    def test_every_identity_resolves_to_real_materialized_bytes(self):
        fixture = materialize(self.root, "accepted")
        prepared, report = fixture["prepared"], fixture["report"]
        self.assertEqual(report.plan_hash, digest(c.dumps(prepared).encode()))
        self.assertEqual(prepared.plan.profile_hash, digest(c.dumps(prepared.plan.profile).encode()))
        self.assertEqual(c.loads(c.Profile, Path(prepared.profile_path).read_bytes()), prepared.plan.profile)
        resource_identity = json.dumps([(r.logical_path, r.path, r.content_hash, r.size_bytes)
                                        for r in prepared.evaluator_resources], separators=(",", ":")).encode()
        self.assertEqual(prepared.plan.evaluator_hash, digest(resource_identity))
        for resource in prepared.evaluator_resources:
            data = Path(resource.path).read_bytes()
            self.assertEqual((resource.content_hash, resource.size_bytes), (digest(data), len(data)))
        for side in ("baseline", "candidate"):
            observation = getattr(report, side)
            self.assertEqual(observation.snapshot, getattr(prepared.plan, side))
            self.assertEqual(capture_scope(Path(observation.snapshot.root), prepared.plan.profile.scope), observation.snapshot)
            for metric in observation.metrics:
                self.assertEqual(metric.scope_hash, observation.snapshot.scope_hash)
                self.assertTrue((fixture["run"] / metric.raw_artifact).is_file())
            self.assertEqual(set(prepared.plan.profile.required_metrics), {m.key for m in observation.metrics})
            for check in observation.checks:
                self.assertEqual((check.source_hash, check.plan_hash), (observation.snapshot.content_hash, report.plan_hash))
                self.assertIn(check.check_id, prepared.plan.profile.checks[0].expected_ids)
        index = json.loads((fixture["run"] / "artifacts.json").read_bytes())
        self.assertIn("plan.json", {a["path"] for a in index["artifacts"]})
        for artifact in index["artifacts"]:
            data = (fixture["run"] / artifact["path"]).read_bytes()
            self.assertEqual((artifact["content_hash"], artifact["size_bytes"]), (digest(data), len(data)))

    def test_corruption_and_interrupted_storage_are_not_evaluation_decisions(self):
        for name in ("corrupt", "partial"):
            fixture = materialize(self.root / name, name)
            self.assertEqual(recover(fixture["run"]).state, "incomplete")
            self.assertIsNone(recover(fixture["run"]).report)
            self.assertEqual(fixture["report"].decision, "accepted")

    def test_independent_findings_retain_insufficient_duplication(self):
        fixture = materialize(self.root, "incomplete")
        metric = fixture["report"].candidate.metrics[-1]
        self.assertEqual(metric.key, "duplication")
        self.assertEqual(metric.status, "unavailable")
        self.assertIsNone(metric.value)
        self.assertIn("inventory", metric.reason)
        self.assertIn("short.py", {f.path for f in fixture["report"].candidate.snapshot.files if f.size_bytes})
        self.assertEqual(fixture["expected"]["findings"][-1], {"rule_id": "duplication", "status": "insufficient", "candidate": None})

    def test_materialization_is_repeatable_without_fixed_machine_paths(self):
        first = materialize(self.root / "one", "accepted")
        second = materialize(self.root / "two", "accepted")
        self.assertEqual(first["report"].candidate.snapshot.content_hash, second["report"].candidate.snapshot.content_hash)
        self.assertEqual(first["prepared"].plan.profile_hash, second["prepared"].plan.profile_hash)
        self.assertEqual(first["expected"], second["expected"])
        self.assertNotEqual(first["report"].plan_hash, second["report"].plan_hash)
        with self.assertRaises(FileExistsError):
            materialize(self.root / "one", "accepted")

    def test_actual_short_nonempty_file_is_not_zero_duplication(self):
        from lesscode_baseline.metrics import measure_duplication
        from tests.test_metrics import DuplicationMetricTests
        node = shutil.which("node")
        entry = Path(__file__).resolve().parents[1] / ".tools/analyzers/node/node_modules/jscpd/bin/jscpd"
        if node is None or not entry.is_file():
            self.skipTest("pinned jscpd runtime unavailable")
        source = self.root / "source"
        source.mkdir()
        (source / "short.py").write_bytes(b"pass\n")
        (source / "other.py").write_bytes(DuplicationMetricTests.duplicate)
        scope = c.ScopeRules(("**/*",), (), (), (), (), ("**/*.py",))
        snapshot = capture_scope(source, scope)
        metric = measure_duplication(snapshot, Path(sys.executable), Path(node), entry,
                                     c.Limits(30, 100000), self.root / "logs")
        self.assertEqual(metric.status, "unavailable", metric.reason)
        self.assertIsNone(metric.value)
        self.assertIn("inventory", metric.reason)
        self.assertEqual(snapshot.files, capture_scope(source, scope).files)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def materialize(root, name="accepted"):
    """Build isolated, hash-consistent synthetic stores for downstream policy tests.

    Baseline contracts, capture_scope, decision.compare and persist/recover remain
    authoritative. This deliberately does not call policy evaluation. Absolute
    roots are captured at materialization time; source/profile identities remain
    portable but plan hashes necessarily change when the root changes. Final
    storage timestamps are owned by RunStore, not deterministic fixture data.
    """
    from lesscode_baseline.rules import contracts as rules

    expected_cases = json.loads((FIXTURES / "expected.json").read_text(encoding="utf-8-sig"))["cases"]
    outcome = "accepted" if name in ("corrupt", "partial") else name
    if outcome not in expected_cases:
        raise ValueError(f"unknown baseline fixture: {name}")
    root = Path(root).resolve()
    root.mkdir(parents=True, exist_ok=True)
    evaluator = root / "evaluator"
    evaluator.mkdir()  # Reusing a fixture root must not overwrite evidence.
    raw = root / "raw"
    raw.mkdir()
    source_bytes = (FIXTURES / ("short.py" if outcome == "incomplete" else "complete.py")).read_bytes()
    scope = c.ScopeRules(("**/*",), (), (), (), (), ("**/*.py",))
    snapshots = []
    for side in ("baseline", "candidate"):
        source = root / side
        source.mkdir()
        (source / "short.py").write_bytes(source_bytes)
        snapshot = capture_scope(source, scope)
        if isinstance(snapshot, c.Failure):
            raise AssertionError(snapshot.message)
        snapshots.append(snapshot)
    limits = c.Limits(30, 100000)
    oracle = evaluator / "oracle.py"
    oracle.write_bytes(b"# Synthetic fixture oracle; normalized evidence is supplied, not executed.\n")
    command = c.CommandSpec("api", ("python", "oracle.py"), ".", {}, limits)
    spec = c.CheckSpec("api", "native", True, command, "checks.json", "normalized", ("test.api",))
    required = ("complexity", "duplication", "source_bytes")
    profile = c.Profile(1, scope, (), (spec,), required, (), limits, ("oracle.py",))
    profile_path = evaluator / "profile.json"
    profile_path.write_text(c.dumps(profile), encoding="utf-8")
    resources = tuple(c.ResourceIdentity(logical, str(path), digest(path.read_bytes()), len(path.read_bytes()))
                      for logical, path in (("oracle.py", oracle), ("_context/profile.json", profile_path)))
    # Matches baseline's documented resource inventory identity, without calling
    # private helpers or substituting a sentinel hash.
    resource_hash = digest(json.dumps([(r.logical_path, r.path, r.content_hash, r.size_bytes)
                                      for r in resources], separators=(",", ":")).encode())
    protection = c.Protection("local_detection", None)
    resolved = c.ResolvedCommand("api", (str(Path(sys.executable).absolute()), str(oracle)),
                                 ".", {}, limits, "3.12.10")
    plan = c.Plan(1, profile, digest(c.dumps(profile).encode()), resource_hash,
                  *snapshots, (resolved,), protection)
    prepared = c.loads(c.PreparedPlan, c.dumps(c.PreparedPlan(
        1, plan, str(profile_path), str(evaluator), resources, ())))
    plan_hash = digest(c.dumps(prepared).encode())
    observations = []
    for side, snapshot in zip(("baseline", "candidate"), snapshots):
        folder = raw / side
        folder.mkdir()
        size_path = f"{side}/size.json"
        size_value = sum(f.size_bytes for f in snapshot.files if f.category == "product")
        (raw / size_path).write_text(json.dumps({"source_hash": snapshot.content_hash,
            "plan_hash": plan_hash, "source_bytes": size_value}), encoding="utf-8")
        metrics = [c.MetricEvidence("source_bytes", "available", size_value, "bytes",
            "raw-source-bytes-v1", "synthetic-fixture", "1", snapshot.scope_hash, None, size_path)]
        complexity = c.ComplexityPayload("complexity", (c.FileComplexity("short.py", 1 if outcome == "incomplete" else 5),), (), 0, None, None, None, None)
        complexity_path = f"{side}/complexity.json"
        (raw / complexity_path).write_text(c.dumps(complexity), encoding="utf-8")
        metrics.append(c.MetricEvidence("complexity", "available", complexity, "CCN",
            "synthetic-complexity-v1", "synthetic-fixture", "1", snapshot.scope_hash, None, complexity_path))
        if outcome != "incomplete":
            duplication = c.DuplicationPayload("duplication", 0, 20, 0, None, "weak", 0, (), ("short.py",), ())
            duplication_path = f"{side}/duplication.json"
            (raw / duplication_path).write_text(c.dumps(duplication), encoding="utf-8")
            metrics.append(c.MetricEvidence("duplication", "available", duplication, "ratio",
                "synthetic-pair-span-v1", "synthetic-fixture", "1", snapshot.scope_hash, None, duplication_path))
        if outcome == "incomplete":
            reason = "duplication inventory omits nonempty short.py; cannot classify missing coverage as zero"
            artifact = f"{side}/duplication.json"
            (raw / artifact).write_text(json.dumps({"provenance": "synthetic omission case",
                "source_hash": snapshot.content_hash, "plan_hash": plan_hash,
                "scanned_paths": [], "omitted_nonempty_paths": ["short.py"], "reason": reason}), encoding="utf-8")
            metrics.append(c.MetricEvidence("duplication", "unavailable", None, "ratio",
                "synthetic-pair-span-v1", "synthetic-fixture", "1", snapshot.scope_hash, reason, artifact))
        status = "failed" if outcome == "rejected" and side == "candidate" else "passed"
        artifact = f"{side}/checks.json"
        check = c.CheckEvidence("test.api", status, "native", True, snapshot.content_hash, plan_hash, artifact)
        (raw / artifact).write_text(c.dumps(check), encoding="utf-8")
        observations.append(c.Observation(snapshot, tuple(metrics), (check,), (), ()))
    integrity = c.IntegrityResult("changed" if outcome == "invalid" else "unchanged",
                                 ("oracle.py",) if outcome == "invalid" else (), protection)
    comparison = compare(*observations, profile, integrity)
    report = c.RunReport(1, "fixture-" + outcome, plan_hash, *observations, integrity,
        comparison.decision, comparison.reasons, comparison.metric_deltas, c.FileDelta((), (), ()),
        limits, 0, ("Synthetic normalized evidence; no workload or policy evaluator executed.",),
        "2026-10-31T00:00:00Z", "2026-10-31T00:00:00Z", c.EnvironmentInfo("fixture", "3.12.10", False))
    run = root / "run"
    persist(report, run, artifact_root=raw, prepared_plan=prepared)
    if recover(run).state != "complete":
        raise AssertionError("fixture must pass baseline recovery before intentional damage")
    if name == "corrupt":
        (run / "plan.json").write_bytes((run / "plan.json").read_bytes() + b" ")
    elif name == "partial":
        (run / "artifacts.json").unlink()
    policy_data = json.loads((FIXTURES / "policy.json").read_text(encoding="utf-8-sig"))
    if outcome != "incomplete":
        policy_data["rules"] = [rule for rule in policy_data["rules"] if rule["rule_id"] != "duplication"]
    policy = rules.Policy.from_dict(policy_data)
    return {"run": run, "prepared": prepared, "report": report, "policy": policy,
            "expected": expected_cases[outcome]}
