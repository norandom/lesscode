"""RecordingCore regression suite (deconvolution-native-core task 3.2).

Unit tests are host-independent and run under the task venv (.venv-task11).
The three-stratum canonical line-count check reproduces the frozen
s10_measure2.py method (ruff==0.17.3 format --line-length 100 canonical
copies; rustfmt'd Rust; registration hooks at their design-frozen
allowances). Specimen integration checks run against the pinned specimen
venv (.runs/e2-facdrone/venv) and the frozen .runs/native-prototype
recordings, and skip with an explicit reason when those are absent —
absence stays absent, never zero.
"""
import hashlib
import json
import os
import subprocess
import sys
import time
from dataclasses import dataclass
from datetime import date, datetime, timezone
from enum import Enum
from pathlib import Path

import pytest

from lesscode_baseline import recording as P

REPO = Path(__file__).resolve().parents[1]
PROTO = REPO / "docs/experiments/deconvolution/native-prototype"
NATIVE = REPO / "native"
SPECIMEN = REPO / ".runs/e2-facdrone/venv/Scripts/python.exe"
PROTO2 = REPO / ".runs/native-prototype"

FROZEN_E3_LF_BODY = "92a48f77f66f2c9d0d0ebaf6e9e4b37f747816d2831dd577b72e183b7bf07671"
FROZEN_E4_LF_BODY = "0b2e383dd6fbbc243215e24a7844674404331308b3a547239594461c5bea5584"


def specimen_env() -> dict:
    return {**os.environ, "PYTHONDONTWRITEBYTECODE": "1"}


# --- verdict() truth table (design.md Testing Strategy; recording.verdict) ---


def test_verdict_truth_table():
    accepted = P.verdict(0, True, 0, None)
    assert accepted == (0, [])
    stale = P.verdict(0, False, 0, None)
    assert stale[0] == 1 and any("fresh" in r for r in stale[1])
    unbacked_l2 = P.verdict(0, True, 3, None)
    assert unbacked_l2[0] == 1 and any("L2" in r for r in unbacked_l2[1])
    invalid_arm = P.verdict(2, True, 0, None)
    assert invalid_arm == (2, ["arm exit 2"])
    refused_arm = P.verdict(1, True, 0, None)
    assert refused_arm[0] == 1 and refused_arm[1] == ["arm exit 1"]


def test_verdict_admits_l2_with_bound_query_evidence(tmp_path):
    evidence = tmp_path / "query-differential.json"
    evidence.write_text("{}", encoding="utf-8")
    assert P.verdict(0, True, 3, evidence) == (0, [])
    missing_evidence = tmp_path / "absent-evidence.json"
    code, reasons = P.verdict(0, True, 3, missing_evidence)
    assert code == 1 and any("L2" in r for r in reasons)


# --- call() deadline enforcement (taskkill tree kill is Windows-only by design) ---


def test_call_returns_arm_exit_without_deadline():
    assert P.call([sys.executable, "-c", "raise SystemExit(7)"], None) == 7


def test_call_returns_arm_exit_before_deadline():
    deadline = time.monotonic() + 30
    assert P.call([sys.executable, "-c", "raise SystemExit(5)"], deadline) == 5


@pytest.mark.skipif(sys.platform != "win32", reason="tree kill uses taskkill (Windows design)")
def test_call_deadline_expiry_returns_124_not_a_verdict():
    t0 = time.monotonic()
    code = P.call([sys.executable, "-c", "import time; time.sleep(30)"], t0 + 1)
    assert code == 124
    assert time.monotonic() - t0 < 20


# --- Norm: declared deterministic normalization ---


def test_norm_datetimes_become_first_seen_ordinals():
    norm = P.Norm()
    a = datetime(2026, 11, 7, 12, 0, 0, tzinfo=timezone.utc)
    b = datetime(2026, 11, 7, 13, 0, 0, tzinfo=timezone.utc)
    assert norm(a) == "<t0>" and norm(b) == "<t1>"
    assert norm(a) == "<t0>"  # first-seen identity is stable across the record


def test_norm_dates_enums_and_scalars():
    norm = P.Norm()
    assert norm(date(2026, 11, 7)) == "2026-11-07"

    class Color(Enum):
        RED = "red"

    assert norm(Color.RED) == "red"
    assert norm(None) is None and norm(True) is True and norm(1.5) == 1.5


def test_norm_bytes_are_digest_bound_not_verbatim():
    norm = P.Norm()
    payload = b"\x00\x01binary"
    assert norm(payload) == {"bytes_sha256": hashlib.sha256(payload).hexdigest()}
    assert norm(bytearray(payload)) == norm(memoryview(payload))


@dataclass
class _Sample:
    name: str
    when: datetime


def test_norm_dataclasses_and_containers():
    norm = P.Norm()
    when = datetime(2026, 11, 7, 12, 0, 0, tzinfo=timezone.utc)
    sample = _Sample(name="x", when=when)
    assert norm(sample) == {"__type__": "_Sample", "name": "x", "when": "<t0>"}
    assert norm({1: "a", "b": (2, 3)}) == {"1": "a", "b": [2, 3]}
    assert norm({3, 1, 2}) == [1, 2, 3]  # sets sorted by canon, never iteration order


def test_norm_model_dump_and_obj_unwrap_and_fallback():
    norm = P.Norm()

    class Model:
        def model_dump(self):
            return {"kind": "model", "stamp": datetime(2026, 1, 1, tzinfo=timezone.utc)}

    class Wrapped:
        def __init__(self):
            self.obj = {"inner": 1}

    class Opaque:
        pass

    assert norm(Model()) == {"kind": "model", "stamp": "<t0>"}
    assert norm(Wrapped()) == {"inner": 1}
    # Fallback never leaks addresses: two distinct instances canon identically.
    assert norm(Opaque()) == "<Opaque>"
    assert P.canon(norm(Opaque())) == P.canon(norm(Opaque()))


def test_norm_is_idempotent_on_normalized_output():
    norm = P.Norm()
    value = {"when": datetime(2026, 11, 7, 12, 0, 0, tzinfo=timezone.utc),
             "blob": b"data", "tags": {2, 1}, "nested": [_Sample("s", datetime(2026, 11, 7, 13, 0, 0, tzinfo=timezone.utc))]}
    once = norm(value)
    assert norm(once) == once


# --- first_diff: structural divergence paths ---


def test_first_diff_reports_structural_paths():
    assert P.first_diff({"a": 1}, {"a": 1}) is None
    leaf = P.first_diff({"a": {"b": 1}}, {"a": {"b": 2}})
    assert leaf is not None and leaf.startswith("/a/b")
    missing_key = P.first_diff({"a": 1}, {"a": 1, "k": 2})
    assert missing_key is not None and missing_key.startswith("/k") and "one side only" in missing_key
    indexed = P.first_diff([1, 2, 3], [1, 4, 3])
    assert indexed is not None and indexed.startswith("[1]")
    length = P.first_diff([1], [1, 2])
    assert length is not None and "length 1 recorded vs 2 replayed" in length
    deep = P.first_diff({"seqs": [{"ops": ["a"]}]}, {"seqs": [{"ops": ["a", "b"]}]})
    assert deep is not None and deep.startswith("/seqs[0]/ops") and "length" in deep


# --- Three-stratum canonical line counts (frozen s10_measure2 method) ---


def canonical_py_lines(path: Path) -> int:
    """ruff==0.17.3 format --line-length 100 canonical copy, counted in lines."""
    formatted = subprocess.run(
        ["uv.exe", "tool", "run", "--from", "ruff==0.17.3", "ruff", "format", "--isolated",
         "--line-length", "100", "--target-version", "py313", "--stdin-filename", str(path), "-"],
        input=path.read_bytes(), capture_output=True, check=True)
    return len(formatted.stdout.decode("utf-8").splitlines())


def canonical_rs_lines(path: Path) -> int:
    formatted = subprocess.run(
        ["rustfmt", "--edition", "2021"], input=path.read_bytes(), capture_output=True, check=True)
    return len(formatted.stdout.decode("utf-8").splitlines())


def test_three_stratum_canonical_line_counts():
    b_app, ops, seeds = 1705, 854, 644  # frozen baselines (s10_measure2.py)
    lib_hook, main_hook = 2, 15  # design-frozen registration allowances on C_app
    py = {
        "recording.py": canonical_py_lines(REPO / "src/lesscode_baseline/recording.py"),
        "scope_e3.py": canonical_py_lines(PROTO / "scope_e3.py"),
        "scope_e4.py": canonical_py_lines(PROTO / "scope_e4.py"),
        "proto_replay_test.py": canonical_py_lines(PROTO / "proto_replay_test.py"),
    }
    rs = {
        "admission.rs": canonical_rs_lines(NATIVE / "src/admission.rs"),
        "receipt.rs": canonical_rs_lines(NATIVE / "src/receipt.rs"),
    }
    # Per-file design ceilings (budgets.rs is governed separately by the
    # size-governance stratum and native/complexity-budgets.json, not here).
    assert py["recording.py"] == 372  # relocated verbatim; net 0 new logic
    assert py["scope_e3.py"] <= 168
    assert py["scope_e4.py"] <= 211
    assert py["proto_replay_test.py"] == 19
    assert rs["admission.rs"] <= 280 and rs["receipt.rs"] <= 150
    assert rs["admission.rs"] + rs["receipt.rs"] <= 430

    c_app = sum(py.values()) + sum(rs.values()) + lib_hook + main_hook
    c_total = c_app + ops + seeds  # ops/seeds retained frozen via import
    replay_share = py["proto_replay_test.py"] / 2
    e3_with_replay_share = py["scope_e3.py"] + replay_share
    e4_with_replay_share = py["scope_e4.py"] + replay_share
    assert c_app <= 1364, f"C_app={c_app} py={py} rs={rs}"
    assert b_app - c_app >= 200, f"B_app-C_app={b_app - c_app}"
    assert c_total <= 3003, f"C_total={c_total}"
    assert e3_with_replay_share < 280
    assert e4_with_replay_share < 358


# --- Specimen-venv integration (env-gated; skips with explicit reason) ---


def require_specimen(name: str) -> Path:
    if not SPECIMEN.is_file():
        pytest.skip(f"{name}: pinned specimen venv {SPECIMEN} absent; absence stays absent")
    return SPECIMEN


def run_specimen(args: list, timeout: int) -> subprocess.CompletedProcess:
    return subprocess.run([str(SPECIMEN), *args], cwd=REPO, env=specimen_env(),
                          capture_output=True, text=True, timeout=timeout)


@pytest.mark.skipif(not (PROTO2 / "proto2-e3-a.json").is_file(),
                    reason="frozen proto2 recordings absent (.runs/native-prototype)")
def test_frozen_double_record_digest_identity():
    e3_a, e3_b = (PROTO2 / f"proto2-e3-{x}.json" for x in "ab")
    e4_a, e4_b = (PROTO2 / f"proto2-e4-{x}.json" for x in "ab")
    for first, second, frozen, sequences, operations in (
        (e3_a, e3_b, FROZEN_E3_LF_BODY, 51, 290),
        (e4_a, e4_b, FROZEN_E4_LF_BODY, 50, 317),
    ):
        assert hashlib.sha256(first.read_bytes()).hexdigest() == frozen
        assert second.read_bytes() == first.read_bytes()  # double-record determinism
        meta = json.loads(first.read_text(encoding="utf-8"))["meta"]
        assert (meta["sequences"], meta["operations"]) == (sequences, operations)


@pytest.mark.skipif(not (PROTO2 / "proto2-e3-a.json").is_file() or not SPECIMEN.is_file(),
                    reason="specimen venv or frozen proto2 E3 recording absent")
def test_e3_frozen_trace_replays_without_divergence():
    run = run_specimen([str(PROTO / "scope_e3.py"), "replay",
                        "--golden", str(PROTO2 / "proto2-e3-a.json")], timeout=300)
    assert run.returncode == 0, run.stdout + run.stderr
    assert "no divergence" in run.stdout


@pytest.mark.skipif(not (PROTO2 / "proto2-e4-a.json").is_file() or not SPECIMEN.is_file(),
                    reason="specimen venv or frozen proto2 E4 recording absent")
def test_e4_frozen_trace_replays_without_divergence():
    run = run_specimen([str(PROTO / "scope_e4.py"), "replay",
                        "--golden", str(PROTO2 / "proto2-e4-a.json")], timeout=300)
    assert run.returncode == 0, run.stdout + run.stderr
    assert "no divergence" in run.stdout


E3_CAND = REPO / ".runs/e3-facdrone/cand"
E4_CAND_TURN3 = REPO / ".runs/e4-lifecycle/cand-turn3"


@pytest.mark.skipif(not E3_CAND.is_dir() or not SPECIMEN.is_file(),
                    reason="specimen venv or E3 candidate root absent (.runs/e3-facdrone/cand)")
def test_e3_candidate_acceptance_arms_pass():
    run = run_specimen([str(PROTO / "scope_e3.py"), "accept", str(E3_CAND), "--round", "21"],
                      timeout=900)
    assert run.returncode == 0, run.stdout[-2000:] + run.stderr[-2000:]
    assert "verdict exit 0" in run.stdout


@pytest.mark.skipif(not E4_CAND_TURN3.is_dir() or not SPECIMEN.is_file(),
                    reason="specimen venv or E4 candidate root absent (.runs/e4-lifecycle/cand-turn3)")
def test_e4_candidate_acceptance_arms_pass():
    run = run_specimen([str(PROTO / "scope_e4.py"), "accept", str(E4_CAND_TURN3), "--round", "21"],
                      timeout=600)
    try:
        assert run.returncode == 0, run.stdout[-2000:] + run.stderr[-2000:]
        assert "verdict exit 0" in run.stdout
    except AssertionError:
        raise  # keep any rerun evidence for inspection
    else:
        # The accept arm rewrites the tracked E4 seeds receipt deterministically;
        # keep the frozen committed bytes once the rerun is confirmed equivalent.
        subprocess.run(["git", "-C", str(REPO), "checkout", "--",
                        "docs/experiments/deconvolution/e4-lifecycle/results/seeds.json"],
                       capture_output=True)


CONTROLS_RECEIPT = "docs/experiments/deconvolution/native-prototype/results/s10-controls-2.json"


@pytest.mark.skipif(not SPECIMEN.is_file(),
                    reason="pinned specimen venv absent; adverse controls need its engine")
def test_nine_control_adverse_suite_fails_closed():
    run = run_specimen([str(PROTO / "s10_controls2.py"), "--with-pg"], timeout=900)
    receipt = REPO / CONTROLS_RECEIPT
    try:
        assert run.returncode == 0, run.stdout + run.stderr
        report = json.loads(receipt.read_text(encoding="utf-8"))
        assert report["all_fail_closed"] is True
        assert len(report["controls"]) == 9
    except AssertionError:
        raise  # keep the freshly written receipt for inspection on failure
    else:
        # The controls rerun overwrites the tracked attempt-2 receipt (its
        # interrupted-arm detail embeds wall time); restore frozen evidence.
        subprocess.run(["git", "-C", str(REPO), "checkout", "--",
                        CONTROLS_RECEIPT, CONTROLS_RECEIPT + ".sha256"],
                       capture_output=True)
