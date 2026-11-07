"""E4 adapter (S10, R30): frozen strata imported from the E4 lifecycle oracle.
Record = e4 recorder semantics (no scope-record guard, scope hashes in meta); accept =
the run_e4 arm (probe + testinfra seeds + fresh replay covering the golden)."""
import json
import os
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
DC = HERE.parent
LESSCODE = DC.parents[2]
E1B, E2, E4 = (DC / d for d in ("e1b-facdrone-evidence-publication", "e2-facdrone-edge-census", "e4-lifecycle"))
RUNS = LESSCODE / ".runs/e4-lifecycle"
VENV = LESSCODE / ".runs/e2-facdrone/venv/Scripts/python.exe"
EXTRAS = LESSCODE / ".runs/e1b-facdrone/extras"
sys.path[:0] = list(map(str, (E1B, E2, E4, HERE, EXTRAS)))
import e4_oracle as O  # noqa: E402
import proto_core as P  # noqa: E402

MARKS = "not port_baseline and not live_broker"
PACKAGES = ("research", "reporting", "dash", "kernel", "factor-kit", "factor-comm-term-structure", "factor-short-reversal")
TESTINFRA, SEEDS_JSON = E4 / "results" / "testinfra.json", E4 / "results" / "seeds.json"
GOLDEN = O.GOLDEN if O.GOLDEN.exists() else RUNS / "golden-a.json"
PROTO_TEST = HERE / "proto_replay_test.py"
ROUND_ENV = "E4_ROUND"


def seeds() -> dict:
    d = json.loads(TESTINFRA.read_text(encoding="utf-8"))
    s, x = d["seeds"], d["deselections"]
    dropped = {f: "gate --ignore" for f in s["ignored_by_gates"]} | {
        f: "collection error on this host (fcntl)" for f in x["collection_errors"] if f in s["direct"]}
    files = [f for f in s["direct"] if f not in dropped]
    bad = sorted(n for v in x["resolved_ids"].values() for n in v if n.split("::")[0] in files)
    out = {"seeds": files, "dropped": dropped, "deselect": bad, "marks": MARKS,
           "testinfra_sha256": __import__("hashlib").sha256(TESTINFRA.read_bytes()).hexdigest()}
    SEEDS_JSON.write_text(json.dumps(out, indent=1), encoding="utf-8")
    return out


def meta_of(seqs: list) -> dict:
    import hashlib
    import hypothesis, sqlalchemy
    seed_sigs = set().union(*(P.signatures(s) for s in seqs if s["origin"] == "seed"))
    gen_sigs = set().union(set(), *(P.signatures(s) for s in seqs if s["origin"] == "generated"))
    files = {f: hashlib.sha256((O.ROOT / f).read_bytes()).hexdigest() for f in sorted(O.SCOPE)}
    return {"hypothesis_seed": O.SEED, "generated_requested": O.N_GENERATED, "python": sys.version,
            "hypothesis": hypothesis.__version__, "sqlalchemy": sqlalchemy.__version__, "root": str(O.ROOT),
            "scope_sha256": files, "alphabet": sorted(a[3:] for a in dir(O.Env) if a.startswith("op_")),
            "sequences": len(seqs), "operations": sum(len(s["ops"]) for s in seqs),
            "seed_signatures": sorted(seed_sigs), "generated_only_signatures": sorted(gen_sigs - seed_sigs),
            "harness_suspects": sorted(x for x in seed_sigs | gen_sigs if any(
                e in x for e in ("AttributeError", "KeyError", "NameError", "UnboundLocalError", "TypeError")))}


def record(out: Path, only: set | None = None) -> dict:
    import hashlib
    seqs = [s for s in ([{"name": k, "origin": "seed", "start": None, "ops": v} for k, v in O.SEEDS]
                        + O.generated(O.N_GENERATED)) if not only or s["name"] in only]
    with P.embedded_engine(E2) as engine:
        env = P.attached(O.Env, engine, O.ROOT, listen_all=True)
        try:
            [s.update(P.run_sequence(env, s, dict(O.SEEDS))) for s in seqs]
        finally:
            P.detach(engine, env, listen_all=True)
        meta = meta_of(seqs)
    digest, body = P.write_golden(out, meta, seqs)
    return {k: meta[k] for k in ("sequences", "operations")} | {"sha256_lf_body": digest, "bytes": len(body.encode())}


def replay(golden: Path | None = None, only: set | None = None) -> str | None:
    with P.embedded_engine(E2) as engine:
        return P.replay_engine(O.Env, engine, O.ROOT, dict(O.SEEDS), golden or GOLDEN, only, listen_all=True)


def run_replay_target(pg_engine, rnd: str | None = None) -> dict:
    out = E4 / "results" / f"sql-divergences-round{rnd if rnd is not None else os.environ.get('E4_ROUND', '0')}.json"
    return P.replay_to_l2(O.Env, pg_engine, O.ROOT, dict(O.SEEDS), [GOLDEN], out, listen_all=True)


def accept(root: Path, rnd: str = "0", deadline: str | None = None) -> int:
    os.environ["E4_ROUND"] = rnd
    sel = seeds()
    files, bad = sel["seeds"], sel["deselect"]
    groups = P.build_groups(files, bad, PACKAGES, PROTO_TEST, HERE)
    env = P.arm_env(root, HERE, [E4, E1B, EXTRAS, E2],
                    {"PROTO_SCOPE": "e4", "E4_SOURCE_ROOT": str(root), "E1B_SOURCE_ROOT": str(root), "E4_ROUND": rnd})
    P.probe(VENV, root, "facdrone.shells.lifecycle_store", env)
    l2, t0 = E4 / "results" / f"sql-divergences-round{rnd}.json", time.time()
    want = json.loads(Path(GOLDEN).read_text(encoding="utf-8"))["meta"]["sequences"]
    print("e4 accept: groups " + json.dumps([[c, sum(1 for x in g if x.endswith(".py"))] for c, g in groups]), flush=True)
    code = P.run_groups(VENV, root, groups, MARKS, env, None if deadline is None else time.monotonic() + float(deadline))
    rep = json.loads(l2.read_text(encoding="utf-8")) if l2.exists() else {}
    fresh = rep.get("written", 0) > t0 and rep.get("sequences") == want
    print(f"e4 accept: groups exit {code}; replay fresh={fresh} sequences={rep.get('sequences')}/{want} "
          f"operations={rep.get('operations')} level2_ops={rep.get('level2_ops')}", flush=True)
    return code if code else (0 if fresh else 1)


if __name__ == "__main__":
    a = sys.argv[1:]
    if a[0] == "record":
        print(json.dumps(record(Path(a[a.index("--out") + 1]) if "--out" in a else HERE / "results" / "proto-e4-golden.json",
                                set(a[a.index("--only") + 1].split(",")) if "--only" in a else None), indent=1))
    elif a[0] == "replay":
        d = replay(Path(a[a.index("--golden") + 1]) if "--golden" in a else None)
        print(d or "replay: no divergence")
        sys.exit(0 if d is None else 1)
    elif a[0] == "seeds":
        print(json.dumps(seeds(), indent=1))
    else:
        opts = dict(zip(a[1::2], a[2::2]))
        sys.exit(accept(Path(a[1]).resolve(), opts.get("--round", "0"), opts.get("--deadline")))
