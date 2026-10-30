"""Exercise the module command as a user starts it."""

import os
from pathlib import Path
import subprocess
import sys
import unittest

from lesscode_baseline import __version__


class CliStartupTests(unittest.TestCase):
    def invoke(self, *args: str) -> subprocess.CompletedProcess[str]:
        environment = os.environ.copy()
        environment["PYTHONPATH"] = str(Path(__file__).resolve().parents[1] / "src")
        # Startup must not need executable discovery, an analyzer or an account.
        environment["PATH"] = ""
        for key in list(environment):
            if key.endswith("_API_KEY"):
                del environment[key]
        return subprocess.run(
            [sys.executable, "-m", "lesscode_baseline", *args],
            env=environment,
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )

    def test_help_is_available_without_external_tools(self) -> None:
        result = self.invoke("--help")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("usage: lesscode-baseline", result.stdout)
        self.assertIn("--version", result.stdout)
        self.assertEqual(result.stderr, "")

    def test_version_matches_package(self) -> None:
        result = self.invoke("--version")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), f"lesscode-baseline {__version__}")
        self.assertEqual(result.stderr, "")

    def test_no_arguments_prints_usage(self) -> None:
        result = self.invoke()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("usage: lesscode-baseline", result.stdout)

    def test_unknown_arguments_fail(self) -> None:
        result = self.invoke("--unknown")
        self.assertEqual(result.returncode, 2)
        self.assertIn("unrecognized arguments: --unknown", result.stderr)


if __name__ == "__main__":
    unittest.main()
