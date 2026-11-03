"""E1c R5 capture hook: wraps the six original functions around the ten targets and appends each call's
plain-data inputs and output to $E1C_CAPTURE (jsonl). No source file is changed.
  pytest -p e1c_capture ...  (seed-test corpus)      e1c_capture.py golden   (E1b golden-trace replay)"""
import json, os, sys

from sqlalchemy import select

from facdrone.schema import LIVE_DECISION_EVENT
from facdrone.shells.cppi import state_repository as live

OUT = os.environ.get("E1C_CAPTURE")


def _put(fn, **kw):
    if OUT:
        with open(OUT, "a", encoding="utf-8") as fh:
            fh.write(json.dumps({"fn": fn, **kw}, default=lambda v: {"__unjson__": repr(v)}) + "\n")


def _rows(engine):  # the whole table in heap order; the lifted function applies the WHERE clause
    with engine.connect() as c:
        return [[r.account_id, r.strategy_id, r.flow, r.committed_version, r.decision_id, r.as_of.isoformat(), r.payload_json]
                for r in c.execute(select(LIVE_DECISION_EVENT))]


hist = lambda h: [[i.kind, i.id, i.parent, i.target.venue_symbol] for i in h]  # the recorded conversion
ident = lambda i: [i.account_id, i.strategy_id]
dumps = lambda xs: [x.model_dump(mode="json") for x in xs]


def _reader(name):
    orig = getattr(live.ProtocolEvidenceReader, name)

    def wrapped(self, identity, history):
        rows, out = _rows(self.engine), orig(self, identity, history)
        _put(name, rows=rows, ident=ident(identity), history=hist(history), out=out)
        return out
    setattr(live.ProtocolEvidenceReader, name, wrapped)


def _install():
    for name in ("opening_tickets", "resting_orders"):
        _reader(name)
    placed, shadow, served = live._placed_under, live._shadow_seed, live.decision_served
    orig_hist = live.SqlLiveStateRepository.instruction_history

    def placed_under(rest, body):
        out = placed(rest, body)
        _put("placed_under", rest=hist([rest])[0], body=body, out=out)
        return out

    def shadow_seed(parent, candidates):
        out = shadow(parent, candidates)
        _put("intention_ids", candidates=candidates, out=out.payload["intention_ids"])
        return out

    def decision_served(repo, identity, decision_id):
        rows, out = _rows(repo.engine), served(repo, identity, decision_id)
        _put("decision_served", rows=rows, ident=ident(identity), decision_id=decision_id, out=out)
        return out

    def instruction_history(self, identity):
        rows, out = _rows(self.engine), orig_hist(self, identity)
        _put("instruction_history", rows=rows, ident=ident(identity), out=dumps(out))
        return out
    live._placed_under, live._shadow_seed, live.decision_served = placed_under, shadow_seed, decision_served
    live.SqlLiveStateRepository.instruction_history = instruction_history


_install()

if __name__ == "__main__" and sys.argv[1:] == ["golden"]:
    import e1b_oracle
    sys.exit(e1b_oracle.main(["replay"]))
