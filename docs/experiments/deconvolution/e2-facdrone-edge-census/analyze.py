"""E2 analysis: joins recordings, coverage and static facts into per-responsibility measures.

usage: analyze.py RESULTS_DIR STATIC_FACTS SOURCE_ROOT OUT_DIR [REPEAT_RESULTS_DIR]
Writes OUT_DIR/edges.json and OUT_DIR/calibration.md. Definitions: plan.md and its amendments.
"""
import glob, json, math, os, sys
from collections import defaultdict
from pathlib import Path

import coverage

BUNDLES = Path(__file__).resolve().parents[2] / "constraint-modeler/facdrone-discovery/run-02"
CASES = {"A": "case-evidence-publication.json", "B": "case-report-envelope.json", "C": "case-index-provider-orchestration.json"}
R11 = {"src/facdrone/shells/cli/burn_in_cache.py", "src/facdrone/shells/wiring/run_config.py"}
MEASURES = ("call_edges", "hops_max", "write_sites", "churn_max", "dos", "ordering")
fileof = lambda c: c.split("::")[0]

def load(rdir):
    edges, closure, invs, seen, outcomes = defaultdict(set), defaultdict(set), defaultdict(list), set(), {}
    for path in glob.glob(os.path.join(rdir, "**", "rec-*.jsonl"), recursive=True):
        ids = {}
        for line in open(path, encoding="utf-8"):
            r = json.loads(line)
            if "k" in r:
                ids[r["k"]] = r["id"]
                continue
            seen.update(ids[d] for _, _, d in r["edges"])
            if r["t"] == "<none>":
                continue
            outcomes[f"{os.path.basename(os.path.dirname(path))}/{r['t']}"] = r.get("out")
            for e, c, d in r["edges"]:
                closure[ids[e]].add(ids[d])
                if c >= 0:
                    edges[ids[e]].add((ids[c], ids[d]))
            for e, n, fx in r["inv"]:
                invs[ids[e]].append(([(s, [ids[x] for x in st], {ids[int(k)]: v for k, v in by.items()}, w)
                                      for s, st, _, by, w in fx], n))
    return edges, closure, invs, seen, outcomes

def covered(rdir, root):
    lines = defaultdict(set)
    for path in glob.glob(os.path.join(rdir, "**", ".coverage.*"), recursive=True):
        data = coverage.CoverageData(basename=path)
        data.read()
        for f in data.measured_files():
            lines[os.path.relpath(f, root).replace("\\", "/")].update(data.lines(f) or ())
    return lines

def measure(S, edge_set, recs, facts, files):
    hops, churn, before, writers, mods = [], [], set(), defaultdict(set), defaultdict(int)
    for c in S:
        f = facts.get(c, {})
        for p in f.get("writes", ()):
            writers[p].add(c)
        mods[files[fileof(c)]["module"]] += len(f.get("lines", ()))
    for fx, _ in recs:
        seq = []
        for sink, st, by, table in fx:
            st = [c for c in st if c in S]
            if not st:
                continue
            hops.append(len({files[fileof(c)]["module"] for c in st}))
            churn.append(sum(v for c, v in by.items() if c in S))
            seq += [(sink, st[-1])] if (sink, st[-1]) not in seq else []
            if table:
                writers["table:" + table].add(st[-1])
        before.update((a, b) for i, a in enumerate(seq) for b in seq[i + 1:])
    total, T = sum(mods.values()), len(files)
    sq = sum((v / total - 1 / T) ** 2 for v in mods.values() if v) + (T - sum(1 for v in mods.values() if v)) / T**2 if total else 0
    return {"call_edges": len(edge_set), "hops_max": max(hops, default=0),
            "hops_mean": round(sum(hops) / len(hops), 3) if hops else 0,
            "write_sites": max(map(len, writers.values()), default=0),
            "churn_max": max(churn, default=0), "churn_mean": round(sum(churn) / len(churn), 3) if churn else 0,
            "dos": round(1 - T / (T - 1) * sq, 4) if total else 0.0, "ordering": sum((b, a) not in before for a, b in before),
            "effects": len(hops), "files": len({fileof(c) for c in S}), "physical_lines": total,
            "ast_statements": sum(facts.get(c, {}).get("stmts", 0) for c in S)}

def ranks(xs):
    order, r, i = sorted(range(len(xs)), key=lambda k: xs[k]), [0.0] * len(xs), 0
    while i < len(xs):
        j = i
        while j + 1 < len(xs) and xs[order[j + 1]] == xs[order[i]]:
            j += 1
        for k in range(i, j + 1):
            r[order[k]] = (i + j) / 2 + 1
        i = j + 1
    return r

def spearman(xs, ys):
    rx, ry = ranks(xs), ranks(ys)
    mx, my = sum(rx) / len(rx), sum(ry) / len(ry)
    num = sum((a - mx) * (b - my) for a, b in zip(rx, ry))
    den = math.sqrt(sum((a - mx) ** 2 for a in rx) * sum((b - my) ** 2 for b in ry))
    return round(num / den, 4) if den else None

def position(v, pop):
    s, n = sorted(pop), len(pop)
    med = s[n // 2] if n % 2 else (s[n // 2 - 1] + s[n // 2]) / 2
    greater = sum(x > v for x in pop)
    return {"value": v, "greater": greater, "top_decile": v > 0 and greater < math.ceil(0.1 * n),
            "below_median": v < med, "above_median": v > med, "median": med}

def main(rdir, facts_path, root, out, repeat=None):
    static = json.load(open(facts_path))
    facts, files = static["callables"], static["files"]
    edges, closure, invs, seen, outcomes = load(rdir)
    cov = covered(rdir, root)
    funcs = [k for k, f in facts.items() if f["kind"] == "function"]
    executed = seen | {k for k in funcs if cov.get(fileof(k), set()) & set(facts[k]["lines"])}
    entries = [e for e in closure if facts.get(e, {"kind": "function"})["kind"] == "function"]
    rows = {e: {"entry": e, **measure(closure[e], edges[e], invs[e], facts, files),
                "closure_files": sorted({fileof(c) for c in closure[e]})} for e in entries}
    pop = {m: [rows[e][m] for e in entries] for m in MEASURES}
    cases = {}
    for name, bundle in CASES.items():
        cfiles = {f["path"] for f in json.load(open(BUNDLES / bundle))["files"]}
        S = {c for c in executed if fileof(c) in cfiles}
        es = {(a, b) for e in entries for a, b in edges[e] if a in S and b in S}
        val = measure(S, es, [r for e in entries if closure[e] & S for r in invs[e]], facts, files)
        inner = [rows[e] for e in entries if fileof(e) in cfiles]
        declared = {"files": len(cfiles), **{k: sum(files[f][k] for f in cfiles) for k in ("physical_lines", "ast_statements")}}
        cases[name] = {"bundle": bundle, "files": sorted(cfiles), "file_set": val, "declared_size": declared,
                       "static_only_callables": sum(1 for k in funcs if fileof(k) in cfiles and k not in executed),
                       "rank": {m: position(val[m], pop[m]) for m in MEASURES},
                       "best_inner_entry": {m: position(max((r[m] for r in inner), default=0), pop[m]) for m in MEASURES}}
    excluded = set().union(*(cases[c]["files"] for c in cases)) | R11
    for e in entries:
        rows[e]["top_decile"] = [m for m in MEASURES[:5] if position(rows[e][m], pop[m])["top_decile"]]
    novel = [e for e in entries if fileof(e) not in excluded and len(rows[e]["top_decile"]) >= 2]
    rho = {m: spearman([rows[e]["physical_lines"] for e in entries], pop[m]) for m in MEASURES}
    A, B, C = (cases[k]["rank"] for k in "ABC")
    verdict = {"C1": A["write_sites"]["top_decile"] and A["churn_max"]["top_decile"],
               "C2": C["write_sites"]["below_median"] and C["churn_max"]["below_median"],
               "C3": B["churn_max"]["above_median"], "C4": len(novel) >= 3,
               "C5": all(r is not None and r < 0.7 for r in rho.values())}
    reached = sum(1 for k in funcs if k in executed) / len(funcs)
    gt = lambda e: sum(sum(x > rows[e][m] for x in pop[m]) for m in MEASURES[:5])
    top = sorted(entries, key=lambda e: (-len(rows[e]["top_decile"]), gt(e)))[:20]
    diffs = {}
    if repeat:
        e2, c2 = load(repeat)[:2]
        diffs = {e: {"call_edges": [len(edges[e]), len(e2[e])], "closure": [len(closure[e]), len(c2[e])]}
                 for e in set(closure) | set(c2) if (len(edges[e]), len(closure[e])) != (len(e2[e]), len(c2[e]))}
    failed = sorted(t for t, o in outcomes.items() if o and any(x.endswith("failed") for x in o))
    result = {"population": len(entries), "functions": len(funcs), "functions_reached": round(reached, 4),
              "census_partial": reached < 0.6, "tests": len(outcomes), "tests_failed_or_errored": failed,
              "cases": cases, "spearman_lines": rho, "novel_top_decile": novel, "verdict": verdict,
              "repeat_differences": diffs, "top20": top, "responsibilities": [rows[e] for e in entries]}
    Path(out).mkdir(parents=True, exist_ok=True)
    Path(out, "edges.json").write_text(json.dumps(result, indent=1))
    cut = math.ceil(0.1 * len(entries))
    ev = {"C1": f"A write sites {A['write_sites']['value']} ({A['write_sites']['greater']} greater), churn "
                f"{A['churn_max']['value']} ({A['churn_max']['greater']} greater); top decile = fewer than {cut} greater",
          "C2": f"C write sites {C['write_sites']['value']} vs median {C['write_sites']['median']}, "
                f"churn {C['churn_max']['value']} vs median {C['churn_max']['median']}",
          "C3": f"B churn {B['churn_max']['value']} vs median {B['churn_max']['median']}",
          "C4": f"{len(novel)} responsibilities outside A, B, C and R11 in the top decile on two or more of the first five",
          "C5": ", ".join(f"{m} {r}" for m, r in rho.items())}
    head = " | ".join(MEASURES)
    md = ["# E2 calibration", "", f"Population: {len(entries)} entry responsibilities; {len(outcomes)} tests, "
          f"{len(failed)} failed or errored; production functions reached {reached:.1%}"
          f"{' (census partial: below 60%)' if reached < 0.6 else ''}.", "",
          "| Prediction | Holds | Evidence |", "| --- | --- | --- |"]
    md += [f"| {k} | {'yes' if v else 'no'} | {ev[k]} |" for k, v in verdict.items()]
    md += ["", "## Cases as declared file sets", "", f"| Case | {head} | lines | statements | static-only |",
           "|---" * (len(MEASURES) + 4) + "|"]
    md += [f"| {k} | " + " | ".join(str(c["file_set"][m]) for m in MEASURES) + f" | {c['file_set']['physical_lines']}"
           f" | {c['file_set']['ast_statements']} | {c['static_only_callables']} |" for k, c in cases.items()]
    md += ["", "Lines and statements above are those of executed callables; declared sizes (R15 basis): "
           + "; ".join(f"{k} {c['declared_size']}" for k, c in cases.items()) + "."]
    md += ["", "## Top twenty responsibilities", "", f"| # | Entry | top decile | {head} | files | lines |",
           "|---" * (len(MEASURES) + 5) + "|"]
    md += [f"| {i} | `{e}` | {', '.join(rows[e]['top_decile'])} | " + " | ".join(str(rows[e][m]) for m in MEASURES)
           + f" | {rows[e]['files']} | {rows[e]['physical_lines']} |" for i, e in enumerate(top, 1)]
    md += ["", "File sets: `edges.json`, `responsibilities[].closure_files`.",
           f"Repeat-run differences: {len(diffs)} entries." if repeat else "No repeat run analysed.", ""]
    Path(out, "calibration.md").write_text("\n".join(md), encoding="utf-8")
    print(json.dumps({"population": len(entries), "reached": round(reached, 4), "verdict": verdict}))

if __name__ == "__main__":
    main(*sys.argv[1:])
