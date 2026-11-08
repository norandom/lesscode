"""E5 replay target (run_e5.py): the golden sequences on Facdrone's `pg_engine`.

Evaluates Level 1 (result/error, tables, files) failing on first divergence,
and collects Level 2 (sql) into results/sql-divergences-round<E5_ROUND>.json.
"""
import json, os, sys, time
from pathlib import Path

HERE = Path(__file__).resolve().parent
E4_DIR = HERE.parent / "e4-lifecycle"
sys.path.insert(0, str(E4_DIR))
import e4_oracle as O  # noqa: E402

GOLDEN = HERE.parents[3] / ".runs/e4-lifecycle/golden-a.json"


def test_e5_replay(pg_engine) -> None:
    trace = json.loads(GOLDEN.read_text(encoding="utf-8"))
    env = O._attached(pg_engine)
    l2 = []
    n = {"sequences": 0, "operations": 0}
    out = HERE / "results" / f"sql-divergences-round{os.environ.get('E5_ROUND', '0')}.json"
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
        out.write_text(json.dumps({
            "written": time.time(),
            "root": str(O.ROOT),
            "golden": str(GOLDEN),
            **n,
            "level2_ops": len(l2),
            "divergences": l2,
        }, indent=1), encoding="utf-8")
