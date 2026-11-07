"""ScopeMeasurer suite (constraint-guided-compression tasks 3.1-3.4).

Offline synthetic-tree tests: as-written measures, EffectProfile structure
validation, statement-level pure share, separated-form v1, canonical
unavailability, edit scope, and measure gaps.
"""
from __future__ import annotations

import textwrap
from pathlib import Path

import pytest

from lesscode_baseline import measures


@pytest.fixture()
def tree(tmp_path: Path) -> Path:
    (tmp_path / "src" / "pkg").mkdir(parents=True)
    (tmp_path / "src" / "pkg" / "mod_a.py").write_text(
        textwrap.dedent(
            """\
            import os

            def pure_fn(a, b):
                x = a + b
                return x

            def shell_fn(conn):
                conn.execute("select 1")
                return 2
            """
        ),
        encoding="utf-8",
    )
    (tmp_path / "src" / "pkg" / "mod_b.py").write_text(
        textwrap.dedent(
            """\
            from pkg import mod_a

            def caller():
                return mod_a.pure_fn(1, 2)
            """
        ),
        encoding="utf-8",
    )
    return tmp_path


SCOPE = ("src/pkg/mod_a.py", "src/pkg/mod_b.py")
STRUCTURE = {
    "pure": ["mod_a.py::pure_fn"],
    "shell": ["mod_a.py::shell_fn", "mod_b.py::caller"],
}


class TestMeasureWritten:
    def test_counts(self, tree: Path) -> None:
        report = measures.measure_written(tree, SCOPE)
        assert report["files"]["mod_a.py"]["lines"] == 9
        assert report["files"]["mod_a.py"]["stmts"] == 7
        assert report["files"]["mod_a.py"]["funcs"] == 2
        assert report["files"]["mod_a.py"]["qualnames"] == 2
        assert report["files"]["mod_a.py"]["writes"] == 1
        assert report["total"]["stmts"] == 7 + 3  # mod_b: ImportFrom + FunctionDef + Return

    def test_edges_and_module_hops(self, tree: Path) -> None:
        report = measures.measure_written(tree, SCOPE)
        assert report["total"]["edges"] == 1
        assert report["total"]["module_hops"] == 1
        assert report["edges"] == [["mod_b.py", "caller", "mod_a.py", "pure_fn"]]

    def test_write_site_attribution(self, tree: Path) -> None:
        report = measures.measure_written(tree, SCOPE)
        site = report["write_sites"][0]
        assert site["file"] == "mod_a.py"
        assert site["in"] == "mod_a.py::shell_fn"

    def test_structure_pass(self, tree: Path) -> None:
        report = measures.measure_written(tree, SCOPE, structure=STRUCTURE)
        assert report["structure"]["pass"] is True
        assert report["structure"]["undeclared"] == []
        assert report["structure"]["pure"]["mod_a.py::pure_fn"]["pass"] is True

    def test_bad_param_in_pure(self, tree: Path) -> None:
        structure = {"pure": ["mod_a.py::shell_fn"], "shell": []}
        report = measures.measure_written(tree, SCOPE, structure=structure)
        why = report["structure"]["pure"]["mod_a.py::shell_fn"]["why"]
        assert any("param conn" in w for w in why)

    def test_sql_attr_call_in_pure(self, tree: Path) -> None:
        (tree / "src" / "pkg" / "mod_c.py").write_text(
            "def sneaky():\n    return db.commit()\n", encoding="utf-8"
        )
        structure = {"pure": ["mod_c.py::sneaky"], "shell": []}
        report = measures.measure_written(tree, ("src/pkg/mod_c.py",), structure=structure)
        why = report["structure"]["pure"]["mod_c.py::sneaky"]["why"]
        assert any("call .commit" in w for w in why)
        assert report["structure"]["pass"] is False

    def test_write_outside_shell(self, tree: Path) -> None:
        structure = {"pure": ["mod_a.py::pure_fn"], "shell": []}
        report = measures.measure_written(tree, ("src/pkg/mod_a.py",), structure=structure)
        assert report["structure"]["pass"] is False
        assert any(site["pass"] is False for site in report["structure"]["write_sites"])

    def test_undeclared_and_unknown(self, tree: Path) -> None:
        structure = {"pure": ["mod_a.py::pure_fn"], "shell": ["mod_a.py::ghost"]}
        report = measures.measure_written(tree, ("src/pkg/mod_a.py",), structure=structure)
        assert report["structure"]["undeclared"] == ["mod_a.py::shell_fn"]
        assert report["structure"]["declared_unknown"] == ["mod_a.py::ghost"]


class TestEffectProfile:
    def test_custom_profile_patterns(self, tree: Path) -> None:
        (tree / "src" / "pkg" / "mod_d.py").write_text(
            "def keeper(db):\n    store.save()\n    return 1\n", encoding="utf-8"
        )
        profile = measures.EffectProfile(
            name="toy",
            write_pattern=r"\.save\(",
            bad_param_pattern=r"^db$",
            sql_attr_names=frozenset({"save"}),
        )
        structure = {"pure": ["mod_d.py::keeper"], "shell": []}
        report = measures.measure_written(
            tree, ("src/pkg/mod_d.py",), structure=structure, effect_profile=profile
        )
        assert report["files"]["mod_d.py"]["writes"] == 1
        why = report["structure"]["pure"]["mod_d.py::keeper"]["why"]
        assert any("param db" in w for w in why)
        assert any("call .save" in w for w in why)

    def test_default_profile_is_sqlalchemy(self) -> None:
        assert "execute" in measures.DEFAULT_EFFECT_PROFILE.sql_attr_names
        assert measures.DEFAULT_EFFECT_PROFILE.name == "sqlalchemy"


class TestMeasurePureShare:
    def test_statement_level_attribution(self, tree: Path) -> None:
        share = measures.measure_pure_share(tree, SCOPE, STRUCTURE)
        stmts = share["stmts"]
        assert stmts["pure"] == 3  # FunctionDef, Assign, Return of pure_fn
        assert stmts["shell"] == 5  # mod_a shell_fn x3 + mod_b caller FunctionDef + Return
        assert stmts["undeclared"] == 0
        assert stmts["outside_defs"] == 2  # the two module-level import statements
        assert share["stmts_in_defs"] == 8
        assert share["pure_share_by_stmts_in_defs"] == 0.375
        assert share["pure_share_by_stmts_all"] == 0.3
        assert share["per_function"]["mod_a.py::pure_fn"] == 3

    def test_undeclared_statements_counted(self, tree: Path) -> None:
        structure = {"pure": ["mod_a.py::pure_fn"], "shell": []}
        share = measures.measure_pure_share(tree, ("src/pkg/mod_a.py",), structure)
        assert share["stmts"]["undeclared"] == 3

    def test_function_share(self, tree: Path) -> None:
        share = measures.measure_pure_share(tree, ("src/pkg/mod_a.py",), STRUCTURE)
        assert share["pure_share_by_funcs"] == 0.5
