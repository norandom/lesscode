"""Tests for editscope.py (task 5.1): identical trees, an added file, a differing scope file only.
Run from this directory: python -m pytest test_editscope.py -q"""
from pathlib import Path

from editscope import check

SCOPE = ["src/facdrone/shells/a.py"]


def tree(root: Path, scope_text: str = "x = 1\n", extra: dict | None = None) -> Path:
    (root / "src/facdrone/shells").mkdir(parents=True)
    (root / "tests").mkdir()
    (root / "packages/p").mkdir(parents=True)
    (root / "pyproject.toml").write_text("[project]\n", encoding="utf-8")
    (root / "src/facdrone/shells/a.py").write_text(scope_text, encoding="utf-8")
    (root / "src/facdrone/other.py").write_text("y = 2\n", encoding="utf-8")
    (root / "tests/test_a.py").write_text("def test(): pass\n", encoding="utf-8")
    (root / "packages/p/__init__.py").write_text("", encoding="utf-8")
    (root / "src/__pycache__").mkdir()
    (root / "src/__pycache__/junk.cpython-313.pyc").write_bytes(b"\x00")  # must be skipped
    for rel, text in (extra or {}).items():
        (root / rel).parent.mkdir(parents=True, exist_ok=True)
        (root / rel).write_text(text, encoding="utf-8")
    return root


def test_identical_trees_pass_edit_scope_but_scope_unchanged(tmp_path):
    r = check(tree(tmp_path / "o"), tree(tmp_path / "c"), SCOPE)
    assert r["edit_scope"] == "pass"  # nothing added, removed or changed outside the scope (req 6.4, 6.5)
    assert r["differing"] == [] and r["only_in_original"] == [] and r["only_in_candidate"] == []
    assert r["verdict"] == "fail" and r["scope_unchanged"] == SCOPE  # no regeneration happened
    assert r["files_compared"] == 5 and "src/__pycache__/junk.cpython-313.pyc" not in r["candidate_hashes"]


def test_extra_file_in_candidate_fails(tmp_path):
    r = check(tree(tmp_path / "o"), tree(tmp_path / "c", extra={"tests/test_new.py": "pass\n"}), SCOPE)
    assert r["edit_scope"] == "fail" and r["verdict"] == "fail"
    assert r["only_in_candidate"] == ["tests/test_new.py"] and r["only_in_original"] == []


def test_differing_scope_file_only_passes(tmp_path):
    r = check(tree(tmp_path / "o"), tree(tmp_path / "c", scope_text="x = 2\n"), SCOPE)
    assert r["edit_scope"] == "pass" and r["verdict"] == "pass"
    assert r["differing"] == SCOPE and r["differing_outside_scope"] == [] and r["scope_unchanged"] == []


def test_changed_file_outside_scope_fails(tmp_path):
    c = tree(tmp_path / "c", scope_text="x = 2\n")
    (c / "src/facdrone/other.py").write_text("y = 3\n", encoding="utf-8")
    r = check(tree(tmp_path / "o"), c, SCOPE)
    assert r["edit_scope"] == "fail" and r["verdict"] == "fail"
    assert r["differing_outside_scope"] == ["src/facdrone/other.py"]


def test_mentions_outside_scope(tmp_path):
    from editscope import mentions
    t = tmp_path / "log.txt"
    t.write_text("changed src/facdrone/shells/a.py and also helpers/b.py; a.py again", encoding="utf-8")
    assert mentions([t], SCOPE) == ["helpers/b.py"]


def test_removed_file_fails(tmp_path):  # added after the 2026-11-06 review: the only_in_original branch
    c = tree(tmp_path / "c", scope_text="x = 2\n")
    (c / "tests/test_a.py").unlink()
    r = check(tree(tmp_path / "o"), c, SCOPE)
    assert r["edit_scope"] == "fail" and r["verdict"] == "fail"
    assert r["only_in_original"] == ["tests/test_a.py"] and r["only_in_candidate"] == []
