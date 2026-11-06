"""Last note wins on a rerun; E3 exits map as its README table; rate and post hoc reading by the stated formulas."""
import t32_summary as t


def test_last_class_note_wins_and_rate_formula():
    receipt = {"notes": [
        {"key": "t31-m0-class", "value": {"k": 0, "classification": "killed"}},
        {"key": "t31-m1-class", "value": {"k": 1, "classification": "killed"}},   # interrupted run
        {"key": "t31-m1-class", "value": {"k": 1, "classification": "survived"}},  # rerun supersedes
        {"key": "t31-m2-class", "value": {"k": 2, "classification": "incompetent"}},
        {"key": "t31-m2-pre", "value": {"identical": True}},
    ], "commands": []}
    a = t.arm("o", t.original_classes(receipt), {"survivors": {"1": {"class": "equivalent"}, "7": {"class": "equivalent"}}})
    assert a["counts"] == {"killed": 1, "survived": 1, "incompetent": 1}
    assert a["rate"] == 0.5 and a["meets_bar"] is False
    assert a["missing"] == list(range(3, 30))
    assert a["post_hoc"]["equivalent_by_inspection"] == [1] and a["post_hoc"]["rate"] == 1.0  # 7 is not a survivor here


def test_candidate_exit_mapping_and_rerun():
    receipt = {"commands": [{"label": "t4-m0", "exit": 1}, {"label": "t4-m1", "exit": 0}, {"label": "t4-m2", "exit": 2},
                            {"label": "t4-m3", "exit": 1}, {"label": "t4-m3-rerun", "exit": 0}, {"label": "t4-m3-guard", "exit": 0}], "notes": []}
    c = t.candidate_classes(receipt)
    assert {k: v["classification"] for k, v in c.items()} == {0: "killed", 1: "survived", 2: "incompetent", 3: "survived"}
    a = t.arm("c", c, {"survivors": {}})
    assert a["rate"] == round(1 / 3, 3) and a["post_hoc"]["rate"] == a["rate"]
