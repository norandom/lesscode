"""Arm B test target (run_arm.py B): replay the frozen golden trace on Facdrone's `pg_engine`."""
import os
from pathlib import Path

import e1b_oracle


def test_e1b_trace_replay(pg_engine) -> None:
    golden = Path(os.environ.get("E1B_GOLDEN", e1b_oracle.GOLDEN))
    only = set(os.environ["E1B_ONLY"].split(",")) if os.environ.get("E1B_ONLY") else None
    divergence = e1b_oracle.replay_engine(pg_engine, golden, only)
    assert divergence is None, divergence
