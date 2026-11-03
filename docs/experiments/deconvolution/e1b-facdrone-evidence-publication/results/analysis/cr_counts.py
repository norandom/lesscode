"""Count one or two cosmic-ray dumps (JSONL) for E1b: cr_counts.py DUMP_A [DUMP_B] > counts.json
Amendments 2.6, 6 and the owner's per-line amendment: `run_arm: INVALID` outputs are void;
`deadline exceeded`/`timeout` count as killed and are listed separately; incompetent excluded;
uncovered mutants (no seed file executes a changed line) count as survived in the primary figure,
their observed outcome is reported beside it. Pending items (no result) are reported as such."""
import json, sys
from pathlib import Path

HERE = Path(__file__).resolve().parents[2]
SAMPLE = {(m["module"], m["operator"], m["occurrence"]): m for m in json.loads((HERE / "results/mutant-sample.json").read_text())["sample"]}
LINES = json.loads((HERE / "results/selection-map.json").read_text())["lines"]


def outcomes(dump: Path) -> dict:
    out = {}
    for line in dump.read_text(encoding="utf-8").splitlines():
        item, res = json.loads(line)
        mu = item["mutations"][0]
        key = (mu["module_path"].replace("\\", "/"), mu["operator_name"], mu["occurrence"])
        spec = SAMPLE[key]
        uncovered = not any(LINES[spec["module"]].get(str(n)) for n in spec["lines"])
        text = (res or {}).get("output") or ""
        if res is None:
            o = "pending"
        elif "run_arm: INVALID" in text:
            o = "void"
        elif str(res.get("worker_outcome")).lower() != "normal":
            o = "worker-" + str(res.get("worker_outcome"))
        elif str(res["test_outcome"]).lower() == "killed" and ("deadline exceeded" in text or text.strip() == "timeout"):
            o = "timeout"
        else:
            o = str(res["test_outcome"]).lower()
        out["|".join(map(str, key))] = {"observed": o, "uncovered": uncovered, "row": spec["row"]}
    return out


def summary(oc: dict, keys=None) -> dict:
    keys = list(oc) if keys is None else keys
    c = {}
    for k in keys:
        o = oc[k]["observed"]
        prim = "survived" if oc[k]["uncovered"] and o in ("killed", "timeout", "survived") else o
        c[prim] = c.get(prim, 0) + 1
    kill = c.get("killed", 0) + c.get("timeout", 0)
    obs = {}
    for k in keys:
        obs[oc[k]["observed"]] = obs.get(oc[k]["observed"], 0) + 1
    okill = obs.get("killed", 0) + obs.get("timeout", 0)
    unc = {o: sum(1 for k in keys if oc[k]["uncovered"] and oc[k]["observed"] == o) for o in {oc[k]["observed"] for k in keys}}
    return {"n": len(keys), "primary": c, "kill_rate": kill / (kill + c.get("survived", 0)) if kill + c.get("survived", 0) else None,
            "observed": obs, "observed_kill_rate": okill / (okill + obs.get("survived", 0)) if okill + obs.get("survived", 0) else None,
            "uncovered_observed": {o: n for o, n in unc.items() if n}}


if __name__ == "__main__":
    arms = {chr(65 + i): outcomes(Path(p)) for i, p in enumerate(sys.argv[1:])}
    res = {a: summary(oc) for a, oc in arms.items()}
    if len(arms) == 2:
        done = lambda oc, k: oc[k]["observed"] in ("killed", "survived", "timeout")
        both = [k for k in arms["A"] if k in arms["B"] and done(arms["A"], k) and done(arms["B"], k)]
        res["intersection"] = {"A": summary(arms["A"], both), "B": summary(arms["B"], both)}
        res["killed_only_by_B"] = [k for k in both if arms["A"][k]["observed"] == "survived" and arms["B"][k]["observed"] in ("killed", "timeout")]
        res["killed_only_by_A"] = [k for k in both if arms["B"][k]["observed"] == "survived" and arms["A"][k]["observed"] in ("killed", "timeout")]
    res["per_mutant"] = arms
    print(json.dumps(res, indent=1))
