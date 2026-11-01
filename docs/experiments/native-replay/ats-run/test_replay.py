"""Contract tests for the isolated ATS replay adapter."""
import copy
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("replay", Path(__file__).with_name("replay.py"))
replay = importlib.util.module_from_spec(spec)
spec.loader.exec_module(replay)

class ReplayTests(unittest.TestCase):
    def test_mutant_is_detected_with_frozen_observations(self):
        original = replay.extract(replay.ATS / "source.py.txt", "baseline")
        mutant = replay.extract_bytes(replay.prepare.ats_mutant_module(), "mutant")
        result = replay.observe(mutant, original)
        self.assertEqual((result["cases"], result["mismatches"]), (8820, 2251))
        self.assertTrue(result["dispatch_preserved"])

    def test_full_module_extraction_cannot_silently_drift(self):
        raw = (replay.ATS / "candidate-module.py.txt").read_bytes()
        with self.assertRaisesRegex(ValueError, "frozen fragment"):
            replay.extract_bytes(raw.replace(b'OVERALL ATS SCORE', b'WRONG SCORE', 1), "candidate")

    def test_summary_binds_raw_observation_and_source_trial(self):
        raw = {"cases": 8820, "mismatches": 7}
        result = replay.check_result(raw, "source-A", "trial-B", "check-C", "raw.json", "hash-D", "differential")
        self.assertEqual(result["status"], "failed")
        self.assertEqual(result["summary"], {"kind": "case_counts", "expected": 8820, "observed": 8820, "mismatches": 7})
        self.assertEqual(result["source_identity"], "source-A")
        self.assertEqual(result["frozen_trial_digest"], "trial-B")
        self.assertEqual(result["artifacts"]["raw"]["sha256"], "hash-D")

if __name__ == "__main__":
    unittest.main()
