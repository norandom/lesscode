# S10 attempt-2 adverse controls (U3, R31): every check must FAIL CLOSED.
# Same control set as s10_controls.py; differences: receipt name, planted-L1
# reads the attempt-2 recording (proto2-e4-a.json), admission runs the
# descoped lesscode-core binary. Receipt written regardless.
import hashlib
import json
import subprocess
import sys
import tempfile
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
DC = HERE.parent
sys.path[:0] = [str(HERE), str(DC / "e1b-facdrone-evidence-publication"), str(DC / "e4-lifecycle"),
                str(DC / "e2-facdrone-edge-census"), str(DC.parents[2] / ".runs/e1b-facdrone/extras"),
                str(DC.parents[2] / "src")]
import lesscode_baseline.recording as P  # noqa: E402
import scope_e3, scope_e4  # noqa: E402

OUT = HERE / "results" / "s10-controls-2.json"
CONTROLS = []


def control(name, ok, detail):
    CONTROLS.append({"control": name, "fails_closed": ok, "detail": detail})
    print(f"{'PASS' if ok else 'FAIL'} {name}: {detail}")


def c_verdict_rules():
    code, r = P.verdict(0, True, 0, None)
    control("accept-clean", code == 0 and not r, f"exit {code} {r}")
    code, r = P.verdict(0, False, 0, None)
    control("stale-replay-refused", code == 1 and any("fresh" in x for x in r), f"exit {code} {r}")
    code, r = P.verdict(0, True, 3, None)
    control("l2-without-query-evidence-refused", code == 1 and any("L2" in x for x in r), f"exit {code} {r}")
    code, r = P.verdict(0, True, 3, scope_e4.QUERY_EVIDENCE)
    control("l2-with-bound-evidence-admissible", code == 0, f"exit {code} {r} (bound file {scope_e4.QUERY_EVIDENCE.name})")
    code, r = P.verdict(2, True, 0, None)
    control("invalid-arm-stays-invalid", code == 2, f"exit {code} {r}")


def c_guard_voids():
    cand = DC.parents[2] / ".runs/e3-facdrone/cand"
    saved = scope_e3.O.ROOT
    try:  # the record path guards O.ROOT; point it at the changed candidate root
        scope_e3.O.ROOT = cand
        scope_e3.guard()
        control("protected-edit-voids-record", False, "guard did not raise on changed root")
    except RuntimeError as e:
        control("protected-edit-voids-record", True, str(e)[:120])
    finally:
        scope_e3.O.ROOT = saved


def c_interrupted_arm():
    t0 = time.monotonic()
    code = P.call([sys.executable, "-c", "import time; time.sleep(30)"], t0 + 2)
    control("interrupted-arm-not-a-kill", code == 124 and (time.monotonic() - t0) < 20,
            f"exit {code} in {time.monotonic() - t0:.0f}s (infrastructure, no verdict)")


def c_planted_l1():
    golden = json.loads((DC.parents[2] / ".runs/native-prototype/proto2-e4-a.json").read_text(encoding="utf-8"))
    victim = golden["sequences"][0]
    if victim.get("observations"):
        victim["observations"][0]["planted_control_difference"] = True
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as fh:
        json.dump(golden, fh)
        doctored = Path(fh.name)
    try:
        with P.embedded_engine(DC / "e2-facdrone-edge-census") as engine:
            d = scope_e4.P.replay_engine(scope_e4.O.Env, engine, scope_e4.O.ROOT, dict(scope_e4.O.SEEDS),
                                         doctored, listen_all=True)
        control("planted-l1-difference-detected", d is not None, str(d)[:120])
    finally:
        doctored.unlink(missing_ok=True)


def c_missing_artifact():
    with tempfile.TemporaryDirectory(prefix="s10-missing-") as td:
        tdp = Path(td)
        (tdp / "b.json").write_text('{"bindings": [{"artifact": "absent-control-artifact.json", "sidecars": ["absent-control-artifact.json.sha256"]}]}', encoding="utf-8")
        exe = DC.parents[2] / "native" / "target" / "debug" / "lesscode-core.exe"
        p = subprocess.run([str(exe), "admission", "--root", str(tdp), "--bindings", str(tdp / "b.json"),
                            "--out", str(tdp / "r.json")], capture_output=True, text=True)
        missing = "absent-control-artifact.json" in p.stdout and "missing" in p.stdout
        control("missing-artifact-refused", p.returncode == 1 and missing,
                f"admission exit {p.returncode}: {[l.strip() for l in p.stdout.splitlines()[:1]]}")


if __name__ == "__main__":
    c_verdict_rules()
    c_missing_artifact()
    c_guard_voids()
    c_interrupted_arm()
    if "--with-pg" in sys.argv:
        c_planted_l1()
    body = json.dumps({"receipt": "s10-controls-2", "attempt": 2, "controls": CONTROLS,
                       "all_fail_closed": all(c["fails_closed"] for c in CONTROLS)}, indent=1)
    OUT.write_text(body, encoding="utf-8", newline="\n")
    Path(str(OUT) + ".sha256").write_text(f"{hashlib.sha256(body.encode()).hexdigest()}  s10-controls-2.json\n",
                                          encoding="ascii", newline="\n")
    print(f"receipt: {OUT}")
    sys.exit(0 if all(c["fails_closed"] for c in CONTROLS) else 1)
