# Rust-port U5 re-measurement — pre-port base receipt (task 8.1, R36).
#
# Captures the live canonical counts of the existing Rust apparatus files
# (admission.rs, receipt.rs) plus the pre-port commit identity, BEFORE any
# port code exists, under the frozen canonical method (rustfmt-formatted
# copies, splitlines() counts; s10_measure2/R31 method, unchanged).
#
# Run:  venv-python rust_port_base.py
from __future__ import annotations

import hashlib
import json
import subprocess
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]
NATIVE = REPO / "native"
OUT = HERE / "results" / "rust-port-base.json"
RS_APP = ["src/admission.rs", "src/receipt.rs"]
R31_FROZEN_HOOKS = {"src/lib.rs": 1, "src/main.rs": 15, "Cargo.toml": 0}  # R31 receipt, unchanged method source
RUSTFMT = ["rustup", "run", "1.98.1", "rustfmt", "--edition", "2021", "--emit", "stdout"]


def canonical_rs(path: Path, tmp: Path) -> int:
    dst = tmp / path.name
    dst.write_bytes(path.read_bytes())
    fmt = subprocess.run([*RUSTFMT, str(dst)], check=True, capture_output=True, text=True)
    return len(fmt.stdout.splitlines())


def main() -> None:
    head = subprocess.run(["git", "-C", str(REPO), "rev-parse", "HEAD"], check=True,
                          capture_output=True, text=True).stdout.strip()
    with tempfile.TemporaryDirectory(prefix="rust-port-base-") as td:
        counts = {p: canonical_rs(NATIVE / p, Path(td)) for p in RS_APP}
    rust_total = sum(counts.values())
    report = {
        "receipt": "rust-port-base",
        "task": "8.1",
        "pre_port_commit": head,
        "method": "rustfmt (toolchain 1.98.1, --edition 2021) formatted copies, splitlines() counts; "
                  "identical to the R27/R31/R34 canonical method",
        "rust_app_live": counts,
        "rust_app_live_total": rust_total,
        "r31_frozen_hooks": R31_FROZEN_HOOKS,
        "frozen_reference": {
            "g1r_frozen_rust_canonical": 1333,
            "python_app_canonical_r34": {"recording.py": 372, "surface.py": 281,
                                          "measures.py": 558, "regenerator.py": 1239},
            "u5_ceiling": 2270, "baseline_expanded": 2838,
        },
        "budget_note": f"new-module budget before hook growth ~= {2270 - rust_total - sum(R31_FROZEN_HOOKS.values())}",
    }
    body = json.dumps(report, indent=1, sort_keys=True)
    OUT.write_text(body, encoding="utf-8", newline="\n")
    Path(str(OUT) + ".sha256").write_text(
        f"{hashlib.sha256(body.encode()).hexdigest()}  rust-port-base.json\n", encoding="ascii", newline="\n")
    print(json.dumps({"head": head, "rust_app_live": counts, "total": rust_total,
                      "new_module_budget": 2270 - rust_total - sum(R31_FROZEN_HOOKS.values())}, indent=1))
    print(f"receipt: {OUT}")


if __name__ == "__main__":
    main()
