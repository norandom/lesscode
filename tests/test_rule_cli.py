"""Rules package discovery and local CLI integration contracts."""

import importlib
from pathlib import Path
import unittest


class RulesPackageTests(unittest.TestCase):
    def test_rules_is_an_importable_regular_package(self):
        rules_package = importlib.import_module("lesscode_baseline.rules")
        self.assertIsNotNone(rules_package.__file__)
        self.assertEqual(Path(rules_package.__file__).name, "__init__.py")
        self.assertIsNotNone(rules_package.__spec__.submodule_search_locations)


if __name__ == "__main__":
    unittest.main()
