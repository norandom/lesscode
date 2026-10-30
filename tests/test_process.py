import os
from pathlib import Path
import runpy
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

from lesscode_baseline.contracts import ContractError, ExecutionResult, Limits, ResolvedCommand
from lesscode_baseline.process import execute


class ProcessCollectionTests(unittest.TestCase):
    def test_collection_when_skip_decorator_returns_original_function(self):
        # skipUnless(True) returns its original function on the matching platform.
        # Exercise that collection behavior without running a foreign OS workload.
        with patch("unittest.skipUnless", side_effect=lambda condition, reason: lambda value: value):
            namespace = runpy.run_path(__file__, run_name="collection_probe")
        self.assertIn("WindowsProcessTests", namespace)


class ProcessTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.logs = self.root / "logs"

    def command(self, code, timeout=2, budget=4096, environment=None, cwd="."):
        return ResolvedCommand("probe", (str(Path(sys.executable).resolve()), "-c", code),
                               cwd, environment or {}, Limits(timeout, budget), sys.version.split()[0])

    def run_command(self, code, **kwargs):
        return execute(self.command(code, **kwargs), self.root, self.logs)

    def test_platform_containment(self):
        marker = self.root / "launched"
        result = self.run_command(f"open({str(marker)!r},'w').write('yes')")
        self.assertIsInstance(result, ExecutionResult)
        self.assertEqual(result.status, "completed")
        self.assertTrue(marker.exists())
        ExecutionResult.from_dict(result.to_dict())

    def test_invalid_inputs_never_launch(self):
        command = self.command("raise RuntimeError()")
        bad = ResolvedCommand(command.command_id, ("python",), ".", {}, command.limits, "3")
        with self.assertRaises(ContractError):
            execute(bad, self.root, self.logs)

    def test_environment_identity_excludes_secret_values(self):
        first = self.run_command("pass", environment={"TOKEN": "first-secret"})
        second = self.run_command("pass", environment={"TOKEN": "second-secret"})
        self.assertEqual(first.environment_hash, second.environment_hash)
        self.assertNotEqual(first.stdout_artifact, second.stdout_artifact)

    def test_log_persistence_failure_propagates(self):
        # Exercise the common error contract without running a foreign platform.
        with patch("lesscode_baseline.process.os", wraps=os) as platform_os:
            platform_os.name = "posix"
            with patch("lesscode_baseline.process._run_posix", side_effect=OSError("disk full")):
                with self.assertRaisesRegex(OSError, "disk full"):
                    self.run_command("pass")

    @unittest.skipUnless(os.name == "posix", "POSIX lifecycle")
    def test_success_at_exact_log_boundary(self):
        result = self.run_command("import os;os.write(1,b'abc');os.write(2,b'def')", budget=6)
        self.assertEqual((result.status, result.exit_code), ("completed", 0))
        self.assertEqual((self.logs / result.stdout_artifact).read_bytes(), b"abc")
        self.assertEqual((self.logs / result.stderr_artifact).read_bytes(), b"def")

    @unittest.skipUnless(os.name == "posix", "POSIX lifecycle; Windows helper is task 2.4")
    def test_outcomes_environment_cwd_and_literal_argv(self):
        (self.root / "sub").mkdir()
        result = self.run_command("import os,sys;print(os.getcwd());print(os.environ.get('DECLARED'));print(os.environ.get('UNDECLARED_TEST'));print('$HOME;echo shell');print('diagnostic',file=sys.stderr);sys.exit(7)",
                                  cwd="sub", environment={"DECLARED": "visible"})
        self.assertEqual((result.status, result.exit_code), ("completed", 7))
        output = (self.logs / result.stdout_artifact).read_text()
        self.assertIn(str(self.root / "sub"), output)
        self.assertIn("visible\nNone\n$HOME;echo shell", output)
        self.assertEqual((self.logs / result.stderr_artifact).read_text(), "diagnostic\n")
        self.assertGreater(result.elapsed_seconds, 0)

    @unittest.skipUnless(os.name == "posix", "POSIX lifecycle")
    def test_log_budget_is_combined_and_bounded(self):
        result = self.run_command("import os;\nwhile True:\n os.write(1,b'a'*3000);os.write(2,b'b'*3000)", budget=1000)
        self.assertEqual(result.status, "log_limit")
        self.assertEqual(sum((self.logs / p).stat().st_size for p in
                             (result.stdout_artifact, result.stderr_artifact)), 1000)

    @unittest.skipUnless(os.name == "posix", "POSIX lifecycle")
    def test_timeout_terminates_parent_and_child_and_retains_logs(self):
        code = "import os,subprocess,sys,time; child=subprocess.Popen([sys.executable,'-c',\"import time;time.sleep(.8);open('survivor','w').write('bad')\"]);print(os.getpid(),child.pid,flush=True);time.sleep(10)"
        result = self.run_command(code, timeout=.25)
        self.assertEqual(result.status, "timeout")
        self.assertLess(result.elapsed_seconds, 1)
        self.assertEqual(len((self.logs / result.stdout_artifact).read_text().split()), 2)
        time.sleep(.9)
        self.assertFalse((self.root / "survivor").exists())

    @unittest.skipUnless(os.name == "posix", "POSIX lifecycle")
    def test_missing_executable_preserves_diagnostic_without_retry(self):
        command = self.command("pass")
        command = ResolvedCommand("missing", (str(self.root / "missing"),), ".", {}, command.limits, "unknown")
        result = execute(command, self.root, self.logs)
        self.assertEqual((result.status, result.exit_code), ("error", None))
        self.assertIn("process_error", (self.logs / result.stderr_artifact).read_text())

    @unittest.skipUnless(os.name == "posix", "POSIX lifecycle")
    def test_leader_exit_cleans_up_background_child(self):
        result = self.run_command("import subprocess,sys;subprocess.Popen([sys.executable,'-c',\"import time;time.sleep(.8);open('orphan','w').write('bad')\"])")
        self.assertEqual((result.status, result.exit_code), ("completed", 0))
        self.assertLess(result.elapsed_seconds, 1)
        time.sleep(.9)
        self.assertFalse((self.root / "orphan").exists())


@unittest.skipUnless(os.name == "nt", "native Windows lifecycle")
class WindowsProcessTests(ProcessTests):
    # Run the same behavioral probes natively rather than pretending Windows is POSIX.
    test_success_at_exact_log_boundary = getattr(ProcessTests.test_success_at_exact_log_boundary, "__wrapped__", ProcessTests.test_success_at_exact_log_boundary)
    test_outcomes_environment_cwd_and_literal_argv = getattr(ProcessTests.test_outcomes_environment_cwd_and_literal_argv, "__wrapped__", ProcessTests.test_outcomes_environment_cwd_and_literal_argv)
    test_log_budget_is_combined_and_bounded = getattr(ProcessTests.test_log_budget_is_combined_and_bounded, "__wrapped__", ProcessTests.test_log_budget_is_combined_and_bounded)
    test_missing_executable_preserves_diagnostic_without_retry = getattr(ProcessTests.test_missing_executable_preserves_diagnostic_without_retry, "__wrapped__", ProcessTests.test_missing_executable_preserves_diagnostic_without_retry)
    test_leader_exit_cleans_up_background_child = getattr(ProcessTests.test_leader_exit_cleans_up_background_child, "__wrapped__", ProcessTests.test_leader_exit_cleans_up_background_child)

    def test_timeout_terminates_parent_and_child_and_retains_logs(self):
        # Windows venv startup can take longer than the POSIX probe's .25s.
        code = "import os,subprocess,sys,time;child=subprocess.Popen([sys.executable,'-c',\"import time;time.sleep(1.8);open('survivor','w').write('bad')\"]);print(os.getpid(),child.pid,flush=True);time.sleep(10)"
        result = self.run_command(code, timeout=1)
        self.assertEqual(result.status, "timeout")
        self.assertLess(result.elapsed_seconds, 1.8)
        self.assertEqual(len((self.logs / result.stdout_artifact).read_text().split()), 2)
        time.sleep(2)
        self.assertFalse((self.root / "survivor").exists())

    def test_containment_failures_never_execute_workload(self):
        for stage in ("_create_job", "_assign_job", "_resume"):
            with self.subTest(stage=stage):
                marker = self.root / stage
                with patch("lesscode_baseline.process_windows." + stage,
                           side_effect=OSError("containment failure")):
                    result = self.run_command(f"open({str(marker)!r},'w').write('bad')")
                self.assertEqual((result.status, result.exit_code), ("error", None))
                self.assertIn("containment failure", (self.logs / result.stderr_artifact).read_text())
                time.sleep(.05)
                self.assertFalse(marker.exists())

    def test_log_persistence_failure_cleans_up_workload(self):
        from lesscode_baseline.process_windows import run_windows
        code = "import os,time;os.write(1,b'x');time.sleep(.5);open('persistence-survivor','w').write('bad')"
        with (self.root / "stderr").open("wb") as stderr:
            class FailedOutput:
                def write(self, data):
                    raise OSError("disk full")
            with self.assertRaisesRegex(OSError, "disk full"):
                run_windows(self.command(code), self.root, FailedOutput(), stderr, time.monotonic())
        time.sleep(.6)
        self.assertFalse((self.root / "persistence-survivor").exists())

    def test_log_limit_terminates_child(self):
        code = "import subprocess,sys,os;subprocess.Popen([sys.executable,'-c',\"import time;time.sleep(.8);open('log-survivor','w').write('bad')\"]);\nwhile True: os.write(1,b'x'*4096)"
        result = self.run_command(code, budget=1000)
        self.assertEqual(result.status, "log_limit")
        time.sleep(.9)
        self.assertFalse((self.root / "log-survivor").exists())


if __name__ == "__main__":
    unittest.main()
