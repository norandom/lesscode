"""Compare testinfra.py output on Facdrone with the hand-built E1b/E3 lists (task 2.2 validation).

  compare.py E3_SCOPE_JSON E1B_SCOPE_JSON OUT_JSON

Hand lists (read-only): E1b results/selection-map.json (191 coverage seeds, 236 deselections),
results/seeds-static-candidates.txt (115), E3 results/t4-mutant-sample.json (19 importing files).
For files where detection and coverage disagree, a static cause hint is computed from the frozen
copy: excluded markers, and the fixture plugin's operator-DB skip (a `db`-marked or `*_e2e` test
that does not request `pg_engine` directly is skipped on the embedded server). The hint reads the
`pg_engine` request from the test signature only, so a transitive request is missed (approximate).
"""
import ast
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
DECONV = HERE.parent
E1B, E3 = DECONV / "e1b-facdrone-evidence-publication" / "results", DECONV / "e3-evidence-regeneration" / "results"
FROZEN = DECONV.parents[2] / ".runs" / "e2-facdrone" / "source"
EXCLUDED = {"port_baseline", "live_broker"}


def marks(node) -> set[str]:
    out = set()
    for d in getattr(node, "decorator_list", []):
        d = d.func if isinstance(d, ast.Call) else d
        if isinstance(d, ast.Attribute) and isinstance(d.value, ast.Attribute) and d.value.attr == "mark":
            out.add(d.attr)
    return out


def hint(rel: str) -> dict:
    tree = ast.parse((FROZEN / rel).read_text(encoding="utf-8"))
    module = set()
    for n in tree.body:
        if isinstance(n, ast.Assign) and any(getattr(t, "id", "") == "pytestmark" for t in n.targets):
            module |= {getattr(x, "attr", "") for x in ast.walk(n.value) if isinstance(x, ast.Attribute)}
    e2e = Path(rel).stem.endswith("_e2e")
    tests, excluded, skipped = 0, 0, 0
    for n in ast.walk(tree):
        if isinstance(n, ast.ClassDef):
            for f in n.body:
                f.cls_marks = marks(n)  # type: ignore[attr-defined]
    for n in ast.walk(tree):
        if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)) and n.name.startswith("test"):
            tests += 1
            m = marks(n) | module | getattr(n, "cls_marks", set())
            args = {a.arg for a in n.args.args}
            if m & EXCLUDED:
                excluded += 1
            elif ("db" in m or e2e) and "pg_engine" not in args:
                skipped += 1
    cause = ("every test deselected by the marker expression" if tests and excluded == tests else
             "every test skipped by the plugin's operator-DB rule (db/_e2e without pg_engine)" if tests and excluded + skipped == tests else
             "no test function found" if not tests else
             "unexplained statically: tests run but coverage saw no scope line in a test context (import-time only, or the "
             "reference is not on an executed path)")
    return {"tests": tests, "excluded_by_marker": excluded, "operator_db_skipped": skipped, "e2e_stem": e2e, "cause": cause}


def main(e3_json: Path, e1b_json: Path, out: Path) -> dict:
    d5, d8 = (json.loads(p.read_text(encoding="utf-8")) for p in (e3_json, e1b_json))
    sel = json.loads((E1B / "selection-map.json").read_text(encoding="utf-8"))
    cov = set(sel["seeds"])
    static = set((E1B / "seeds-static-candidates.txt").read_text(encoding="utf-8").split())
    t4 = set(json.loads((E3 / "t4-mutant-sample.json").read_text(encoding="utf-8"))["seed_files"])
    pub = set(d5["scope_references"]["by_module"]["facdrone.shells.decision_evidence_publication"])
    resolved = {x for v in d5["deselections"]["resolved_ids"].values() for x in v}
    s = d8["seeds"]
    rec, upper = set(s["recommended"]), set(s["recommended"]) | set(s["via_production"])
    hand_rec = static - rec
    body = {
        "importing_files": {"hand": len(t4), "detected_publication_module": len(pub), "match": sorted(pub & t4),
                            "extra": sorted(pub - t4), "missing": sorted(t4 - pub),
                            "detected_all_five_modules": len(d5["scope_references"]["test_files"])},
        "deselections": {"hand_total": len(sel["deselect"]), "detected_total": d5["deselections"]["total"],
                         "set_equal": resolved == set(sel["deselect"]), "symmetric_difference": sorted(resolved ^ set(sel["deselect"])),
                         "collection_errors": d5["deselections"]["collection_errors"],
                         "applicable_to_191": d5["deselections"]["applicable_to_supplied_seeds_count"],
                         "applicable_to_recommended_e3_scope": d5["deselections"]["applicable_count"],
                         "applicable_to_recommended_e1b_scope": d8["deselections"]["applicable_count"],
                         "applicable_191_by_cause": {c: len(v) for c, v in d5["deselections"]["applicable_to_supplied_seeds"].items()}},
        "seeds_e1b_scope": {"coverage_191": len(cov), "hand_static_115": len(static), "counts": s["counts"],
                            "recommended_in_coverage": len(rec & cov), "recommended_not_in_coverage": {f: hint(f) for f in sorted(rec - cov)},
                            "coverage_only_via_production": len(cov & set(s["via_production"])), "coverage_outside_upper_bound": sorted(cov - upper),
                            "upper_bound": len(upper), "hand_static_not_in_coverage": {f: hint(f) for f in sorted(static - cov)},
                            "vs_hand_static": {"hand_only": sorted(hand_rec), "detected_only": sorted(rec - static),
                                               "detected_only_in_coverage": sorted((rec - static) & cov),
                                               "hand_only_in_coverage": sorted(hand_rec & cov)},
                            "ignored_by_gates": s["ignored_by_gates"]},
        "marker_expression": {"hand": "not port_baseline and not live_broker", "suggested": d5["marker_expression"]["suggested"],
                              "equal": d5["marker_expression"]["suggested"] == "not port_baseline and not live_broker",
                              "signals": d5["marker_expression"]["signals"], "gates": d5["marker_expression"]["distinct_gate_expressions"]},
        "host": {"hazards": d5["external_store"]["host_hazards"], "store": d5["external_store"]["summary"]},
    }
    out.write_text(json.dumps(body, indent=1, sort_keys=True), encoding="utf-8")
    return body


if __name__ == "__main__":
    b = main(*map(Path, sys.argv[1:4]))
    i, x, s = b["importing_files"], b["deselections"], b["seeds_e1b_scope"]
    print(f"importing {i['detected_publication_module']}/{i['hand']} extra {len(i['extra'])} missing {len(i['missing'])}; "
          f"deselections {x['detected_total']}/{x['hand_total']} equal={x['set_equal']} applicable {x['applicable_to_191']}; "
          f"seeds recommended {s['counts']['recommended']} in-coverage {s['recommended_in_coverage']}/191 upper {s['upper_bound']} "
          f"outside-upper {len(s['coverage_outside_upper_bound'])}; marker equal={b['marker_expression']['equal']}")
