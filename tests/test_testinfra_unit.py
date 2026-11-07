"""TestInfraDetector suite (constraint-guided-compression tasks 4.1-4.2).

Ports the frozen synthetic-repository tests for the demonstrated detector
(deconvolution-loop task 2.2) to the relocated package module, and adds the
cross-language envelope assertions (rule_version; unknown runner on
non-Python layouts, R32).
"""
import json
import textwrap
from pathlib import Path

import pytest

from lesscode_baseline import testinfra

FILES = {
    "pyproject.toml": """
        [tool.pytest.ini_options]
        testpaths = ["tests"]
        pythonpath = ["src"]
        addopts = "-p pkg_testing.extra"
        markers = [
            "slow: takes long",
            "live_net: reaches the public network. Opt-in: `pytest -m live_net`.",
            "db: needs PostgreSQL; skips off a configured host",
        ]
        """,
    "src/pkg/__init__.py": "",
    "src/pkg/core.py": "X = 1\ndef f():\n    return X\nclass K:\n    pass\n",
    "src/pkg/other.py": "def g():\n    return 2\n",
    "src/pkg_testing/__init__.py": "",
    "src/pkg_testing/helpers.py": "from pkg.core import f\ndef h():\n    return f()\n",
    "src/pkg_testing/extra.py": "",
    "src/pkg_testing/plugin.py": """
        import os
        import pytest
        import pgembed
        DSN_ENV = "APP_TEST_PG_DSN"
        def pytest_configure(config):
            if os.geteuid() == 0:
                return
            os.environ.get(DSN_ENV)
        @pytest.fixture(scope="session")
        def pg_engine():
            return os.getenv("APP_TEST_PG_DSN")
        """,
    "tests/conftest.py": """
        import pytest
        pytest_plugins = ["pkg_testing.plugin"]
        @pytest.fixture
        def thing():
            return 1
        """,
    "tests/test_direct.py": "from pkg.core import f\ndef test_a():\n    assert f() == 1\n",
    "tests/test_alias.py": """
        import pkg.core as c
        def test_b(monkeypatch):
            monkeypatch.setattr(c, "f", lambda: 0)
            assert c.X == 1
        """,
    "tests/test_patchstr.py": """
        def test_c(monkeypatch):
            monkeypatch.setattr("pkg.core.f", lambda: 0)
        """,
    "tests/test_helper.py": "from pkg_testing.helpers import h\ndef test_d():\n    assert h() == 1\n",
    "tests/test_none.py": "from pkg.other import g\ndef test_e():\n    assert g() == 2\n",
    "packages/sub/pyproject.toml": '[tool.pytest.ini_options]\ntestpaths = ["tests"]\n',
    "packages/sub/tests/test_sub.py": "from pkg import core\ndef test_f():\n    assert core.K\n",
    "scripts/gates.sh": 'python -m pytest tests -q -m "not live_net" --ignore=tests/test_none.py\n',
}
FAILS = {"env": ["tests.test_direct::test_a", "tests.test_none::test_e", "::tests.test_broken"],
         "other": ["tests.test_sub::test_f", "tests.test_missing::test_x"]}


@pytest.fixture
def repo(tmp_path: Path) -> Path:
    for rel, text in FILES.items():
        p = tmp_path / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(textwrap.dedent(text).lstrip("\n"), encoding="utf-8")
    (tmp_path / "tests" / "test_broken.py").write_text("def test_z(:\n", encoding="utf-8")
    (tmp_path / "fails.json").write_text(json.dumps(FAILS), encoding="utf-8")
    return tmp_path


def detect(repo: Path) -> dict:
    out = repo / "out"
    rc = testinfra.main([str(repo), "--scope", "src/pkg/core.py", "--baseline-failures", str(repo / "fails.json"),
                         "--out", str(out)])
    assert rc == 0
    assert (out / "testinfra.md").read_text(encoding="utf-8").startswith("# Test infrastructure")
    return json.loads((out / "testinfra.json").read_text(encoding="utf-8"))


def test_runner_config_markers_and_rootdirs(repo):
    d = detect(repo)
    r = d["runner"]
    assert r["name"] == "pytest"
    roots = {c["rootdir"]: c for c in r["configs"]}
    assert set(roots) == {".", "packages/sub"}
    assert roots["."]["file"] == "pyproject.toml" and roots["."]["testpaths"] == ["tests"]
    assert set(roots["."]["markers"]) == {"slow", "live_net", "db"}
    assert "pkg_testing.extra" in r["plugins"]


def test_fixtures_plugins_and_external_store(repo):
    d = detect(repo)
    f = d["fixtures"]
    assert f["conftest"]["tests/conftest.py"] == ["thing"]
    assert f["pytest_plugins"] == {"tests/conftest.py": ["pkg_testing.plugin"]}
    assert f["plugin_modules"]["pkg_testing.plugin"]["fixtures"] == ["pg_engine"]
    store = d["external_store"]
    kinds = {(x["kind"], x["detail"]) for x in store["facts"]}
    assert ("import", "pgembed") in kinds
    assert ("env", "APP_TEST_PG_DSN") in kinds
    assert ("fixture", "pg_engine") in kinds
    assert any(h["detail"] == "os.geteuid" and h["file"] == "src/pkg_testing/plugin.py" for h in store["host_hazards"])


def test_scope_references_with_kinds(repo):
    d = detect(repo)
    refs = d["scope_references"]["test_files"]
    assert set(refs) == {"tests/test_direct.py", "tests/test_alias.py", "tests/test_patchstr.py",
                         "packages/sub/tests/test_sub.py"}
    assert refs["tests/test_direct.py"]["pkg.core"] == {"f": ["import"]}
    assert refs["tests/test_alias.py"]["pkg.core"] == {"<module>": ["module"], "X": ["attribute"], "f": ["patched"]}
    assert refs["tests/test_patchstr.py"]["pkg.core"] == {"f": ["patched"]}
    assert "src/pkg_testing/helpers.py" in d["scope_references"]["support_files"]
    assert d["scope_references"]["parse_errors"] == ["tests/test_broken.py"]


def test_seeds_deselections_and_marker_expression(repo):
    d = detect(repo)
    s = d["seeds"]
    assert s["direct"] == sorted(["packages/sub/tests/test_sub.py", "tests/test_alias.py", "tests/test_direct.py",
                                  "tests/test_patchstr.py"])
    assert s["via_support"] == ["tests/test_helper.py"]
    assert "tests/test_none.py" not in s["recommended"]
    assert len(s["recommended"]) == 5
    x = d["deselections"]
    assert x["total"] == 4 and x["collection_errors"] == ["tests/test_broken.py"]
    assert x["unresolved"] == ["tests.test_missing::test_x"]
    assert x["applicable"] == {"env": ["tests/test_direct.py::test_a"], "other": ["packages/sub/tests/test_sub.py::test_f"]}
    m = d["marker_expression"]
    assert m["suggested"] == "not live_net"
    assert m["excluded"] == {"live_net": ["opt-in", "resource:network"]}
    assert m["gate_expressions"][0]["expression"] == "not live_net"
    assert m["gate_expressions"][0]["ignores"] == ["tests/test_none.py"]


def test_cross_language_envelope_carries_rule_version(repo):
    d = detect(repo)
    for key in ("runner", "fixtures", "external_store", "scope_references", "seeds",
                "marker_expression", "deselections", "rule_version"):
        assert key in d, key
    assert d["rule_version"] == testinfra.RULE_VERSION


def test_non_python_layout_returns_unknown_runner_cleanly(tmp_path: Path):
    (tmp_path / "src").mkdir()
    (tmp_path / "tests").mkdir()
    (tmp_path / "Cargo.toml").write_text("[package]\nname = 'toy'\n", encoding="utf-8")
    (tmp_path / "src" / "main.rs").write_text("fn main() {}\n", encoding="utf-8")
    (tmp_path / "tests" / "integration.rs").write_text("#[test]\nfn t() {}\n", encoding="utf-8")
    out = tmp_path / "out"
    rc = testinfra.main([str(tmp_path), "--scope", "src/main.rs", "--out", str(out)])
    assert rc == 0
    d = json.loads((out / "testinfra.json").read_text(encoding="utf-8"))
    assert d["runner"]["name"] == "unknown"
    assert d["test_files_total"] == 0
    assert d["seeds"]["counts"]["recommended"] == 0
    assert d["rule_version"] == testinfra.RULE_VERSION
