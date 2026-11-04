"""E3 step 3, T5: CrossHair differential, original (frozen copy, loaded as orig_*) vs candidate pure core (cand/src).
Pairs o_X / c_X take identical arguments and return a canonical repr; an exception is ("raise", type, message).
  dedup    deduplicated_inputs on a list (a one-shot iterator, review H1, is outside the domain by construction)
  capture  capture_payload on a fresh ConsumedCapture, with capture.absences afterwards (review H4)
  seal     seal_evidence / seal_evidence_with_run_slices, the folded `_seal` (review H7; H8 signal checks)
Controls: k_dedup (candidate, line 135 `!=` -> `>`) must differ from c_dedup; e_tiny (candidate, locals renamed)
must exhaust against c_tiny. Bounds by construction (E1c-2 drivers2): strings over `abc`, length <= 8 via zip,
lists <= 3, enumerations as int indexes, no slicing of symbolic strings, no rejection guard.
  t5_drivers.py sound | replay LOG...    crosshair diffbehavior t5_drivers.o_dedup t5_drivers.c_dedup"""
import base64, importlib.util, inspect, json, os, re, sys, types, typing, zlib
from dataclasses import replace
from pathlib import Path
from typing import Optional
sys.dont_write_bytecode = True  # importing the original must not write .pyc into the frozen copy
import facdrone.shells.decision_evidence_publication as CAND  # noqa: E402
from facdrone.shells.policy_trace import ConsumedCapture  # noqa: E402
from facdrone_kernel.domain.decision_evidence import ConsumedInput, InputSlice, SignalVersion  # noqa: E402
from facdrone_kernel.domain.mandate import Currency  # noqa: E402
from facdrone_testing.core.consumed_evidence import NOW, UNKNOWN  # noqa: E402
from facdrone_testing.core.evidence import evidence_batch  # noqa: E402
HERE, STORE = Path(__file__).resolve().parent, "facdrone.shells.decision_evidence_store"
FROZEN = HERE.parents[3] / ".runs/e2-facdrone/source/src/facdrone/shells"
assert "e3-facdrone" in CAND.__file__, CAND.__file__


def _load(name: str, file: Path | None = None, text: str | None = None, store=None) -> types.ModuleType:
    spec = importlib.util.spec_from_file_location(name, file) if file else None
    m = sys.modules[name] = importlib.util.module_from_spec(spec) if spec else types.ModuleType(name)
    saved, sys.modules[STORE] = sys.modules[STORE], store or sys.modules[STORE]  # the original binds the original store
    try:
        spec.loader.exec_module(m) if spec else exec(compile(text, f"<{name}>", "exec"), m.__dict__)  # noqa: S102
    finally:
        sys.modules[STORE] = saved
    return m


ORIG = _load("orig_pub", FROZEN / "decision_evidence_publication.py", store=_load("orig_store", FROZEN / "decision_evidence_store.py"))
SRC = Path(CAND.__file__).read_text(encoding="utf-8").split("\n")
assert SRC[134].strip() == "if digest is not None and first[digest] != index else item", SRC[134]
KILL = _load("pub_kill", text="\n".join(SRC[:134] + [SRC[134].replace("!=", ">")] + SRC[135:]))
EQUIV = _load("pub_equiv", text="\n".join(SRC[:126] + [re.sub(r"\b(digests?|first|index|item)\b", r"r_\1", x) for x in SRC[126:137]] + SRC[137:]))
Str = list[int]
Cell = tuple[int, int, Str]  # kind (str, int, -n/4 incl. -0.0, nan|inf, bool, None, bytes), number, string
Item = tuple[int, Cell, list[Str], list[list[Cell]]]  # kind (new slice, the previous slice object, scalar), cell, columns, rows


def _s(cs) -> str:
    return "".join(chr(97 + c % 3) for _, c in zip(range(8), cs))


def _n(xs, k: int = 3) -> list:
    return [x for _, x in zip(range(k), xs)]


def _cell(c: Cell, kinds: int):
    return (_s(c[2]), c[1], -c[1] / 4, float("nan") if c[1] % 2 else float("inf"), c[1] % 2 == 1, None, b"x")[c[0] % kinds]


def _inputs(items, kinds: int = 7) -> list:
    out, last = [], None
    for k, c, cols, rows in _n(items):
        if k % 3 == 0 or last is None and k % 3 == 1:
            value = last = InputSlice(tuple(_s(x) for x in _n(cols)), tuple(tuple(_cell(x, kinds) for x in _n(r)) for r in _n(rows)))
        else:
            value = last if k % 3 == 1 else _cell(c, kinds)
        out.append(ConsumedInput("prices", "GC=F", value, "USD", NOW, UNKNOWN, "unknown"))
    return out


def _call(f, *a, **kw) -> object:
    try:
        return f(*a, **kw)
    except Exception as e:  # noqa: BLE001
        return ("raise", type(e).__name__, str(e))


def _capture(M, items) -> str:
    cap = ConsumedCapture("pending", currency=Currency.EUR)
    cap.inputs.extend(_inputs(items))
    return repr((_call(M.capture_payload, cap), cap.absences))


B, R = evidence_batch(), evidence_batch(rule=True)


def _seal(M, items, sigs, prem, rule, ns):
    signals = tuple(SignalVersion(_s(n), v) for n, v in _n(sigs))
    pv = tuple((signals[p % len(signals)], 2) for p in _n(prem, 1)) if signals else ()
    routes = (("scored", replace(B.route, premise_versions=pv)),) + ((("rule_book:r", replace(R.route, premise_versions=pv)),) if rule else ())
    cap = ORIG._Captured(B.stages, (), tuple(_inputs(items, 6)), routes, signals, (("r", signals),), B.absences)
    payload = {"as_of": NOW.isoformat(), "consumed_capture": base64.b64encode(zlib.compress(ORIG._CAPTURE.dump_json(cap), 1)).decode()}
    r = M.seal_evidence(payload, scope="live", execution_id="decision:test", code="code-sha", configuration="config-sha") if ns is None \
        else M.seal_evidence_with_run_slices(payload, M.EvidenceBinding("live", "decision:test", _s(ns), "code-sha", "config-sha"))
    return r.details, [(s.canonical, s.stored, s.digest, s.schema_version) for s in r.batches or ()], r.slices


TINY = (InputSlice(("a",), ((1,),)), InputSlice(("a",), ((2,),)))
def o_dedup(items: list[Item]) -> object: return repr(_call(ORIG.deduplicated_inputs, _inputs(items)))  # noqa: E302,E704
def c_dedup(items: list[Item]) -> object: return repr(_call(CAND.deduplicated_inputs, _inputs(items)))  # noqa: E704
def k_dedup(items: list[Item]) -> object: return repr(_call(KILL.deduplicated_inputs, _inputs(items)))  # noqa: E704
def c_tiny(a: bool, b: bool, c: bool) -> object: return repr(_call(CAND.deduplicated_inputs, [ConsumedInput("p", "k", TINY[x], "u", NOW, UNKNOWN, "unknown") for x in (a, b, c)]))  # noqa: E501,E704
def e_tiny(a: bool, b: bool, c: bool) -> object: return repr(_call(EQUIV.deduplicated_inputs, [ConsumedInput("p", "k", TINY[x], "u", NOW, UNKNOWN, "unknown") for x in (a, b, c)]))  # noqa: E501,E704
def o_capture(items: list[Item]) -> object: return _capture(ORIG, items)  # noqa: E704
def c_capture(items: list[Item]) -> object: return _capture(CAND, items)  # noqa: E704
def o_seal(items: list[Item], sigs: list[tuple[Str, int]], prem: list[int], rule: bool, ns: Optional[Str]) -> object: return repr(_call(_seal, ORIG, items, sigs, prem, rule, ns))  # noqa: E501,E704
def c_seal(items: list[Item], sigs: list[tuple[Str, int]], prem: list[int], rule: bool, ns: Optional[Str]) -> object: return repr(_call(_seal, CAND, items, sigs, prem, rule, ns))  # noqa: E501,E704


def smoke(*args) -> object:  # T5_SMOKE names the pair; both sides run under a trivial postcondition
    """post: True"""
    return globals()["o_" + os.environ["T5_SMOKE"]](*args), globals()["c_" + os.environ["T5_SMOKE"]](*args)


if os.environ.get("T5_SMOKE"):
    smoke.__signature__ = inspect.signature(globals()["o_" + os.environ["T5_SMOKE"]])
    smoke.__annotations__ = globals()["o_" + os.environ["T5_SMOKE"]].__annotations__
    for M, fn in [(M, fn) for M in (ORIG, CAND) for fn in ("deduplicated_inputs", "capture_payload", "seal_evidence", "seal_evidence_with_run_slices")]:
        setattr(M, fn, (lambda f, tag: lambda *a, **k: (print("ENTER " + tag, file=sys.stderr), f(*a, **k))[1])(getattr(M, fn), f"{M.__name__}.{fn}"))

if __name__ == "__main__" and sys.argv[1] == "sound":  # >= 300 Hypothesis inputs per pair, fixed seed
    from hypothesis import HealthCheck, given, seed, settings, strategies as st
    out = {}

    def corpus(p: str, res: dict):
        hints = typing.get_type_hints(globals()["o_" + p])
        hints.pop("return")

        @seed(20261104)
        @settings(max_examples=300, database=None, deadline=None, suppress_health_check=list(HealthCheck))
        @given(st.fixed_dictionaries({k: st.from_type(t) for k, t in hints.items()}))
        def run(kw):
            res["calls"] += 1
            o, c = globals()["o_" + p](**kw), globals()["c_" + p](**kw)
            res["mismatches"] += [] if o == c else [{"args": repr(kw), "o": o[:400], "c": c[:400]}]
        return run
    for p in ("dedup", "capture", "seal"):
        out[p] = res = {"calls": 0, "mismatches": []}
        corpus(p, res)()
        print(p, res["calls"], len(res["mismatches"]), flush=True)
    (HERE / "results" / "t5-soundness.json").write_text(json.dumps({"seed": 20261104, **out}, indent=1), encoding="utf-8")
elif __name__ == "__main__" and sys.argv[1] == "replay":  # check (4): every counterexample, concretely, both sides
    import ast
    for log in sys.argv[2:]:
        text = Path(log).read_text(encoding="utf-8")
        o, c = re.findall(r"t5_drivers\.(\w+) : (?:returns|raises)", text)[:2]
        call = ast.parse(f"f{re.search(r'Given: (\(.*\)),', text).group(1)}", mode="eval").body
        kw = {k.arg: ast.literal_eval(k.value) for k in call.keywords}
        r = [globals()[f](**kw) for f in (o, c)]
        print(json.dumps({"log": Path(log).name, "args": repr(kw), o: r[0], c: r[1], "differ": r[0] != r[1]}))
