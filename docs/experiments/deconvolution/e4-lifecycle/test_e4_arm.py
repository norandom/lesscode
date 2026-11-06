"""6.4 apparatus checks for what e4_arm.py adds to t31_arm's logic: the E4 classification (no R16 here; reasons are the
first FAILED or ERROR node with its assertion, the replay's first divergence, INVALID scope, a SyntaxError, a timeout) and
the parse of run_e4's status line. Never touches the .runs trees."""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import e4_arm  # noqa: E402

DIVERGE = ("E   AssertionError: S02 op 7 {\"op\": \"standing\"}: result.census: None != 'enrollment:operator'\n"
           "FAILED ../../../docs/experiments/deconvolution/e4-lifecycle/test_e4_replay.py::test_e4_replay - AssertionError: S02...\n")


def test_classify():
    c = e4_arm.classify
    assert c(0, "") == ("survived", "")
    assert c(2, "run_e4: INVALID scope not imported from x\n  SyntaxError: invalid syntax") == ("incompetent", "scope not imported from the mutant copy (SyntaxError)")
    assert c(2, "run_e4: INVALID scope not imported")[0] == "incompetent"
    assert c(3, "")[0] == "apparatus-fault" and c(4, "")[0] == "apparatus-fault"
    assert c("timeout", "")[1] == "timeout" and c(1, "run_arm: deadline exceeded")[1] == "timeout"
    assert c(1, "E   SyntaxError: invalid\nERROR tests/unit/x.py\n1 error during collection") == ("incompetent", "SyntaxError at collection")
    out, why = c(1, DIVERGE)
    assert out == "killed" and why.startswith("FAILED ../../../docs/experiments/deconvolution/e4-lifecycle/test_e4_replay.py::test_e4_replay - AssertionError: S02 op 7")
    assert c(1, "FAILED tests/unit/t.py::test_a - assert 1 == 0\n")[1] == "FAILED tests/unit/t.py::test_a"
    assert c(1, "nothing parsable") == ("killed", "exit 1")


def test_parse_arm_status():
    assert e4_arm.parse_arm("run_e4 .: exit 0 in 3s\nrun_e4: exit 0; replay file fresh=True sequences=50/50") == (0, True)
    assert e4_arm.parse_arm("run_e4: exit 1; replay file fresh=False sequences=None/None") == (1, False)
    assert e4_arm.parse_arm("killed before the status line") == (None, False)
    assert e4_arm.SEVEN[0].endswith("lifecycle_monitor_runner.py") and len(e4_arm.SEVEN) == 7
    assert e4_arm.TREES["orig"][2] == "e4o" and e4_arm.TREES["cand"][2] == "e4c"
