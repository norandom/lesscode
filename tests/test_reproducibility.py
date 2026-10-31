"""Resume verifies durable evidence without repairing inputs or issuing work."""

from dataclasses import replace
from contextlib import chdir, redirect_stdout
import hashlib
import io
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from lesscode_baseline import contracts as c
from lesscode_baseline.config import prepare
from lesscode_baseline.experiments import aggregate
from lesscode_baseline.report import RunStore, persist_experiment
from tests.test_experiment_contracts import manifest
from tests.test_experiments import evaluated, producer
from tools import validate_pilot as pilot


class PilotResumeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.output = self.root / "evidence"
        self.inputs = self.output / "inputs"
        self.inputs.mkdir(parents=True)
        self.source = self.inputs / "candidate"
        self.source.mkdir()
        (self.source / "subject.py").write_text("value = 1\n")
        for name in ("tests", "generated", "vendor"):
            (self.source / name).mkdir()
            (self.source / name / "added.py").write_text("# retained input\n")
        self.evaluator = self.inputs / "evaluator"
        (self.evaluator / "checks").mkdir(parents=True)
        (self.evaluator / "checks/oracle.py").write_text("raise AssertionError('must never execute')\n")
        data = json.loads((Path(__file__).resolve().parents[1] / "benchmarks/profiles/python.json").read_bytes())
        data["checks"] = data["checks"][:1]
        data["checks"][0]["command"]["argv"] = ["python", "checks/oracle.py"]
        data["evaluator_resources"] = ["profile.json", "checks/oracle.py"]
        self.profile = self.evaluator / "profile.json"
        pilot.write_json(self.profile, data)
        self.tool_paths = self.root / "tools.json"
        self.tool_paths.write_text("{}\n")
        self.prepared = prepare(self.profile, self.source, self.source, self.output / "control-plan.json", tool_paths={})
        self.assertIsInstance(self.prepared, c.PreparedPlan)
        self.plan_hash = hashlib.sha256(c.dumps(self.prepared).encode("utf-8")).hexdigest()
        (self.output / "control-plan.json").write_text(c.dumps(self.prepared), encoding="utf-8")
        report = evaluated()
        obs = replace(report.baseline, snapshot=self.prepared.plan.baseline, checks=())
        self.report = replace(report, plan_hash=self.plan_hash, baseline=obs, candidate=obs)
        store = RunStore.create(self.output / "runs/control", self.plan_hash,
                                run_id=self.report.run_id, prepared_plan=self.prepared)
        store.finalize(self.report)
        experiment = manifest()
        experiment["corpus"][0]["content_hash"] = obs.snapshot.content_hash
        experiment = c.ExperimentManifest.from_dict(experiment)
        record = producer(starting_source_hash=obs.snapshot.content_hash, run_artifact="control")
        pilot.write_json(self.inputs / "experiment.json", experiment.to_dict())
        pilot.write_json(self.inputs / "control.json", record.to_dict())
        persist_experiment(aggregate(experiment, (record,), artifact_root=self.output / "runs"), self.output / "summary")
        pilot.write_json(self.output / "validation.json", {
            "status": "passed", "tool_paths": str(self.tool_paths),
            "tool_paths_sha256": hashlib.sha256(self.tool_paths.read_bytes()).hexdigest(),
            "source_fingerprints": {str(self.source): pilot.fingerprints(self.source)},
            "checkout_fingerprints": {}, "run_decisions": {"control": "accepted"}})

    def resume(self):
        with patch("subprocess.run", side_effect=AssertionError("resume must issue no workloads")):
            return pilot.resume_completed(self.output, self.tool_paths)

    def assert_rejected_without_writes(self):
        before = pilot.fingerprints(self.root)
        with self.assertRaises((AssertionError, c.ContractError)):
            self.resume()
        self.assertEqual(before, pilot.fingerprints(self.root))

    def test_repeated_resume_preserves_existing_directories_and_completed_summary(self):
        before = pilot.fingerprints(self.root)
        self.resume()
        self.resume()
        self.assertEqual(before, pilot.fingerprints(self.root))

    def test_main_repeated_resume_is_read_only_before_any_setup(self):
        before = pilot.fingerprints(self.root)
        argv = ["validate_pilot.py", "--checkout", str(Path(__file__).resolve().parents[1]),
                "--tool-paths", str(self.tool_paths), "--out", str(self.output), "--resume"]
        with chdir(self.root), patch.dict(os.environ), patch("sys.argv", argv), \
             patch("lesscode_baseline.__file__", str(self.root / "installed/lesscode_baseline/__init__.py")), \
             patch("subprocess.run", side_effect=AssertionError("resume must issue no workloads")), \
             redirect_stdout(io.StringIO()):
            os.environ.pop("PYTHONPATH", None)
            pilot.main()
            pilot.main()
        self.assertEqual(before, pilot.fingerprints(self.root))

    def test_tampered_candidate_is_preserved_and_rejected(self):
        (self.source / "subject.py").write_text("developer changes must survive\n")
        self.assert_rejected_without_writes()

    def test_tampered_evaluator_is_preserved_and_rejected(self):
        (self.evaluator / "checks/oracle.py").write_text("changed oracle\n")
        self.assert_rejected_without_writes()

    def test_changed_config_is_rejected_without_writes(self):
        self.tool_paths.write_text('{"python":"changed"}\n')
        self.assert_rejected_without_writes()

    def test_outer_plan_must_match_indexed_frozen_plan(self):
        changed = replace(self.prepared, plan=replace(self.prepared.plan,
            protection=c.Protection("external_isolation_claimed", "changed declaration")))
        (self.output / "control-plan.json").write_text(c.dumps(changed), encoding="utf-8")
        self.assert_rejected_without_writes()

    def test_report_hash_must_bind_canonical_outer_prepared_plan(self):
        # Self-consistent report/index recovery alone is insufficient: this
        # deliberately finalized run has the wrong canonical plan binding.
        from shutil import rmtree
        run = self.output / "runs/control"
        rmtree(run)
        report = replace(self.report, plan_hash="a" * 64)
        store = RunStore.create(run, report.plan_hash, run_id=report.run_id, prepared_plan=self.prepared)
        store.finalize(report)
        self.assert_rejected_without_writes()

    def test_tampered_completed_summary_is_rejected_without_writes(self):
        (self.output / "summary/summary.md").write_text("changed summary\n")
        self.assert_rejected_without_writes()


if __name__ == "__main__":
    unittest.main()
