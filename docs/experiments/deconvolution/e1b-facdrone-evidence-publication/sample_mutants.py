"""Fixed random mutant sample shared by both arms, and the per-mutant timeout rule.

  sample_mutants.py sample SESSION_A.sqlite SESSION_B.sqlite [N] [SEED]
      after `cosmic-ray init` of both configs: identical specs required; draw N (60, amendment 3) with
      random.Random(SEED) (20261103) from the sorted keys; delete the rest from both sessions
      run with cosmic-ray's interpreter: each sampled mutant also gets its changed lines (amendment 7)
  sample_mutants.py timeout   run_arm --deadline auto (results/deadlines.json, `run_arm.py time`);
      cosmic-ray backstop timeout = the arm's largest measured deadline + 120 s"""
import json, random, re, sqlite3, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CONFIGS = {"A": HERE / "cosmic-ray-arm-a.toml", "B": HERE / "cosmic-ray-arm-b.toml"}


def specs(session: Path) -> dict[tuple, str]:
    with sqlite3.connect(session) as db:
        rows = db.execute("SELECT job_id, module_path, operator_name, operator_args, occurrence, "
                          "start_pos_row, start_pos_col FROM mutation_specs").fetchall()
    keys = {(m.replace("\\", "/"), op, args, occ, row, col): job for job, m, op, args, occ, row, col in rows}
    assert len(keys) == len(rows), "duplicate mutation specs"
    return keys


def sample(a: Path, b: Path, n: int, seed: int) -> dict:
    sa, sb = specs(a), specs(b)
    if set(sa) != set(sb):
        raise SystemExit("the two sessions hold different mutation specs; refuse to sample")
    chosen = sorted(random.Random(seed).sample(sorted(sa), n))
    for session, keys in ((a, sa), (b, sb)):
        keep = {keys[k] for k in chosen}
        drop = [(job,) for job in keys.values() if job not in keep]
        with sqlite3.connect(session) as db:
            if db.execute("SELECT COUNT(*) FROM work_results").fetchone()[0]:
                raise SystemExit(f"{session} already has results; sample before exec")
            db.executemany("DELETE FROM mutation_specs WHERE job_id = ?", drop)
            db.executemany("DELETE FROM work_items WHERE job_id = ?", drop)
    mods = sorted({k[0] for k in sa})
    out = {"seed": seed, "n": n, "population": len(sa), "per_module": {m: sum(k[0] == m for k in chosen) for m in mods},
           "population_per_module": {m: sum(k[0] == m for k in sa) for m in mods},
           "sample": [dict(zip(("module", "operator", "args", "occurrence", "row", "col"), k), lines=lines(*k[:4]))
                      for k in chosen]}
    (HERE / "results" / "mutant-sample.json").write_text(json.dumps(out, indent=1))
    return {k: v for k, v in out.items() if k != "sample"}


def lines(module: str, operator: str, args: str, occurrence: int) -> list[int]:
    """The frozen copy's lines this mutant changes, as run_arm.py computes them from the diff."""
    import scope
    from cosmic_ray.mutating import mutate_code
    from cosmic_ray.plugins import get_operator
    src = (scope.FROZEN / module).read_text(encoding="utf-8")
    kw = json.loads(args) if args else {}  # cosmic-ray stores the arguments JSON-encoded twice
    mutated = mutate_code(src, get_operator(operator)(**(json.loads(kw) if isinstance(kw, str) else kw)), occurrence)
    return scope.changed_lines(src, mutated) if mutated else []


def timeout() -> None:
    table = json.loads((HERE / "results" / "deadlines.json").read_text(encoding="utf-8"))
    for arm in CONFIGS:
        top = max(v["deadline"] for k, v in table.items() if k.startswith(arm + ":"))
        text = CONFIGS[arm].read_text(encoding="utf-8")
        text = re.sub(r"^timeout = .*$", f"timeout = {top + 120:.1f}  # backstop: largest measured deadline + 120 s", text, flags=re.M)
        text = re.sub(r'--deadline [^"\s]+', "--deadline auto", text)
        CONFIGS[arm].write_text(text, encoding="utf-8")


if __name__ == "__main__":
    a = sys.argv[1:]
    if a[0] == "sample":
        print(json.dumps(sample(Path(a[1]), Path(a[2]), int(a[3]) if len(a) > 3 else 60,
                                int(a[4]) if len(a) > 4 else 20261103), indent=1))
    else:
        timeout()
