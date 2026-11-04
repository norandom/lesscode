"""E1c's run_e1c_arm.py (E1b's run_arm.py) with the replay target swapped for test_e1c2_replay.py.
  run_e1c2_arm.py B [--root DIR] [--cov DIR] [--deadline S|auto]   (cwd = the mutant copy)"""
import os, sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "e1c-concolic-rung"))
import run_e1c_arm as R  # noqa: E402
R.E1C_TEST = (HERE / "test_e1c2_replay.py").as_posix()
if __name__ == "__main__":
    a = sys.argv[1:]
    opts = dict(zip(a[1::2], a[2::2]))
    cov = Path(opts["--cov"]).resolve() if "--cov" in opts else None
    sys.exit(R.run_arm.run(a[0], Path(opts.get("--root", os.getcwd())).resolve(), cov, opts.get("--deadline")))
