"""E3 Level 2 differential: `qdiff.py results/sql-divergences-roundN.json | --controls [--weak]` (README: method).
Each differing statement pair runs on the recorded pre/post tables and >= 100 seeded perturbations of them; outcome
= sorted canonical rows, or rowcount + all tables for writes, or the error type; pass if every state agrees.
--weak: flip = reverse only, no boundary row (the first, unstrengthened version, kept for the receipt)."""
import copy, difflib, json, random, re, sys
from datetime import datetime, timedelta, timezone
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path[:0] = [str(HERE.parent / d) for d in ("e1b-facdrone-evidence-publication", "e1c-concolic-rung", "e1c2-concolic-rerun")]
import e1c2_source  # noqa: E402
from psycopg.types.json import Jsonb  # noqa: E402
from sqlalchemy import JSON, Integer, String, null, select, text  # noqa: E402
O, SEED, N, WEAK = e1c2_source.O, 20261104, 100, "--weak" in sys.argv
GOLD = {g.name: g for g in (O.GOLDEN, e1c2_source.E.GOLDEN, e1c2_source.GOLDEN)}
T0, TS, KEY = datetime(2026, 1, 1, tzinfo=timezone.utc), re.compile(r"^<t(\d+)>$"), lambda r, c, n: f"{r[c]}~{n}" if isinstance(r[c], str) else r[c] + 1000 + n
BASE = lambda k: re.sub(r"_\d+$", "", k)  # SQLAlchemy parameter name -> column name
def de(v, col=None):  # undo the oracle's normalisation: time ordinals, byte digests, JSON text; dict params as JSONB
    m = TS.match(v) if isinstance(v, str) else None
    return T0 + timedelta(minutes=int(m[1])) if m else bytes.fromhex(v["bytes_sha256"]) if isinstance(v, dict) and set(v) == {"bytes_sha256"} \
        else (null() if v is None else json.loads(v)) if col is not None and isinstance(col.type, JSON) else Jsonb(v) if col is None and isinstance(v, dict) else v
def outcome(conn, stmt, params):
    sp = conn.begin_nested()
    try:
        r = conn.exec_driver_sql(stmt, [{k: de(v) for k, v in d.items()} for d in params] if isinstance(params, list) else {k: de(v) for k, v in (params or {}).items()})
        out = sorted(json.dumps(list(x), default=str) for x in r) if r.returns_rows else [r.rowcount, [
            sorted(json.dumps(dict(x), sort_keys=True, default=str) for x in conn.execute(select(t)).mappings()) for t in O.TABLES]]
    except Exception as e:  # noqa: BLE001 — an error is an outcome
        out = ["error", type(getattr(e, "orig", e)).__name__]
    sp.rollback()
    return out
def perturb(base: dict, rnd: random.Random, params: dict, names: str) -> tuple[dict, list]:
    s, log, strs = copy.deepcopy(base), [], sorted({v for v in params.values() if isinstance(v, str)})
    if (tabs := [t for t in O.TABLES if s.get(t.name) and t.name in names]) and not WEAK:  # strengthening (control ii): a row of a
        t = rnd.choice(tabs)  # table the statements name, new key, each column a parameter binds set to p or a neighbour
        r, n, pk = dict(rnd.choice(s[t.name])), len(s[t.name]), [c.name for c in t.primary_key]
        r.update({c: KEY(r, c, n) for c in pk})
        for k, p in params.items():
            if (c := BASE(k)) in r and c not in pk and type(p) in (str, int):
                r[c] = rnd.choice([p, p[:-1], p + "~"] if isinstance(p, str) else [p - 1, p, p + 1])
        s[t.name].append(r)
        log.append(["boundary", t.name, {k: r[k] for k in r if k in set(map(BASE, params))}])
    for _ in range(rnd.randint(0 if log else 1, 3 if log else 4)):
        if not (tabs := [t for t in O.TABLES if s.get(t.name)]):
            break
        t = rnd.choice(tabs)
        rows, i, kind = s[t.name], rnd.randrange(len(s[t.name])), rnd.choice(["drop", "dup", "flip", "bump"])
        r, pk, c = rows[i], [c.name for c in t.primary_key], None
        if kind == "drop":
            rows.pop(i)
        elif kind == "dup":
            rows.append(r := {**r, **{c: KEY(r, c, len(rows)) for c in pk}})
        elif cols := [x.name for x in t.c if isinstance(r.get(x.name), str if kind == "flip" else int) and not isinstance(r[x.name], bool)
                      and isinstance(x.type, String if kind == "flip" else Integer) and not TS.match(str(r[x.name]))]:
            c = rnd.choice(cols)
            r[c] = r[c] + 1 if kind == "bump" else r[c][::-1] if WEAK else rnd.choice([r[c][::-1], r[c][:-1], r[c] + "~", *strs])
        log.append([kind, t.name, i, c])
    return s, log
def check(pair: list, bases: list, rnd: random.Random, engine) -> dict:
    params = {k: v for st in pair if st and isinstance(st[1], dict) for k, v in st[1].items()}
    names, states, fails, first = " ".join(st[0] for st in pair if st), 0, 0, None
    with engine.connect() as conn:
        trans = conn.begin()
        for k in range(5 * N):
            if states >= N + len(bases):
                break
            st, log = (bases[k], ["base"]) if k < len(bases) else perturb(rnd.choice(bases), rnd, params, names)
            conn.execute(text("TRUNCATE " + ", ".join(t.name for t in O.TABLES)))
            try:
                with conn.begin_nested():
                    [conn.execute(t.insert(), [{c: de(v, t.c[c]) for c, v in r.items()} for r in st[t.name]]) for t in O.TABLES if st.get(t.name)]
            except Exception:  # noqa: BLE001 — a perturbed state the schema refuses is not a state
                fails += 1
                continue
            states += 1
            a, b = (outcome(conn, *side) if side else ["absent"] for side in pair)
            first = first or ({"state": states, "perturbation": log, "recorded": a, "replayed": b} if a != b else None)
        trans.rollback()
    return {"pass": first is None and states >= N, "states": states, "load_failures": fails, "first_difference": first}
def pairs_of(div: dict) -> dict:
    out = {}
    for e in div["divergences"]:
        seq = next(s for s in json.loads(GOLD[e["golden"]].read_text(encoding="utf-8"))["sequences"] if s["name"] == e["sequence"])
        i, obs, a, b = e["op_index"], seq["observations"], e["recorded"] or [], e["replayed"] or []
        sm = difflib.SequenceMatcher(None, [O.canon(x) for x in a], [O.canon(x) for x in b], autojunk=False)
        for tag, i1, i2, j1, j2 in (x for x in sm.get_opcodes() if x[0] != "equal"):
            for k in range(max(i2 - i1, j2 - j1)):
                p = [a[i1 + k] if i1 + k < i2 else None, b[j1 + k] if j1 + k < j2 else None]
                v = out.setdefault(O.canon(p), {"pair": p, "bases": [], "at": []})
                v["bases"] += [obs[i - 1]["tables"] if i else seq["start_tables"] or {}, obs[i]["tables"]]
                v["at"].append(f"{e['golden']} {e['sequence']} op {i}")
    return out
if __name__ == "__main__":
    if "--controls" in sys.argv:  # (i) identical statements must pass; (ii) R24 mutant 150 (`strategy_id <=`) must fail
        seq = json.loads(e1c2_source.GOLDEN.read_text(encoding="utf-8"))["sequences"][0]
        rec, bases = seq["observations"][0]["sql"][2], [seq["start_tables"], seq["observations"][0]["tables"]]
        mut = [rec[0].replace("live_decision_event.strategy_id = ", "live_decision_event.strategy_id <= "), rec[1]]
        todo, out = {"identical": {"pair": [rec, rec], "bases": bases}, "m150-shape": {"pair": [rec, mut], "bases": bases}}, f"controls{'-weak' * WEAK}"
    else:
        todo, out = pairs_of(json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))), Path(sys.argv[1]).stem
    res = {}
    with O.embedded_engine() as engine:
        O._attached(engine)  # migrates the fresh schema with Facdrone's Alembic chain
        for k, v in todo.items():
            res[k] = {**v, "bases": len(v["bases"]), **check(v["pair"], v["bases"], random.Random(SEED), engine)}
            print(k[:120], {x: res[k][x] for x in ("pass", "states", "load_failures")}, flush=True)
    (HERE / "results" / f"qdiff-{out}.json").write_text(json.dumps({"seed": SEED, "weak": WEAK, "pairs": len(res), "results": res,
                                                       "failed": sum(not r["pass"] for r in res.values())}, indent=1, default=str), encoding="utf-8")
    sys.exit(1 if any(not r["pass"] for r in res.values()) else 0)
