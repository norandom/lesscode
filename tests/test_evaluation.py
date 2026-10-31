"""Pair integration preserves declared inventory and durable diagnostics."""

import hashlib
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch

from lesscode_baseline.config import prepare
from lesscode_baseline.contracts import PreparedPlan, dumps
from lesscode_baseline.evaluation import OptionalEvidenceInput, evaluate
from lesscode_baseline.report import PersistenceError, RunStore, recover

ROOT = Path(__file__).resolve().parents[1]
PILOT = ROOT / "tests/fixtures/real/boltons_mathutils/source"


class EvaluationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.output = self.root / "run"
        for side in ("baseline", "candidate"):
            shutil.copytree(PILOT, self.root / side)

    def plan(self, tools=None):
        result = prepare(ROOT / "benchmarks/profiles/python.json",
                         self.root / "baseline", self.root / "candidate",
                         self.output, tool_paths={} if tools is None else tools)
        self.assertIsInstance(result, PreparedPlan)
        return result

    def test_missing_bindings_retain_inventory_and_do_not_execute(self):
        plan = self.plan()
        result = evaluate(plan, self.output)
        self.assertEqual(result.decision, "baseline_ineligible")
        self.assertEqual(result.plan_hash, hashlib.sha256(dumps(plan).encode()).hexdigest())
        self.assertEqual(len(result.baseline.checks), 3)
        self.assertTrue(all(c.status == "missing" for c in result.baseline.checks))
        self.assertEqual(result.baseline.executions, ())
        self.assertTrue(result.baseline.failures)
        self.assertEqual(recover(self.output).report, result)

    def test_changed_source_is_invalid_and_persisted(self):
        plan = self.plan()
        (self.root / "candidate" / "untracked.py").write_text("value = 1\n")
        result = evaluate(plan, self.output)
        self.assertEqual(result.decision, "invalid")
        self.assertIn("candidate/source", result.integrity.changed_resources)
        self.assertEqual(result.candidate.executions, ())
        self.assertEqual(recover(self.output).report, result)

    def test_changed_evaluator_is_invalid_without_workloads(self):
        from dataclasses import replace
        plan = self.plan()
        plan = replace(plan, plan=replace(plan.plan, evaluator_hash="f" * 64))
        result = evaluate(plan, self.output)
        self.assertEqual(result.decision, "invalid")
        self.assertIn("_context/evaluator-inventory", result.integrity.changed_resources)
        self.assertEqual(recover(self.output).report, result)

    def test_interruption_retains_running_manifest_and_partial_observations(self):
        from lesscode_baseline import evaluation
        plan = self.plan()
        def interrupted(*args, **kwargs):
            raise KeyboardInterrupt("stop after initial checkpoint")
        with patch.object(evaluation.snapshot, "disposable_workspace", interrupted):
            with self.assertRaises(KeyboardInterrupt):
                evaluate(plan, self.output)
        recovery = recover(self.output)
        self.assertIsNone(recovery.report)
        self.assertTrue(list((self.output / "partial").glob("*.json")))
        self.assertFalse((self.output / "artifacts.json").exists())

    def test_persistence_failure_propagates_surviving_directory(self):
        plan = self.plan()
        with patch.object(RunStore, "finalize", side_effect=PersistenceError(self.output, "disk failure")):
            with self.assertRaises(PersistenceError) as caught:
                evaluate(plan, self.output)
        self.assertEqual(caught.exception.directory, self.output)
        self.assertEqual(caught.exception.exit_code, 4)
        self.assertIsNone(recover(self.output).report)

    def test_lost_registered_artifact_cannot_publish_a_complete_run(self):
        finalize = RunStore.finalize
        def lose_artifact(store, report):
            artifact = store.manifest.raw_artifacts[0]
            (store.directory / artifact.path).unlink()
            return finalize(store, report)
        with patch.object(RunStore, "finalize", lose_artifact):
            with self.assertRaises(PersistenceError):
                evaluate(self.plan(), self.output)
        self.assertFalse((self.output / "artifacts.json").exists())
        self.assertIsNone(recover(self.output).report)

    def test_test_file_changes_are_reported_outside_product_metrics(self):
        testdir = self.root / "candidate/tests"
        testdir.mkdir()
        (testdir / "test_added.py").write_text("assert True\n")
        result = evaluate(self.plan(), self.output)
        self.assertEqual(result.test_file_delta.added, ("tests/test_added.py",))

    def test_during_workload_tampering_retains_measurement_and_invalidates(self):
        from lesscode_baseline import evaluation
        evaluator = self.root / "evaluator"
        shutil.copytree(ROOT / "benchmarks/profiles", evaluator)
        tools = json.loads((ROOT / ".tools/analyzers/entrypoints.json").read_text())
        plan = prepare(evaluator / "python.json", self.root / "baseline", self.root / "candidate",
                       self.output, tool_paths={"python": tools["python"]})
        measure = evaluation.metrics.measure_sizes
        def tampering(*args, **kwargs):
            result = measure(*args, **kwargs)
            with (evaluator / "pilot/check.py").open("a") as handle:
                handle.write("\n# changed during workload\n")
            return result
        with patch.object(evaluation.metrics, "measure_sizes", tampering):
            report = evaluate(plan, self.output)
        self.assertEqual(report.decision, "invalid")
        self.assertIn("pilot/check.py", report.integrity.changed_resources)
        self.assertTrue(report.baseline.metrics)
        self.assertEqual(report.baseline.executions, ())
        self.assertTrue(all(check.status == "missing" for check in report.candidate.checks))
        self.assertEqual(recover(self.output).report, report)


class OptionalEvaluationTests(unittest.TestCase):
    # Reuse only setup/helpers, keeping these regressions independent of real
    # analyzer execution and the longer pinned-pair tests.
    setUp = EvaluationTests.setUp
    plan = EvaluationTests.plan
    def optional(self, plan, *, source_hash=None, plan_hash=None, key="coverage", origin="producer_supplied"):
        from lesscode_baseline.contracts import MetricEvidence, OptionalEvidenceArtifact, PopulationPayload
        payload = PopulationPayload("population", ("case.one", "case.two"), "a" * 64, 1, 2,
                                    "Measured fixed population")
        metric = MetricEvidence(key, "available", payload, "ratio", "fixture-coverage-v1",
                                "fixture-coverage", "1", plan.plan.baseline.scope_hash, None, None)
        envelope = OptionalEvidenceArtifact(1, "coverage-one", source_hash or plan.plan.baseline.content_hash,
            plan_hash or hashlib.sha256(dumps(plan).encode()).hexdigest(), "2026-10-01T00:00:00Z",
            origin, (metric,), ())
        path = self.root / (key + ".json")
        path.write_text(dumps(envelope), encoding="utf-8")
        return envelope, path

    def test_optional_payload_and_claim_are_retained(self):
        plan = self.plan()
        envelope, path = self.optional(plan)
        result = evaluate(plan, self.output, optional_evidence=(
            OptionalEvidenceInput("baseline", path, "producer_supplied"),))
        metric = next(metric for metric in result.baseline.metrics if metric.key == "coverage")
        self.assertEqual(metric.value, envelope.metrics[0].value)
        self.assertEqual(json.loads((self.output / metric.raw_artifact).read_text()), envelope.to_dict())
        self.assertTrue(any("producer claim" in limitation for limitation in result.limitations))

    def test_optional_foreign_source_and_plan_are_rejected(self):
        for changed in ("source_hash", "plan_hash"):
            with self.subTest(changed=changed):
                self.output = self.root / changed
                plan = self.plan()
                envelope, path = self.optional(plan, **{changed: "f" * 64})
                result = evaluate(plan, self.output, optional_evidence=(
                    OptionalEvidenceInput("baseline", path, "producer_supplied"),))
                self.assertFalse(any(metric.key == "coverage" for metric in result.baseline.metrics))
                self.assertTrue(any(failure.code == "optional_evidence_invalid" for failure in result.baseline.failures))
                self.assertEqual(recover(self.output).report, result)

    def test_optional_duplicate_cannot_override_a_core_metric(self):
        plan = self.plan()
        envelope, path = self.optional(plan, key="source_bytes")
        result = evaluate(plan, self.output, optional_evidence=(
            OptionalEvidenceInput("baseline", path, "producer_supplied"),))
        core = next(metric for metric in result.baseline.metrics if metric.key == "source_bytes")
        self.assertEqual(core.status, "unavailable")
        self.assertTrue(any(failure.code == "optional_evidence_duplicate" for failure in result.baseline.failures))

    def test_optional_duplicate_requests_are_visible_and_never_double_inserted(self):
        plan = self.plan()
        envelope, path = self.optional(plan)
        request = OptionalEvidenceInput("baseline", path, "producer_supplied")
        result = evaluate(plan, self.output, optional_evidence=(request, request))
        self.assertEqual(sum(metric.key == "coverage" for metric in result.baseline.metrics), 1)
        self.assertTrue(any(failure.code == "optional_evidence_duplicate" for failure in result.baseline.failures))
        self.assertEqual(recover(self.output).report, result)

    def test_optional_missing_artifact_is_a_reported_limitation(self):
        plan = self.plan()
        result = evaluate(plan, self.output, optional_evidence=(
            OptionalEvidenceInput("candidate", self.root / "missing.json", "producer_supplied"),))
        self.assertTrue(any(failure.code == "optional_evidence_missing" for failure in result.candidate.failures))
        self.assertTrue(any("optional_evidence_missing" in reason for reason in result.limitations))

    def test_optional_generated_requires_explicit_aware_freshness(self):
        from datetime import datetime, timezone
        from lesscode_baseline.contracts import ContractError
        plan = self.plan()
        envelope, path = self.optional(plan, origin="generated")
        with self.assertRaises(ContractError):
            evaluate(plan, self.output, optional_evidence=(OptionalEvidenceInput("baseline", path, "generated"),))
        self.assertFalse(self.output.exists())
        result = evaluate(plan, self.output, optional_evidence=(
            OptionalEvidenceInput("baseline", path, "generated", datetime.now(timezone.utc)),))
        self.assertFalse(any(metric.key == "coverage" for metric in result.baseline.metrics))
        self.assertTrue(any(failure.code == "optional_evidence_invalid" for failure in result.baseline.failures))


class RealEvaluationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary = tempfile.TemporaryDirectory()
        cls.addClassCleanup(cls.temporary.cleanup)
        cls.root = Path(cls.temporary.name)
        cls.tools = json.loads((ROOT / ".tools/analyzers/entrypoints.json").read_text())
        cls.results = {}
        cls.check_order = []
        for variant in ("unchanged", "broken"):
            root = cls.root / variant
            root.mkdir()
            for side in ("baseline", "candidate"):
                shutil.copytree(PILOT, root / side)
            if variant == "broken":
                source = root / "candidate/mathutils.py"
                text = source.read_text(encoding="utf-8")
                start = text.index("def clamp(")
                end = text.index("\n\ndef ", start + 1)
                source.write_text(text[:start] + "def clamp(x, lower=-float('inf'), upper=float('inf')):\n    return x\n" + text[end:], encoding="utf-8")
            plan = prepare(ROOT / "benchmarks/profiles/python.json", root / "baseline",
                           root / "candidate", root / "run", tool_paths=cls.tools)
            if not isinstance(plan, PreparedPlan) or any(tool.status != "available" for tool in plan.tools):
                raise AssertionError("real integration requires explicit installed pinned tools")
            if variant == "unchanged":
                from lesscode_baseline import evaluation
                runcheck = evaluation.process.execute_check
                original = (root / "baseline/mathutils.py").read_bytes()
                def writing_check(spec, command, workspace, *args):
                    cls.check_order.append(spec.check_id)
                    if (workspace / "mathutils.py").read_bytes() != original:
                        raise AssertionError("check inherited writes from prior check")
                    result = runcheck(spec, command, workspace, *args)
                    (workspace / "mathutils.py").write_text("ordinary test modification\n")
                    return result
                with patch.object(evaluation.process, "execute_check", writing_check):
                    report = evaluate(plan, root / "run")
            else:
                report = evaluate(plan, root / "run")
            cls.results[variant] = (plan, report, root)

    def test_real_unchanged_pair_is_accepted_with_complete_artifact_index(self):
        plan, report, root = self.results["unchanged"]
        self.assertEqual(report.decision, "accepted", report.reasons)
        self.assertEqual(report.integrity.status, "unchanged")
        self.assertEqual([(m.key, m.value) for m in report.baseline.metrics],
                         [(m.key, m.value) for m in report.candidate.metrics])
        self.assertEqual(len(report.baseline.executions), 2)
        self.assertEqual(self.check_order, ["native", "independent", "native", "independent"])
        self.assertTrue(all(c.status == "passed" for c in report.candidate.checks))
        self.assertEqual(recover(root / "run").report, report)
        from lesscode_baseline.snapshot import capture_scope
        self.assertEqual(capture_scope(root / "baseline", plan.plan.profile.scope).content_hash,
                         plan.plan.baseline.content_hash)
        self.assertEqual(capture_scope(root / "candidate", plan.plan.profile.scope).content_hash,
                         plan.plan.candidate.content_hash)
        for observation in (report.baseline, report.candidate):
            for metric in observation.metrics:
                if metric.raw_artifact:
                    self.assertTrue((root / "run" / metric.raw_artifact).is_file())
            for execution in observation.executions:
                self.assertTrue((root / "run" / execution.stdout_artifact).is_file())
                self.assertTrue((root / "run" / execution.stderr_artifact).is_file())

    def test_real_shorter_broken_pair_rejected_without_reverting_source(self):
        plan, report, root = self.results["broken"]
        self.assertEqual(report.decision, "rejected", report.reasons)
        self.assertTrue(any(c.status == "failed" for c in report.candidate.checks))
        self.assertTrue(any(d.key == "source_bytes" and d.absolute_delta < 0 for d in report.metric_deltas))
        from lesscode_baseline.snapshot import capture_scope
        self.assertEqual(capture_scope(root / "candidate", plan.plan.profile.scope).content_hash,
                         plan.plan.candidate.content_hash)
        self.assertEqual(recover(root / "run").report, report)
