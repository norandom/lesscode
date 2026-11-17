"""Additional fixed observations; external components remain deterministic fakes."""

from pathlib import Path
import os
from types import SimpleNamespace

import pytest

from x_likes_exporter import exporter as module
from x_likes_mcp import tools
from x_likes_mcp.errors import ToolError
from test_state_observations import harness, tweet


def test_imports_are_from_the_isolated_tree():
    assert os.geteuid() != 0
    assert Path(module.__file__).resolve() == Path("/case/x_likes_exporter/exporter.py")
    assert Path(tools.__file__).resolve() == Path("/case/x_likes_mcp/tools.py")
    assert not os.access("/case/ms1_pins/test_state_observations.py", os.W_OK)
    assert not os.access("/case/x_likes_mcp/tools.py", os.W_OK)


@pytest.mark.parametrize("checkpoints", [True, False])
def test_construction_retains_empty_state_and_component_options(monkeypatch, tmp_path, checkpoints):
    events = []
    cookie = SimpleNamespace(validate=lambda: events.append("validate") or True)
    monkeypatch.setattr(
        module, "CookieManager", lambda value: events.append(("cookies", value)) or cookie
    )
    monkeypatch.setattr(
        module, "XAPIClient", lambda value: events.append(("client", value)) or "client"
    )
    monkeypatch.setattr(
        module, "MediaDownloader", lambda value: events.append(("media", value)) or "media"
    )
    monkeypatch.setattr(
        module, "Checkpoint", lambda value: events.append(("checkpoint", value)) or "checkpoint"
    )
    result = module.XLikesExporter("fake-cookie-path", str(tmp_path), checkpoints)
    assert result.tweets == []
    assert result.api_client == "client"
    assert result.media_downloader == "media"
    assert result.checkpoint == ("checkpoint" if checkpoints else None)
    assert events[:3] == [
        ("cookies", "fake-cookie-path"),
        ("client", cookie),
        ("media", str(tmp_path / "media")),
    ]
    assert events[-1] == "validate"


@pytest.mark.parametrize("resume", [True, False])
@pytest.mark.parametrize("fresh", [[], [tweet("2")]])
def test_resume_disabled_or_different_user_preserves_baseline_merge(resume, fresh, capsys):
    exporter, events = harness([tweet("saved")], fresh)
    prior = exporter.tweets
    exporter.checkpoint.load = lambda: {"user_id": "OTHER", "tweets": [], "cursor": "wrong"}
    result = exporter.fetch_likes("A", resume=resume, download_media=False)
    assert events[1 if resume else 0][0:2] == ("fetch", None)
    if resume:
        assert result is prior
        assert [t.id for t in result] == ["prior"] + [t.id for t in fresh]
        assert "different user" in capsys.readouterr().out
        assert [e[0] for e in events] == ["clear", "fetch", "save", "clear"]
    else:
        assert result is fresh
        assert [e[0] for e in events] == ["fetch", "save", "clear"]


@pytest.mark.parametrize("boundary", ["exists", "load", "clear"])
def test_checkpoint_failure_retains_preexisting_list(boundary):
    exporter, events = harness([tweet("saved")], [tweet("fresh")])
    prior = exporter.tweets

    def fail():
        raise RuntimeError(boundary)

    if boundary == "clear":
        exporter.checkpoint.load = lambda: {"user_id": "OTHER"}
    setattr(exporter.checkpoint, boundary, fail)
    with pytest.raises(RuntimeError, match=boundary):
        exporter.fetch_likes("A", resume=True, download_media=False)
    assert exporter.tweets is prior
    assert events == []


def test_progress_and_stop_callbacks_are_forwarded_by_identity():
    exporter, events = harness([], [])
    progress = lambda current, total: None
    stop = lambda: True

    def fetch(**kwargs):
        assert kwargs["progress_callback"] is progress
        assert kwargs["stop_callback"] is stop
        assert kwargs["user_id"] == "A"
        assert kwargs["start_cursor"] is None
        return []

    exporter.api_client.fetch_all_likes = fetch
    exporter.fetch_likes("A", False, progress, stop, False)
    assert events == []


def test_custom_integer_comparisons_preserve_short_circuit_order():
    seen = []

    class Year(int):
        def __lt__(self, other):
            seen.append(("lt", other))
            return False

        def __gt__(self, other):
            seen.append(("gt", other))
            return False

        def __bool__(self):
            raise AssertionError("year truthiness is not part of the contract")

    assert tools._validate_filter(Year(2026), "01", "02") is None
    assert seen == [("lt", tools._YEAR_MIN), ("gt", tools._YEAR_MAX)]


def test_custom_string_truthiness_is_not_validation():
    class Month(str):
        def __bool__(self):
            raise AssertionError("month truthiness is not part of the contract")

    assert tools._validate_filter(2026, Month("01"), Month("02")) is None


@pytest.mark.parametrize("value", [True, False, 0, [], {}, object()])
def test_nonstring_months_are_rejected(value):
    with pytest.raises(ToolError, match="month_start must match"):
        tools._validate_filter(2026, value, None)


@pytest.mark.parametrize("fail", [False, True])
def test_real_checkpoint_callback_arguments_and_failure(fail):
    exporter = object.__new__(module.XLikesExporter)
    retained = [tweet("old")]
    values = [tweet("new")]
    exporter.tweets = retained
    calls = []

    def save(**kwargs):
        calls.append(kwargs)
        assert exporter.tweets is retained
        if fail:
            raise RuntimeError("checkpoint storage failed")

    exporter.checkpoint = SimpleNamespace(save=save)
    if fail:
        with pytest.raises(RuntimeError, match="checkpoint storage failed"):
            exporter._save_checkpoint("A", values, "cursor", True)
    else:
        assert exporter._save_checkpoint("A", values, "cursor", True) is None
    assert calls == [
        dict(user_id="A", tweets=values, cursor="cursor", total_fetched=1, download_media=True)
    ]
    assert calls[0]["tweets"] is values
    assert exporter.tweets is retained


def test_absent_checkpoint_does_not_save():
    exporter = object.__new__(module.XLikesExporter)
    exporter.checkpoint = None
    assert exporter._save_checkpoint("A", [tweet("new")], "cursor", False) is None


def test_public_search_forwards_custom_filter_objects_to_the_resolver():
    class Year(int):
        def __bool__(self):
            raise AssertionError("unexpected year truthiness")

    class Month(str):
        def __bool__(self):
            raise AssertionError("unexpected month truthiness")

    year, start, end = Year(2026), Month("01"), Month("12\n")
    calls = []
    index = SimpleNamespace(search=lambda *args: calls.append(args) or [])
    assert tools.search_likes(index, " query ", year, start, end) == []
    assert len(calls) == 1 and calls[0][0] == "query"
    assert all(actual is expected for actual, expected in zip(calls[0][1:], (year, start, end)))
    with pytest.raises(ToolError, match="year must be an integer"):
        tools.search_likes(index, "query", True, "bad", "bad")
    assert len(calls) == 1
