"""E1c input source for the E1b oracle: database rows and history objects realised from the CrossHair
counterexamples, as new operations on e1b_oracle.Env (e1b_oracle.py itself is unchanged).
  e1c_source.py record    record the original's observations to results/golden-e1c.json (+ .sha256)
  e1c_source.py replay    replay them (E1B_SOURCE_ROOT selects the root, as in E1b)"""
import hashlib, json, sys
from pathlib import Path

import e1b_oracle as O
from e1b_oracle import IDENTITY, NOW, live
from facdrone.schema import LIVE_DECISION_EVENT
from facdrone_wire.instructions import InstructionPayload

HERE = Path(__file__).resolve().parent
GOLDEN = HERE / "results" / "golden-e1c.json"
A, S = IDENTITY.account_id, IDENTITY.strategy_id
ULID = lambda n: "01E1C" + f"{n:021d}"  # solver ids -> valid ULIDs (equal ids map to equal ULIDs)


def instr(n, kind, parent, venue):
    return {"id": ULID(n), "parent": parent, "kind": kind, "target": {"symbol": venue, "venue_symbol": venue},
            "grant": {"n": 1, "children": ["market_order"]}}


def wire(v):  # a value as it arrives from a JSON wire: strings are fresh objects, never the interned literal
    return json.loads(json.dumps(v))


def op_rows(self, o):
    with self.engine.begin() as c:
        for i, r in enumerate(o["rows"]):
            c.execute(LIVE_DECISION_EVENT.insert().values(
                decision_id=r.get("decision_id", f"e1c:{o.get('tag', 'r')}:{i}"), account_id=r.get("account", A),
                strategy_id=r.get("strategy", S), committed_version=r.get("version", 1), as_of=NOW, created_at=NOW,
                payload_json=r["payload"], flow=r["flow"]))
    return len(o["rows"])


def op_reader(self, o):  # ProtocolEvidenceReader.<fn>(identity, history) over the stored rows
    history = [InstructionPayload.model_validate(h) for h in o["history"]]
    return getattr(live.ProtocolEvidenceReader(self.engine), o["fn"])(IDENTITY, history)


def op_ihist(self, o):
    return live.SqlLiveStateRepository(self.engine).instruction_history(IDENTITY)


def op_served(self, o):
    return live.decision_served(live.SqlLiveStateRepository(self.engine), IDENTITY, o["decision_id"])


def op_commit_instr(self, o):  # a decision whose payload carries instructions (the _shadow_seed path)
    cur, bodies = self.state, wire(o["instructions"]) if o.get("wire") else o["instructions"]
    did = f"{O.WRITER_NS}:v{cur.version + 1}"
    nxt = O.replace(cur, version=cur.version + 1, last_decision_id=did)
    self.state = self.repo.commit(cur, nxt, live.DecisionEventRecord(did, IDENTITY, nxt.version, NOW, {"instructions": bodies}))
    return O._state(self.state)


for _f in (op_rows, op_reader, op_ihist, op_served, op_commit_instr):
    setattr(O.Env, _f.__name__, _f)

ROW = lambda flow, payload, n, version=0: {"flow": flow, "payload": payload, "version": version, "decision_id": f"e1c:{n}"}
SEQUENCES = [  # realised counterexamples (mapping in the README); each starts from an empty database
    # ch3-418: two instruction rows with the same key (solver key '\x00..\x01..' -> 'k') and version 0, one flow-NULL row
    ("C418-superseded-instruction", [
        {"op": "rows", "rows": [ROW("instruction", {"key": "k", "instruction": instr(0, "open", "p", "v")}, 0),
                                ROW("instruction", {"key": "k", "instruction": instr(0, "open", "p", "v")}, 1),
                                ROW(None, {"instruction": instr(0, "open", "p", "v")}, 2)]},
        {"op": "ihist"}]),
    # supplementary, diag-probe-241: legacy attempt, rest.parent '' < turn_id '\x00\x00' (-> 'a' < 'b'), venue equal
    ("X241-legacy-parent-order", [
        {"op": "rows", "rows": [ROW("attempt", {"representation": "legacy", "turn_id": "b", "venue_symbol": "v",
                                                "child_kind": "limit_order", "order_id": "o1"}, 0)]},
        {"op": "reader", "fn": "resting_orders", "history": [instr(1, "rest", "a", "v")]}]),
]


def record(engine, out: Path) -> dict:
    rec = O.scope.record(O.ROOT)
    if rec["changed"]:
        raise RuntimeError(f"record only from the unmodified copy; changed: {rec['changed']}")
    seqs = [{"name": n, "origin": "e1c", "start": None, "ops": ops} for n, ops in SEQUENCES]
    env = O._attached(engine)
    try:
        [s.update(O.run_sequence(env, s)) for s in seqs]
    finally:
        O.event.remove(engine, "before_cursor_execute", env.on_sql)
    meta = {"source": "e1c_source.py", "e1b_golden_sha256": (O.GOLDEN.parent / "golden-trace.json.sha256").read_text().split()[0],
            "python": sys.version, "scope": rec, "sequences": len(seqs), "operations": sum(len(s["ops"]) for s in seqs),
            "errors": sorted({f"{s['name']} op {i}: {ob['error'][0]}" for s in seqs for i, ob in enumerate(s["observations"]) if "error" in ob})}
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
