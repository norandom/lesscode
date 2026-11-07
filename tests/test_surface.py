"""SurfaceDeriver suite (constraint-guided-compression tasks 2.1-2.3).

Offline synthetic-tree tests: Contract A (tree scan) vs Contract B (recorder
imports join the contract), unresolved-vs-missing semantics, check_syntax
line anchoring, and candidate verification violation classes.
"""
from __future__ import annotations

import textwrap
from pathlib import Path

import pytest

from lesscode_baseline import surface


@pytest.fixture()
def tree(tmp_path: Path) -> Path:
    """A minimal specimen: one scope module, one importer, one test, one recorder."""
    (tmp_path / "src" / "pkg").mkdir(parents=True)
    (tmp_path / "tests").mkdir()
    (tmp_path / "tools").mkdir()
    (tmp_path / "src" / "pkg" / "__init__.py").write_text("", encoding="utf-8")
    (tmp_path / "src" / "pkg" / "mod.py").write_text(
        textwrap.dedent(
            """\
            def used_fn() -> int:
                return 1

            class Used:
                pass

            USED_CONST = 7

            def unused_fn() -> int:
                return 2
            """
        ),
        encoding="utf-8",
    )
    (tmp_path / "src" / "pkg" / "other.py").write_text(
        textwrap.dedent(
            """\
            from pkg.mod import used_fn, USED_CONST
            import pkg.mod as pm

            pm.Used()
            pm.ghost_attr()
            setattr(pm, "patched_target", 1)
            """
        ),
        encoding="utf-8",
    )
    (tmp_path / "tests" / "test_mod.py").write_text(
        "from pkg.mod import used_fn\n\ndef test_it():\n    assert used_fn() == 1\n",
        encoding="utf-8",
    )
    (tmp_path / "tools" / "recorder.py").write_text(
        "from pkg.mod import labeled_reason\n\ndef rec():\n    return labeled_reason\n",
        encoding="utf-8",
    )
    return tmp_path


SCOPE = ("src/pkg/mod.py",)


def mod_name() -> str:
    return "pkg.mod"


class TestDeriveSurfaceContractA:
    def test_kinds_and_tags(self, tree: Path) -> None:
        report = surface.derive_surface(tree, SCOPE)
        names = report["modules"][mod_name()]["names"]
        assert names["used_fn"]["kind"] == "function"
        assert names["Used"]["kind"] == "class"
        assert names["USED_CONST"]["kind"] == "constant"
        assert names["used_fn"]["tags"] == ["prod", "test"]
        assert names["used_fn"]["via"] == ["import"]
        assert names["Used"]["via"] == ["attribute"]
        assert names["patched_target"]["via"] == ["patched"]

    def test_files_scanned_and_importers(self, tree: Path) -> None:
        report = surface.derive_surface(tree, SCOPE)
        assert report["files_scanned"] == 4  # __init__, mod, other (src) + test_mod (tests); tools/ is not a scan dir
        assert "src/pkg/other.py" in report["modules"][mod_name()]["importers"]
        assert report["modules"][mod_name()]["importers"]["src/pkg/other.py"] == "prod"
        assert report["modules"][mod_name()]["importers"]["tests/test_mod.py"] == "test"

    def test_unresolved_records_missing_attribute_reads(self, tree: Path) -> None:
        report = surface.derive_surface(tree, SCOPE)
        assert report["unresolved"][mod_name()].get("ghost_attr") == ["src/pkg/other.py"]

    def test_recorder_dir_not_scanned_by_default(self, tree: Path) -> None:
        report = surface.derive_surface(tree, SCOPE)
        assert "labeled_reason" not in report["modules"][mod_name()]["names"]

    def test_extractor_version_recorded(self, tree: Path) -> None:
        report = surface.derive_surface(tree, SCOPE)
        assert report["extractor_version"] == surface.EXTRACTOR_VERSION


class TestDeriveSurfaceContractB:
    def test_recorder_import_joins_contract(self, tree: Path) -> None:
        report = surface.derive_surface(tree, SCOPE, recorder_files=("tools/recorder.py",))
        entry = report["modules"][mod_name()]["names"]["labeled_reason"]
        assert entry["kind"] == "missing"
        assert "recorder" in entry["tags"]
        assert entry["importers"] == ["tools/recorder.py"]

    def test_recorder_files_listed(self, tree: Path) -> None:
        report = surface.derive_surface(tree, SCOPE, recorder_files=("tools/recorder.py",))
        assert report["recorder_files"] == ["tools/recorder.py"]

    def test_intra_set_reads_among_scope_files(self, tree: Path) -> None:
        (tree / "src" / "pkg" / "mod2.py").write_text(
            "from pkg.mod import used_fn\n", encoding="utf-8"
        )
        report = surface.derive_surface(tree, ("src/pkg/mod.py", "src/pkg/mod2.py"))
        assert "used_fn" in report["intra_set"]["pkg.mod"]


class TestCheckSyntax:
    def test_clean_tree(self, tree: Path) -> None:
        assert surface.check_syntax(tree, SCOPE) == []

    def test_line_anchored_errors(self, tree: Path) -> None:
        (tree / "src" / "pkg" / "mod.py").write_text("def f(:\n    pass\n", encoding="utf-8")
        errors = surface.check_syntax(tree, SCOPE)
        assert len(errors) == 1
        assert errors[0].startswith("src/pkg/mod.py:1:")

    def test_multiple_files(self, tree: Path) -> None:
        (tree / "src" / "pkg" / "mod.py").write_text("def f(:\n", encoding="utf-8")
        (tree / "src" / "pkg" / "mod2.py").write_text("def g(:\n", encoding="utf-8")
        errors = surface.check_syntax(tree, ("src/pkg/mod.py", "src/pkg/mod2.py"))
        assert len(errors) == 2

    def test_unsupported_language_is_explicit(self, tree: Path) -> None:
        errors = surface.check_syntax(tree, SCOPE, language="typescript")
        assert errors == ["check_syntax: unsupported language 'typescript'"]

    def test_missing_file_reported(self, tree: Path) -> None:
        errors = surface.check_syntax(tree, ("src/pkg/absent.py",))
        assert any("src/pkg/absent.py" in e for e in errors)

    def test_never_imports_candidate(self, tree: Path) -> None:
        (tree / "src" / "pkg" / "mod.py").write_text(
            "import os\nraise SystemExit('must not execute')\n", encoding="utf-8"
        )
        assert surface.check_syntax(tree, SCOPE) == []
