"""E3 replay target (run_e3.py): the 64 golden sequences (E1b, E1c, E1c-2) on Facdrone's `pg_engine`, with
Level 1 (result or error, tables, files) failing on the first divergence and Level 2 (`sql`) collected per
operation into results/sql-divergences-round<E3_ROUND>.json without failing. Oracles imported unchanged."""
import json, os, sys, time
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path[:0] = [str(HERE.parent / d) for d in ("e1c-concolic-rung", "e1c2-concolic-rerun")]
import e1c2_source  # noqa: E402  (imports e1c_source and e1b_oracle; registers every operation on O.Env)
O, E1C = e1c2_source.O, e1c2_source.E
GOLDENS = (O.GOLDEN, E1C.GOLDEN, e1c2_source.GOLDEN)


def test_e3_replay(pg_engine) -> None:
    env, l2, n = O._attached(pg_engine), [], {"sequences": 0, "operations": 0}
    out = HERE / "results" / f"sql-divergences-round{os.environ.get('E3_ROUND', '0')}.json"
    try:
        for g in GOLDENS:
            for seq in json.loads(g.read_text(encoding="utf-8"))["sequences"]:
                got = O.run_sequence(env, seq)
                d = O.first_diff(seq["start_tables"], got["start_tables"])
                assert d is None, f"{g.name} {seq['name']} start state: {d}"
                for i, (o, a, b) in enumerate(zip(seq["ops"], seq["observations"], got["observations"])):
                    sa, sb = a.pop("sql", None), b.pop("sql", None)
                    d = O.first_diff(a, b)
                    assert d is None, f"{g.name} {seq['name']} op {i} {O.canon(o)[:160]}: {d}"
                    if O.canon(sa) != O.canon(sb):
                        l2.append({"golden": g.name, "sequence": seq["name"], "op_index": i, "op": o, "recorded": sa, "replayed": sb})
                    n["operations"] += 1
                n["sequences"] += 1
    finally:
        O.event.remove(pg_engine, "before_cursor_execute", env.on_sql)
        out.write_text(json.dumps({"written": time.time(), "root": str(O.ROOT), **n, "level2_ops": len(l2),
                                   "divergences": l2}, indent=1), encoding="utf-8")
