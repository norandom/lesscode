"""Usefulness check of the missing-test report (deconvolution-loop task 2.1; preregistered in
results/missing-tests-experiment.md). Scores one externally written test file; never edits the frozen trees.
  missing_tests_check.py --tests FILE [--label mt]     orig copy, unmodified candidate copy, each real T4 gap
  missing_tests_check.py --tests FILE --dry-run        mechanics only: orig copy, one mutant, one pytest run
Targets: ORIG_CHECK (.runs/e3-facdrone/orig-check, a copytree of the frozen original, five regenerated files
hash-verified), then MUT unmodified (= the candidate), then MUT with each real T4 gap's hunks applied (gaps
from results/missing-tests.json). The test file is staged as tests/unit/STAGED in the target and removed after.
Every pytest run: guard.py, then receipt_atomic.run into results/receipt-missing-tests.json (logs in
results/logs/LABEL.log, junit in results/logs/LABEL.xml), deadline adaptive.scaled(600). Exit 0 done,
3 guard refused, 4 a copy failed its hash check, 2 invalid (scope probe, staging)."""
from __future__ import annotations
import argparse, hashlib, json, os, shutil, subprocess, sys
import xml.etree.ElementTree as ET
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path[:0] = [str(HERE), str(HERE.parent)]
import adaptive, guard, receipt_atomic  # noqa: E402
from t4_arm import CAND, MUT, REL, RES  # noqa: E402  (module level only reads public-surface.json)

LESSCODE = HERE.parents[3]
VENV = LESSCODE / ".runs/e2-facdrone/venv/Scripts/python.exe"
FROZEN, ORIG_CHECK = LESSCODE / ".runs/e2-facdrone/source", LESSCODE / ".runs/e3-facdrone/orig-check"
E1B, EXTRAS, E2 = HERE.parent / "e1b-facdrone-evidence-publication", LESSCODE / ".runs/e1b-facdrone/extras", HERE.parent / "e2-facdrone-edge-census"
RECEIPT, LOGS = RES / "receipt-missing-tests.json", RES / "logs"
FIVE = tuple(f"src/facdrone/shells/{m}.py" for m in ("decision_evidence_publication", "decision_evidence_store",
             "decision_evidence_reader", "decision_input_slice_store", "decision_slice_publisher"))
STAGED = "test_zz_missing_tests_experiment.py"
MARKS, BASE_S = "not port_baseline and not live_broker", 600
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()  # noqa: E731


def verify(root: Path, ref: Path, label: str) -> None:
    hashes = {f: [sha(root / f), sha(ref / f)] for f in FIVE}
    ok = all(a == b for a, b in hashes.values())
    receipt_atomic.add_note(RECEIPT, label, {"root": str(root), "reference": str(ref), "identical": ok, "sha256": hashes})
    print(f"{label}: identical={ok}", flush=True)
    if not ok:
        raise SystemExit(4)


def ensure_orig(prefix: str) -> None:
    if not ORIG_CHECK.exists():
        code = receipt_atomic.run(RECEIPT, f"{prefix}-copy-orig", [str(VENV), "-c",
                                  f"import shutil; shutil.copytree(r'{FROZEN}', r'{ORIG_CHECK}', symlinks=False)"])
        if code != 0:
            raise SystemExit(4)
    verify(ORIG_CHECK, FROZEN, f"{prefix}-verify-orig")


def apply_mutant(k: int) -> dict:
    """Verbatim from t4_arm.mutant (which also runs its arm, so it is not imported)."""
    s = json.loads((RES / "t4-mutant-sample.json").read_text(encoding="utf-8"))["sample"][k]
    assert sha(MUT / REL) == sha(CAND / REL), "mut copy not restored"
    lines = (MUT / REL).read_bytes().decode("utf-8").split("\n")
    for i1, i2, new in reversed(s["hunks"]):
        lines[i1:i2] = new
    (MUT / REL).write_bytes("\n".join(lines).encode("utf-8"))
    return s


def restore(label: str) -> None:
    shutil.copy2(CAND / REL, MUT / REL)
    verify(MUT, CAND, f"{label}-restore")


def outcomes(xml: Path) -> dict[str, str]:
    if not xml.exists():
        return {}
    out = {}
    for case in ET.parse(xml).getroot().iter("testcase"):
        tags = {c.tag for c in case}
        out[f"{case.get('classname')}::{case.get('name')}"] = ("failed" if "failure" in tags else "error" if "error" in tags
                                                               else "skipped" if "skipped" in tags else "passed")
    return out


def pytest_run(label: str, root: Path, tests: Path) -> dict:
    """Stage the test file, probe the import scope, guard, run through the receipt, unstage."""
    dest, xml = root / "tests" / "unit" / STAGED, LOGS / f"{label}.xml"
    if dest.exists():
        print(f"{label}: INVALID staged file already present: {dest}")
        raise SystemExit(2)
    before = set(os.listdir(root)) | {"tests/unit/" + n for n in os.listdir(root / "tests" / "unit")}
    env = {"OPENBLAS_NUM_THREADS": "1", "OMP_NUM_THREADS": "1", "MKL_NUM_THREADS": "1", "PYTHONWARNINGS": "ignore",
           "PYTHONDONTWRITEBYTECODE": "1", "E1B_SOURCE_ROOT": str(root),
           "PYTHONPATH": os.pathsep.join(map(str, (root / "src", E1B, EXTRAS, E2)))}
    shutil.copyfile(tests, dest)
    try:
        probe = subprocess.run([str(VENV), "-c", "import facdrone.shells.decision_evidence_publication as m; print(m.__file__)"],
                               env=dict(os.environ, **env), capture_output=True, text=True)
        where = probe.stdout.strip()
        inside = bool(where) and Path(where).resolve().is_relative_to(root.resolve())
        receipt_atomic.add_note(RECEIPT, f"{label}-scope", {"module_file": where, "inside_root": inside})
        if not inside:
            print(f"{label}: INVALID scope not imported from {root}: {where}{probe.stderr}")
            raise SystemExit(2)
        if guard.guard(label, RECEIPT) != 0:
            raise SystemExit(guard.STOP)
        deadline, info = adaptive.scaled(BASE_S)
        receipt_atomic.add_note(RECEIPT, f"{label}-deadline", {"base_s": BASE_S, "deadline_s": deadline, "probe": info})
        cmd = [str(VENV), "-m", "pytest", "-q", "-c", "pyproject.toml", "--rootdir", ".", "-p", "no:cacheprovider",
               "-p", "e2_winshim", "--timeout=600", "-m", MARKS, f"--junitxml={xml}", f"tests/unit/{STAGED}"]
        code = receipt_atomic.run(RECEIPT, label, cmd, cwd=str(root), env=env, timeout=deadline)
        if code == "timeout" and adaptive.factor()[0] > 1.5:  # req 11.4: rerun once when idle
            receipt_atomic.add_note(RECEIPT, f"{label}-wait", adaptive.wait_for_idle(1800))
            label = f"{label}-rerun"
            xml = LOGS / f"{label}.xml"
            cmd[-2] = f"--junitxml={xml}"
            code = receipt_atomic.run(RECEIPT, label, cmd, cwd=str(root), env=env, timeout=deadline)
    finally:
        dest.unlink(missing_ok=True)
        new = (set(os.listdir(root)) | {"tests/unit/" + n for n in os.listdir(root / "tests" / "unit")}) - before
        for n in sorted(new):  # pytest/hypothesis droppings in the copy; anything else is reported, not removed
            if Path(n).name in (".hypothesis", ".pytest_cache", "__pycache__"):
                shutil.rmtree(root / n, ignore_errors=True)
        left = sorted(n for n in new if (root / n).exists())
        receipt_atomic.add_note(RECEIPT, f"{label}-unstage", {"removed": sorted(set(new) - set(left)), "left": left})
    return {"label": label, "exit": code, "tests": outcomes(xml)}


def score(orig: dict, cand: dict, mutants: dict[str, dict]) -> dict:
    """The preregistered rule: tests passing on the original are valid; a gap is killed by a valid test that
    also passes on the unmodified candidate and fails (or errors) with the gap's mutant applied."""
    bad = ("failed", "error")
    valid = sorted(t for t, o in orig["tests"].items() if o == "passed")
    on_cand = sorted(t for t in valid if cand["tests"].get(t) == "passed")
    exposed = sorted(t for t in valid if cand["tests"].get(t) in bad)
    killed = {g: sorted(t for t in on_cand if m["tests"].get(t) in bad) for g, m in mutants.items()}
    n = sum(1 for v in killed.values() if v)
    return {"tests_total": len(orig["tests"]), "valid_on_original": valid, "failing_on_original": sorted(set(orig["tests"]) - set(valid)),
            "passing_on_candidate": on_cand, "exception_path_exposed_by": exposed, "killed_by": killed,
            "gaps_killed": f"{n}/{len(mutants)}", "kill_criterion_report_not_actionable": n < 2,
            "prediction_held": n == len(mutants) == 3 and bool(exposed)}


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--tests", required=True, type=Path)
    p.add_argument("--label", default="mt")
    p.add_argument("--dry-run", action="store_true")
    a = p.parse_args(argv)
    tests, prefix = a.tests.resolve(), "dry" if a.dry_run else a.label
    report = json.loads((RES / "missing-tests.json").read_text(encoding="utf-8"))
    gaps = [int(i["id"].split("-")[1]) for i in report["real"] if i["id"].startswith("T4-")]
    receipt_atomic.add_note(RECEIPT, f"{prefix}-start", {"tests": str(tests), "tests_sha256": sha(tests), "gaps": gaps,
                            "dry_run": a.dry_run, "report_sha256": sha(RES / "missing-tests.md"), "runner_sha256": sha(__file__)})
    ensure_orig(prefix)
    verify(MUT, CAND, f"{prefix}-verify-mut")
    if a.dry_run:  # one mutant, one pytest run: proves apply, stage, run, unstage, restore
        gaps = gaps[:1]
    runs: dict[str, dict] = {}
    if not a.dry_run:
        runs["orig"] = pytest_run(f"{prefix}-orig", ORIG_CHECK, tests)
        runs["cand"] = pytest_run(f"{prefix}-cand", MUT, tests)
        verify(MUT, CAND, f"{prefix}-cand-after")
    for k in gaps:
        label = f"{prefix}-m{k}"
        s = apply_mutant(k)
        receipt_atomic.add_note(RECEIPT, f"{label}-apply", {"mutant": k, "operator": s["operator"], "row": s["row"],
                                "old": s["old"], "new": [h[2] for h in s["hunks"]], "mut_sha256": sha(MUT / REL)})
        try:
            runs[f"T4-{k}"] = pytest_run(label, MUT, tests)
        finally:
            restore(label)
    fact = {"experiment": "missing-tests-usefulness", "label_prefix": prefix, "dry_run": a.dry_run,
            "tests_sha256": sha(tests), "runs": runs}
    if not a.dry_run:
        fact["scoring"] = score(runs["orig"], runs["cand"], {g: r for g, r in runs.items() if g.startswith("T4-")})
    receipt_atomic.add_fact(RECEIPT, fact)
    print(json.dumps(fact.get("scoring", {g: {"exit": r["exit"], "tests": r["tests"]} for g, r in runs.items()}), indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
