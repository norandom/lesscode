"""Public command contracts and local durable plan/run integration."""

from contextlib import redirect_stdout, redirect_stderr
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from lesscode_baseline.cli import main
from lesscode_baseline.contracts import PreparedPlan, loads
from lesscode_baseline import __version__

ROOT = Path(__file__).resolve().parents[1]
PILOT = ROOT / 'tests/fixtures/real/boltons_mathutils/source'


class CliTests(unittest.TestCase):
    def test_tool_metadata_rejects_invalid_json_types_and_keeps_other_path_rules(self):
        from lesscode_baseline.cli import _tool_paths
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            metadata = root / "metadata"
            metadata.mkdir()
            path = metadata / "tools.json"
            for invalid in ([], None, {"python": None}, {"python": 123}, {"python": ["python"]}):
                path.write_text(json.dumps(invalid))
                with self.subTest(document=invalid), self.assertRaises(ValueError):
                    _tool_paths(path)
            path.write_text(json.dumps({"node": "../node", "jscpd_entry": "../tools/jscpd.js"}))
            self.assertEqual(_tool_paths(path), {"node": str((root / "node").resolve()),
                                                "jscpd_entry": str((root / "tools/jscpd.js").resolve())})

    def invoke(self, argv):
        out, err = io.StringIO(), io.StringIO()
        with redirect_stdout(out), redirect_stderr(err):
            result = main(argv)
        return result, out.getvalue(), err.getvalue()

    def test_durable_plan_and_real_missing_tools_run(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            tools = root / 'tools.json'
            tools.write_text('{}')
            plan = root / 'plan.json'
            args = ['plan', '--profile', str(ROOT / 'benchmarks/profiles/python.json'),
                    '--baseline', str(PILOT), '--candidate', str(PILOT),
                    '--tool-paths', str(tools), '--out', str(plan)]
            code, out, err = self.invoke(args)
            self.assertEqual(code, 0, err)
            self.assertIn('baseline', out)
            self.assertIsInstance(loads(PreparedPlan, plan.read_bytes()), PreparedPlan)
            self.assertEqual(self.invoke(args)[0], 2)
            run = root / 'run'
            code, out, err = self.invoke(['run', '--plan', str(plan), '--out', str(run)])
            self.assertEqual(code, 3, err)
            self.assertIn('baseline_ineligible', out)
            self.assertTrue((run / 'summary.md').is_file())
            self.assertTrue((run / 'report.json').is_file())

    def test_invalid_document_and_cli_arguments(self):
        with tempfile.TemporaryDirectory() as tmp:
            plan = Path(tmp) / 'bad.json'
            plan.write_text('{}')
            self.assertEqual(self.invoke(['run', '--plan', str(plan), '--out', str(Path(tmp)/'run')])[0], 2)
            self.assertEqual(self.invoke(['run'])[0], 2)

    def test_bundled_locks_match_authoritative_sources(self):
        package = ROOT / 'src/lesscode_baseline/data'
        for logical in ('requirements-tools.lock', 'tools/package-lock.json'):
            self.assertEqual((ROOT / logical).read_bytes(), (package / logical).read_bytes())

    def test_decision_mapping(self):
        from lesscode_baseline.cli import decision_exit_code
        for decision, expected in [('accepted', 0), ('rejected', 1), ('invalid', 2),
                                   ('incomplete', 3), ('baseline_ineligible', 3)]:
            self.assertEqual(decision_exit_code(decision), expected)

    def test_commands_flushed_before_execution_and_surviving_storage_failure(self):
        from lesscode_baseline.config import prepare
        from lesscode_baseline.contracts import dumps
        from lesscode_baseline.report import PersistenceError
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            plan = root / 'plan.json'
            prepared = prepare(ROOT / 'benchmarks/profiles/python.json', PILOT, PILOT,
                               plan, tool_paths={})
            plan.write_text(dumps(prepared))
            output = root / 'run'
            class Stream(io.StringIO):
                flushed = False
                def flush(self):
                    self.flushed = True
            stream, err = Stream(), io.StringIO()
            def fail(*args, **kwargs):
                self.assertTrue(stream.flushed)
                self.assertIn('baseline', stream.getvalue())
                evidence = kwargs['optional_evidence']
                self.assertEqual([request.side for request in evidence], ['baseline', 'baseline', 'candidate'])
                self.assertTrue(all(request.origin == 'producer_supplied' and request.generated_since is None
                                    for request in evidence))
                self.assertEqual(evidence[0].path, (root / 'one.json').resolve())
                raise PersistenceError(output, 'disk failure')
            with patch('lesscode_baseline.evaluation.evaluate', side_effect=fail), redirect_stdout(stream), redirect_stderr(err):
                code = main(['run', '--plan', str(plan), '--out', str(output),
                             '--baseline-evidence', str(root / 'one.json'),
                             '--baseline-evidence', str(root / 'two.json'),
                             '--candidate-evidence', str(root / 'three.json')])
            self.assertEqual(code, 4)
            self.assertIn(str(output), err.getvalue())

    def test_profile_error_and_storage_error_have_distinct_codes(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            tools = root / 'tools.json'
            tools.write_text('{}')
            common = ['plan', '--profile', str(ROOT / 'benchmarks/profiles/python.json'),
                      '--baseline', str(PILOT), '--candidate', str(PILOT), '--tool-paths', str(tools)]
            self.assertEqual(self.invoke(common + ['--out', str(PILOT / 'plan.json')])[0], 2)
            # A regular file used as parent causes an actual persistence failure.
            self.assertEqual(self.invoke(common + ['--out', str(tools / 'plan.json')])[0], 4)

    def test_help_and_version_do_not_prepare_or_evaluate(self):
        with patch('lesscode_baseline.config.prepare') as prepare, patch('lesscode_baseline.evaluation.evaluate') as evaluate:
            for args in (['--help'], ['--version'], ['plan', '--help'], ['run', '--help']):
                self.assertEqual(self.invoke(args)[0], 0)
            prepare.assert_not_called()
            evaluate.assert_not_called()


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
