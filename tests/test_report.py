"""Run persistence must survive partial writes without publishing acceptance."""

from dataclasses import replace
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from lesscode_baseline.contracts import ContractError, RunReport, loads
from lesscode_baseline.report import PersistenceError, RunStore, persist, recover, render_summary
from tests.test_contracts import HASH, report


class RunPersistenceTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.target = self.base / "run"
        self.report = RunReport.from_dict(report())

    def store(self):
        return RunStore.create(self.target, HASH, run_id="run-1")

    def raw(self, store):
        store.write_artifact("baseline/size.json", b'{"bytes":12}')
        store.write_artifact("baseline/native.log", b"diagnostic\x00bytes\n")

    def test_prepared_running_and_recovery_are_incomplete(self):
        store = self.store()
        self.assertEqual(json.loads((self.target / "manifest.json").read_text())["state"], "prepared")
        self.assertEqual(recover(self.target).state, "incomplete")
        store.start()
        self.raw(store)
        state = recover(self.target)
        self.assertEqual(state.state, "incomplete")
        self.assertIsNone(state.report)
        self.assertEqual(state.manifest.state, "running")
        self.assertEqual(len(state.manifest.raw_artifacts), 2)

    def test_finalize_roundtrips_report_and_hashes_actual_artifacts(self):
        store = self.store()
        store.start()
        self.raw(store)
        index = store.finalize(self.report)
        self.assertEqual(index.run_id, "run-1")
        self.assertEqual(loads(RunReport, (self.target / "report.json").read_bytes()), self.report)
        self.assertEqual({a.path for a in index.artifacts}, {"manifest.json", "report.json", "summary.md", "baseline/size.json", "baseline/native.log"})
        for artifact in index.artifacts:
            data = (self.target / artifact.path).read_bytes()
            self.assertEqual(artifact.content_hash, hashlib.sha256(data).hexdigest())
            self.assertEqual(artifact.size_bytes, len(data))
        self.assertEqual(recover(self.target).state, "complete")
        self.assertEqual(recover(self.target).report, self.report)
        with self.assertRaises(PersistenceError):
            store.finalize(self.report)

    def test_missing_reference_fails_without_a_completion_index(self):
        store = self.store()
        with self.assertRaises(PersistenceError) as caught:
            store.finalize(self.report)
        self.assertEqual(caught.exception.directory, self.target.resolve())
        self.assertEqual(caught.exception.exit_code, 4)
        self.assertFalse((self.target / "artifacts.json").exists())
        self.assertEqual(recover(self.target).state, "incomplete")

    def test_new_directory_required_and_generated_run_ids_are_distinct(self):
        first = RunStore.create(self.target, HASH)
        with self.assertRaises(PersistenceError):
            self.store()
        second = RunStore.create(self.base / "another", HASH)
        self.assertNotEqual(first.run_id, second.run_id)

    def test_paths_cannot_escape_or_overwrite_owned_files(self):
        store = self.store()
        for path in ["../outside", "/outside", "C:/outside", "baseline/../x", "baseline\\x", "baseline/a.", "manifest.json", "report.json", "artifacts.json", "plan.json"]:
            with self.subTest(path=path), self.assertRaises((ContractError, PersistenceError)):
                store.write_artifact(path, b"bad")
        store.write_artifact("baseline/out.log", b"first")
        with self.assertRaises(PersistenceError):
            store.write_artifact("baseline/out.log", b"second")
        self.assertEqual((self.target / "baseline/out.log").read_bytes(), b"first")

    def test_symlink_escape_is_rejected(self):
        store = self.store()
        outside = self.base / "outside"
        outside.mkdir()
        try:
            (self.target / "link").symlink_to(outside, target_is_directory=True)
        except OSError as exc:
            self.skipTest(f"symlink creation unavailable: {exc}")
        with self.assertRaises((ContractError, PersistenceError)):
            store.write_artifact("link/out.log", b"bad")
        self.assertFalse((outside / "out.log").exists())

    def test_disk_failure_retains_raw_bytes_and_surviving_directory(self):
        store = self.store()
        self.raw(store)
        real_replace = __import__("os").replace
        def fail_report(source, target):
            if Path(target).name == "report.json":
                raise OSError("disk full")
            return real_replace(source, target)
        with patch("lesscode_baseline.report.os.replace", side_effect=fail_report):
            with self.assertRaises(PersistenceError) as caught:
                store.finalize(self.report)
        self.assertIn(str(self.target.resolve()), str(caught.exception))
        self.assertEqual((self.target / "baseline/native.log").read_bytes(), b"diagnostic\x00bytes\n")
        self.assertEqual(recover(self.target).state, "incomplete")
        self.assertFalse((self.target / "artifacts.json").exists())

    def test_interruption_before_index_never_publishes_accepted_run(self):
        store = self.store()
        self.raw(store)
        real_replace = __import__("os").replace
        def interrupt(source, target):
            if Path(target).name == "artifacts.json":
                raise KeyboardInterrupt()
            return real_replace(source, target)
        with patch("lesscode_baseline.report.os.replace", side_effect=interrupt):
            with self.assertRaises(KeyboardInterrupt):
                store.finalize(self.report)
        state = recover(self.target)
        self.assertEqual(state.state, "incomplete")
        self.assertIsNone(state.report)
        self.assertTrue((self.target / "report.json").exists())

    def test_tampered_artifact_or_foreign_index_is_incomplete(self):
        store = self.store()
        self.raw(store)
        store.finalize(self.report)
        (self.target / "baseline/native.log").write_bytes(b"changed")
        self.assertEqual(recover(self.target).state, "incomplete")
        index_path = self.target / "artifacts.json"
        index = json.loads(index_path.read_text())
        index["schema_version"] = 999
        index_path.write_text(json.dumps(index))
        self.assertEqual(recover(self.target).state, "incomplete")

    def test_binding_mismatch_and_invalid_contract_do_not_finalize(self):
        store = self.store()
        self.raw(store)
        for changed in [replace(self.report, run_id="another"), replace(self.report, plan_hash="b" * 64), replace(self.report, schema_version=2)]:
            with self.assertRaises((ContractError, PersistenceError)):
                store.finalize(changed)
        self.assertFalse((self.target / "artifacts.json").exists())

    def test_convenience_persist_copies_references_and_preserves_bytes(self):
        source = self.base / "raw"
        (source / "baseline").mkdir(parents=True)
        (source / "baseline/size.json").write_bytes(b'{"bytes":12}')
        (source / "baseline/native.log").write_bytes(b"original")
        persist(self.report, self.target, artifact_root=source)
        self.assertEqual((self.target / "baseline/native.log").read_bytes(), b"original")
        self.assertEqual(recover(self.target).report, self.report)
        with self.assertRaises(PersistenceError):
            persist(self.report, self.target, artifact_root=source)

    def test_prepared_plan_is_retained_with_configuration_and_tool_identities(self):
        from lesscode_baseline.contracts import PreparedPlan
        from tests.test_plan_contracts import prepared
        plan = PreparedPlan.from_dict(prepared())
        store = RunStore.create(self.target, HASH, run_id="run-1", prepared_plan=plan)
        self.assertEqual(loads(PreparedPlan, (self.target / "plan.json").read_bytes()), plan)
        self.raw(store)
        index = store.finalize(self.report)
        self.assertIn("plan.json", [a.path for a in index.artifacts])

    def test_execution_outputs_and_failure_diagnostics_are_retained(self):
        from lesscode_baseline.contracts import ExecutionResult, Failure, ResolvedCommand
        from tests.test_contracts import command
        store = self.store()
        self.raw(store)
        resolved = ResolvedCommand.from_dict({**command(), "argv": ["C:/Python/python.exe"], "executable_version": "3.12.10"})
        execution = ExecutionResult(resolved, None, "timeout", 30,
                                    "candidate/stdout.log", "candidate/stderr.log",
                                    self.report.started_at, HASH)
        failure = Failure("timeout", "exceeded time", None, "native", None,
                          False, "candidate/diagnostic.json")
        for path in (execution.stdout_artifact, execution.stderr_artifact, failure.raw_artifact):
            store.write_artifact(path, path.encode())
        changed = replace(self.report, decision="incomplete", candidate=replace(
            self.report.candidate, executions=(execution,), failures=(failure,)))
        index = store.finalize(changed)
        self.assertTrue({execution.stdout_artifact, execution.stderr_artifact, failure.raw_artifact}.issubset({a.path for a in index.artifacts}))
        self.assertEqual(recover(self.target).report, changed)

    def test_manifest_update_failure_leaves_unregistered_diagnostics_on_disk(self):
        store = self.store()
        with patch("lesscode_baseline.report.os.replace", side_effect=OSError("disk full")):
            with self.assertRaises(PersistenceError):
                store.write_artifact("candidate/partial.log", b"last command output")
        self.assertEqual((self.target / "candidate/partial.log").read_bytes(), b"last command output")
        state = recover(self.target)
        self.assertEqual(state.state, "incomplete")
        self.assertIsNone(state.report)
        self.assertEqual(state.manifest.raw_artifacts, ())

    def test_unowned_report_file_is_never_overwritten(self):
        store = self.store()
        self.raw(store)
        (self.target / "report.json").write_bytes(b"user file")
        with self.assertRaises(PersistenceError):
            store.finalize(self.report)
        self.assertEqual((self.target / "report.json").read_bytes(), b"user file")
        self.assertFalse((self.target / "artifacts.json").exists())

    def test_foreign_version_or_truncated_report_is_incomplete(self):
        store = self.store()
        self.raw(store)
        store.finalize(self.report)
        report_path = self.target / "report.json"
        report_path.write_text('{"schema_version":2}')
        self.assertEqual(recover(self.target).state, "incomplete")
        report_path.write_text('{')
        self.assertEqual(recover(self.target).state, "incomplete")

    def test_no_implicit_artifact_root_and_no_escape_through_import_root(self):
        with self.assertRaises(PersistenceError):
            persist(self.report, self.target)
        self.assertEqual(recover(self.target).state, "incomplete")
        new_target = self.base / "rerun"
        with patch("pathlib.Path.is_symlink", return_value=True):
            with self.assertRaises(ContractError):
                RunStore.create(new_target, HASH)
        self.assertFalse(new_target.exists())

    def test_case_alias_registration_is_rejected(self):
        store = self.store()
        store.write_artifact("baseline/Log.txt", b"text")
        other = self.target / "baseline/log.txt"
        if not other.exists():
            other.write_bytes(b"text")
        with self.assertRaises(PersistenceError):
            store.register_artifact("baseline/log.txt")

class ComparisonSummaryTests(unittest.TestCase):
    setUp = RunPersistenceTests.setUp
    store = RunPersistenceTests.store
    raw = RunPersistenceTests.raw

    def test_structured_payloads_remain_data_without_inferred_deltas(self):
        import html
        from lesscode_baseline.contracts import PopulationPayload
        population = PopulationPayload("population", ("test.one", "test.two"), HASH, 1, 2, "new population")
        metric = replace(self.report.baseline.metrics[0], key="coverage", value=population, unit="population", definition_id="coverage-v1")
        changed = replace(population, numerator=2)
        data = replace(self.report, baseline=replace(self.report.baseline, metrics=(metric,)), candidate=replace(self.report.candidate, metrics=(replace(metric, value=changed),)))
        text = html.unescape(render_summary(data))
        for payload in (population, changed):
            self.assertIn(json.dumps(payload.to_dict(), ensure_ascii=True, sort_keys=True), text)
        self.assertNotIn("50.0", text)
        self.assertNotIn("100.0", text)
        self.assertIn("no comparison recorded", text)

    def test_numeric_deltas_and_zero_percentage_reason_are_preserved(self):
        from lesscode_baseline.contracts import MetricDelta
        delta = MetricDelta("source_bytes", "available", 0, 12, 12, None, None, "zero baseline", "bytes")
        text = render_summary(replace(self.report, metric_deltas=(delta,)))
        self.assertIn("before `0`; after `12`; absolute delta `12` `bytes`", text)
        self.assertIn("percentage delta unavailable", text)
        self.assertIn("zero baseline", text)

    def test_every_decision_retains_reasons_and_version_in_both_formats(self):
        from lesscode_baseline.contracts import DECISIONS
        for decision in DECISIONS:
            with self.subTest(decision=decision):
                data = replace(self.report, decision=decision, reasons=("first reason", "second reason"))
                text = render_summary(data)
                self.assertIn(f"Decision: `{decision}`", text)
                self.assertIn("first reason", text)
                self.assertIn("second reason", text)
                self.assertIn("format version: 1", text)
                target = self.base / decision
                source = self.base / "source"
                (source / "baseline").mkdir(parents=True, exist_ok=True)
                (source / "baseline/size.json").write_bytes(b"raw")
                (source / "baseline/native.log").write_bytes(b"log")
                persist(data, target, artifact_root=source)
                self.assertEqual((target / "summary.md").read_text(encoding="utf-8"), text)
                self.assertEqual(recover(target).report, data)

    def test_summary_preserves_missing_checks_metric_reasons_and_producers(self):
        from lesscode_baseline.contracts import MetricDelta
        missing = replace(self.report.candidate.metrics[0], status="unavailable", value=None, reason="analyzer absent", producer="different", producer_version="9")
        data = replace(self.report, candidate=replace(self.report.candidate, metrics=(missing,), checks=(replace(self.report.candidate.checks[0], status="skipped"),)), metric_deltas=(MetricDelta("source_bytes", "unavailable", 12, None, None, None, "analyzer absent", "candidate unavailable", "bytes"),))
        text = render_summary(data)
        for value in ("skipped", "analyzer absent", "different", "9", "lesscode", "0.1", "bytes", "candidate unavailable", "raw-source-bytes-v1", HASH):
            self.assertIn(value, text)
        self.assertLess(text.index("Declared checks"), text.index("Structural and other metrics"))
        self.assertIn("not proof of equivalence", text)
        self.assertIn("Unchecked behavior remains", text)

    def test_changes_integrity_and_time_are_separate(self):
        from lesscode_baseline.contracts import FileDelta, Protection
        changed_file = replace(self.report.candidate.snapshot.files[0], content_hash="b" * 64)
        added_file = replace(changed_file, path="src/new.py")
        data = replace(self.report, candidate=replace(self.report.candidate, snapshot=replace(self.report.candidate.snapshot, files=(changed_file, added_file))), test_file_delta=FileDelta(("tests/new.py",), ("tests/old.py",), ("tests/edit.py",)), integrity=replace(self.report.integrity, status="changed", changed_resources=("checks/oracle.py",), protection=Protection("external_isolation_claimed", "CI job reference")), elapsed_seconds=17.25)
        text = render_summary(data)
        for value in ("src/main.py", "src/new.py", "tests/new.py", "tests/old.py", "tests/edit.py", "checks/oracle.py", "external_isolation_claimed", "CI job reference", "17.25", "Agent execution time: unavailable"):
            self.assertIn(value, text)
        self.assertIn("Product files", text)
        self.assertIn("Test files", text)

    def test_untrusted_text_cannot_inject_markdown_or_html(self):
        evil = "[click](https://evil.invalid) <script>x</script> `oops`\n# injected"
        data = replace(self.report, reasons=(evil,), limitations=(evil,))
        text = render_summary(data)
        self.assertNotIn("<script>", text)
        self.assertNotIn("`oops`", text)
        self.assertNotIn("\n# injected", text)
        self.assertIn("&lt;script&gt;", text)

    def test_summary_disk_failure_is_incomplete_with_diagnostics_retained(self):
        store = self.store()
        self.raw(store)
        real_replace = __import__("os").replace
        def fail_summary(source, target):
            if Path(target).name == "summary.md":
                raise OSError("summary disk full")
            return real_replace(source, target)
        with patch("lesscode_baseline.report.os.replace", side_effect=fail_summary):
            with self.assertRaises(PersistenceError):
                store.finalize(self.report)
        self.assertFalse((self.target / "artifacts.json").exists())
        self.assertEqual(recover(self.target).state, "incomplete")
        self.assertEqual((self.target / "baseline/native.log").read_bytes(), b"diagnostic\x00bytes\n")

    def test_missing_tampered_or_unindexed_summary_is_incomplete(self):
        store = self.store()
        self.raw(store)
        store.finalize(self.report)
        path = self.target / "summary.md"
        self.assertTrue(path.exists())
        original = path.read_bytes()
        path.write_bytes(b"Decision: accepted")
        self.assertEqual(recover(self.target).state, "incomplete")
        path.write_bytes(original)
        index_path = self.target / "artifacts.json"
        index = json.loads(index_path.read_text())
        index["artifacts"] = [a for a in index["artifacts"] if a["path"] != "summary.md"]
        index_path.write_text(json.dumps(index))
        self.assertEqual(recover(self.target).state, "incomplete")
        path.unlink()
        self.assertEqual(recover(self.target).state, "incomplete")
