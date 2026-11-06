"""6.4 rates from the receipt only: the last `<pfx>-m<k>-class` note per label (sampled mutants) and `<pfx>-M<j>-class`
(hand-written) for e4o (original) and e4c (candidate). Rate = killed / (killed + survived), incompetent excluded (R22,
requirement 8.2); the 0.70 bar (E3 T4) stated met or not met per tree; T5 (band 0.60 to 0.75, falsifier < 0.55 or > 0.85)
read on the candidate only. Two post hoc readings, labelled and not the result (8.3): survivors that
results/e4-survivor-readings.json classes as `equivalent` excluded, and additionally those classed `closed_assumption`
(residue-sited or patched-away code the arm never executes, as the plan's 6.4 words it) excluded. Writes
results/e4-rates.json; `--check` reproduces it byte for byte."""
import json, re, sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
RES = HERE / "results"
TREES = {"e4o": "original (.runs/e2-facdrone/source via orig-mut)", "e4c": "candidate (.runs/e4-lifecycle/cand-turn3 via cand-mut)"}


def rates(receipt: dict, readings: dict) -> dict:
    last = {}
    for n in receipt["notes"]:
        if (m := re.fullmatch(r"(e4[oc])-([mM]\d+)-class", n["key"])):
            last[m.group(1), m.group(2)] = n["value"]
    out = {"convention": "killed / (killed + survived), incompetent excluded (R22); bar 0.70 (E3 T4) unchanged", "trees": {}}
    for pfx, name in TREES.items():
        rows = {k: v for (p, k), v in last.items() if p == pfx}
        sampled = {k: v for k, v in rows.items() if k.startswith("m")}
        cls = {c: sorted(int(k[1:]) for k, v in sampled.items() if v["classification"] == c) for c in ("killed", "survived", "incompetent", "apparatus-fault")}
        kd, sv = len(cls["killed"]), len(cls["survived"])
        rate = round(kd / (kd + sv), 3) if kd + sv else None
        by = lambda *classes: sorted(int(k) for k, v in readings.get(pfx, {}).items() if v.get("class") in classes and int(k) in cls["survived"])  # noqa: E731
        equiv, unreached = by("equivalent"), by("equivalent", "closed_assumption")
        post = round(kd / (kd + sv - len(equiv)), 3) if kd + sv - len(equiv) else None
        post2 = round(kd / (kd + sv - len(unreached)), 3) if kd + sv - len(unreached) else None
        hand = {k: {"classification": v["classification"], "caught": v["classification"] == "killed", "first_failing_reason": v["first_failing_reason"],
                    "seconds": v["seconds"]} for k, v in sorted(rows.items()) if k.startswith("M")}
        out["trees"][pfx] = {"tree": name, "sampled": len(sampled), **{c: len(v) for c, v in cls.items()}, "by_class": cls, "rate": rate,
                             "bar_0.70": None if rate is None else ("met" if rate >= 0.70 else "not met"),
                             "post_hoc_equivalent_excluded": {"label": "post hoc, not the result (8.3)", "excluded": equiv, "rate": post},
                             "post_hoc_equivalent_and_residue_excluded": {"label": "post hoc, not the result (8.3)", "excluded": unreached, "rate": post2},
                             "hand_written": hand}
    r = out["trees"]["e4c"]["rate"]
    out["T5"] = {"read_on": "candidate's own sample (e4c-m*)", "rate": r, "band": [0.60, 0.75], "falsifier": "< 0.55 or > 0.85",
                 "reading": None if r is None else ("held" if 0.60 <= r <= 0.75 else "failed" if r < 0.55 or r > 0.85 else "outside the band, not falsified")}
    return out


if __name__ == "__main__":
    receipt = json.loads((RES / "receipt.json").read_text(encoding="utf-8"))
    rd = RES / "e4-survivor-readings.json"
    readings = json.loads(rd.read_text(encoding="utf-8")).get("survivors", {}) if rd.exists() else {}
    body = json.dumps(rates(receipt, readings), indent=1)
    out = RES / "e4-rates.json"
    if "--check" in sys.argv:
        same = out.exists() and out.read_text(encoding="utf-8") == body
        print(f"e4_rates --check: {'identical' if same else 'DIFFERS'}")
        sys.exit(0 if same else 1)
    out.write_text(body, encoding="utf-8")
    print(body)
