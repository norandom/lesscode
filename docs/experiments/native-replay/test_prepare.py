"""Research fixture contracts; no original collector or workload is executed."""
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
spec = importlib.util.spec_from_file_location("replay_prepare", HERE / "prepare.py")
replay = importlib.util.module_from_spec(spec)
spec.loader.exec_module(replay)

class ReplayInputs(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory(prefix="native-replay-")
        cls.output = Path(cls.temp.name) / "inputs"
        cls.receipt = replay.prepare_inputs(cls.output)

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def test_full_replacements_and_frozen_definitions(self):
        for name in ("ats", "npu"):
            frozen = json.loads((self.output / name / "frozen.json").read_text())
            self.assertTrue(frozen["baseline_identity"])
            self.assertIn("Python/LogicalStatements", frozen["definitions"])
            for arm in ("baseline", "candidate", "mutant"):
                self.assertEqual(self.receipt["trials"][name]["source_identities"][arm],
                    json.loads((self.output / name / (arm + "-identity.json")).read_text())["baseline_identity"])
        ownership = json.loads((self.output / "npu/candidate-evidence.json").read_text())["candidate_ownership"]
        paths = {s["path"] for values in ownership["replacements"].values() for s in values}
        self.assertIn("src/npu_rag/embedding/providers/checked_run.py", paths)
        self.assertEqual(len(paths), 3)
        self.assertEqual(len(list((self.output / "ats/candidate").glob("*.py"))), 1)

    def test_historical_records_cannot_be_admitted_as_new_runs(self):
        self.assertEqual(self.receipt["originals_before_sha256"], self.receipt["originals_after_sha256"])
        for name in ("ats", "npu"):
            historical = self.receipt["trials"][name]["historical"]
            self.assertEqual(historical["native_execution_provenance"], "unavailable")
            self.assertTrue(historical["current_artifact_sha256"])
            for arm in ("candidate", "mutant"):
                evidence = json.loads((self.output / name / (arm + "-evidence.json")).read_text())
                self.assertTrue(all(p["baseline"] is None and p["candidate"] is None for p in evidence["checks"].values()))
            self.assertGreater(historical["observations"]["mutant"]["mismatches"], 0)

    def test_native_missing_evidence_and_stale_binding(self):
        trial = self.output / "ats"
        args = [str(replay.NATIVE), "compare", "--frozen", str(trial / "frozen.json"),
                "--baseline", str(trial / "baseline"), "--candidate", str(trial / "candidate"),
                "--evidence", str(trial / "candidate-evidence.json"), "--out", str(trial / "missing-report")]
        result = subprocess.run(args, capture_output=True, text=True, timeout=60)
        self.assertEqual(result.returncode, 2, result.stderr)
        report = json.loads((trial / "missing-report/result.json").read_text())
        self.assertEqual(report["comparison"]["status"], "incomplete")
        source = trial / "baseline/ats_agent.py"
        original = source.read_bytes()
        try:
            source.write_bytes(original + b"\n# stale binding\n")
            args[-1] = str(trial / "stale-report")
            result = subprocess.run(args, capture_output=True, text=True, timeout=60)
            self.assertEqual(result.returncode, 3)
            self.assertIn("baseline", result.stderr)
        finally:
            source.write_bytes(original)

    def test_stale_check_definition_cannot_validate(self):
        trial = self.output / "ats"
        frozen = json.loads((trial / "frozen.json").read_text())
        frozen["manifest"]["checks"]["differential"]["definition_digest"] = "changed-definition"
        changed = trial / "changed-checks.json"
        changed.write_text(json.dumps(frozen))
        result = subprocess.run([str(replay.NATIVE), "compare", "--frozen", str(changed),
            "--baseline", str(trial / "baseline"), "--candidate", str(trial / "candidate"),
            "--evidence", str(trial / "candidate-evidence.json"), "--out", str(trial / "changed-report")],
            capture_output=True, text=True, timeout=60)
        self.assertEqual(result.returncode, 3, result.stderr)
        self.assertIn("check", result.stderr)

if __name__ == "__main__":
    unittest.main()

