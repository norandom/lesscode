"""Proposed specimen pins; admission requires unchanged-source and adverse runs."""

from types import SimpleNamespace

import pytest

from x_likes_exporter.exporter import XLikesExporter


def tweet(value):
    return SimpleNamespace(id=value)


def harness(resumed, fresh, fail_at=None):
    events = []
    exporter = object.__new__(XLikesExporter)
    exporter.tweets = [tweet("prior")]
    checkpoint = SimpleNamespace(
        exists=lambda: True,
        load=lambda: {"user_id": "A", "tweets": resumed, "cursor": "saved"},
        clear=lambda: events.append(("clear", list(exporter.tweets))),
    )
    exporter.checkpoint = checkpoint

    def save(user, values, cursor, media):
        events.append(("save", user, list(values), cursor, media))
        if fail_at == "save":
            raise RuntimeError("save failed")

    def fetch(**kwargs):
        events.append(("fetch", kwargs["start_cursor"], exporter.tweets))
        kwargs["checkpoint_callback"](fresh, "next")
        if fail_at == "fetch":
            raise RuntimeError("fetch failed")
        return fresh

    def download(values, **kwargs):
        events.append(("download", values))
        if fail_at == "download":
            raise RuntimeError("download failed")
        return 0

    exporter._save_checkpoint = save
    exporter.api_client = SimpleNamespace(fetch_all_likes=fetch)
    exporter.media_downloader = SimpleNamespace(download_all_media=download)
    return exporter, events


def test_resume_alias_and_callback_order(capsys):
    old = [tweet("1")]
    fresh = [tweet("1"), tweet("2"), tweet("2")]
    exporter, events = harness(old, fresh)
    result = exporter.fetch_likes("A", resume=True, download_media=True)
    assert result is old is exporter.tweets
    # The original deduplicates against old IDs, not within the fresh batch.
    assert [t.id for t in result] == ["1", "2", "2"]
    assert events[0][0:2] == ("fetch", "saved")
    assert events[0][2] is old
    assert events[1][0:2] == ("save", "A")
    assert [t.id for t in events[1][2]] == ["1", "1", "2", "2"]
    assert [e[0] for e in events] == ["fetch", "save", "clear", "download"]
    assert events[-1][1] is result
    assert capsys.readouterr().out == (
        "✓ Resuming from checkpoint: 1 tweets already fetched\n"
        "  Starting from cursor: saved...\n"
        "✓ Added 2 new tweets (total: 3)\n"
        "\nDownloading media from 3 tweets...\n\nDownloaded 0 media files\n"
    )


@pytest.mark.parametrize("failure", ["save", "fetch", "download"])
def test_failure_retains_the_observable_state(failure):
    old = [tweet("1")]
    fresh = [tweet("2")]
    exporter, events = harness(old, fresh, failure)
    with pytest.raises(RuntimeError, match=f"{failure} failed"):
        exporter.fetch_likes("A", resume=True, download_media=True)
    assert exporter.tweets is old
    expected = ["1", "2"] if failure == "download" else ["1"]
    assert [t.id for t in exporter.tweets] == expected
    assert any(e[0] == "clear" for e in events) == (failure == "download")


def test_empty_resume_replaces_list_and_stop_callback_prevents_clear():
    old = []
    fresh = [tweet("2")]
    exporter, events = harness(old, fresh)
    result = exporter.fetch_likes(
        "A", resume=True, download_media=False, stop_callback=lambda: False
    )
    assert result is fresh is exporter.tweets
    assert result is not old
    assert not any(e[0] == "clear" for e in events)
