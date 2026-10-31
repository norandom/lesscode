"""Installation contract checks; run real analyzers with tools/probe_toolchain.py."""

import json
import hashlib
from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[1]


class ToolchainLockTests(unittest.TestCase):
    def test_python_analyzers_are_exact_and_hashed(self):
        lock = (ROOT / "requirements-tools.lock").read_text()
        requirements = re.findall(r"^([\w-]+)==([^\s]+)\s+--hash=sha256:([a-f0-9]{64})$", lock, re.M)
        self.assertTrue(requirements)
        self.assertEqual(dict((name, version) for name, version, _ in requirements)["lizard"], "1.17.31")
        self.assertTrue(all(line.startswith("#") or not line.strip() or re.fullmatch(r"[\w-]+==[^\s]+\s+--hash=sha256:[a-f0-9]{64}", line) for line in lock.splitlines()))

    def test_npm_transitive_packages_have_integrity(self):
        manifest = json.loads((ROOT / "tools/package.json").read_text())
        lock = json.loads((ROOT / "tools/package-lock.json").read_text())
        self.assertEqual(manifest["dependencies"], {"jscpd": "4.0.5"})
        self.assertEqual(lock["packages"]["node_modules/jscpd"]["version"], "4.0.5")
        self.assertGreater(len(lock["packages"]), 2)
        for path, package in lock["packages"].items():
            if path:
                self.assertRegex(package["integrity"], r"^sha512-")
                self.assertTrue(package["resolved"].startswith("https://registry.npmjs.org/"))

    def test_dependency_notices_cover_locked_packages(self):
        inventory = json.loads((ROOT / "tools/licenses.json").read_text())
        lock = json.loads((ROOT / "tools/package-lock.json").read_text())
        self.assertEqual(set(inventory["npm"]), set(lock["packages"]) - {""})
        for entry in [*inventory["npm"].values(), *inventory["python"].values()]:
            self.assertTrue(entry["license"])
            self.assertTrue(entry["notices"])
            for path in entry["notices"]:
                self.assertTrue((ROOT / "tools" / path).is_file(), path)
        self.assertIn("Apache", inventory["python"]["lizard"]["source_notice"])

    def test_explicit_setup_and_probe_are_separate(self):
        self.assertTrue((ROOT / "tools/setup_toolchain.py").is_file())
        self.assertTrue((ROOT / "tools/probe_toolchain.py").is_file())

    def test_retained_notice_names_bind_exact_distribution_bytes(self):
        notices = list((ROOT / "tools/notices").glob("*.txt"))
        self.assertTrue(notices)
        for path in notices:
            self.assertEqual(path.stem, hashlib.sha256(path.read_bytes()).hexdigest(), path.name)

    def test_notice_bytes_are_protected_from_git_line_conversion(self):
        import subprocess
        paths = sorted((ROOT / "tools/notices").glob("*.txt"))
        result = subprocess.run(["git", "check-attr", "text", "--", *[
            p.relative_to(ROOT).as_posix() for p in paths]], cwd=ROOT,
            capture_output=True, text=True, check=True)
        self.assertEqual(len(result.stdout.splitlines()), len(paths))
        self.assertTrue(all(line.endswith(": text: unset") for line in result.stdout.splitlines()), result.stdout)


if __name__ == "__main__":
    unittest.main()
