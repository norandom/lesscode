"""E3 acceptance against a candidate root: E1b's run_arm.run (imported, unchanged) with all 191 seed files.
  run_e3.py ROOT [--round N] [--deadline S]
run_arm.run does the scope-import probe and the R16 check, then pytest per group (serial, -x, -p e2_winshim,
marks "not port_baseline and not live_broker", PYTHONPATH ROOT/src first). Here run_arm.seeds is replaced
by the full selection map (run_arm selects nothing for a change outside its mutated modules) with the
baseline deselections of those files, and the replay target by test_e3_replay.py. Exit 0 only if every
group passes and the replay wrote this run's Level 2 file with all 64 sequences replayed."""
import json, os, sys, time
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "e1b-facdrone-evidence-publication"))
import run_arm  # noqa: E402

SEL = json.loads(run_arm.SELECTION.read_text(encoding="utf-8"))
E1B_TEST, E3_TEST = (run_arm.HERE / "test_e1b_replay.py").as_posix(), (HERE / "test_e3_replay.py").as_posix()


def seeds(root: Path) -> tuple[list[str], list[str]]:
    files = SEL["seeds"]
    run_arm.SELECTED[:] = files
    bad = [n for n in SEL["deselect"] if n.split("::")[0] in files]  # the others name no collected node
    print(f"run_e3: {len(files)} seed files, {len(bad)} of {len(SEL['deselect'])} deselections apply", flush=True)
    return files, bad


run_arm.seeds = seeds
_groups = run_arm.groups
run_arm.groups = lambda arm, root: [(cwd, [E3_TEST if a == E1B_TEST else a for a in args]) for cwd, args in _groups(arm, root)]

if __name__ == "__main__":
    a = sys.argv[1:]
    opts = dict(zip(a[1::2], a[2::2]))
    root, rnd = Path(a[0]).resolve(), opts.get("--round", "0")
    os.environ["E3_ROUND"] = rnd
    l2 = HERE / "results" / f"sql-divergences-round{rnd}.json"
    t0 = time.time()
    gs = run_arm.groups("B", root)
    print("run_e3: groups " + json.dumps([[c, sum(1 for x in g if x.endswith(".py"))] for c, g in gs]), flush=True)
    code = run_arm.run("B", root, None, opts.get("--deadline"))
    rep = json.loads(l2.read_text(encoding="utf-8")) if l2.exists() else {}
    fresh = rep.get("written", 0) > t0 and rep.get("sequences") == 64
    print(f"run_e3: run_arm exit {code}; replay file fresh={fresh} sequences={rep.get('sequences')} "
          f"operations={rep.get('operations')} level2_ops={rep.get('level2_ops')}", flush=True)
    sys.exit(code if code else (0 if fresh else 1))
