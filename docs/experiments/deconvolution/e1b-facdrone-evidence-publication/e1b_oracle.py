"""E1b trace oracle: record and replay the effects of the evidence-publication scope.

  e1b_oracle.py record [--out PATH] [--only NAME,...] [--generated N]
  e1b_oracle.py replay [--golden PATH] [--only NAME,...]
Driver mode starts Facdrone's embedded PostgreSQL through e2_winshim and migrates it with
Facdrone's own Alembic chain. In pytest, test_e1b_replay.py calls replay_engine(pg_engine).
Environment: E1B_SOURCE_ROOT (source root whose src/ must provide `facdrone`; default the
frozen copy). Record refuses a root whose scope differs from the R15 manifest.
"""
import base64, contextlib, hashlib, io, json, os, runpy, sys, tempfile, zlib
from dataclasses import fields, is_dataclass, replace
from datetime import date, datetime
from enum import Enum
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import scope  # noqa: E402

GOLDEN = HERE / "results" / "golden-trace.json"
SEED, N_GENERATED = 20261103, 40
ROOT = Path(os.environ.get("E1B_SOURCE_ROOT", scope.FROZEN)).resolve()

from sqlalchemy import JSON, Text, create_engine, event, select, text  # noqa: E402

from facdrone.schema import (DECISION_EVIDENCE, DECISION_INPUT_SLICE, LIVE_DECISION_EVENT, LIVE_RUNTIME_STATE,  # noqa: E402
                             SIM_COST_EVENTS, SIM_EQUITY, SIM_EVENTS, SIM_HOLDINGS, SIM_RUNS, SIM_TRADES)
from facdrone.shells import decision_evidence_publication as P  # noqa: E402
from facdrone.shells.cppi import state_repository as live  # noqa: E402
from facdrone.shells.decision_evidence_reader import SliceResolver, read_resolved_evidence  # noqa: E402
from facdrone.shells.wiring import run_evidence_store as sim  # noqa: E402
from facdrone_kernel.domain.decision_evidence import InputSlice, SliceReference  # noqa: E402
from facdrone_kernel.events import SimEvent  # noqa: E402
from facdrone_testing.core.cppi import make_state  # noqa: E402
from facdrone_testing.core.evidence import evidence_batch  # noqa: E402
from facdrone_testing.core.run_slices import capture_details  # noqa: E402
from facdrone_testing.core.runs import INPUTS, FakeBroker, full_result  # noqa: E402

for _m in (P, live, sim):  # fail closed: the scope must come from the root under test
    if not Path(_m.__file__).resolve().is_relative_to(ROOT):
        raise RuntimeError(f"{_m.__name__} imported from {_m.__file__}, not under {ROOT}")

TABLES = (DECISION_INPUT_SLICE, DECISION_EVIDENCE, LIVE_DECISION_EVENT, LIVE_RUNTIME_STATE,
          SIM_RUNS, SIM_EVENTS, SIM_EQUITY, SIM_TRADES, SIM_HOLDINGS, SIM_COST_EVENTS)
NOW = evidence_batch().identity.as_of
IDENTITY = live.RuntimeIdentity("e1b-account", "e1b-strategy")
WRITER_NS = f"{IDENTITY.account_id}:{IDENTITY.strategy_id}"
canon = lambda v: json.dumps(v, sort_keys=True, separators=(",", ":"))


class Norm:
    """Declared normalisation: datetimes become first-seen ordinals; everything else verbatim."""

    def __init__(self):
        self.times: dict[str, int] = {}

    def __call__(self, v):  # noqa: PLR0911
        if isinstance(v, datetime): return "<t%d>" % self.times.setdefault(v.isoformat(), len(self.times))  # noqa: E701
        if isinstance(v, date): return v.isoformat()  # noqa: E701
        if v is None or isinstance(v, (str, bool, int, float)): return v  # noqa: E701
        if isinstance(v, Enum): return self(v.value)  # noqa: E701
        if isinstance(v, (bytes, bytearray, memoryview)): return {"bytes_sha256": hashlib.sha256(bytes(v)).hexdigest()}  # noqa: E701
        if is_dataclass(v) and not isinstance(v, type):
            return {"__type__": type(v).__name__, **{f.name: self(getattr(v, f.name)) for f in fields(v)}}
        if isinstance(v, dict): return {str(k): self(x) for k, x in v.items()}  # noqa: E701
        if isinstance(v, (list, tuple)): return [self(x) for x in v]  # noqa: E701
        if isinstance(v, (set, frozenset)): return sorted((self(x) for x in v), key=canon)  # noqa: E701
        if hasattr(v, "model_dump"): return self(v.model_dump())  # noqa: E701
        if hasattr(v, "obj"): return self(v.obj)  # psycopg Json/Jsonb wrappers  # noqa: E701
        return f"<{type(v).__name__}>"  # never repr: addresses would vary between runs


def vspec(col: str = "value", n: int = 4, var: str = "plain") -> dict:
    """One-column slice: -0.0, 1, 2, ... (prefixes grow into deltas); variants break the prefix."""
    head = {"plain": -0.0, "zero": 0.0, "head99": 99}.get(var)
    rows = [[i + 0.5] for i in range(n)] if var == "float" else [[head if i == 0 else i] for i in range(n)]
    return {"c": [col], "r": rows}


def capture(spec: dict, shape: str) -> dict[str, object]:
    """Facdrone's capture_details with the first slice replaced (as slice_capture does) and the
    input list arranged by shape: duplicate, forward or missing consumed references, or twins."""
    payload = capture_details()
    c = P._CAPTURE.validate_json(zlib.decompress(base64.b64decode(str(payload["consumed_capture"]))))
    value = InputSlice(tuple(spec["c"]), tuple(tuple(r) for r in spec["r"]))
    s, m = replace(c.inputs[0], value=value), c.inputs[1]
    digest = hashlib.sha256(P._canonical(value)).hexdigest()
    ref = replace(s, key="other:GC=F", value=SliceReference(P.CONSUMED_URI + digest, digest))
    inputs = {"single": (s, m), "dup": (s, m, ref), "forward": (ref, m, s), "missing": (m, ref),
              "twin": (s, replace(s, key="other:GC=F"))}[shape]
    raw = zlib.compress(P._CAPTURE.dump_json(replace(c, inputs=inputs)))
    return payload | {"consumed_capture": base64.b64encode(raw).decode()}


def _state(s):
    return None if s is None else {"version": s.version, "last": s.last_decision_id, "handover": s.handover_id}


class Env:
    def __init__(self, engine, source_root: Path):
        self.engine, self.source_root, self.sql, self.recording = engine, source_root, [], False
        self.n, self.root, self.repo, self.state, self.last = Norm(), None, None, None, None

    def on_sql(self, conn, cursor, statement, parameters, context, executemany):
        if self.recording:
            self.sql.append([statement, self.n(parameters)])

    def reset(self, root: Path):
        with self.engine.begin() as c:
            c.execute(text("TRUNCATE " + ", ".join(t.name for t in TABLES)))
        self.n, self.root, self.repo, self.state, self.last = Norm(), root, None, None, None

    def dump(self) -> dict:
        out = {}
        with self.engine.connect() as c:
            for t in TABLES:  # JSON columns as stored text: representation, not only content
                cols = [col.cast(Text).label(col.name) if isinstance(col.type, JSON) else col for col in t.c]
                rows = [dict(r) for r in c.execute(select(*cols)).mappings()]
                rows.sort(key=lambda r: json.dumps(r, sort_keys=True, default=str))
                out[t.name] = [self.n(r) for r in rows]
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

    # ── live writer: cppi/state_repository ──
    def op_boot(self, o):
        self.repo = live.SqlLiveStateRepository(self.engine)
        initial = live.PersistedLiveState(IDENTITY, 1, make_state(), (), None, (), None, NOW)
        self.state = self.repo.bootstrap(IDENTITY, initial)
        return _state(self.state)

    def op_restart(self, o):  # a new writer process: fresh slice ledger, state re-read
        self.repo = live.SqlLiveStateRepository(self.engine)
        self.state = self.repo.load(IDENTITY)
        return _state(self.state)

    def op_handover(self, o):
        with self.engine.begin() as c:
            c.execute(LIVE_RUNTIME_STATE.update().values(handover_id=o["ns"]))
        self.state = self.repo.load(IDENTITY)
        return _state(self.state)

    def op_commit(self, o):
        cur = self.state  # "dup" injects a rollback: the conflict comes after slices were published
        did = self.last if o["decision"] == "dup" and self.last else f"{WRITER_NS}:v{cur.version + 1}"
        ns = (cur.handover_id or WRITER_NS) if o["ns"] == "auto" else o["ns"]
        src = {"as_of": NOW.isoformat()} if o["mode"] == "absent" else capture(o["v"], o["shape"])
        binding = P.EvidenceBinding("live", did, ns, "c", "cfg")
        if o["mode"] == "legacy":
            payload = P.bind_evidence(src, scope="live", execution_id=did, code="c", configuration="cfg")
        elif o["mode"] == "legacy-refs":  # dictionary-shaped batches carrying run-slice references
            bound = P.seal_evidence_with_run_slices(src, binding)
            payload = {"consumed_evidence": [json.loads(b.canonical) for b in bound.batches or ()]}
        else:
            payload = P.bind_evidence_with_run_slices(src, binding)
        basis = replace(cur, version=cur.version - 1) if o["decision"] == "stale" else cur
        nxt = replace(basis, version=basis.version + 1, last_decision_id=did)
        self.state = self.repo.commit(basis, nxt, live.DecisionEventRecord(did, IDENTITY, nxt.version, NOW, payload))
        self.last = did
        return _state(self.state)

    def op_tamper(self, o):
        table, column = (DECISION_INPUT_SLICE, "row_digest") if o["table"] == "slice" else (DECISION_EVIDENCE, "content_digest")
        with self.engine.begin() as c:
            return c.execute(table.update().values({column: "0" * 64})).rowcount

    # ── reader ──
    def op_read(self, o):
        with self.engine.connect() as c:
            ids = sorted(c.execute(select(DECISION_EVIDENCE.c.execution_id).where(
                DECISION_EVIDENCE.c.scope == o["scope"]).distinct()).scalars())
            out = {}
            for i in [*ids, "e1b-absent"]:
                try:
                    out[i] = read_resolved_evidence(c, o["scope"], i, NOW, SliceResolver())
                except Exception as e:  # noqa: BLE001
                    out[i] = {"error": [type(e).__name__, str(e)]}
            return out

    # ── simulation writer: wiring/run_evidence_store ──
    def op_sim_persist(self, o):
        events = tuple(SimEvent(NOW.date(), "decision_evidence", details=capture(v, o["shape"])) for v in o["vs"])
        return sim.persist_run(self.engine, replace(full_result(), evidence_events=events), INPUTS, FakeBroker(160_000.0))

    def op_sim_supersede(self, o):
        return sim.delete_run_evidence(self.engine, INPUTS)

    # ── pure publication codecs ──
    def op_seal(self, o):
        src = capture(o["v"], o["shape"])
        if o["mode"] == "legacy":
            args = dict(scope=o["scope"], execution_id="execution:pure", code="c", configuration="cfg")
            bound, envelope = P.seal_evidence(src, **args), P.bind_evidence(src, **args)
        else:
            binding = P.EvidenceBinding(o["scope"], "execution:pure", o["ns"], "c", "cfg")
            bound, envelope = P.seal_evidence_with_run_slices(src, binding), P.bind_evidence_with_run_slices(src, binding)
        sealed = bound.batches or ()
        return {"stored": P.stored_bound_payload(bound), "slices": [P.slice_identity(s) for s in bound.slices],
                "sealed": [[s.digest, s.schema_version, hashlib.sha256(s.stored.encode()).hexdigest()] for s in sealed],
                "batches_agree": P.payload_batches(envelope) == tuple(s.batch for s in sealed),
                "stored_agree": P.stored_evidence_payload(envelope) == P.stored_bound_payload(bound)}

    def op_compact(self, o):
        from facdrone_research.commands.study_arm_evidence import StudySlices
        return P.compact_capture(capture(o["v"], o["shape"]) | {"marker": [1, 2]}, StudySlices(self.root).write)

    def op_r16(self, o):  # the R16 codec harness, unchanged, run in-process on the loaded scope
        saved, buf = (sys.argv[:], sys.path[:]), io.StringIO()
        sys.argv = [str(scope.R16_CHECK), str(self.source_root)]
        try:
            with contextlib.redirect_stdout(buf):
                runpy.run_path(str(scope.R16_CHECK), run_name="__main__")
        finally:
            sys.argv, sys.path[:] = saved
        return json.loads(buf.getvalue())


BASE, GROWN, LATER = vspec(n=4), vspec(n=5), vspec(n=6)
C = lambda v, decision="next", mode="typed", ns="auto", shape="single": {
    "op": "commit", "v": v, "decision": decision, "mode": mode, "ns": ns, "shape": shape}
SEEDS = [  # transcribed from the seed tests (named) and the R16 harness
    ("S01-live-reuse-grow-restart", [{"op": "boot"}, {"op": "handover", "ns": "handover:one"}, C(BASE), C(BASE),
                                     C(GROWN), {"op": "restart"}, C(LATER), {"op": "read", "scope": "live"}]),
    ("S02-namespace-change", [{"op": "boot"}, {"op": "handover", "ns": "handover:one"}, C(BASE),
                              {"op": "handover", "ns": "handover:two"}, C(GROWN, ns="handover:one"), C(GROWN)]),
    ("S03-rollback-then-retry", [{"op": "boot"}, C(BASE), C(GROWN, decision="dup"), C(GROWN),
                                 {"op": "restart"}, C(LATER), {"op": "read", "scope": "live"}]),
    ("S04-live-legacy-v1", [{"op": "boot"}, C(BASE, mode="legacy"), C(GROWN, mode="legacy", shape="dup"),
                            {"op": "read", "scope": "live"}]),
    ("S05-legacy-dict-refs-refused", [{"op": "boot"}, C(BASE, mode="legacy-refs"), {"op": "read", "scope": "live"}]),
    ("S06-stale-and-absent", [{"op": "boot"}, C(BASE, decision="stale"), C(BASE, mode="absent"), C(BASE, shape="missing")]),
    ("S07-sim-retry-supersede", [*({"op": "sim_persist", "vs": [v], "shape": "single"} for v in (BASE, BASE, GROWN)),
                                 {"op": "read", "scope": "simulation"}, {"op": "sim_supersede"}, {"op": "read", "scope": "simulation"}]),
    ("S08-sim-tamper", [{"op": "sim_persist", "vs": [BASE], "shape": "dup"}, {"op": "tamper", "table": "slice"},
                        {"op": "sim_persist", "vs": [BASE], "shape": "dup"}, {"op": "read", "scope": "simulation"}]),
    ("S09-codecs-compact", [{"op": "seal", "v": BASE, "shape": sh, "mode": m, "scope": "live", "ns": "run:test"}
                            for sh in ("single", "dup", "forward", "missing", "twin") for m in ("legacy", "run")]
     + [{"op": "compact", "v": BASE, "shape": sh} for sh in ("single", "dup", "forward")]),
    ("S10-r16-codec-harness", [{"op": "r16"}]),
    ("S11-delta-chain-keyframe", [{"op": "boot"}, *(C(vspec(n=4 + k)) for k in range(22))]),
]
SEED_STARTS = ("S01-live-reuse-grow-restart", "S02-namespace-change", "S03-rollback-then-retry", "S07-sim-retry-supersede")


def generated(n: int) -> list[dict]:
    if n <= 0: return []  # noqa: E701
    from hypothesis import HealthCheck, Phase, given, seed, settings, strategies as st

    vals = st.builds(vspec, st.sampled_from(["value", "other"]), st.integers(0, 8),
                     st.sampled_from(["plain", "zero", "head99", "float"]))
    shapes = st.sampled_from(["single", "single", "dup", "forward", "missing", "twin"])
    d = lambda name, **kw: st.fixed_dictionaries({"op": st.just(name), **kw})
    commit = d("commit", v=vals, shape=shapes, mode=st.sampled_from(["typed", "typed", "legacy", "legacy-refs", "absent"]),
               ns=st.sampled_from(["auto", "auto", "handover:one", "handover:two", " "]),
               decision=st.sampled_from(["next", "next", "next", "dup", "stale"]))
    options = [  # commits weighted x3: the transactional writer is the scope's core
        commit, commit, commit,
        d("restart"), d("handover", ns=st.sampled_from(["handover:one", "handover:two"])),
        d("read", scope=st.sampled_from(["live", "simulation"])), d("tamper", table=st.sampled_from(["slice", "evidence"])),
        d("sim_persist", vs=st.lists(vals, min_size=1, max_size=2), shape=shapes), d("sim_supersede"),
        d("compact", v=vals, shape=shapes),
        d("seal", v=vals, shape=shapes, mode=st.sampled_from(["legacy", "run"]),
          scope=st.sampled_from(["live", "simulation"]), ns=st.sampled_from(["run:test", "other/run"]))]
    op = st.sampled_from(options).flatmap(lambda strategy: strategy)
    found = []

    @seed(SEED)
    @settings(max_examples=n, database=None, phases=[Phase.generate], deadline=None,
              suppress_health_check=list(HealthCheck))
    @given(st.fixed_dictionaries({"start": st.sampled_from([None, *SEED_STARTS]), "ops": st.lists(op, min_size=3, max_size=8)}))
    def collect(s):
        found.append(s)

    collect()
    return [{"name": f"G{i:02d}", "origin": "generated", "start": s["start"], "ops": s["ops"]} for i, s in enumerate(found)]


def run_sequence(env: Env, seq: dict) -> dict:
    with tempfile.TemporaryDirectory(prefix="e1b-artifacts-") as root:
        env.reset(Path(root))
        for o in dict(SEEDS).get(seq["start"], []) if seq["start"] else []:
            env.apply(o, record=False)  # generated sequences may start from a recorded state
        if seq["origin"] == "generated" and env.repo is None:
            env.apply({"op": "boot"}, record=False)  # every generated sequence has a live runtime
        start = env.dump() if seq["start"] else None
        return {"start_tables": start, "observations": [env.apply(o) for o in seq["ops"]]}


def signatures(seq: dict) -> set[str]:
    return {f"{o['op']}:{'ok' if 'result' in ob else ob['error'][0] + '<' + str(ob['error'][2])}"
            for o, ob in zip(seq["ops"], seq["observations"])}


def first_diff(a, b, path=""):
    if canon(a) == canon(b):
        return None
    if isinstance(a, dict) and isinstance(b, dict):
        for k in sorted(set(a) | set(b)):
            d = f"{path}/{k} (present on one side only)" if k not in a or k not in b else first_diff(a[k], b[k], f"{path}/{k}")
            if d is not None:
                return d
    if isinstance(a, list) and isinstance(b, list):
        d = next((d for i, (x, y) in enumerate(zip(a, b)) if (d := first_diff(x, y, f"{path}[{i}]")) is not None), None)
        return d or f"{path} (length {len(a)} recorded vs {len(b)} replayed)"
    return f"{path}: recorded {canon(a)[:300]} vs replayed {canon(b)[:300]}"


def _attached(engine):
    from facdrone_testing.migrated_schema import is_migrated, migrate
    if not is_migrated(engine):
        migrate(engine)
    env = Env(engine, ROOT)
    event.listen(engine, "before_cursor_execute", env.on_sql)
    return env


def replay_engine(engine, golden: Path = GOLDEN, only: set | None = None) -> str | None:
    """Re-run every recorded operation sequence; return the first divergence or None."""
    trace, env = json.loads(Path(golden).read_text(encoding="utf-8")), _attached(engine)
    try:
        for seq in trace["sequences"]:
            if only and seq["name"] not in only:
                continue
            got = run_sequence(env, seq)
            if (d := first_diff(seq["start_tables"], got["start_tables"])) is not None:
                return f"{seq['name']} start state: {d}"
            for i, (o, a, b) in enumerate(zip(seq["ops"], seq["observations"], got["observations"])):
                if (d := first_diff(a, b)) is not None:
                    return f"{seq['name']} op {i} {canon(o)[:160]}: {d}"
        return None
    finally:
        event.remove(engine, "before_cursor_execute", env.on_sql)


@contextlib.contextmanager
def embedded_engine():
    sys.path.insert(0, str(HERE.parent / "e2-facdrone-edge-census"))
    import e2_winshim
    e2_winshim._patch()
    from facdrone_data.testing import pg_server
    server = pg_server.start()
    engine = create_engine(server.dsn)
    try:
        yield engine
    finally:
        engine.dispose()
        server.stop()


def record(engine, out: Path, only: set | None, n: int) -> dict:
    rec = scope.record(ROOT)
    if rec["changed"]:
        raise RuntimeError(f"record only from the unmodified copy; changed: {rec['changed']}")
    seqs = [{"name": k, "origin": "seed", "start": None, "ops": v} for k, v in SEEDS] + generated(n)
    seqs = [s for s in seqs if not only or s["name"] in only]
    env = _attached(engine)
    try:
        [s.update(run_sequence(env, s)) for s in seqs]
    finally:
        event.remove(engine, "before_cursor_execute", env.on_sql)
    import hypothesis, sqlalchemy
    seed_sigs = set().union(*(signatures(s) for s in seqs if s["origin"] == "seed"))
    gen_sigs = set().union(set(), *(signatures(s) for s in seqs if s["origin"] == "generated"))
    meta = {"hypothesis_seed": SEED, "generated_requested": n, "python": sys.version,
            "hypothesis": hypothesis.__version__, "sqlalchemy": sqlalchemy.__version__, "scope": rec,
            "sequences": len(seqs), "operations": sum(len(s["ops"]) for s in seqs),
            "seed_signatures": sorted(seed_sigs), "generated_only_signatures": sorted(gen_sigs - seed_sigs),
            "harness_suspects": sorted(x for x in seed_sigs | gen_sigs if any(
                e in x for e in ("AttributeError", "KeyError", "NameError", "UnboundLocalError")))}
    body = json.dumps({"meta": meta, "sequences": seqs}, sort_keys=True, indent=0, allow_nan=True)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(body, encoding="utf-8")
    digest = hashlib.sha256(body.encode()).hexdigest()
    Path(str(out) + ".sha256").write_text(f"{digest}  {out.name}\n", encoding="ascii")
    return {k: meta[k] for k in ("sequences", "operations", "generated_only_signatures", "harness_suspects")} | {"sha256": digest}


def main(argv: list[str]) -> int:
    mode, opts = argv[0], dict(zip(argv[1::2], argv[2::2]))
    only = set(opts["--only"].split(",")) if "--only" in opts else None
    with embedded_engine() as engine:
        if mode == "record":
            print(json.dumps(record(engine, Path(opts.get("--out", GOLDEN)), only,
                                    int(opts.get("--generated", N_GENERATED))), indent=1))
            return 0
        print(divergence := replay_engine(engine, Path(opts.get("--golden", GOLDEN)), only) or "replay: no divergence")
        return 0 if divergence == "replay: no divergence" else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
