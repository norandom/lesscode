"""E4 task 6.5: differential drivers, the frozen original against the accepted candidate (cand-turn3), 3 pairs (pairs.md).
Both trees' scope modules are loaded from their own files under distinct names in ONE process (the E3 t5_drivers loader:
module objects, no sys.path switching); non-scope packages come from the venv's editable install of the frozen tree, which
is byte-identical on both trees (editscope-e4-t3). Pairs: compare   lifecycle_group._compare adapted to (lines, code)  H1/F3, H11
  standing  lifecycle_projection.signal_standing (six helpers inlined)  H8, H7     window  LiveEvidence.for_window (_lookback inlined)  H13
Per pair P: o_P / c_P (original / candidate), k_P (a one-line planted mutant in a scratch copy of the candidate module, gate 3a),
e_P (locals renamed in a scratch copy, gate 3b), ct_P / et_P (candidate / renamed on a 3-boolean domain so exhaustion is reachable).
A result is a canonical repr; an exception is ("raise", type, message). Domains by construction (plan 6.5, E3 Amendment 2.7): lists
of at most 3 via zip, enumerations as int indexes, dates as day offsets, no rejection guard, no slicing of symbolic strings.
  --list | --smoke P [--t 15] | --corpus P [--n 300] [--seed 20261106] | --plant P | --rename P | --gates      (phase 1, light)
  --solve P --deadline S [--kind pair|plant|rename] | --replay P LOG...                                        (phase 2, solver go)"""
import inspect, json, os, re, subprocess, sys, types
from dataclasses import dataclass
from datetime import UTC, date, datetime, timedelta
from pathlib import Path
from types import SimpleNamespace as NS
from typing import Optional
sys.dont_write_bytecode = True  # nothing may write .pyc into the frozen copy or the accepted candidate
from facdrone.pipelines.evaluation.monitor_windows import Window  # noqa: E402  (non-scope, frozen editable install)
from facdrone_kernel.domain.signal_lifecycle import LifecycleState, OpenFlag, StandingRevision, UnrecordedStanding  # noqa: E402
AS_OF = datetime(2026, 9, 7, 12, tzinfo=UTC)  # = facdrone_testing.core.cppi.AS_OF; not imported: a parent holding that import dies
HERE = Path(__file__).resolve().parent  # with 0xE0000001 when a child imports it (receipt e5-compare-smoke-diag3/4/5, fix note e5-driver-fix-1)
OUT, RUNS = HERE.parent / "results", HERE.parents[4] / ".runs"
ROOTS = {"orig": RUNS / "e2-facdrone/source", "cand": RUNS / "e4-lifecycle/cand-turn3"}
SCOPE = {"lifecycle_store": "lifecycle_store", "lifecycle_monitor_runner": "lifecycle_monitor_runner", "lifecycle_evidence": "lifecycle_evidence",
         "lifecycle_projection": "lifecycle_projection", "lifecycle_root": "wiring.lifecycle_root", "lifecycle_group": "cli.lifecycle_group"}
LAUNCH = "import sys, e5_drivers; from crosshair.main import main; sys.argv = ['crosshair', *sys.argv[1:]]; sys.exit(main())"


def _load(name, file, text=None, swap=()):  # exec the file's text as module `name`; `swap`: this tree's siblings under their real names
    m = sys.modules[name] = types.ModuleType(name); m.__file__ = str(file)
    saved = {k: sys.modules.get(k) for k, _ in swap}; sys.modules.update(dict(swap))
    try: exec(compile(text or file.read_text(encoding="utf-8"), str(file), "exec"), m.__dict__)  # noqa: S102
    finally: [sys.modules.pop(k) if v is None else sys.modules.__setitem__(k, v) for k, v in saved.items()]
    return m


def _tree(tag):  # the six scope modules the pairs can reach, in dependency order (seams excluded)
    mods, swap = {}, []
    for attr, mod in SCOPE.items():
        mods[attr] = _load(f"{tag}_{attr}", ROOTS[tag] / "src/facdrone/shells" / (mod.replace(".", "/") + ".py"), swap=tuple(swap))
        swap.append((f"facdrone.shells.{mod}", mods[attr]))
    return NS(swap=tuple(swap), **mods)


LIGHT = __name__ == "__main__" and len(sys.argv) > 1 and sys.argv[1] in ("--smoke", "--solve", "--gates")  # the launcher's parent must not
def _n(xs, k=3): return [x for _, x in zip(range(k), xs)]  # noqa: E302,E704  load the trees: it dies 0xE0000001 (note e5-driver-fix-2)
def _call(f, *a):  # noqa: E302
    try: return f(*a)  # noqa: E701
    except Exception as e: return ("raise", type(e).__name__, str(e))  # noqa: BLE001,E701
DAYS = tuple(AS_OF - timedelta(days=i) for i in range(30))  # concrete instants indexed by symbolic ints: CrossHair's symbolic timedelta
NOWS, AGE = tuple(AS_OF + timedelta(days=i) for i in range(3)), tuple((d, d - timedelta(seconds=1)) for d in DAYS)  # cannot meet a concrete datetime
MON, DISP = ("survival", "ic_gap", "window"), ("within", "breach", "not_run")  # index 2: the not-run marker row / disposition
Rowv = tuple[int, int, int, int, int]  # stored row: monitor, disposition, value, boundary, shape (3 well formed, else one key absent)
Fresh = tuple[int, int, int, int]  # re-measurement: monitor, value, boundary, disposition
@dataclass(frozen=True)  # noqa: E302
class R: value: object; boundary: object; disposition: str  # noqa: E701  the fresh MonitorReading stand-in (the three attributes read)


def _rows(rows):
    out = []
    for m, d, v, b, s in _n(rows):
        row = {"monitor": MON[m % 3], "disposition": DISP[d % 3], "value": v, "boundary": b}
        row.pop(("disposition", "value", "monitor", "")[s % 4], None); out.append(row)
    return out
def _compare(M, rows: list[Rowv], fresh: list[Fresh]):  # noqa: E302  H1/F3, H11: the original echoes as it goes, the candidate returns (lines, code)
    lines = []; M.typer = NS(echo=lines.append)
    r = _call(M._compare, _rows(rows), {MON[m % 3]: R(v, b, DISP[d % 3]) for m, v, b, d in _n(fresh)})
    return repr((r[0], r[1]) if isinstance(r, tuple) and r[0] != "raise" else (lines, r))


Rd = tuple[int, int, int, int, int]  # reading row: window, monitor, disposition, basis (digest, rules), shape (payload kind; key absent)
def _rd(w, m, d, g, s):  # noqa: E302
    row = {"window_id": ("w0", "w1", "w2")[w % 3], "monitor": MON[m % 3], "disposition": DISP[d % 3], "baseline_digest": ("d0", "d1")[g % 2], "rules_revision": ("r0", "r1")[g // 2 % 2],
           "value": w, "boundary": m, "comparator": ">=", "units": "u", "known_at": AS_OF, "payload": ({"limitation": "no IC"}, None, {})[s % 3]}
    row.pop(("monitor", "disposition", "window_id", "")[s // 3 % 4], None); return row
def _standing(M, standing: tuple[int, int, int], baseline: Optional[tuple[int, int]], readings: list[Rd], flag: Optional[tuple[int, int]], now: int):  # noqa: E302
    k, st, days = standing  # H8, H7: kind (unrecorded, revision, None: malformed), state, recorded days ago
    s = (UnrecordedStanding("carry", 3), StandingRevision("carry", 3, list(LifecycleState)[st % 4], 2, DAYS[days % 30], "census:x"), None)[k % 3]
    b = None if baseline is None else {"route": ("scored", None)[baseline[0] % 2], "traded_ic": ({"mean": 0.05}, 0.1, None)[baseline[0] // 2 % 3],
        "survival": ({"ratio": 0.8}, None)[baseline[1] % 2], "states": ([{"state": "calm"}, {"state": "stress"}], ["calm"], None)[baseline[1] // 2 % 3],
        **({"digest": "a" * 40} if baseline[1] // 6 % 2 else {})}
    f = None if flag is None else OpenFlag("carry", 3, "FEV", ("traded_horizon_ic breached on 3 consecutive windows", "plain")[flag[1] % 2],
                                          AGE[flag[0] % 30][flag[0] // 30 % 2], "dec")
    return repr(_call(M.signal_standing, s, b, [_rd(*r) for r in _n(readings)], f, (NOWS[now % 3], timedelta(days=14))))


IC = tuple[int, int, int, int, int]  # consumed IC: observed day, value, known day (offsets from 2026-09-01), kind (0 aware, 1 naive, 2 observed as a datetime, 3 window absent), horizon
WIN, D7 = Window("2026-09-05", date(2026, 9, 3), date(2026, 9, 5)), tuple(date(2026, 9, 1 + i) for i in range(7))  # concrete tables, as DAYS
K7, A7 = tuple(datetime(2026, 9, 1 + i, 12, tzinfo=UTC) for i in range(7)), tuple(datetime(2026, 9, 1 + i, 5, tzinfo=UTC) for i in range(7))
def _ic(d, v, k, kind, h):  # noqa: E302
    known, day = K7[k % 7], D7[d % 7]
    return NS(signal=NS(name="carry", version=3), horizon_days=1 + h % 2, knowledge_at=known.replace(tzinfo=None) if kind % 4 == 1 else known,
              window="absent" if kind % 4 == 3 else (NS(observed_at=A7[d % 7] if kind % 4 == 2 else day, available_at=known, value=(0.1, 0.2, 0.3)[v % 3]),))
def _window(M, consumed: list[IC], states: list[tuple[int, int]], ic_days: int):  # noqa: E302  H13: _lookback inlined; readings scripted
    rd = {D7[d % 7]: NS(entry=NS(state=NS(value=("calm", "stress")[s % 2]))) for d, s in _n(states)}
    return repr(_call(M.LiveEvidence(lambda cutoff: rd, [_ic(*c) for c in _n(consumed)], tuple(range(12))[ic_days % 12]).for_window, ("carry", 3, 1), WIN))


PAIRS = {  # driver, module, function path, planted mutant (line, old, new), renaming (first line, last line, locals), tiny domain (pairs.md)
    "compare": (_compare, "lifecycle_group", "_compare", (204, "        differs += not same", "        differs += same"),
                (197, 207, "lines|differs|again|now|same|verdict|row"), lambda a, b, c: ([(0, 0, 1, 2, 3 if a else 0), (1, 1, 1, 2, 3 if b else 1)], [(0, 1, 2, 0)] if c else [])),
    "standing": (_standing, "lifecycle_projection", "signal_standing", (79, '        if r["disposition"] == "not_run":', '        if r["disposition"] != "not_run":'),
                 (71, 139, "now|after|newest|basis|latest|shown|shown_readings|shown_flag|no_reading|r"), lambda a, b, c: ((1 if a else 0, 1, 3), None if b else (0, 0), [(0, 0, 2, 0, 0)] if c else [], None, 0)),
    "window": (_window, "lifecycle_evidence", "LiveEvidence.for_window", (162, "        traded = [c for c in mine if c.horizon_days == horizon]", "        traded = [c for c in mine if c.horizon_days != horizon]"),
               (150, 206, "ic_window|since|through|mine|series|recent|state_of|by_state|day|known|horizons|by_horizon"), lambda a, b, c: ([(1, 2, 3, 0 if a else 1, 0)] if c else [], [(2, 0)] if b else [], 10)),
}


def _scratch(pair, kind):  # "k": the planted one-line mutant; "e": the equivalent renaming; both on the candidate's text, never on disk
    _, mod, _, (line, old, new), (lo, hi, names), _ = PAIRS[pair]
    file = ROOTS["cand"] / "src/facdrone/shells" / (SCOPE[mod].replace(".", "/") + ".py"); src = file.read_text(encoding="utf-8").split("\n")
    if kind == "k": assert src[line - 1] == old, src[line - 1]; src[line - 1] = new  # noqa: E701
    else: src[lo - 1:hi] = [re.sub(rf"\b({names})\b", r"r_\1", x) for x in src[lo - 1:hi]]  # noqa: E701
    return _load(f"{kind}_{pair}", file, "\n".join(src), CAND.swap)


def _bind(name, fn, M, tiny=None):  # a module-level driver with the recorded domain as its signature (CrossHair and Hypothesis read it)
    def drv(*a, **kw): return fn(M, *tiny(*a, **kw)) if tiny else fn(M, *a, **kw)  # noqa: E704
    hints = {"a": bool, "b": bool, "c": bool} if tiny else {k: v for k, v in fn.__annotations__.items() if k != "M"}
    drv.__signature__ = inspect.Signature([inspect.Parameter(k, inspect.Parameter.POSITIONAL_OR_KEYWORD, annotation=v) for k, v in hints.items()], return_annotation=object)
    drv.__annotations__ = {**hints, "return": object}; drv.__name__ = drv.__qualname__ = name; globals()[name] = drv
if not LIGHT:  # noqa: E305  the trees, the scratch copies and the drivers; a parent that only spawns CrossHair skips this
    ORIG, CAND = _tree("orig"), _tree("cand")
    for _p, (_fn, _mod, _path, _, _, _tiny) in PAIRS.items():
        _e = _scratch(_p, "e")
        for _pre, _M in (("o", getattr(ORIG, _mod)), ("c", getattr(CAND, _mod)), ("k", _scratch(_p, "k")), ("e", _e)): _bind(f"{_pre}_{_p}", _fn, _M)  # noqa: E701
        _bind(f"ct_{_p}", _fn, getattr(CAND, _mod), _tiny); _bind(f"et_{_p}", _fn, _e, _tiny)


def smoke(*args) -> object:  # gate 1: E5_SMOKE names the pair; both sides run under a trivial postcondition with entry markers
    """post: True"""
    return globals()["o_" + os.environ["E5_SMOKE"]](*args), globals()["c_" + os.environ["E5_SMOKE"]](*args)
if os.environ.get("E5_SMOKE") and not LIGHT:  # noqa: E305
    _o = globals()["o_" + os.environ["E5_SMOKE"]]; smoke.__signature__, smoke.__annotations__ = _o.__signature__, _o.__annotations__
    for _tag, _T in (("orig", ORIG), ("cand", CAND)):
        _fn, _mod, _path, *_ = PAIRS[os.environ["E5_SMOKE"]]; _holder = getattr(_T, _mod)
        for _part in _path.split(".")[:-1]: _holder = getattr(_holder, _part)  # noqa: E701
        setattr(_holder, _path.split(".")[-1], (lambda f, t: lambda *a, **k: (print("ENTER " + t, file=sys.stderr), f(*a, **k))[1])(getattr(_holder, _path.split(".")[-1]), _tag))


def corpus(a, b, n, seed):  # gate 2 (and the light form of 3a and 3b): n Hypothesis inputs from the driver signature, fixed seed
    from hypothesis import HealthCheck, given, seed as fix, settings, strategies as st
    fa, fb = globals()[a], globals()[b]; res = {"a": a, "b": b, "n": n, "seed": seed, "calls": 0, "mismatches": []}

    @fix(seed)
    @settings(max_examples=n, database=None, deadline=None, suppress_health_check=list(HealthCheck))
    @given(st.fixed_dictionaries({k: st.from_type(t) for k, t in fa.__annotations__.items() if k != "return"}))
    def run(kw):
        res["calls"] += 1; x, y = fa(**kw), fb(**kw)
        if x != y: res["mismatches"].append({"args": repr(kw), a: x[:600], b: y[:600]})  # noqa: E701
    run(); res["first"] = res["mismatches"][0] if res["mismatches"] else None; return res


def _crosshair(args, env=None):  # E1c-2 Amendment 4 launcher: the drivers are imported before CrossHair's main (audit wall)
    r = subprocess.run([sys.executable, "-c", LAUNCH, *args], cwd=HERE, env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1", **(env or {})},
                       capture_output=True, text=True, encoding="utf-8", errors="replace")
    return r.returncode, r.stdout + r.stderr


if __name__ == "__main__":
    mode, p, opt = sys.argv[1], (sys.argv[2:3] or [""])[0], dict(zip(sys.argv[3::2], sys.argv[4::2]))
    n, seed, kind = int(opt.get("--n", 300)), int(opt.get("--seed", 20261106)), opt.get("--kind", "pair")
    if mode == "--list":
        res = {q: {"domain": str(globals()["o_" + q].__signature__), "sides": [f"{x}_{q}" for x in ("o", "c", "k", "e", "ct", "et")]} for q in PAIRS}
    elif mode == "--smoke":
        code, text = _crosshair(["check", "--per_condition_timeout", opt.get("--t", "15"), "e5_drivers.smoke"], {"E5_SMOKE": p})
        res = {"pair": p, "per_condition_timeout": opt.get("--t", "15"), "exit": code, "entries": {t: text.count(f"ENTER {t}\n") for t in ("orig", "cand")},
               "output": [x for x in text.splitlines() if not x.startswith("ENTER ")][-30:]}; res["symbolic"] = min(res["entries"].values()) > 1
    elif mode in ("--corpus", "--plant", "--rename"):
        a, b = {"--corpus": ("o", "c"), "--plant": ("c", "k"), "--rename": ("c", "e")}[mode]; res = corpus(f"{a}_{p}", f"{b}_{p}", n, seed)
        res["verdict"] = {"--corpus": ("agree", "differ"), "--plant": ("NOT FOUND", "found"), "--rename": ("no spurious", "SPURIOUS")}[mode][bool(res["mismatches"])]
    elif mode == "--solve":  # phase 2 only; the per-condition timeout is the scaled deadline, the receipt's --timeout the backstop
        x, y = {"pair": ("o", "c"), "plant": ("k", "c"), "rename": ("et", "ct")}[kind]
        code, text = _crosshair(["diffbehavior", "--per_condition_timeout", opt["--deadline"], f"e5_drivers.{x}_{p}", f"e5_drivers.{y}_{p}"])
        res = {"pair": p, "kind": kind, "sides": [f"{x}_{p}", f"{y}_{p}"], "deadline_s": float(opt["--deadline"]), "exit": code, "output": text,
               "verdict": "counterexample" if "Given:" in text else "exhausted" if "exhausted" in text else "inconclusive"}
    elif mode == "--replay":  # gate 4: every counterexample concretely, both sides, from the solver log
        import ast
        res = []
        for log in sys.argv[3:]:
            text = Path(log).read_text(encoding="utf-8"); fs = re.findall(r"e5_drivers\.(\w+) : (?:returns|raises)", text)[:2]
            kw = {k.arg: ast.literal_eval(k.value) for k in ast.parse(f"f{re.search(r'Given: (\(.*\)),', text).group(1)}", mode="eval").body.keywords}
            r = [globals()[f](**kw) for f in fs]; res.append({"log": Path(log).name, "args": repr(kw), fs[0]: r[0], fs[1]: r[1], "differ": r[0] != r[1]})
    elif mode == "--gates":  # the per-pair summary so far; solve and replay stay pending until phase 2 writes their files
        res = {q: {g: (json.loads((OUT / f"e5-{f % q}.json").read_text(encoding="utf-8")) if (OUT / f"e5-{f % q}.json").exists() else "pending")
                   for g, f in (("smoke", "smoke-%s"), ("corpus", "corpus-%s"), ("plant", "plant-%s"), ("rename", "rename-%s"), ("solve_plant", "solve-%s-plant"),
                                ("solve_rename", "solve-%s-rename"), ("solve_pair", "solve-%s-pair"), ("replay", "replay-%s"))} for q in PAIRS}
        res = {q: {g: v if isinstance(v, (str, list)) else {k: w for k, w in v.items() if k not in ("mismatches", "output")} for g, v in gs.items()} for q, gs in res.items()}
    if mode != "--list":
        (OUT / f"e5-{mode[2:]}{'-' + p if p else ''}{'-' + kind if mode == '--solve' else ''}.json").write_text(json.dumps(res, indent=1, default=str), encoding="utf-8")
    print(json.dumps(res if isinstance(res, list) or mode in ("--list", "--gates") else {k: v for k, v in res.items() if k not in ("mismatches", "output")}, indent=1, default=str))
