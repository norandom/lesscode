"""E3 step 3, T4: sensitivity of the acceptance to cosmic-ray mutants of the candidate's publication module.
  t4_arm.py sample SESSION         (cosmic-ray's interpreter) 30 specs, seed 20261104 -> results/t4-mutant-sample.json
  t4_arm.py arm [--deadline S]     run_e3's arm on MUT: probe, R16, the seed files importing the module, replay
  t4_arm.py mutant K [--deadline S]  apply sample K to MUT, arm, restore from the candidate, verify by sha256
Kill = nonzero; 2 with "INVALID scope" = incompetent; 3 = replay file not fresh (fail closed, an apparatus fault)."""
import difflib, hashlib, json, os, random, re, shutil, sys, time
from pathlib import Path
HERE = Path(__file__).resolve().parent
RUNS, REL = HERE.parents[3] / ".runs" / "e3-facdrone", "src/facdrone/shells/decision_evidence_publication.py"
CAND, MUT, RES = RUNS / "cand", RUNS / "mut", HERE / "results"
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()  # noqa: E731
SURF = json.loads((RES / "public-surface.json").read_text(encoding="utf-8"))["modules"]
FILES = sorted(f for f in SURF["facdrone.shells.decision_evidence_publication"]["importers"]
               if re.match(r"(tests|packages/[^/]+/tests)/", f))  # test files importing the module or its names


def sample(session: str) -> None:
    sys.path.insert(0, str(HERE.parent / "e1b-facdrone-evidence-publication"))
    import sample_mutants
    from cosmic_ray.mutating import mutate_code
    from cosmic_ray.plugins import get_operator
    keys, src, out = sample_mutants.specs(Path(session)), (MUT / REL).read_bytes().decode("utf-8"), []
    for _m, op, args, occ, row, col in sorted(random.Random(20261104).sample(sorted(keys), 30)):
        kw = json.loads(args) if args else {}  # stored JSON-encoded twice (E1b)
        a, b = src.split("\n"), mutate_code(src, get_operator(op)(**(json.loads(kw) if isinstance(kw, str) else kw)), occ).split("\n")
        hunks = [(i1, i2, b[j1:j2]) for t, i1, i2, j1, j2 in difflib.SequenceMatcher(None, a, b, autojunk=False).get_opcodes() if t != "equal"]
        out.append(dict(operator=op, args=args, occurrence=occ, row=row, col=col, hunks=hunks, old=[a[h[0]:h[1]] for h in hunks]))
    RES.joinpath("t4-mutant-sample.json").write_text(json.dumps({"seed": 20261104, "population": len(keys), "module": REL,
        "candidate_sha256": sha(CAND / REL), "seed_files": FILES, "sample": out}, indent=1), encoding="utf-8")


def arm(deadline: str | None) -> int:
    sys.path.insert(0, str(HERE))
    import run_e3  # patches run_arm (all seeds, E3 replay target); the seed selection is replaced here

    def seeds(root: Path) -> tuple[list[str], list[str]]:
        run_e3.run_arm.SELECTED[:] = FILES
        return FILES, [n for n in run_e3.SEL["deselect"] if n.split("::")[0] in FILES]
    run_e3.run_arm.seeds = seeds
    os.environ.update(E3_ROUND="t4", PYTHONDONTWRITEBYTECODE="1")
    l2, t0 = RES / "sql-divergences-roundt4.json", time.time()
    print(f"t4_arm: {len(FILES)} seed files {FILES}", flush=True)
    code = run_e3.run_arm.run("B", MUT, None, deadline)
    rep = json.loads(l2.read_text(encoding="utf-8")) if l2.exists() else {}
    fresh = rep.get("written", 0) > t0 and rep.get("sequences") == 64
    print(f"t4_arm: run_arm exit {code}; replay fresh={fresh} sequences={rep.get('sequences')}", flush=True)
    return code if code else (0 if fresh else 3)


def mutant(k: int, deadline: str | None) -> int:
    s = json.loads((RES / "t4-mutant-sample.json").read_text(encoding="utf-8"))["sample"][k]
    assert sha(MUT / REL) == sha(CAND / REL), "mut copy not restored"
    lines = (MUT / REL).read_bytes().decode("utf-8").split("\n")
    for i1, i2, new in reversed(s["hunks"]):
        lines[i1:i2] = new
    (MUT / REL).write_bytes("\n".join(lines).encode("utf-8"))
    print(f"t4_arm: mutant {k} {s['operator']} occ {s['occurrence']} line {s['row']}: {s['old']} -> {[h[2] for h in s['hunks']]}", flush=True)
    try:
        return arm(deadline)
    finally:
        shutil.copy2(CAND / REL, MUT / REL)
        print(f"t4_arm: restored, identical={sha(MUT / REL) == sha(CAND / REL)}", flush=True)
        if sha(MUT / REL) != sha(CAND / REL):
            raise SystemExit(4)


if __name__ == "__main__":
    a = sys.argv[1:]
    dl = a[a.index("--deadline") + 1] if "--deadline" in a else None
    sys.exit(sample(a[1]) if a[0] == "sample" else arm(dl) if a[0] == "arm" else mutant(int(a[1]), dl))
