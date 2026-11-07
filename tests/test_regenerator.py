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
