"""E1c Arm B replay target: the E1c sequences only (golden-e1c.json) on Facdrone's `pg_engine`."""
import e1c_source


def test_e1c_replay(pg_engine) -> None:
    divergence = e1c_source.O.replay_engine(pg_engine, e1c_source.GOLDEN)
    assert divergence is None, divergence
