# Rust-apparatus slim pass — pre-slim baseline receipt (task 9.1, R37).
#
# Captures the live canonical counts of the full counted apparatus (six Rust
# modules + the embedded Python shim under the ruff rule) plus the pre-slim
# commit identity, BEFORE any slim code exists, under the frozen canonical
# method (rustfmt-formatted copies, splitlines() counts; shim under
# ruff==0.17.3 format --isolated; R27-R36 method, unchanged).
#
# Run:  venv-python rust_slim_base.py
from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]
NATIVE = REPO / "native"
OUT = HERE / "results" / "rust-slim-base.json"
RS_APP = ["src/admission.rs", "src/receipt.rs", "src/jsonval.rs", "src/surface.rs",
          "src/measures.rs", "src/recording.rs", "src/regenerator.rs"]
SHIM = "shim/recording_shim.py"
HOOKS_AT_BASELINE = {"r31_frozen": 16, "group8_enumerated": 21, "task93_jsonval_registration": 1}
RUSTFMT = ["rustup", "run", "1.98.1", "rustfmt", "--edition", "2021", "--emit", "stdout"]
RUFF = [sys.executable, "-m", "ruff", "format", "--isolated", "--line-length", "100",
        "--target-version", "py313"]


def canonical_rs(path: Path, tmp: Path) -> int:
    dst = tmp / path.name
    dst.write_bytes(path.read_bytes())
    fmt = subprocess.run([*RUSTFMT, str(dst)], check=True, capture_output=True, text=True)
    return len(fmt.stdout.splitlines())


def canonical_py(path: Path, tmp: Path) -> int:
    dst = tmp / path.name
    dst.write_bytes(path.read_bytes())
    subprocess.run([*RUFF, str(dst)], check=True, capture_output=True, text=True)
    return len(dst.read_text(encoding="utf-8").splitlines())


def main() -> None:
    head = subprocess.run(["git", "-C", str(REPO), "rev-parse", "HEAD"], check=True,
                          capture_output=True, text=True).stdout.strip()
    with tempfile.TemporaryDirectory(prefix="rust-slim-base-") as td:
        counts = {p: canonical_rs(NATIVE / p, Path(td)) for p in RS_APP}
        shim = canonical_py(NATIVE / SHIM, Path(td))
    rs_total = sum(counts.values())
    hooks_total = sum(HOOKS_AT_BASELINE.values())
    pre_slim_total = rs_total + shim + hooks_total
    report = {
        "receipt": "rust-slim-base",
        "task": "9.1",
        "pre_slim_commit": head,
        "method": "rustfmt (toolchain 1.98.1, --edition 2021) formatted copies, splitlines() "
                  "counts; shim under ruff==0.17.3 format --isolated --line-length 100 "
                  "--target-version py313; identical to the R27/R31/R34/R36 canonical method",
        "rust_app_live": counts,
        "rust_app_live_total": rs_total,
        "shim_ruff_live": {SHIM: shim},
        "hooks_at_baseline": HOOKS_AT_BASELINE,
        "pre_slim_total": pre_slim_total,
        "frozen_reference": {
            "r36_gate_commit": "7657969",
            "r36_measured_total": 7049,
            "r36_per_module": {"surface.rs": 1154, "measures.rs": 1428, "recording.rs": 1002,
                                "regenerator.rs": 2798, "admission.rs": 274, "receipt.rs": 178,
                                "shim": 178, "hooks_r31": 16, "hooks_group8": 21},
            "u5_ceiling": 2270,
        },
        "budget_note": f"required reduction to pass = {pre_slim_total - 2270} of {pre_slim_total} "
                       f"canonical lines; ceiling and baseline unchanged (R37)",
    }
    body = json.dumps(report, indent=1, sort_keys=True)
    OUT.write_text(body, encoding="utf-8", newline="\n")
    Path(str(OUT) + ".sha256").write_text(
        f"{hashlib.sha256(body.encode()).hexdigest()}  rust-slim-base.json\n", encoding="ascii", newline="\n")
    print(json.dumps({"head": head, "rust_app_live": counts, "rs_total": rs_total,
                      "shim": shim, "hooks": hooks_total, "pre_slim_total": pre_slim_total,
                      "r36_reference_total": 7049,
                      "matches_r36": pre_slim_total == 7049}, indent=1))
    print(f"receipt: {OUT}")


if __name__ == "__main__":
    main()
