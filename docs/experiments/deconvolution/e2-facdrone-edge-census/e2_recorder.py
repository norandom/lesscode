"""E2 recorder, a pytest plugin loaded with `-p e2_recorder` (this directory on PYTHONPATH).

Env: E2_SOURCE_ROOT (frozen copy), E2_MANIFEST (R15 source manifest), E2_OUT (output dir).
Writes E2_OUT/rec-<worker>-<pid>.jsonl and a coverage.py data file per process. Facdrone code
and conftests are untouched. Load together with `-p e2_winshim` on Windows (see README).
"""
import importlib, importlib.util, itertools, json, os, re, sys, threading, time
from pathlib import Path

import pytest

HERE = Path(__file__).resolve().parent
ROOT = Path(os.environ["E2_SOURCE_ROOT"]).resolve()
OUT = Path(os.environ["E2_OUT"]).resolve()
OUT.mkdir(parents=True, exist_ok=True)

def _load(name):
    spec = importlib.util.spec_from_file_location("e2_" + name, HERE / f"{name}.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod

def _resolve(dotted):
    parts = dotted.split(".")
    for i in range(len(parts) - 1, 0, -1):
        try:
            obj = importlib.import_module(".".join(parts[:i]))
        except ImportError:
            continue
        for p in parts[i:]:
            obj = obj if p == "*" else getattr(obj, p)
        return [getattr(obj, n) for n in dir(obj) if not n.startswith("_") and callable(getattr(obj, n))] if parts[-1] == "*" else [obj]
    raise ImportError(dotted)

SINKS, CODECS = _load("sinks"), _load("codecs")
_objs = {}  # callable or code object -> (sink name | None, codec name | None)
for kind, names in ((0, SINKS.SINKS), (1, CODECS.CODECS + CODECS.NONLITERAL_CTORS)):
    for name in names:
        for obj in _resolve(name if "." in name else "builtins." + name):
            key = getattr(obj, "__code__", obj)
            cur = list(_objs.get(key, (None, None)))
            cur[kind] = name
            _objs[key] = tuple(cur)
SUPPORT = lambda p: p.startswith("packages/facdrone-testing/") or "/testing/" in p or p.endswith("/testing.py")
FILES = {str(ROOT / f["path"]).replace("/", "\\").lower(): f["path"]  # test-support files count as test frames
         for f in json.load(open(os.environ["E2_MANIFEST"]))["files"] if not SUPPORT(f["path"])}
M = sys.monitoring
TOOL, E = 3, M.events
WORKER = os.environ.get("PYTEST_XDIST_WORKER", "main")
_out = open(OUT / f"rec-{WORKER}-{os.getpid()}.jsonl", "a", encoding="utf-8")
_ids, _cls, _stacks, _invs, _n = {}, {}, {}, {}, itertools.count()
_test = {"t": "<none>", "edges": set(), "inv": {}}
FX_CAP = 200

def _cid(code):
    r = _ids.get(code)
    if r is None:
        rel = FILES.get(code.co_filename.replace("/", "\\").lower())  # C only: Python here re-fires events
        r = _ids[code] = next(_n) if rel else False
        if rel:
            _out.write(json.dumps({"k": r, "id": f"{rel}::{code.co_qualname}::{code.co_firstlineno}"}) + "\n")
    return r

def _classify(c, arg0):
    f = getattr(c, "__func__", c)
    key = getattr(f, "__code__", f)
    try:
        r = _cls[key]
    except KeyError:
        r = _objs.get(key, (None, None))
        name = getattr(f, "__name__", "")
        if r == (None, None) and isinstance(name, str) and name.startswith(CODECS.PREFIX_METHODS) \
                and str(getattr(f, "__module__", "")).startswith("pydantic"):
            r = (None, "pydantic." + name)
        _cls[key] = r
    except TypeError:
        return None, None
    if r[1] in CODECS.NONLITERAL_CTORS and arg0 is M.MISSING:
        return r[0], None
    return r

def _start(code, offset, exc=None):
    i = _cid(code)
    if i is False:
        return M.DISABLE if exc is None else None
    tid = threading.get_ident()
    st = _stacks.setdefault(tid, [])
    if not st:
        _invs[tid] = {"e": i, "t": _test, "fx": {}, "c": 0, "by": {}}
    entry = _invs[tid]["e"]
    _test["edges"].add((entry, st[-1][1] if st else -1, i))
    st.append((sys._getframe(1), i))

def _stop(code, offset, value):
    i = _cid(code)
    if i is False:
        return M.DISABLE if not isinstance(value, BaseException) else None
    tid = threading.get_ident()
    st, frame = _stacks.get(tid, []), sys._getframe(1)
    for k in range(len(st) - 1, -1, -1):
        if st[k][0] is frame:
            del st[k:]
            break
    if not st and tid in _invs:
        inv = _invs.pop(tid)
        sig = (inv["e"], json.dumps(list(inv["fx"].values())))
        inv["t"]["inv"][sig] = inv["t"]["inv"].get(sig, 0) + 1

def _effect(sink, table=None):
    tid = threading.get_ident()
    st, inv = _stacks.get(tid), _invs.get(tid)
    if st and inv:  # distinct (sink, stack, table) in first-seen order; codec counts at last occurrence
        key = (sink, tuple(s[1] for s in st), table)
        if key in inv["fx"] or len(inv["fx"]) < FX_CAP:
            inv["fx"][key] = [sink, key[1], inv["c"], dict(inv["by"]), table]

def _call(code, offset, c, arg0):
    i = _cid(code)
    if i is False:
        return M.DISABLE
    sink, codec = _classify(c, arg0)
    if sink:
        _effect(sink)
    if codec:
        inv = _invs.get(threading.get_ident())
        if inv:
            inv["c"] += 1
            inv["by"][i] = inv["by"].get(i, 0) + 1

_WRITE = re.compile(r"^\s*(?:INSERT\s+(?:OR\s+\w+\s+)?INTO|UPDATE|DELETE\s+FROM)\s+\"?([\w.]+)", re.I)

def _sql(conn, cursor, statement, params, context, executemany):
    m = _WRITE.match(statement or "")
    _effect(SINKS.SQL_SINK, m.group(1).lower() if m else None)

try:
    from sqlalchemy import event
    from sqlalchemy.engine import Engine
    event.listen(Engine, "before_cursor_execute", _sql)
except ImportError:
    pass

import coverage  # noqa: E402

_cov = coverage.Coverage(data_file=str(OUT / ".coverage"), data_suffix=True,
                         include=[str(ROOT / "src" / "*"), str(ROOT / "packages" / "*")])
_cov.start()
M.use_tool_id(TOOL, "e2-recorder")
for ev, cb in ((E.PY_START, _start), (E.PY_RESUME, _start), (E.PY_THROW, _start), (E.PY_RETURN, _stop),
               (E.PY_YIELD, _stop), (E.PY_UNWIND, _stop), (E.CALL, _call)):
    M.register_callback(TOOL, ev, cb)
M.set_events(TOOL, E.PY_START | E.PY_RESUME | E.PY_THROW | E.PY_RETURN | E.PY_YIELD | E.PY_UNWIND | E.CALL)

def _flush(rec, extra):
    rows = [[e, n, json.loads(fx)] for (e, fx), n in rec["inv"].items()]
    _out.write(json.dumps({"t": rec["t"], "edges": sorted(rec["edges"]), "inv": rows, **extra}) + "\n")
    _out.flush()

@pytest.hookimpl(hookwrapper=True, tryfirst=True)
def pytest_runtest_protocol(item, nextitem):
    global _test
    outer, _test = _test, {"t": item.nodeid, "edges": set(), "inv": {}, "out": []}
    t0 = time.perf_counter()
    yield
    _flush(_test, {"wall": round(time.perf_counter() - t0, 4), "out": _test["out"]})
    _test = outer

def pytest_runtest_logreport(report):
    if report.nodeid == _test["t"] and (report.when == "call" or report.outcome != "passed"):
        _test["out"].append(f"{report.when}:{report.outcome}")

def pytest_unconfigure(config):
    M.set_events(TOOL, 0)
    M.free_tool_id(TOOL)
    _cov.stop()
    _cov.save()
    _flush(_test, {"wall": None})
    _out.close()
