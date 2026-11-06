"""Apparatus tests for e4_oracle.py (no database): canonicalisation determinism, the declared orderings and the
driver's clock injection. Written before the oracle (red), then made green."""
import json
import sys
from datetime import UTC, datetime, timedelta
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import e4_oracle as O  # noqa: E402


def test_canon_is_key_sorted_and_repeatable():
    a = {"b": [1, {"y": 2, "x": 1}], "a": None}
    b = {"a": None, "b": [1, {"x": 1, "y": 2}]}
    assert O.canon(a) == O.canon(b) == '{"a":null,"b":[1,{"x":1,"y":2}]}'


def test_norm_maps_datetimes_to_first_seen_ordinals_and_dates_verbatim():
    n = O.Norm()
    t0, t1 = datetime(2026, 9, 7, 12, tzinfo=UTC), datetime(2026, 9, 8, 12, tzinfo=UTC)
    assert n([t1, t0, t1]) == ["<t0>", "<t1>", "<t0>"]
    assert n(t0.date()) == "2026-09-07"


def test_rows_are_ordered_by_canonical_json_whatever_the_input_order():
    rows = [{"k": 2, "v": "b"}, {"k": 1, "v": "a"}, {"k": 1, "v": "0"}]
    assert O.ordered(rows) == O.ordered(list(reversed(rows)))
    assert [r["v"] for r in O.ordered(rows)] == ["0", "a", "b"]


def test_events_order_is_moment_then_revision_then_id():
    class E:
        def __init__(self, at, rev, eid):
            self.event_at, self.revision, self.event_id = at, rev, eid
    t0, t1 = datetime(2026, 9, 7, 12, tzinfo=UTC), datetime(2026, 9, 8, 12, tzinfo=UTC)
    es = [E(t1, 1, "a"), E(t0, 2, "b"), E(t0, 1, "z"), E(t0, 1, "c")]
    assert [e.event_id for e in O.ordered_events(es)] == ["c", "z", "b", "a"]


def test_emitted_records_are_grouped_per_signal_in_emission_order():
    recs = [{"event": "lifecycle.reading", "signal": "cot", "version": 1, "window": "w1"},
            {"event": "lifecycle.reading", "signal": "carry", "version": 3, "window": "w1"},
            {"event": "lifecycle.flag", "signal": "cot", "version": 1}]
    g = O.grouped(recs)
    assert list(g) == ["carry@3", "cot@1"]
    assert [r["event"] for r in g["cot@1"]] == ["lifecycle.reading", "lifecycle.flag"]
    assert O.canon(O.grouped(recs)) == O.canon(O.grouped([recs[1], recs[0], recs[2]]))


def test_the_driver_clock_is_injected_and_only_tick_moves_it():
    from facdrone.shells.lifecycle_store import LifecycleStore
    O.set_clock(0)
    store = LifecycleStore(engine=None, now=O.clock_now)
    assert store._now() == O.AS_OF  # noqa: SLF001
    assert store._now() == O.AS_OF, "two reads at one moment are one moment"  # noqa: SLF001
    O.set_clock(4)
    assert store._now() == O.AS_OF + timedelta(days=4)  # noqa: SLF001
    assert O._Clocked(engine=None)._now() == O.AS_OF + timedelta(days=4), "a store built without a clock gets the driver's"  # noqa: SLF001
    O.set_clock(0)


def test_seed_sequences_are_well_formed_and_named_uniquely():
    names = [n for n, _ in O.SEEDS]
    assert len(names) == len(set(names)) and len(names) >= 20
    for _, ops in O.SEEDS:
        for o in ops:
            assert hasattr(O.Env, "op_" + o["op"]), o
            json.dumps(o)
