"""G1-R deterministic verification driver (constraint-guided-compression task 6.1).

Discharges the deterministic prototype-gate.md checklist on frozen artifacts and
writes the gate evidence record with every item PASS or FAIL (negative results
retained, Req 7.8):

  1. Six Tier-1 adverse controls fail closed (R < 0.50) with zero runtime calls.
  2. The E4 three-turn transition replay: Turn 1 (max_tokens) and Turn 2
     (missing `labeled_reason` under Contract B) are caught in Tier 1 < 2 s;
     Turn 3 passes Tier 1 (Tier-2 verdict replayed from the frozen R31 receipt).
  3. k = 2 no-progress exhaustion halts with the best trajectory retained.
  4. U5 whole-replacement accounting: canonical lines of the replaceable
     apparatus against the expanded baseline stratum (ceiling 2,270).
  5. `lesscode-core admission` verifies every emitted receipt and sidecar
     through bindings.json.

The E4 replay derives Contract B on a scratch byte-copy of the frozen tree with
the recorder oracle (``e4_oracle.py``) placed at its repo-relative path inside
that copy (``derive_surface`` resolves ``recorder_files`` as ``root/rel``); the
frozen specimen itself is never modified. Admission is verified for every run
directory that emitted ``bindings.json``.

Run:  venv-python g1r_spike.py [--lesscode-core PATH]
"""
from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
RESULTS = HERE / "results"
LESSCODE = HERE.parents[3]
DC = HERE.parents[0]
sys.path.insert(0, str(LESSCODE / "src"))

from lesscode_baseline import measures, regenerator as rg, surface  # noqa: E402

E4_TURNS = DC / "regenerator" / "results" / "turns"
E4_ROOT = LESSCODE / ".runs" / "e2-facdrone" / "source"
E4_SURFACE = DC / "e4-lifecycle" / "results" / "public-surface.json"
E4_RECORDER = DC / "e4-lifecycle" / "e4_oracle.py"

REPLACEABLE_PY = ["src/lesscode_baseline/recording.py", "src/lesscode_baseline/surface.py",
                  "src/lesscode_baseline/measures.py", "src/lesscode_baseline/regenerator.py"]
# Frozen rustfmt canonical share of the replaceable apparatus (R27/R31 records:
# admission.rs + receipt.rs + registration hooks), with recording-era Python
# excluded because the live Python side above supersedes it.
FROZEN_RUST_CANONICAL = 1_333
B_APP_EXPANDED = 2_838
U5_CEILING = 2_270
FROZEN_TIER2_ACCEPTED = {"exit": 0, "first_divergence": None,
                         "evidence": "R31 attempt-2 receipts, docs/research/2026-11-07/05-descoped-stratum-rerun.md"}

GOOD = "def f():\n    return 1\n"
SYNTAX = "def f(:\n"


def _response_text(files: dict, structure: dict | None = None) -> str:
    return json.dumps({"files": [{"path": p, "content": c} for p, c in files.items()],
                       "structure": structure or {"pure": [], "shell": []}, "change_log": "spike"})


def _fake_call(script: list):
    def call_fn(system: str, user: str, params: dict):
        text, meta = script.pop(0)
        return text, dict(meta, **params)

    return call_fn


def _mini_request(work: Path) -> tuple[rg.TurnRequest, Path]:
    frozen = work / "frozen"
    (frozen / "src").mkdir(parents=True)
    (frozen / "src" / "mod.py").write_text(GOOD, encoding="utf-8")
    (frozen / "src" / "user.py").write_text("from mod import f\n", encoding="utf-8")
    report = surface.derive_surface(frozen, ["src/mod.py"])
    (frozen / "public-surface.json").write_text(json.dumps(report), encoding="utf-8")
    req = rg.TurnRequest(scope_root=str(frozen), scope_files=["src/mod.py"],
                         surface_path=str(frozen / "public-surface.json"), definitions="d",
                         targets={}, contract="c", acceptance_command="pytest -q")
    return req, frozen


def item_controls(work: Path) -> dict:
    """Checklist 1: the six Tier-1 controls fail closed with zero runtime calls."""
    controls = [
        ("ctrl_max_tokens", [(None, {"stop_reason": "max_tokens"})], 0.00),
        ("ctrl_out_of_scope", [(_response_text({"src/other.py": "x = 1\n"}), {"stop_reason": "end_turn"})], 0.20),
        ("ctrl_syntax_error", [(_response_text({"src/mod.py": SYNTAX}), {"stop_reason": "end_turn"})], 0.30),
        ("ctrl_missing_surface_b", [(_response_text({"src/mod.py": "def g():\n    return 2\n"}), {"stop_reason": "end_turn"})], 0.40),
        ("ctrl_write_in_pure", [(_response_text({"src/mod.py": "def f(conn):\n    conn.execute('s')\n    return 1\n"}, {"pure": ["mod.py::f"], "shell": []}), {"stop_reason": "end_turn"})], 0.45),
    ]
    detail, ok = {}, True
    for name, script, expected in controls:
        req, frozen = _mini_request(work / name)
        evaluator_calls = []

        def evaluator(root, results, _c=evaluator_calls):
            _c.append(root)
            return (0, None, {})

        outcome = rg.run_refine_loop(req, work / name / "cand", frozen, work / name / "receipt.json", name,
                                     rg.RefineBudget(max_turns=1, max_static_retries=3),
                                     runtime_evaluator=evaluator, call_fn=_fake_call(script * 3))
        passed = outcome.best.score == expected and not evaluator_calls
        detail[name] = {"score": outcome.best.score, "expected": expected,
                        "runtime_calls": len(evaluator_calls), "pass": passed}
        ok = ok and passed
    leak_payload = {"kind": "failed", "first_divergence": "div in HOLDOUT-1", "measure_gaps": {}}
    try:
        rg.enforce_feedback_boundary(leak_payload, held_out_names=("HOLDOUT-1",))
        detail["ctrl_feedback_leak"] = {"pass": False, "error": "held-out name was not rejected"}
        ok = False
    except rg.FeedbackBoundaryError as e:
        detail["ctrl_feedback_leak"] = {"pass": True, "error": str(e)[:200]}
    return {"item": "six tier-1 controls fail closed, zero runtime calls", "pass": ok, "detail": detail}


def item_transition_replay(work: Path) -> dict:
    """Checklist 2: the frozen E4 three-turn transition through Tier 1, < 2 s per check."""
    detail: dict = {"artifacts": str(E4_TURNS)}
    scope = _e4_scope_files()
    # One byte-copy of the frozen tree serves as the Contract-B root so the
    # recorder oracle sits under root/ (derive_surface resolves recorder_files
    # as root/rel); the frozen specimen itself is never modified.
    contract_root = work / "frozen-contract"
    rg.prepare_candidate(E4_ROOT, contract_root, scope)
    recorder_rel = E4_RECORDER.relative_to(LESSCODE).as_posix()
    oracle_copy = contract_root / recorder_rel
    oracle_copy.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(E4_RECORDER, oracle_copy)
    contract_b = surface.derive_surface(contract_root, scope, recorder_files=(recorder_rel,))
    detail["contract_b_recorder_symbols"] = sorted(
        name for info in contract_b["modules"].values() for name, e in info["names"].items()
        if "recorder" in e["tags"])
    outcomes = {}
    # Turn 1: recorded stop_reason max_tokens (frozen receipt: e4-turn1).
    t0 = time.perf_counter()
    reward = rg.evaluate_tier1_static(
        _e4_request(work), {"ok": False, "kind": "stop_reason", "detail": "max_tokens"}, E4_ROOT, contract_b)
    outcomes["t1_max_tokens"] = {"score": reward.score, "kind": reward.failure_kind,
                                 "seconds": round(time.perf_counter() - t0, 3)}
    # Turn 2: the recorded response omitted `labeled_reason` (Contract B catches it).
    t2_text = _frozen_response("e4-turn2")
    cand2 = _materialize(work / "e4t2", t2_text, contract_root)
    t0 = time.perf_counter()
    reward2 = rg.evaluate_tier1_static(_e4_request(work), {"ok": True, "result": None, "written": []},
                                      cand2, contract_b)
    missing = reward2.diagnostics.get("missing_symbols")
    outcomes["t2_missing_labeled_reason"] = {
        "score": reward2.score, "kind": reward2.failure_kind, "missing": missing,
        "caught_labeled_reason": bool(missing and any(n == "labeled_reason" for _, n in missing)),
        "seconds": round(time.perf_counter() - t0, 3)}
    # Turn 3: the recorded accepted response passes Tier 1; the Tier-2 verdict is
    # replayed from the frozen R31 receipt (deterministic, no PostgreSQL here).
    t3_text = _frozen_response("e4-turn3")
    cand3 = _materialize(work / "e4t3", t3_text, contract_root)
    t0 = time.perf_counter()
    reward3 = rg.evaluate_tier1_static(_e4_request(work), {"ok": True, "result": None, "written": []},
                                      cand3, contract_b)
    tier2 = rg.evaluate_tier2_runtime(FROZEN_TIER2_ACCEPTED["exit"], FROZEN_TIER2_ACCEPTED["first_divergence"], {})
    outcomes["t3_acceptance"] = {"tier1_score": reward3.score, "tier1_kind": reward3.failure_kind,
                                 "tier2_replayed": tier2.score, "tier2_evidence": FROZEN_TIER2_ACCEPTED["evidence"],
                                 "seconds": round(time.perf_counter() - t0, 3)}
    detail["turns"] = outcomes
    passed = (outcomes["t1_max_tokens"]["score"] == 0.00
              and outcomes["t1_max_tokens"]["seconds"] < 2.0
              and outcomes["t2_missing_labeled_reason"]["caught_labeled_reason"]
              and outcomes["t2_missing_labeled_reason"]["score"] == 0.40
              and outcomes["t2_missing_labeled_reason"]["seconds"] < 2.0
              and outcomes["t3_acceptance"]["tier1_score"] == 0.50
              and outcomes["t3_acceptance"]["seconds"] < 2.0
              and outcomes["t3_acceptance"]["tier2_replayed"] == 1.00)
    return {"item": "E4 transition replay: t1/t2 tier-1 <2s, t3 accepted", "pass": passed, "detail": detail}


def _e4_scope_files() -> list[str]:
    """The frozen seven-file E4 scope, read from surface4.SET the way s9 did (import, not re-parse)."""
    import importlib.util

    spec = importlib.util.spec_from_file_location("surface4_spike", DC / "e4-lifecycle" / "surface4.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)  # frozen apparatus: stdlib-only top level, verified in R26
    return list(module.SET)


def _e4_request(work: Path) -> rg.TurnRequest:
    return rg.TurnRequest(scope_root=str(E4_ROOT), scope_files=_e4_scope_files(),
                          surface_path=str(E4_SURFACE), definitions="", targets={},
                          contract="", acceptance_command="")


def _frozen_response(turn: str) -> str:
    """The frozen model text: the first text block of the stored response envelope."""
    msg = json.loads((E4_TURNS / turn / "response.json").read_text(encoding="utf-8"))
    return next(b["text"] for b in msg["content"] if b["type"] == "text")


def _materialize(work: Path, response_text: str, root: Path) -> Path:
    result = rg.parse_result(response_text, {})
    cand = work / "cand"
    shutil.rmtree(work, ignore_errors=True)
    rg.prepare_candidate(root, cand, _e4_scope_files())
    rg.write_candidate(result, cand, _e4_scope_files())
    return cand


def item_no_progress(work: Path) -> dict:
    """Checklist 3: k=2 no-progress exhaustion halts, best trajectory retained."""
    req, frozen = _mini_request(work)
    text = _response_text({"src/mod.py": GOOD})
    verdicts = [{"seed_group_pass_fraction": 0.4, "sequence_pass_fraction": 0.4},
                {"seed_group_pass_fraction": 0.4, "sequence_pass_fraction": 0.4},
                {"seed_group_pass_fraction": 0.4, "sequence_pass_fraction": 0.4}]
    state = {"n": 0}

    def evaluator(root, results):
        v = verdicts[min(state["n"], len(verdicts) - 1)]
        state["n"] += 1
        return (1, f"seq::{state['n']}", v)

    outcome = rg.run_refine_loop(req, work / "no_progress" / "cand", frozen, work / "no_progress" / "receipt.json", "no_progress",
                                 rg.RefineBudget(max_turns=4, no_progress_limit=2),
                                 runtime_evaluator=evaluator, call_fn=_fake_call([(text, {"stop_reason": "end_turn"})] * 8))
    passed = outcome.status == "exhausted" and outcome.best.score > 0 and outcome.usage["calls"] <= 4
    return {"item": "k=2 no-progress exhaustion retains best", "pass": passed,
            "detail": {"status": outcome.status, "best_score": outcome.best.score,
                       "calls": outcome.usage["calls"], "unresolved": outcome.unresolved_obligations[:5]}}


def item_u5(work: Path) -> dict:
    """Checklist 4: replaceable apparatus canonical lines against the expanded baseline."""
    report = measures.measure_canonical(LESSCODE, REPLACEABLE_PY, work / "fmt")
    if report is None:
        return {"item": "U5 replaceable apparatus ceiling", "pass": False,
                "detail": {"error": "pinned ruff unavailable; canonical reading must never be zero (Req 6.3)"}}
    per_file = {f: v["lines"] for f, v in report["files"].items()}
    python_total = report["total"]["lines"]
    total = python_total + FROZEN_RUST_CANONICAL
    return {"item": "U5 replaceable apparatus <= 2,270 canonical lines", "pass": total <= U5_CEILING,
            "detail": {"python": per_file, "python_total": python_total,
                       "frozen_rust_and_hooks_canonical": FROZEN_RUST_CANONICAL,
                       "provenance": "admission.rs + receipt.rs + registration hooks, rustfmt counts frozen in R27/R31",
                       "total": total, "ceiling": U5_CEILING, "baseline_expanded": B_APP_EXPANDED,
                       "note": "testinfra.py (1,081 canonical) is accounted under U7 verbatim relocation, outside U5"}}


def item_admission(core: Path, work: Path) -> dict:
    """Checklist 5: lesscode-core admission verifies every emitted receipt and sidecar via bindings.json."""
    if not core.exists():
        return {"item": "admission verifies receipts via bindings.json", "pass": False,
                "detail": {"error": f"lesscode-core binary not found at {core}"}}
    runs = []
    ok = True
    for root in sorted(p for p in work.iterdir() if (p / "bindings.json").is_file()):
        proc = subprocess.run([str(core), "admission", "--root", str(root), "--bindings",
                               str(root / "bindings.json"), "--out", str(root / "admission.json")],
                              capture_output=True, text=True, cwd=str(root), timeout=120)
        runs.append({"run": root.name, "exit": proc.returncode, "pass": proc.returncode == 0,
                     "tail": (proc.stdout or proc.stderr)[-300:]})
        ok = ok and proc.returncode == 0
    return {"item": "admission verifies receipts via bindings.json", "pass": ok and bool(runs),
            "detail": {"runs": runs}}


def main(argv: list[str]) -> int:
    core = Path(argv[argv.index("--lesscode-core") + 1]) if "--lesscode-core" in argv else (
        LESSCODE / "native" / "target" / "release" / "lesscode-core.exe")
    if not core.exists():
        core = LESSCODE / "native" / "target" / "debug" / "lesscode-core.exe"
    RESULTS.mkdir(parents=True, exist_ok=True)
    record = {"receipt": "g1-r-spike-1", "written_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
              "deterministic": True, "items": []}
    with tempfile.TemporaryDirectory(prefix="g1r-", dir=str(RESULTS)) as scratch:
        work = Path(scratch)
        checklist = [("tier1_controls", lambda: item_controls(work)),
                     ("transition_replay", lambda: item_transition_replay(work)),
                     ("no_progress", lambda: item_no_progress(work)),
                     ("u5_ceiling", lambda: item_u5(work)),
                     ("admission", lambda: item_admission(core, work))]
        for name, fn in checklist:
            try:
                record["items"].append(fn())
            except Exception as e:  # negative results complete the stage (Req 7.8)
                record["items"].append({"item": name, "pass": False, "error": f"{type(e).__name__}: {e}"})
    record["all_pass"] = all(item.get("pass") for item in record["items"])
    (RESULTS / "g1r-record.json").write_text(json.dumps(record, indent=1), encoding="utf-8")
    for item in record["items"]:
        print(f"[{'PASS' if item.get('pass') else 'FAIL'}] {item['item']}")
        if not item.get("pass"):
            print("       ", json.dumps(item.get("detail", item.get("error")))[:400])
    print(f"all_pass: {record['all_pass']} -> {RESULTS / 'g1r-record.json'}")
    return 0 if record["all_pass"] else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
