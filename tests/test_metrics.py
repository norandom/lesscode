"""Independent size definitions and failure evidence over real staged inputs."""

from dataclasses import replace
import gzip
import json
from io import BytesIO
from pathlib import Path
import platform
import struct
import sys
import tempfile
import unittest
from unittest.mock import patch
import zlib

from lesscode_baseline.contracts import Failure, Limits, MetricEvidence, ScopeRules, Snapshot
from lesscode_baseline.metrics import measure_complexity, measure_duplication, measure_sizes, stage_product
from lesscode_baseline.snapshot import capture_scope


class SizeMetricTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.root = self.base / "source"
        self.root.mkdir()
        self.logs = self.base / "logs"
        self.limits = Limits(10, 100000)

    def write(self, path, content):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(content)

    def snapshot(self, product=("**/*.py",)):
        snapshot = capture_scope(self.root, ScopeRules(("**/*",), ("runs/**",),
            ("generated/**",), ("vendor/**",), ("tests/**",), product))
        self.assertIsInstance(snapshot, Snapshot)
        return snapshot

    def measure(self, snapshot=None, interpreter=None):
        return measure_sizes(snapshot or self.snapshot(),
            Path(interpreter or sys.executable), self.limits, self.logs)

    def assert_unavailable(self, metrics, reason):
        self.assertEqual(len(metrics), 3)
        for metric in metrics:
            self.assertEqual(metric.status, "unavailable")
            self.assertIsNone(metric.value)
            self.assertIn(reason, metric.reason)
            self.assertEqual(MetricEvidence.from_dict(metric.to_dict()), metric)

    def test_exact_definitions_determinism_and_provenance(self):
        files = {"z.py": b"# utf8: \xc3\xa9\r\nx = 1", "a.py": b"\n# hello\n"}
        for path, content in files.items():
            self.write(path, content)
        for path in ("tests/t.py", "vendor/v.py", "generated/g.py", "runs/r.py"):
            self.write(path, b"invalid python !")
        self.write(".gitignore", b"*.py")
        self.write(".jscpd.json", b'{"ignore": ["**/*"]}')
        snapshot = self.snapshot()
        first = self.measure(snapshot)
        second = self.measure(replace(snapshot, files=tuple(reversed(snapshot.files))))
        self.assertEqual([(m.key, m.value) for m in first], [(m.key, m.value) for m in second])
        values = {m.key: m.value for m in first}
        self.assertEqual(values["source_bytes"], sum(map(len, files.values())))
        self.assertEqual(values["physical_lines"], 4)
        framed = b""
        for name in sorted(files):
            path = name.encode("utf-8")
            framed += struct.pack(">Q", len(path)) + path
            framed += struct.pack(">Q", len(files[name])) + files[name]
        stream = BytesIO()
        with gzip.GzipFile(fileobj=stream, mode="wb", filename="", compresslevel=9, mtime=0) as output:
            output.write(framed)
        self.assertEqual(values["compressed_source_bytes"], len(stream.getvalue()))
        for metric in first:
            self.assertEqual(metric.status, "available")
            self.assertEqual(metric.scope_hash, snapshot.scope_hash)
            self.assertIn(platform.python_version(), metric.producer_version)
            self.assertEqual(MetricEvidence.from_dict(metric.to_dict()), metric)
        compressed = first[2]
        self.assertIn(zlib.ZLIB_RUNTIME_VERSION, compressed.producer_version)
        self.assertIn("level9", compressed.definition_id)
        self.assertIn("mtime0", compressed.definition_id)

    def test_stage_contains_only_sorted_product_and_cleans_up(self):
        self.write(".hidden.py", b"pass")
        self.write("nested/main.py", b"pass")
        self.write("tests/t.py", b"invalid !")
        self.write(".jscpd.json", b"{}")
        with stage_product(self.snapshot()) as stage:
            self.assertIsInstance(stage, Snapshot)
            root = Path(stage.root)
            self.assertEqual([r.path for r in stage.files], [".hidden.py", "nested/main.py"])
            self.assertEqual(sorted(p.relative_to(root).as_posix() for p in root.rglob("*") if p.is_file()),
                             [".hidden.py", "nested/main.py"])
        self.assertFalse(root.exists())
        self.assertTrue((self.root / "nested/main.py").exists())

    def test_empty_scope_and_empty_file_are_valid(self):
        empty = {m.key: m.value for m in self.measure()}
        self.assertEqual(empty["source_bytes"], 0)
        self.assertEqual(empty["physical_lines"], 0)
        self.assertEqual(empty["compressed_source_bytes"], 20)
        self.write("empty.py", b"")
        measured = {m.key: m.value for m in self.measure()}
        self.assertEqual(measured["physical_lines"], 0)
        self.assertGreater(measured["compressed_source_bytes"], 20)

    def test_physical_lines_preserve_byte_newlines(self):
        for content, lines in [(b"x=1", 1), (b"x=1\n", 1), (b"x=1\r\n\r\n", 2),
                               (b"# a\r# b\r", 2)]:
            with self.subTest(content=content):
                self.write("main.py", content)
                values = {m.key: m.value for m in self.measure()}
                self.assertEqual(values["physical_lines"], lines)
                self.assertEqual(values["source_bytes"], len(content))

    def test_invalid_syntax_and_encoding_have_raw_diagnostics(self):
        for content in (b"def broken(:", b"x = '\xff'", b"x=\x00"):
            self.write("broken.py", content)
            metrics = self.measure()
            self.assert_unavailable(metrics, "syntax")
            self.assertTrue((self.logs / metrics[0].raw_artifact).read_bytes())

    def test_missing_changed_and_unsupported_product_unavailable(self):
        self.write("main.py", b"pass")
        snapshot = self.snapshot()
        self.write("main.py", b"exit")
        self.assert_unavailable(self.measure(snapshot), "main.py")
        (self.root / "main.py").unlink()
        self.assert_unavailable(self.measure(snapshot), "main.py")
        self.write("main.txt", b"pass")
        snapshot = self.snapshot(("**/*",))
        unsupported = replace(snapshot, files=tuple(replace(r, category="product") for r in snapshot.files))
        self.assert_unavailable(self.measure(unsupported), "unsupported")

    def test_missing_interpreter_unavailable(self):
        self.write("main.py", b"pass")
        self.assert_unavailable(self.measure(interpreter=self.base / "missing-python.exe"), "syntax")

    def test_bounded_execution_failure_and_configuration_isolation(self):
        self.write("main.py", b"raise RuntimeError('must only parse')")
        from lesscode_baseline.process import execute
        commands = []
        def observe(command, workspace, logs):
            commands.append(command)
            result = execute(command, workspace, logs)
            return replace(result, status="timeout", exit_code=None)
        with patch("lesscode_baseline.metrics.execute", side_effect=observe):
            self.assert_unavailable(self.measure(), "timeout")
        self.assertEqual(commands[0].argv[1:3], ("-I", "-S"))
        self.assertEqual(commands[0].environment, {})
        self.assertEqual(commands[0].limits, self.limits)
        self.assertTrue(all(m.status == "available" for m in self.measure()))

    def test_unsafe_manifest_path_and_root_fail_closed(self):
        self.write("main.py", b"pass")
        snapshot = self.snapshot()
        unsafe = replace(snapshot, files=(replace(snapshot.files[0], path="../escape.py"),))
        with self.assertRaises(ValueError):
            self.measure(unsafe)
        missing = replace(snapshot, root=str(self.base / "missing"))
        self.assert_unavailable(self.measure(missing), "input")

    def test_stage_cleanup_on_caller_exception(self):
        self.write("main.py", b"pass")
        with self.assertRaisesRegex(RuntimeError, "caller"):
            with stage_product(self.snapshot()) as staged:
                root = Path(staged.root)
                raise RuntimeError("caller")
        self.assertFalse(root.exists())

    def test_successful_exit_without_syntax_inventory_is_unavailable(self):
        self.write("main.py", b"pass")
        from lesscode_baseline.process import execute
        def corrupt_report(command, workspace, logs):
            execution = execute(command, workspace, logs)
            (Path(logs) / execution.stdout_artifact).write_text("{}", encoding="utf-8")
            return execution
        with patch("lesscode_baseline.metrics.execute", side_effect=corrupt_report):
            self.assert_unavailable(self.measure(), "inventory")


class ComplexityMetricTests(unittest.TestCase):
    setUp = SizeMetricTests.setUp
    write = SizeMetricTests.write
    snapshot = SizeMetricTests.snapshot
    interpreter = Path(__file__).resolve().parents[1] / ".tools/analyzers/python/Scripts/python.exe"

    def measure(self, snapshot=None, interpreter=None):
        return measure_complexity(snapshot or self.snapshot(), interpreter or self.interpreter,
                                  self.limits, self.logs)

    def assert_unavailable(self, metric, reason):
        self.assertEqual(metric.status, "unavailable")
        self.assertIsNone(metric.value)
        self.assertIn(reason, metric.reason)
        self.assertEqual(MetricEvidence.from_dict(metric.to_dict()), metric)

    def test_known_functions_zero_function_files_distribution_and_determinism(self):
        self.write("nested/main.py", b"def classify(value):\n    result = 0\n    if value > 0:\n        result += value\n    for item in range(value):\n        result += item\n    result += 1\n    return result\n\ndef plain():\n    return 1\n")
        self.write("empty.py", b"")
        self.write("constant.py", b"VALUE = 3\n")
        self.write("tests/invalid.py", b"invalid !")
        self.write(".gitignore", b"*.py")
        self.write("lizard.py", b"raise RuntimeError('project must never import')\n")
        snapshot = self.snapshot()
        metric = self.measure(snapshot)
        self.assertEqual(metric.status, "available", metric.reason)
        payload = metric.value
        self.assertEqual([f.path for f in payload.files], ["constant.py", "empty.py", "lizard.py", "nested/main.py"])
        self.assertEqual([(f.name, f.start_line, f.end_line, f.ccn, f.nloc, f.token_count)
                          for f in payload.functions], [("classify", 1, 8, 3, 8, 32), ("plain", 10, 11, 1, 2, 6)])
        self.assertEqual((payload.count, payload.minimum, payload.maximum, payload.median, payload.p95),
                         (2, 1, 3, 2, 3))
        self.assertEqual(metric.scope_hash, snapshot.scope_hash)
        self.assertEqual(metric.producer, "lizard")
        self.assertEqual(metric.producer_version, "1.17.31")
        self.assertIn("nearest-rank", metric.definition_id)
        self.assertEqual(MetricEvidence.from_dict(metric.to_dict()), metric)
        second = self.measure(replace(snapshot, files=tuple(reversed(snapshot.files))))
        self.assertEqual(second.value, payload)
        self.assertTrue((self.logs / metric.raw_artifact).exists())

    def test_empty_scope_and_no_functions_have_null_distribution(self):
        for data in (None, b"", b"x = 1\n"):
            if data is not None:
                self.write("main.py", data)
            metric = self.measure()
            self.assertEqual(metric.status, "available", metric.reason)
            value = metric.value
            self.assertEqual(value.count, 0)
            self.assertEqual((value.minimum, value.maximum, value.median, value.p95), (None,) * 4)
            self.assertEqual(len(value.files), int(data is not None))

    def test_nearest_rank_p95_odd_median_and_hotspot_order(self):
        source = ""
        for index in range(1, 22):
            source += f"def function_{index}(x):\n"
            source += "    if x:\n        x -= 1\n" * (index - 1)
            source += "    return x\n\n"
        self.write("main.py", source.encode())
        metric = self.measure()
        self.assertEqual(metric.status, "available", metric.reason)
        value = metric.value
        self.assertEqual((value.count, value.minimum, value.maximum, value.median, value.p95),
                         (21, 1, 21, 11, 20))
        self.assertEqual([f.ccn for f in value.functions], list(range(21, 0, -1)))

    def test_async_nested_methods_and_original_newlines(self):
        self.write("main.py", b"class C:\r\n    async def method(self):\r\n        def inner():\r\n            return 1\r\n        return inner()\r\n")
        metric = self.measure()
        self.assertEqual(metric.status, "available", metric.reason)
        self.assertEqual([(f.name, f.start_line) for f in metric.value.functions],
                         [("method", 2), ("method.inner", 3)])

    def test_utf8_source_and_bom_are_locale_independent(self):
        for prefix in (b"", b"\xef\xbb\xbf"):
            with self.subTest(prefix=prefix):
                self.write("main.py", prefix + "def café():\n    return 'é'\n".encode())
                metric = self.measure()
                self.assertEqual(metric.status, "available", metric.reason)
                self.assertEqual(metric.value.functions[0].name, "café")

    def test_all_pinned_source_suppression_variants(self):
        markers = ("# GENERATED CODE", "#lizard forgive", "# lizard forgive", "###lizard forgive",
                   "#lizard forgive global", "# #lizard forgive global tail", "#lizard forgive_suffix")
        for marker in markers:
            with self.subTest(marker=marker):
                self.write("main.py", (marker + "\ndef simple():\n    return 1\n").encode())
                self.assert_unavailable(self.measure(), "suppression")
        self.write("main.py", b"def literal():\n    return '# GENERATED CODE #lizard forgive'\n")
        self.assertEqual(self.measure().status, "available")

    def test_invalid_syntax_encoding_missing_runtime_and_unsupported_inputs(self):
        for data in (b"def broken(:", b"x = '\xff'", b"x = \x00"):
            self.write("main.py", data)
            self.assert_unavailable(self.measure(), "syntax")
        self.write("main.py", b"def simple():\n    return 1\n")
        self.assert_unavailable(self.measure(interpreter=self.base / "missing.exe"), "execution")
        self.assert_unavailable(self.measure(interpreter=Path(sys.executable)), "execution")
        snapshot = self.snapshot()
        self.write("main.py", b"pass")
        self.assert_unavailable(self.measure(snapshot), "input")
        self.write("main.txt", b"pass")
        all_files = self.snapshot(("**/*",))
        unsupported = replace(all_files, files=tuple(replace(f, category="product") for f in all_files.files))
        self.assert_unavailable(self.measure(unsupported), "unsupported")

    def test_bounded_worker_and_failed_or_corrupt_output(self):
        self.write("main.py", b"def simple():\n    return 1\n")
        from lesscode_baseline.process import execute
        commands = []
        def observe(command, workspace, logs):
            commands.append(command)
            return execute(command, workspace, logs)
        with patch("lesscode_baseline.metrics.execute", side_effect=observe):
            self.assertEqual(self.measure().status, "available")
        self.assertEqual(commands[0].argv[1], "-I")
        self.assertEqual(commands[0].environment, {})
        self.assertEqual(commands[0].limits, self.limits)
        for corruption in ("malformed", "version", "omitted_file", "omitted_function", "wrong_path", "bad_count", "bad_line", "bad_nloc", "timeout", "stderr"):
            def corrupt(command, workspace, logs):
                result = execute(command, workspace, logs)
                stdout = Path(logs) / result.stdout_artifact
                report = json.loads(stdout.read_text(encoding="utf-8"))
                if corruption == "malformed":
                    stdout.write_text("{}", encoding="utf-8")
                elif corruption == "timeout":
                    return replace(result, status="timeout", exit_code=None)
                elif corruption == "stderr":
                    (Path(logs) / result.stderr_artifact).write_text("[skip] incomplete", encoding="utf-8")
                else:
                    if corruption == "version": report["version"] = "0.0"
                    elif corruption == "omitted_file": report["files"] = []
                    elif corruption == "omitted_function": report["functions"] = []
                    elif corruption == "wrong_path": report["functions"][0]["path"] = "../foreign.py"
                    elif corruption == "bad_count": report["expected_functions"]["main.py"] = []
                    elif corruption == "bad_line": report["functions"][0]["end_line"] = 999
                    elif corruption == "bad_nloc": report["files"][0]["nloc"] = -1
                    stdout.write_text(json.dumps(report), encoding="utf-8")
                return result
            with self.subTest(corruption=corruption), patch("lesscode_baseline.metrics.execute", side_effect=corrupt):
                self.assertEqual(self.measure().status, "unavailable")


class DuplicationMetricTests(unittest.TestCase):
    setUp = SizeMetricTests.setUp
    write = SizeMetricTests.write
    snapshot = SizeMetricTests.snapshot
    assert_unavailable = ComplexityMetricTests.assert_unavailable
    node = Path("C:/Program Files/nodejs/node.exe")
    entry = Path(__file__).resolve().parents[1] / ".tools/analyzers/node/node_modules/jscpd/bin/jscpd"
    duplicate = (b"def classify(value):\n    result = 0\n    if value > 0:\n        result += value\n    for item in range(value):\n        result += item\n"
                 + b"    result += 1\n" * 9 + b"    return result\n")

    def measure(self, snapshot=None, node=None, entry=None):
        return measure_duplication(snapshot or self.snapshot(), Path(sys.executable),
            node or self.node, entry or self.entry, self.limits, self.logs)

    def test_real_pair_span_counts_locations_and_configuration_isolation(self):
        self.write("nested/a.py", self.duplicate)
        self.write("b.py", self.duplicate)
        self.write("empty.py", b"")
        self.write(".jscpd.json", b'{"ignore":["**/*"]}')
        self.write("tests/broken.py", b"invalid !")
        first = self.measure()
        self.assertEqual(first.status, "available", first.reason)
        # Original token-position span is 62 + eight added 7-token lines.
        self.assertEqual((first.value.total_tokens, first.value.duplicated_tokens), (236, 118))
        self.assertEqual(first.value.ratio, .5)
        self.assertEqual(first.value.upstream_percentage, 50)
        self.assertEqual(first.value.scanned_paths, ("b.py", "nested/a.py"))
        self.assertEqual(first.value.zero_token_paths, ("empty.py",))
        self.assertEqual(len(first.value.clones), 1)
        clone = first.value.clones[0]
        self.assertEqual({clone.first.path, clone.second.path}, {"b.py", "nested/a.py"})
        self.assertEqual((clone.first.start_line, clone.first.end_line), (1, 16))
        self.assertEqual(first.key, "duplication")
        self.assertEqual(first.producer, "jscpd")
        self.assertEqual(first.definition_id,
                         "jscpd-python-weak-minlines5-mintokens50-aggregate-pair-span-v1")
        self.assertEqual(first.unit, "ratio")
        self.assertEqual(first.producer_version, "4.0.5")
        self.assertEqual(MetricEvidence.from_dict(first.to_dict()), first)
        self.assertEqual(self.measure().value, first.value)
        self.assertTrue((self.logs / first.raw_artifact).is_file())

    def test_real_duplication_satisfies_required_profile_and_decision_lookup(self):
        from lesscode_baseline import contracts as c
        from lesscode_baseline.decision import compare, compare_metrics
        from tests.test_contracts import observation, profile

        self.write("a.py", self.duplicate)
        self.write("b.py", self.duplicate)
        snapshot = self.snapshot()
        evidence = self.measure(snapshot)
        self.assertEqual(evidence.status, "available", evidence.reason)
        self.assertIsInstance(evidence.value, c.DuplicationPayload)
        frozen = c.Profile.from_dict(profile())
        base = c.Observation.from_dict(observation())
        # Other mandatory evidence is valid; use the unmodified adapter result
        # for duplication, exactly as the evaluator's metric lookup receives it.
        other_metrics = tuple(replace(base.metrics[0], key=key, scope_hash=snapshot.scope_hash)
                              for key in frozen.required_metrics if key != "duplication")
        base = replace(base, snapshot=snapshot, metrics=other_metrics + (evidence,),
                       checks=tuple(replace(check, source_hash=snapshot.content_hash)
                                    for check in base.checks))
        result = compare(base, base, frozen,
                         c.IntegrityResult("unchanged", (), c.Protection("local_detection", None)))
        self.assertEqual(result.decision, "accepted", result.reasons)
        metrics = {item.delta.key: item for item in compare_metrics(base.metrics, base.metrics)}
        duplicate = metrics["duplication"]
        self.assertEqual(duplicate.status, "available")
        self.assertEqual(duplicate.before.value, evidence.value)
        self.assertEqual(duplicate.after.value, evidence.value)
        self.assertEqual(duplicate.delta.status, "unavailable")
        self.assertIsNone(duplicate.delta.absolute_delta)
        self.assertIsNone(duplicate.delta.percentage_delta)

    def test_unavailable_duplication_retains_public_key_and_definition(self):
        self.write("a.py", self.duplicate)
        for arguments in ({"node": self.base / "missing-node.exe"},
                          {"entry": self.base / "missing-entry"}):
            with self.subTest(arguments=arguments):
                metric = self.measure(**arguments)
                self.assertEqual(metric.status, "unavailable")
                self.assertEqual(metric.key, "duplication")
                self.assertIsNone(metric.value)
                self.assertTrue(metric.reason)
                self.assertEqual((metric.producer, metric.producer_version, metric.unit),
                                 ("jscpd", "4.0.5", "ratio"))
                self.assertEqual(metric.definition_id,
                                 "jscpd-python-weak-minlines5-mintokens50-aggregate-pair-span-v1")
                self.assertEqual(MetricEvidence.from_dict(metric.to_dict()), metric)

    def test_empty_only_retains_missing_vendor_report_and_mixed_zero_inventory(self):
        for content in (None, b"", b"# only a comment\n\n", b" \n\t\n"):
            if content is not None:
                self.write("main.py", content)
            metric = self.measure()
            self.assert_unavailable(metric, "missing fresh")
        self.write("nonempty.py", self.duplicate)
        for content in (b"", b"# comment\n\n", b" \n\t\n"):
            self.write("main.py", content)
            metric = self.measure()
            self.assertEqual(metric.status, "available", metric.reason)
            self.assertEqual(metric.value.zero_token_paths, ("main.py",))

    def test_suppression_syntax_and_silent_short_file_omission(self):
        for marker in (b"jscpd:ignore-start", b"jscpd:ignore-end"):
            self.write("main.py", b"# " + marker + b"\n" + self.duplicate)
            self.assert_unavailable(self.measure(), "suppression")
        self.write("main.py", b"def broken(:")
        self.assert_unavailable(self.measure(), "syntax")
        self.write("main.py", b"# coding: latin-1\nx = '\xe9'\n")
        self.assert_unavailable(self.measure(), "encoded data")
        self.write("other.py", self.duplicate)
        self.write("main.py", b"pass\n")
        self.assert_unavailable(self.measure(), "inventory")

    def test_zero_position_span_with_meaningful_token_is_scanned(self):
        self.write("main.py", b"pass\n\n\n\n\n")
        metric = self.measure()
        self.assertEqual(metric.status, "available", metric.reason)
        self.assertEqual(metric.value.scanned_paths, ("main.py",))
        self.assertEqual(metric.value.zero_token_paths, ())
        self.assertEqual(metric.value.total_tokens, 0)
        self.assertIsNone(metric.value.ratio)
        self.assertTrue(metric.value.ratio_reason)

    def test_actual_large_input_explicit_bounds_and_bounded_commands(self):
        self.write("main.py", self.duplicate + b"#" + b"x" * 110000 + b"\n" * 1100)
        from lesscode_baseline.process import execute
        commands = []
        def observe(command, workspace, logs):
            commands.append(command)
            return execute(command, workspace, logs)
        with patch("lesscode_baseline.metrics.execute", side_effect=observe):
            metric = self.measure()
        self.assertEqual(metric.status, "available", metric.reason)
        self.assertEqual(len(commands), 3)
        for command in commands:
            self.assertEqual(command.limits, self.limits)
            self.assertFalse({"NODE_OPTIONS", "PYTHONPATH", "PATH"} & set(command.environment))
        argv = commands[-1].argv
        self.assertTrue(all("\\" not in argument for argument in argv))
        self.assertGreater(int(argv[argv.index("--max-lines") + 1]), 1100)
        self.assertGreater(int(argv[argv.index("--max-size") + 1][:-1]), 110000)
        self.assertEqual(argv[argv.index("--min-tokens") + 1], "50")
        with patch("lesscode_baseline.metrics._duplication_bounds", side_effect=ValueError("unsupported bounds")):
            self.assert_unavailable(self.measure(), "unsupported bounds")

    def test_missing_runtime_entry_and_wrong_pinned_version(self):
        self.write("main.py", self.duplicate)
        self.assert_unavailable(self.measure(node=self.base / "missing-node.exe"), "execution")
        self.assert_unavailable(self.measure(entry=self.base / "missing-entry"), "entry")
        fake = self.base / "other-package"
        (fake / "bin").mkdir(parents=True)
        (fake / "package.json").write_text('{"version":"0.0.0"}', encoding="utf-8")
        (fake / "bin/jscpd").write_text("", encoding="utf-8")
        metric = self.measure(entry=fake / "bin/jscpd")
        self.assert_unavailable(metric, "wrong jscpd version")
        self.assertTrue((self.logs / metric.raw_artifact).read_bytes())

    def test_missing_malformed_omitted_foreign_and_bounded_cli_output(self):
        self.write("a.py", self.duplicate)
        self.write("b.py", self.duplicate)
        from lesscode_baseline.process import execute
        for corruption in ("missing", "malformed", "omitted", "foreign", "denominator", "numerator",
                           "location", "percentage", "sources_schema", "timeout", "stderr", "inventory_version"):
            def corrupt(command, workspace, logs):
                result = execute(command, workspace, logs)
                if command.command_id == "duplication-inventory" and corruption == "inventory_version":
                    output = Path(logs) / result.stdout_artifact
                    value = json.loads(output.read_text())
                    value["version"] = "0.0"
                    output.write_text(json.dumps(value), encoding="utf-8")
                if command.command_id != "metrics-duplication":
                    return result
                report_path = Path(command.argv[command.argv.index("--output") + 1]) / "jscpd-report.json"
                if corruption == "missing":
                    report_path.unlink()
                elif corruption == "malformed":
                    report_path.write_text("{}", encoding="utf-8")
                elif corruption == "timeout":
                    return replace(result, status="timeout", exit_code=None)
                elif corruption == "stderr":
                    (Path(logs) / result.stderr_artifact).write_text("[skip] incomplete", encoding="utf-8")
                else:
                    report = json.loads(report_path.read_text())
                    sources = report["statistics"]["formats"]["python"]["sources"]
                    if corruption == "omitted": sources.pop(next(iter(sources)))
                    elif corruption == "foreign": sources["../foreign.py"] = sources.pop(next(iter(sources)))
                    elif corruption == "denominator": report["statistics"]["total"]["tokens"] += 1
                    elif corruption == "numerator": report["statistics"]["total"]["duplicatedTokens"] = 0
                    elif corruption == "location": report["duplicates"][0]["firstFile"]["end"] = 100000
                    elif corruption == "percentage": report["statistics"]["total"]["percentageTokens"] = float("nan")
                    elif corruption == "sources_schema": report["statistics"]["formats"]["python"] = []
                    report_path.write_text(json.dumps(report), encoding="utf-8")
                return result
            with self.subTest(corruption=corruption), patch("lesscode_baseline.metrics.execute", side_effect=corrupt):
                metric = self.measure()
                self.assertEqual(metric.status, "unavailable", metric)
                self.assertIsNone(metric.value)


if __name__ == "__main__":
    unittest.main()
