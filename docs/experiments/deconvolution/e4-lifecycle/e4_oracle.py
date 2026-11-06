"""E4 trace oracle: record and replay the effects of the lifecycle monitor group (deconvolution-loop task 5.2).

  e4_oracle.py record [--out PATH] [--generated N] [--only NAME,...]
  e4_oracle.py replay [--golden PATH] [--only NAME,...]
Driver mode starts Facdrone's embedded PostgreSQL through e1b_oracle.embedded_engine (e2_winshim) and migrates it with
Facdrone's Alembic chain; in pytest, test_e4_replay.py uses `pg_engine`. E4_SOURCE_ROOT is the root whose src/ provides
`facdrone` (default the frozen copy); the scope must import from it. Generic parts (Norm, canon, first_diff,
embedded_engine) come from e1b_oracle unchanged; the alphabet (scope-rationale section 4 minus `lifecycle_review`),
the environment and the normalisations are this scope's. Declared normalisations: datetimes become first-seen
ordinals (Norm); table dumps are sorted by canonical JSON; `events` by (moment, revision, event_id); `admitted`,
`open_flags`, `overdue_flags` and a seams run's monitored targets by (signal, version), since the specimen leaves
those orders to PostgreSQL; emitted records are grouped per signal@version in emission order; a MonitorSummary's
flags and unavailable tuples are sorted. The clock is the driver's (`tick` moves it); the CLI's own stores get it
by patching lifecycle_store.LifecycleStore (the CLI builds stores on the wall clock, lifecycle_store.py:69).
"""
import hashlib, json, os, sys, tempfile
from dataclasses import asdict, replace
from datetime import timedelta
from pathlib import Path
from types import SimpleNamespace

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "e1b-facdrone-evidence-publication"))
ROOT = Path(os.environ.get("E4_SOURCE_ROOT", HERE.parents[3] / ".runs/e2-facdrone/source")).resolve()
os.environ.setdefault("E1B_SOURCE_ROOT", str(ROOT))
import e1b_oracle  # noqa: E402  (generic: Norm, canon, first_diff, embedded_engine)
from e1b_oracle import Norm, canon, embedded_engine, first_diff  # noqa: E402, F401

import pytest  # noqa: E402  (MonkeyPatch, used outside pytest too)
from sqlalchemy import Engine, JSON, Text, create_engine, event, select, text  # noqa: E402
from typer.testing import CliRunner  # noqa: E402

from facdrone.schema import DECISION_EVIDENCE, DECISION_INPUT_SLICE, SIGNAL_LIFECYCLE_EVENT, SIGNAL_MONITOR_READING  # noqa: E402
from facdrone.shells import lifecycle_evidence, lifecycle_store  # noqa: E402
from facdrone.shells.cli import lifecycle_group  # noqa: E402
from facdrone.shells.cppi.lifecycle_seams import bind_lifecycle_monitor  # noqa: E402
from facdrone.shells.lifecycle_monitor_runner import MonitoredSignal, MonitorRunner, MonitorSources  # noqa: E402
from facdrone.shells.lifecycle_projection import labeled_reason, monitor_label, signal_standing  # noqa: E402
from facdrone.shells.lifecycle_store import LifecycleStore, TransitionRequest, bind_writers  # noqa: E402
from facdrone_kernel.domain.signal_lifecycle import (AdmissionEvidence, Enrollment, LifecycleState, OpenFlag,  # noqa: E402
                                                     QualifiedBreach, RecordedHumanRetirement, StandingRevision,
                                                     UnrecordedStanding)
from facdrone_testing.core.cppi import AS_OF  # noqa: E402
from facdrone_testing.core.lifecycle import ANCHOR, BASELINE, BREACH, DECLARED, evidence as _evidence  # noqa: E402

for _m in (lifecycle_store, lifecycle_evidence, lifecycle_group):  # fail closed: the scope comes from the root under test
    if not Path(_m.__file__).resolve().is_relative_to(ROOT):
        raise RuntimeError(f"{_m.__name__} imported from {_m.__file__}, not under {ROOT}")

GOLDEN = HERE / "results" / "golden-e4.json"
SEED, N_GENERATED = 20261106, 24
TABLES = (SIGNAL_LIFECYCLE_EVENT, SIGNAL_MONITOR_READING, DECISION_EVIDENCE)
SIXTY = lambda c: c * 64  # noqa: E731
ADMISSIONS = {"pass": AdmissionEvidence("f", "d", "2026-09-01", SIXTY("v"), SIXTY("b"), True, True),
              "renewed": AdmissionEvidence("f", "d", "2026-09-07", SIXTY("w"), SIXTY("c"), True, True),
              "incomplete": AdmissionEvidence("f", "d", "x", "v", "b", True, False),
              "failing": AdmissionEvidence("f", "d", "x", "v", "b", False, True)}
BREACH_EVIDENCE = QualifiedBreach("decay", ("r1",), ("w1", "w2"), "IC below the baseline twice")
HUMAN = RecordedHumanRetirement("operator", "superseded", "notes/retire.md")
WINDOW_EVIDENCE = {"breach": BREACH, "within": _evidence((0.1, 0.1, 0.1)), "thin": _evidence(()), "none": "no consumed IC"}
BASELINE_ROW = {"route": "scored", "traded_horizon_days": 10, "traded_ic": {"mean": 0.05, "dispersion": 0.2, "count": 120},
                "survival": {"ratio": 0.5}, "states": [{"state": "calm", "share": 0.75, "ic_mean": 0.04},
                {"state": "stressed", "share": 0.25, "ic_mean": None}], "cost": {"financing_annual_rate": 0.03},
                "premise": {"sensitivities": [["rates", "tested"], ["inflation", "untested"]]}, "digest": SIXTY("b")}
_CLOCK = [AS_OF]
clock_now = lambda: _CLOCK[0]  # noqa: E731


def set_clock(days: float) -> None:
    _CLOCK[0] = AS_OF + timedelta(days=days)


class _Clocked(LifecycleStore):
    """The specimen's store with the driver's clock as the default (the CLI passes none)."""

    def __init__(self, engine, now=None):
        super().__init__(engine, now or clock_now)


def ordered(rows: list) -> list:
    return sorted(rows, key=lambda r: json.dumps(r, sort_keys=True, default=str))


def ordered_events(events) -> list:
    return sorted(events, key=lambda e: (e.event_at, e.revision, e.event_id))


def grouped(records: list[dict]) -> dict:
    out: dict = {}
    for r in records:
        out.setdefault(f"{r.get('signal')}@{r.get('version')}", []).append(r)
    return dict(sorted(out.items()))


def _old_rules() -> str:  # the earlier method revision the tests build (facdrone_testing/core/lifecycle.py)
    original = asdict(DECLARED)
    del original["runner"]["ic_trailing_days"]
    return hashlib.sha256(json.dumps(original, sort_keys=True).encode()).hexdigest()[:16]


def _ledger_row(number: int) -> dict:  # as tests/unit/test_lifecycle_monitor_binding_db.py builds them
    from facdrone_testing.core.census_verdicts import verdict_row
    return verdict_row(True) | {"run_date": f"2026-09-0{number}", "trial": {"result_digest": str(number) * 64},
                                "baseline": {"signal": "sig", "version": 1, "digest": str(number) * 64, "route": "scored",
                                             "traded_horizon_days": 1, "traded_ic": {"mean": 0.05 if number == 1 else 0.4}}}


class Env:
    def __init__(self, engine, source_root: Path):
        self.engine, self.source_root, self.sql, self.recording = engine, source_root, [], False
        self.n, self.root, self.store = Norm(), None, None
        os.environ["FACDRONE_TEST_PG_DSN"] = engine.url.render_as_string(hide_password=False)  # the CLI's seam; never recorded

    def on_sql(self, conn, cursor, statement, parameters, context, executemany):
        if self.recording:
            self.sql.append([statement, self.n(parameters)])

    def reset(self, root: Path):
        with self.engine.begin() as c:
            c.execute(text("TRUNCATE " + ", ".join(t.name for t in (*TABLES, DECISION_INPUT_SLICE))))
        set_clock(0)
        self.n, self.root, self.store = Norm(), root, LifecycleStore(self.engine, clock_now)

    def dump(self) -> dict:
        out = {}
        with self.engine.connect() as c:
            for t in TABLES:  # JSON columns as stored text: representation, not only content
                cols = [col.cast(Text).label(col.name) if isinstance(col.type, JSON) else col for col in t.c]
                out[t.name] = [self.n(r) for r in ordered([dict(r) for r in c.execute(select(*cols)).mappings()])]
        return out

    def files(self) -> dict:
        return {p.relative_to(self.root).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
                for p in sorted(self.root.rglob("*")) if p.is_file()}

    def apply(self, o: dict, record: bool = True):
        self.sql, self.recording = [], record
        try:
            out = {"result": self.n(getattr(self, "op_" + o["op"])(o))}
        except Exception as e:  # noqa: BLE001 — every failure is an observation
            cause = e.__cause__ or e.__context__
            out = {"error": [type(e).__name__, str(e), type(cause).__name__ if cause else None]}
        finally:
            self.recording = False
        if record:
            out.update(sql=self.sql, tables=self.dump(), files=self.files())
            return out

    # ── clock and store writers (lifecycle_store) ──
    def op_tick(self, o):
        set_clock(o["days"])
        return {"clock": clock_now().isoformat()}

    def op_enroll(self, o):
        return bind_writers(self.store).enroll(o["signal"], o["version"], Enrollment(o.get("actor", "operator"), o.get("reason", "new")), o.get("reason", "enrolled"))

    def op_admit(self, o):
        return bind_writers(self.store).admit(o["signal"], o["version"], ADMISSIONS[o.get("evidence", "pass")], self._expected(o), o.get("reason", "census passed"))

    def op_flag(self, o):
        if "rollback" not in o:
            return bind_writers(self.store).flag(o["signal"], o["version"], BREACH_EVIDENCE, self._expected(o), o.get("reason", "persistent breach"))
        with self.engine.begin() as conn:  # the monitor's shape: the writer bound into the caller's transaction
            receipt = bind_writers(self.store, conn, o.get("decision", "d-txn")).flag(o["signal"], o["version"], BREACH_EVIDENCE, self._expected(o), o.get("reason", "in transaction"))
            if o["rollback"]:
                raise RuntimeError("caller rolled back after the flag")
        return receipt

    def op_retire(self, o):
        return bind_writers(self.store).retire(o["signal"], o["version"], HUMAN, self._expected(o), o.get("reason", "superseded"))

    def op_transition(self, o):  # the raw request: a wrong authority or evidence kind for the target
        request = TransitionRequest(o["signal"], o["version"], LifecycleState(o["target"]), o["authority"], ADMISSIONS[o.get("evidence", "pass")], self._expected(o), o.get("reason", "x"))
        return self.store.transition(request)

    def _expected(self, o) -> int:
        if o.get("expected") != "standing":
            return o.get("expected", 0)
        s = self.store.standing(o["signal"], o["version"])
        return s.revision if isinstance(s, StandingRevision) else 0

    # ── store readers ──
    def op_events(self, o):
        return ordered_events(self.store.events(o["signal"], o["version"]))

    def op_standing(self, o):
        return self.store.standing(o["signal"], o["version"])

    def op_census_references(self, o):
        return self.store.census_references(o["signal"], o["version"])

    def op_admitted(self, o):
        return sorted(self.store.admitted(), key=lambda s: (s.signal, s.version))

    def op_open_flags(self, o):
        return sorted(self.store.open_flags(), key=lambda f: (f.signal, f.version))

    def op_overdue_flags(self, o):
        return sorted(self.store.overdue_flags(timedelta(days=o.get("days", 14))), key=lambda f: (f.signal, f.version))

    def op_readings(self, o):
        return self.store.readings(o["signal"], o["version"])

    # ── the runner over scripted sources (lifecycle_monitor_runner) ──
    def _targets(self, specs):
        out = []
        for t in specs:
            s = self.store.standing(t["signal"], t["version"])
            expected = t.get("expected", "standing")
            expected = (s.revision if isinstance(s, StandingRevision) else 0) if expected == "standing" else expected
            out.append(MonitoredSignal(t["signal"], t["version"], BASELINE, SIXTY(t.get("digest", "b")), expected, AS_OF + timedelta(days=t.get("anchor_days", -4))))
        return tuple(out)

    def _evidence_for(self, spec):
        def for_window(target, window):
            kind = spec.get("signals", {}).get(target.signal, spec.get("windows", {}).get(window.window_id, spec.get("default", "breach")))
            if kind == "raise":
                raise RuntimeError("no evidence source")
            return WINDOW_EVIDENCE[kind]
        return for_window

    def _flag_in(self, mode):
        store = self.store
        if mode == "never":
            return lambda _c, _d: lambda *_: (_ for _ in ()).throw(AssertionError("no flag may be attempted"))
        if mode == "broken":
            return lambda _c, _d: (_ for _ in ()).throw(RuntimeError("killed before flag commit"))
        if mode != "racing":
            return lambda conn, d: bind_writers(store, conn, d).flag

        def racing(conn, decision):  # another writer lands the flag first; the runner's own clashes inside its transaction
            def flag(signal, version, breach, expected, reason):
                seen = store.standing(signal, version)
                bind_writers(store).flag(signal, version, breach, expected, "the other writer")
                store.standing = lambda _s, _v: seen  # the stale read
                try:
                    return bind_writers(store, conn, decision).flag(signal, version, breach, expected, reason)
                finally:
                    del store.standing
            return flag
        return racing

    def _run(self, runner, o, emitted):
        say = lambda ev, msg, **f: emitted.append({"event": ev, "message": msg, **f})  # noqa: E731
        summary = runner.run(AS_OF + timedelta(days=o.get("as_of_days", 0)), o.get("decision", "d1"), say)
        return replace(summary, flags=tuple(sorted(summary.flags)), unavailable=tuple(sorted(summary.unavailable)))

    def op_run(self, o):
        targets = o.get("targets", [{"signal": "carry", "version": 3}])
        monitored = (lambda: (_ for _ in ()).throw(RuntimeError("the store is unreachable"))) if targets == "raise" else (lambda: self._targets(targets))
        sources = MonitorSources(monitored, lambda _w: self._evidence_for(o.get("evidence", {})), self._flag_in(o.get("flag", "bound")))
        runner = MonitorRunner(self.engine, DECLARED, sources)
        if o.get("rules") == "old":
            runner._rules = _old_rules()
        if o.get("append") == "twice":
            original = runner._append
            runner._append = lambda *a: (original(*a), original(*a))  # ON CONFLICT for every reading id
        emitted: list = []
        summary = self._run(runner, o, emitted)
        return {"summary": summary, "emitted": grouped(emitted)}

    # ── evidence projections (lifecycle_evidence) ──
    def op_seed_evidence(self, o):
        from facdrone.shells.decision_evidence_store import append_decision_evidence
        from facdrone_testing.core.evidence import evidence_batch
        with self.engine.begin() as conn:
            return append_decision_evidence(conn, evidence_batch())

    def op_live_evidence(self, o):
        from datetime import date
        from facdrone.pipelines.evaluation.monitor_windows import Window
        from facdrone_kernel.domain.enums import MarketState
        states = {date.fromisoformat(d): MarketState(s) for d, s in o.get("states", {}).items()}
        readings = lambda _through: {d: SimpleNamespace(entry=SimpleNamespace(state=s)) for d, s in states.items()}  # noqa: E731
        w = o["window"]
        return lifecycle_evidence.live_evidence(self.engine, readings, tuple(o["signal"]), Window(w[0], date.fromisoformat(w[1]), date.fromisoformat(w[2])), o.get("ic_days", 450))

    def op_baseline_view(self, o):
        row = dict(BASELINE_ROW)
        if o.get("route") == "rule_book":
            row.update(route="rule_book", traded_ic=None, survival=None)
        return lifecycle_evidence.baseline_view(row)

    # ── pure projection (lifecycle_projection) ──
    def op_standing_view(self, o):
        now = clock_now()
        standing = {"admitted": StandingRevision("carry", 3, LifecycleState.ADMITTED, 2, now - timedelta(days=30), "c"),
                    "flagged": StandingRevision("carry", 3, LifecycleState.FLAGGED, 3, now, None),
                    "unrecorded": UnrecordedStanding("cot", 1)}[o.get("state", "admitted")]
        row = lambda w, m, d, v=None, **x: {"window_id": w, "monitor": m, "disposition": d, "value": v, "boundary": 0.025, "comparator": "<",  # noqa: E731
                                            "units": "IC", "known_at": now, "payload": {"limitation": "no IC consumed" if d == "not_run" else None}, **x}
        readings = {"none": (), "not_run": (row("2026-09-04", "window", "not_run"),),
                    "mix": (row("2026-09-04", "traded_horizon_ic", "within", 0.03), row("2026-09-05", "traded_horizon_ic", "breach", -0.1), row("2026-09-05", "survival", "unmeasured")),
                    "superseded": (row("2026-09-05", "traded_horizon_ic", "breach", -0.1, baseline_digest="old", rules_revision="r1"),
                                   row("2026-09-08", "survival", "unmeasured", baseline_digest="new", rules_revision="r1"))}[o.get("readings", "none")]
        reason = labeled_reason(o.get("reason", "state_ic:calm breached on 3 consecutive windows (the latest 3, a to c, qualify); last reading -0.1 < 0.02 IC"))
        flag = {None: None, "recent": OpenFlag("carry", 3, "e1", reason, now - timedelta(days=3), "d-7"),
                "old": OpenFlag("carry", 3, "e2", reason, now - timedelta(days=20), "d-2")}[o.get("flag")]
        view = signal_standing(standing, BASELINE_ROW if o.get("baseline", True) else None, readings, flag, (now, timedelta(days=14)))
        return {"view": view, "labels": {k: monitor_label(k) for k in ("traded_horizon_ic", "state_ic:calm", "cost:spread", "some_future_monitor")}}

    # ── the daemon seams over a ledger file (cppi/lifecycle_seams, wiring/lifecycle_root is residue) ──
    def op_ledger(self, o):
        path = self.root / "research" / "equity_census_verdicts.jsonl"
        path.parent.mkdir(exist_ok=True)
        path.write_text("".join(json.dumps(_ledger_row(n)) + "\n" for n in o["rows"]), encoding="utf-8")
        return path.name

    def op_admit_durable(self, o):  # recensus (outside the scope) puts the ledger row to the bound admission writer
        from facdrone.pipelines.evaluation.recensus import admit_durable
        return admit_durable(_ledger_row(o["row"]), self.store.standing, self.store.census_references, bind_writers(self.store).admit)

    def op_seams(self, o):
        from facdrone_testing.core.composed_books import make_book
        patch, emitted = pytest.MonkeyPatch(), []
        try:
            if o.get("evidence") == "breach":  # as the binding test: the retained-evidence join is not the subject
                patch.setattr(lifecycle_evidence.LiveEvidence, "for_window", lambda *_: BREACH)
            runner = bind_lifecycle_monitor(self.root, replace(make_book(False), engine_db=self.engine), self.store, DECLARED)
            monitored = sorted(runner._monitored(), key=lambda t: (t.signal, t.version))
            summary = self._run(runner, o, emitted) if o.get("run", True) else None
        finally:
            patch.undo()
        return {"monitored": monitored, "summary": summary, "emitted": grouped(emitted)}

    # ── the operator commands (cli/lifecycle_group; `admit` and `review` are residue) ──
    def _cli(self, args, o):
        from facdrone.shells.cli.app import app
        patch = pytest.MonkeyPatch()
        patch.setattr(lifecycle_group, "_lifecycle_store", lambda: self.store)
        patch.setattr(lifecycle_store, "LifecycleStore", _Clocked)  # `retire` builds its own store
        if args[0] == "readings":  # the calculator inputs the runner test supplies (method_revisions)
            cited = {"digest": SIXTY("b"), "traded_horizon_days": 1} if o.get("cited", "match") == "match" else None
            patch.setattr(lifecycle_group, "read_parameters", lambda _: {"lifecycle_monitor": asdict(DECLARED)})
            patch.setattr(lifecycle_group, "_cited_baseline", lambda *_: cited)
            patch.setattr(lifecycle_group, "_pod_of", lambda *_: "fixture")
            patch.setattr(lifecycle_group, "configured_pods", lambda **_: SimpleNamespace(get_scored=lambda _: SimpleNamespace(dynamics=None)))
            patch.setattr(lifecycle_evidence, "baseline_view", lambda _: BASELINE)
            patch.setattr(lifecycle_evidence, "live_evidence", lambda *_: WINDOW_EVIDENCE[o.get("evidence", "breach")])
        try:
            r = CliRunner().invoke(app, ["lifecycle", *args])
        finally:
            patch.undo()
        return {"exit_code": r.exit_code, "output": r.output}

    def op_cli_enroll(self, o):
        return self._cli(["enroll", o["identity"], "--reason", o.get("reason", "running incumbent"), "--actor", o.get("actor", "op")], o)

    def op_cli_retire(self, o):
        return self._cli(["retire", o["identity"], "--reason", o.get("reason", "superseded"), "--actor", "op", "--record", "n", "--expected-revision", str(o["expected"])], o)

    def op_cli_readings(self, o):
        return self._cli(["readings", o.get("target", "carry@3"), "--window", o.get("window", "2026-09-05")], o)


E = lambda sig, v=1: [{"op": "enroll", "signal": sig, "version": v}, {"op": "admit", "signal": sig, "version": v, "expected": 1}]  # noqa: E731
RUN = lambda **k: {"op": "run", **k}  # noqa: E731
SEEDS = [  # transcribed from the seed tests (named)
    ("S01-store-full-cycle", [{"op": "standing", "signal": "cot", "version": 1}, *E("cot"), {"op": "standing", "signal": "cot", "version": 1},
                              {"op": "flag", "signal": "cot", "version": 1, "expected": 2}, {"op": "open_flags"}, {"op": "admit", "signal": "cot", "version": 1, "expected": 3, "reason": "fresh census with a new baseline"},
                              {"op": "open_flags"}, {"op": "retire", "signal": "cot", "version": 1, "expected": 4}, {"op": "events", "signal": "cot", "version": 1},
                              {"op": "admit", "signal": "cot", "version": 1, "expected": 5, "reason": "too late"}, {"op": "events", "signal": "cot", "version": 1}, {"op": "admitted"}]),
    ("S02-refusals-recorded", [{"op": "enroll", "signal": "econ", "version": 1}, {"op": "admit", "signal": "econ", "version": 1, "expected": 0, "reason": "stale"},
                               {"op": "admit", "signal": "econ", "version": 1, "expected": 1, "evidence": "incomplete"}, {"op": "admit", "signal": "econ", "version": 1, "expected": 1, "evidence": "failing"},
                               {"op": "flag", "signal": "econ", "version": 1, "expected": 1}, {"op": "transition", "signal": "econ", "version": 1, "target": "admitted", "authority": "monitor", "expected": 1},
                               {"op": "events", "signal": "econ", "version": 1}, {"op": "standing", "signal": "econ", "version": 1}, {"op": "census_references", "signal": "econ", "version": 1}]),
    ("S03-flag-closes-by-census-or-retirement", [*E("cot", 2), {"op": "flag", "signal": "cot", "version": 2, "expected": 2}, {"op": "flag", "signal": "cot", "version": 2, "expected": 3},
                                                 {"op": "overdue_flags"}, {"op": "tick", "days": 30}, {"op": "overdue_flags"}, {"op": "retire", "signal": "cot", "version": 2, "expected": 3}, {"op": "open_flags"}]),
    ("S04-refused-row-at-the-flags-revision", [*E("term"), {"op": "admit", "signal": "term", "version": 1, "expected": 2}, {"op": "flag", "signal": "term", "version": 1, "expected": 2},
                                               {"op": "flag", "signal": "term", "version": 1, "expected": 3}, {"op": "events", "signal": "term", "version": 1}, {"op": "open_flags"},
                                               {"op": "admit", "signal": "term", "version": 1, "expected": 3}, {"op": "open_flags"}]),
    ("S05-identical-refusal-once", [{"op": "enroll", "signal": "clock", "version": 1}, {"op": "admit", "signal": "clock", "version": 1, "expected": 0}, {"op": "admit", "signal": "clock", "version": 1, "expected": 0},
                                    {"op": "events", "signal": "clock", "version": 1}, {"op": "tick", "days": 1}, {"op": "admit", "signal": "clock", "version": 1, "expected": 0}, {"op": "events", "signal": "clock", "version": 1}]),
    ("S06-cli-enroll", [{"op": "cli_enroll", "identity": "inc@1"}, {"op": "cli_enroll", "identity": "inc@1"}, {"op": "cli_enroll", "identity": "inc"}, {"op": "standing", "signal": "inc", "version": 1}]),
    ("S07-cli-retire", [{"op": "enroll", "signal": "cli", "version": 1}, {"op": "cli_retire", "identity": "cli@1", "expected": 0}, {"op": "cli_retire", "identity": "cli@1", "expected": 1},
                        {"op": "cli_retire", "identity": "cli@1", "expected": 2}, {"op": "cli_retire", "identity": "cli", "expected": 1}, {"op": "events", "signal": "cli", "version": 1}]),
    ("S08-runner-flags-once-and-repeat-serves-nothing", [*E("carry", 3), RUN(as_of_days=-2, decision="d1"), RUN(decision="d2"), RUN(decision="d3"), {"op": "events", "signal": "carry", "version": 3},
                                                         {"op": "readings", "signal": "carry", "version": 3}, {"op": "open_flags"}, {"op": "admitted"}]),
    ("S09-watch-unmeasured-not-run", [*E("carry", 3), RUN(evidence={"windows": {"2026-09-04": "none", "2026-09-05": "thin", "2026-09-06": "breach"}}), {"op": "readings", "signal": "carry", "version": 3}, {"op": "open_flags"}]),
    ("S10-within-resets", [*E("carry", 3), RUN(evidence={"windows": {"2026-09-05": "within"}}), {"op": "open_flags"}, RUN(as_of_days=2, decision="d2"), {"op": "open_flags"}]),
    ("S11-refused-flag-then-next-window", [*E("carry", 3), RUN(targets=[{"signal": "carry", "version": 3, "expected": 7}]), {"op": "events", "signal": "carry", "version": 3}, RUN(as_of_days=1, decision="d2"), {"op": "open_flags"}]),
    ("S12-renewed-baseline-fresh-sequence", [*E("carry", 3), RUN(), {"op": "admit", "signal": "carry", "version": 3, "expected": "standing", "evidence": "renewed"},
                                             RUN(as_of_days=2, decision="d2", targets=[{"signal": "carry", "version": 3, "digest": "c", "anchor_days": 0}]), {"op": "readings", "signal": "carry", "version": 3}, {"op": "open_flags"}]),
    ("S13-racing-writer-inside-the-transaction", [*E("carry", 3), RUN(flag="racing"), {"op": "events", "signal": "carry", "version": 3}, {"op": "readings", "signal": "carry", "version": 3}]),
    ("S14-failing-signal-named-others-run", [*E("carry", 3), RUN(targets=[{"signal": "momentum", "version": 1, "expected": 0}, {"signal": "carry", "version": 3}], evidence={"default": "within", "signals": {"momentum": "raise"}}),
                                             {"op": "readings", "signal": "carry", "version": 3}, {"op": "readings", "signal": "momentum", "version": 1}]),
    ("S15-monitored-raises", [*E("carry", 3), RUN(targets="raise"), {"op": "readings", "signal": "carry", "version": 3}]),
    ("S16-window-commits-resume-and-idempotent-append", [*E("carry", 3), RUN(evidence={"windows": {"2026-09-05": "raise"}}, decision="interrupted"), {"op": "readings", "signal": "carry", "version": 3},
                                                         RUN(append="twice", decision="resumed"), {"op": "readings", "signal": "carry", "version": 3}, RUN(decision="repeat"), {"op": "open_flags"}]),
    ("S17-flag-failure-leaves-final-window-due", [*E("carry", 3), RUN(flag="broken", decision="interrupted-flag"), {"op": "readings", "signal": "carry", "version": 3}, RUN(decision="resumed-flag"), {"op": "open_flags"}]),
    ("S18-time-anchor", [RUN(targets=[{"signal": "carry", "version": 3, "expected": 2, "anchor_days": 0}], flag="never", as_of_days=1),
                         RUN(targets=[{"signal": "carry", "version": 3, "expected": 2, "anchor_days": 0}], flag="never", as_of_days=2), {"op": "readings", "signal": "carry", "version": 3}]),
    ("S19-method-revisions-and-cli-readings", [*E("carry", 3), RUN(rules="old", as_of_days=-1, decision="first"), {"op": "cli_readings"}, RUN(as_of_days=-1, decision="second"), {"op": "cli_readings"},
                                               {"op": "readings", "signal": "carry", "version": 3}, {"op": "open_flags"}, RUN(decision="third"), {"op": "open_flags"}]),
    ("S20-cli-readings-refusals", [*E("carry", 3), RUN(), {"op": "cli_readings", "window": "2026-01-01"}, {"op": "cli_readings", "window": "2026-09-04", "cited": "none"}, {"op": "cli_readings", "target": "carry"},
                                   {"op": "cli_readings", "window": "2026-09-04", "evidence": "none"}, {"op": "cli_readings", "window": "2026-09-04", "evidence": "within"}]),
    ("S21-live-evidence-over-seeded-rows", [{"op": "seed_evidence"}, {"op": "live_evidence", "signal": ["carry", 3, 10], "window": ["2026-09-14", "2026-08-01", "2026-09-14"], "states": {"2026-08-03": "stressed", "2026-08-04": "calm"}},
                                            {"op": "live_evidence", "signal": ["carry", 3, 10], "window": ["2026-09-10", "2026-08-01", "2026-09-10"]}, {"op": "live_evidence", "signal": ["momentum", 1, 10], "window": ["2026-09-14", "2026-08-01", "2026-09-14"]},
                                            {"op": "live_evidence", "signal": ["carry", 3, 20], "window": ["2026-09-14", "2026-08-01", "2026-09-14"]}, {"op": "live_evidence", "signal": ["carry", 3, 10], "window": ["2026-09-14", "2026-08-01", "2026-09-14"], "ic_days": 5}]),
    ("S22-baseline-view", [{"op": "baseline_view"}, {"op": "baseline_view", "route": "rule_book"}]),
    ("S23-projection", [{"op": "standing_view"}, {"op": "standing_view", "state": "flagged"}, {"op": "standing_view", "state": "unrecorded", "baseline": False}, {"op": "standing_view", "readings": "not_run"},
                        {"op": "standing_view", "readings": "mix", "flag": "recent"}, {"op": "standing_view", "readings": "superseded", "flag": "old"}, {"op": "standing_view", "flag": "recent", "reason": "IC below"}]),
    ("S24-seams-renewed-census", [{"op": "enroll", "signal": "sig", "version": 1}, {"op": "ledger", "rows": [1]}, {"op": "admit_durable", "row": 1}, {"op": "seams", "evidence": "breach", "run": False},
                                  {"op": "tick", "days": 4}, {"op": "seams", "evidence": "breach", "as_of_days": 4, "decision": "before-renewal"}, {"op": "ledger", "rows": [1, 2]}, {"op": "admit_durable", "row": 2},
                                  {"op": "admit_durable", "row": 2}, {"op": "seams", "evidence": "breach", "as_of_days": 6, "decision": "after-renewal"}, {"op": "readings", "signal": "sig", "version": 1},
                                  {"op": "ledger", "rows": [1]}, {"op": "seams", "run": False}, {"op": "open_flags"}]),
    ("S25-seams-real-evidence-path", [{"op": "enroll", "signal": "sig", "version": 1}, {"op": "ledger", "rows": [1]}, {"op": "admit_durable", "row": 1}, {"op": "seams", "as_of_days": 4}, {"op": "readings", "signal": "sig", "version": 1},
                                      {"op": "seed_evidence"}, {"op": "seams", "as_of_days": 5, "decision": "d2"}, {"op": "seams", "as_of_days": 5, "decision": "d3", "evidence": "breach"}]),
    ("S26-flag-inside-a-caller-transaction", [*E("carry", 3), {"op": "flag", "signal": "carry", "version": 3, "expected": 2, "rollback": True}, {"op": "events", "signal": "carry", "version": 3},
                                              {"op": "flag", "signal": "carry", "version": 3, "expected": 2, "rollback": False}, {"op": "events", "signal": "carry", "version": 3}, {"op": "open_flags"},
                                              {"op": "flag", "signal": "carry", "version": 3, "expected": 2, "rollback": False}, {"op": "events", "signal": "carry", "version": 3}]),
]
SEED_STARTS = ("S01-store-full-cycle", "S08-runner-flags-once-and-repeat-serves-nothing", "S24-seams-renewed-census")


def generated(n: int) -> list[dict]:
    if n <= 0: return []  # noqa: E701
    from hypothesis import HealthCheck, Phase, given, seed, settings, strategies as st
    sig, ver, exp = st.sampled_from(["carry", "cot", "sig"]), st.sampled_from([1, 3]), st.sampled_from([0, 1, 2, 3, "standing"])
    d = lambda name, **kw: st.fixed_dictionaries({"op": st.just(name), **kw})  # noqa: E731
    kinds = st.sampled_from(["breach", "within", "thin", "none", "raise"])
    ev = st.fixed_dictionaries({"default": kinds, "windows": st.dictionaries(st.sampled_from(["2026-09-04", "2026-09-05", "2026-09-06"]), kinds, max_size=2)})
    options = [d("enroll", signal=sig, version=ver), d("admit", signal=sig, version=ver, expected=exp, evidence=st.sampled_from(["pass", "pass", "renewed", "incomplete", "failing"])),
               d("flag", signal=sig, version=ver, expected=exp), d("retire", signal=sig, version=ver, expected=exp),
               d("transition", signal=sig, version=ver, target=st.sampled_from(["admitted", "flagged", "retired", "candidate"]), authority=st.sampled_from(["census", "monitor", "human", "registry"]), expected=exp),
               d("events", signal=sig, version=ver), d("standing", signal=sig, version=ver), d("admitted"), d("open_flags"), d("overdue_flags", days=st.sampled_from([1, 14])),
               d("readings", signal=sig, version=ver), d("tick", days=st.sampled_from([1, 2, 30])),
               d("run", targets=st.lists(st.fixed_dictionaries({"signal": sig, "version": ver, "expected": exp}), min_size=1, max_size=2), evidence=ev,
                 flag=st.sampled_from(["bound", "bound", "never", "broken", "racing"]), as_of_days=st.sampled_from([-2, 0, 1, 2]), decision=st.sampled_from(["g1", "g2"])),
               d("cli_enroll", identity=st.sampled_from(["carry@3", "cot@1", "cot"])), d("cli_retire", identity=st.sampled_from(["carry@3", "cot@1"]), expected=st.sampled_from([0, 1, 2, 3])),
               d("cli_readings", window=st.sampled_from(["2026-09-04", "2026-09-05", "2026-01-01"]), cited=st.sampled_from(["match", "none"]), evidence=st.sampled_from(["breach", "within", "none"]))]
    op = st.sampled_from(options).flatmap(lambda s: s)
    found = []

    @seed(SEED)
    @settings(max_examples=n, database=None, phases=[Phase.generate], deadline=None, suppress_health_check=list(HealthCheck))
    @given(st.fixed_dictionaries({"start": st.sampled_from([None, *SEED_STARTS]), "ops": st.lists(op, min_size=3, max_size=8)}))
    def collect(s):
        found.append(s)

    collect()
    return [{"name": f"G{i:02d}", "origin": "generated", "start": s["start"], "ops": s["ops"]} for i, s in enumerate(found)]


def run_sequence(env: Env, seq: dict) -> dict:
    with tempfile.TemporaryDirectory(prefix="e4-project-") as root:
        env.reset(Path(root))
        for o in dict(SEEDS).get(seq["start"], []) if seq["start"] else []:
            env.apply(o, record=False)  # a generated sequence may start from a recorded state
        start = env.dump() if seq["start"] else None
        return {"start_tables": start, "observations": [env.apply(o) for o in seq["ops"]]}


def signatures(seq: dict) -> set[str]:
    return {f"{o['op']}:{'ok' if 'result' in ob else ob['error'][0] + '<' + str(ob['error'][2])}" for o, ob in zip(seq["ops"], seq["observations"])}


def _attached(engine):
    from facdrone_testing.migrated_schema import is_migrated, migrate
    if not is_migrated(engine):
        migrate(engine)
    env = Env(engine, ROOT)
    event.listen(Engine, "before_cursor_execute", env.on_sql)  # class level: the CLI builds engines of its own
    return env


def _detach(env):
    event.remove(Engine, "before_cursor_execute", env.on_sql)


def replay_engine(engine, golden: Path = GOLDEN, only: set | None = None) -> tuple[str | None, list[str]]:
    """Re-run every recorded sequence: the first Level 1 divergence (or None) and every Level 2 difference."""
    trace, env, l2 = json.loads(Path(golden).read_text(encoding="utf-8")), _attached(engine), []
    try:
        for seq in trace["sequences"]:
            if only and seq["name"] not in only:
                continue
            got = run_sequence(env, seq)
            if (d := first_diff(seq["start_tables"], got["start_tables"])) is not None:
                return f"{seq['name']} start state: {d}", l2
            for i, (o, a, b) in enumerate(zip(seq["ops"], seq["observations"], got["observations"])):
                if (d := first_diff({k: v for k, v in a.items() if k != "sql"}, {k: v for k, v in b.items() if k != "sql"})) is not None:
                    return f"{seq['name']} op {i} {canon(o)[:160]}: {d}", l2
                if canon(a.get("sql")) != canon(b.get("sql")):
                    l2.append(f"{seq['name']} op {i}")
        return None, l2
    finally:
        _detach(env)


def record(engine, out: Path, only: set | None, n: int) -> dict:
    seqs = [{"name": k, "origin": "seed", "start": None, "ops": v} for k, v in SEEDS] + generated(n)
    seqs = [s for s in seqs if not only or s["name"] in only]
    env = _attached(engine)
    try:
        [s.update(run_sequence(env, s)) for s in seqs]
    finally:
        _detach(env)
    import hypothesis, sqlalchemy
    seed_sigs = set().union(*(signatures(s) for s in seqs if s["origin"] == "seed"))
    gen_sigs = set().union(set(), *(signatures(s) for s in seqs if s["origin"] == "generated"))
    files = {f: hashlib.sha256((ROOT / f).read_bytes()).hexdigest() for f in sorted(SCOPE)}
    meta = {"hypothesis_seed": SEED, "generated_requested": n, "python": sys.version, "hypothesis": hypothesis.__version__,
            "sqlalchemy": sqlalchemy.__version__, "root": str(ROOT), "scope_sha256": files, "alphabet": sorted(a[3:] for a in dir(Env) if a.startswith("op_")),
            "sequences": len(seqs), "operations": sum(len(s["ops"]) for s in seqs), "seed_signatures": sorted(seed_sigs),
            "generated_only_signatures": sorted(gen_sigs - seed_sigs),
            "harness_suspects": sorted(x for x in seed_sigs | gen_sigs if any(e in x for e in ("AttributeError", "KeyError", "NameError", "UnboundLocalError", "TypeError")))}
    body = json.dumps({"meta": meta, "sequences": seqs}, sort_keys=True, indent=0, allow_nan=True)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(body, encoding="utf-8", newline="\n")  # raw bytes equal the hashed body (review 2026-11-06: write_text gave CRLF)
    digest = hashlib.sha256(body.encode()).hexdigest()
    Path(str(out) + ".sha256").write_text(f"{digest}  {out.name}\n", encoding="ascii", newline="\n")
    return {k: meta[k] for k in ("sequences", "operations", "generated_only_signatures", "harness_suspects")} | {"sha256": digest, "bytes": len(body.encode())}


SCOPE = ("src/facdrone/shells/lifecycle_monitor_runner.py", "src/facdrone/shells/lifecycle_store.py", "src/facdrone/shells/cli/lifecycle_group.py",
         "src/facdrone/shells/lifecycle_evidence.py", "src/facdrone/shells/lifecycle_projection.py", "src/facdrone/shells/cppi/lifecycle_seams.py",
         "src/facdrone/shells/wiring/lifecycle_root.py")


def main(argv: list[str]) -> int:
    mode, opts = argv[0], dict(zip(argv[1::2], argv[2::2]))
    only = set(opts["--only"].split(",")) if "--only" in opts else None
    with embedded_engine() as engine:
        if mode == "record":
            print(json.dumps(record(engine, Path(opts.get("--out", GOLDEN)), only, int(opts.get("--generated", N_GENERATED))), indent=1))
            return 0
        l1, l2 = replay_engine(engine, Path(opts.get("--golden", GOLDEN)), only)
        print(f"replay: level1 {l1 or 'no divergence'}; level2 differences {len(l2)} {l2[:5]}")
        return 0 if l1 is None and not l2 else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
