r"""Parity of the native spike with the Python prototype (read-only use of the prototype).

Run from docs/experiments/annotation-driver with the prototype environment:

    $env:UV_PROJECT_ENVIRONMENT = 'D:\BuildArtifacts\lesscode\annotation-driver\venv'
    $env:PYTHONDONTWRITEBYTECODE = '1'
    uv run --no-sync python native-spike/parity.py --bin D:\...\annotation-driver-spike.exe
        [--scopes-dir D:\BuildArtifacts\lesscode\annotation-driver\scopes-ro]
        [--work D:\BuildArtifacts\lesscode\annotation-driver\spike-parity] [--packs DIR]

Writes reference and spike outputs plus parity.json under --work; prints the
parity table. Exit 0 only when every comparison is identical.
"""

from __future__ import annotations

import argparse
import copy
import dataclasses
import hashlib
import json
import subprocess
import sys
from pathlib import Path

sys.dont_write_bytecode = True
PROTOTYPE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(PROTOTYPE))

import engine as e  # noqa: E402
import pack_check as pc  # noqa: E402
import run_controls as rc  # noqa: E402
import validator as v  # noqa: E402

SHOW = 5


def site_key(s: e.Site) -> str:
    return f"{s.file}:{s.kind}:{s.start_line}:{s.start_column}-{s.end_line}:{s.end_column}"


def spike(args, *argv: str) -> object:
    out = args.work / "out.json"
    proc = subprocess.run([str(args.bin), *argv, "--out", str(out)], capture_output=True, text=True)
    if proc.returncode == 2:
        raise SystemExit(f"spike failed: {' '.join(argv)}\n{proc.stderr}")
    return json.loads(out.read_text(encoding="utf-8"))


def common(args) -> list[str]:
    return ["--packs", str(args.packs), "--scopes-dir", str(args.scopes_dir), "--manifest", str(PROTOTYPE / "scopes.json")]


def diff(expected: list, got: list) -> dict:
    """Counts and the first differing positions."""
    first = [{"index": i, "prototype": a, "spike": b}
             for i, (a, b) in enumerate(zip(expected, got)) if a != b][:SHOW]
    if len(expected) != len(got) and len(first) < SHOW:
        n = min(len(expected), len(got))
        first.append({"index": n, "prototype": expected[n:n + 2], "spike": got[n:n + 2]})
    return {"prototype": len(expected), "spike": len(got), "identical": expected == got, "first_differences": first}


def sites_and_mutants(args, scopes) -> dict:
    out = {}
    for name, scope in scopes.items():
        ref_sites = e.eligible_sites(scope)
        sites = [dataclasses.asdict(s) for s in ref_sites]
        muts = [{"site": site_key(s), "operator": m.operator, "sha256": hashlib.sha256(m.text.encode("utf-8")).hexdigest()}
                for s in ref_sites for m in e.mutants(scope, s)]
        got_sites = spike(args, "sites", *common(args), "--scope", name)
        got_muts = spike(args, "mutants", *common(args), "--scope", name)
        out[name] = {"sites": diff(sites, got_sites), "mutants": diff(muts, got_muts)}
    return out


def controls(args, scopes) -> dict:
    spec_all = json.loads((PROTOTYPE / "controls.json").read_text(encoding="utf-8"))["scopes"]
    ref = []
    for name, spec in spec_all.items():
        scope = scopes[name]
        for label, annotations, expected in rc.annotation_cases(scope, spec):
            got = [rc.outcome(d) for d in v.validate(scope, annotations)]
            ref.append({"scope": name, "control": label, "expected": expected, "got": got})
        original = scope.source(spec["seeds"][0]["file"])
        for label, mutated, expected in rc.mutant_cases(scope, spec):
            ref.append({"scope": name, "control": f"mutant:{label}", "expected": expected,
                        "got": v.classify_mutant(scope.language, original, mutated)})
    got = spike(args, "controls", *common(args), "--controls", str(PROTOTYPE / "controls.json"))["results"]
    strip = [{k: r[k] for k in ("scope", "control", "expected", "got")} for r in got]
    as_expected = sum(r["expected"] == r["got"] for r in ref)
    return {"prototype_as_expected": f"{as_expected} of {len(ref)}",
            "spike_as_expected": f"{sum(r['pass'] for r in got)} of {len(got)}", **diff(ref, strip)}


def negative_packs(work: Path) -> list[tuple[str, str, str]]:
    base = json.loads((PROTOTYPE / "tables" / "rust.json").read_text(encoding="utf-8"))
    bad_kind = copy.deepcopy(base)
    bad_kind["kinds"]["failure_state"]["nodes"].append("no_such_node_kind")
    missing = {k: val for k, val in base.items() if k != "noop_statement"}
    template = copy.deepcopy(base)
    template["operators"]["state_write"]["statement"] = "{nope}"
    long_text = (PROTOTYPE / "tables" / "rust.json").read_text(encoding="utf-8").rstrip("\n")
    while pc.line_count(long_text) < 41:
        long_text += "\n "
    assert pc.line_count(long_text) == 41
    cases = [("invalid_node_kind", json.dumps(bad_kind, indent=None)), ("missing_key", json.dumps(missing)),
             ("unknown_placeholder", json.dumps(template)), ("41_lines", long_text)]
    out = []
    for label, text in cases:
        path = work / f"neg-{label}.json"
        path.write_bytes(text.encode("utf-8"))
        out.append((label, str(path), "rust"))
    return out


def packs(args) -> dict:
    found = [(f"tables/{p.name}", str(p), p.stem) for p in sorted((PROTOTYPE / "tables").glob("*.json"))]
    found += [(f"d2/{p.name}", str(p), p.stem) for p in sorted((PROTOTYPE / "results/attempt2/d2/packs").glob("*.json"))]
    found += negative_packs(args.work)
    out = {}
    for label, path, language in found:
        ref = pc.check_pack(Path(path).read_bytes().decode("utf-8"), language)
        got = spike(args, "check-pack", path, "--language", language)
        brief = lambda r: {"admitted": r["admitted"], "failed": [c["name"] for c in r["checks"] if not c["pass"]]}  # noqa: E731
        details = [{"check": a["name"], "prototype": a["detail"], "spike": b["detail"]}
                   for a, b in zip(ref["checks"], got["checks"]) if a["detail"] != b["detail"]]
        out[label] = {"prototype": brief(ref), "spike": brief(got), "identical": brief(ref) == brief(got)
                      and [c["name"] for c in ref["checks"]] == [c["name"] for c in got["checks"]],
                      "detail_differences": details}
    return out


def classification(args, scopes) -> dict:
    spec_all = json.loads((PROTOTYPE / "controls.json").read_text(encoding="utf-8"))["scopes"]
    out = {}
    for name, scope in scopes.items():
        original = e.read_text(scope, scope.files[0])
        inputs = {"identical": original, "appended_comment": original + "\n" + spec_all[name]["comment_text"] + "\n",
                  "unbalanced_brackets": original + "\n((((\n"}
        orig_path = args.work / f"classify-{name}-original.txt"
        orig_path.write_bytes(original.encode("utf-8"))
        rows = {}
        for label, mutated in inputs.items():
            path = args.work / f"classify-{name}-{label}.txt"
            path.write_bytes(mutated.encode("utf-8"))
            got = spike(args, "classify", "--packs", str(args.packs), "--language", scope.language,
                        "--original", str(orig_path), "--mutated", str(path))
            rows[label] = {"prototype": v.classify_mutant(scope.language, original, mutated), "spike": got}
        out[name] = rows
    return out


def trees(args, scopes) -> dict:
    """Every node of every scope file (exact text): kind, named, 1-based lines, character columns."""
    from ast_grep_py import SgRoot
    out = {}
    for name, scope in scopes.items():
        for file in scope.files:
            ref = []
            for n in v._walk(SgRoot(e.read_text(scope, file), scope.language).root()):
                r = n.range()
                ref.append([n.kind(), n.is_named(), [r.start.line + 1, r.start.column, r.end.line + 1, r.end.column]])
            got = spike(args, "tree", str(scope.root / file), "--language", scope.language)
            out[f"{name}:{file}"] = diff(ref, got)
    return out


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--bin", type=Path, required=True)
    parser.add_argument("--scopes-dir", type=Path, default=Path(r"D:\BuildArtifacts\lesscode\annotation-driver\scopes-ro"))
    parser.add_argument("--work", type=Path, default=Path(r"D:\BuildArtifacts\lesscode\annotation-driver\spike-parity"))
    parser.add_argument("--packs", type=Path, default=PROTOTYPE / "tables")
    args = parser.parse_args()
    args.work.mkdir(parents=True, exist_ok=True)
    scopes = v.load_scopes(args.scopes_dir)
    report = {"a_sites_mutants": sites_and_mutants(args, scopes), "b_controls": controls(args, scopes),
              "c_packs": packs(args), "d_classification": classification(args, scopes),
              "e_trees": trees(args, scopes)}
    (args.work / "parity.json").write_text(json.dumps(report, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    ok = True
    print("a. sites / mutants (prototype vs spike)")
    for name, r in report["a_sites_mutants"].items():
        s, m = r["sites"], r["mutants"]
        ok &= s["identical"] and m["identical"]
        print(f"  {name}: sites {s['prototype']}/{s['spike']} identical={s['identical']}; "
              f"mutants {m['prototype']}/{m['spike']} identical={m['identical']}")
    c = report["b_controls"]
    ok &= c["identical"]
    print(f"b. controls: prototype {c['prototype_as_expected']}, spike {c['spike_as_expected']}, "
          f"identical outcomes={c['identical']}")
    print("c. pack admission")
    for label, r in report["c_packs"].items():
        ok &= r["identical"]
        print(f"  {label}: prototype {r['prototype']} spike {r['spike']} identical={r['identical']}"
              f" detail_differences={len(r['detail_differences'])}")
    print("d. classification")
    for name, rows in report["d_classification"].items():
        same = all(x["prototype"] == x["spike"] for x in rows.values())
        ok &= same
        print(f"  {name}: " + ", ".join(f"{k} {x['prototype']}/{x['spike']}" for k, x in rows.items()))
    print("e. full syntax trees of the scope files (kind, named, range of every node)")
    for name, r in report["e_trees"].items():
        ok &= r["identical"]
        print(f"  {name}: nodes {r['prototype']}/{r['spike']} identical={r['identical']}")
    print(f"report: {args.work / 'parity.json'}")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
