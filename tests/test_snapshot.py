"""Source capture observes bytes and portable scope, independently of Git."""

import hashlib
import json
from pathlib import Path
import stat
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from lesscode_baseline.contracts import Failure, ScopeRules, Snapshot
from lesscode_baseline.snapshot import capture, capture_scope, disposable_workspace, file_changes, matches


def rules(**overrides):
    values = dict(include=("**/*",), exclude=(".git/**", "runs/**"),
                  generated=("generated/**",), vendor=("vendor/**",),
                  tests=("tests/**",), product=("**/*.py",))
    values.update(overrides)
    return ScopeRules(**values)


class SnapshotTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def write(self, name, content=b"x"):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
        return path

    def capture(self, scope=None):
        result = capture_scope(self.root, scope or rules())
        self.assertIsInstance(result, Snapshot, result)
        return result

    def test_portable_whole_path_globs(self):
        for path, pattern, expected in [
            ("main.py", "**/*.py", True), ("src/main.py", "*.py", False),
            ("src/deep/a.py", "src/**/?.py", True), ("src/a.py", "src/**/?.py", True),
            ("A.py", "a.py", False), ("src/a.py", "src/[ab].py", True),
            ("src/a.txt", "**/*.py", False), ("a/b/c", "a/*", False),
            (".hidden.py", "**/*", True),
        ]:
            with self.subTest(path=path, pattern=pattern):
                self.assertEqual(matches(path, pattern), expected)

    def test_classification_precedence_and_explicit_exclusions(self):
        for name in ("main.py", "README", "tests/t.py", "vendor/v.py", "vendor/tests/t.py",
                     "generated/vendor/t.py", "runs/result.py", ".git/config", "omit.txt", "Upper.PY"):
            self.write(name)
        result = self.capture(rules(include=("**/*.py", "README", "Upper.PY"),
                                    generated=("generated/**", "vendor/tests/**")))
        categories = {f.path: f.category for f in result.files}
        self.assertEqual(categories, {"README": "support", "Upper.PY": "support",
            "generated/vendor/t.py": "generated", "main.py": "product",
            "tests/t.py": "tests", "vendor/tests/t.py": "generated", "vendor/v.py": "vendor"})
        self.assertEqual(result.exclusions, (".git/config", "omit.txt", "runs/result.py"))
        self.assertEqual(Snapshot.from_dict(result.to_dict()), result)

    def test_identity_is_sorted_actual_content_not_git_revision(self):
        self.write("z.py", b"z\r\n")
        self.write("a.py", b"a\n")
        first = self.capture()
        self.assertEqual([f.path for f in first.files], ["a.py", "z.py"])
        manifest = [[f.path, f.size_bytes, f.content_hash] for f in first.files]
        expected = hashlib.sha256(json.dumps(manifest, ensure_ascii=False,
            separators=(",", ":")).encode("utf-8")).hexdigest()
        self.assertEqual(first.content_hash, expected)
        self.assertEqual(first, self.capture())
        self.write("z.py", b"different")
        self.assertNotEqual(first.content_hash, self.capture().content_hash)

    def test_added_removed_modified_and_test_changes(self):
        self.write("old.py")
        self.write("tests/t.py")
        self.write("same.py")
        before = self.capture()
        (self.root / "old.py").unlink()
        self.write("new.py")
        self.write("tests/t.py", b"changed")
        changes = file_changes(before, self.capture())
        self.assertEqual([(a.path if a else None, b.path if b else None) for a, b in changes],
                         [(None, "new.py"), ("old.py", None), ("tests/t.py", "tests/t.py")])
        self.assertEqual(changes[-1][1].category, "tests")

    def test_existing_fixture_identity_and_deleted_file(self):
        base = Path(__file__).parent / "fixtures/python/pairs/duplicated"
        before = capture_scope(base / "baseline", rules())
        after = capture_scope(base / "candidate", rules())
        self.assertIsInstance(before, Snapshot)
        self.assertIsInstance(after, Snapshot)
        self.assertIn("redundant.py", [a.path for a, b in file_changes(before, after) if a and not b])

    def test_unreadable_file_and_missing_root_return_failure(self):
        target = self.write("bad.py")
        read_bytes = Path.read_bytes
        def read(path):
            if path == target:
                raise PermissionError("denied")
            return read_bytes(path)
        with patch.object(Path, "read_bytes", read):
            result = capture_scope(self.root, rules())
        self.assertIsInstance(result, Failure)
        self.assertEqual(result.affected_path, "bad.py")
        self.assertIsInstance(capture_scope(self.root / "missing", rules()), Failure)

    def test_symlink_and_reparse_are_rejected_before_following(self):
        self.write("target.py")
        try:
            (self.root / "link.py").symlink_to(self.root / "target.py")
        except OSError:
            pass  # Windows may not grant symlink creation; reparse probe still runs.
        else:
            self.assertIsInstance(capture_scope(self.root, rules()), Failure)
            (self.root / "link.py").unlink()
        original = Path.lstat
        class Reparse:
            st_file_attributes = stat.FILE_ATTRIBUTE_REPARSE_POINT
            st_mode = stat.S_IFREG | 0o644
        with patch.object(Path, "lstat", lambda path: Reparse() if path.name == "target.py" else original(path)):
            result = capture_scope(self.root, rules())
        self.assertIsInstance(result, Failure)
        self.assertEqual(result.affected_path, "target.py")

    def test_scope_hash_and_empty_capture(self):
        first = self.capture()
        self.assertEqual(first.files, ())
        self.assertNotEqual(first.scope_hash, self.capture(rules(tests=("spec/**",))).scope_hash)

    def test_broad_product_pattern_still_measures_only_python(self):
        self.write("config.json")
        self.write("module.py")
        snapshot = self.capture(rules(product=("**/*",)))
        self.assertEqual([(f.path, f.category) for f in snapshot.files],
                         [("config.json", "support"), ("module.py", "product")])

    def test_case_collision_fails_on_case_sensitive_and_insensitive_hosts(self):
        first = self.write("Case.py")
        second = self.root / "case.py"
        original = Path.lstat
        with patch.object(Path, "iterdir", return_value=iter((first, second))), \
                patch.object(Path, "lstat", lambda p: original(first) if p == second else original(p)):
            result = capture_scope(self.root, rules())
        self.assertIsInstance(result, Failure)
        self.assertIn("case collision", result.message)
        self.assertEqual(result.affected_path, "case.py")

    def test_excluded_bytes_do_not_affect_content_identity(self):
        self.write("main.py")
        output = self.write("runs/report.py", b"old output")
        before = self.capture()
        output.write_bytes(b"new output")
        self.assertEqual(before.content_hash, self.capture().content_hash)

    def test_tracked_working_tree_and_untracked_bytes_are_captured(self):
        subprocess.run(["git", "init", "-q", str(self.root)], check=True, capture_output=True)
        tracked = self.write("tracked.py", b"original")
        subprocess.run(["git", "-C", str(self.root), "add", "tracked.py"], check=True, capture_output=True)
        tracked.write_bytes(b"uncommitted")
        self.write("untracked.py", b"not in index")
        captured = {f.path: f for f in self.capture().files}
        self.assertEqual(set(captured), {"tracked.py", "untracked.py"})
        self.assertEqual(captured["tracked.py"].content_hash, hashlib.sha256(b"uncommitted").hexdigest())
        self.assertEqual(captured["untracked.py"].content_hash, hashlib.sha256(b"not in index").hexdigest())


class DisposableTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.source = self.root / "source"
        self.source.mkdir()
        self.destination = self.root / "disposable"
        self.main = self.source / "main.py"
        self.main.write_bytes(b"original\r\n")
        (self.source / "runs").mkdir()
        (self.source / "runs/report.py").write_bytes(b"excluded")

    def test_copy_preserves_bytes_identity_and_modes(self):
        self.main.chmod(0o755)
        original = capture_scope(self.source, rules())
        result = capture(self.source, rules(), self.destination)
        self.assertIsInstance(result, Snapshot, result)
        self.assertEqual(result.content_hash, original.content_hash)
        self.assertEqual(result.files, original.files)
        self.assertEqual(Path(result.root), self.destination.resolve())
        self.assertEqual((self.destination / "main.py").read_bytes(), b"original\r\n")
        self.assertEqual(stat.S_IMODE((self.destination / "main.py").stat().st_mode),
                         stat.S_IMODE(self.main.stat().st_mode))
        self.assertFalse((self.destination / "runs").exists())

    def test_no_overwrite_or_overlap_with_source(self):
        original = self.main.read_bytes()
        for target in (self.source, self.source / "copy", self.root):
            with self.subTest(target=target):
                self.assertIsInstance(capture(self.source, rules(), target), Failure)
        self.destination.mkdir()
        sentinel = self.destination / "keep"
        sentinel.write_bytes(b"untouched")
        self.assertIsInstance(capture(self.source, rules(), self.destination), Failure)
        self.assertEqual(sentinel.read_bytes(), b"untouched")
        self.assertEqual(self.main.read_bytes(), original)

    def test_copy_race_mutation_addition_and_deletion_fail_and_clean(self):
        original_read = Path.read_bytes
        for mutation in ("modify", "add", "delete"):
            self.main.write_bytes(b"original\r\n")
            extra = self.source / "added.py"
            extra.unlink(missing_ok=True)
            count = 0
            def read(path):
                nonlocal count
                content = original_read(path)
                if path == self.main:
                    count += 1
                    if count == 2:
                        if mutation == "modify":
                            self.main.write_bytes(b"changed")
                        elif mutation == "add":
                            extra.write_bytes(b"new")
                        else:
                            self.main.unlink()
                return content
            with self.subTest(mutation=mutation), patch.object(Path, "read_bytes", read):
                result = capture(self.source, rules(), self.destination)
            self.assertIsInstance(result, Failure, result)
            self.assertFalse(self.destination.exists())

    def test_writable_checks_are_independent_and_cleanup_is_owned(self):
        before = capture_scope(self.source, rules())
        with disposable_workspace(self.source, rules(), self.destination) as workspace:
            self.assertNotIsInstance(workspace, Failure)
            measurement = Path(workspace.measurement.root) / "main.py"
            check = workspace.checks / "main.py"
            self.assertFalse(measurement.stat().st_mode & stat.S_IWUSR)
            self.assertTrue(check.stat().st_mode & stat.S_IWUSR)
            check.write_bytes(b"test side effect")
            (workspace.checks / "new.py").write_bytes(b"generated by check")
            self.assertEqual(measurement.read_bytes(), b"original\r\n")
            self.assertEqual(capture_scope(self.source, rules()), before)
        self.assertFalse(self.destination.exists())
        self.assertTrue(self.source.exists())

    def test_explicit_diagnostic_retention_and_exception_cleanup(self):
        with disposable_workspace(self.source, rules(), self.destination, retain=True) as workspace:
            self.assertNotIsInstance(workspace, Failure)
            (workspace.checks / "diagnostic.txt").write_text("failure evidence")
        self.assertEqual((self.destination / "checks/diagnostic.txt").read_text(), "failure evidence")
        second = self.root / "second"
        with self.assertRaisesRegex(RuntimeError, "check failed"):
            with disposable_workspace(self.source, rules(), second):
                raise RuntimeError("check failed")
        self.assertFalse(second.exists())

    def test_symlink_destination_or_parent_and_source_are_rejected(self):
        original = Path.lstat
        class Reparse:
            st_file_attributes = stat.FILE_ATTRIBUTE_REPARSE_POINT
            st_mode = stat.S_IFDIR | 0o755
        for unsafe in (self.source, self.destination, self.root):
            with self.subTest(unsafe=unsafe), patch.object(Path, "lstat",
                    lambda path: Reparse() if path == unsafe else original(path)):
                self.assertIsInstance(capture(self.source, rules(), self.destination), Failure)
        self.assertEqual(self.main.read_bytes(), b"original\r\n")

    def test_copy_read_error_reports_path_and_removes_partial_copy(self):
        original = Path.read_bytes
        count = 0
        def read(path):
            nonlocal count
            if path == self.main:
                count += 1
                if count == 2:
                    raise PermissionError("copy read denied")
            return original(path)
        with patch.object(Path, "read_bytes", read):
            result = capture(self.source, rules(), self.destination)
        self.assertIsInstance(result, Failure)
        self.assertEqual(result.affected_path, "main.py")
        self.assertIn("copy read denied", result.message)
        self.assertFalse(self.destination.exists())
        self.assertEqual(self.main.read_bytes(), b"original\r\n")

    def test_mode_change_during_copy_fails(self):
        original = Path.read_bytes
        count = 0
        def read(path):
            nonlocal count
            content = original(path)
            if path == self.main:
                count += 1
                if count == 2:
                    self.main.chmod(stat.S_IREAD)
            return content
        try:
            with patch.object(Path, "read_bytes", read):
                result = capture(self.source, rules(), self.destination)
            self.assertIsInstance(result, Failure)
            self.assertEqual(result.affected_path, "main.py")
            self.assertFalse(self.destination.exists())
        finally:
            self.main.chmod(stat.S_IREAD | stat.S_IWRITE)

    def test_cleanup_refuses_replaced_root(self):
        moved = self.root / "moved-owned-copy"
        with self.assertRaisesRegex(OSError, "refusing cleanup"):
            with disposable_workspace(self.source, rules(), self.destination) as workspace:
                self.assertNotIsInstance(workspace, Failure)
                self.destination.rename(moved)
                self.destination.mkdir()
                (self.destination / "unrelated").write_bytes(b"keep")
        self.assertEqual((self.destination / "unrelated").read_bytes(), b"keep")
        # Restore for the test fixture's ordinary read-only-file cleanup.
        for path in moved.rglob("*"):
            if path.is_file():
                path.chmod(stat.S_IREAD | stat.S_IWRITE)

    def test_cleanup_refuses_reparse_entries_without_removing_source(self):
        original = Path.lstat
        suspect = None
        class Reparse:
            st_file_attributes = stat.FILE_ATTRIBUTE_REPARSE_POINT
            st_mode = stat.S_IFREG | 0o644
        with self.assertRaisesRegex(OSError, "refusing cleanup"):
            with patch.object(Path, "lstat",
                    lambda path: Reparse() if path == suspect else original(path)):
                with disposable_workspace(self.source, rules(), self.destination) as workspace:
                    self.assertNotIsInstance(workspace, Failure)
                    suspect = workspace.checks / "main.py"
        self.assertEqual(self.main.read_bytes(), b"original\r\n")
        for path in self.destination.rglob("*"):
            if path.is_file():
                path.chmod(stat.S_IREAD | stat.S_IWRITE)


if __name__ == "__main__":
    unittest.main()
