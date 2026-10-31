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
    def test_installed_console_keeps_relative_metadata_python_in_its_venv(self):
        uv = shutil.which("uv")
        self.assertIsNotNone(uv, "explicit analyzer setup requires uv")
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            environment = root / "analyzer"
            subprocess.run([uv, "venv", "--python", sys.executable, str(environment)], check=True, capture_output=True)
            python = environment / "bin/python"
            subprocess.run([uv, "pip", "install", "--python", str(python), "--require-hashes",
                            "--only-binary=:all:", "-r", str(ROOT / "requirements-tools.lock")], check=True, capture_output=True)
            wheels = root / "wheel"
            subprocess.run([uv, "build", "--python", sys.executable, "--wheel", "--out-dir", str(wheels), str(ROOT)],
                           check=True, capture_output=True)
            wheel, = wheels.glob("*.whl")
            subprocess.run([uv, "pip", "install", "--python", str(python), "--no-deps", str(wheel)],
                           check=True, capture_output=True)
            outside = root / "outside"
            outside.mkdir()
            source = ROOT / "tests/fixtures/real/boltons_mathutils/source"
            clean = dict(os.environ)
            clean.pop("PYTHONPATH", None)
            for index, (metadata, value) in enumerate(((root, "analyzer/bin/python"),
                                                      (root / "metadata", "../analyzer/bin/python"))):
                metadata.mkdir(exist_ok=True)
                tools = metadata / "entrypoints.json"
                # Relative entries are anchored to metadata, including parents.
                tools.write_text(json.dumps({"python": value}))
                plan = root / f"plan-{index}.json"
                result = subprocess.run([str(python.with_name("lesscode-baseline")), "plan", "--profile",
                    str(ROOT / "benchmarks/profiles/python.json"), "--baseline", str(source), "--candidate", str(source),
                    "--tool-paths", str(tools), "--out", str(plan)], cwd=outside, env=clean, capture_output=True, text=True)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                bindings = {binding["name"]: binding for binding in json.loads(plan.read_bytes())["tools"]}
                self.assertEqual(bindings["lizard"]["status"], "available", bindings["lizard"]["reason"])
                self.assertEqual(bindings["python"]["executable"], str(python))

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
