"""Patching sentinel check (E4 plan 6.2 step 2e, kill criterion 2b): `patchcheck.py ROOT --out O.json`. Tier 1: each patched
name is a key of vars(module) (dotted: an attribute of that binding) and no function body of its module re-imports, re-binds, takes
as a parameter or defaults it; the instance attributes exist on a stub-built MonitorRunner and LifecycleStore. Tier 2: the name is
replaced by a PatchSentinel-raising callable, a store-free trigger runs with the other names faked, and the sentinel must propagate.
No database; ROOT/src is first on sys.path and the modules must import from ROOT (fail closed). Exit 0 iff every check passes."""
import ast, hashlib, json, os, sys
from contextlib import ExitStack; from dataclasses import asdict; from datetime import date; from pathlib import Path  # noqa: E702
from types import SimpleNamespace as NS; from unittest.mock import patch  # noqa: E702
ROOT = Path(sys.argv[1]).resolve(); os.environ["FACDRONE_PROJECT_ROOT"] = str(ROOT); sys.path.insert(0, str(ROOT / "src"))  # noqa: E702
from typer.testing import CliRunner  # noqa: E402
from facdrone.shells import lifecycle_evidence as EV, lifecycle_monitor_runner as RN, lifecycle_store as ST  # noqa: E402
from facdrone.shells.cli import lifecycle_group as GR  # noqa: E402
from facdrone.shells.cli.app import app  # noqa: E402
from facdrone.shells.wiring import lifecycle_root as RT  # noqa: E402
from facdrone.pipelines.evaluation.monitor_windows import Window  # noqa: E402
from facdrone_testing.core.cppi import AS_OF  # noqa: E402  (test infrastructure, outside the scope)
from facdrone_testing.core.lifecycle import BASELINE, BREACH, DECLARED  # noqa: E402
MODS = {"lifecycle_group": GR, "lifecycle_evidence": EV, "lifecycle_root": RT, "lifecycle_store": ST, "lifecycle_monitor_runner": RN}
assert all(Path(m.__file__).resolve().is_relative_to(ROOT) for m in MODS.values()), "scope not imported from ROOT"
TIER1 = {"lifecycle_group": ["_cited_baseline", "_lifecycle_store", "_pod_of", "configured_pods", "read_parameters"], "lifecycle_store": ["LifecycleStore"],
         "lifecycle_evidence": ["_consumed", "baseline_view", "live_evidence", "LiveEvidence.for_window"], "lifecycle_root": ["project_root", "study_baseline"]}
PatchSentinel = type("PatchSentinel", (Exception,), {"__doc__": "raised by the sentinel; it must propagate out of the trigger"})
sentinel = lambda *_a, **_k: (_ for _ in ()).throw(PatchSentinel("looked up at call time"))  # noqa: E731
def rebound(tree, name) -> list:  # function bodies of the module that bind the name locally (store, import, parameter, default)
    fns = [n for n in ast.walk(tree) if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda))]
    return [f"line {n.lineno}" for fn in fns for n in ast.walk(fn) if (isinstance(n, ast.Name) and n.id == name and isinstance(n.ctx, ast.Store))
            or (isinstance(n, ast.alias) and (n.asname or n.name) == name) or (isinstance(n, ast.arg) and n.arg == name)] + \
           [f"default line {d.lineno}" for fn in fns for d in fn.args.defaults + fn.args.kw_defaults if isinstance(d, ast.Name) and d.id == name]
def check1(mod, name) -> dict:
    tree, scope, (head, *rest) = ast.parse(Path(MODS[mod].__file__).read_text(encoding="utf-8")), vars(MODS[mod]), name.split(".")
    return {"bound": head in scope and all(p in vars(scope[head]) for p in rest), "local_rebinding": [x for p in (head, *rest) for x in rebound(tree, p)]}
def under(fakes, trigger):  # run the trigger with the fakes in place; the exception that came out (a CliRunner result's too), or None
    with ExitStack() as s:
        [s.enter_context(patch.object(o, n, r)) for o, n, r in fakes]
        try:
            return getattr(trigger(), "exception", None)
        except Exception as e:  # noqa: BLE001 — any exception is the outcome
            return e
propagates = lambda target, name, trigger, *fakes: isinstance(under([*fakes, (target, name, sentinel)], trigger), PatchSentinel)  # noqa: E731
tier1 = {f"{m}.{n}": check1(m, n) for m, ns in TIER1.items() for n in ns}
srcs = lambda: RN.MonitorSources(lambda: (), lambda _w: (lambda *_: "no evidence"), lambda *_: None)  # noqa: E731
attrs = {"MonitorRunner": {a: hasattr(RN.MonitorRunner(NS(), DECLARED, srcs()), a) for a in ("_rules", "_append", "_evidence", "_monitored", "_engine")},
         "LifecycleStore": {a: hasattr(ST.LifecycleStore(NS()), a) for a in ("_engine", "standing")}}
W, DIGEST, REVISION = Window("2026-09-05", date(2026, 9, 3), date(2026, 9, 5)), "b" * 64, hashlib.sha256(json.dumps({"method": "dated-ic-v2", **asdict(DECLARED)}, sort_keys=True).encode()).hexdigest()[:16]
ROW = {"window_id": "2026-09-05", "rules_revision": REVISION, "baseline_digest": DIGEST, "monitor": "traded_horizon_ic", "disposition": "breach", "value": -0.1, "boundary": 0.025}
CLI = {(GR, "_lifecycle_store"): lambda: NS(_engine=None, readings=lambda *_: [ROW]), (GR, "read_parameters"): lambda _: {"lifecycle_monitor": asdict(DECLARED)},
       (GR, "_cited_baseline"): lambda *_: {"digest": DIGEST, "traded_horizon_days": 1}, (GR, "configured_pods"): lambda **_: NS(get_scored=lambda _: NS(dynamics=None)),
       (GR, "_pod_of"): lambda *_: "fixture", (EV, "baseline_view"): lambda _: BASELINE, (EV, "live_evidence"): lambda *_: BREACH}
readings = lambda: CliRunner().invoke(app, ["lifecycle", "readings", "carry@3", "--window", "2026-09-05"])  # noqa: E731
exc = under([(o, n, f) for (o, n), f in CLI.items()], readings)  # control: with every fake in place the command reaches its own exit
control = {"exception": None if exc is None or isinstance(exc, SystemExit) else repr(exc), "exit": repr(exc)}
via_runner = lambda attr: (lambda r: propagates(r, attr, lambda: r.run(AS_OF, "d")))(RN.MonitorRunner(NS(), DECLARED, srcs()))  # noqa: E731
tier2 = {"lifecycle_evidence._consumed": propagates(EV, "_consumed", lambda: EV.read_evidence(None, lambda d: {}, (W,), 1)),
         "lifecycle_evidence.LiveEvidence.for_window": propagates(EV.LiveEvidence, "for_window", lambda: EV.live_evidence(None, lambda d: {}, ("carry", 3, 1), W, 1), (EV, "_consumed", lambda *_: [])),
         **{f"{o.__name__.rsplit('.', 1)[1]}.{n}": propagates(o, n, readings, *[(p, q, f) for (p, q), f in CLI.items() if (p, q) != (o, n)]) for (o, n) in CLI},
         "MonitorRunner._monitored": via_runner("_monitored"), "MonitorRunner._evidence": via_runner("_evidence")}
fails = [k for k, v in tier1.items() if not v["bound"] or v["local_rebinding"]] + [f"{c}.{a}" for c, d in attrs.items() for a, v in d.items() if not v] + [k for k, v in tier2.items() if not v] + (["cli_control"] if control["exception"] else [])
res = {"root": str(ROOT), "modules": {k: str(Path(m.__file__).relative_to(ROOT)) for k, m in MODS.items()}, "tier1": tier1, "instance_attributes": attrs, "cli_control": control, "tier2": tier2, "failed": fails, "pass": not fails}
Path(sys.argv[sys.argv.index("--out") + 1]).write_text(json.dumps(res, indent=1), encoding="utf-8")
sys.exit(print(json.dumps({"pass": not fails, "failed": fails, "cli_control": control})) or (0 if not fails else 1))
