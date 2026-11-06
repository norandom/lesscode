"""deconvolution-loop task 3.2: the two comparable sensitivity rates, original (t31 series) and candidate (E3 T4).
Reads evidence only: the last `t31-m<k>-class` note per k from results/receipt.json (a rerun supersedes an
interrupted run; the note `t31-resume-*` says why), the last `t4-m<k>` or `t4-m<k>-rerun` command exit per k
from E3's receipt (its classification rule: 0 survived, 2 incompetent, other killed, as E3's README table),
and the two hand-written survivor readings for the post hoc, labelled equivalent-excluded reading.
Rate (R22 convention, preregistered): killed / (killed + survived); incompetent excluded; bar 0.70 unchanged.
  t32_summary.py            writes results/t32-summary.json and prints the markdown table
  t32_summary.py --check    exit 0 if the file on disk equals a fresh computation, else 1"""
from __future__ import annotations
import json, re, sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
E3 = HERE.parent / "e3-evidence-regeneration"
BAR, N = 0.70, 30
EXIT_CLASS = {0: "survived", 2: "incompetent"}


def original_classes(receipt: dict) -> dict[int, dict]:
    out = {}
    for n in receipt["notes"]:
        m = re.fullmatch(r"t31-m(\d+)-class", n.get("key", ""))
        if m:
            out[int(m.group(1))] = n["value"]  # last note wins
    return out


def candidate_classes(receipt: dict) -> dict[int, dict]:
    out = {}
    for c in receipt["commands"]:
        m = re.fullmatch(r"t4-m(\d+)(-rerun)?", c.get("label", ""))
        if m:
            out[int(m.group(1))] = {"k": int(m.group(1)), "label": c["label"], "exit": c["exit"],
                                    "classification": EXIT_CLASS.get(c["exit"], "killed")}
    return out


def arm(name: str, classes: dict[int, dict], readings: dict) -> dict:
    missing = sorted(set(range(N)) - set(classes))
    tally = {c: sorted(k for k, v in classes.items() if v["classification"] == c) for c in ("killed", "survived", "incompetent")}
    other = sorted(k for k, v in classes.items() if v["classification"] not in tally)
    k, s = len(tally["killed"]), len(tally["survived"])
    equivalent = sorted(int(i) for i, v in readings.get("survivors", {}).items() if v["class"] == "equivalent" and int(i) in tally["survived"])
    rate = round(k / (k + s), 3) if k + s else None
    post = round(k / (k + s - len(equivalent)), 3) if k + s - len(equivalent) else None
    return {"arm": name, "n": N, "missing": missing, "other": other, "counts": {c: len(v) for c, v in tally.items()}, "mutants": tally,
            "rate": rate, "rate_formula": "killed / (killed + survived)", "bar": BAR, "meets_bar": None if rate is None else rate >= BAR,
            "post_hoc": {"label": "equivalent-excluded, not preregistered, not the result", "equivalent_by_inspection": equivalent, "rate": post}}


def build() -> dict:
    t31 = json.loads((HERE / "results/receipt.json").read_text(encoding="utf-8"))
    t4 = json.loads((E3 / "results/receipt.json").read_text(encoding="utf-8"))
    r_o = json.loads((HERE / "results/t32-survivor-readings.json").read_text(encoding="utf-8"))
    r_c = json.loads((E3 / "results/t4-survivor-readings.json").read_text(encoding="utf-8"))
    return {"about": "Task 3.2: sensitivity of the same acceptance arm on the original and on the E3 candidate, same seed (20261104), "
                     "30 draws each from the sorted spec keys of the module's own cosmic-ray population; populations 307 and 317.",
            "resume": [n["value"] for n in t31["notes"] if n.get("key", "").startswith("t31-resume")],
            "original": arm("original (.runs/e2-facdrone/source, t31 series)", original_classes(t31), r_o),
            "candidate": arm("candidate (.runs/e3-facdrone/cand, E3 T4)", candidate_classes(t4), r_c)}


def table(s: dict) -> str:
    rows = ["| Arm | Killed | Survived | Incompetent | Rate | Bar 0.70 | Post hoc equivalent-excluded (labelled) |", "| --- | ---: | ---: | ---: | ---: | --- | ---: |"]
    for a in (s["original"], s["candidate"]):
        c = a["counts"]
        rows.append(f"| {a['arm']} | {c['killed']} | {c['survived']} | {c['incompetent']} | {a['rate']} | {'met' if a['meets_bar'] else 'not met'} | "
                    f"{a['post_hoc']['rate']} ({len(a['post_hoc']['equivalent_by_inspection'])} excluded) |")
    return "\n".join(rows)


if __name__ == "__main__":
    s, out = build(), HERE / "results/t32-summary.json"
    text = json.dumps(s, indent=1) + "\n"
    if "--check" in sys.argv:
        same = out.exists() and out.read_text(encoding="utf-8") == text
        print(f"t32_summary --check: {'identical' if same else 'DIFFERS'}")
        sys.exit(0 if same else 1)
    out.write_text(text, encoding="utf-8")
    print(table(s))
    for a in (s["original"], s["candidate"]):
        if a["missing"] or a["other"]:
            print(f"INCOMPLETE {a['arm']}: missing {a['missing']} other {a['other']}")
