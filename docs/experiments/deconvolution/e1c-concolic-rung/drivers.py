"""CrossHair drivers: bounded plain-data domains around lifted.py; `o_*` original, `m<line>` mutant.
Bounds, by construction (amendment 3: rejecting guards starved the search): <= 3 rows, instructions and
candidates; free strings cut to 12 characters; the literals the predicates compare against are enumerated,
with one extra value, as int indexes taken modulo the enumeration (amendment 2: CrossHair 0.0.112 cannot
unify typing.Literal); an identity column is None for the oracle's identity literal or a free string.
Run: crosshair diffbehavior drivers.o_open drivers.m150 ..."""
from typing import Optional, Union

import lifted

ID = ("e1b-account", "e1b-strategy")  # the oracle's runtime identity
FLOW = (None, "attempt", "instruction", "decision", "other")
CHILD = (None, "market_order", "stop_attach", "limit_order", "other")
OUTCOME = (None, "rejected", "skipped", "unavailable", "accepted")
KIND = ("open", "rest", "set_stops", "other")
S, O = str, Optional[str]
# attempt body: child_kind, outcome (indexes), ticket, order_id, instruction_id, representation, turn_id, parent_id, venue_symbol
Body = tuple[int, int, O, O, O, O, O, O, O]
KEYS = ("child_kind", "outcome", "ticket", "order_id", "instruction_id", "representation", "turn_id", "parent_id", "venue_symbol")
Row = tuple[O, O, int, Body]  # account_id, strategy_id (None = identity), flow index, payload
Instr = tuple[int, S, S, S]  # kind index, id, parent, target.venue_symbol


def _t(x):  # a free string, cut to the bound
    return None if x is None else x[:12]


def _e(i: int, opts: tuple):
    return opts[i % len(opts)]


def _id(a: O, s: O) -> tuple:
    return (ID[0] if a is None else _t(a)), (ID[1] if s is None else _t(s))


def _body(b: Body) -> dict:
    vals = (_e(b[0], CHILD), _e(b[1], OUTCOME), *map(_t, b[2:]))
    return {k: v for k, v in zip(KEYS, vals) if v is not None}


def _instr(h: Instr) -> tuple:
    return (_e(h[0], KIND), *map(_t, h[1:]))


def _attempts(rows: list[Row], hist: list[Instr], fn, mut: int):
    return fn([(*_id(a, s), _e(f, FLOW), 0, "d", 0, _body(b)) for a, s, f, b in rows[:3]], ID,
              [_instr(h) for h in hist[:3]], mut)


def o_open(rows: list[Row], hist: list[Instr]) -> object: return _attempts(rows, hist, lifted.opening_tickets, 0)
def m150(rows: list[Row], hist: list[Instr]) -> object: return _attempts(rows, hist, lifted.opening_tickets, 150)
def m166(rows: list[Row], hist: list[Instr]) -> object: return _attempts(rows, hist, lifted.opening_tickets, 166)
def m169(rows: list[Row], hist: list[Instr]) -> object: return _attempts(rows, hist, lifted.opening_tickets, 169)
def o_rest(rows: list[Row], hist: list[Instr]) -> object: return _attempts(rows, hist, lifted.resting_orders, 0)
def m186(rows: list[Row], hist: list[Instr]) -> object: return _attempts(rows, hist, lifted.resting_orders, 186)
def m200(rows: list[Row], hist: list[Instr]) -> object: return _attempts(rows, hist, lifted.resting_orders, 200)


def o_placed(rest: Instr, body: Body) -> object: return lifted.placed_under(_instr(rest), _body(body), 0)
def m240(rest: Instr, body: Body) -> object: return lifted.placed_under(_instr(rest), _body(body), 240)
def m241(rest: Instr, body: Body) -> object: return lifted.placed_under(_instr(rest), _body(body), 241)


IRow = tuple[O, O, int, int, O, int]  # account_id, strategy_id, flow index, committed_version, payload key, instruction token


def _hist(rows: list[IRow], mut: int):
    pay = lambda k, t: {"instruction": t} if k is None else {"key": k, "instruction": t}
    return lifted.instruction_history([(*_id(a, s), _e(f, FLOW), v, "d", 0, pay(_t(k), t)) for a, s, f, v, k, t in rows[:3]], ID, mut)


def o_hist(rows: list[IRow]) -> object: return _hist(rows, 0)
def m418(rows: list[IRow]) -> object: return _hist(rows, 418)


Cand = Union[int, tuple[int, S]]  # a non-dict candidate, or {"kind": ((None,) + KIND)[index], "id": ...}


def _cands(cands: list[Cand], mut: int):
    kind = lambda i: {} if _e(i, (None, *KIND)) is None else {"kind": _e(i, (None, *KIND))}
    return lifted.intention_ids([c if isinstance(c, int) else {"id": _t(c[1])} | kind(c[0]) for c in cands[:3]], mut)


def o_intent(cands: list[Cand]) -> object: return _cands(cands, 0)
def m584(cands: list[Cand]) -> object: return _cands(cands, 584)


DRow = tuple[O, O, int, int, S, int, S, Optional[float]]  # ..., version, decision_id, as_of, broker_symbol, sl


def _served(rows: list[DRow], decision_id: str, mut: int):
    rows = rows[:3]
    if any(sl is not None and sl != sl for *_, sl in rows):  # NaN is unequal to itself on both sides
        return "nan"
    return lifted.decision_served([(*_id(a, s), _e(f, FLOW), v, _t(d), t, {"targets": [{"broker_symbol": _t(b), "sl": sl}]})
                                   for a, s, f, v, d, t, b, sl in rows], ID, _t(decision_id), mut)


def o_served(rows: list[DRow], decision_id: str) -> object: return _served(rows, decision_id, 0)
def m642(rows: list[DRow], decision_id: str) -> object: return _served(rows, decision_id, 642)
