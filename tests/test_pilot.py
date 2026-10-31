"""The licensed pilot is pinned, independently checkable, and contract-valid."""

import hashlib
from dataclasses import replace
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
import xml.etree.ElementTree as ET

from lesscode_baseline import contracts as c
from lesscode_baseline.snapshot import capture_scope
from lesscode_baseline.experiments import reconcile

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests/fixtures/real/boltons_mathutils"
PROFILE = ROOT / "benchmarks/profiles/python.json"


class PilotTests(unittest.TestCase):
    def test_manifest_profile_and_upstream_byte_identity(self):
        manifest = c.loads(c.ExperimentManifest, (ROOT / "benchmarks/corpus.json").read_text())
        profile = c.loads(c.Profile, PROFILE.read_text())
        provenance = json.loads((FIXTURE / "provenance.json").read_text())
        entry, = manifest.corpus
        self.assertRegex(entry.revision, r"^[0-9a-f]{40}$")
        self.assertEqual(entry.spdx_license, "BSD-3-Clause")
        self.assertEqual(entry.generation_provenance.status, "unknown")
        snapshot = capture_scope(FIXTURE / "source", profile.scope)
        self.assertIsInstance(snapshot, c.Snapshot)
        self.assertEqual(entry.content_hash, snapshot.content_hash)
        self.assertNotEqual(entry.content_hash, provenance["files"]["source/mathutils.py"]["sha256"])
        for name, identity in provenance["files"].items():
            content = (FIXTURE / name).read_bytes()
            self.assertEqual(hashlib.sha256(content).hexdigest(), identity["sha256"])
            self.assertEqual(len(content), identity["bytes"])
            self.assertIn(entry.revision, identity["upstream_url"])
        self.assertEqual((ROOT / entry.license_artifact).resolve(), (FIXTURE / "LICENSE").resolve())
        self.assertIn("Redistribution and use", (FIXTURE / "LICENSE").read_text())
        for resource in profile.evaluator_resources:
            self.assertTrue((PROFILE.parent / resource).is_file(), resource)
        self.assertEqual({t.name: t.version for t in profile.tools}, {"lizard": "1.17.31", "jscpd": "4.0.5"})
        self.assertEqual(manifest.controls.prompt_identity,
                         hashlib.sha256((PROFILE.parent / "pilot/prompt.txt").read_bytes()).hexdigest())
        self.assertGreaterEqual(len(manifest.arms), 2)
        self.assertEqual({t.arm_id for t in manifest.planned_trials}, {a.arm_id for a in manifest.arms})

    def test_real_scoped_source_matches_manifest_without_false_confounding(self):
        manifest = c.loads(c.ExperimentManifest, (ROOT / "benchmarks/corpus.json").read_bytes())
        profile = c.loads(c.Profile, PROFILE.read_bytes())
        source_hash = capture_scope(FIXTURE / "source", profile.scope).content_hash
        def unavailable(unit):
            return {"status": "unavailable", "value": None, "unit": unit,
                "source": None, "observed_at": None, "reason": "no model or agent usage measured"}
        telemetry = c.Telemetry.from_dict({"token_usage": unavailable("tokens"), "provider_cost": unavailable("USD"),
            "agent_elapsed": unavailable("seconds"), "runtime_benchmarks": []})
        arms = {arm.arm_id: arm for arm in manifest.arms}
        records = tuple(c.TrialRecord(1, trial.trial_id, trial.arm_id, trial.corpus_id,
            trial.task_id, trial.repetition, source_hash, None,
            replace(manifest.controls, interventions=arms[trial.arm_id].interventions), order,
            0, "incomplete", False, telemetry)
            for order, trial in enumerate(manifest.planned_trials, 1))
        result = reconcile(manifest, records)
        self.assertTrue(all(not trial.confounded_differences for trial in result.planned_trials))
        self.assertTrue(all(pair.comparable for pair in result.pairings))
        # Raw upstream file SHA-256 remains a distinct, invalid starting identity.
        raw_hash = hashlib.sha256((FIXTURE / "source/mathutils.py").read_bytes()).hexdigest()
        wrong = reconcile(manifest, (replace(records[0], starting_source_hash=raw_hash), *records[1:]))
        self.assertTrue(wrong.planned_trials[0].confounded_differences)

    def run_checks(self, source, output):
        profile = c.loads(c.Profile, PROFILE.read_text())
        results = []
        for check in profile.checks:
            argv = list(check.command.argv)
            argv[0] = sys.executable
            argv[1] = str((PROFILE.parent / argv[1]).resolve())
            # The disposable check root is the command cwd and report root.
            report = output / check.result_path
            argv[argv.index("--output") + 1] = str(report)
            result = subprocess.run(argv, cwd=source, capture_output=True, text=True,
                                    env={**os.environ, **check.command.environment},
                                    timeout=check.command.limits.timeout_seconds)
            suite = ET.parse(report).getroot()
            cases = {item.attrib["name"]: item for item in suite.findall("testcase")}
            self.assertEqual(set(cases), set(check.expected_ids))
            self.assertEqual(int(suite.attrib["tests"]), len(cases))
            results.append((check, result, cases))
        return results

    def test_real_baseline_passes_native_and_independent_checks(self):
        with tempfile.TemporaryDirectory() as tmp:
            workspace = Path(tmp)
            source = workspace / "source"
            shutil.copytree(FIXTURE / "source", source)
            for check, result, cases in self.run_checks(source, workspace):
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertTrue(all(case.find("failure") is None and case.find("error") is None
                                    for case in cases.values()), check.check_id)

    def test_independent_checks_reject_changed_behavior_with_native_examples_deleted(self):
        with tempfile.TemporaryDirectory() as tmp:
            workspace = Path(tmp)
            source = workspace / "source"
            source.mkdir()
            (source / "mathutils.py").write_text("def clamp(x, lower=0, upper=1): return x\n")
            for check, result, cases in self.run_checks(source, workspace):
                self.assertNotEqual(result.returncode, 0, check.check_id)
                self.assertTrue(any(case.find("failure") is not None or case.find("error") is not None
                                    for case in cases.values()), check.check_id)


if __name__ == "__main__":
    unittest.main()
