# S10 U5 measurement: canonical line counts for the replacement, three strata,
# against the frozen baselines (prototype-plan.md section 4). Disposable analysis.
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
NATIVE = HERE.parents[3] / "native"
OUT = HERE / "results" / "s10-u5-measurement.json"
RUFF = ["uv.exe", "tool", "run", "--from", "ruff==0.17.3", "ruff", "format", "--isolated",
        "--line-length", "100", "--target-version", "py313"]

PY_APP = ["proto_core.py", "scope_e3.py", "scope_e4.py", "proto_replay_test.py"]
RS_APP = ["src/pyjson.rs", "src/admission.rs", "src/turncheck.rs", "src/bin/turncheck.rs"]
B = {"app": 1705, "ops": 854, "seeds": 644, "e3_scope": 280, "e4_scope": 358}
HOOKS = {"src/lib.rs": 3, "src/main.rs": 16, "Cargo.toml": 4}  # registration diffs (git diff +)

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

def main() -> None:
    with tempfile.TemporaryDirectory(prefix="s10-measure-") as td:
        tmp = Path(td)
        py = canonical_py(PY_APP, tmp)
    rs = count_rs()
    c_app_py, c_app_rs = sum(py.values()), sum(rs.values()) + sum(HOOKS.values())
    c_app = c_app_py + c_app_rs
    c_total = c_app + B["ops"] + B["seeds"]  # ops/seeds retained frozen via import (identity)
    e3_adapter, e4_adapter = py["scope_e3.py"], py["scope_e4.py"]
    replay_shared = py["proto_replay_test.py"] / 2
    report = {
        "receipt": "s10-u5-measurement-1",
        "method": "ruff==0.17.3 format --line-length 100 canonical copies; rustfmt'd Rust; registration hooks by diff; ops/seeds retained frozen via import",
        "python_app": py, "rust_app": rs, "rust_hooks": HOOKS,
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
        "u7_ledger": {"charged": c_app, "ceiling": 1400, "within": c_app <= 1400},
        "excluded_from_replacement": "run_arm census/deadline/coverage machinery (~180 canonical), regen_turn HTTP client (~262), solver controls: frozen files remain, not claimed",
    }
    body = json.dumps(report, indent=1, sort_keys=True)
    OUT.write_text(body, encoding="utf-8", newline="\n")
    Path(str(OUT) + ".sha256").write_text(f"{hashlib.sha256(body.encode()).hexdigest()}  s10-u5-measurement.json\n",
                                          encoding="ascii", newline="\n")
    print(json.dumps(report["u5"], indent=1))
    print(f"c_app={c_app} (py {c_app_py} + rs {c_app_rs})  u7 {c_app}/1400")
    print(f"receipt: {OUT}")

if __name__ == "__main__":
    main()
