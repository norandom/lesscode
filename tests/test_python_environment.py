"""Real POSIX analyzer venvs retain package context and immutable bindings."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

from lesscode_baseline.config import prepare, verify_integrity
from lesscode_baseline.contracts import PreparedPlan


ROOT = Path(__file__).resolve().parents[1]


@unittest.skipUnless(os.name == "posix", "native POSIX interpreter symlink binding")
class PythonEnvironmentTests(unittest.TestCase):
    def test_locked_lizard_context_and_environment_retargeting(self):
        uv = shutil.which("uv")
        self.assertIsNotNone(uv, "explicit analyzer setup requires uv")
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            environment = root / "analyzer"
            subprocess.run([uv, "venv", "--python", sys.executable, str(environment)], check=True, capture_output=True)
            python = environment / "bin/python"
            subprocess.run([uv, "pip", "install", "--python", str(python), "--require-hashes",
                            "--only-binary=:all:", "-r", str(ROOT / "requirements-tools.lock")],
                           check=True, capture_output=True)
            evaluator = root / "evaluator"
            evaluator.mkdir()
            (evaluator / "runner.py").write_text("pass\n")
            profile = json.loads((ROOT / "benchmarks/profiles/python.json").read_text())
            profile["evaluator_resources"] = ["profile.json", "runner.py"]
            for check in profile["checks"]:
                check["command"]["argv"][1] = "runner.py"
            profile_path = evaluator / "profile.json"
            profile_path.write_text(json.dumps(profile))
            source = root / "source"
            source.mkdir()
            (source / "module.py").write_text("value = 1\n")
            prepared = prepare(profile_path, source, source, root / "plan.json", tool_paths={"python": str(python)})
            self.assertIsInstance(prepared, PreparedPlan)
            bindings = {binding.name: binding for binding in prepared.tools}
            self.assertEqual(bindings["lizard"].status, "available", bindings["lizard"].reason)
            self.assertEqual(bindings["python"].executable, str(python))
            self.assertEqual(verify_integrity(prepared).status, "unchanged")
            original = os.readlink(python)
            alternate = root / "same-bytes-other-runtime"
            shutil.copyfile(python.resolve(), alternate)
            python.unlink()
            python.symlink_to(alternate)
            self.assertEqual(verify_integrity(prepared).status, "changed", "retargeted same-byte runtime was trusted")
            python.unlink()
            python.symlink_to(original)
            self.assertEqual(verify_integrity(prepared).status, "unchanged")
            config = environment / "pyvenv.cfg"
            config.write_bytes(config.read_bytes() + b"\n# modified environment\n")
            self.assertEqual(verify_integrity(prepared).status, "changed", "changed venv context was trusted")


if __name__ == "__main__":
    unittest.main()
