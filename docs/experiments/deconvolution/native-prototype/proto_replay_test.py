"""Shared S10 replay target (replaces test_e1b/e3/e4_replay.py inside acceptance arms).
PROTO_SCOPE selects the adapter; the pg_engine fixture comes from facdrone_testing."""
import importlib
import os
from pathlib import Path

HERE = Path(__file__).resolve().parent


def test_proto_replay(pg_engine) -> None:
    adapter = importlib.import_module({"e3": "scope_e3", "e4": "scope_e4"}[os.environ["PROTO_SCOPE"]])
    print(f"proto replay: scope={os.environ['PROTO_SCOPE']} round={os.environ.get(adapter.ROUND_ENV, '0')}", flush=True)
    adapter.run_replay_target(pg_engine)
