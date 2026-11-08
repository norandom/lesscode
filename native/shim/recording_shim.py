"""Embedded oracle instrumentation shim for the Rust recording engine (R36, task 8.4).

Runs INSIDE the pinned specimen venv (``.runs/e2-facdrone/venv``). Owns only
what cannot leave Python: frozen scope-adapter/oracle imports, the embedded
engine, listener attach/detach, and sequence execution with the frozen
start-state semantics. Everything else — declared meta assembly, golden body
bytes, LF-body digests, replay diffing, L2 collection, deadline-killed arms,
serial pytest groups — lives in ``native/src/recording.rs``. Declared
normalization itself executes inside the frozen oracle envs (their own
``Norm``), so this shim only transports already-normalized JSON.

Counted apparatus code (R36: new shared code counts wherever it lives). The
frozen adapters import the stage-1 reference engine module for their own use;
this shim never calls it. Interface: JSON files under ``--out-dir``; stdout
stays empty (Rust captures it).
"""

import contextlib
import importlib
import json
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
LESSCODE = HERE.parents[1]
PROTO = LESSCODE / "docs" / "experiments" / "deconvolution" / "native-prototype"
sys.path[:0] = [str(PROTO)]


def adapter_of(scope: str):
    """The frozen scope adapter (scope_e3/scope_e4) — descriptors + oracle."""
    return importlib.import_module(f"scope_{scope}")


@contextlib.contextmanager
def embedded_engine(e2_dir: Path):
    """The frozen embedded engine: winshim patch, ephemeral pg_server, engine."""
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


def attached(env_cls, engine, root: Path, listen_all: bool):
    """Migrate the schema, build the env, listen for SQL (engine or class)."""
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


def detach(engine, env, listen_all: bool) -> None:
    from sqlalchemy import event

    if listen_all:
        from sqlalchemy import Engine

        event.remove(Engine, "before_cursor_execute", env.on_sql)
    else:
        event.remove(engine, "before_cursor_execute", env.on_sql)


def run_sequence(env, seq: dict, seeds: dict) -> dict:
    """Frozen sequence execution: reset, seed start state, boot generated, run."""
    with tempfile.TemporaryDirectory(prefix="proto-artifacts-") as root:
        env.reset(Path(root))
        for o in seeds.get(seq["start"], []) if seq["start"] else []:
            env.apply(o, record=False)  # generated sequences may start from a recorded state
        if seq["origin"] == "generated" and hasattr(env, "repo") and env.repo is None:
            env.apply({"op": "boot"}, record=False)  # E1b: generated sequences have a live runtime
        start = env.dump() if seq["start"] else None
        return {"start_tables": start, "observations": [env.apply(o) for o in seq["ops"]]}


def meta_inputs(scope: str, O) -> dict:
    """Live-version and scope-fingerprint inputs; Rust assembles the final meta."""
    import hashlib
    import hypothesis
    import sqlalchemy

    meta = {
        "python": sys.version,
        "hypothesis": hypothesis.__version__,
        "sqlalchemy": sqlalchemy.__version__,
        "hypothesis_seed": O.SEED,
        "generated_requested": O.N_GENERATED,
        "root": str(O.ROOT),
    }
    if scope == "e3":
        meta["scope"] = sys.modules["scope"].record(
            O.ROOT
        )  # the adapter imported the fingerprint module
    else:
        meta["scope_sha256"] = {
            f: hashlib.sha256((O.ROOT / f).read_bytes()).hexdigest() for f in sorted(O.SCOPE)
        }
        meta["alphabet"] = sorted(a[3:] for a in dir(O.Env) if a.startswith("op_"))
    return meta


def write(out_dir: Path, name: str, payload: dict) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / name).write_text(json.dumps(payload, indent=1), encoding="utf-8")


def cmd_record(scope: str, out_dir: Path) -> int:
    adapter = adapter_of(scope)
    O = adapter.O
    seqs = [
        {"name": k, "origin": "seed", "start": None, "ops": v} for k, v in O.SEEDS
    ] + O.generated(O.N_GENERATED)
    listen_all = scope == "e4"
    with embedded_engine(adapter.E2) as engine:
        env = attached(O.Env, engine, O.ROOT, listen_all)
        try:
            [s.update(run_sequence(env, s, dict(O.SEEDS))) for s in seqs]
        finally:
            detach(engine, env, listen_all)
        meta = meta_inputs(scope, O)
    write(out_dir, "seqs.json", {"sequences": seqs})
    write(out_dir, "meta-input.json", meta)
    return 0


def cmd_replay(scope: str, goldens: list, out_dir: Path) -> int:
    adapter = adapter_of(scope)
    O = adapter.O
    listen_all = scope == "e4"
    got = []
    with embedded_engine(adapter.E2) as engine:
        env = attached(O.Env, engine, O.ROOT, listen_all)
        try:
            for golden in goldens:
                for seq in json.loads(Path(golden).read_text(encoding="utf-8"))["sequences"]:
                    got.append(
                        {
                            "golden": Path(golden).name,
                            "name": seq["name"],
                            **run_sequence(env, seq, dict(O.SEEDS)),
                        }
                    )
        finally:
            detach(engine, env, listen_all)
    write(out_dir, "got.json", {"sequences": got})
    return 0


def main(argv: list) -> int:
    cmd, scope = argv[1], argv[argv.index("--scope") + 1]
    out_dir = Path(argv[argv.index("--out-dir") + 1])
    if cmd == "record":
        return cmd_record(scope, out_dir)
    goldens = [a for a in argv[argv.index("--goldens") + 1 :]]
    return cmd_replay(scope, goldens, out_dir)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
