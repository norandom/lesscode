"""Generator test for missing_tests.py (deconvolution-loop task 2.1) on a small synthetic fixture."""
import hashlib
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import missing_tests  # noqa: E402

CAND = "def f(x):\n    if x != 1:\n        return 0\n    return 1\n\n\ndef g(y: int | None):\n    def inner():\n        return y == 2\n    return inner()\n\n\ndef cap(c):\n    a = dedup(c)\n    return a\n"
ORIG = "def f(x):\n    # original\n    if x != 1:\n        return 0\n    return 1\n\n\ndef cap(c):\n    r = route(c)\n    a = dedup(c)\n    return a\n"


def fixture():
    sample = {"candidate_sha256": hashlib.sha256(CAND.encode()).hexdigest(), "module": "m.py", "sample": [
        {"operator": "core/NotEq_Is", "row": 2, "col": 9, "hunks": [[1, 2, ["    if x is 1:"]]], "old": [["    if x != 1:"]]},
        {"operator": "core/BitOr_Add", "row": 7, "col": 14, "hunks": [[6, 7, ["def g(y: int + None):"]]], "old": [["def g(y: int | None):"]]},
        {"operator": "core/Eq_IsNot", "row": 9, "col": 16, "hunks": [[8, 9, ["        return y is not 2"]]], "old": [["        return y == 2"]]},
        {"operator": "core/AddNot", "row": 2, "col": 7, "hunks": [[1, 2, ["    if not x != 1:"]]], "old": [["    if x != 1:"]]},
    ]}
    facts = [{"step": "2"}, {"t4": {"survivors": [0, 1, 2], "post_hoc_excluding_equivalent": "2/3 (excludes 1; labelled post hoc)"}}]
    replay = [{"log": "ctl.log", "k_dedup": "a", "c_dedup": "b", "differ": True},
              {"log": "ch-cap.log", "args": "{'c': 1}", "o_cap": "(('raise', 'TypeError', 'bytes'), [A])", "c_cap": "(('raise', 'TypeError', 'bytes'), [])", "differ": True}]
    soundness = {"cap": {"calls": 4, "mismatches": [
        {"args": "x", "o": "(('raise', 'ValueError', 'nan'), [A])", "c": "(('raise', 'ValueError', 'nan'), [])"},
        {"args": "y", "o": "(('raise', 'TypeError', 'bytes'), [A])", "c": "(('raise', 'TypeError', 'bytes'), [])"},
        {"args": "z", "o": "(('raise', 'ValueError', 'inf'), [A])", "c": "(('raise', 'ValueError', 'inf'), [])"}]}}
    readings = {"module": "m.py", "survivors": {
        "0": {"class": "real", "original_function": "f", "original_anchor": "x != 1", "entry": ["f"], "observable": "return", "input_class": "x equal to 1 but not identical"},
        "1": {"class": "equivalent", "reason": "annotation only"},
        "2": {"class": "closed_assumption", "reason": "interned", "input_class": "non-identical 2"}},
        "differential": {"cap": {"hypothesis": "H4", "function": "cap", "anchor": "dedup(c)", "original_function": "cap",
                                 "original_anchor": "dedup(c)", "entry": ["cap"], "condition": "order", "observable": "absences",
                                 "input_class": "raising input"}}}
    return sample, facts, replay, soundness, readings


def test_items_sections_and_locations():
    r = missing_tests.build(*fixture(), CAND, ORIG)
    assert [i["id"] for i in r["real"]] == ["T4-0", "T5-cap"]
    t4 = r["real"][0]
    assert (t4["function"], t4["line"], t4["source"]) == ("f", 2, "T4 survivor")
    assert (t4["written"], t4["mutated"]) == ("if x != 1:", "if x is 1:")
    assert (t4["original_function"], t4["original_line"]) == ("f", 3)
    assert t4["input_class"] == "x equal to 1 but not identical"
    d = r["real"][1]
    assert (d["function"], d["line"], d["original_line"]) == ("cap", 14, 10)
    assert d["source"] == "T5 counterexample (ch-cap.log) + soundness corpus (3/4 differ)"
    assert d["corpus_shapes"] == {"TypeError": 1, "ValueError": 2}
    assert d["replay"]["differ"] is True
    assert [(i["id"], i["function"], i["reason"]) for i in r["equivalent"]] == [("T4-1", "g", "annotation only")]
    c = r["closed_assumption"][0]
    assert (c["id"], c["function"], c["line"]) == ("T4-2", "inner", 9)  # innermost enclosing def
    assert r["controls_excluded"] == ["ctl.log"]


def test_post_hoc_list_must_match_readings():
    sample, facts, replay, soundness, readings = fixture()
    facts[-1]["t4"]["post_hoc_excluding_equivalent"] = "2/3 (excludes 2; labelled post hoc)"
    try:
        missing_tests.build(sample, facts, replay, soundness, readings, CAND, ORIG)
    except ValueError as e:
        assert "post hoc" in str(e)
    else:
        raise AssertionError("mismatch between receipt and readings not detected")


def test_markdown_has_three_sections():
    md = missing_tests.render(missing_tests.build(*fixture(), CAND, ORIG))
    assert md.index("## Real gaps") < md.index("## Equivalent by inspection") < md.index("## Equivalent only under a closed assumption")
    assert "t4-survivor-readings.json" in md
