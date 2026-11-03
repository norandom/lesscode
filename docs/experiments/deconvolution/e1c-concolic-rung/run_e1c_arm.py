"""E1b's run_arm.py with the Arm B replay target swapped for test_e1c_replay.py (E1c sequences only).
  run_e1c_arm.py B [--root DIR] [--cov DIR] [--deadline S|auto]   (cwd = the mutant copy, as in E1b)
Seed selection, R16 check, baseline deselects and deadlines are E1b's, unchanged."""
import os, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "e1b-facdrone-evidence-publication"))
import run_arm  # noqa: E402

E1B_TEST, E1C_TEST = (run_arm.HERE / "test_e1b_replay.py").as_posix(), (HERE / "test_e1c_replay.py").as_posix()
_groups = run_arm.groups
run_arm.groups = lambda arm, root: [(cwd, [E1C_TEST if a == E1B_TEST else a for a in args]) for cwd, args in _groups(arm, root)]

if __name__ == "__main__":
    a = sys.argv[1:]
    opts = dict(zip(a[1::2], a[2::2]))
    cov = Path(opts["--cov"]).resolve() if "--cov" in opts else None
    sys.exit(run_arm.run(a[0], Path(opts.get("--root", os.getcwd())).resolve(), cov, opts.get("--deadline")))
