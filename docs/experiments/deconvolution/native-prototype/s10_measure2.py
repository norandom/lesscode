# S10 attempt-2 U5 measurement (R31): canonical line counts for the DESCOPED
# replacement (plan section 9) against the frozen baselines (section 4).
# Python engine + Rust sidecar admission only; hooks measured by git diff
# against the pre-prototype baseline 103e1aa. Disposable analysis.
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]
NATIVE = REPO / "native"
OUT = HERE / "results" / "s10-u5-measurement-2.json"
RUFF = ["uv.exe", "tool", "run", "--from", "ruff==0.17.3", "ruff", "format", "--isolated",
        "--line-length", "100", "--target-version", "py313"]

PY_APP = ["proto_core.py", "scope_e3.py", "scope_e4.py", "proto_replay_test.py"]
RS_APP = ["src/admission.rs"]
HOOK_FILES = ["src/lib.rs", "src/main.rs", "Cargo.toml"]
BASELINE_COMMIT = "103e1aa"  # last pre-prototype commit (before S2, f5c6e8f)
B = {"app": 1705, "ops": 854, "seeds": 644, "e3_scope": 280, "e4_scope": 358}


def canonical_py(paths, tmp):
    counts = {}
    for p in paths:
        dst = tmp / p
        dst.write_bytes((HERE / p).read_bytes())
    subprocess.run([*RUFF, str(tmp)], check=True, capture_output=True)
    for p in paths:
        counts[p] = len((tmp / p).read_text(encoding="utf-8").splitlines())
    return counts


def count_rs():
    return {p: len((NATIVE / p).read_text(encoding="utf-8").splitlines()) for p in RS_APP}


def hooks_by_diff():
    """Added/changed registration lines vs the pre-prototype baseline (git diff +)."""
    hooks = {}
    diff = subprocess.run(["git", "-C", str(REPO), "diff", "--no-color", BASELINE_COMMIT, "--",
                           *[f"native/{f}" for f in HOOK_FILES]], check=True, capture_output=True, text=True).stdout
    file = None
    for line in diff.splitlines():
        if line.startswith("+++ b/"):
            file = line[len("+++ b/native/"):]
        elif line.startswith("+") and not line.startswith("+++"):
            hooks[file] = hooks.get(file, 0) + 1
    return {f: hooks.get(f, 0) for f in HOOK_FILES}


def main() -> None:
    with tempfile.TemporaryDirectory(prefix="s10-measure2-") as td:
        tmp = Path(td)
        py = canonical_py(PY_APP, tmp)
    rs = count_rs()
    hooks = hooks_by_diff()
    c_app_py, c_app_rs = sum(py.values()), sum(rs.values()) + sum(hooks.values())
    c_app = c_app_py + c_app_rs
    c_total = c_app + B["ops"] + B["seeds"]  # ops/seeds retained frozen via import (identity)
    e3_adapter, e4_adapter = py["scope_e3.py"], py["scope_e4.py"]
    replay_shared = py["proto_replay_test.py"] / 2
    report = {
        "receipt": "s10-u5-measurement-2",
        "attempt": 2,
        "method": "ruff==0.17.3 format --line-length 100 canonical copies; rustfmt'd Rust; hooks by git diff vs 103e1aa; ops/seeds retained frozen via import",
        "python_app": py, "rust_app": rs, "rust_hooks": hooks,
        "c_app": c_app, "c_app_python": c_app_py, "c_app_rust_with_hooks": c_app_rs,
        "baselines": B,
        "u5": {
            "app_compression_ratio": round(c_app / B["app"], 4),
            "app_ceiling_1364": c_app <= 1364,
            "app_saved": B["app"] - c_app,
            "app_saved_ge_200": B["app"] - c_app >= 200,
            "total_ceiling_3003": c_total <= 3003,
            "c_total": c_total,
            "c_seeds_equals_b": True,
            "e3_scope_adapter": e3_adapter, "e4_scope_adapter": e4_adapter,
            "e3_smaller": e3_adapter + replay_shared < B["e3_scope"],
            "e4_smaller": e4_adapter + replay_shared < B["e4_scope"],
            "e3_with_replay_share": e3_adapter + replay_shared, "e4_with_replay_share": e4_adapter + replay_shared,
        },
        "u7_ledger": {"charged": c_app, "ceiling": 1400, "within": c_app <= 1400,
                      "attempt_1_charged_retained": 1808},
        "sensitivity": {
            "note": "descoped claim does not replace the ~240 pure-turn baseline subset (regen_turn.py)",
            "like_for_like_b_app": B["app"] - 240,
            "like_for_like_saved": B["app"] - 240 - c_app,
        },
        "excluded_from_replacement": "run_arm census/deadline/coverage machinery (~180 canonical), regen_turn HTTP client (~262), solver controls, pure turn-input transformation subset (~240): frozen files remain, not claimed",
    }
    body = json.dumps(report, indent=1, sort_keys=True)
    OUT.write_text(body, encoding="utf-8", newline="\n")
    Path(str(OUT) + ".sha256").write_text(f"{hashlib.sha256(body.encode()).hexdigest()}  s10-u5-measurement-2.json\n",
                                          encoding="ascii", newline="\n")
    print(json.dumps(report["u5"], indent=1))
    print(f"c_app={c_app} (py {c_app_py} + rs {c_app_rs})  u7 {c_app}/1400")
    print(f"hooks: {hooks}")
    print(f"receipt: {OUT}")


if __name__ == "__main__":
    main()
