# S9: compare the two frozen surface contracts for E3 and E4 (R26 amendment 1 evidence).
# Contract A = tree imports (frozen results/public-surface.json, surface.py/surface4.py).
# Contract B = A + recorder (oracle) imports/usages, scanned with the SAME existing
# scan()/kinds() functions. Evidence only: no adoption, no surface file is rewritten.
import ast, hashlib, json, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent                  # docs/experiments/deconvolution/native-prototype
DC = HERE.parent                                        # docs/experiments/deconvolution
LESSCODE = DC.parents[2]
ROOT = LESSCODE / ".runs/e2-facdrone/source"            # frozen specimen copy (untracked, sidecar-verified)
OUT = HERE / "results"

sys.path.insert(0, str(DC / "e3-evidence-regeneration"))
sys.path.insert(0, str(DC / "e4-lifecycle"))
import surface as s3   # noqa: E402  (existing E3 apparatus)
import surface4 as s4  # noqa: E402  (existing E4 apparatus)

SCOPES = [
    {"scope": "E3", "set": "five decision_evidence modules", "mod": s3.MOD, "kinds": s3.kinds,
     "surface_a": DC / "e3-evidence-regeneration/results/public-surface.json",
     "recorder": DC / "e1b-facdrone-evidence-publication/e1b_oracle.py", "scan": s3.scan},
    {"scope": "E4", "set": "seven lifecycle modules", "mod": s4.MOD, "kinds": s4.kinds,
     "surface_a": DC / "e4-lifecycle/results/public-surface.json",
     "recorder": DC / "e4-lifecycle/e4_oracle.py", "scan": s4.scan},
]

def main() -> None:
    if not ROOT.is_dir():
        raise SystemExit(f"frozen specimen root missing: {ROOT} (artifact bytes are required evidence)")
    report = {"receipt": "s9-surface-contracts-1", "purpose": "R26 amendment 1 evidence: tree-only (A) vs tree+recorder (B) surface contracts",
              "root": str(ROOT), "scopes": {}, "notes": [
                  "B is computed by scanning the recorder oracle with the same scan()/kinds() the frozen surface derivers use; no surface file is rewritten and no contract is adopted."]}
    for spec in SCOPES:
        a = json.loads(spec["surface_a"].read_text(encoding="utf-8"))
        kinds = {m: spec["kinds"](ROOT, f) for m, f in spec["mod"].items()}
        additions, unresolved_hits = {}, {m: [] for m in spec["mod"]}
        rel = spec["recorder"].relative_to(LESSCODE).as_posix()
        def add(m, name, via, m_kinds=kinds, a=a):
            k = "module" if name == "<module>" else m_kinds[m].get(name)
            if name in a["unresolved"].get(m, {}):
                unresolved_hits[m].append(name)
            existing = a["modules"][m]["names"].get(name)
            if existing is None and not (k is None and via == "attribute"):
                e = additions.setdefault(m, {}).setdefault(name, {"kind": k or "missing", "via": [], "importers": []})
                if via not in e["via"]:
                    e["via"].append(via)
                if rel not in e["importers"]:
                    e["importers"].append(rel)
        tree = ast.parse(spec["recorder"].read_text(encoding="utf-8"))
        spec["scan"](tree, add)
        n_names = sum(len(v) for v in additions.values())
        report["scopes"][spec["scope"]] = {
            "set": spec["set"], "surface_a": spec["surface_a"].relative_to(LESSCODE).as_posix(),
            "surface_a_modules": {m: len(d["names"]) for m, d in a["modules"].items()},
            "recorder": rel,
            "b_additions": additions, "b_addition_count": n_names,
            "additions_already_unresolved_in_a": {m: sorted(set(v)) for m, v in unresolved_hits.items() if v},
            "named_checks": {},
        }
        if spec["scope"] == "E4":
            lp = "facdrone.shells.lifecycle_projection"
            report["scopes"]["E4"]["named_checks"]["labeled_reason_missing_in_A"] = "labeled_reason" not in a["modules"][lp]["names"]
            report["scopes"]["E4"]["named_checks"]["labeled_reason_added_by_B"] = "labeled_reason" in additions.get(lp, {})
    OUT.mkdir(parents=True, exist_ok=True)
    body = json.dumps(report, indent=1, sort_keys=True)
    out = OUT / "s9-surface-contracts.json"
    out.write_text(body, encoding="utf-8", newline="\n")
    digest = hashlib.sha256(body.encode()).hexdigest()
    Path(str(out) + ".sha256").write_text(f"{digest}  s9-surface-contracts.json\n", encoding="ascii", newline="\n")
    for scope, data in report["scopes"].items():
        print(f"{scope}: A names/module={data['surface_a_modules']} B additions={data['b_addition_count']}")
        for m, adds in data["b_additions"].items():
            print(f"  +{m}: {sorted(adds)}")
        print(f"  unresolved-in-A additions: {data['additions_already_unresolved_in_a']}")
    print(f"receipt: {out}")

if __name__ == "__main__":
    main()
