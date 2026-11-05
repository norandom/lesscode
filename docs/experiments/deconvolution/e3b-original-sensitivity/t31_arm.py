"""deconvolution-loop task 3.1: E3's T4 sensitivity procedure (t4_arm.py) on the ORIGINAL's publication module.
Same seed, sample size, sorted-key rule, acceptance arm (scope probe, R16, the 19 importing seed files with
their 6 deselections, the 64-sequence replay), deadline rule, classification and restore-and-verify; only the
roots differ: reference .runs/e2-facdrone/source (frozen, read only), mutant copy .runs/e3-facdrone/orig-mut.
Inner commands (run under the receipt by the drivers):
  t31_arm.py sample SESSION          (cosmic-ray's interpreter) 30 specs -> results/t31-mutant-sample.json
  t31_arm.py arm [--deadline S]      the arm on orig-mut; 0 survived, 1 killed, 2 + "INVALID scope" incompetent,
                                     3 replay file not fresh (apparatus fault)
  t31_arm.py mutant K [--deadline S] apply sample K to orig-mut, arm, restore from the frozen original, verify
Drivers (guard.py first, then receipt_atomic into results/receipt.json, thread variables pinned to 1):
  t31_arm.py prepare                 copytree the frozen original to orig-mut if absent; verify five files
  t31_arm.py cr-init                 cosmic-ray 8.7.0 init cosmic-ray-t31.toml -> .runs/e3-facdrone/t31.sqlite
  t31_arm.py draw                    `sample` through uv's cosmic-ray interpreter; overlap with E3's sample
  t31_arm.py base                    unmutated arm, base 1200 s scaled (as t4-base)
  t31_arm.py dry K [K ...]           mutants K: deadline scaled(max(120, 5 x t31-base seconds)); classify;
                                     restore and verify by sha256; a timeout at factor > 1.5 reruns once when idle
Driver exit 3: the guard refused (host busy); 4: a copy failed its hash check."""
from __future__ import annotations
import difflib, hashlib, json, os, random, re, shutil, sys, time
from pathlib import Path
HERE = Path(__file__).resolve().parent
DEC, E3 = HERE.parent, HERE.parent / "e3-evidence-regeneration"
sys.path[:0] = [str(E3), str(DEC)]
from t4_arm import FILES, REL  # noqa: E402  (module level only reads public-surface.json)

LESSCODE = HERE.parents[3]
VENV = LESSCODE / ".runs/e2-facdrone/venv/Scripts/python.exe"
FROZEN, ORIG_MUT = LESSCODE / ".runs/e2-facdrone/source", LESSCODE / ".runs/e3-facdrone/orig-mut"
SESSION, TOML = LESSCODE / ".runs/e3-facdrone/t31.sqlite", HERE / "cosmic-ray-t31.toml"
RES = HERE / "results"
RECEIPT, LOGS, SAMPLE = RES / "receipt.json", RES / "logs", RES / "t31-mutant-sample.json"
SEED, N, ROUND = 20261104, 30, "t31"  # E3's seed and size; ROUND names the replay's Level 2 file
FIVE = tuple(f"src/facdrone/shells/{m}.py" for m in ("decision_evidence_publication", "decision_evidence_store",
             "decision_evidence_reader", "decision_input_slice_store", "decision_slice_publisher"))  # as missing_tests_check.py
PINS = {"OPENBLAS_NUM_THREADS": "1", "OMP_NUM_THREADS": "1", "MKL_NUM_THREADS": "1"}
APPARATUS = (HERE / "t31_arm.py", TOML, E3 / "t4_arm.py", E3 / "run_e3.py", E3 / "test_e3_replay.py",
             DEC / "e1b-facdrone-evidence-publication/run_arm.py", DEC / "guard.py", DEC / "receipt_atomic.py", DEC / "adaptive.py")
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()  # noqa: E731


def draw(keys, seed: int = SEED, n: int = N) -> list:
    """E3's rule: n of the sorted spec keys with random.Random(seed), returned sorted."""
    return sorted(random.Random(seed).sample(sorted(keys), n))


def hunks_of(src: str, mutated: str) -> tuple[list, list]:
    a, b = src.split("\n"), mutated.split("\n")
    hunks = [(i1, i2, b[j1:j2]) for t, i1, i2, j1, j2 in difflib.SequenceMatcher(None, a, b, autojunk=False).get_opcodes() if t != "equal"]
    return hunks, [a[h[0]:h[1]] for h in hunks]


def apply_hunks(path: Path, hunks) -> None:
    lines = path.read_bytes().decode("utf-8").split("\n")
    for i1, i2, new in reversed(hunks):
        lines[i1:i2] = new
    path.write_bytes("\n".join(lines).encode("utf-8"))


def restore(path: Path, ref: Path) -> bool:
    shutil.copy2(ref, path)
    return sha(path) == sha(ref)


def classify(code, log: str) -> tuple[str, str]:
    """(outcome, first failing reason) as in E3: exit 0 survived, nonzero killed, 2 with INVALID scope incompetent."""
    if code == 0:
        return "survived", ""
    if code == 2 and "INVALID scope" in log:
        return "incompetent", "scope not imported from the mutant copy"
    if code == 3:
        return "apparatus-fault", "replay file not fresh"
    if code in ("timeout", 124) or "deadline exceeded" in log:
        return "killed", "timeout"
    if "R16 harness differs" in log:
        return "killed", "R16 harness differs"
    m = re.search(r"^FAILED (\S+)", log, re.M) or re.search(r"^ERROR (\S+)", log, re.M) or re.search(r"run_arm B (\S+): exit (\d+)", log)
    return "killed", (m.group(0).strip() if m else f"exit {code}")


def five(root: Path) -> dict:
    return {f: [sha(root / f), sha(FROZEN / f)] for f in FIVE}


# ---- inner commands ------------------------------------------------------------------------------------------

def sample(session: str) -> int:  # t4_arm.sample with the original's roots and the reference hash
    sys.path.insert(0, str(DEC / "e1b-facdrone-evidence-publication"))
    import sample_mutants
    from cosmic_ray.mutating import mutate_code
    from cosmic_ray.plugins import get_operator
    assert sha(ORIG_MUT / REL) == sha(FROZEN / REL), "orig-mut copy differs from the frozen original"
    keys, src, out = sample_mutants.specs(Path(session)), (ORIG_MUT / REL).read_bytes().decode("utf-8"), []
    assert {k[0] for k in keys} == {REL}, f"population is not the publication module alone: {sorted({k[0] for k in keys})}"
    for _m, op, args, occ, row, col in draw(keys):
        kw = json.loads(args) if args else {}  # stored JSON-encoded twice (E1b)
        mutated = mutate_code(src, get_operator(op)(**(json.loads(kw) if isinstance(kw, str) else kw)), occ)
        hunks, old = hunks_of(src, mutated)
        out.append(dict(operator=op, args=args, occurrence=occ, row=row, col=col, hunks=hunks, old=old))
    SAMPLE.write_text(json.dumps({"seed": SEED, "population": len(keys), "module": REL, "original_sha256": sha(FROZEN / REL),
                                  "reference": str(FROZEN), "seed_files": FILES, "sample": out}, indent=1), encoding="utf-8")
    print(f"t31_arm: population {len(keys)}, sampled {len(out)}", flush=True)
    return 0


def arm(deadline: str | None, tag: str = "base") -> int:  # t4_arm.arm on orig-mut
    import run_e3  # patches run_arm (E3 replay target); the seed selection is replaced here as in t4_arm

    def seeds(root: Path) -> tuple[list[str], list[str]]:
        run_e3.run_arm.SELECTED[:] = FILES
        return FILES, [n for n in run_e3.SEL["deselect"] if n.split("::")[0] in FILES]
    run_e3.run_arm.seeds = seeds
    os.environ.update(E3_ROUND=ROUND, PYTHONDONTWRITEBYTECODE="1")
    l2, t0 = E3 / "results" / f"sql-divergences-round{ROUND}.json", time.time()  # path fixed by the frozen replay
    print(f"t31_arm: root {ORIG_MUT}; {len(FILES)} seed files {FILES}", flush=True)
    try:
        code = run_e3.run_arm.run("B", ORIG_MUT, None, deadline)
        rep = json.loads(l2.read_text(encoding="utf-8")) if l2.exists() else {}
        fresh = rep.get("written", 0) > t0 and rep.get("sequences") == 64
        print(f"t31_arm: run_arm exit {code}; replay fresh={fresh} sequences={rep.get('sequences')}", flush=True)
    finally:  # keep E3's results directory unchanged: the Level 2 file moves here
        if l2.exists():
            shutil.move(l2, RES / f"sql-divergences-t31-{tag}.json")
    return code if code else (0 if fresh else 3)


def mutant(k: int, deadline: str | None) -> int:  # t4_arm.mutant, restoring from the frozen original
    s = json.loads(SAMPLE.read_text(encoding="utf-8"))["sample"][k]
    assert sha(ORIG_MUT / REL) == sha(FROZEN / REL), "orig-mut copy not restored"
    apply_hunks(ORIG_MUT / REL, s["hunks"])
    print(f"t31_arm: mutant {k} {s['operator']} occ {s['occurrence']} line {s['row']}: {s['old']} -> {[h[2] for h in s['hunks']]}", flush=True)
    try:
        return arm(deadline, f"m{k}")
    finally:
        ok = restore(ORIG_MUT / REL, FROZEN / REL)
        print(f"t31_arm: restored, identical={ok}", flush=True)
        if not ok:
            raise SystemExit(4)


# ---- drivers ---------------------------------------------------------------------------------------------------

def _guard(label: str) -> None:
    import guard
    if guard.guard(label, RECEIPT) == guard.STOP:
        print(f"{label}: BLOCKED, guard refused (host busy after two waits)", flush=True)
        raise SystemExit(3)


def _run(label: str, cmd: list[str], **kw):
    import receipt_atomic
    code = receipt_atomic.run(RECEIPT, label, cmd, env=PINS, **kw)
    entry = [c for c in json.loads(RECEIPT.read_text(encoding="utf-8"))["commands"] if c["label"] == label][-1]
    return code, entry


def _verify(label: str, root: Path = ORIG_MUT) -> bool:
    import receipt_atomic
    h = five(root)
    ok = all(a == b for a, b in h.values())
    receipt_atomic.add_note(RECEIPT, label, {"root": str(root), "reference": str(FROZEN), "identical": ok, "sha256": h})
    print(f"{label}: identical={ok}", flush=True)
    return ok


def step(label: str, base_s: float, inner: list[str]) -> dict:
    """Guard, scaled deadline, one receipt entry; a timeout at factor > 1.5 waits for idle and reruns once."""
    import adaptive, receipt_atomic
    _guard(label)
    lab = label
    while True:
        secs, info = adaptive.scaled(base_s)
        receipt_atomic.add_note(RECEIPT, f"{lab}-deadline", {"base_s": base_s, "scaled_s": secs, "factor": info["factor"], "probe": info})
        code, entry = _run(lab, [str(VENV), str(HERE / "t31_arm.py"), *inner, "--deadline", str(secs)], timeout=secs + 120)
        log = (LOGS / f"{lab}.log").read_text(encoding="utf-8", errors="replace")
        timed_out = code == "timeout" or "deadline exceeded" in log
        if not (timed_out and info["factor"] > 1.5) or lab.endswith("-rerun"):
            return dict(label=lab, exit=code, seconds=entry["seconds"], factor=info["factor"], scaled_s=secs, log=log)
        receipt_atomic.add_note(RECEIPT, f"{label}-wait", adaptive.wait_for_idle(1800))
        lab = label + "-rerun"


def prepare() -> int:
    import receipt_atomic
    receipt_atomic.add_fact(RECEIPT, {"key": "t31-apparatus", "frozen_original": str(FROZEN), "orig_mut": str(ORIG_MUT),
                                      "sha256": {p.relative_to(LESSCODE).as_posix(): sha(p) for p in APPARATUS}})
    if not ORIG_MUT.exists():
        _guard("t31-copy")
        _run("t31-copy", [str(VENV), "-c", f"import shutil; shutil.copytree(r'{FROZEN}', r'{ORIG_MUT}', symlinks=False)"])
    return 0 if _verify("t31-copy-verify") else 4


def _uv(label: str, args: list[str], **kw) -> int:
    _guard(label)
    code, _ = _run(label, [shutil.which("uv.exe") or "uv.exe", "tool", "run", "--python", "3.13", "--from", "cosmic-ray==8.7.0", *args], **kw)
    return 0 if code == 0 else 1


def draw_sample() -> int:
    import receipt_atomic
    if _uv("t31-sample", ["python", str(HERE / "t31_arm.py"), "sample", str(SESSION)]):
        return 1
    ours = json.loads(SAMPLE.read_text(encoding="utf-8"))
    e3 = json.loads((E3 / "results/t4-mutant-sample.json").read_text(encoding="utf-8"))
    both = lambda f: sorted({f(s) for s in ours["sample"]} & {f(s) for s in e3["sample"]})  # noqa: E731
    ov = {"population": ours["population"], "e3_population": e3["population"], "sample_file_sha256": sha(SAMPLE),
          "operators_in_both": both(lambda s: s["operator"]), "lines_in_both": both(lambda s: s["row"]),
          "operator_line_in_both": both(lambda s: (s["operator"], s["row"])),
          "note": "different modules; overlap is informational only"}
    receipt_atomic.add_note(RECEIPT, "t31-sample-overlap", ov)
    print(json.dumps(ov), flush=True)
    return 0


def base() -> int:
    if not _verify("t31-base-pre"):
        return 4
    r = step("t31-base", 1200, ["arm"])
    print(json.dumps({k: r[k] for k in ("label", "exit", "seconds", "factor", "scaled_s")}), flush=True)
    return 0 if r["exit"] == 0 else 1


def dry(ks: list[int]) -> int:
    import receipt_atomic
    cmds = json.loads(RECEIPT.read_text(encoding="utf-8"))["commands"]
    passed = [c for c in cmds if c["label"] in ("t31-base", "t31-base-rerun") and c["exit"] == 0]
    if not passed:
        print("t31_arm: no passing t31-base in the receipt; run `base` first", flush=True)
        return 2
    base_s = round(max(120.0, 5 * passed[-1]["seconds"]), 1)
    specs = json.loads(SAMPLE.read_text(encoding="utf-8"))["sample"]
    for k in ks:
        if not _verify(f"t31-m{k}-pre"):
            return 4
        r = step(f"t31-m{k}", base_s, ["mutant", str(k)])
        if sha(ORIG_MUT / REL) != sha(FROZEN / REL):  # the inner process was killed before its own restore
            shutil.copy2(FROZEN / REL, ORIG_MUT / REL)
        restored = _verify(f"t31-m{k}-restore")
        outcome, reason = classify(r["exit"], r["log"])
        s = specs[k]
        row = {"k": k, "label": r["label"], "operator": s["operator"], "row": s["row"], "col": s["col"], "occurrence": s["occurrence"],
               "exit": r["exit"], "seconds": r["seconds"], "factor": r["factor"], "deadline_base_s": base_s, "scaled_s": r["scaled_s"],
               "classification": outcome, "first_failing_reason": reason, "restore_identical": restored}
        receipt_atomic.add_note(RECEIPT, f"t31-m{k}-class", row)
        print(json.dumps(row), flush=True)
        if not restored:
            return 4
    return 0


if __name__ == "__main__":
    a = sys.argv[1:]
    if not a or a[0] in ("-h", "--help"):
        print(__doc__)
        sys.exit(0 if a else 2)
    dl = a[a.index("--deadline") + 1] if "--deadline" in a else None
    cmds = {"sample": lambda: sample(a[1]), "arm": lambda: arm(dl), "mutant": lambda: mutant(int(a[1]), dl),
            "prepare": prepare, "cr-init": lambda: _uv("t31-cr-init", ["cosmic-ray", "init", str(TOML), str(SESSION)], cwd=str(ORIG_MUT)),
            "draw": draw_sample, "base": base, "dry": lambda: dry([int(x) for x in a[1:]])}
    sys.exit(cmds[a[0]]() if a[0] in cmds else f"unknown command {a[0]}")
