"""Recorded experiments use recovered evidence and publish a sealed summary."""

from contextlib import redirect_stdout, redirect_stderr
import hashlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from lesscode_baseline import cli, contracts as c
from lesscode_baseline.report import RunStore, recover_experiment
from tests.test_experiment_contracts import manifest
from tests.test_experiments import evaluated, producer


class ExperimentCLI(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.input = self.root / "inputs"
        self.input.mkdir()
        self.experiment = self.input / "experiment.json"
        self.experiment.write_text(json.dumps(manifest()), encoding="utf-8")
        self.runs = self.root / "evidence"
        self.runs.mkdir()
        self.out = self.root / "summary"
        self.records = []

    def add(self, arm="control", decision="accepted", **changes):
        report = evaluated(decision=decision,
                           baseline="failed" if decision == "baseline_ineligible" else "passed",
                           candidate="missing" if decision == "incomplete" else "passed")
        store = RunStore.create(self.runs / "runs" / arm, report.plan_hash, run_id=report.run_id)
        store.finalize(report)
        record = producer(arm, run_artifact=f"runs/{arm}/report.json", **changes)
        path = self.input / (arm + ".json")
        path.write_text(c.dumps(record), encoding="utf-8")
        self.records.append(path)
        return store.directory

    def run_cli(self):
        args = ["summarize", "--experiment", str(self.experiment), "--runs", str(self.runs), "--out", str(self.out)]
        for path in self.records:
            args += ["--trials", str(path)]
        self.stdout, self.stderr = io.StringIO(), io.StringIO()
        with redirect_stdout(self.stdout), redirect_stderr(self.stderr):
            return cli.main(args)

    def test_multi_arm_indexed_machine_and_human_evidence(self):
        self.add()
        self.add("skill", seed=99)
        before = {p: p.read_bytes() for root in (self.input, self.runs) for p in root.rglob("*") if p.is_file()}
        with patch("lesscode_baseline.evaluation.evaluate", side_effect=AssertionError("must not execute")):
            self.assertEqual(self.run_cli(), 0, self.stderr.getvalue())
        result = c.loads(c.ExperimentReport, (self.out / "report.json").read_bytes())
        self.assertEqual(len(result.trials), 2)
        self.assertEqual(result.arm_summaries[0].eligible_attempts, 1)
        text = (self.out / "summary.md").read_text(encoding="utf-8")
        for fragment in ("Correctness", "Acceptance", "All-attempt", "Accepted-only", "Descriptive", "confound", "provider usage export", "unavailable", "Evaluation elapsed", "Runtime", "MIT", "generation_provenance"):
            self.assertIn(fragment, text)
        evidence = json.loads((self.out / "evaluations.json").read_bytes())
        self.assertEqual(evidence["runs"][0]["report"]["elapsed_seconds"], 0.1)
        self.assertEqual(evidence["runs"][0]["origin"], "verified_artifacts")
        index = json.loads((self.out / "artifacts.json").read_bytes())
        for artifact in index["artifacts"]:
            data = (self.out / artifact["path"]).read_bytes()
            self.assertEqual(artifact["content_hash"], hashlib.sha256(data).hexdigest())
        self.assertEqual(before, {p: p.read_bytes() for p in before})
        self.assertEqual(recover_experiment(self.out).report, result)
        (self.out / "evaluations.json").write_bytes(b"corrupted")
        self.assertEqual(recover_experiment(self.out).state, "incomplete")

    def test_exit_precedence_and_ineligible_report_success(self):
        for decisions, expected in ((("accepted", "accepted"), 0), (("rejected", "accepted"), 1),
                                    (("rejected", "incomplete"), 3), (("invalid", "incomplete"), 2),
                                    (("baseline_ineligible", "accepted"), 0)):
            with self.subTest(decisions=decisions), tempfile.TemporaryDirectory() as directory:
                old_runs, old_out, old_records = self.runs, self.out, self.records
                self.runs, self.out, self.records = Path(directory) / "evidence", Path(directory) / "out", []
                self.runs.mkdir()
                for arm, decision in zip(("control", "skill"), decisions):
                    self.add(arm, decision)
                self.assertEqual(self.run_cli(), expected, self.stderr.getvalue())
                self.runs, self.out, self.records = old_runs, old_out, old_records

    def test_missing_trials_and_extras_keep_scheduled_denominator(self):
        self.add()
        self.add("skill", trial_id="extra")
        self.assertEqual(self.run_cli(), 3)
        report = c.loads(c.ExperimentReport, (self.out / "report.json").read_bytes())
        self.assertEqual(sum(a.planned_attempts for a in report.arm_summaries), 2)
        self.assertEqual(len(report.extra_trials), 1)

    def test_duplicate_and_unknown_record_fields_are_input_errors(self):
        self.add()
        self.records.append(self.records[0])
        self.assertEqual(self.run_cli(), 2)
        self.assertFalse(self.out.exists())
        self.records.pop()
        value = json.loads(self.records[0].read_bytes()) | {"unexpected": 1}
        self.records[0].write_text(json.dumps(value), encoding="utf-8")
        self.assertEqual(self.run_cli(), 2)
        self.assertFalse(self.out.exists())

    def test_duplicate_json_keys_rejected(self):
        self.add()
        path = self.records[0]
        path.write_text('{"trial_id":"forged",' + path.read_text(encoding="utf-8")[1:], encoding="utf-8")
        self.assertEqual(self.run_cli(), 2)
        self.assertFalse(self.out.exists())

    def test_unsealed_or_tampered_run_stays_incomplete(self):
        directory = self.add()
        self.add("skill")
        (directory / "report.json").write_bytes((directory / "report.json").read_bytes() + b" ")
        self.assertEqual(self.run_cli(), 3)
        value = json.loads((self.out / "report.json").read_bytes())
        self.assertEqual(value["outcomes"][0]["decision"], "incomplete")

    def test_existing_output_and_input_overlap_rejected(self):
        self.add()
        for output in (self.runs / "out", self.experiment, self.runs.parent):
            self.out = output
            self.assertEqual(self.run_cli(), 2)

    def test_new_output_sibling_to_manifest_preserves_unconfounded_pairs(self):
        self.add()
        self.add("skill")
        self.out = self.input / "summary"
        self.assertEqual(self.run_cli(), 0, self.stderr.getvalue())
        result = recover_experiment(self.out).report
        self.assertEqual(result.paired_summaries[0].differences[0].n, 1)
        self.assertTrue(all(not outcome.confounded_differences for outcome in result.outcomes))

    def test_producer_markup_is_inert(self):
        value = manifest()
        value["corpus"][0]["origin"] = "<script>alert(1)</script>\n# forged"
        self.experiment.write_text(json.dumps(value), encoding="utf-8")
        self.assertEqual(self.run_cli(), 3)
        text = (self.out / "summary.md").read_text(encoding="utf-8")
        self.assertNotIn("<script>", text)
        self.assertNotIn("\n# forged", text)
        self.assertIn("&lt;script&gt;", text)

    def test_write_failure_has_no_completion_index(self):
        self.add()
        from lesscode_baseline import report
        original = report.os.replace
        def fail(source, target):
            if Path(target) == self.out / "summary.md":
                raise OSError("injected disk error")
            return original(source, target)
        with patch.object(report.os, "replace", side_effect=fail):
            self.assertEqual(self.run_cli(), 4)
        self.assertTrue(self.out.exists())
        self.assertFalse((self.out / "artifacts.json").exists())
        self.assertIn(str(self.out), self.stderr.getvalue())
        self.assertTrue(list(self.out.glob("*.tmp")) or list(self.out.glob(".*.tmp")))
        self.assertEqual(recover_experiment(self.out).state, "incomplete")

    def test_source_root_overlap_is_rejected(self):
        self.add()
        self.add("skill")
        run = evaluated()
        self.out = Path(run.baseline.snapshot.root) / "summary"
        self.assertEqual(self.run_cli(), 2)

    def test_empty_schedule_records_and_exhaustion_are_explicit(self):
        self.assertEqual(self.run_cli(), 3)
        result = recover_experiment(self.out).report
        self.assertEqual(len(result.outcomes), 2)
        self.out = self.root / "second"
        self.add(completion_state="budget_exhausted", budget_exhausted=True)
        self.add("skill")
        self.assertEqual(self.run_cli(), 0)
        self.assertEqual(recover_experiment(self.out).report.arm_summaries[0].budget_exhausted_attempts, 1)

    def test_final_index_failure_never_recovers_complete(self):
        self.add()
        from lesscode_baseline import report
        original = report.os.replace
        def fail(source, target):
            if Path(target).name == "artifacts.json":
                raise OSError("index failure")
            return original(source, target)
        with patch.object(report.os, "replace", side_effect=fail):
            self.assertEqual(self.run_cli(), 4)
        self.assertEqual(recover_experiment(self.out).state, "incomplete")
