"""E4 acceptance arm against a root (deconvolution-loop task 5.2): `run_e4.py ROOT [--round N] [--deadline S]`.
Scope-import probe (lifecycle_store must import from ROOT), then the seed test files per pytest group (serial, -x,
-p e2_winshim, marks "not port_baseline and not live_broker", ROOT/src first on PYTHONPATH) with the baseline
deselections, then the replay target test_e4_replay.py. Seeds and deselections come from results/testinfra.json as
results/seeds.json froze them (`run_e4.py seeds` writes it: direct referencing test files minus gate-ignored and
collection-error files; node deselections applicable to them). No R16 harness: it belongs to the E3 scope. Exit 0 only
if every group passes and the replay wrote this run's Level 2 file covering every golden sequence."""
import json, os, subprocess, sys, time
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "e1b-facdrone-evidence-publication"))
import run_arm  # noqa: E402  (E1b: _call with the deadline kill, MARKS, PACKAGES, VENV, EXTRAS, E2)
SEEDS, TESTINFRA = HERE / "results" / "seeds.json", HERE / "results" / "testinfra.json"
PROBE = "import facdrone.shells.lifecycle_store as m; print(m.__file__)"


def seeds() -> dict:
    d = json.loads(TESTINFRA.read_text(encoding="utf-8"))
    s, x = d["seeds"], d["deselections"]
    dropped = {f: "gate --ignore" for f in s["ignored_by_gates"]} | {f: "collection error on this host (fcntl)" for f in x["collection_errors"] if f in s["direct"]}
    files = [f for f in s["direct"] if f not in dropped]
    bad = sorted(n for v in x["resolved_ids"].values() for n in v if n.split("::")[0] in files)
    out = {"seeds": files, "dropped": dropped, "deselect": bad, "marks": run_arm.MARKS, "testinfra_sha256": __import__("hashlib").sha256(TESTINFRA.read_bytes()).hexdigest()}
    SEEDS.write_text(json.dumps(out, indent=1), encoding="utf-8")
    return out


def groups(root: Path) -> list[tuple[str, list[str]]]:
    """(cwd relative to root, pytest args): the root suite with the replay, then each satellite package."""
    sel = json.loads(SEEDS.read_text(encoding="utf-8"))
    pick = lambda pre, xs: [x.removeprefix(pre) for x in xs if x.startswith(pre)]  # noqa: E731
    out = [(".", pick("", [f for f in sel["seeds"] if f.startswith("tests/")]))]
    out += [(f"packages/facdrone-{p}", pick(f"packages/facdrone-{p}/", sel["seeds"])) for p in run_arm.PACKAGES]
    out = [(cwd, args + [f"--deselect={n}" for n in pick("" if cwd == "." else cwd + "/", sel["deselect"]) if cwd != "." or n.startswith("tests/")] if args else [])
           for cwd, args in out]
    out[0][1].extend(["-p", "facdrone_testing.fixtures", (HERE / "test_e4_replay.py").as_posix()])  # outside tests/: name the fixture plugin
    return [g for g in out if g[1]]


def run(root: Path, deadline: float | None) -> int:
    start, gs = time.monotonic(), groups(root)
    deadline = None if deadline is None else start + deadline
    env = dict(os.environ, OPENBLAS_NUM_THREADS="1", OMP_NUM_THREADS="1", MKL_NUM_THREADS="1", PYTHONWARNINGS="ignore",
               E4_SOURCE_ROOT=str(root), E1B_SOURCE_ROOT=str(root),
               PYTHONPATH=os.pathsep.join(map(str, (root / "src", HERE, run_arm.HERE, run_arm.EXTRAS, run_arm.E2))))
    probe = subprocess.run([str(run_arm.VENV), "-c", PROBE], env=env, capture_output=True, text=True)
    if not Path(probe.stdout.strip() or "/").resolve().is_relative_to(root.resolve()):
        print(f"run_e4: INVALID scope not imported from {root}: {probe.stdout}{probe.stderr}")
        return 2
    for cwd, args in gs:
        cmd = [str(run_arm.VENV), "-m", "pytest", "-q", "-c", "pyproject.toml", "--rootdir", ".", "-p", "no:cacheprovider",
               "-p", "e2_winshim", "-x", "--timeout=600", "-m", run_arm.MARKS, *args]
        t0 = time.perf_counter()
        code = run_arm._call(cmd, deadline, cwd=root / cwd, env=env)
        print(f"run_e4 {cwd}: exit {code} in {time.perf_counter() - t0:.1f}s", flush=True)
        if code not in (0, 5):  # 5: every test in the group deselected
            return 1
    return 0


if __name__ == "__main__":
    a = sys.argv[1:]
    if a[0] == "seeds":
        print(json.dumps(seeds(), indent=1))
        sys.exit(0)
    opts = dict(zip(a[1::2], a[2::2]))
    root, rnd = Path(a[0]).resolve(), opts.get("--round", "0")
    os.environ["E4_ROUND"] = rnd
    l2, t0 = HERE / "results" / f"sql-divergences-round{rnd}.json", time.time()
    gs = groups(root)
    print("run_e4: groups " + json.dumps([[c, sum(1 for x in g if x.endswith(".py"))] for c, g in gs]), flush=True)
    code = run(root, float(opts["--deadline"]) if "--deadline" in opts else None)
    rep = json.loads(l2.read_text(encoding="utf-8")) if l2.exists() else {}
    want = json.loads(Path(rep["golden"]).read_text(encoding="utf-8"))["meta"]["sequences"] if rep.get("golden") else None
    fresh = rep.get("written", 0) > t0 and want is not None and rep.get("sequences") == want
    print(f"run_e4: exit {code}; replay file fresh={fresh} sequences={rep.get('sequences')}/{want} operations={rep.get('operations')} level2_ops={rep.get('level2_ops')}", flush=True)
    sys.exit(code if code else (0 if fresh else 1))
