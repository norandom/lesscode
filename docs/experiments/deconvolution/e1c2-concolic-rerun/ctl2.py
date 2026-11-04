"""Amendment 2 verifier. Control (b): `ctl2.o_ctl` / `ctl2.m_ctl` call decision_evidence_publication.capture_payload,
original vs the line-191 annotation mutant (`-> str - None`) compiled in memory from the frozen file's text.
Check (4): `ctl2.py replay LOG...` re-runs each "Given:" counterexample concretely through both drivers."""
import ast, json, re, sys, types
from pathlib import Path
from facdrone.shells import decision_evidence_publication as P
from facdrone.shells.policy_trace import ConsumedCapture
from facdrone_kernel.domain.mandate import Currency
from facdrone_testing.core.evidence import evidence_batch
SRC = Path(P.__file__).read_text(encoding="utf-8").split("\n")
OLD, NEW = "def capture_payload(capture: ConsumedCapture | None) -> str | None:", "def capture_payload(capture: ConsumedCapture | None) -> str - None:"
assert SRC[190] == OLD, SRC[190]  # cosmic-ray's exact diff line (E1b cr-arm-b.jsonl, BitOr_Sub occurrence 1)
SRC[190] = NEW
M = sys.modules["pub_m191"] = types.ModuleType("pub_m191")
exec(compile("\n".join(SRC), "<pub_m191>", "exec"), M.__dict__)  # noqa: S102
CAP = ConsumedCapture("pending", currency=Currency.EUR)  # as tests/unit/test_transient_capture_codec.py builds one
CAP.inputs.extend(evidence_batch().inputs), CAP.absences.extend(evidence_batch().absences)
def o_ctl(present: bool) -> object: return P.capture_payload(CAP if present else None)  # noqa: E704
def m_ctl(present: bool) -> object: return M.capture_payload(CAP if present else None)  # noqa: E704
def _call(f, kw):
    try:
        return repr(f(**kw))
    except Exception as e:  # noqa: BLE001
        return f"raises {type(e).__name__}: {e}"
if __name__ == "__main__" and sys.argv[1] == "replay":
    import drivers2
    for log in sys.argv[2:]:
        text = Path(log).read_text(encoding="utf-8")
        o, m = re.findall(r"drivers2\.(\w+) : (?:returns|raises)", text)[:2]
        call = ast.parse(f"f{re.search(r'Given: (\(.*\)),', text).group(1)}", mode="eval").body
        kw = {k.arg: ast.literal_eval(k.value) for k in call.keywords}
        out = [_call(getattr(drivers2, f), kw) for f in (o, m)]
        print(json.dumps({"log": Path(log).name, "args": repr(kw), o: out[0], m: out[1], "differ": out[0] != out[1]}))
