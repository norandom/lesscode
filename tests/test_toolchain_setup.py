"""Explicit setup must never clear an existing or linked tool root."""

import importlib.util
import os
from pathlib import Path
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]


def setup_module():
    spec = importlib.util.spec_from_file_location("setup_toolchain", ROOT / "tools/setup_toolchain.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class SetupRootTests(unittest.TestCase):
    @unittest.skipUnless(os.name == "posix", "native POSIX tool-root symlink protection")
    def test_linked_parent_is_rejected_and_original_is_preserved(self):
        setup = setup_module()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            actual = root / "actual"
            actual.mkdir()
            linked = root / "linked"
            linked.symlink_to(actual, target_is_directory=True)
            with self.assertRaisesRegex(ValueError, "symlink|reparse"):
                setup.fresh_tool_root(linked / "tools")
            self.assertFalse((actual / "tools").exists())

    def test_existing_tool_root_is_rejected_without_changing_contents(self):
        setup = setup_module()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            marker = root / "keep.txt"
            marker.write_bytes(b"keep")
            with self.assertRaisesRegex(ValueError, "new|fresh|exist"):
                setup.fresh_tool_root(root)
            self.assertEqual(marker.read_bytes(), b"keep")

    def test_fresh_tool_root_is_an_absolute_owned_directory(self):
        setup = setup_module()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve() / "tools"
            self.assertEqual(setup.fresh_tool_root(root), root)
            self.assertTrue(root.is_dir())


if __name__ == "__main__":
    unittest.main()
