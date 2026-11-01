"""Integrated native NPU replay admission fixtures."""
import importlib.util
import os
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('npu_replay', Path(__file__).with_name('replay.py'))
replay = importlib.util.module_from_spec(spec)
spec.loader.exec_module(replay)

class ReplayTests(unittest.TestCase):
    def test_fresh_run_and_manipulations(self):
        with tempfile.TemporaryDirectory(prefix='npu63-') as directory:
            out = Path(os.environ['NPU_REPLAY_VALIDATION_OUT']).resolve() if os.environ.get('NPU_REPLAY_VALIDATION_OUT') else Path(directory) / 'run'
            if not os.environ.get('NPU_REPLAY_VALIDATION_OUT'):
                replay.run(out, replay.INPUTS, replay.PYTHON)
            results = replay.read(out / 'outcomes.json')
            receipt = replay.read(out / 'receipt.json')
            self.assertEqual(receipt['runner_sha256'], replay.sha(Path(replay.__file__)))
            self.assertEqual(receipt['originals_before_sha256'], receipt['originals_after_sha256'])
            self.assertEqual(receipt['prepared_inputs_before_sha256'], receipt['prepared_inputs_after_sha256'])
            self.assertEqual(results['candidate'], 'accepted')
            self.assertEqual(results['mutant'], 'rejected-behavior')
            for name in ('moved-helper', 'scope-narrowing', 'classification', 'missing-check', 'stale-evidence'):
                self.assertIn(results[name], ('invalid', 'incomplete', 'rejected-behavior'), name)
                if results[name] == 'invalid':
                    execution = replay.read(out / (name + '-native-execution.json'))
                    self.assertEqual(execution['code'], 3)
                    self.assertIn('source identity mismatch' if name == 'stale-evidence' else 'checked_run.py', execution['stderr'])
            for arm, mismatches in (('baseline', 0), ('candidate', 0), ('mutant', 572)):
                raw = replay.read(out / (arm + '-raw/differential.json'))
                self.assertEqual((raw['cases'], raw['mismatches']), (3168, mismatches))
            for arm in ('baseline', 'candidate'):
                self.assertEqual(replay.read(out / (arm + '-raw/native-tests.json'))['tests'], 339)
                self.assertEqual(replay.read(out / (arm + '-raw/types.json'))['code'], 0)

    def test_unknown_call_contract_stops_review(self):
        with self.assertRaisesRegex(ValueError, 'unreviewed'):
            replay.contract_for('mysterious.operation')

if __name__ == '__main__':
    unittest.main()





