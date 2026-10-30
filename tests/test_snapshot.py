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
from lesscode_baseline.snapshot import capture_scope, file_changes, matches


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


if __name__ == "__main__":
    unittest.main()
