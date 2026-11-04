"""E1c-2 CrossHair drivers around E1c's lifted.py (imported unchanged); `o_*` original, `m<line>` mutant.
Bounds by construction, no string slicing, no rejection guard: a string is built from a symbolic int list,
one character per element from a three-letter alphabet, at most 8 elements consumed (zip with range); lists
consume at most 3 elements the same way; enumerated literals are int indexes taken modulo the enumeration.
Strings the production code reads from JSON and compares by identity (attempt `representation`, candidate
`kind`) pass through json.loads of a JSON string, so they are never the interned literal (method step 2).
Run: crosshair diffbehavior drivers2.o_open drivers2.m150 ; smoke: crosshair check --verbose drivers2.o_open"""
import inspect, json, os, sys
from pathlib import Path
from typing import Optional

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "e1c-concolic-rung"))
import lifted  # noqa: E402

ID = ("e1b-account", "e1b-strategy")  # the oracle's runtime identity
FLOW = (None, "attempt", "instruction", "decision", "other")
CHILD = (None, "market_order", "stop_attach", "limit_order", "other")
OUTCOME = (None, "rejected", "skipped", "unavailable", "accepted")
KIND = ("open", "rest", "set_stops", "other")
REP = (None, "legacy", "instructions")
Str = list[int]  # a symbolic string: chr(97 + c % 3) per element, <= 8 elements
O = Optional[Str]
KEYS = ("child_kind", "outcome", "ticket", "order_id", "instruction_id", "representation", "turn_id", "parent_id", "venue_symbol")
# attempt body: child_kind, outcome (indexes), ticket, order_id, instruction_id, representation (index, free), turn_id, parent_id, venue
Body = tuple[int, int, O, O, O, int, Str, O, O, O]
Row = tuple[O, O, int, Body]  # account_id, strategy_id (None = the identity), flow index, payload
Instr = tuple[int, Str, Str, Str]  # kind index, id, parent, target.venue_symbol


def _s(cs) -> str:
    return "".join(chr(97 + c % 3) for _, c in zip(range(8), cs))


def _o(cs):
    return None if cs is None else _s(cs)


def _e(i: int, opts: tuple):
    return opts[i % len(opts)]


def _n(xs, k: int = 3) -> list:
    return [x for _, x in zip(range(k), xs)]


def _wire(i: int, opts: tuple, cs):  # the literal opts[k], or the free string at k == len(opts), decoded from JSON
    k = i % (len(opts) + 1)
    v = _s(cs) if k == len(opts) else opts[k]
    return None if v is None else json.loads('"' + v + '"')


def _id(a, s) -> tuple:
    return (ID[0] if a is None else _s(a)), (ID[1] if s is None else _s(s))


def _body(b: Body) -> dict:
    vals = (_e(b[0], CHILD), _e(b[1], OUTCOME), *map(_o, b[2:5]), _wire(b[5], REP, b[6]), *map(_o, b[7:]))
    return {k: v for k, v in zip(KEYS, vals) if v is not None}


def _instr(h: Instr) -> tuple:
    return (_e(h[0], KIND), _s(h[1]), _s(h[2]), _s(h[3]))


def _attempts(rows, hist, fn, mut: int):
    return fn([(*_id(a, s), _e(f, FLOW), 0, "d", 0, _body(b)) for a, s, f, b in _n(rows)], ID,
              [_instr(h) for h in _n(hist)], mut)


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


def _hist(rows, mut: int):
    pay = lambda k, t: {"instruction": t} if k is None else {"key": k, "instruction": t}  # noqa: E731
    return lifted.instruction_history([(*_id(a, s), _e(f, FLOW), v, "d", 0, pay(_o(k), t)) for a, s, f, v, k, t in _n(rows)], ID, mut)


def o_hist(rows: list[IRow]) -> object: return _hist(rows, 0)
def m418(rows: list[IRow]) -> object: return _hist(rows, 418)


Cand = Optional[tuple[int, Str, Str]]  # None: a non-dict candidate; else (kind index over (None,) + KIND + free, free kind, id)


def _cands(cands, mut: int):
    out = []
    for c in _n(cands):
        k = None if c is None else _wire(c[0], (None, *KIND), c[1])
        out.append(None if c is None else {"id": _s(c[2])} | ({} if k is None else {"kind": k}))
    return lifted.intention_ids(out, mut)


def o_intent(cands: list[Cand]) -> object: return _cands(cands, 0)
def m584(cands: list[Cand]) -> object: return _cands(cands, 584)


DRow = tuple[O, O, int, int, Str, int, Str, Optional[int]]  # ..., version, decision_id, as_of, broker_symbol, sl


def _served(rows, did, mut: int):
    return lifted.decision_served([(*_id(a, s), _e(f, FLOW), v, _s(d), t, {"targets": [{"broker_symbol": _s(b), "sl": None if sl is None else float(sl)}]})
                                   for a, s, f, v, d, t, b, sl in _n(rows)], ID, _s(did), mut)


def o_served(rows: list[DRow], did: Str) -> object: return _served(rows, did, 0)
def m642(rows: list[DRow], did: Str) -> object: return _served(rows, did, 642)


def smoke(*args) -> object:  # S5 smoke: E1C2_SMOKE names the driver; its signature, a trivial postcondition
    """post: True"""
    return globals()[os.environ["E1C2_SMOKE"]](*args)


if os.environ.get("E1C2_SMOKE"):
    smoke.__signature__ = inspect.signature(globals()[os.environ["E1C2_SMOKE"]])
    smoke.__annotations__ = globals()[os.environ["E1C2_SMOKE"]].__annotations__
    for _name in ("opening_tickets", "resting_orders", "placed_under", "instruction_history", "intention_ids", "decision_served"):
        setattr(lifted, _name, (lambda f: lambda *a: (print("ENTER lifted." + f.__name__, file=sys.stderr), f(*a))[1])(getattr(lifted, _name)))
