"""E3 adapter (S10, R30): frozen strata imported from the E1b/E1c/E1c-2 oracles.
Record = E1b recorder semantics; accept = the run_e3 arm (R16 + full seed selection)."""
import json
import os
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
DC = HERE.parent
LESSCODE = DC.parents[2]
E1B, E1C, E1C2 = (DC / d for d in ("e1b-facdrone-evidence-publication", "e1c-concolic-rung", "e1c2-concolic-rerun"))
E2, E3 = DC / "e2-facdrone-edge-census", DC / "e3-evidence-regeneration"
sys.path[:0] = list(map(str, (E1B, E1C, E1C2, E2, HERE, LESSCODE / ".runs/e1b-facdrone/extras")))
import e1c2_source  # noqa: E402  (registers E1c/E1c-2 operations on the E1b Env, as test_e3_replay did)
import e1b_oracle as O  # noqa: E402
import proto_core as P  # noqa: E402
import run_arm  # noqa: E402  (frozen selection map + R16 baseline + constants)
import scope  # noqa: E402

VENV, MARKS, PACKAGES = run_arm.VENV, run_arm.MARKS, run_arm.PACKAGES
GOLDENS = (O.GOLDEN, e1c2_source.E.GOLDEN, e1c2_source.GOLDEN)
ROUND_ENV = "E3_ROUND"
SEL = json.loads(run_arm.SELECTION.read_text(encoding="utf-8"))
PROTO_TEST = HERE / "proto_replay_test.py"
QUERY_EVIDENCE = None  # recorded E3 rounds all have level2_ops=0 (R25); L2>0 without evidence refuses


def meta_of(seqs: list) -> dict:
    import hypothesis, sqlalchemy
    rec = scope.record(O.ROOT)
    seed_sigs = set().union(*(P.signatures(s) for s in seqs if s["origin"] == "seed"))
    gen_sigs = set().union(set(), *(P.signatures(s) for s in seqs if s["origin"] == "generated"))
    return {"hypothesis_seed": O.SEED, "generated_requested": O.N_GENERATED, "python": sys.version,
            "hypothesis": hypothesis.__version__, "sqlalchemy": sqlalchemy.__version__, "scope": rec,
            "sequences": len(seqs), "operations": sum(len(s["ops"]) for s in seqs),
            "seed_signatures": sorted(seed_sigs), "generated_only_signatures": sorted(gen_sigs - seed_sigs),
            "harness_suspects": sorted(x for x in seed_sigs | gen_sigs if any(
                e in x for e in ("AttributeError", "KeyError", "NameError", "UnboundLocalError")))}


def guard() -> None:
    rec = scope.record(O.ROOT)
    if rec["changed"]:
        raise RuntimeError(f"record only from the unmodified copy; changed: {rec['changed']}")


def record(out: Path, only: set | None = None) -> dict:
    seqs = [s for s in ([{"name": k, "origin": "seed", "start": None, "ops": v} for k, v in O.SEEDS]
                        + O.generated(O.N_GENERATED)) if not only or s["name"] in only]
    with P.embedded_engine(E2) as engine:
        env = P.attached(O.Env, engine, O.ROOT)
        try:
            [s.update(P.run_sequence(env, s, dict(O.SEEDS))) for s in seqs]
        finally:
            P.detach(engine, env)
        meta = meta_of(seqs)
    digest, _ = P.write_golden(out, meta, seqs)
    return {k: meta[k] for k in ("sequences", "operations")} | {"sha256_lf_body": digest}


def replay(golden: Path | None = None, only: set | None = None) -> str | None:
    with P.embedded_engine(E2) as engine:
        return P.replay_engine(O.Env, engine, O.ROOT, dict(O.SEEDS), golden or GOLDENS[0], only)


def run_replay_target(pg_engine, rnd: str | None = None) -> dict:
    out = E3 / "results" / f"sql-divergences-round{rnd if rnd is not None else os.environ.get('E3_ROUND', '0')}.json"
    return P.replay_to_l2(O.Env, pg_engine, O.ROOT, dict(O.SEEDS), list(GOLDENS), out)


def accept(root: Path, rnd: str = "0", deadline: str | None = None) -> int:
    os.environ["E3_ROUND"] = rnd
    files, bad = SEL["seeds"], [n for n in SEL["deselect"] if n.split("::")[0] in SEL["seeds"]]
    print(f"e3 accept: {len(files)} seed files, {len(bad)} deselections", flush=True)
    groups = P.build_groups(files, bad, PACKAGES, PROTO_TEST, HERE)
    env = P.arm_env(root, HERE, [E1B, E1C, E1C2, E2, run_arm.EXTRAS],
                    {"PROTO_SCOPE": "e3", "E1B_SOURCE_ROOT": str(root), "E3_ROUND": rnd})
    P.probe(VENV, root, "facdrone.shells.cppi.state_repository", env)
    r16 = subprocess.run([str(VENV), "-B", str(scope.R16_CHECK), str(root)], env=env, capture_output=True, text=True)
    if r16.returncode or r16.stdout != run_arm.R16_BASELINE.read_text(encoding="utf-8"):
        print(f"e3 accept: R16 harness differs from its baseline (exit {r16.returncode})")
        return 1
    l2, t0 = E3 / "results" / f"sql-divergences-round{rnd}.json", time.time()
    print("e3 accept: groups " + json.dumps([[c, sum(1 for x in g if x.endswith(".py"))] for c, g in groups]), flush=True)
    code = P.run_groups(VENV, root, groups, MARKS, env, None if deadline is None else time.monotonic() + float(deadline))
    rep = json.loads(l2.read_text(encoding="utf-8")) if l2.exists() else {}
    fresh = rep.get("written", 0) > t0 and rep.get("sequences") == 64
    code, reasons = P.verdict(code, fresh, rep.get("level2_ops", 0), QUERY_EVIDENCE)
    print(f"e3 accept: verdict exit {code}; fresh={fresh} sequences={rep.get('sequences')}/64 "
          f"operations={rep.get('operations')} level2_ops={rep.get('level2_ops')}; {reasons or 'accepted'}", flush=True)
    return code


if __name__ == "__main__":
    a = sys.argv[1:]
    if a[0] == "record":
        print(json.dumps(record(Path(a[a.index("--out") + 1]) if "--out" in a else HERE / "results" / "proto-e3-golden.json",
                                set(a[a.index("--only") + 1].split(",")) if "--only" in a else None), indent=1))
    elif a[0] == "replay":
        d = replay(Path(a[a.index("--golden") + 1]) if "--golden" in a else None)
        print(d or "replay: no divergence")
        sys.exit(0 if d is None else 1)
    else:
        opts = dict(zip(a[2::2], a[3::2]))
        sys.exit(accept(Path(a[1]).resolve(), opts.get("--round", "0"), opts.get("--deadline")))
