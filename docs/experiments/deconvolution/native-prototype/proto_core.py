"""S10 shared recording/replay/acceptance engine (R30 preregistration).

One generic implementation; scope adapters (scope_e3.py / scope_e4.py) import the frozen
oracles' strata (Env classes with op_*, SEEDS, generated) and supply descriptors. This
module owns only the generic engine, faithfully reproducing the frozen recorders'
semantics: declared normalization, L1/L2 replay, golden body bytes (json.dumps
sort_keys/indent=0/allow_nan, LF-body digests), deadline-killed subprocess arms and
serial pytest groups. Runs under the specimen venv like the frozen apparatus."""
import hashlib
import json
import os
import subprocess
import sys
import tempfile
import time
from contextlib import contextmanager
from datetime import date, datetime
from enum import Enum
from pathlib import Path

canon = lambda v: json.dumps(v, sort_keys=True, separators=(",", ":"))  # noqa: E731


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
        from dataclasses import fields, is_dataclass
        if is_dataclass(v) and not isinstance(v, type):
            return {"__type__": type(v).__name__, **{f.name: self(getattr(v, f.name)) for f in fields(v)}}
        if isinstance(v, dict): return {str(k): self(x) for k, x in v.items()}  # noqa: E701
        if isinstance(v, (list, tuple)): return [self(x) for x in v]  # noqa: E701
        if isinstance(v, (set, frozenset)): return sorted((self(x) for x in v), key=canon)  # noqa: E701
        if hasattr(v, "model_dump"): return self(v.model_dump())  # noqa: E701
        if hasattr(v, "obj"): return self(v.obj)  # noqa: E701
        return f"<{type(v).__name__}>"  # never repr: addresses would vary between runs


def run_sequence(env, seq: dict, seeds: dict) -> dict:
    with tempfile.TemporaryDirectory(prefix="proto-artifacts-") as root:
        env.reset(Path(root))
        for o in seeds.get(seq["start"], []) if seq["start"] else []:
            env.apply(o, record=False)  # generated sequences may start from a recorded state
        if seq["origin"] == "generated" and hasattr(env, "repo") and env.repo is None:
            env.apply({"op": "boot"}, record=False)  # E1b: every generated sequence has a live runtime
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


def attached(env_cls, engine, root, listen_all: bool = False):
    """E1b listens on the engine instance; E4 on the Engine class (its CLI builds engines)."""
    from facdrone_testing.migrated_schema import is_migrated, migrate
    from sqlalchemy import event
    if not is_migrated(engine):
        migrate(engine)
    env = env_cls(engine, root)
    if listen_all:
        from sqlalchemy import Engine
        event.listen(Engine, "before_cursor_execute", env.on_sql)
    else:
        event.listen(engine, "before_cursor_execute", env.on_sql)
    return env


def detach(engine, env, listen_all: bool = False) -> None:
    from sqlalchemy import event
    if listen_all:
        from sqlalchemy import Engine
        event.remove(Engine, "before_cursor_execute", env.on_sql)
    else:
        event.remove(engine, "before_cursor_execute", env.on_sql)


@contextmanager
def embedded_engine(e2_dir: Path):
    sys.path.insert(0, str(e2_dir))
    import e2_winshim
    e2_winshim._patch()
    from facdrone_data.testing import pg_server
    from sqlalchemy import create_engine
    server = pg_server.start()
    engine = create_engine(server.dsn)
    try:
        yield engine
    finally:
        engine.dispose()
        server.stop()


def write_golden(out: Path, meta: dict, seqs: list) -> tuple[str, str]:
    """Frozen recorder output shape: sort_keys/indent=0/allow_nan; LF body digest;
    as-stored bytes LF (declared normalization recorded beside the digest)."""
    body = json.dumps({"meta": meta, "sequences": seqs}, sort_keys=True, indent=0, allow_nan=True)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(body, encoding="utf-8", newline="\n")
    digest = hashlib.sha256(body.encode()).hexdigest()
    Path(str(out) + ".sha256").write_text(
        f"{digest}  {out.name}\n# lf-body sha256 {digest} {len(body.encode()):,} bytes (proto_core writer, LF as stored)\n",
        encoding="ascii", newline="\n")
    return digest, body


def record(env_cls, engine, root, seeds_pairs, generated_seqs, guard, meta_of, out: Path) -> dict:
    """Generic recorder: guard the unmodified copy, run seed+generated sequences, emit the
    frozen golden shape. guard() raises on a changed root; meta_of(seqs) builds scope meta."""
    guard()
    seqs = [{"name": k, "origin": "seed", "start": None, "ops": v} for k, v in seeds_pairs] + generated_seqs
    env = attached(env_cls, engine, root)
    try:
        [s.update(run_sequence(env, s, dict(seeds_pairs))) for s in seqs]
    finally:
        detach(engine, env)
    meta = meta_of(seqs)
    digest, _ = write_golden(out, meta, seqs)
    return {k: meta[k] for k in ("sequences", "operations")} | {"sha256_lf_body": digest}


def replay_engine(env_cls, engine, root, seeds_pairs, golden: Path, only: set | None = None, listen_all: bool = False) -> str | None:
    """Re-run every recorded operation sequence; return the first L1 divergence or None."""
    trace = json.loads(Path(golden).read_text(encoding="utf-8"))
    env = attached(env_cls, engine, root, listen_all)
    try:
        for seq in trace["sequences"]:
            if only and seq["name"] not in only:
                continue
            got = run_sequence(env, seq, dict(seeds_pairs))
            if (d := first_diff(seq["start_tables"], got["start_tables"])) is not None:
                return f"{seq['name']} start state: {d}"
            for i, (o, a, b) in enumerate(zip(seq["ops"], seq["observations"], got["observations"])):
                if (d := first_diff(a, b)) is not None:
                    return f"{seq['name']} op {i} {canon(o)[:160]}: {d}"
        return None
    finally:
        detach(engine, env, listen_all)


def replay_to_l2(env_cls, engine, root, seeds_pairs, goldens: list, out: Path, listen_all: bool = False) -> dict:
    """The replay target the frozen arms ran inside pytest: L1 asserts on first divergence,
    L2 collected per operation without failing; writes the round report."""
    env, l2, n = attached(env_cls, engine, root, listen_all), [], {"sequences": 0, "operations": 0}
    try:
        for g in goldens:
            for seq in json.loads(Path(g).read_text(encoding="utf-8"))["sequences"]:
                got = run_sequence(env, seq, dict(seeds_pairs))
                d = first_diff(seq["start_tables"], got["start_tables"])
                assert d is None, f"{Path(g).name} {seq['name']} start state: {d}"
                for i, (o, a, b) in enumerate(zip(seq["ops"], seq["observations"], got["observations"])):
                    sa, sb = a.pop("sql", None), b.pop("sql", None)
                    d = first_diff(a, b)
                    assert d is None, f"{Path(g).name} {seq['name']} op {i} {canon(o)[:160]}: {d}"
                    if canon(sa) != canon(sb):
                        l2.append({"golden": Path(g).name, "sequence": seq["name"], "op_index": i, "op": o,
                                   "recorded": sa, "replayed": sb})
                    n["operations"] += 1
                n["sequences"] += 1
    finally:
        detach(engine, env, listen_all)
        out.write_text(json.dumps({"written": time.time(), "root": str(root), "golden": str(goldens[0]), **n,
                                   "level2_ops": len(l2), "divergences": l2}, indent=1), encoding="utf-8")
    return {**n, "level2_ops": len(l2)}


def call(cmd, deadline, **kw) -> int:
    """Run cmd; past the deadline kill its whole tree (Windows has no process-group kill)."""
    proc = subprocess.Popen(cmd, **kw)
    try:
        return proc.wait(timeout=None if deadline is None else max(1.0, deadline - time.monotonic()))
    except subprocess.TimeoutExpired:
        subprocess.run(["taskkill", "/F", "/T", "/PID", str(proc.pid)], capture_output=True)
        print("proto: deadline exceeded (infrastructure, not a verdict)")
        return 124


def probe(venv: Path, root: Path, module: str, env: dict) -> None:
    p = subprocess.run([str(venv), "-c", f"import {module} as m; print(m.__file__)"], env=env, capture_output=True, text=True)
    if not Path(p.stdout.strip() or "/").resolve().is_relative_to(root.resolve()):
        raise SystemExit(f"INVALID scope not imported from {root}: {p.stdout}{p.stderr}")


def run_groups(venv: Path, root: Path, groups: list, marks: str, env: dict, deadline=None, extra_first=None) -> int:
    """Serial per-group pytest with the frozen flags; exit 0 only if every group passes."""
    for cwd, args in groups:
        cmd = [str(venv), "-m", "pytest", "-q", "-c", "pyproject.toml", "--rootdir", ".", "-p", "no:cacheprovider",
               "-p", "e2_winshim", *(extra_first or []), "-x", "--timeout=600", "-m", marks, *args]
        t0 = time.perf_counter()
        code = call(cmd, deadline, cwd=root / cwd, env=env)
        print(f"proto {cwd}: exit {code} in {time.perf_counter() - t0:.1f}s", flush=True)
        if code not in (0, 5):  # 5: every test in the group deselected
            return 1
    return 0


def arm_env(root: Path, here: Path, paths: list, extra: dict | None = None) -> dict:
    e = dict(os.environ, OPENBLAS_NUM_THREADS="1", OMP_NUM_THREADS="1", MKL_NUM_THREADS="1",
             PYTHONWARNINGS="ignore", PYTHONDONTWRITEBYTECODE="1",
             PYTHONPATH=os.pathsep.join(map(str, [root / "src", *paths])))
    return e | (extra or {})


def build_groups(files: list, bad: list, packages: tuple, replay_target: Path | None, here: Path) -> list:
    """(cwd relative to root, pytest args): root-suite seed files, then satellite packages;
    baseline deselections applied; the replay target rides the root group via -p fixtures."""
    pick = lambda pre, xs: [x.removeprefix(pre) for x in xs if x.startswith(pre)]  # noqa: E731
    out = [(".", pick("", [f for f in files if f.startswith("tests/")]))]
    out += [(f"packages/facdrone-{p}", pick(f"packages/facdrone-{p}/", files)) for p in packages]
    out = [(cwd, args + [f"--deselect={n}" for n in pick("" if cwd == "." else cwd + "/", bad)
                         if cwd != "." or n.startswith("tests/")] if args else []) for cwd, args in out]
    if replay_target is not None and out:
        out[0][1].extend(["-p", "facdrone_testing.fixtures", str(replay_target)])
    return [g for g in out if g[1]]
