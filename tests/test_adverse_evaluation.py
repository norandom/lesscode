"""Adverse integrated evidence and durable analyzer audit regressions."""

import json
from dataclasses import replace
import hashlib
from pathlib import Path
import shutil
import sys
import tempfile
import unittest
from unittest.mock import patch

from lesscode_baseline import config, evaluation, metrics, report as reports
from lesscode_baseline.contracts import ExecutionResult, Failure, Limits, PreparedPlan, dumps
from lesscode_baseline.snapshot import capture_scope
from tests import test_config as support

ROOT = Path(__file__).resolve().parents[1]


class FixedOptionsTests(unittest.TestCase):
    setUp = support.PreparationTests.setUp
    save = support.PreparationTests.save
    prepare = support.PreparationTests.prepare

    def test_unsupported_options_fail_before_any_probe(self):
        for name, key, value in (("jscpd", "mode", "strict"),
                                 ("jscpd", "unknown", "ignored"),
                                 ("jscpd", "maxLines", 100000),
                                 ("jscpd", "maxSize", "10mb"),
                                 ("jscpd", "skipComments", False),
                                 ("jscpd", "minLines", True),
                                 ("jscpd", "ignoreCase", 0),
                                 ("lizard", "extensions", 0)):
            with self.subTest(name=name, key=key):
                data = json.loads(self.path.read_text())
                tool = next(t for t in data["tools"] if t["name"] == name)
                tool["options"][key] = value
                self.path.write_text(json.dumps(data))
                with patch.object(config, "_binding", wraps=config._binding) as probe:
                    result = self.prepare(tool_paths={})
                self.assertIsInstance(result, Failure)
                self.assertIn(key, result.message)
                probe.assert_not_called()
                self.save()

    def test_omitted_options_mean_fixed_defaults(self):
        for tool in self.profile["tools"]:
            tool["options"] = {}
        self.save()
        self.assertIsInstance(self.prepare(tool_paths={}), PreparedPlan)

    def test_rehashed_frozen_rule_change_cannot_diverge_from_recorded_profile(self):
        plan = self.prepare(tool_paths={})
        changed = replace(plan.plan.profile.thresholds[0], value=999)
        profile = replace(plan.plan.profile, thresholds=(changed,))
        altered = replace(plan, plan=replace(plan.plan, profile=profile,
            profile_hash=hashlib.sha256(dumps(profile).encode()).hexdigest()))
        with patch.object(config, "_probe", side_effect=AssertionError("must not probe")):
            integrity = config.verify_integrity(altered)
        self.assertEqual(integrity.status, "changed")
        self.assertIn("_context/frozen-profile-content", integrity.changed_resources)


class AnalyzerAuditTests(unittest.TestCase):
    def test_size_execution_and_actual_inventory_survive_staging_cleanup(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source"
            source.mkdir()
            (source / "module.py").write_text("def answer():\n    return 42\n")
            scope = json.loads(
                (Path(__file__).resolve().parents[1] / "benchmarks/profiles/python.json").read_text())["scope"]
            from lesscode_baseline.contracts import ScopeRules
            snapshot = capture_scope(source, ScopeRules.from_dict(scope))
            logs = root / "logs"
            for _ in range(2):
                values = metrics.measure_sizes(snapshot, Path(sys.executable), Limits(10, 1048576), logs)
                self.assertTrue(all(value.status == "available" for value in values))
            audits = list(logs.glob("*.execution.json"))
            self.assertEqual(len(audits), 2)
            for path in audits:
                audit = json.loads(path.read_text())
                self.assertEqual(audit["schema_version"], 1)
                execution = ExecutionResult.from_dict(audit["execution"])
                self.assertEqual(execution.exit_code, 0)
                self.assertEqual(execution.status, "completed")
                self.assertGreater(execution.elapsed_seconds, 0)
                self.assertFalse(Path(audit["workspace"]).exists())
                self.assertEqual(audit["cwd"], audit["workspace"])
                self.assertTrue((logs / execution.stdout_artifact).is_file())
                inventory = Path(execution.command.argv[-1])
                self.assertTrue(inventory.is_relative_to(logs))
                self.assertEqual(json.loads(inventory.read_text()), ["module.py"])

    def test_audit_write_failure_propagates(self):
        from lesscode_baseline.contracts import ScopeRules
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "source").mkdir()
            (root / "source/main.py").write_text("value = 1\n")
            snapshot = capture_scope(root / "source", ScopeRules.from_dict(json.loads(
                (ROOT / "benchmarks/profiles/python.json").read_text())["scope"]))
            original = Path.open
            def fail(path, *args, **kwargs):
                if path.name.endswith(".execution.json"):
                    raise OSError("audit disk full")
                return original(path, *args, **kwargs)
            with patch.object(Path, "open", fail):
                with self.assertRaisesRegex(OSError, "audit disk full"):
                    metrics.measure_sizes(snapshot, Path(sys.executable), Limits(10, 1048576), root / "logs")


class AdverseEvaluationTests(unittest.TestCase):
    """Actual pinned analyzers and actual independent checks unless stated otherwise."""

    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.output = self.root / "run"
        for side in ("baseline", "candidate"):
            shutil.copytree(ROOT / "tests/fixtures/real/boltons_mathutils/source", self.root / side)
        self.evaluator = self.root / "evaluator"
        shutil.copytree(ROOT / "benchmarks/profiles", self.evaluator)
        self.profile = json.loads((self.evaluator / "python.json").read_text())
        self.tools = json.loads((ROOT / ".tools/analyzers/entrypoints.json").read_text())

    def plan(self, tools=None):
        (self.evaluator / "python.json").write_text(json.dumps(self.profile))
        plan = config.prepare(self.evaluator / "python.json", self.root / "baseline",
                              self.root / "candidate", self.output,
                              tool_paths=self.tools if tools is None else tools)
        self.assertIsInstance(plan, PreparedPlan)
        return plan

    def extra_check(self, body, *, required=True, expected=("extra.one",), result="extra.xml"):
        (self.evaluator / "extra.py").write_text(body, encoding="utf-8")
        self.profile["evaluator_resources"].append("extra.py")
        self.profile["checks"].append({"check_id":"extra", "kind":"invariant", "required":required,
            "command":{"command_id":"extra", "argv":["python", "extra.py"], "cwd":".",
                       "environment":{}, "limits":{"timeout_seconds":10,"max_log_bytes":1048576}},
            "result_path":result,"result_format":"junit","expected_ids":list(expected)})

    def run_plan(self, plan, **kwargs):
        report = evaluation.evaluate(plan, self.output, **kwargs)
        recovered = reports.recover(self.output)
        self.assertEqual(recovered.state, "complete", recovered.diagnostics)
        self.assertEqual(recovered.report, report)
        self.assertEqual(report.integrity.protection.mode, "local_detection")
        return report

    def test_optional_baseline_failure_is_ineligible_with_passing_core_evidence(self):
        self.extra_check("from pathlib import Path\nPath('extra.xml').write_text("
            "'<testsuite><testcase classname=\"extra\" name=\"one\"><failure/></testcase></testsuite>')\n",
            required=False)
        result = self.run_plan(self.plan())
        self.assertEqual(result.decision, "baseline_ineligible", result.reasons)
        self.assertTrue(all(m.status == "available" for m in result.baseline.metrics))
        self.assertTrue(all(c.status == "passed" for c in result.baseline.checks if c.required))
        self.assertEqual(next(c for c in result.baseline.checks if not c.required).status, "failed")

    def test_optional_metric_absence_is_nonblocking_and_audits_are_indexed(self):
        result = self.run_plan(self.plan(), optional_evidence=(evaluation.OptionalEvidenceInput(
            "candidate", self.root / "absent-coverage.json", "producer_supplied"),))
        self.assertEqual(result.decision, "accepted", result.reasons)
        self.assertTrue(any(f.code == "optional_evidence_missing" for f in result.candidate.failures))
        index = reports.recover(self.output).index
        indexed = {item.path for item in index.artifacts}
        audits = list(self.output.rglob("*.execution.json"))
        self.assertEqual(len(audits), 10)  # syntax, Lizard, duplication syntax/inventory/CLI per side
        for path in audits:
            self.assertIn(path.relative_to(self.output).as_posix(), indexed)
            audit = json.loads(path.read_text())
            execution = ExecutionResult.from_dict(audit["execution"])
            self.assertFalse(Path(audit["workspace"]).exists())
            for artifact in (execution.stdout_artifact, execution.stderr_artifact):
                self.assertIn((path.parent / artifact).relative_to(self.output).as_posix(), indexed)
            for argument in execution.command.argv:
                if argument.endswith((".inventory.json", ".jscpd.json")):
                    self.assertIn(Path(argument).relative_to(self.output).as_posix(), indexed)
            if execution.command.command_id == "metrics-duplication":
                effective = json.loads(Path(execution.command.argv[3]).read_text())
                self.assertEqual(effective["mode"], "weak")
                self.assertGreater(effective["maxLines"], 100)
                self.assertTrue(effective["maxSize"].endswith("b"))
        self.assertEqual(len(result.baseline.executions), 2)
        self.assertTrue(all(e.command.command_id in ("native", "independent")
                            for e in result.baseline.executions))

    def test_missing_analyzers_are_incomplete_when_behavior_checks_pass(self):
        result = self.run_plan(self.plan(tools={"python":self.tools["python"]}))
        self.assertEqual(result.decision, "incomplete", result.reasons)
        self.assertTrue(all(c.status == "passed" for c in result.baseline.checks))
        unavailable = [m for m in result.candidate.metrics if m.key == "duplication"]
        self.assertEqual(len(unavailable), 1)
        self.assertTrue(all(m.status == "unavailable" and m.value is None and m.reason for m in unavailable))

    def test_candidate_suppression_and_silent_omission_are_incomplete(self):
        for variant, content, key, diagnostic in (
            ("suppression", "# jscpd:ignore-start\n", "duplication", "suppression"),
            ("omitted", "value = 1\n", "duplication", "inventory")):
            with self.subTest(variant=variant):
                self.output = self.root / variant
                path = self.root / "candidate/added.py"
                path.write_text(content)
                # Keep the predeclared size threshold out of this missing-evidence
                # test so a larger fixture cannot mask the incomplete state.
                self.profile["thresholds"] = []
                result = self.run_plan(self.plan())
                metric = next(m for m in result.candidate.metrics if m.key == key)
                self.assertEqual(metric.status, "unavailable")
                self.assertIn(diagnostic, metric.reason)
                self.assertTrue(all(c.status == "passed" for c in result.candidate.checks))
                self.assertEqual(result.decision, "incomplete", result.reasons)

    def test_parse_error_retains_independent_successes_and_rejects_behavior(self):
        (self.root / "candidate/mathutils.py").write_text("def invalid(:\n")
        result = self.run_plan(self.plan())
        self.assertTrue(all(m.status == "available" for m in result.baseline.metrics))
        self.assertTrue(all(m.status == "unavailable" and m.value is None for m in result.candidate.metrics))
        self.assertEqual(result.decision, "rejected", result.reasons)
        self.assertTrue(all(c.status != "passed" for c in result.candidate.checks))

    def test_deleted_candidate_check_cannot_reduce_frozen_inventory(self):
        self.extra_check("from pathlib import Path\nexists=Path('tests/contract.txt').exists()\n"
            "Path('extra.xml').write_text('<testsuite>'+"
            "('<testcase classname=\"extra\" name=\"one\"/>' if exists else '')+'</testsuite>')\n")
        (self.root / "baseline/tests").mkdir()
        (self.root / "baseline/tests/contract.txt").write_text("declared test")
        result = self.run_plan(self.plan())
        self.assertEqual(result.decision, "incomplete", result.reasons)
        self.assertEqual(next(c for c in result.baseline.checks if c.check_id == "extra.one").status, "passed")
        self.assertEqual(next(c for c in result.candidate.checks if c.check_id == "extra.one").status, "missing")
        self.assertIn("tests/contract.txt", result.test_file_delta.removed)

    def test_stale_report_cannot_substitute_for_new_output(self):
        self.extra_check("pass\n")
        for side in ("baseline", "candidate"):
            (self.root / side / "extra.xml").write_text(
                '<testsuite><testcase classname="extra" name="one"/></testsuite>')
        # Persistence/check-specific fault: avoid unrelated expensive metrics.
        # This test makes no claim of real analyzer evidence.
        with patch.object(evaluation.metrics, "measure_sizes", return_value=()):
            plan = self.plan(tools={"python":self.tools["python"]})
            result = self.run_plan(plan)
        self.assertEqual(result.decision, "baseline_ineligible")
        self.assertTrue(any(f.code == "check_report_missing" for f in result.baseline.failures))

    def test_timeout_of_child_spawning_check_retains_diagnostics(self):
        marker = self.root / "descendant-survived"
        child = f"import time;time.sleep(1.8);open({str(marker)!r},'w').write('bad')"
        self.extra_check("import os,subprocess,sys,time\n"
            f"child=subprocess.Popen([sys.executable,'-c',{child!r}])\n"
            "print(os.getpid(),child.pid,flush=True)\ntime.sleep(30)\n")
        self.profile["checks"][-1]["command"]["limits"]["timeout_seconds"] = 1
        result = self.run_plan(self.plan())
        self.assertEqual(result.decision, "baseline_ineligible")
        for observation in (result.baseline, result.candidate):
            execution = next(e for e in observation.executions if e.command.command_id == "extra")
            self.assertEqual(execution.status, "timeout")
            self.assertEqual(len((self.output / execution.stdout_artifact).read_text().split()), 2)
        import time
        time.sleep(1)
        self.assertFalse(marker.exists())

    def test_real_evaluator_mutation_is_observable(self):
        plan = self.plan(tools={"python":self.tools["python"]})
        original = evaluation.metrics.measure_sizes
        def tamper(*args, **kwargs):
            result = original(*args, **kwargs)
            with (self.evaluator / "pilot/golden.json").open("a") as handle:
                handle.write("\n")
            return result
        with patch.object(evaluation.metrics, "measure_sizes", side_effect=tamper):
            result = self.run_plan(plan)
        self.assertEqual(result.decision, "invalid")
        self.assertIn("pilot/golden.json", result.integrity.changed_resources)
        self.assertTrue(list(self.output.rglob("*.execution.json")))
        self.assertTrue(all(c.status == "missing" for c in result.candidate.checks))

    def test_interrupted_final_index_retains_partial_run_without_recovery_success(self):
        # Index publication is independent of analyzer execution; isolate this
        # persistence fault using unavailable bindings, not fabricated metrics.
        plan = self.plan(tools={})
        atomic = reports.RunStore._atomic
        def interrupt(store, relative, data):
            if relative == "artifacts.json":
                raise KeyboardInterrupt("interrupted completion publication")
            return atomic(store, relative, data)
        with patch.object(reports.RunStore, "_atomic", interrupt):
            with self.assertRaisesRegex(KeyboardInterrupt, "publication"):
                evaluation.evaluate(plan, self.output)
        recovered = reports.recover(self.output)
        self.assertEqual(recovered.state, "incomplete")
        self.assertIsNone(recovered.report)
        self.assertTrue((self.output / "report.json").is_file())
        self.assertTrue(list((self.output / "partial").glob("*.json")))
        self.assertFalse((self.output / "artifacts.json").exists())
