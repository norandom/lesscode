"""E6 acceptance test runner against a specimen root: `run_e6.py ROOT [--round N]`."""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import time

HERE = Path(__file__).resolve().parent
REPO_ROOT = HERE.parents[3]
VENV = REPO_ROOT / ".runs" / "e2-facdrone" / "venv" / "Scripts" / "python.exe"
SHIM = REPO_ROOT / "docs" / "experiments" / "deconvolution" / "e2-facdrone-edge-census"

TEST_SUITES = [
    ("packages/facdrone-reporting", [
        "tests/test_store_faults.py",
        "tests/test_record_simulation_request.py",
        "tests/test_export_on_request.py",
        "tests/test_factor_basket_request.py",
        "tests/test_canonical_hydration.py",
    ]),
    ("packages/facdrone-kernel", [
        "tests/test_canonical.py",
        "tests/test_canonical_serialization.py",
    ]),
    (".", [
        "tests/unit/test_cli_sim.py",
        "tests/unit/test_legacy_run_read_path.py",
    ]),
]

PROBE = "import facdrone_reporting.shells.reporting_request as m; print(m.__file__)"


def run(root: Path) -> int:
    env = dict(
        os.environ,
        OPENBLAS_NUM_THREADS="1",
        OMP_NUM_THREADS="1",
        MKL_NUM_THREADS="1",
        PYTHONWARNINGS="ignore",
        E6_SOURCE_ROOT=str(root),
        PYTHONPATH=os.pathsep.join(map(str, (
            root / "src",
            root / "packages" / "facdrone-reporting",
            root / "packages" / "facdrone-kernel",
            SHIM,
            HERE,
        ))),
    )

    probe = subprocess.run([str(VENV), "-c", PROBE], env=env, capture_output=True, text=True)
    probe_out = probe.stdout.strip()
    if not probe_out or not Path(probe_out).resolve().is_relative_to(root.resolve()):
        print(f"run_e6: INVALID scope not imported from {root}: {probe.stdout} {probe.stderr}")
        return 2

    for cwd_rel, test_files in TEST_SUITES:
        cwd = root / cwd_rel
        cmd = [
            str(VENV), "-m", "pytest", "-q", "-c", "pyproject.toml",
            "-p", "no:cacheprovider", "-p", "e2_winshim",
            *test_files
        ]
        t0 = time.perf_counter()
        proc = subprocess.run(cmd, cwd=str(cwd), env=env, capture_output=True, text=True)
        elapsed = time.perf_counter() - t0
        print(f"run_e6 {cwd_rel}: exit {proc.returncode} in {elapsed:.2f}s")
        if proc.returncode != 0:
            print(proc.stdout)
            print(proc.stderr)
            return proc.returncode

    return 0


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Run E6 acceptance test battery")
    parser.add_argument("root", type=Path, help="Root directory of specimen")
    parser.add_argument("--round", type=str, default="0", help="Round identifier")
    args = parser.parse_args()

    exit_code = run(args.root.resolve())
    print(f"run_e6 final exit: {exit_code}")
    sys.exit(exit_code)
