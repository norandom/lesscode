"""ProbabilisticRegenerator suite (constraint-guided-compression tasks 5.1-5.5).

Offline tests with fake endpoints and injected evaluators: turn contracts and
endpoints, receipt/bindings provenance, the Tier-1 ladder and adaptive
schedule, the Tier-2 pinned formula and feedback boundary, and the
Refine/BestOfN controller with its stops and outcomes.
"""
from __future__ import annotations

import json
from pathlib import Path

import pytest

from lesscode_baseline import regenerator as rg


@pytest.fixture()
def frozen_tree(tmp_path: Path) -> Path:
    (tmp_path / "src").mkdir()
    (tmp_path / "src" / "mod.py").write_text("def f():\n    return 1\n", encoding="utf-8")
    (tmp_path / "public-surface.json").write_text('{"modules": {}}', encoding="utf-8")
    return tmp_path


@pytest.fixture()
def request_path(frozen_tree: Path) -> Path:
    req = {
        "scope_root": str(frozen_tree),
        "scope_files": ["src/mod.py"],
        "surface_path": str(frozen_tree / "public-surface.json"),
        "definitions": "module definitions prose",
        "targets": {"written_stmts_max": 50},
        "contract": "all functions declared",
        "acceptance_command": "python -m pytest tests -q",
        "language": "python",
    }
    path = frozen_tree / "request.json"
    path.write_text(json.dumps(req), encoding="utf-8")
    return path


class TestTurnRequestLanguage:
    def test_language_defaults_to_python(self, frozen_tree: Path) -> None:
        req = rg.TurnRequest(
            scope_root=str(frozen_tree), scope_files=["src/mod.py"], surface_path="x",
            definitions="", targets={}, contract="", acceptance_command="",
        )
        assert req.language == "python"

    def test_load_roundtrip(self, request_path: Path) -> None:
        assert rg.TurnRequest.load(request_path).language == "python"


class TestBuildPromptLanguageSeam:
    def test_python_fences_and_labels(self, request_path: Path) -> None:
        system, user = rg.build_prompt(rg.TurnRequest.load(request_path))
        assert "```python\n" in user
        assert "```json\n" in user
        assert "python" in system.lower()

    def test_other_language_drives_fence(self, request_path: Path, frozen_tree: Path) -> None:
        req = rg.TurnRequest.load(request_path)
        req.language = "rust"
        _, user = rg.build_prompt(req)
        assert "```rust\n" in user
        assert "```python\n" not in user

    def test_feedback_section_included(self, request_path: Path) -> None:
        req = rg.TurnRequest.load(request_path)
        req.feedback = {"kind": "failed", "first_divergence": "static:missing_surface", "measure_gaps": {}}
        _, user = rg.build_prompt(req)
        assert "## Feedback from the previous round" in user

    def test_feedback_kind_and_keys_validated(self, request_path: Path) -> None:
        req = rg.TurnRequest.load(request_path)
        req.feedback = {"kind": "bogus"}
        with pytest.raises(ValueError, match="feedback"):
            rg.build_prompt(req)


class TestEndpoint:
    def test_key_redacted_everywhere(self) -> None:
        ep = rg.Endpoint("anthropic", "https://api.example.com/v1", "model-x", "sk-secret")
        assert "sk-secret" not in repr(ep)
        assert "sk-secret" not in str(ep)
        assert "sk-secret" not in json.dumps(ep.public())
        assert ep.public() == {
            "style": "anthropic",
            "host": "api.example.com",
            "model": "model-x",
            "model_sha256": rg.sha256_text("model-x"),
        }

    def test_load_endpoint_parses_env_file(self, tmp_path: Path) -> None:
        env = tmp_path / "model.env"
        env.write_text(
            "LESSCODE_MODEL_API_STYLE=anthropic\n"
            "LESSCODE_MODEL_BASE_URL=https://api.example.com/v1\n"
            "LESSCODE_MODEL_NAME=model-x\n"
            "LESSCODE_MODEL_API_KEY=sk-secret # trailing comment\n",
            encoding="utf-8",
        )
        ep = rg.load_endpoint(env)
        assert ep.host == "api.example.com"
        assert ep.api_key() == "sk-secret"

    def test_load_endpoint_missing_keys(self, tmp_path: Path) -> None:
        env = tmp_path / "model.env"
        env.write_text("LESSCODE_MODEL_API_STYLE=anthropic\n", encoding="utf-8")
        with pytest.raises(ValueError, match="missing"):
            rg.load_endpoint(env)


class TestCandidatePreparation:
    def test_fresh_copy_and_hash_check(self, frozen_tree: Path, tmp_path: Path) -> None:
        cand = tmp_path / "cand"
        hashes = rg.prepare_candidate(frozen_tree, cand, ["src/mod.py"])
        assert (cand / "src" / "mod.py").read_text(encoding="utf-8") == "def f():\n    return 1\n"
        assert set(hashes) == {"src/mod.py"}

    def test_copy_mismatch_raises(self, frozen_tree: Path, tmp_path: Path) -> None:
        cand = tmp_path / "cand"
        rg.prepare_candidate(frozen_tree, cand, ["src/mod.py"])
        (cand / "src" / "mod.py").write_text("tampered\n", encoding="utf-8")
        with pytest.raises(rg.CopyMismatch):
            rg.prepare_candidate(frozen_tree, cand, ["src/mod.py"])


class TestWriteCandidate:
    def test_out_of_scope_refused_before_writing(self, frozen_tree: Path, tmp_path: Path) -> None:
        cand = tmp_path / "cand"
        rg.prepare_candidate(frozen_tree, cand, ["src/mod.py"])
        result = rg.TurnResult(
            files=[{"path": "src/other.py", "content": "x = 1\n"}, {"path": "src/mod.py", "content": "y = 2\n"}],
            structure={"pure": [], "shell": []},
            change_log="",
            meta={},
        )
        with pytest.raises(rg.ScopeError, match="src/other.py"):
            rg.write_candidate(result, cand, ["src/mod.py"])
        assert not (cand / "src" / "other.py").exists()
        assert (cand / "src" / "mod.py").read_text(encoding="utf-8") == "def f():\n    return 1\n"

    def test_write_records_hashes_and_crlf_convention(self, frozen_tree: Path, tmp_path: Path) -> None:
        cand = tmp_path / "cand"
        rg.prepare_candidate(frozen_tree, cand, ["src/mod.py"])
        (cand / "src" / "mod.py").write_bytes(b"def f():\r\n    return 1\r\n")
        result = rg.TurnResult(
            files=[{"path": "src/mod.py", "content": "def g():\r\n    return 2\r\n"}],
            structure={"pure": [], "shell": []},
            change_log="",
            meta={},
        )
        written = rg.write_candidate(result, cand, ["src/mod.py"])
        assert written[0]["crlf"] is True
        assert (cand / "src" / "mod.py").read_bytes() == b"def g():\r\n    return 2\r\n"


class TestReceiptWriter:
    def test_fact_and_note_structure(self, tmp_path: Path) -> None:
        receipt = tmp_path / "receipt.json"
        writer = rg.ReceiptWriter(receipt)
        writer.add_fact({"key": "inputs", "value": 1})
        writer.add_note("turn-1", {"score": 0.5})
        data = json.loads(receipt.read_text(encoding="utf-8"))
        assert data["facts"][0]["key"] == "inputs"
        assert data["notes"][0]["key"] == "turn-1"
        assert data["notes"][0]["time_utc"]

    def test_labels_case_insensitively_unique(self, tmp_path: Path) -> None:
        writer = rg.ReceiptWriter(tmp_path / "receipt.json")
        writer.add_note("Turn-1", {"a": 1})
        with pytest.raises(rg.DuplicateReceiptLabelError):
            writer.add_note("tURN-1", {"b": 2})

    def test_sidecars_for_admission(self, tmp_path: Path) -> None:
        receipt = tmp_path / "receipt.json"
        writer = rg.ReceiptWriter(receipt)
        writer.add_note("turn-1", {"score": 1.0})
        digest = rg.sha256_text(receipt.read_text(encoding="utf-8"))
        sidecar = (tmp_path / "receipt.json.sha256").read_text(encoding="utf-8")
        assert sidecar == f"{digest}  receipt.json\n"
        bindings = json.loads((tmp_path / "bindings.json").read_text(encoding="utf-8"))
        assert bindings["bindings"][0]["artifact"] == "receipt.json"
        assert bindings["bindings"][0]["sidecars"] == ["receipt.json.sha256"]

    def test_binding_artifacts_cover_saved_turn_files(self, tmp_path: Path) -> None:
        receipt = tmp_path / "receipt.json"
        writer = rg.ReceiptWriter(receipt)
        turn_dir = tmp_path / "turns" / "L1"
        rg._save(turn_dir, "system.txt", "sys")
        rg._save(turn_dir, "user.txt", "usr")
        writer.add_bound_artifact(turn_dir / "system.txt")
        writer.add_bound_artifact(turn_dir / "user.txt")
        bindings = json.loads((tmp_path / "bindings.json").read_text(encoding="utf-8"))
        artifacts = {b["artifact"] for b in bindings["bindings"]}
        assert artifacts == {"receipt.json", "turns/L1/system.txt", "turns/L1/user.txt"}


class TestSaveSidecars:
    def test_disk_bytes_equal_hashed_text(self, tmp_path: Path) -> None:
        digest = rg._save(tmp_path, "note.txt", "line\nother\n")
        raw = (tmp_path / "note.txt").read_bytes()
        assert raw == b"line\nother\n"  # R26 amendment 3: bytes on disk == hashed text
        assert digest == rg.sha256_text("line\nother\n")
        assert (tmp_path / "note.txt.sha256").read_text(encoding="utf-8") == f"{digest}  note.txt\n"


@pytest.fixture()
def surface_report(frozen_tree: Path) -> dict:
    from lesscode_baseline import surface

    (frozen_tree / "src" / "user.py").write_text("from mod import f\n", encoding="utf-8")
    return surface.derive_surface(frozen_tree, ["src/mod.py"])


def _outcome(**kw) -> dict:
    base = {"ok": True, "result": None, "written": []}
    base.update(kw)
    return base


class TestTier1Ladder:
    def _gate(self, frozen_tree: Path, request_path: Path, surface_report: dict, outcome: dict, cand: Path,
              targets: dict | None = None) -> rg.RewardEvaluation:
        req = rg.TurnRequest.load(request_path)
        return rg.evaluate_tier1_static(
            req, outcome, cand if cand else frozen_tree, surface_report, targets or {}
        )

    def test_transport_rung(self, frozen_tree, request_path, surface_report, tmp_path):
        reward = self._gate(frozen_tree, request_path, surface_report,
                            _outcome(ok=False, kind="transport", detail="boom"), None)
        assert (reward.score, reward.tier, reward.failure_kind) == (0.00, 1, "transport")

    def test_stop_reason_rung(self, frozen_tree, request_path, surface_report):
        reward = self._gate(frozen_tree, request_path, surface_report,
                            _outcome(ok=False, kind="stop_reason", detail="max_tokens"), None)
        assert (reward.score, reward.failure_kind) == (0.00, "stop_reason")

    def test_invalid_json_rung(self, frozen_tree, request_path, surface_report):
        reward = self._gate(frozen_tree, request_path, surface_report,
                            _outcome(ok=False, kind="invalid_json", detail="{"), None)
        assert (reward.score, reward.failure_kind) == (0.10, "invalid_json")

    def test_out_of_scope_rung(self, frozen_tree, request_path, surface_report):
        reward = self._gate(frozen_tree, request_path, surface_report,
                            _outcome(ok=False, kind="out_of_scope", detail="src/other.py"), None)
        assert (reward.score, reward.failure_kind) == (0.20, "out_of_scope")

    def test_syntax_rung_uses_check_syntax(self, frozen_tree, request_path, surface_report, tmp_path):
        cand = tmp_path / "cand"
        rg.prepare_candidate(frozen_tree, cand, ["src/mod.py"])
        (cand / "src" / "mod.py").write_text("def f(:\n", encoding="utf-8")
        reward = self._gate(frozen_tree, request_path, surface_report, _outcome(), cand)
        assert (reward.score, reward.failure_kind) == (0.30, "syntax_error")
        assert "src/mod.py:1:" in reward.diagnostics["errors"][0]

    def test_surface_rung_reports_missing_symbol(self, frozen_tree, request_path, surface_report, tmp_path):
        cand = tmp_path / "cand"
        rg.prepare_candidate(frozen_tree, cand, ["src/mod.py"])
        (cand / "src" / "mod.py").write_text("def g():\n    return 2\n", encoding="utf-8")
        reward = self._gate(frozen_tree, request_path, surface_report, _outcome(), cand)
        assert (reward.score, reward.failure_kind) == (0.40, "surface")
        assert reward.diagnostics["missing_symbols"] == [("mod", "f")]

    def test_structure_rung_reports_violation(self, frozen_tree, request_path, surface_report, tmp_path):
        cand = tmp_path / "cand"
        rg.prepare_candidate(frozen_tree, cand, ["src/mod.py"])
        (cand / "src" / "mod.py").write_text("def f(session):\n    return 1\n", encoding="utf-8")
        result = rg.TurnResult(files=[], structure={"pure": ["mod.py::f"], "shell": []}, change_log="", meta={})
        reward = self._gate(frozen_tree, request_path, surface_report, _outcome(result=result), cand)
        assert (reward.score, reward.failure_kind) == (0.45, "structure")
        assert any("param session" in w for w in reward.diagnostics["why"])

    def test_static_targets_rung(self, frozen_tree, request_path, surface_report, tmp_path):
        cand = tmp_path / "cand"
        rg.prepare_candidate(frozen_tree, cand, ["src/mod.py"])
        (cand / "src" / "mod.py").write_text("def f():\n    return 1\n\n\ndef h():\n    return 2\n", encoding="utf-8")
        reward = self._gate(frozen_tree, request_path, surface_report, _outcome(), cand,
                            targets={"funcs_max": 1})
        assert (reward.score, reward.failure_kind) == (0.49, "static_targets")
        assert reward.diagnostics["gaps"]["funcs_max"]["excess"] == 1

    def test_pass_rung(self, frozen_tree, request_path, surface_report, tmp_path):
        cand = tmp_path / "cand"
        rg.prepare_candidate(frozen_tree, cand, ["src/mod.py"])
        (cand / "src" / "mod.py").write_text("def f():\n    return 1\n", encoding="utf-8")
        result = rg.TurnResult(files=[], structure={"pure": ["mod.py::f"], "shell": []}, change_log="", meta={})
        reward = self._gate(frozen_tree, request_path, surface_report, _outcome(result=result), cand,
                            targets={"funcs_max": 5})
        assert reward.score == 0.50
        assert reward.passed_tier1 is True
        assert reward.failure_kind is None

    def test_ladder_is_fast_and_offline(self, frozen_tree, request_path, surface_report, tmp_path):
        import time

        cand = tmp_path / "cand"
        rg.prepare_candidate(frozen_tree, cand, ["src/mod.py"])
        t0 = time.perf_counter()
        self._gate(frozen_tree, request_path, surface_report, _outcome(), cand)
        assert time.perf_counter() - t0 < 2.0


class TestAdaptiveSchedule:
    def test_thinking_budget_recovery(self):
        prev = rg.RewardEvaluation(score=0.00, tier=1, passed_tier1=False, failure_kind="stop_reason",
                                   diagnostics={})
        params = rg.next_rollout_params(prev, attempt=0)
        assert params["effort"] == "medium"
        assert "concise" in params["directive"].lower()

    def test_temperature_steps_bounded(self):
        prev = rg.RewardEvaluation(score=0.30, tier=1, passed_tier1=False, failure_kind="syntax_error",
                                   diagnostics={})
        assert [rg.next_rollout_params(prev, attempt=a)["temperature"] for a in (0, 1, 2, 3)] == [0.0, 0.2, 0.4, 0.4]

    def test_missing_symbol_injection_hint(self):
        prev = rg.RewardEvaluation(score=0.40, tier=1, passed_tier1=False, failure_kind="surface",
                                   diagnostics={"missing_symbols": [("mod", "f")]})
        params = rg.next_rollout_params(prev, attempt=1)
        assert params["inject_missing_symbols"] == ["f"]

    def test_pass_needs_no_adaptation(self):
        prev = rg.RewardEvaluation(score=0.50, tier=1, passed_tier1=True, failure_kind=None, diagnostics={})
        assert rg.next_rollout_params(prev, attempt=0) == {}


class TestTier2Runtime:
    def test_infrastructure_timeout_is_never_a_kill(self):
        reward = rg.evaluate_tier2_runtime(124, None, {})
        assert (reward.score, reward.failure_kind) == (0.50, "runtime_infrastructure")
        reward = rg.evaluate_tier2_runtime(2, None, {})
        assert (reward.score, reward.failure_kind) == (0.50, "runtime_infrastructure")

    def test_formula_on_synthetic_fractions(self):
        cases = [
            (1.0, 0.0, 0.73),   # 0.60 + 0.25*(0.5)
            (0.8, 0.6, 0.78),   # 0.60 + 0.25*(0.7)
            (1.0, 1.0, 0.85),
            (0.0, 0.0, 0.60),
            (0.5, 0.5, 0.73),   # 0.60+0.25*0.5 = 0.725 -> half-up 0.73
        ]
        for g, s, expected in cases:
            reward = rg.evaluate_tier2_runtime(1, "seq::first", {
                "seed_group_pass_fraction": g, "sequence_pass_fraction": s})
            assert reward.score == expected, (g, s)
            assert reward.tier == 2
            assert reward.diagnostics["first_divergence"] == "seq::first"

    def test_missing_fraction_key_scores_zero_fail_closed(self):
        reward = rg.evaluate_tier2_runtime(1, None, {"sequence_pass_fraction": 1.0})
        assert reward.score == 0.73  # 0.60 + 0.25*(0.5*0 + 0.5*1.0) = 0.725, half-up to 0.73

    def test_accepted_with_gaps(self):
        reward = rg.evaluate_tier2_runtime(0, None, {"measure_gaps": {"stmts_max": {}}})
        assert (reward.score, reward.failure_kind) == (0.90, "gaps")

    def test_accepted_with_scored_faults(self):
        reward = rg.evaluate_tier2_runtime(0, None, {"surviving_faults": [{"line": 3}]})
        assert (reward.score, reward.failure_kind) == (0.95, "scored_faults")

    def test_full_acceptance(self):
        reward = rg.evaluate_tier2_runtime(0, None, {})
        assert reward.score == 1.00
        assert reward.failure_kind is None

    def test_unknown_exit_is_infrastructure_fail_closed(self):
        reward = rg.evaluate_tier2_runtime(7, None, {})
        assert (reward.score, reward.failure_kind) == (0.50, "runtime_infrastructure")


class TestFeedbackBoundary:
    def test_held_out_name_rejected_before_dispatch(self):
        with pytest.raises(rg.FeedbackBoundaryError, match="held"):
            rg.enforce_feedback_boundary(
                {"kind": "failed", "first_divergence": "diff in seq HOLDOUT-7::op", "measure_gaps": {}},
                held_out_names=("HOLDOUT-7", "HOLDOUT-12"),
            )

    def test_normal_divergence_passes(self):
        rg.enforce_feedback_boundary(
            {"kind": "failed", "first_divergence": "static:missing_surface:mod::f", "measure_gaps": {}},
            held_out_names=("HOLDOUT-7",),
        )

    def test_accepted_scored_payload_checked_too(self):
        with pytest.raises(rg.FeedbackBoundaryError):
            rg.enforce_feedback_boundary(
                {"kind": "accepted_scored", "first_divergence": None, "measure_gaps": {},
                 "surviving_faults": [], "counterexamples": ["replay of HOLDOUT-12 differs"]},
                held_out_names=("HOLDOUT-7", "HOLDOUT-12"),
            )
