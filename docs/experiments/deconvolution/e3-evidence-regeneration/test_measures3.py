"""Hand-counted separated-form statement counts for measures3.py (deconvolution-loop task 1.3), rule version 1."""
import ast
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import measures3  # noqa: E402

CASES = [  # (source, separated count, extras by rule), counted by hand under rule 1
    ("x = y\n", 1, {}),
    ("import a, b\n", 2, {"import": 1}),
    ("import a\n", 1, {}),
    ("from m import a, b, c as c\n", 3, {"from_import": 2}),
    ("from m import *\n", 1, {}),
    ("a, b = 1, 2\n", 2, {"tuple_assign": 1}),
    ("[a, (b, c)] = 1, (2, 3)\n", 3, {"tuple_assign": 2}),
    ("a, b = f()\n", 1, {}),
    ("a, *b = 1, 2, 3\n", 1, {}),
    ("a = b = 0\n", 2, {"multi_target": 1}),
    ("x += 1\ny: int = 2\n", 2, {}),
    ("if 0 < x < 1:\n    pass\n", 3, {"chained_compare": 1}),
    ("ok = a < b <= c < d\n", 3, {"chained_compare": 2}),
    ("ok = a and b or c\n", 1, {}),
    ("del a, b\n", 2, {"del": 1}),
    ("def f():\n    global a, b\n", 3, {"global_nonlocal": 1}),
    ("with p as x, q as y:\n    pass\n", 3, {"with_items": 1}),
    ("for a, b in z:\n    pass\n", 2, {}),
    ("def f(a, b):\n    return a, b\n", 2, {}),
]


@pytest.mark.parametrize("src,n,extra", CASES)
def test_hand_counted(src, n, extra):
    tree = ast.parse(src)
    got = measures3.separated(tree)
    assert got["stmts"] == n
    assert {k: v for k, v in got["extra"].items() if v} == extra
    assert got["stmts"] == sum(isinstance(x, ast.stmt) for x in ast.walk(tree)) + sum(got["extra"].values())


def test_rule_is_declared_and_versioned():
    assert measures3.RULE["version"] == "1"
    assert all(f"{i}." in measures3.RULE["text"] for i in range(1, 10))
