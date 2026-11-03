"""Run one E1b arm against a source root (the cosmic-ray test command; exit 0 = survived).

  run_arm.py A|B [--root DIR] [--cov DIR] [--deadline S|auto]  seed tests (A); seeds + trace replay (B)
  run_arm.py all --cov DIR      whole gate tier with per-file coverage contexts (seed confirmation)
  run_arm.py seeds DIR | report DIR [ROOT] | r16-record     seed list + selection map; coverage; R16
  run_arm.py time A|B | cap SECONDS -- CMD...   per-set deadlines (amendment 7); wall-clock cap (amendment 6)
Both arms first run the unchanged R16 harness and compare it with results/r16-baseline.json.
Amendment 7 (owner's per-line amendment): a mutated root runs only the seed files whose coverage
context executes a line the mutant changes (diff against the frozen copy); unmutated, all seeds.
Root defaults to the cwd (cosmic-ray's mutated copy). `-p run_arm` labels coverage contexts.
"""
import hashlib, json
import os, re, subprocess, sys, time
from pathlib import Path

HERE = Path(__file__).resolve().parent
LESSCODE = HERE.parents[3]
VENV = LESSCODE / ".runs/e2-facdrone/venv/Scripts/python.exe"
EXTRAS = LESSCODE / ".runs/e1b-facdrone/extras"
E2 = HERE.parent / "e2-facdrone-edge-census"
R16_BASELINE = Path(os.environ.get("E1B_R16_BASELINE", HERE / "results" / "r16-baseline.json"))
SELECTION = HERE / "results" / "selection-map.json"  # frozen by `seeds` (amendments 3 and 7)
DEADLINES = HERE / "results" / "deadlines.json"  # written by `time` (amendment 7)
SELECTED: list[str] = []
MARKS = "not port_baseline and not live_broker"
PACKAGES = ("research", "reporting", "dash", "kernel", "factor-kit", "factor-comm-term-structure", "factor-short-reversal")


def pytest_runtest_protocol(item, nextitem):  # plugin hook: one coverage context per test file
    import coverage
    if (cov := coverage.Coverage.current()) is not None:
        cov.switch_context(Path(item.path).resolve().as_posix())


def seeds(root: Path) -> tuple[list[str], list[str]]:
    """Amendment 7: seed files executing a changed line of a mutated module, and baseline failures in them."""
    if "E1B_SEEDS" in os.environ:  # smoke runs only
        return [f for f in os.environ["E1B_SEEDS"].split(",") if f.strip()], []
    import scope
    sel = json.loads(SELECTION.read_text(encoding="utf-8"))
    changed = scope.record(root)["changed"]  # hashes against the R15 manifest: the mutated module(s)
    lines = {m: scope.changed_lines((scope.FROZEN / m).read_text(encoding="utf-8"), (root / m).read_text(encoding="utf-8"))
             for m in changed} if changed and set(changed) <= set(scope.MUTATED) else {}
    if not changed and os.environ.get("E1B_LINES"):  # timing a mutant's set on the frozen copy
        m, ls = os.environ["E1B_LINES"].split(":")
        lines = {m: [int(x) for x in ls.split(",") if x]}
    files = sorted({f for m, ls in lines.items() for n in ls for f in sel["lines"][m].get(str(n), [])}) \
        if lines or changed else sel["seeds"]  # an unknown change selects nothing: the R16 check only
    SELECTED[:] = files
    tag = "uncovered (no seed test executes a changed line)" if (lines or changed) and not files else f"{len(files)} selected seed files"
    print(f"run_arm: changed {changed or 'none'} lines {lines or '-'}; {tag}", flush=True)
    return files, [n for n in sel["deselect"] if n.split("::")[0] in files]


def set_key(arm: str) -> str:
    return f"{arm}:" + hashlib.sha256("\n".join(SELECTED).encode()).hexdigest()[:16]


def groups(arm: str, root: Path) -> list[tuple[str, list[str]]]:
    """(cwd relative to root, pytest args): the root suite, then each satellite package."""
    if arm == "all":
        out = [(".", ["tests"])] + [(f"packages/facdrone-{p}", ["--ignore=tests/test_lifecycle_full_loop_db.py"]
                                      if p == "research" else []) for p in PACKAGES]
        return [g for g in out if os.environ.get("E1B_GROUP", g[0]) == g[0]]  # amendment 5: one-group rerun
    files, bad = seeds(root)
    pick = lambda pre, xs: [x.removeprefix(pre) for x in xs if x.startswith(pre)]
    out = [(".", pick("", [f for f in files if f.startswith("tests/")]))]
    for p in PACKAGES:
        prefix = f"packages/facdrone-{p}/"
        out.append((prefix.rstrip("/"), pick(prefix, files)))
    out = [(cwd, args + [f"--deselect={n}" for n in pick("" if cwd == "." else cwd + "/", bad)
                         if cwd != "." or n.startswith("tests/")] if args else []) for cwd, args in out]
    if arm == "B":  # the replay lives outside tests/, so name the fixture plugin explicitly;
        # safe only because PYTHONPATH puts ROOT/src first (a -p import precedes ini pythonpath)
        out[0][1].extend(["-p", "facdrone_testing.fixtures", (HERE / "test_e1b_replay.py").as_posix()])
    return [g for g in out if g[1]]


def _call(cmd, deadline, **kw) -> int:
    """Run cmd; past the arm deadline kill its whole tree (cosmic-ray cannot on Windows)."""
    proc = subprocess.Popen(cmd, **kw)
    try:
        return proc.wait(timeout=None if deadline is None else max(1.0, deadline - time.monotonic()))
    except subprocess.TimeoutExpired:
        subprocess.run(["taskkill", "/F", "/T", "/PID", str(proc.pid)], capture_output=True)
        print("run_arm: deadline exceeded (counted as killed: timeout)")
        return 124


def run(arm: str, root: Path, cov: Path | None, deadline: str | None) -> int:
    import scope
    start, gs = time.monotonic(), groups(arm, root)
    if deadline == "auto":  # amendment 7: the measured deadline of exactly this selected set
        table = json.loads(DEADLINES.read_text(encoding="utf-8")) if DEADLINES.exists() else {}
        if set_key(arm) not in table:
            print(f"run_arm: INVALID no measured deadline for {set_key(arm)}")
            return 2
        deadline = table[set_key(arm)]["deadline"]
    deadline = None if deadline is None else start + float(deadline)
    env = dict(os.environ, OPENBLAS_NUM_THREADS="1", OMP_NUM_THREADS="1", MKL_NUM_THREADS="1",
               E1B_SOURCE_ROOT=str(root), PYTHONWARNINGS="ignore",
               PYTHONPATH=os.pathsep.join(map(str, (root / "src", HERE, EXTRAS, E2))))
    probe = subprocess.run([str(VENV), "-c", "import facdrone.shells.cppi.state_repository as m; print(m.__file__)"],
                           env=env, capture_output=True, text=True)
    if not Path(probe.stdout.strip() or "/").resolve().is_relative_to(root.resolve()):
        print(f"run_arm: INVALID scope not imported from {root}: {probe.stdout}{probe.stderr}")
        return 2  # fail closed; cosmic-ray counts 2 as killed, so analysis voids INVALID outputs
    r16 = subprocess.run([str(VENV), "-B", str(scope.R16_CHECK), str(root)], env=env, capture_output=True, text=True)
    if r16.returncode or r16.stdout != R16_BASELINE.read_text(encoding="utf-8"):
        print(f"run_arm {arm}: R16 harness differs from its baseline (exit {r16.returncode})")
        return 1
    py = [str(VENV)]
    if cov:
        cov.mkdir(parents=True, exist_ok=True)
        include = ",".join(str(root / f) for f in scope.FILES)
        py += ["-m", "coverage", "run", "--branch", "-p", f"--data-file={cov / '.coverage'}", f"--include={include}"]
    failed = 0
    for cwd, args in gs:
        extra = ["-p", "run_arm"] if cov else ["-x"]
        if arm == "all":  # baseline census: every group runs; failing node ids go to <group>.fails
            label = "root" if cwd == "." else cwd.replace("/", "~")
            env["E1B_FAILS"] = str(cov / f"{label}.fails")
            extra += [f"--junitxml={cov / label}.xml", "--continue-on-collection-errors"]  # amendment 5
        cmd = [*py, "-m", "pytest", "-q", "-c", "pyproject.toml", "--rootdir", ".",  # else D:/Source/lesscode's ini wins
               "-p", "no:cacheprovider", "-p", "e2_winshim", *extra, "--timeout=600", "-m", MARKS, *args]
        t0 = time.perf_counter()
        code = _call(cmd, deadline, cwd=root / cwd, env=env)
        print(f"run_arm {arm} {cwd}: exit {code} in {time.perf_counter() - t0:.1f}s", flush=True)
        if code not in (0, 5):  # 5: every test in the group deselected
            failed = 1
            if arm != "all":
                return 1
    return failed


def pytest_runtest_logreport(report):  # plugin hook: record baseline failures in the `all` run
    if report.failed and (p := os.environ.get("E1B_FAILS")):
        with open(p, "a", encoding="utf-8") as fh:
            fh.write(report.nodeid + "\n")


def seed_files(cov: Path) -> list[str]:
    import coverage, scope
    subprocess.run([str(VENV), "-m", "coverage", "combine", "--keep", f"--data-file={cov / '.coverage'}", str(cov)])
    data = coverage.CoverageData(basename=str(cov / ".coverage"))
    data.read()
    scoped = [f for f in data.measured_files() if any(f.replace("\\", "/").endswith(s) for s in scope.FILES)]
    root, hits, mods = scope.FROZEN.resolve(), [], {m: set() for m in scope.MUTATED}
    per_line = {m: {} for m in scope.MUTATED}  # amendment 7: module -> line -> seed files executing it
    for ctx in sorted(c for c in data.measured_contexts() if c and Path(c).resolve().is_relative_to(root)):
        data.set_query_contexts([f"^{re.escape(ctx)}$"])
        rel, hit = Path(ctx).resolve().relative_to(root).as_posix(), {f: data.lines(f) for f in scoped}
        hits += [rel] if any(hit.values()) else []
        for m in mods:  # amendment 3: a seed file is selected for m if it executes >= 1 line of m
            for f in (f for f, ls in hit.items() if ls and f.replace("\\", "/").endswith(m)):
                mods[m].add(rel)
                for n in hit[f]:
                    per_line[m].setdefault(str(n), set()).add(rel)
    bad = []  # baseline failures of the `all` run, as node ids relative to the frozen root
    for fails in cov.glob("*.fails"):
        pre = "" if fails.stem == "root" else fails.stem.replace("~", "/") + "/"
        bad += [pre + n for n in fails.read_text(encoding="utf-8").split("\n") if n]
    lines = {m: {n: sorted(v) for n, v in sorted(d.items(), key=lambda kv: int(kv[0]))} for m, d in per_line.items()}
    SELECTION.write_text(json.dumps({"seeds": sorted(set(hits)), "modules": {m: sorted(v) for m, v in mods.items()},
                                     "lines": lines, "deselect": sorted(set(bad))}, indent=1), encoding="utf-8")
    return sorted(set(hits))


def report(cov: Path, root: Path) -> dict:
    """coverage.py figures over all eight scope files, unexecuted files included at 0%."""
    import coverage, json, scope
    c = coverage.Coverage(data_file=str(cov / ".coverage"), branch=True)
    c.load()
    nums = {f: c._analyze(str(root / f)).numbers for f in scope.FILES}  # private, stable in 7.x
    files = {f: dict(statements=n.n_statements, missing=n.n_missing, branches=n.n_branches,
                     missing_branches=n.n_missing_branches) for f, n in nums.items()}
    t = {k: sum(v[k] for v in files.values()) for k in ("statements", "missing", "branches", "missing_branches")}
    t["coverage_py_percent"] = 100 * (t["statements"] - t["missing"] + t["branches"] - t["missing_branches"]) / (t["statements"] + t["branches"])
    t["branch_percent"] = 100 * (t["branches"] - t["missing_branches"]) / t["branches"]
    out = {"files": files, "total": t}
    (cov / "scope-coverage.json").write_text(json.dumps(out, indent=1))
    return out


if __name__ == "__main__":
    a = sys.argv[1:]
    if a[0] in ("report", "seeds", "r16-record"):
        import scope
        if a[0] == "report":  # after `coverage combine`
            print(report(Path(a[1]), Path(a[2]) if len(a) > 2 else Path(os.getcwd()))["total"])
        elif a[0] == "seeds":
            print("\n".join(seed_files(Path(a[1]))))
        else:
            R16_BASELINE.write_text(subprocess.run([str(VENV), "-B", str(scope.R16_CHECK), str(scope.FROZEN)],
                                                   capture_output=True, text=True, check=True).stdout, encoding="utf-8")
        sys.exit(0)
    if a[0] == "cap":  # amendment 6: hard wall-clock cap on a whole command (cosmic-ray exec)
        sys.exit(_call(a[a.index("--") + 1:], time.monotonic() + float(a[1])))
    if a[0] == "time":  # amendment 7: unmutated time of each distinct selected set of the sample
        import scope
        table = json.loads(DEADLINES.read_text(encoding="utf-8")) if DEADLINES.exists() else {}
        sample = json.loads((HERE / "results" / "mutant-sample.json").read_text(encoding="utf-8"))["sample"]
        for spec in [None] + [f"{s['module']}:{','.join(map(str, s['lines']))}" for s in sample]:
            os.environ.pop("E1B_LINES", None) if spec is None else os.environ.update(E1B_LINES=spec)
            groups(a[1], scope.FROZEN)
            if (key := set_key(a[1])) in table:
                continue
            t0 = time.perf_counter()
            if code := run(a[1], scope.FROZEN, None, None):
                print(f"run_arm: INVALID selected set {key} fails unmutated (exit {code})")
                sys.exit(1)
            t = time.perf_counter() - t0
            table[key] = {"files": len(SELECTED), "seconds": round(t, 1), "deadline": round(max(120.0, 5 * t), 1)}
            DEADLINES.write_text(json.dumps(table, indent=1), encoding="utf-8")
        sys.exit(0)
    opts = dict(zip(a[1::2], a[2::2]))
    if opts.get("--deadline") in ("0", "0.0"):
        print("run_arm: INVALID deadline 0: run sample_mutants.py timeout first")
        sys.exit(2)
    cov = Path(opts["--cov"]).resolve() if "--cov" in opts else None
    sys.exit(run(a[0], Path(opts.get("--root", os.getcwd())).resolve(), cov, opts.get("--deadline")))
