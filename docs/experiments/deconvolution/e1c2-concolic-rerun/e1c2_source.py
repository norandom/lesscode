"""E1c-2 oracle alphabet: one new operation, `readers`, on e1b_oracle.Env (e1b_oracle.py and e1c_source.py
unchanged, imported). It calls opening_tickets, resting_orders, attempt_history and decision_served (every
stored decision id plus an absent one) on the current state; each reader's failure is its own observation.
Sequences: the readers on the recorded E1b seed states (R-*), then the realised solver inputs (C-*).
  e1c2_source.py record | replay      results/golden-e1c2.json (+ .sha256); E1B_SOURCE_ROOT as in E1b"""
import hashlib, json, sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "e1c-concolic-rung"))
import e1c_source as E  # noqa: E402  (registers E1c's rows/reader/ihist/served/commit_instr operations)
from e1c_source import O, IDENTITY, ROW, instr, live  # noqa: E402
from facdrone_wire.instructions import InstructionPayload  # noqa: E402
from sqlalchemy import select  # noqa: E402
GOLDEN = HERE / "results" / "golden-e1c2.json"
def _try(f):
    try:
        return f()
    except Exception as e:  # noqa: BLE001 — a reader's failure is an observation
        return {"error": [type(e).__name__, str(e)]}
def op_readers(self, o):
    repo, reader = live.SqlLiveStateRepository(self.engine), live.ProtocolEvidenceReader(self.engine)
    hist = _try(lambda: repo.instruction_history(IDENTITY) if "history" not in o
                else [InstructionPayload.model_validate(h) for h in o["history"]])
    hist = hist if isinstance(hist, list) else []
    with self.engine.connect() as c:
        dids = sorted(c.execute(select(live.LIVE_DECISION_EVENT.c.decision_id)).scalars()) + ["e1c2-absent"]
    return {"opening_tickets": _try(lambda: reader.opening_tickets(IDENTITY, hist)),
            "resting_orders": _try(lambda: reader.resting_orders(IDENTITY, hist)),
            "attempt_history": _try(lambda: reader.attempt_history(IDENTITY)),
            "decision_served": {d: _try(lambda d=d: live.decision_served(repo, IDENTITY, d)) for d in dids}}
O.Env.op_readers = op_readers
R = {"op": "readers"}
SEQUENCES = [(f"R-{s}", s, [R]) for s in ("S01-live-reuse-grow-restart", "S02-namespace-change", "S03-rollback-then-retry",
                                          "S04-live-legacy-v1", "S05-legacy-dict-refs-refused", "S06-stale-and-absent",
                                          "S11-delta-chain-keyframe")]
I, LEG = lambda n, k="open": instr(n, k, "p", "v"), lambda **b: {"representation": "legacy", "child_kind": "limit_order", "order_id": "o1", **b}
SEQUENCES += [(n, None, ops) for n, ops in [  # realised counterexamples (mapping in the README); each starts from an empty database
    ("C418-null-and-empty-keys", [{"op": "rows", "rows": [ROW("instruction", {"instruction": I(0)}, 0), ROW("instruction", {"instruction": I(0)}, 1),
                                                          ROW("instruction", {"key": "", "instruction": I(0)}, 2)]}, {"op": "ihist"}, R]),
    ("C584-wire-set-stops", [{"op": "boot"}, {"op": "commit_instr", "wire": True, "instructions": [I(0, "set_stops")]}, R]),
    ("C240-legacy-parent-id", [{"op": "rows", "rows": [ROW("attempt", LEG(parent_id="p", venue_symbol="p"), 0)]},
                               {"op": "readers", "history": [instr(1, "rest", "p", "p")]}]),
    ("C241-legacy-parent-order", [{"op": "rows", "rows": [ROW("attempt", LEG(parent_id="pa", venue_symbol="p"), 0)]},
                                  {"op": "readers", "history": [instr(2, "rest", "p", "p")]}])]]
def record(engine, out: Path) -> dict:
    rec = O.scope.record(O.ROOT)
    if rec["changed"]:
        raise RuntimeError(f"record only from the unmodified copy; changed: {rec['changed']}")
    seqs, env = [{"name": n, "origin": "e1c2", "start": s, "ops": ops} for n, s, ops in SEQUENCES], O._attached(engine)
    try:
        [s.update(O.run_sequence(env, s)) for s in seqs]
    finally:
        O.event.remove(engine, "before_cursor_execute", env.on_sql)
    errs = sorted({f"{s['name']} op {i}: {ob['error'][0]}" for s in seqs for i, ob in enumerate(s["observations"]) if "error" in ob})
    meta = {"source": "e1c2_source.py", "python": sys.version, "scope": rec, "sequences": len(seqs), "errors": errs,
            "operations": sum(len(s["ops"]) for s in seqs), "e1c_golden_sha256": E.GOLDEN.with_suffix(".json.sha256").read_text().split()[0]}
    body = json.dumps({"meta": meta, "sequences": seqs}, sort_keys=True, indent=0, allow_nan=True)
    out.write_text(body, encoding="utf-8")
    digest = hashlib.sha256(body.encode()).hexdigest()
    Path(str(out) + ".sha256").write_text(f"{digest}  {out.name}\n", encoding="ascii")
    return {k: meta[k] for k in ("sequences", "operations", "errors")} | {"sha256": digest}
if __name__ == "__main__":
    with O.embedded_engine() as engine:
        if sys.argv[1] == "record":
            print(json.dumps(record(engine, GOLDEN), indent=1))
            sys.exit(0)
        print(d := O.replay_engine(engine, GOLDEN) or "replay: no divergence")
        sys.exit(0 if d == "replay: no divergence" else 1)
