"""Replay only the named E1c sequences (R4 split into preregistered and supplementary inputs)."""
import sys
import e1c_source as E

with E.O.embedded_engine() as engine:
    print(E.O.replay_engine(engine, E.GOLDEN, set(sys.argv[1].split(","))) or "replay: no divergence")
