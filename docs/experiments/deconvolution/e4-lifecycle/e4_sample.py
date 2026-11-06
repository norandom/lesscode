"""deconvolution-loop task 6.1 (for 6.4): the 30-mutant sample of the ORIGINAL's seven lifecycle files, drawn as E3's T4
and E3b's t31 drew theirs: `t31_arm.draw` = sorted(random.Random(SEED).sample(sorted(keys), N)) over the population that
cosmic-ray 8.7.0 enumerated (`cosmic-ray init cosmic-ray-e4.toml SESSION`, keys read by `sample_mutants.specs`). Seed
20261106, 30 draws, one population over all seven modules. Each drawn spec is applied with cosmic-ray's `mutate_code` to
the frozen file's text (read only) and stored as exact hunks (`t31_arm.hunks_of`), so the 6.4 arm applies it with
`t31_arm.apply_hunks` in the mutable copy and restores by copy. Run under cosmic-ray's interpreter:
  uv tool run --python 3.13 --from cosmic-ray==8.7.0 python e4_sample.py SESSION [--root FROZEN] [--out results/e4-orig-mutant-sample.json]
The 6.4 candidate-side sample is drawn the same way from the candidate's own population (its own `init`, same seed)."""
import json, sys
from collections import Counter
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path[:0] = [str(HERE), str(HERE.parent / "e3b-original-sensitivity"), str(HERE.parent / "e1b-facdrone-evidence-publication")]
import sample_mutants  # noqa: E402  (E1b: specs(session) -> {(module, operator, args, occurrence, row, col): job_id})
import t31_arm  # noqa: E402  (E3b: draw, hunks_of, sha; its module level reads only E3's public-surface.json)
from surface4 import SET  # noqa: E402
SEED, N = 20261106, 30
FROZEN = HERE.parents[3] / ".runs/e2-facdrone/source"
SEEDS = HERE / "results" / "seeds.json"


def sample(session: Path, root: Path, out: Path) -> dict:
    from cosmic_ray.mutating import mutate_code
    from cosmic_ray.plugins import get_operator
    keys = sample_mutants.specs(session)
    mods = sorted({k[0] for k in keys})
    assert mods == sorted(SET), f"population modules {mods} are not the seven scope files"
    src = {m: (root / m).read_bytes().decode("utf-8") for m in mods}
    rows = []
    for m, op, args, occ, row, col in t31_arm.draw(keys, SEED, N):
        kw = json.loads(args) if args else {}  # stored JSON-encoded twice (E1b)
        mutated = mutate_code(src[m], get_operator(op)(**(json.loads(kw) if isinstance(kw, str) else kw)), occ)
        hunks, old = t31_arm.hunks_of(src[m], mutated)
        assert hunks, f"spec {(m, op, occ)} produced no change"
        rows.append(dict(module=m, operator=op, args=args, occurrence=occ, row=row, col=col, hunks=hunks, old=old))
    seeds = json.loads(SEEDS.read_text(encoding="utf-8"))
    body = {"seed": SEED, "n": N, "rule": "sorted(random.Random(seed).sample(sorted(keys), n)); keys = (module, operator, args, occurrence, row, col)",
            "population": len(keys), "population_per_module": {m: sum(k[0] == m for k in keys) for m in mods},
            "sample_per_module": {m: sum(r["module"] == m for r in rows) for m in mods},
            "operators": dict(sorted(Counter(r["operator"] for r in rows).items())),
            "reference": str(root), "module_sha256": {m: t31_arm.sha(root / m) for m in mods}, "session": str(session),
            "seed_files": seeds["seeds"], "deselect": seeds["deselect"], "marks": seeds["marks"], "sample": rows}
    out.write_text(json.dumps(body, indent=1), encoding="utf-8")
    return {k: v for k, v in body.items() if k not in ("sample", "seed_files", "deselect")}


if __name__ == "__main__":
    a = sys.argv[1:]
    if not a or a[0] in ("-h", "--help"):
        sys.exit(print(__doc__) or (0 if a else 2))
    opt = lambda k, d: a[a.index(k) + 1] if k in a else d  # noqa: E731
    summary = sample(Path(a[0]), Path(opt("--root", FROZEN)).resolve(), Path(opt("--out", HERE / "results" / "e4-orig-mutant-sample.json")))
    print(json.dumps(summary, indent=1))
