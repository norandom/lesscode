"""E4 replay target (run_e4.py): the golden sequences on Facdrone's `pg_engine`, with Level 1 (result or error,
tables, files) failing on the first divergence and Level 2 (`sql`) collected per operation into
results/sql-divergences-round<E4_ROUND>.json without failing. The golden file is results/golden-e4.json when it is
committed (under 1 MB), else .runs/e4-lifecycle/golden-a.json, whose .sha256 sidecar is committed instead."""
import json, os, sys, time
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import e4_oracle as O  # noqa: E402
GOLDEN = O.GOLDEN if O.GOLDEN.exists() else HERE.parents[3] / ".runs/e4-lifecycle/golden-a.json"


def test_e4_replay(pg_engine) -> None:
    trace, env, l2, n = json.loads(GOLDEN.read_text(encoding="utf-8")), O._attached(pg_engine), [], {"sequences": 0, "operations": 0}
    out = HERE / "results" / f"sql-divergences-round{os.environ.get('E4_ROUND', '0')}.json"
    try:
        for seq in trace["sequences"]:
            got = O.run_sequence(env, seq)
            d = O.first_diff(seq["start_tables"], got["start_tables"])
            assert d is None, f"{seq['name']} start state: {d}"
            for i, (o, a, b) in enumerate(zip(seq["ops"], seq["observations"], got["observations"])):
                sa, sb = a.pop("sql", None), b.pop("sql", None)
                d = O.first_diff(a, b)
                assert d is None, f"{seq['name']} op {i} {O.canon(o)[:160]}: {d}"
                if O.canon(sa) != O.canon(sb):
                    l2.append({"sequence": seq["name"], "op_index": i, "op": o, "recorded": sa, "replayed": sb})
                n["operations"] += 1
            n["sequences"] += 1
    finally:
        O._detach(env)
        out.write_text(json.dumps({"written": time.time(), "root": str(O.ROOT), "golden": str(GOLDEN), **n, "level2_ops": len(l2),
                                   "divergences": l2}, indent=1), encoding="utf-8")
