"""E1c-2 Arm B replay target: the E1c-2 sequences only (golden-e1c2.json) on Facdrone's `pg_engine`."""
import e1c2_source
def test_e1c2_replay(pg_engine) -> None:
    divergence = e1c2_source.O.replay_engine(pg_engine, e1c2_source.GOLDEN)
    assert divergence is None, divergence
