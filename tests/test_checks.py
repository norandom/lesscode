import hashlib
import json
from pathlib import Path
import shutil
import sys
import tempfile
import unittest

from lesscode_baseline.contracts import CheckEvidence, CheckSpec, CommandSpec, ContractError, Limits, ResolvedCommand
from lesscode_baseline.process import execute_check


ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "tests/fixtures/python"


class CheckTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.logs = self.root / "logs"
        self.source = "a" * 64
        self.plan = "b" * 64

    def run_report(self, content=None, fmt="junit", ids=("one", "two"),
                   exit_code=0, required=True, code=None, timeout=5):
        if code is None:
            code = (f"from pathlib import Path; Path('result').write_bytes({content!r}); "
                    f"raise SystemExit({exit_code})") if content is not None else "pass"
        command = ResolvedCommand("check", (str(Path(sys.executable).resolve()), "-c", code),
                                  ".", {}, Limits(timeout, 8192), sys.version.split()[0])
        declared = CommandSpec(command.command_id, command.argv, command.cwd,
                               command.environment, command.limits)
        spec = CheckSpec("check", "native", required, declared, "result", fmt, ids)
        return execute_check(spec, command, self.root, self.logs, self.source, self.plan)

    def records(self, **overrides):
        return [dict(check_id=name, status="passed", kind="native", required=True,
                     source_hash=self.source, plan_hash=self.plan, diagnostic_artifact=None)
                | overrides for name in ("one", "two")]

    def test_junit_outcomes_binding_and_raw_diagnostics(self):
        report = b'<testsuites><testsuite><testcase name="one"><failure message="mismatch"/></testcase><testcase name="two"><skipped/></testcase></testsuite></testsuites>'
        checks, execution, failures = self.run_report(report)
        self.assertEqual([c.status for c in checks], ["failed", "skipped"])
        self.assertFalse(failures)
        for check in checks:
            self.assertEqual((check.kind, check.required, check.source_hash, check.plan_hash),
                             ("native", True, self.source, self.plan))
            self.assertEqual((self.logs / check.diagnostic_artifact).read_bytes(), report)
            CheckEvidence.from_dict(check.to_dict())
        self.assertTrue((self.logs / execution.stdout_artifact).exists())

    def test_normalized_and_missing_expected_identity(self):
        records = self.records()[:1]
        checks, _, failures = self.run_report(json.dumps(records).encode(), "normalized")
        self.assertEqual([c.status for c in checks], ["passed", "missing"])
        self.assertFalse(failures)

    def test_zero_tests_and_deleted_or_stale_reports_never_pass(self):
        for report in (b'<testsuite/>', None):
            (self.root / "result").write_text("old evidence")
            checks, _, _ = self.run_report(report)
            self.assertEqual([c.status for c in checks], ["missing", "missing"])

    def test_success_output_cannot_override_nonzero_exit(self):
        xml = b'<testsuite><testcase name="one"/><testcase name="two"/></testsuite>'
        checks, _, failures = self.run_report(xml, exit_code=7)
        self.assertEqual([c.status for c in checks], ["error", "error"])
        self.assertTrue(failures)
        checks, _, _ = self.run_report(b'<testsuite><testcase name="one"><failure/></testcase><testcase name="two"><error/></testcase></testsuite>', exit_code=1)
        self.assertEqual([c.status for c in checks], ["failed", "error"])

    def test_timeout_after_report_preserves_raw_but_cannot_pass(self):
        xml = b'<testsuite><testcase name="one"/><testcase name="two"/></testsuite>'
        code = f"from pathlib import Path; import time; Path('result').write_bytes({xml!r}); time.sleep(20)"
        checks, execution, failures = self.run_report(code=code, timeout=1)
        self.assertEqual(execution.status, "timeout")
        self.assertEqual([c.status for c in checks], ["error", "error"])
        self.assertEqual((self.logs / checks[0].diagnostic_artifact).read_bytes(), xml)
        self.assertTrue(failures)

    def test_recreated_report_with_old_timestamp_is_stale(self):
        code = "from pathlib import Path; import os; Path('result').write_text('<testsuite/>'); os.utime('result', (1, 1))"
        checks, _, failures = self.run_report(code=code)
        self.assertEqual([c.status for c in checks], ["error", "error"])
        self.assertIn("stale", failures[0].message)
        self.assertEqual((self.logs / failures[0].raw_artifact).read_bytes(), b'<testsuite/>')

    def test_invalid_source_binding_prevents_execution(self):
        self.source = "invalid"
        with self.assertRaises(ContractError):
            self.run_report(code="from pathlib import Path; Path('launched').touch()")
        self.assertFalse((self.root / "launched").exists())

    def test_unsafe_malformed_duplicate_unknown_and_foreign_evidence(self):
        invalid_xml = [b'<!DOCTYPE testsuite [<!ENTITY x "hello">]><testsuite/>',
                       '<!DOCTYPE testsuite><testsuite/>'.encode("utf-16"),
                       b'<testsuite><testcase name="one"/><testcase name="one"/></testsuite>',
                       b'<testsuite><testcase name="foreign"/></testsuite>',
                       b'<testsuite failures="1"><testcase name="one"/></testsuite>',
                       b'<testsuite><testcase name="one" status="notrun"/></testsuite>',
                       b'<testsuite><testcase name="one" status="failed"/></testsuite>',
                       b'<testsuite><testcase name="one"><disabled/></testcase></testsuite>',
                       b'<testsuite><unknown><testcase name="one"/></unknown></testsuite>',
                       b'<other/>', b'<testsuite>']
        invalid_json = [self.records(source_hash="c" * 64), self.records(plan_hash="c" * 64),
                        self.records(kind="api"), self.records(required=False),
                        self.records() * 2, self.records(check_id="foreign")]
        for fmt, reports in (("junit", invalid_xml),
                             ("normalized", [json.dumps(r).encode() for r in invalid_json] + [b'{'])):
            for report in reports:
                with self.subTest(fmt=fmt, report=report):
                    checks, _, failures = self.run_report(report, fmt)
                    self.assertTrue(failures)
                    self.assertEqual([c.status for c in checks], ["error", "error"])

    def test_junit_metadata_cannot_supply_executed_checks(self):
        for metadata in ("properties", "system-out", "system-err"):
            with self.subTest(metadata=metadata):
                report = (f'<testsuite tests="2"><{metadata}>'
                          '<testcase name="one"/><testcase name="two"/>'
                          f'</{metadata}></testsuite>').encode()
                checks, _, failures = self.run_report(report)
                self.assertEqual([c.status for c in checks], ["error", "error"])
                self.assertTrue(failures)

    def test_optional_statuses_are_retained(self):
        checks, _, _ = self.run_report(b'<testsuite><testcase name="one"><error/></testcase></testsuite>', required=False)
        self.assertEqual([c.status for c in checks], ["error", "missing"])
        self.assertTrue(all(not c.required for c in checks))

    def test_junit_classname_names_are_unambiguous(self):
        checks, _, failures = self.run_report(b'<testsuite><testcase classname="suite" name="one"/></testsuite>', ids=("suite.one",))
        self.assertEqual(checks[0].status, "passed")
        self.assertFalse(failures)

    def test_independent_oracle_on_unchanged_and_broken_pairs(self):
        for pair, expected in (("unchanged", "passed"), ("shorter_broken", "failed")):
            shutil.copyfile(FIXTURES / f"pairs/{pair}/candidate/subject.py", self.root / "subject.py")
            self.source = hashlib.sha256((self.root / "subject.py").read_bytes()).hexdigest()
            script = FIXTURES / "oracle/check.py"
            command = ResolvedCommand("oracle", (str(Path(sys.executable).resolve()), str(script),
                "subject.py", "result", "junit", self.plan), ".", {}, Limits(5, 8192), sys.version.split()[0])
            spec = CheckSpec("oracle", "invariant", True,
                CommandSpec(command.command_id, command.argv, ".", {}, command.limits),
                "result", "junit", ("clamp.below", "clamp.inside", "clamp.above", "clamp.bounds"))
            checks, execution, _ = execute_check(spec, command, self.root, self.logs, self.source, self.plan)
            self.assertIn(expected, [c.status for c in checks])
            self.assertEqual(execution.exit_code, int(expected == "failed"))

    def test_real_pilot_native_and_independent_runner(self):
        shutil.copyfile(ROOT / "tests/fixtures/real/boltons_mathutils/source/mathutils.py", self.root / "mathutils.py")
        self.source = hashlib.sha256((self.root / "mathutils.py").read_bytes()).hexdigest()
        for mode, ids in (("native", ("native.doctests",)),
                          ("independent", ("independent.golden", "independent.invariants"))):
            script = ROOT / "benchmarks/profiles/pilot/check.py"
            command = ResolvedCommand("pilot", (str(Path(sys.executable).resolve()), str(script),
                "--source", ".", "--output", "result", "--mode", mode), ".", {}, Limits(5, 8192), sys.version.split()[0])
            spec = CheckSpec("pilot", "native" if mode == "native" else "invariant", True,
                CommandSpec(command.command_id, command.argv, ".", {}, command.limits), "result", "junit", ids)
            checks, execution, failures = execute_check(spec, command, self.root, self.logs, self.source, self.plan)
            self.assertEqual([c.status for c in checks], ["passed"] * len(ids))
            self.assertEqual(execution.exit_code, 0)
            self.assertFalse(failures)
