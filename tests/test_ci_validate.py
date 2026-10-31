"""CI orchestration contracts independent of a hosted runner."""

import importlib.util
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
import subprocess
import venv


ROOT = Path(__file__).resolve().parents[1]


def load_helper():
    spec = importlib.util.spec_from_file_location("ci_validate", ROOT / "tools/ci_validate.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class CIHelperTests(unittest.TestCase):
    @unittest.skipUnless(os.name == "posix", "native POSIX venv symlink context")
    def test_real_probe_invokes_python_in_its_virtual_environment(self):
        spec = importlib.util.spec_from_file_location("probe_toolchain", ROOT / "tools/probe_toolchain.py")
        probe = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(probe)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            environment = root / "python"
            venv.EnvBuilder(with_pip=False, symlinks=True).create(environment)
            python = environment / "bin/python"
            node = root / "node"
            entry = root / "jscpd"
            node.touch()
            entry.touch()
            (root / "entrypoints.json").write_text(json.dumps({
                "python": str(python), "node": str(node), "jscpd_entry": str(entry)}))
            class CheckedContext(Exception):
                pass
            def check_context(argv, **kwargs):
                prefix = subprocess.check_output([argv[0], "-c", "import sys;print(sys.prefix)"], text=True).strip()
                self.assertEqual(Path(prefix).resolve(), environment.resolve())
                raise CheckedContext()
            with patch.object(probe, "run", side_effect=check_context):
                with self.assertRaises(CheckedContext):
                    probe.probe(root)

    def test_discovers_windows_and_posix_npm_javascript_without_shell_wrapper(self):
        helper = load_helper()
        for layout in ("windows", "posix"):
            with self.subTest(layout=layout), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                node = root / ("node.exe" if layout == "windows" else "bin/node")
                node.parent.mkdir(parents=True, exist_ok=True)
                node.touch()
                npm = (node.parent / "node_modules/npm/bin/npm-cli.js" if layout == "windows"
                       else node.parent.parent / "lib/node_modules/npm/bin/npm-cli.js")
                npm.parent.mkdir(parents=True)
                npm.touch()
                self.assertEqual(helper.npm_entry(node), npm.resolve())
                npm.unlink()
                with self.assertRaises(FileNotFoundError):
                    helper.npm_entry(node)

    def test_venv_interpreter_path_is_not_dereferenced(self):
        helper = load_helper()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            interpreter = helper.venv_python(root)
            self.assertEqual(interpreter, root / ("Scripts/python.exe" if os.name == "nt" else "bin/python"))
            if os.name == "posix":
                interpreter.parent.mkdir()
                interpreter.symlink_to(sys.executable)
                self.assertNotEqual(interpreter, interpreter.resolve())
                self.assertEqual(helper.venv_python(root), interpreter)

    def test_failure_keeps_command_output_and_later_evidence(self):
        helper = load_helper()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            evidence = helper.Evidence(root)
            self.assertFalse(evidence.run("failed", [sys.executable, "-c",
                "import sys; print('out'); print('err',file=sys.stderr); sys.exit(7)"], cwd=root))
            self.assertTrue(evidence.run("later", [sys.executable, "-c", "print('later')"], cwd=root))
            self.assertIn(b"out", (root / "failed.stdout.txt").read_bytes())
            self.assertIn(b"err", (root / "failed.stderr.txt").read_bytes())
            summary = json.loads((root / "commands.json").read_text())
            self.assertEqual([x["exit_code"] for x in summary], [7, 0])
            self.assertEqual(summary[0]["cwd"], str(root))


if __name__ == "__main__":
    unittest.main()
