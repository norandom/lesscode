"""Reviewer probes for E4 task 6.3. usage: probe.py ROOT OUT.json  (one tree per process; no database)."""
import ast, hashlib, inspect, json, os, sys, traceback
from contextlib import contextmanager
from dataclasses import asdict, replace
from datetime import date, datetime, timedelta, UTC
from pathlib import Path
from types import SimpleNamespace as NS
from unittest.mock import patch

ROOT = Path(sys.argv[1]).resolve(); OUT = sys.argv[2]
os.environ["FACDRONE_PROJECT_ROOT"] = str(ROOT); sys.path.insert(0, str(ROOT / "src"))
from sqlalchemy.dialects import postgresql
from sqlalchemy.exc import IntegrityError
from typer.testing import CliRunner
from facdrone.shells import lifecycle_evidence as EV, lifecycle_monitor_runner as RN, lifecycle_store as ST, lifecycle_projection as PR
from facdrone.shells.cli import lifecycle_group as GR
from facdrone.shells.cli.app import app
from facdrone.shells.wiring import lifecycle_root as RT
from facdrone.shells.cppi import lifecycle_seams as SE
from facdrone.pipelines.evaluation.monitor_windows import Window
from facdrone_testing.core.cppi import AS_OF
from facdrone_testing.core.lifecycle import BASELINE, BREACH, DECLARED, ANCHOR, ADMISSION, evidence as mkev
from facdrone_kernel.domain.signal_lifecycle import (LifecycleEvent, LifecycleState as LS, StandingRevision, UnrecordedStanding, OpenFlag,
    Enrollment, AdmissionEvidence, QualifiedBreach, RecordedHumanRetirement, TransitionRefused, TransitionReceipt)
from facdrone_kernel.lifecycle_monitor import assess_lifecycle
assert all(Path(m.__file__).resolve().is_relative_to(ROOT) for m in (EV, RN, ST, PR, GR, RT, SE))

RES = []
def norm(x):
    if isinstance(x, BaseException): return f"EXC {type(x).__name__}: {x}"
    if isinstance(x, (str, int, float, bool, type(None))): return x
    if isinstance(x, (datetime, date)): return x.isoformat()
    if isinstance(x, dict): return {str(k): norm(v) for k, v in x.items()}
    if isinstance(x, (list, tuple, set)): return [norm(v) for v in (sorted(x, key=repr) if isinstance(x, set) else x)]
    if hasattr(x, "__dataclass_fields__"): return {"__": type(x).__name__, **{k: norm(v) for k, v in asdict(x).items()}} if False else {"__": type(x).__name__, **{k: norm(getattr(x, k)) for k in x.__dataclass_fields__}}
    return repr(x)
def P(pid, hyp, desc):
    def deco(fn):
        try: out = fn()
        except BaseException as e:  # noqa: BLE001
            out = {"EXC": type(e).__name__, "msg": str(e)}
        RES.append({"id": pid, "hyp": hyp, "desc": desc, "out": norm(out)}); return fn
    return deco
def exc_info(e): return {"EXC": type(e).__name__, "msg": str(e), "cause": repr(e.__cause__), "context": type(e.__context__).__name__ if e.__context__ else None, "suppress_ctx": e.__suppress_context__}

# ------------------------------------------------------------------ H1/H11 CLI readings (fakes as in patchcheck)
DIGEST = "b" * 64
REVISION = hashlib.sha256(json.dumps({"method": "dated-ic-v2", **asdict(DECLARED)}, sort_keys=True).encode()).hexdigest()[:16]
FRESH = {r.monitor: r for r in assess_lifecycle(BREACH, BASELINE, DECLARED.rules)}
def row(monitor, **kw):
    f = FRESH.get(monitor)
    base = {"window_id": "2026-09-05", "rules_revision": REVISION, "baseline_digest": DIGEST, "monitor": monitor,
            "disposition": getattr(f, "disposition", "breach"), "value": getattr(f, "value", -0.1), "boundary": getattr(f, "boundary", 0.025)}
    base.update(kw); return base
def cli(rows, args=("carry@3", "--window", "2026-09-05"), ev=BREACH, cited="ok", rev=None):
    fakes = [(GR, "_lifecycle_store", lambda: NS(_engine=None, readings=lambda *_: rows)), (GR, "read_parameters", lambda _: {"lifecycle_monitor": asdict(DECLARED)}),
             (GR, "_cited_baseline", (lambda *_: {"digest": DIGEST, "traded_horizon_days": 1}) if cited == "ok" else (lambda *_: None)),
             (GR, "configured_pods", lambda **_: NS(get_scored=lambda _: NS(dynamics=None))), (GR, "_pod_of", lambda *_: "fixture"),
             (EV, "baseline_view", lambda _: BASELINE), (EV, "live_evidence", lambda *_: ev)]
    from contextlib import ExitStack
    with ExitStack() as s:
        for o, n, r in fakes: s.enter_context(patch.object(o, n, r))
        r = CliRunner().invoke(app, ["lifecycle", "readings", *args])
    return {"exit": r.exit_code, "out": r.output, "exc": None if r.exception is None or isinstance(r.exception, SystemExit) else f"{type(r.exception).__name__}: {r.exception}"}
MON = sorted(FRESH)
@P("P01", "H11", "readings: every stored monitor reproduced")
def _(): return cli([row(m) for m in MON])
@P("P02", "H11", "readings: one stored value differs from the re-measurement")
def _(): return cli([row(MON[0]), row(MON[1], value=123.0)])
@P("P03", "H1", "readings: second stored row lacks 'disposition' (malformed row after a good one)")
def _():
    bad = row(MON[1]); del bad["disposition"]; return cli([row(MON[0]), bad])
@P("P04", "H1", "readings: third stored row lacks 'value' after two good rows (partial output before the fault)")
def _():
    bad = row(MON[2]); del bad["value"]; return cli([row(MON[0]), row(MON[1]), bad])
@P("P05", "H11", "readings: only the not-run marker row stored, window now measurable")
def _(): return cli([{**row("window"), "monitor": "window", "disposition": "not_run"}])
@P("P06", "H11", "readings: evidence is a string (not run) and a stored not_run row")
def _(): return cli([{**row("window"), "monitor": "window", "disposition": "not_run"}], ev="no IC consumed")
@P("P07", "H11", "readings: evidence is a string and no stored not_run row")
def _(): return cli([row(MON[0])], ev="no IC consumed")
@P("P08", "H1", "readings: no stored rows for the window")
def _(): return cli([])
@P("P09", "H7", "readings: stored rows under another rules revision")
def _(): return cli([row(MON[0], rules_revision="deadbeef")])
@P("P10", "H1", "readings: target without @")
def _(): return cli([row(MON[0])], args=("carry", "--window", "2026-09-05"))
@P("P11", "H7", "readings: window id that is not a date, stored row carries the same id")
def _(): return cli([row(MON[0], window_id="bogus")], args=("carry@3", "--window", "bogus"))
@P("P12", "H7", "readings: window id 2026-02-30 (invalid date) with a matching row")
def _(): return cli([row(MON[0], window_id="2026-02-30")], args=("carry@3", "--window", "2026-02-30"))
@P("P13", "H1", "readings: baseline not on record (cited None)")
def _(): return cli([row(MON[0])], cited="none")
@P("P14", "H1", "readings: a stored monitor the fresh run no longer produces")
def _(): return cli([row("ghost_monitor", value=1.0, boundary=2.0)])
@P("P15", "H1", "readings: value None, boundary None stored (unmeasured monitor)")
def _(): return cli([row(MON[0], value=None, boundary=None, disposition="unmeasured")])
@P("P16", "H11", "readings: version with leading zeros / large (carry@0007)")
def _(): return cli([row(MON[0])], args=("carry@0007", "--window", "2026-09-05"))
# _compare adapter (private; interface differs by design)
@P("P17", "H11", "_compare directly: stored two rows, fresh two (output text + code; candidate adapted: lines printed then code)")
def _():
    import io, contextlib
    stored = [row(MON[0]), row(MON[1], value=5.0)]
    fresh = {r.monitor: r for r in FRESH.values()}
    r = CliRunner()
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        res = GR._compare(stored, fresh)
        if isinstance(res, tuple):
            for l in res[0]: GR.typer.echo(l)
            res = res[1]
    return {"code": res, "out": buf.getvalue()}
@P("P18", "H1", "_compare directly: stored row lacks 'value' after a good row (partial output)")
def _():
    import io, contextlib
    bad = row(MON[1]); del bad["value"]
    buf = io.StringIO(); err = None
    with contextlib.redirect_stdout(buf):
        try:
            res = GR._compare([row(MON[0]), bad], dict(FRESH))
            if isinstance(res, tuple):
                for l in res[0]: GR.typer.echo(l)
        except Exception as e: err = f"{type(e).__name__}: {e}"
    return {"err": err, "out": buf.getvalue()}

# ------------------------------------------------------------------ H2 bind_writers
class TStore:
    def __init__(self): self.calls = []
    def transition(self, request, connection=None): self.calls.append((asdict(request) if False else {k: norm(getattr(request, k)) for k in request.__dataclass_fields__}, connection)); return "RECEIPT"
CONN = object()
@P("P19", "H2", "bind_writers: admit with the five declared arguments, connection and decision id bound")
def _():
    s = TStore(); w = ST.bind_writers(s, "CONN", "D1"); r = w.admit("carry", 3, ADMISSION, 1, "why"); return [r, s.calls]
@P("P20", "H2", "bind_writers: admit called with a sixth positional argument (malformed call)")
def _():
    s = TStore(); w = ST.bind_writers(s); r = w.admit("carry", 3, ADMISSION, 1, "why", LS.RETIRED); return [r, s.calls]
@P("P21", "H2", "bind_writers: retire called with a sixth AND seventh positional argument")
def _():
    s = TStore(); w = ST.bind_writers(s); r = w.retire("carry", 3, "ev", 1, "why", LS.ADMITTED, "census"); return [r, s.calls]
@P("P22", "H2", "bind_writers: flag called with keyword arguments")
def _():
    s = TStore(); w = ST.bind_writers(s); r = w.flag(signal="a", version=1, evidence="e", expected=2, reason="r"); return [r, s.calls]
@P("P23", "H2", "bind_writers: unknown keyword argument to admit (_t=)")
def _():
    s = TStore(); w = ST.bind_writers(s); r = w.admit("a", 1, "e", 2, "r", _t=LS.RETIRED); return [r, s.calls]
@P("P24", "H2", "bind_writers: enroll takes (signal, version, act, reason); expected revision 0 bound")
def _():
    s = TStore(); w = ST.bind_writers(s); r = w.enroll("a", 1, Enrollment("me", "why"), "why"); return [r, s.calls]
@P("P25", "H2", "bind_writers: enroll with five arguments (malformed)")
def _():
    s = TStore(); w = ST.bind_writers(s); r = w.enroll("a", 1, Enrollment("me", "why"), 7, "why"); return [r, s.calls]
@P("P26", "H2", "bind_writers: signature and name of each writer as a caller can inspect them")
def _():
    w = ST.bind_writers(TStore()); return {n: [str(inspect.signature(getattr(w, n))), getattr(w, n).__name__] for n in ("enroll", "admit", "flag", "retire")}
@P("P27", "H2", "bind_writers: the four writers are distinct targets/authorities")
def _():
    s = TStore(); w = ST.bind_writers(s)
    w.enroll("a", 1, "x", "r"); w.admit("a", 1, "x", 1, "r"); w.flag("a", 1, "x", 1, "r"); w.retire("a", 1, "x", 1, "r")
    return [(c[0]["target"], c[0]["authority"], c[0]["expected_revision"]) for c in s.calls]

# ------------------------------------------------------------------ H3/H4 store logic over an in-memory event list
class Clock:
    def __init__(self): self.n = 0
    def __call__(self): self.n += 1; return AS_OF + timedelta(seconds=self.n)
class FakeEngine:
    def __init__(self, pairs): self.pairs = pairs; self.connects = 0
    def connect(self):
        eng = self
        class C:
            def __enter__(s): eng.connects += 1; return s
            def __exit__(s, *a): return False
            def execute(s, stmt): return NS(all=lambda: list(eng.pairs))
        return C()
class MemStore(ST.LifecycleStore):
    def __init__(self, events=(), plan=(), pairs=()):
        self.clock = Clock(); super().__init__(FakeEngine(list(pairs)), self.clock)
        self.ev = list(events); self.log = []; self.plan = list(plan)
    def events(self, signal, version):
        self.log.append(("events", signal, version)); return tuple(e for e in self.ev if (e.signal, e.version) == (signal, version))
    def _write(self, event, connection=None):
        self.log.append(("write", event.disposition, event.revision, event.reason, connection is not None))
        if self.plan:
            x = self.plan.pop(0)
            if x is not None: raise x
        _ = self._now()  # the real _write stamps known_at with the clock
        stamped = replace(event, event_id=ST._event_id(event)); self.ev.append(stamped); return stamped.event_id
def E(rev, new, prior=None, auth="registry", disp="applied", ref="r", sig="carry", ver=3, eid=None, at=0, reason="x", closed=None):
    return LifecycleEvent(sig, ver, prior, new, rev, AS_OF + timedelta(seconds=at), auth, reason, ref, disp, closed, None, eid or f"id{rev}{disp[0]}{new.value[0]}")
def req(target, auth, ev, exp, reason="why"): return ST.TransitionRequest("carry", 3, target, auth, ev, exp, reason, "dec")
def tr(store, request, connection=None):
    try: r = store.transition(request, connection); out = {"receipt": r}
    except BaseException as e: out = exc_info(e) if isinstance(e, TransitionRefused) else {"EXC": type(e).__name__, "msg": str(e), "ctx": type(e.__context__).__name__ if e.__context__ else None}
    out["log"] = store.log; out["clock_calls"] = store.clock.n; out["events"] = [(e.revision, e.disposition, e.new_state.value, e.reason, e.closed_flag_id, e.event_id[:12]) for e in store.ev]
    return out
FLAGEV = QualifiedBreach("traded_horizon_ic", ("r1", "r2", "r3"), ("w1", "w2", "w3"), "bad")
ENR = Enrollment("me", "why")
@P("P28", "H3", "transition: enrol on an empty store")
def _(): return tr(MemStore(), req(LS.CANDIDATE, "registry", ENR, 0))
@P("P29", "H3", "transition: stale expected revision (standing at 1, expected 0)")
def _(): return tr(MemStore([E(1, LS.CANDIDATE)]), req(LS.CANDIDATE, "registry", ENR, 0))
@P("P30", "H3", "transition: wrong authority for the target (admit by registry)")
def _(): return tr(MemStore([E(1, LS.CANDIDATE)]), req(LS.ADMITTED, "registry", ADMISSION, 1))
@P("P31", "H3", "transition: wrong evidence type (retire with admission evidence)")
def _(): return tr(MemStore([E(1, LS.CANDIDATE)]), req(LS.RETIRED, "human", ADMISSION, 1))
@P("P32", "H3", "transition: admission with a failing census")
def _(): return tr(MemStore([E(1, LS.CANDIDATE)]), req(LS.ADMITTED, "census", replace(ADMISSION, checks_all=False), 1))
@P("P33", "H3", "transition: write clash (IntegrityError) outside a caller transaction")
def _():
    s = MemStore([E(1, LS.CANDIDATE)], plan=[IntegrityError("stmt", {}, Exception("dup"))]); return tr(s, req(LS.ADMITTED, "census", ADMISSION, 1))
@P("P34", "H3", "transition: write clash inside a caller's transaction (connection given)")
def _():
    s = MemStore([E(1, LS.CANDIDATE)], plan=[IntegrityError("stmt", {}, Exception("dup"))]); return tr(s, req(LS.ADMITTED, "census", ADMISSION, 1), connection=object())
@P("P35", "H3", "transition: write fails with a non-integrity error")
def _():
    s = MemStore([E(1, LS.CANDIDATE)], plan=[RuntimeError("db down")]); return tr(s, req(LS.ADMITTED, "census", ADMISSION, 1))
@P("P36", "H3", "transition: stale revision AND the refused row write clashes (IntegrityError swallowed)")
def _():
    s = MemStore([E(1, LS.CANDIDATE)], plan=[IntegrityError("stmt", {}, Exception("dup"))]); return tr(s, req(LS.CANDIDATE, "registry", ENR, 0))
@P("P37", "H3", "transition: stale revision AND the refused row write fails with a non-integrity error")
def _():
    s = MemStore([E(1, LS.CANDIDATE)], plan=[RuntimeError("down")]); return tr(s, req(LS.CANDIDATE, "registry", ENR, 0))
@P("P38", "H3", "transition: write clash then the refused write also raises RuntimeError")
def _():
    s = MemStore([E(1, LS.CANDIDATE)], plan=[IntegrityError("stmt", {}, Exception("dup")), RuntimeError("down2")]); return tr(s, req(LS.ADMITTED, "census", ADMISSION, 1))
@P("P39", "H3", "transition: admit after a flag closes the flag (closed_flag_id = the flag event id)")
def _():
    evs = [E(1, LS.CANDIDATE), E(2, LS.ADMITTED, LS.CANDIDATE, "census", ref="c1"), E(3, LS.FLAGGED, LS.ADMITTED, "monitor", eid="FLAGEVT")]
    return tr(MemStore(evs), req(LS.ADMITTED, "census", ADMISSION, 3))
@P("P40", "H3", "transition: flagged standing, two applied events at the flag's revision and a refused one (which id closes)")
def _():
    evs = [E(1, LS.CANDIDATE), E(2, LS.ADMITTED, LS.CANDIDATE, "census"), E(3, LS.FLAGGED, LS.ADMITTED, "monitor", eid="F1", at=1), E(3, LS.FLAGGED, LS.ADMITTED, "monitor", eid="F2", at=2),
           E(3, LS.RETIRED, LS.FLAGGED, "human", disp="refused", eid="R1", at=3)]
    return tr(MemStore(evs), req(LS.ADMITTED, "census", ADMISSION, 3))
@P("P41", "H3", "transition: retire from flagged by human (flag closed)")
def _():
    evs = [E(1, LS.CANDIDATE), E(2, LS.ADMITTED, LS.CANDIDATE, "census"), E(3, LS.FLAGGED, LS.ADMITTED, "monitor", eid="FL")]
    return tr(MemStore(evs), req(LS.RETIRED, "human", RecordedHumanRetirement("a", "r", "rec"), 3))
@P("P42", "H3", "transition: out of retired (nothing leaves retired)")
def _():
    evs = [E(1, LS.CANDIDATE), E(2, LS.RETIRED, LS.CANDIDATE, "human")]
    return tr(MemStore(evs), req(LS.ADMITTED, "census", ADMISSION, 2))
@P("P43", "H3", "transition: flag by monitor from admitted (revision bump 2 -> 3)")
def _():
    evs = [E(1, LS.CANDIDATE), E(2, LS.ADMITTED, LS.CANDIDATE, "census")]
    return tr(MemStore(evs), req(LS.FLAGGED, "monitor", FLAGEV, 2))
@P("P44", "H3", "transition: duplicate request (same moment) after an applied enrol -> refused no-transition")
def _():
    s = MemStore(); a = tr(s, req(LS.CANDIDATE, "registry", ENR, 0)); b = tr(s, req(LS.CANDIDATE, "registry", ENR, 1)); return [a, b]
@P("P45", "H4", "standing/census_references/open_flags/admitted: empty store")
def _():
    s = MemStore(); return {"standing": s.standing("carry", 3), "census": s.census_references("carry", 3), "admitted": s.admitted(), "flags": s.open_flags(), "log": s.log, "connects": s._engine.connects}
EV_MIX = [E(1, LS.CANDIDATE), E(2, LS.ADMITTED, LS.CANDIDATE, "census", ref="c1", at=1), E(3, LS.FLAGGED, LS.ADMITTED, "monitor", ref="m", eid="F3", at=2),
          E(3, LS.RETIRED, LS.FLAGGED, "human", disp="refused", ref="x", at=3), E(4, LS.ADMITTED, LS.FLAGGED, "census", ref="c2", at=4), E(4, LS.ADMITTED, LS.FLAGGED, "census", ref="c2", at=5, disp="refused"),
          E(5, LS.FLAGGED, LS.ADMITTED, "monitor", eid="F5", at=6, reason="late")]
@P("P46", "H4", "standing + census_references over a mixed history (refused rows, repeated census ref, re-flag)")
def _():
    s = MemStore(EV_MIX); return {"standing": s.standing("carry", 3), "census": s.census_references("carry", 3), "log": s.log}
@P("P47", "H4", "standing: two applied events tie at the highest revision (first wins)")
def _():
    s = MemStore([E(1, LS.CANDIDATE), E(2, LS.ADMITTED, LS.CANDIDATE, "census", ref="a", at=1), E(2, LS.FLAGGED, LS.ADMITTED, "monitor", ref="b", at=2)]); return s.standing("carry", 3)
@P("P48", "H4", "standing: only refused events recorded -> unrecorded")
def _():
    s = MemStore([E(1, LS.ADMITTED, None, "census", disp="refused")]); return s.standing("carry", 3)
@P("P49", "H4", "open_flags + admitted over three pairs (flagged, admitted, retired, unrecorded), engine pairs fetched once")
def _():
    evs = [E(1, LS.CANDIDATE, sig="a"), E(2, LS.ADMITTED, LS.CANDIDATE, "census", sig="a"), E(3, LS.FLAGGED, LS.ADMITTED, "monitor", sig="a", eid="FA"),
           E(1, LS.CANDIDATE, sig="b"), E(2, LS.ADMITTED, LS.CANDIDATE, "census", sig="b"),
           E(1, LS.CANDIDATE, sig="c"), E(2, LS.RETIRED, LS.CANDIDATE, "human", sig="c"), E(1, LS.ADMITTED, None, "census", sig="d", disp="refused")]
    s = MemStore(evs, pairs=[("a", 3), ("b", 3), ("c", 3), ("d", 3)])
    return {"admitted": s.admitted(), "flags": s.open_flags(), "log": s.log, "connects": s._engine.connects}
@P("P50", "H7", "overdue_flags: exactly at the boundary and one second beyond (clock call count)")
def _():
    evs = [E(1, LS.CANDIDATE, sig="a"), E(2, LS.ADMITTED, LS.CANDIDATE, "census", sig="a"), E(3, LS.FLAGGED, LS.ADMITTED, "monitor", sig="a", eid="FA", at=0)]
    s = MemStore(evs, pairs=[("a", 3)]); s.clock.n = 0
    s._now = lambda: AS_OF + timedelta(days=14); at = s.overdue_flags(timedelta(days=14))
    s._now = lambda: AS_OF + timedelta(days=14, seconds=1); over = s.overdue_flags(timedelta(days=14))
    return {"at_boundary": at, "beyond": over}
@P("P51", "H4", "open_flags: events whose new_state is the plain string 'flagged'")
def _():
    e = E(3, LS.FLAGGED, LS.ADMITTED, "monitor", sig="a", eid="FA"); e2 = replace(e, new_state="flagged")
    s = MemStore([E(1, LS.CANDIDATE, sig="a"), e2], pairs=[("a", 3)]); return s.open_flags()
@P("P52", "H3", "event identity: the id is the content digest, identical for identical events (pure _event_id)")
def _(): return [ST._event_id(E(2, LS.ADMITTED, LS.CANDIDATE, "census")), ST._event_id(E(2, LS.ADMITTED, LS.CANDIDATE, "census", at=1))]

# ------------------------------------------------------------------ H5/H6 runner over a fake engine
class Tx:
    def __init__(self, eng, mode): self.eng = eng; self.mode = mode
    def __enter__(self): self.eng.log.append((self.mode, "open")); return Conn(self.eng)
    def __exit__(self, et, ev, tb):
        self.eng.log.append((self.mode, "close", "rollback" if et and self.mode == "begin" else "ok")); return False
class Conn:
    def __init__(self, eng): self.eng = eng
    def execute(self, stmt, *a, **k):
        if type(stmt).__name__ == "Select":
            self.eng.log.append(("select",)); return NS(all=lambda: list(self.eng.recorded))
        c = stmt.compile(dialect=postgresql.dialect()); p = c.params
        ids = sorted(v for kk, v in p.items() if kk.startswith("reading_id"))
        rows = {}
        for kk, v in p.items():
            base, _, idx = kk.rpartition("_m");
            if base and idx.isdigit(): rows.setdefault(int(idx), {})[base] = v
        self.eng.log.append(("insert", str(c)[:40], [dict(sorted(rows[i].items())) for i in sorted(rows)])); return NS()
class Eng:
    def __init__(self, recorded=(), fail_connect_on=None): self.recorded = list(recorded); self.log = []; self.fail = fail_connect_on
    def connect(self):
        if self.fail and self.fail(): raise RuntimeError("db down on connect")
        return Tx(self, "connect")
    def begin(self): return Tx(self, "begin")
TARGET = RN.MonitoredSignal("carry", 3, BASELINE, "b" * 64, 2, ANCHOR)
TARGET2 = RN.MonitoredSignal("zzz", 1, BASELINE, "c" * 64, 4, ANCHOR)
class Flag:
    def __init__(self, refuse=None, boom=None): self.calls = []; self.refuse = refuse; self.boom = boom
    def __call__(self, conn, decision):
        def w(signal, version, evidence, expected, reason):
            self.calls.append((signal, version, norm(evidence), expected, reason, decision))
            if self.boom: raise self.boom
            if self.refuse: raise TransitionRefused(self.refuse)
            return TransitionReceipt("FLAG-EVT", signal, version, LS.ADMITTED, LS.FLAGGED, expected + 1, None)
        return w
def runner(targets, ev, eng, flag=None, declared=DECLARED, rules=None):
    flag = flag or Flag(); r = RN.MonitorRunner(eng, declared, RN.MonitorSources(lambda: tuple(targets), lambda ws: (setattr(r, "_seen", [w.window_id for w in ws]) or ev), flag))
    if rules: r._rules = rules
    return r, flag
def go(targets, ev, recorded=(), emit_fail_at=None, flag=None, as_of=AS_OF, rules=None, fail_connect_on=None, with_emit=True):
    eng = Eng(recorded, fail_connect_on); r, fl = runner(targets, ev, eng, flag, rules=rules); em = []
    def emit(event, message, **f):
        if emit_fail_at is not None and len(em) == emit_fail_at: raise OSError("emitter down")
        em.append((event, message, list(f.items())))
    try: s = r.run(as_of, "dec1", emit if with_emit else None); out = {"summary": s}
    except BaseException as e: out = {"EXC": type(e).__name__, "msg": str(e)}
    out.update(emitted=em, log=eng.log, flag_calls=fl.calls, seen=getattr(r, "_seen", None)); return out
def ev_const(e): return lambda target, window: e
@P("P53", "H5", "run: no monitored signals")
def _(): return go([], ev_const(BREACH))
@P("P54", "H5", "run: as_of the day of admission (no window due): empty final transaction")
def _(): return go([TARGET], ev_const(BREACH), as_of=ANCHOR)
@P("P55", "H5", "run: no window due but three breach windows already recorded (no fresh window, no flag)")
def _(): return go([TARGET], ev_const(BREACH), recorded=[(w, "traded_horizon_ic", "breach") for w in ("2026-09-04", "2026-09-05", "2026-09-06")], as_of=ANCHOR + timedelta(days=4))
@P("P56", "H6", "run: three due windows of breach evidence -> one flag with the completing window, rows per window")
def _(): return go([TARGET], ev_const(BREACH))
@P("P57", "H5", "run: two windows already recorded as breach, one new window completes the run -> flag")
def _(): return go([TARGET], ev_const(BREACH), recorded=[(w, "traded_horizon_ic", "breach") for w in ("2026-09-04", "2026-09-05")])
@P("P58", "H6", "run: flag writer refuses (standing moved) -> refused outcome emitted")
def _(): return go([TARGET], ev_const(BREACH), flag=Flag(refuse="carry v3 -> flagged: stale"))
@P("P59", "H6", "run: flag writer raises RuntimeError -> signal unavailable, transaction rolled back")
def _(): return go([TARGET], ev_const(BREACH), flag=Flag(boom=RuntimeError("pg gone")))
@P("P60", "H5", "run: every window without evidence (strings) -> not-run rows, emissions")
def _(): return go([TARGET], ev_const("no IC consumed at the traded horizon (1d)"))
@P("P61", "H5", "run: mixed evidence, first window string then breach")
def _():
    seq = iter(["gap", BREACH, BREACH]); return go([TARGET], lambda t, w: next(seq))
@P("P62", "H1", "run: the evidence callable returns None for a window (malformed evidence)")
def _(): return go([TARGET], ev_const(None))
@P("P63", "H1", "run: the evidence callable returns an int for a window")
def _(): return go([TARGET], ev_const(7))
@P("P64", "H1", "run: the evidence callable raises ValueError for the second window")
def _():
    seq = iter([0, 1, 2])
    def ev(t, w):
        if next(seq) == 1: raise ValueError("bad window")
        return BREACH
    return go([TARGET], ev)
@P("P65", "H6", "run: _plan fails for the first target (connect raises); the second still runs; emission order")
def _():
    n = {"i": 0}
    def fc():
        n["i"] += 1; return n["i"] == 1
    return go([TARGET, TARGET2], ev_const(BREACH), fail_connect_on=fc)
@P("P66", "H6", "run: the emitter raises on the 2nd record")
def _(): return go([TARGET], ev_const(BREACH), emit_fail_at=1)
@P("P67", "H6", "run: no emitter given (default sink)")
def _(): return go([TARGET], ev_const(BREACH), with_emit=False)
@P("P68", "H6", "run: two targets, second fails in its cycle (evidence raises for it only)")
def _():
    def ev(t, w):
        if t.signal == "zzz": raise KeyError("missing")
        return BREACH
    return go([TARGET, TARGET2], ev)
@P("P69", "H6", "run: two targets both flag; summary flags order and counts")
def _(): return go([TARGET, TARGET2], ev_const(BREACH))
@P("P70", "H6", "run: unmeasured readings counted (evidence with no IC values)")
def _(): return go([TARGET], ev_const(mkev(())))
@P("P71", "H6", "run: a runner whose _rules was replaced after construction (older revision) records under it")
def _(): return go([TARGET], ev_const(BREACH), rules="0123456789abcdef")
@P("P72", "H7", "run: as_of naive datetime")
def _(): return go([TARGET], ev_const(BREACH), as_of=datetime(2026, 9, 7, 12))
@P("P73", "H7", "run: as_of at 00:00 on the day after the third cutoff (boundary of 'due the day after')")
def _(): return go([TARGET], ev_const(BREACH), as_of=datetime(2026, 9, 7, 0, 0, tzinfo=UTC))
@P("P74", "H7", "run: as_of one second before the day boundary of the last window")
def _(): return go([TARGET], ev_const(BREACH), as_of=datetime(2026, 9, 6, 23, 59, 59, tzinfo=UTC))
@P("P75", "H5", "run: persistence count 1 (declared) flags on the first breach window")
def _():
    d = replace(DECLARED, runner=replace(DECLARED.runner, persistence_count=1)); eng = Eng(); fl = Flag()
    r = RN.MonitorRunner(eng, d, RN.MonitorSources(lambda: (TARGET,), lambda ws: ev_const(BREACH), fl)); em = []
    s = r.run(AS_OF, "dec1", lambda e, m, **f: em.append((e, m, list(f.items()))))
    return {"summary": s, "emitted": em, "flags": fl.calls, "log": eng.log}
@P("P76", "H5", "run: signal watch (breach on two of three windows, then within) -> watch counted, no flag")
def _():
    seq = iter([BREACH, BREACH, mkev((0.2, 0.2, 0.2))]); return go([TARGET], lambda t, w: next(seq))
@P("P77", "H6", "run: _unavailable payload field order and summary tuple for a failing plan")
def _():
    out = go([TARGET], ev_const(BREACH), fail_connect_on=lambda: True); return out

# ------------------------------------------------------------------ H8 projection
def S(standing, baseline, readings, flag, moment=(AS_OF, timedelta(days=14))): return PR.signal_standing(standing, baseline, readings, flag, moment)
ST_OK = StandingRevision("carry", 3, LS.ADMITTED, 2, AS_OF - timedelta(days=3), "census:x")
BL = {"route": "scored", "traded_horizon_days": 1, "traded_ic": {"mean": 0.05}, "survival": {"ratio": 0.8}, "states": [{"state": "calm"}, {"state": "stress"}], "digest": "a" * 40}
def rd(window, monitor, disp="within", digest="d1", rules="r1", **kw):
    return {"window_id": window, "monitor": monitor, "disposition": disp, "baseline_digest": digest, "rules_revision": rules, "value": 0.1, "boundary": 0.2, "comparator": ">=", "units": "u", "known_at": AS_OF, "payload": {"limitation": None}, **kw}
FLG = OpenFlag("carry", 3, "FEV", "traded_horizon_ic breached on 3 consecutive windows", AS_OF - timedelta(days=15), "dec")
@P("P78", "H8", "projection: nothing recorded (no baseline, no reading, no flag, unrecorded standing)")
def _(): return S(UnrecordedStanding("carry", 3), None, [], None)
@P("P79", "H8", "projection: two bases, newest basis wins; older-basis rows hidden, including their not_run rows")
def _(): return S(ST_OK, BL, [rd("w1", "traded_horizon_ic", digest="old"), rd("w1", "window", "not_run", digest="old", payload={"limitation": "gone"}), rd("w2", "survival", digest="new"), rd("w3", "ic_gap", "breach", digest="new")], None)
@P("P80", "H8", "projection: latest per monitor is the last row in window order; not_run windows listed")
def _(): return S(ST_OK, BL, [rd("w1", "survival"), rd("w2", "survival", "breach", value=0.9), rd("w2", "window", "not_run", payload={"limitation": "no IC"}), rd("w3", "window", "not_run", payload=None)], FLG)
@P("P81", "H8", "projection: reading row without the 'monitor' key")
def _(): r = rd("w1", "survival"); del r["monitor"]; return S(ST_OK, BL, [r], None)
@P("P82", "H8", "projection: reading row without 'disposition'")
def _(): r = rd("w1", "survival"); del r["disposition"]; return S(ST_OK, BL, [r], None)
@P("P83", "H8", "projection: a not_run row without 'window_id'")
def _(): r = rd("w1", "window", "not_run"); del r["window_id"]; return S(ST_OK, BL, [r], None)
@P("P84", "H8", "projection: baseline row that is empty / lacks route and digest")
def _(): return S(ST_OK, {}, [], None)
@P("P85", "H8", "projection: baseline with traded_ic not a mapping, survival None, states None")
def _(): return S(ST_OK, {"route": "r", "traded_ic": 0.1, "survival": None, "states": None, "digest": 123456789012345678901234}, [], None)
@P("P86", "H8", "projection: baseline states contain a non-mapping")
def _(): return S(ST_OK, {**BL, "states": ["calm"]}, [], None)
@P("P87", "H8", "projection: an unknown disposition label and an unknown monitor key")
def _(): return S(ST_OK, BL, [rd("w1", "state_ic:calm", "weird"), rd("w1", "cost:financing", "descriptive"), rd("w1", "my_new_one", "within")], None)
@P("P88", "H7", "projection: flag exactly at the overdue age, and one second past")
def _():
    f = replace(FLG, opened_at=AS_OF - timedelta(days=14)); g = replace(FLG, opened_at=AS_OF - timedelta(days=14, seconds=1))
    return [S(ST_OK, BL, [], f)["flag"], S(ST_OK, BL, [], g)["flag"]]
@P("P89", "H7", "projection: now naive while flag aware")
def _(): return S(ST_OK, BL, [], FLG, (datetime(2026, 9, 7), timedelta(days=14)))
@P("P90", "H8", "projection: standing that is neither a revision nor unrecorded (None)")
def _(): return S(None, BL, [], None)
@P("P91", "H8", "projection: standing is a revision of a state outside the label table (retired)")
def _(): return S(StandingRevision("carry", 3, LS.RETIRED, 4, AS_OF, None), None, [rd("w1", "survival")], None)
@P("P92", "H8", "projection: readings passed as a tuple / readings as generator")
def _(): return [S(ST_OK, BL, tuple([rd("w1", "survival")]), None)["readings"], S(ST_OK, BL, (r for r in [rd("w1", "survival")]), None)]
@P("P93", "H8", "projection: error ordering, malformed reading AND malformed baseline together")
def _():
    r = rd("w1", "survival"); del r["window_id"]; return S(ST_OK, {"route": 1, "states": [3]}, [r], None)
@P("P94", "H8", "projection: error ordering, malformed baseline AND malformed flag together")
def _():
    bad = NS(reason=3, opened_at=None, flag_event_id="x", decision_id=None, overdue=lambda *a: True)
    return S(ST_OK, {"route": 1, "states": [3]}, [], bad)
@P("P95", "H8", "projection: monitor_label/labeled_reason pure helpers")
def _(): return [PR.monitor_label("state_ic:calm"), PR.monitor_label("cost:x"), PR.monitor_label("a_b"), PR.labeled_reason("survival breached on 3"), PR.labeled_reason("plain")]

# ------------------------------------------------------------------ H13 evidence (lookback inlined) with SimpleNamespace consumed ICs
def ic(day, val, known, horizon=1, sig=("carry", 3)): return NS(signal=NS(name=sig[0], version=sig[1]), horizon_days=horizon, knowledge_at=known, window=(NS(observed_at=day, available_at=known, value=val),))
WIN = Window("2026-09-05", date(2026, 9, 3), date(2026, 9, 5))
def le(consumed, readings=lambda d: {}, ic_days=10): return EV.LiveEvidence(readings, consumed, ic_days).for_window(("carry", 3, 1), WIN)
K = lambda d, h=12: datetime(2026, 9, d, h, tzinfo=UTC)
@P("P96", "H13", "for_window: no consumed IC -> reason string")
def _(): return le([])
@P("P97", "H13", "for_window: two ICs, latest lookback chosen")
def _(): return le([ic(date(2026, 9, 3), 0.1, K(3)), ic(date(2026, 9, 4), 0.2, K(4))])
@P("P98", "H13", "for_window: two consumptions at the same latest knowledge instant with different windows (conflict)")
def _(): return le([ic(date(2026, 9, 3), 0.1, K(4)), ic(date(2026, 9, 4), 0.3, K(4))])
@P("P99", "H13", "for_window: observed date after consumption")
def _(): return le([ic(date(2026, 9, 5), 0.1, K(4))])
@P("P100", "H13", "for_window: naive knowledge_at skipped; aware kept")
def _(): return le([ic(date(2026, 9, 4), 0.1, datetime(2026, 9, 4, 12)), ic(date(2026, 9, 4), 0.2, K(4))])
@P("P101", "H13", "for_window: knowledge instant before window.start counts for the series but not the lookback")
def _(): return le([ic(date(2026, 9, 1), 0.4, K(2))], ic_days=10)
@P("P102", "H13", "for_window: AuditedAbsence-like window (non-tuple) as latest lookback")
def _():
    c = ic(date(2026, 9, 4), 0.1, K(4)); c.window = "absent"; return le([c])
@P("P103", "H13", "for_window: ambiguous observed datetime (non-midnight)")
def _(): return le([ic(datetime(2026, 9, 4, 5, tzinfo=UTC), 0.1, K(4))])
@P("P104", "H13", "for_window: states joined from readings (dwell, state_ic)")
def _(): return le([ic(date(2026, 9, 4), 0.2, K(4)), ic(date(2026, 9, 3), 0.1, K(3))], readings=lambda d: {date(2026, 9, 3): NS(entry=NS(state=NS(value="calm"))), date(2026, 9, 4): NS(entry=NS(state=NS(value="stress")))})
@P("P105", "H13", "read_evidence: consumed read once for the union of due windows; no windows -> none")
def _():
    calls = []
    with patch.object(EV, "_consumed", lambda e, s, t: calls.append((s, t)) or []):
        EV.read_evidence(None, lambda d: {}, (), 5); EV.read_evidence(None, lambda d: {}, (WIN, Window("2026-09-06", date(2026, 9, 4), date(2026, 9, 6))), 5)
    return calls

# ------------------------------------------------------------------ H9/H10 seams: lazy import timing, census references
LEDGER_ROWS = [{"baseline": {"digest": "dig1", "route": "scored", "traded_horizon_days": 1}, "id": "A"}, {"baseline": {"digest": "dig2"}, "id": "B"}, {"id": "C"}]
def seam(admitted, bind_patches, call_patches, ledger=True, admission=None):
    from facdrone.pipelines.evaluation import census_protocol as CP, recensus as RC
    import tempfile
    root = Path(tempfile.mkdtemp()); (root / "research").mkdir()
    if ledger: (root / "research" / "equity_census_verdicts.jsonl").write_text("{}\n")
    ref = {"A": "refA", "B": "refB"}
    adm = admission or (lambda r: NS(reference=lambda: ref[r["id"]]) if r["id"] in ref else None)
    store = NS(admitted=lambda: tuple(admitted), engine=None)
    book = NS(engine_db=None, deps=NS(policy=NS(pod=NS(dynamics=None))))
    from contextlib import ExitStack
    def ctx(ps):
        s = ExitStack()
        for mod, name, val in ps: s.enter_context(patch.object(mod, name, val))
        return s
    base = [(CP, "parse_ledger", lambda text: LEDGER_ROWS), (RC, "admission_evidence", adm), (EV, "baseline_view", lambda b: BASELINE)]
    with ctx(base):
        with ctx(bind_patches): r = SE.bind_lifecycle_monitor(root, book, store, DECLARED)
        with ctx(call_patches): return [norm(m) for m in r._monitored()]
SA = lambda sig, census: StandingRevision(sig, 1, LS.ADMITTED, 2, AS_OF, census)
@P("P106", "H10", "monitored: admitted without census, with unknown census, with cited census A and B, duplicates")
def _(): return seam([SA("s1", None), SA("s2", "nope"), SA("s3", "refA"), SA("s4", "refB"), SA("s3", "refA")], [], [])
@P("P107", "H10", "monitored: ledger file absent -> nothing monitorable")
def _(): return seam([SA("s3", "refA")], [], [], ledger=False)
@P("P108", "H10", "monitored: cited baseline row lacks 'digest' (refB's baseline has one; make refA's lack it)")
def _():
    global LEDGER_ROWS
    saved = LEDGER_ROWS; LEDGER_ROWS = [{"baseline": {"route": "scored"}, "id": "A"}]
    try: return seam([SA("s3", "refA")], [], [])
    finally: LEDGER_ROWS = saved
@P("P109", "H9", "lazy import: baseline_view patched only while binding (not while the runner asks)")
def _():
    sentinel = lambda b: "BOUND-VIEW"; return seam([SA("s3", "refA")], [(EV, "baseline_view", sentinel)], [(EV, "baseline_view", EV.baseline_view)])
@P("P110", "H9", "lazy import: baseline_view patched only while the runner asks (not while binding)")
def _():
    sentinel = lambda b: "CALL-VIEW"; return seam([SA("s3", "refA")], [], [(EV, "baseline_view", sentinel)])
@P("P111", "H9", "lazy import: admission_evidence patched only while binding")
def _():
    from facdrone.pipelines.evaluation import recensus as RC
    return seam([SA("s3", "refA")], [(RC, "admission_evidence", lambda r: NS(reference=lambda: "refA") if "baseline" in r else None)], [])
@P("P112", "H9", "lazy import: admission_evidence patched only while the runner asks")
def _():
    from facdrone.pipelines.evaluation import recensus as RC
    return seam([SA("s3", "refA")], [], [(RC, "admission_evidence", lambda r: NS(reference=lambda: "refA") if "baseline" in r else None)])
@P("P113", "H9", "lazy import: MonitoredSignal replaced on the runner module only while the runner asks")
def _():
    return seam([SA("s3", "refA")], [], [(RN, "MonitoredSignal", lambda *a: ("PATCHED-MS", a[0]))])
@P("P114", "H9", "lazy import: MonitoredSignal replaced on the runner module only while binding")
def _():
    return seam([SA("s3", "refA")], [(RN, "MonitoredSignal", lambda *a: ("PATCHED-MS", a[0]))], [])
@P("P115", "H9", "lazy import: LiveEvidence.for_window patched after bind; per-signal horizon passed through the bound runner")
def _():
    from facdrone.shells.cppi import snapshot_sources as SS
    import tempfile
    root = Path(tempfile.mkdtemp()); (root / "research").mkdir()
    book = NS(engine_db=None, deps=NS(policy=NS(pod=NS(dynamics=None))))
    calls = []
    with patch.object(SS, "readings_source", lambda *a: (lambda d: {})), patch.object(EV, "read_evidence", lambda e, rs, ws, n: EV.LiveEvidence(rs, [], n)):
        r = SE.bind_lifecycle_monitor(root, book, NS(admitted=lambda: ()), DECLARED)
    fn = r._evidence((WIN,))
    with patch.object(EV.LiveEvidence, "for_window", lambda self, ident, w: calls.append((ident, w.window_id)) or "PF"):
        out = fn(RN.MonitoredSignal("a", 2, replace(BASELINE, traded_horizon_days=7), "d", 1, ANCHOR), WIN)
    return {"r": out, "calls": calls, "unpatched": fn(RN.MonitoredSignal("a", 2, BASELINE, "d", 1, ANCHOR), WIN)}
@P("P125", "H1", "run: the evidence callable returns the empty string for every window (falsy reason)")
def _(): return go([TARGET], ev_const(""))
@P("P126", "H1", "run: evidence None for the LAST window only (earlier windows fine)")
def _():
    seq = iter([BREACH, BREACH, None]); return go([TARGET], lambda t, w: next(seq))
@P("P127", "H1", "run: evidence is an empty tuple for a window")
def _(): return go([TARGET], ev_const(()))
@P("P128", "H5", "run: duplicate monitored target listed twice (same windows twice)")
def _(): return go([TARGET, TARGET], ev_const(BREACH))

# ------------------------------------------------------------------ H12 root.pod_lifecycle_standing and study_baseline ordering
@P("P116", "H12", "pod_lifecycle_standing: call order of store/study_baseline for two members, flags read first")
def _():
    log = []
    class FS:
        def open_flags(self): log.append("open_flags"); return ()
        def readings(self, n, v): log.append(("readings", n, v)); return [rd("w1", "survival", rules=RN._rules_revision(DECLARED)), rd("w1", "ic_gap", rules="old")]
        def standing(self, n, v): log.append(("standing", n, v)); return UnrecordedStanding(n, v)
    members = {"carry"}
    class Meta: pass
    fac = lambda n, v: NS(metadata=NS(name=n, version=v))
    with patch("facdrone.factors.registry.production_factors", lambda: NS(factors=lambda: [fac("carry", 3), fac("other", 1), fac("carry", 4)])), \
         patch.object(RT, "study_baseline", lambda st, n, v: log.append(("study", n, v))):
        out = RT.pod_lifecycle_standing(FS(), {"lifecycle_monitor": asdict(DECLARED)}, members, AS_OF)
    return {"log": log, "out": out}
@P("P117", "H12", "pod_lifecycle_standing: no members match")
def _():
    with patch("facdrone.factors.registry.production_factors", lambda: NS(factors=lambda: [NS(metadata=NS(name="x", version=1))])):
        return RT.pod_lifecycle_standing(NS(open_flags=lambda: ()), {"lifecycle_monitor": asdict(DECLARED)}, {"carry"}, AS_OF)
@P("P118", "H12", "pod_lifecycle_standing: store.readings raises for the second member (nothing returned, first signal already projected)")
def _():
    log = []
    class FS:
        def open_flags(self): return ()
        def readings(self, n, v):
            log.append(("readings", n, v))
            if v == 4: raise RuntimeError("db")
            return []
        def standing(self, n, v): log.append(("standing", n, v)); return UnrecordedStanding(n, v)
    with patch("facdrone.factors.registry.production_factors", lambda: NS(factors=lambda: [NS(metadata=NS(name="carry", version=3)), NS(metadata=NS(name="carry", version=4))])), \
         patch.object(RT, "study_baseline", lambda st, n, v: None):
        try: RT.pod_lifecycle_standing(FS(), {"lifecycle_monitor": asdict(DECLARED)}, {"carry"}, AS_OF)
        except RuntimeError as e: return {"exc": str(e), "log": log}
@P("P119", "H12", "pod_lifecycle_standing: parameters without lifecycle_monitor (declaration error)")
def _(): return RT.pod_lifecycle_standing(NS(open_flags=lambda: ()), {}, {"carry"}, AS_OF)
@P("P120", "H10", "study_baseline: latest cited reference wins among several; none cited -> None (store stub, ledger patched)")
def _():
    from facdrone.pipelines.evaluation import census_protocol as CP, recensus as RC
    ref = {"A": "refA", "B": "refB"}
    with patch.object(RT, "project_root", lambda: NS(__truediv__=lambda s, x: NS(__truediv__=lambda s2, y: NS(read_text=lambda: "")))) if False else patch.object(RT, "project_root", lambda: ROOT / "docs_missing_probe"), \
         patch.object(CP, "parse_ledger", lambda t: LEDGER_ROWS[:2]), patch.object(RC, "admission_evidence", lambda r: NS(reference=lambda: ref[r["id"]]) if r["id"] in ref else None), \
         patch("pathlib.Path.read_text", lambda self, *a, **k: ""):
        a = RT.study_baseline(NS(census_references=lambda n, v: ("refA", "zzz", "refB")), "s", 1)
        b = RT.study_baseline(NS(census_references=lambda n, v: ("zzz",)), "s", 1)
        c = RT.study_baseline(NS(census_references=lambda n, v: ()), "s", 1)
    return [a, b, c]

# ------------------------------------------------------------------ module-level facts
@P("P121", "H9", "import-time: facdrone/facdrone_kernel modules loaded by importing lifecycle_seams alone (fresh interpreter)")
def _():
    import subprocess
    code = ("import sys,os; sys.path.insert(0,%r); os.environ['FACDRONE_PROJECT_ROOT']=%r; before=set(sys.modules); import facdrone.shells.cppi.lifecycle_seams; "
            "print(sorted(m for m in set(sys.modules)-before if m.startswith(('facdrone','facdrone_kernel'))))") % (str(ROOT / "src"), str(ROOT))
    r = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True, env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"}); return r.stdout.strip() + r.stderr.strip()[-200:]
@P("P122", "H9", "import-time: modules loaded by importing lifecycle_projection alone (fresh interpreter)")
def _():
    import subprocess
    code = ("import sys,os; sys.path.insert(0,%r); os.environ['FACDRONE_PROJECT_ROOT']=%r; before=set(sys.modules); import facdrone.shells.lifecycle_projection; "
            "print(sorted(m for m in set(sys.modules)-before if m.startswith(('facdrone','facdrone_kernel'))))") % (str(ROOT / "src"), str(ROOT))
    r = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True, env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"}); return r.stdout.strip() + r.stderr.strip()[-200:]
@P("P123", "H9", "review command output unaffected: lifecycle_review module-level names (timedelta/datetime present as module globals)")
def _(): return {"timedelta_global": "timedelta" in vars(GR), "datetime_global": "datetime" in vars(GR), "date_global": "date" in vars(GR)}
@P("P124", "H2", "rules revision of the declaration (pure) and flag reason text (pure)")
def _():
    lat = FRESH["traded_horizon_ic"]; return [RN._rules_revision(DECLARED), RN._flag_reason("traded_horizon_ic", 3, ("a", "b", "c"), lat)]

json.dump(RES, open(OUT, "w", encoding="utf-8"), indent=1, default=str)
print(len(RES), "probes")
