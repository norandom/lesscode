"""Missing-test report (deconvolution-loop task 2.1, req 8.7): T4 survivors and T5 realised differences.
  missing_tests.py [--out DIR]   -> DIR/missing-tests.md and DIR/missing-tests.json (DIR defaults to results)
Inputs (read only): results/t4-mutant-sample.json, the last fact of results/receipt.json (t4.survivors and the
post hoc equivalence list), results/logs/t5-cx-replay.log, results/t5-soundness.json, the hand-written
results/t4-survivor-readings.json, and the candidate and original publication modules."""
import argparse, ast, hashlib, json, re, sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
RUNS = HERE.parents[3] / ".runs"
CAND, ORIG, RES = RUNS / "e3-facdrone" / "cand", RUNS / "e2-facdrone" / "source", HERE / "results"
EXC = re.compile(r"\(\('raise', '(\w+)'")


def _defs(src: str) -> list:
    return [n for n in ast.walk(ast.parse(src)) if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef))]


def enclosing(src: str, line: int) -> str:
    """Innermost def whose span holds the line (a def's own header line counts)."""
    hits = [n for n in _defs(src) if n.lineno <= line <= n.end_lineno]
    if not hits:
        raise ValueError(f"line {line} is outside every def")
    return max(hits, key=lambda n: n.lineno).name


def anchor_line(src: str, function: str, anchor: str) -> int:
    """The single line of the named def that contains anchor."""
    spans = [(n.lineno, n.end_lineno) for n in _defs(src) if n.name == function]
    if len(spans) != 1:
        raise ValueError(f"{len(spans)} defs named {function}")
    lines = src.split("\n")
    hits = [i for i in range(spans[0][0], spans[0][1] + 1) if anchor in lines[i - 1]]
    if len(hits) != 1:
        raise ValueError(f"anchor {anchor!r} occurs {len(hits)} times in {function}")
    return hits[0]


def _exc(text: str) -> str | None:
    m = EXC.match(text)
    return m.group(1) if m else None


def build(sample: dict, facts: list, replay: list, soundness: dict, readings: dict, cand_src: str, orig_src: str) -> dict:
    if hashlib.sha256(cand_src.encode("utf-8")).hexdigest() != sample["candidate_sha256"]:
        raise ValueError("candidate module differs from the sampled one")
    t4 = facts[-1]["t4"]
    listed = re.search(r"excludes ([\d,\s]+)", t4["post_hoc_excluding_equivalent"]).group(1)
    post_hoc = sorted(int(x) for x in listed.split(",") if x.strip())
    eq = sorted(int(k) for k, v in readings["survivors"].items() if v["class"] == "equivalent")
    if post_hoc != eq:
        raise ValueError(f"post hoc equivalents in the receipt {post_hoc} differ from the readings {eq}")
    if sorted(map(int, readings["survivors"])) != sorted(t4["survivors"]):
        raise ValueError("readings do not cover exactly the recorded survivors")
    out: dict = {"module": readings["module"], "candidate_sha256": sample["candidate_sha256"],
                 "real": [], "equivalent": [], "closed_assumption": [],
                 "controls_excluded": sorted({x["log"] for x in replay if not any(k.startswith("o_") for k in x)})}
    for k in sorted(t4["survivors"]):
        s, r = sample["sample"][k], readings["survivors"][str(k)]
        item = {"id": f"T4-{k}", "source": "T4 survivor", "operator": s["operator"],
                "function": enclosing(cand_src, s["row"]), "file": readings["module"], "line": s["row"],
                "written": " / ".join(x.strip() for o in s["old"] for x in o),
                "mutated": " / ".join(x.strip() for h in s["hunks"] for x in h[2])}
        if r["class"] == "real":
            out["real"].append(item | {
                "original_function": r["original_function"], "entry": r["entry"], "observable": r["observable"],
                "original_line": anchor_line(orig_src, r["original_function"], r["original_anchor"]),
                "input_class": r["input_class"]})
        else:
            extra = {"input_class": r["input_class"]} if "input_class" in r else {}
            out["equivalent" if r["class"] == "equivalent" else "closed_assumption"].append(item | {"reason": r["reason"]} | extra)
    for pair, r in readings["differential"].items():
        recs = [x for x in replay if f"o_{pair}" in x and x.get("differ")]
        mm, calls, shapes = soundness[pair]["mismatches"], soundness[pair]["calls"], {}
        for x in mm:  # shape: the exception type when both sides raise the same one
            key = _exc(x["o"]) if _exc(x["o"]) and _exc(x["o"]) == _exc(x["c"]) else "other"
            shapes[key] = shapes.get(key, 0) + 1
        rep = recs[0] if recs else None
        out["real"].append({
            "id": f"T5-{pair}", "hypothesis": r["hypothesis"], "function": r["function"], "file": readings["module"],
            "line": anchor_line(cand_src, r["function"], r["anchor"]),
            "source": (f"T5 counterexample ({rep['log']}) + " if rep else "") + f"soundness corpus ({len(mm)}/{calls} differ)",
            "written": r["condition"], "mutated": None, "observable": r["observable"], "entry": r["entry"],
            "original_function": r["original_function"],
            "original_line": anchor_line(orig_src, r["original_function"], r["original_anchor"]),
            "corpus_shapes": dict(sorted(shapes.items())), "input_class": r["input_class"],
            "replay": None if rep is None else {"args": rep["args"], "original": rep[f"o_{pair}"],
                                                 "candidate": rep[f"c_{pair}"], "differ": rep["differ"]}})
    return out


def render(r: dict) -> str:
    L = ["# Missing-test report, E3 (T4 survivors and T5 realised differences)", "",
         f"Module `{r['module']}`, candidate sha256 `{r['candidate_sha256'][:16]}…`. Generated by `missing_tests.py` from",
         "`t4-mutant-sample.json`, the last fact of `receipt.json`, `logs/t5-cx-replay.log` and `t5-soundness.json`.",
         "The per-survivor readings (class, reason, original anchor, input class) were written by hand from the",
         "README's Step 3 section into `t4-survivor-readings.json`. They are readings, not measurements. Line numbers",
         "are candidate lines. The original's function and line are also given, for a test writer who has only the",
         "original tree. The module path is the same in both trees.", "", "## Real gaps", ""]
    for i in r["real"]:
        L += [f"### {i['id']}: `{i['function']}` (candidate line {i['line']}; original `{i['original_function']}` line {i['original_line']})", "",
              f"- Source: {i['source']}" + (f"; reviewer hypothesis {i['hypothesis']}" if i.get("hypothesis") else f"; operator `{i['operator']}`"),
              f"- Reached through: {', '.join(f'`{e}`' for e in i['entry'])}"]
        if i["mutated"] is None:
            L.append(f"- Differing condition: {i['written']}")
        else:
            L += [f"- Condition as written: `{i['written']}`", f"- As mutated: `{i['mutated']}`"]
        L += [f"- Observable: {i['observable']}", f"- Input class that distinguishes it: {i['input_class']}"]
        if i.get("corpus_shapes"):
            L.append("- Corpus shapes (exception type, the same on both sides): " + ", ".join(f"{k} {v}" for k, v in i["corpus_shapes"].items()))
        if i.get("replay"):
            p = i["replay"]
            L += [f"- Concrete replay (args in the T5 driver's own encoding, given for provenance; the input class above is the reading): args `{p['args']}`",
                  f"  original `{p['original']}`; candidate `{p['candidate']}`; differ {str(p['differ']).lower()}"]
        L.append("")
    for key, title in (("equivalent", "Equivalent by inspection (post hoc, labelled; not the preregistered measure)"),
                       ("closed_assumption", "Equivalent only under a closed assumption (not counted as equivalent)")):
        L += [f"## {title}", "", "| Id | Function | Line | Written | Mutated | Reason |", "| --- | --- | ---: | --- | --- | --- |"]
        esc = lambda s: s.replace("|", "\\|")  # noqa: E731  (a pipe inside a table cell)
        L += [f"| {i['id']} | `{i['function']}` | {i['line']} | `{esc(i['written'])}` | `{esc(i['mutated'])}` | {esc(i['reason'])}"
              + (f" Distinguishing input: {esc(i['input_class'])}" if i.get("input_class") else "") + " |" for i in r[key]]
        L.append("")
    L += [f"Replay records excluded as session controls (planted mutants, not gaps): {', '.join(r['controls_excluded']) or 'none'}.", ""]
    return "\n".join(L)


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--out", type=Path, default=RES)
    out = p.parse_args(argv).out
    out = out if out.is_absolute() else HERE / out
    sample = json.loads((RES / "t4-mutant-sample.json").read_text(encoding="utf-8"))
    rel = sample["module"]
    log = (RES / "logs" / "t5-cx-replay.log").read_text(encoding="utf-8").splitlines()
    r = build(sample, json.loads((RES / "receipt.json").read_text(encoding="utf-8"))["facts"],
              [json.loads(x) for x in log if x.startswith("{")],
              json.loads((RES / "t5-soundness.json").read_text(encoding="utf-8")),
              json.loads((RES / "t4-survivor-readings.json").read_text(encoding="utf-8")),
              (CAND / rel).read_bytes().decode("utf-8"), (ORIG / rel).read_bytes().decode("utf-8"))
    (out / "missing-tests.json").write_text(json.dumps(r, indent=1), encoding="utf-8")
    (out / "missing-tests.md").write_text(render(r), encoding="utf-8")
    print(f"missing_tests: {len(r['real'])} real, {len(r['equivalent'])} equivalent, "
          f"{len(r['closed_assumption'])} closed-assumption -> {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
