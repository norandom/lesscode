"""deconvolution-loop task 6.4: E4 sensitivity on both trees, t31_arm.py's procedure with the roots, the seven files and the
arm parameterised. TREE is `orig` (reference .runs/e2-facdrone/source, mutable copy .runs/e4-lifecycle/orig-mut, labels
e4o-*) or `cand` (reference .runs/e4-lifecycle/cand-turn3, copy .runs/e4-lifecycle/cand-mut, labels e4c-*). References are
only read; mutants live in the copies; restore is by copying the reference file back, verified by sha256 of all seven.
Inner commands (run under the receipt by the drivers; TAG names run_e4's Level 2 file results/sql-divergences-round<TAG>.json):
  e4_arm.py arm TREE --round TAG [--deadline S]       run_e4.py on the copy: 0 survived, 1 killed, 2 + "INVALID scope"
                                                      incompetent, 3 replay file not fresh (apparatus fault), 4 hash mismatch
  e4_arm.py mutant TREE K --round TAG [--deadline S]  apply sample K (t31_arm.apply_hunks, CRLF kept), arm, restore, verify
  e4_arm.py hand TREE J --round TAG [--deadline S]    git apply mutants/M<J>[-cand].patch, verify the recorded mutated sha256,
                                                      arm, restore, verify
Drivers (guard.py first, then receipt_atomic into results/receipt.json, numeric threads pinned to 1, serial):
  e4_arm.py prepare TREE            copytree reference -> copy if absent (no .git, __pycache__, .pytest_cache); verify seven
  e4_arm.py base TREE               unmutated arm `<pfx>-base`, base 1200 s scaled
  e4_arm.py dry TREE K [K ...]      sampled mutants `<pfx>-m<K>`: deadline scaled(max(120, 5 x base seconds of TREE))
  e4_arm.py hand-run TREE J [J ...] hand-written mutants `<pfx>-M<J>`: base 1200 s scaled
Per mutant: notes -pre, -deadline (the probe that set it), -restore, -class; a timeout at factor > 1.5 waits for idle and
reruns once as -rerun. Driver exit 3: the guard refused; 4: a copy failed its hash check (stop and report)."""
from __future__ import annotations
import json, re, shutil, subprocess, sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
DEC = HERE.parent
sys.path[:0] = [str(HERE), str(DEC), str(DEC / "e3b-original-sensitivity")]
import t31_arm  # noqa: E402  (E3b: draw, hunks_of, apply_hunks, restore, sha; module level reads E3's public-surface.json)
from surface4 import SET as SEVEN  # noqa: E402

LESSCODE, RUNS = HERE.parents[3], HERE.parents[3] / ".runs/e4-lifecycle"
VENV = LESSCODE / ".runs/e2-facdrone/venv/Scripts/python.exe"
RES, MUT = HERE / "results", HERE / "mutants"
RECEIPT, LOGS = RES / "receipt.json", RES / "logs"
TREES = {"orig": (LESSCODE / ".runs/e2-facdrone/source", RUNS / "orig-mut", "e4o", RES / "e4-orig-mutant-sample.json", ""),
         "cand": (RUNS / "cand-turn3", RUNS / "cand-mut", "e4c", RES / "e4-cand-mutant-sample.json", "-cand")}
PINS = {"OPENBLAS_NUM_THREADS": "1", "OMP_NUM_THREADS": "1", "MKL_NUM_THREADS": "1", "PYTHONWARNINGS": "ignore",
        "PYTHONDONTWRITEBYTECODE": "1", "PYTHONIOENCODING": "utf-8"}
sha = t31_arm.sha


def tree(name: str) -> tuple[Path, Path, str, Path, str]:
    return TREES[name]


def seven(ref: Path, root: Path) -> dict:
    return {f: [sha(root / f), sha(ref / f)] for f in SEVEN}


def parse_arm(out: str) -> tuple[int | None, bool]:
    """run_e4's last status line: (its run() exit, replay fresh)."""
    m = re.search(r"^run_e4: exit (\d+); replay file fresh=(\w+)", out, re.M)
    return (int(m.group(1)), m.group(2) == "True") if m else (None, False)


def classify(code, log: str) -> tuple[str, str]:
    """(outcome, first failing reason): 0 survived; 2 + INVALID scope incompetent (a SyntaxError at import is one); 3 or 4
    apparatus fault; timeout killed; else killed with the first FAILED/ERROR node and its assertion (the replay's first
    divergence reads `<sequence> op <i> <op>: <path>`)."""
    if code == 0:
        return "survived", ""
    if code == 2 and "INVALID scope" in log:
        return "incompetent", "scope not imported from the mutant copy" + (" (SyntaxError)" if "SyntaxError" in log else "")
    if code in (3, 4):
        return "apparatus-fault", "replay file not fresh" if code == 3 else "hash mismatch (see the log)"
    if code in ("timeout", 124) or "deadline exceeded" in log:
        return "killed", "timeout"
    if "SyntaxError" in log and "error during collection" in log:
        return "incompetent", "SyntaxError at collection"
    node = re.search(r"^(FAILED|ERROR) (\S+)", log, re.M)
    why = re.search(r"^E\s+(\w*Error: .+)$", log, re.M)
    reason = (node.group(0).strip() if node else f"exit {code}") + (f" - {why.group(1).strip()[:400]}" if why else "")
    return "killed", reason


# ---- inner commands ------------------------------------------------------------------------------------------

def arm(name: str, tag: str, deadline: str | None) -> int:
    ref, root, *_ = tree(name)
    cmd = [str(VENV), str(HERE / "run_e4.py"), str(root), "--round", tag] + (["--deadline", deadline] if deadline else [])
    print(f"e4_arm: {name} root {root}; {' '.join(cmd[1:])}", flush=True)
    p = subprocess.run(cmd, cwd=str(HERE), capture_output=True, text=True, encoding="utf-8", errors="replace")
    print(p.stdout, flush=True)
    inner, fresh = parse_arm(p.stdout)
    if inner is None:
        return p.returncode or 1
    return inner if inner else (0 if fresh else 3)


def _restore_all(name: str) -> bool:
    ref, root, *_ = tree(name)
    ok = all(t31_arm.restore(root / f, ref / f) for f in SEVEN)
    print(f"e4_arm: restored, identical={ok}", flush=True)
    return ok


def mutant(name: str, k: int, tag: str, deadline: str | None) -> int:
    ref, root, _, sample, _ = tree(name)
    s = json.loads(sample.read_text(encoding="utf-8"))["sample"][k]
    assert all(a == b for a, b in seven(ref, root).values()), f"{root} not restored"
    t31_arm.apply_hunks(root / s["module"], s["hunks"])
    print(f"e4_arm: mutant {k} {s['module']} {s['operator']} occ {s['occurrence']} line {s['row']}: {s['old']} -> "
          f"{[h[2] for h in s['hunks']]}; mutated sha256={sha(root / s['module'])}", flush=True)
    try:
        return arm(name, tag, deadline)
    finally:
        if not _restore_all(name):
            raise SystemExit(4)


def hand(name: str, j: int, tag: str, deadline: str | None) -> int:
    ref, root, _, _, suffix = tree(name)
    rec = json.loads((MUT / "hand-mutants.json").read_text(encoding="utf-8"))["mutants"][f"M{j}"][name]
    patch, rel = MUT / f"M{j}{suffix}.patch", rec["target"]
    assert all(a == b for a, b in seven(ref, root).values()), f"{root} not restored"
    if sha(patch) != rec["patch_sha256"]:
        print(f"e4_arm: {patch.name} sha256 {sha(patch)} != recorded {rec['patch_sha256']}", flush=True)
        return 4
    p = subprocess.run(["git", "apply", str(patch)], cwd=str(root), capture_output=True, text=True)
    got = sha(root / rel)
    print(f"e4_arm: M{j} {patch.name} git apply exit {p.returncode} {p.stderr.strip()}; {rel} mutated sha256={got} "
          f"recorded={rec['mutated_sha256']} match={got == rec['mutated_sha256']}", flush=True)
    try:
        return arm(name, tag, deadline) if got == rec["mutated_sha256"] else 4
    finally:
        if not _restore_all(name):
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


def _verify(label: str, name: str) -> bool:
    import receipt_atomic
    ref, root, *_ = tree(name)
    h = seven(ref, root)
    ok = all(a == b for a, b in h.values())
    receipt_atomic.add_note(RECEIPT, label, {"root": str(root), "reference": str(ref), "identical": ok, "sha256": h})
    print(f"{label}: identical={ok}", flush=True)
    return ok


def step(label: str, base_s: float, inner: list[str]) -> dict:
    """Guard, scaled deadline from a receipted probe, one receipt entry; a timeout at factor > 1.5 waits for idle, reruns once."""
    import adaptive, receipt_atomic
    _guard(label)
    lab = label
    while True:
        secs, info = adaptive.scaled(base_s)
        receipt_atomic.add_note(RECEIPT, f"{lab}-deadline", {"base_s": base_s, "scaled_s": secs, "factor": info["factor"], "probe": info})
        code, entry = _run(lab, [str(VENV), str(HERE / "e4_arm.py"), *inner, "--round", lab, "--deadline", str(secs)], cwd=str(HERE), timeout=secs + 120)
        log = (LOGS / f"{lab}.log").read_text(encoding="utf-8", errors="replace")
        timed_out = code == "timeout" or "deadline exceeded" in log
        if not (timed_out and info["factor"] > 1.5) or lab.endswith("-rerun"):
            return dict(label=lab, exit=code, seconds=entry["seconds"], factor=info["factor"], scaled_s=secs, log=log)
        receipt_atomic.add_note(RECEIPT, f"{label}-wait", adaptive.wait_for_idle(1800))
        lab = label + "-rerun"


def prepare(name: str) -> int:
    ref, root, pfx, *_ = tree(name)
    if not root.exists():
        _guard(f"{pfx}-copy")
        _run(f"{pfx}-copy", [str(VENV), "-c", f"import shutil; shutil.copytree(r'{ref}', r'{root}', symlinks=False, "
                             "ignore=shutil.ignore_patterns('.git', '__pycache__', '.pytest_cache')); print('copied')"])
    return 0 if _verify(f"{pfx}-copy-verify", name) else 4


def base(name: str) -> int:
    pfx = tree(name)[2]
    if not _verify(f"{pfx}-base-pre", name):
        return 4
    r = step(f"{pfx}-base", 1200, ["arm", name])
    print(json.dumps({k: r[k] for k in ("label", "exit", "seconds", "factor", "scaled_s")}), flush=True)
    return 0 if r["exit"] == 0 else 1


def _base_seconds(pfx: str) -> float | None:
    cmds = json.loads(RECEIPT.read_text(encoding="utf-8"))["commands"]
    passed = [c for c in cmds if c["label"] in (f"{pfx}-base", f"{pfx}-base-rerun") and c["exit"] == 0]
    return round(max(120.0, 5 * passed[-1]["seconds"]), 1) if passed else None


def _after(label: str, name: str, base_s: float, r: dict, extra: dict) -> bool:
    """Restore anything the killed inner left behind, verify, classify, write the -class note."""
    import receipt_atomic
    ref, root, *_ = tree(name)
    for f in SEVEN:
        if sha(root / f) != sha(ref / f):  # the inner process was killed before its own restore
            shutil.copy2(ref / f, root / f)
    restored = _verify(f"{label}-restore", name)
    outcome, reason = classify(r["exit"], r["log"])
    m = re.search(r"mutated sha256=(\w+)", r["log"])
    row = {"tree": name, "label": r["label"], **extra, "exit": r["exit"], "seconds": r["seconds"], "factor": r["factor"],
           "deadline_base_s": base_s, "scaled_s": r["scaled_s"], "classification": outcome, "first_failing_reason": reason,
           "mutated_sha256": m.group(1) if m else None, "restore_identical": restored}
    receipt_atomic.add_note(RECEIPT, f"{label}-class", row)
    print(json.dumps(row), flush=True)
    return restored


def dry(name: str, ks: list[int]) -> int:
    pfx, sample = tree(name)[2], tree(name)[3]
    base_s = _base_seconds(pfx)
    if base_s is None:
        print(f"e4_arm: no passing {pfx}-base in the receipt; run `base {name}` first", flush=True)
        return 2
    specs = json.loads(sample.read_text(encoding="utf-8"))["sample"]
    for k in ks:
        if not _verify(f"{pfx}-m{k}-pre", name):
            return 4
        r = step(f"{pfx}-m{k}", base_s, ["mutant", name, str(k)])
        s = specs[k]
        extra = {"k": k, "module": s["module"], "operator": s["operator"], "row": s["row"], "col": s["col"], "occurrence": s["occurrence"]}
        if not _after(f"{pfx}-m{k}", name, base_s, r, extra):
            return 4
    return 0


def hand_run(name: str, js: list[int]) -> int:
    pfx = tree(name)[2]
    rec = json.loads((MUT / "hand-mutants.json").read_text(encoding="utf-8"))["mutants"]
    for j in js:
        if not _verify(f"{pfx}-M{j}-pre", name):
            return 4
        r = step(f"{pfx}-M{j}", 1200, ["hand", name, str(j)])
        extra = {"j": j, "patch": rec[f"M{j}"][name]["patch"], "module": rec[f"M{j}"][name]["target"],
                 "expected_mutated_sha256": rec[f"M{j}"][name]["mutated_sha256"]}
        if not _after(f"{pfx}-M{j}", name, 1200, r, extra):
            return 4
    return 0


if __name__ == "__main__":
    a = sys.argv[1:]
    if not a or a[0] in ("-h", "--help"):
        print(__doc__)
        sys.exit(0 if a else 2)
    opt = lambda k: a[a.index(k) + 1] if k in a else None  # noqa: E731
    dl, tag = opt("--deadline"), opt("--round") or "0"
    cmds = {"arm": lambda: arm(a[1], tag, dl), "mutant": lambda: mutant(a[1], int(a[2]), tag, dl), "hand": lambda: hand(a[1], int(a[2]), tag, dl),
            "prepare": lambda: prepare(a[1]), "base": lambda: base(a[1]), "dry": lambda: dry(a[1], [int(x) for x in a[2:]]),
            "hand-run": lambda: hand_run(a[1], [int(x) for x in a[2:]])}
    sys.exit(cmds[a[0]]() if a[0] in cmds else f"unknown command {a[0]}")
