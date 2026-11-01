"""Task-local regression guard for the only permitted source edit."""
import importlib.util
from pathlib import Path
import unittest

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('typescript_replay', HERE / 'replay.py')
replay = importlib.util.module_from_spec(spec)
spec.loader.exec_module(replay)


class CandidateBoundary(unittest.TestCase):
    def test_replaces_only_the_complete_terminal_helper(self):
        original = 'export const retained = 1\n' + replay.ORIGINAL
        actual = replay.replace_helper(original)
        self.assertEqual(actual, 'export const retained = 1\n' + replay.REPLACEMENT)
        self.assertNotIn('needsSeparator', actual)

    def test_drift_and_additional_callers_fail_closed(self):
        for text in (replay.ORIGINAL.replace('"unknown"', '"changed"'),
                     replay.ORIGINAL + '\nexport const appended = 2\n'):
            with self.subTest(text=text), self.assertRaises(ValueError):
                replay.replace_helper(text)


if __name__ == '__main__':
    unittest.main()
