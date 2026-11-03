"""E1c lifted predicates: the ten target sites of state_repository.py (frozen copy, Facdrone d49f48bd)
as pure functions of plain data. `mut` = the target line applies that cosmic-ray mutation; 0 = original.
A row is (account_id, strategy_id, flow, committed_version, decision_id, as_of, payload) in heap order;
an instruction is (kind, id, parent, target.venue_symbol). The SQL WHERE clauses are modelled in
Python (string equality; for the 150 mutant, code-point order instead of the database collation)."""
import json

SKIP = ("rejected", "skipped", "unavailable")


def _attempts(rows, ident, mut):
    """WHERE account_id == ? AND strategy_id == ? AND flow == 'attempt' (lines 148-151, 185-189).
    186 `is not`: a Column is never the str, so the account predicate becomes Python True."""
    return [p for a, s, f, *_, p in rows
            if (True if mut == 186 else a == ident[0]) and (s <= ident[1] if mut == 150 else s == ident[1])
            and f == "attempt"]


def opening_tickets(rows, ident, history, mut=0):  # lines 144-178
    tickets = {}
    for body in _attempts(rows, ident, mut):
        if body.get("child_kind") not in ("market_order", "stop_attach") or not body.get("ticket"):
            continue
        if body.get("outcome") in SKIP:
            continue

        def joined(i):  # i.id == instruction_id or (legacy and parent and venue), lines 167-172
            return i[1] == body.get("instruction_id") or (
                (body.get("representation") <= "legacy" if mut == 169 else body.get("representation") == "legacy")
                and i[2] == body.get("turn_id", body.get("parent_id"))
                and i[3] == body.get("venue_symbol"))
        candidates = [i for i in history if ((i[0] == "open" or joined(i)) if mut == 166 else (i[0] == "open" and joined(i)))]
        identities = {i[1] for i in candidates}
        if len(identities) == 1:
            tickets.setdefault(identities.pop(), set()).add(str(body["ticket"]))
    return {key: tuple(sorted(values)) for key, values in tickets.items()}


def resting_orders(rows, ident, history, mut=0):  # lines 180-202
    rests = [i for i in history if i[0] == "rest"]
    orders = {}
    for body in _attempts(rows, ident, mut):
        order = body.get("order_id") or body.get("ticket")
        if body.get("child_kind") != "limit_order" or not order:
            continue
        if body.get("outcome") in SKIP:
            continue
        matched = {r[1] for r in rests if placed_under(r, body, mut)}
        if (len(matched) >= 1) if mut == 200 else (len(matched) == 1):
            orders[matched.pop()] = str(order)
    return orders


def placed_under(rest, body, mut=0):  # lines 236-243
    if body.get("instruction_id"):
        return rest[1] == body["instruction_id"]
    return (
        (body.get("representation") is "legacy" if mut == 240 else body.get("representation") == "legacy")  # noqa: F632
        and (rest[2] <= body.get("turn_id", body.get("parent_id")) if mut == 241
             else rest[2] == body.get("turn_id", body.get("parent_id")))
        and rest[3] == body.get("venue_symbol"))


def _text(v):  # payload_json['key'].as_string(): JSONB ->> text
    return v if v is None or isinstance(v, str) else json.dumps(v)


def instruction_history(rows, ident, mut=0):  # lines 398-419: row_number() over (partition by key order by version desc)
    sel = [(_text(r[6].get("key")), r[3], n, r[6]) for n, r in enumerate(rows)
           if r[0] == ident[0] and r[1] == ident[1] and r[2] == "instruction"]
    sel.sort(key=lambda x: (x[0] is None, x[0] or "", -x[1], x[2]))  # the window's sort, NULL keys last
    out, rank, prev = [], 0, object()
    for key, _, _, payload in sel:
        rank = rank + 1 if key == prev else 1
        prev = key
        if (rank >= 1) if mut == 418 else (rank == 1):
            out.append(payload["instruction"])
    return out


def intention_ids(candidates, mut=0):  # line 581-585 of _shadow_seed
    return [body["id"] for body in candidates
            if isinstance(body, dict) and (body.get("kind") is "set_stops" if mut == 584 else body.get("kind") == "set_stops")]  # noqa: F632


def served(payload):  # _served, lines 613-616
    targets = payload.get("targets") or []
    return {str(t["broker_symbol"]): float(t["sl"]) for t in targets if t.get("sl") is not None}


def decision_served(rows, ident, decision_id, mut=0):  # lines 598-610 and 636-643
    sel = [r for r in rows if r[0] == ident[0] and r[1] == ident[1] and r[2] == "decision" and r[4] == decision_id]
    sel.sort(key=lambda r: (r[3], r[5]), reverse=True)  # ORDER BY committed_version DESC, as_of DESC
    top = sel[:2 if mut == 642 else 1]
    return None if not top else served(top[0][6])
