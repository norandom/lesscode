"""Independent size definitions and failure evidence over real staged inputs."""

from dataclasses import replace
import gzip
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
from lesscode_baseline.metrics import measure_sizes, stage_product
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


if __name__ == "__main__":
    unittest.main()
