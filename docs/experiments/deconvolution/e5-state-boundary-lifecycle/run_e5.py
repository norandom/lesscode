"""E5 acceptance arm against a candidate root: `run_e5.py ROOT [--round N] [--deadline S]`."""
import json, os, subprocess, sys, time
from pathlib import Path

HERE = Path(__file__).resolve().parent
E4_DIR = HERE.parent / "e4-lifecycle"
E1B_DIR = HERE.parent / "e1b-facdrone-evidence-publication"
sys.path.insert(0, str(E1B_DIR))
sys.path.insert(0, str(E4_DIR))
import run_arm  # noqa: E402

SEEDS = E4_DIR / "results" / "seeds.json"
PROBE = "import facdrone.shells.lifecycle_store as m; print(m.__file__)"


def groups(root: Path) -> list[tuple[str, list[str]]]:
    sel = json.loads(SEEDS.read_text(encoding="utf-8"))
    pick = lambda pre, xs: [x.removeprefix(pre) for x in xs if x.startswith(pre)]  # noqa: E731
    out = [(".", pick("", [f for f in sel["seeds"] if f.startswith("tests/")]))]
    out += [(f"packages/facdrone-{p}", pick(f"packages/facdrone-{p}/", sel["seeds"])) for p in run_arm.PACKAGES]
    out = [
        (cwd, args + [f"--deselect={n}" for n in pick("" if cwd == "." else cwd + "/", sel["deselect"]) if cwd != "." or n.startswith("tests/")] if args else [])
        for cwd, args in out
    ]
    out[0][1].extend(["-p", "facdrone_testing.fixtures", (HERE / "test_e5_replay.py").as_posix()])
    return [g for g in out if g[1]]


def run(root: Path, deadline: float | None) -> int:
    start, gs = time.monotonic(), groups(root)
    deadline = None if deadline is None else start + deadline
    env = dict(
        os.environ,
        OPENBLAS_NUM_THREADS="1",
        OMP_NUM_THREADS="1",
        MKL_NUM_THREADS="1",
        PYTHONWARNINGS="ignore",
        E4_SOURCE_ROOT=str(root),
        E1B_SOURCE_ROOT=str(root),
        PYTHONPATH=os.pathsep.join(map(str, (root / "src", HERE, E4_DIR, run_arm.HERE, run_arm.EXTRAS, run_arm.E2))),
    )
    probe = subprocess.run([str(run_arm.VENV), "-c", PROBE], env=env, capture_output=True, text=True)
    if not Path(probe.stdout.strip() or "/").resolve().is_relative_to(root.resolve()):
        print(f"run_e5: INVALID scope not imported from {root}: {probe.stdout}{probe.stderr}")
        return 2
    for cwd, args in gs:
        cmd = [
            str(run_arm.VENV), "-m", "pytest", "-q", "-c", "pyproject.toml", "--rootdir", ".",
            "-p", "no:cacheprovider", "-p", "e2_winshim", "-x", "--timeout=600",
            "-m", run_arm.MARKS, *args
        ]
        t0 = time.perf_counter()
        code = run_arm._call(cmd, deadline, cwd=root / cwd, env=env)
        print(f"run_e5 {cwd}: exit {code} in {time.perf_counter() - t0:.1f}s", flush=True)
        if code not in (0, 5):
            return 1
    return 0


if __name__ == "__main__":
    a = sys.argv[1:]
    opts = dict(zip(a[1::2], a[2::2]))
    root, rnd = Path(a[0]).resolve(), opts.get("--round", "0")
    os.environ["E5_ROUND"] = rnd
    os.environ["E4_ROUND"] = rnd
    l2, t0 = HERE / "results" / f"sql-divergences-round{rnd}.json", time.time()
    gs = groups(root)
    print("run_e5: groups " + json.dumps([[c, sum(1 for x in g if x.endswith(".py"))] for c, g in gs]), flush=True)
    code = run(root, float(opts["--deadline"]) if "--deadline" in opts else None)
    rep = json.loads(l2.read_text(encoding="utf-8")) if l2.exists() else {}
    want = json.loads(Path(rep["golden"]).read_text(encoding="utf-8"))["meta"]["sequences"] if rep.get("golden") else None
    fresh = rep.get("written", 0) > t0 and want is not None and rep.get("sequences") == want
    print(
        f"run_e5: exit {code}; replay file fresh={fresh} sequences={rep.get('sequences')}/{want} operations={rep.get('operations')} level2_ops={rep.get('level2_ops')}",
        flush=True,
    )
    sys.exit(code if code else (0 if fresh else 1))
