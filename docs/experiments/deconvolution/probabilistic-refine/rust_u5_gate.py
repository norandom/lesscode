# Rust-port U5 re-measurement gate driver (task 8.6, R36; re-run for R37).
#
# R37 amendments (task 9.6, slim-pass gate): the counted set extends to
# jsonval.rs (defect-7 split / shared kernels, counted in full), the hook
# enumeration is the unchanged net git-diff vs the pre-port base (now 38
# added registration lines), and the record is written to a NEW file so the
# R36 gate record is preserved (Req 8.4). Ceiling unchanged at 2,270.
#
# Consolidates the deterministic G1-R checklist re-discharge (items 1, 2, 3, 5)
# through the Rust substrate (the lesscode-core crate, via its native test
# suite) with the U5 canonical measurement under the preregistered R36 counted
# set, and writes the gate evidence record with every item PASS or FAIL
# (negatives retained as measured, Req 7.8):
#
#   1. Six Tier-1 adverse controls fail closed (R < 0.50) with zero runtime
#      calls (controls_fail_closed_without_runtime,
#      control_feedback_leak_rejected_before_dispatch).
#   2. The E4 three-turn transition replay through the Rust Tier-1 ladder at
#      the recorded scores (e4_three_turn_replay_reproduces_recorded_transitions),
#      corroborated by the frozen-identity recording gates (e3/e4 double-record
#      digests, frozen trace replay).
#   3. k = 2 no-progress exhaustion halts with the best trajectory retained
#      (loop_exhaustion_k2_halts_and_retains_best,
#      loop_exhaustion_retains_best_trajectory).
#   4. U5 re-measured under the R36 counted set (this gate's own measurement):
#      native/src/{surface,measures,recording,regenerator}.rs full canonical +
#      native/src/{admission,receipt}.rs re-measured live + the counted Python
#      shim + R31-frozen 16 hook lines + this task group's enumerated
#      registration hunks, against the unchanged ceiling 2,270.
#   5. `lesscode-core admission` verifies every emitted receipt through
#      bindings.json (loop_receipts_verify_under_admission).
#
# Live weaker-model recovery is NOT re-run by this gate: R34 run B-2 stands on
# the Python substrate and no live transfer is claimed (R36 behavioral
# boundary). Experiment drivers and tests stay uncounted (g1r convention).
#
# Run (PowerShell, from D:\Source\lesscode, clean PATH per the R36 protocol):
#   $env:PATH = 'C:\Windows\System32;C:\Windows;C:\Users\mariu\.cargo\bin;C:\Program Files\Git\cmd'
#   .runs/e2-facdrone/venv/Scripts/python.exe docs/experiments/deconvolution/probabilistic-refine/rust_u5_gate.py
from __future__ import annotations

import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
RESULTS = HERE / "results"
REPO = HERE.parents[3]
NATIVE = REPO / "native"
sys.path.insert(0, str(REPO / "src"))

from lesscode_baseline import measures  # noqa: E402

PRE_PORT_COMMIT = "f2fe982"  # rust-port-base.json pre_port_commit (task 8.1)
RS_NEW = ["src/surface.rs", "src/measures.rs", "src/recording.rs", "src/regenerator.rs",
          "src/jsonval.rs"]
RS_BASE = ["src/admission.rs", "src/receipt.rs"]
SHIM_REL = "native/shim/recording_shim.py"
HOOK_FILES = ["native/src/lib.rs", "native/src/main.rs", "native/Cargo.toml"]
R31_FROZEN_HOOKS = {"src/lib.rs": 1, "src/main.rs": 15, "Cargo.toml": 0}  # R31 receipt, frozen
RUSTFMT = ["rustup", "run", "1.98.1", "rustfmt", "--edition", "2021", "--emit", "stdout"]
NEXTEST = ["rustup", "run", "1.98.1", "cargo", "nextest", "run",
           "--manifest-path", "native/Cargo.toml"]
U5_CEILING = 2_270
B_APP_EXPANDED = 2_838
NEW_MODULE_BUDGET = 1_828  # preregistered: 2,270 - 442 pre-port base
NEXT_LINE = re.compile(r"^\s*(PASS|FAIL|SKIP|ABORT|TIMEOUT)\s+\[\s*([0-9.]+)s\]")
ANSI = re.compile(r"\x1b\[[0-9;]*m")

ITEM_TESTS = {
    "tier1_controls": ["controls_fail_closed_without_runtime",
                       "control_feedback_leak_rejected_before_dispatch"],
    "transition_replay": ["e4_three_turn_replay_reproduces_recorded_transitions"],
    "no_progress": ["loop_exhaustion_k2_halts_and_retains_best",
                    "loop_exhaustion_retains_best_trajectory"],
    "admission": ["loop_receipts_verify_under_admission"],
}
CORROBORATING_IDENTITY_TESTS = [
    "e3_double_record_reproduces_frozen_lf_body_digest",
    "e4_double_record_reproduces_frozen_lf_body_digest",
    "frozen_traces_replay_without_divergence",
]
ITEM_LABELS = {
    "tier1_controls": "six tier-1 controls fail closed, zero runtime calls (Rust substrate)",
    "transition_replay": "E4 transition replay: t1/t2 tier-1 at recorded scores, t3 accepted",
    "no_progress": "k=2 no-progress exhaustion retains best (Rust substrate)",
    "admission": "lesscode-core admission verifies loop receipts via bindings.json",
}


def preflight() -> None:
    missing = [t for t in ("rustup", "cargo", "git", "uv") if shutil.which(t) is None]
    if missing:
        sys.exit(f"preflight failed: {missing} not on PATH — run with the clean PATH from the header")


def run_substrate_test(name: str) -> dict:
    """Run one native test through cargo nextest (pinned toolchain) and parse its outcome."""
    t0 = time.perf_counter()
    env = dict(os.environ, NO_COLOR="1")
    proc = subprocess.run([*NEXTEST, name], capture_output=True, text=True,
                          cwd=str(REPO), timeout=900, env=env)
    runs = []
    # nextest writes the per-test final-status lines to stderr.
    for line in (proc.stderr or "").splitlines():
        line = ANSI.sub("", line)
        m = NEXT_LINE.match(line)
        if m:
            runs.append({"status": m.group(1), "seconds": float(m.group(2)),
                         "test": line.split()[-1]})
    matched = [r for r in runs if name in r["test"]]
    return {"filter": name, "exit": proc.returncode, "runs": runs,
            "matched": matched, "pass": proc.returncode == 0 and bool(matched)
            and all(r["status"] == "PASS" for r in matched),
            "wall_seconds": round(time.perf_counter() - t0, 1),
            "stderr_tail": (proc.stderr or "")[-400:] if proc.returncode != 0 else ""}


def item_from_tests(key: str) -> dict:
    tests = [run_substrate_test(t) for t in ITEM_TESTS[key]]
    ok = all(t["pass"] for t in tests) and bool(tests)
    return {"item": ITEM_LABELS[key], "pass": ok,
            "channel": "cargo nextest (pinned toolchain 1.98.1) over the lesscode-core native suite",
            "tests": [{k: t[k] for k in ("filter", "exit", "matched", "pass", "wall_seconds")}
                      for t in tests]}


def item_transition_replay() -> dict:
    item = item_from_tests("transition_replay")
    # Tier-2 replay identities: the frozen E3/E4 digests and trace replay
    # through the Rust recording driver corroborate the replayed verdict.
    item["tier2_replay_identities"] = [
        {k: t[k] for k in ("filter", "exit", "matched", "pass")}
        for t in (run_substrate_test(n) for n in CORROBORATING_IDENTITY_TESTS)]
    item["pass"] = item["pass"] and all(t["pass"] for t in item["tier2_replay_identities"])
    return item


def canonical_rs(rel: str, tmp: Path) -> int:
    dst = tmp / Path(rel).name
    dst.write_bytes((NATIVE / rel).read_bytes())
    fmt = subprocess.run([*RUSTFMT, str(dst)], check=True, capture_output=True, text=True)
    return len(fmt.stdout.splitlines())


def measure_hooks() -> dict:
    """Enumerated git-diff hunks of this task group's registration hooks (R36 method)."""
    proc = subprocess.run(["git", "-C", str(REPO), "diff", PRE_PORT_COMMIT, "HEAD", "--",
                           *HOOK_FILES], check=True, capture_output=True, text=True)
    added: dict[str, int] = {}
    hunks: list[dict] = []
    current = None
    for line in proc.stdout.splitlines():
        if line.startswith("+++ b/"):
            current = line[6:]
            added.setdefault(current, 0)
        elif line.startswith("@@") and current is not None:
            hunks.append({"file": current, "header": line})
        elif line.startswith("+") and current is not None:
            added[current] += 1
    return {"base": PRE_PORT_COMMIT, "added_lines": added, "total_added": sum(added.values()),
            "hunks": hunks, "diff": proc.stdout}


def item_u5(work: Path) -> dict:
    """U5 re-measurement under the R36 counted set (the gate's own arithmetic)."""
    rs_counts: dict[str, int] = {}
    with tempfile.TemporaryDirectory(prefix="rust-u5-") as td:
        tmp = Path(td)
        for rel in RS_NEW + RS_BASE:
            rs_counts[f"native/{rel}"] = canonical_rs(rel, tmp)
        shim_report = measures.measure_canonical(REPO, [SHIM_REL], tmp / "fmt")
    if shim_report is None:
        return {"item": "U5 replaceable apparatus <= 2,270 canonical lines (Rust substrate, R37 set)",
                "pass": False,
                "detail": {"error": "pinned ruff unavailable; canonical reading must never be zero (Req 6.3)"}}
    hooks = measure_hooks()
    # measure_canonical keys files by basename (frozen measurer shape).
    shim_lines = shim_report["files"][Path(SHIM_REL).name]["lines"]
    new_total = sum(rs_counts[f"native/{rel}"] for rel in RS_NEW)
    base_total = sum(rs_counts[f"native/{rel}"] for rel in RS_BASE)
    frozen_hooks_total = sum(R31_FROZEN_HOOKS.values())
    total = new_total + base_total + shim_lines + frozen_hooks_total + hooks["total_added"]
    return {
        "item": "U5 replaceable apparatus <= 2,270 canonical lines (Rust substrate, R37 set)",
        "pass": total <= U5_CEILING,
        "detail": {
            "rs_canonical": rs_counts,
            "new_modules_total": new_total,
            "base_remeasured_total": base_total,
            "shim_canonical": {SHIM_REL: shim_lines},
            "r31_frozen_hooks": R31_FROZEN_HOOKS, "r31_frozen_hooks_total": frozen_hooks_total,
            "task_group_hooks": {k: hooks[k] for k in ("base", "added_lines", "total_added", "hunks")},
            "total": total, "ceiling": U5_CEILING, "baseline_expanded": B_APP_EXPANDED,
            "new_module_budget_preregistered": NEW_MODULE_BUDGET,
            "overage": total - U5_CEILING,
            "verdict_note": "measured as-is; negative retained (Req 7.8); owner decides amend/close (R37 slim levers exhausted)",
            "method": "R36 amendment (prototype-gate.md): rustfmt (toolchain 1.98.1, --edition 2021) "
                      "formatted copies, splitlines() counts; shim under the Python rule "
                      "(ruff==0.17.3 format --isolated --line-length 100 --target-version py313); "
                      "hooks by enumerated git-diff hunks; drivers and tests uncounted (g1r convention)",
            "commands": {
                "rustfmt": "rustup run 1.98.1 rustfmt --edition 2021 --emit stdout <file-copy>",
                "shim": "lesscode_baseline.measures.measure_canonical -> uv tool run --offline "
                        "--python 3.13 ruff==0.17.3 format --isolated --line-length 100 "
                        "--target-version py313",
                "hooks": f"git diff {PRE_PORT_COMMIT} HEAD -- " + " ".join(HOOK_FILES),
                "tests": "rustup run 1.98.1 cargo nextest run --manifest-path "
                         "native/Cargo.toml <test-name>"},
            "excluded": {
                "python_reference_modules": "src/lesscode_baseline/{recording,surface,measures,"
                                            "regenerator}.py (superseded stage-1 reference, R36)",
                "product_modules": "native scope.rs, metrics.rs, compare.rs, model.rs, budgets.rs "
                                   "(uncounted product, reuse boundary)",
                "testinfra": "outside U5 under U7 verbatim relocation",
                "drivers_and_tests": "this driver and the native/ Python test suites (g1r strata)"},
        }}


def main() -> int:
    preflight()
    head = subprocess.run(["git", "-C", str(REPO), "rev-parse", "HEAD"], check=True,
                          capture_output=True, text=True).stdout.strip()
    RESULTS.mkdir(parents=True, exist_ok=True)
    record = {
        "receipt": "rust-u5-gate-r37-1",
        "task": "9.6",
        "written_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "deterministic": True,
        "substrate": {"crate": "lesscode-core (native)", "toolchain": "1.98.1",
                      "head": head, "pre_port_commit": PRE_PORT_COMMIT},
        "checklist_note": "G1-R items 1, 2, 3 and 5 re-discharged through the Rust substrate; "
                          "item 4 (U5) is this gate's own re-measurement under the R37 counted "
                          "set (R36 set + jsonval.rs, hooks net vs pre-port); live weaker-model "
                          "recovery is not re-run (R34 run B-2 stands on Python; no live "
                          "transfer claimed).",
        "items": [],
    }
    with tempfile.TemporaryDirectory(prefix="rust-u5-gate-") as scratch:
        work = Path(scratch)
        order = [("tier1_controls", lambda: item_from_tests("tier1_controls")),
                 ("transition_replay", item_transition_replay),
                 ("no_progress", lambda: item_from_tests("no_progress")),
                 ("u5_ceiling", lambda: item_u5(work)),
                 ("admission", lambda: item_from_tests("admission"))]
        for key, fn in order:
            try:
                record["items"].append(fn())
            except Exception as e:  # negatives complete the stage (Req 7.8)
                record["items"].append({"item": key, "pass": False,
                                        "error": f"{type(e).__name__}: {e}"})
    record["all_pass"] = all(item.get("pass") for item in record["items"])
    body = json.dumps(record, indent=1, sort_keys=True)
    out = RESULTS / "rust-u5-gate-r37.json"
    out.write_text(body, encoding="utf-8", newline="\n")
    Path(str(out) + ".sha256").write_text(
        f"{hashlib.sha256(body.encode()).hexdigest()}  rust-u5-gate-r37.json\n",
        encoding="ascii", newline="\n")
    for item in record["items"]:
        print(f"[{'PASS' if item.get('pass') else 'FAIL'}] {item['item']}")
        if not item.get("pass"):
            print("       ", json.dumps(item.get("detail", item.get("error", item.get("tests"))))[:400])
    print(f"all_pass: {record['all_pass']} -> {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
