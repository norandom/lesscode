"""Tests for regen_turn.py (no network, no endpoint)."""
from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import regen_turn as rt  # noqa: E402

FAKE_KEY = "sk-fake-abc123-never-real"


def _env_file(tmp_path: Path) -> Path:
    p = tmp_path / "model.env"
    p.write_text(
        "# endpoint for tests\n"
        "LESSCODE_MODEL_API_STYLE=anthropic   # provider style\n"
        "LESSCODE_MODEL_BASE_URL = https://example.invalid \n"
        "\n"
        "LESSCODE_MODEL_NAME=model-x # trailing comment\n"
        f"LESSCODE_MODEL_API_KEY={FAKE_KEY}\n",
        encoding="utf-8",
    )
    return p


def _scope(tmp_path: Path) -> rt.TurnRequest:
    root = tmp_path / "scope"
    (root / "src" / "pkg").mkdir(parents=True)
    (root / "src" / "pkg" / "a.py").write_text("def a():\n    return 1\n", encoding="utf-8")
    (root / "src" / "pkg" / "b.py").write_text("def b():\n    return 2\n", encoding="utf-8")
    surface = tmp_path / "surface.json"
    surface.write_text(json.dumps({"modules": {"pkg.a": {"names": {"a": {"kind": "function"}}}}}), encoding="utf-8")
    return rt.TurnRequest(
        scope_root=str(root), scope_files=["src/pkg/a.py", "src/pkg/b.py"], surface_path=str(surface),
        definitions="DEFS-TEXT", targets={"statements_max": 10}, contract="CONTRACT-TEXT",
        acceptance_command="python run_x.py <root>", feedback=None)


def test_load_endpoint_strips_comments_and_hides_key(tmp_path):
    ep = rt.load_endpoint(_env_file(tmp_path))
    assert (ep.style, ep.base_url, ep.model) == ("anthropic", "https://example.invalid", "model-x")
    assert ep.api_key() == FAKE_KEY
    assert FAKE_KEY not in repr(ep) and FAKE_KEY not in str(ep)
    assert FAKE_KEY not in json.dumps(ep.public())
    assert ep.host == "example.invalid"


def test_load_endpoint_requires_all_keys(tmp_path):
    p = tmp_path / "model.env"
    p.write_text("LESSCODE_MODEL_NAME=m\n", encoding="utf-8")
    with pytest.raises(ValueError):
        rt.load_endpoint(p)


def test_build_prompt_without_feedback(tmp_path):
    req = _scope(tmp_path)
    system, user = rt.build_prompt(req)
    assert "JSON" in system and "only" in system.lower()
    assert "src/pkg/a.py" in user and "def a():" in user and "def b():" in user
    assert '"pkg.a"' in user and "DEFS-TEXT" in user and "CONTRACT-TEXT" in user
    assert "python run_x.py <root>" in user and '"statements_max": 10' in user
    assert rt.FEEDBACK_HEADING not in user
    for label in rt.FEEDBACK_FIELDS:
        assert label not in user


def test_build_prompt_failed_feedback_has_exactly_two_fields(tmp_path):
    req = _scope(tmp_path)
    req.feedback = {"kind": "failed", "first_divergence": "seq7 op 3 result", "measure_gaps": {"lines": 12}}
    _, user = rt.build_prompt(req)
    assert rt.FEEDBACK_HEADING in user and "seq7 op 3 result" in user and '"lines": 12' in user
    present = [f for f in rt.FEEDBACK_FIELDS if f in user]
    assert present == [rt.FEEDBACK_FIELDS[0], rt.FEEDBACK_FIELDS[1]]


def test_build_prompt_accepted_scored_feedback_adds_two_fields(tmp_path):
    req = _scope(tmp_path)
    req.feedback = {"kind": "accepted_scored", "first_divergence": None, "measure_gaps": {},
                    "surviving_faults": [{"line": 5, "op": "x"}], "counterexamples": ["f(1) != g(1)"]}
    _, user = rt.build_prompt(req)
    present = [f for f in rt.FEEDBACK_FIELDS if f in user]
    assert present == list(rt.FEEDBACK_FIELDS)
    assert '"op": "x"' in user and "f(1) != g(1)" in user


def test_build_prompt_rejects_unknown_feedback(tmp_path):
    req = _scope(tmp_path)
    req.feedback = {"kind": "other"}
    with pytest.raises(ValueError):
        rt.build_prompt(req)
    req.feedback = {"kind": "failed", "first_divergence": "x", "measure_gaps": {}, "extra": 1}
    with pytest.raises(ValueError):
        rt.build_prompt(req)


def test_write_candidate_refuses_path_outside_scope_before_writing(tmp_path):
    req = _scope(tmp_path)
    cand = tmp_path / "cand"
    cand.mkdir()
    result = rt.TurnResult(files=[{"path": "src/pkg/a.py", "content": "def a():\n    return 9\n"},
                                  {"path": "src/pkg/evil.py", "content": "x = 1\n"}],
                           structure={"pure": [], "shell": []}, change_log="log", meta={})
    with pytest.raises(rt.ScopeError):
        rt.write_candidate(result, cand, req.scope_files)
    assert list(cand.rglob("*")) == []
    result.files.pop()
    written = rt.write_candidate(result, cand, req.scope_files)
    assert (cand / "src" / "pkg" / "a.py").read_text(encoding="utf-8") == "def a():\n    return 9\n"
    assert written[0]["path"] == "src/pkg/a.py" and len(written[0]["sha256"]) == 64


class _Usage:
    input_tokens, output_tokens, cache_creation_input_tokens, cache_read_input_tokens = 30, 7, 0, 0

    def model_dump(self):
        return {"input_tokens": 30, "output_tokens": 7}


class _Msg:
    model, id, _request_id, stop_reason, stop_details, usage = "model-x-20260101", "msg_1", "req_1", "end_turn", None, _Usage()

    def to_json(self):
        return '{"id": "msg_1", "content": [{"type": "text", "text": "{}"}]}'


def test_hash_set_over_fake_message():
    kwargs = {"model": "model-x", "max_tokens": 10, "system": "SYS", "messages": [{"role": "user", "content": "USR"}]}
    h = rt.hash_set(kwargs, _Msg())
    assert h["system_sha256"] == rt.sha256_text("SYS") and h["user_sha256"] == rt.sha256_text("USR")
    assert h["request_sha256"] == rt.sha256_text(json.dumps(kwargs, sort_keys=True, ensure_ascii=False))
    assert h["response_sha256"] == rt.sha256_text(_Msg().to_json())
    assert h["model_sent"] == "model-x" and h["model_returned"] == "model-x-20260101"
    assert h["seed"] is None and "no seed" in h["seed_note"]
    assert FAKE_KEY not in json.dumps(h)
