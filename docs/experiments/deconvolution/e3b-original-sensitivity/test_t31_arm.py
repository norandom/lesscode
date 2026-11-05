"""Task 3.1 apparatus checks: deterministic sampling (seed 20261104, sorted keys, sorted draw) and hunk
application plus restore-and-verify on a temp copy of a small fake file. Never touches the .runs trees."""
import random, shutil, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import t31_arm  # noqa: E402

FAKE = {("m.py", f"core/Op{i % 7}", '"{}"', i % 3, 10 + i, i % 5): f"job{i}" for i in range(317)}


def test_draw_is_deterministic_sized_and_sorted():
    a = t31_arm.draw(FAKE)
    keys = list(FAKE)
    random.Random(1).shuffle(keys)
    assert a == t31_arm.draw(dict.fromkeys(keys)) == t31_arm.draw(FAKE, 20261104, 30)  # order of the population is irrelevant
    assert len(a) == 30 and len(set(a)) == 30 and a == sorted(a) and set(a) <= set(FAKE)
    assert a == sorted(random.Random(20261104).sample(sorted(FAKE), 30))  # E3's rule, written out
    assert a != t31_arm.draw(FAKE, 20261105, 30)
    assert (t31_arm.SEED, t31_arm.N) == (20261104, 30)


def test_hunks_apply_and_restore_verified(tmp_path):
    src = "a = 1\nif (x\n    or y):\n    pass\nb = 2\n"
    mutated = "a = 1\nif not (x\n    or y):\n    pass\nb = 3\n"
    hunks, old = t31_arm.hunks_of(src, mutated)
    assert old == [["if (x"], ["b = 2"]] and [h[2] for h in hunks] == [["if not (x"], ["b = 3"]]
    ref, mut = tmp_path / "ref.py", tmp_path / "mut.py"
    ref.write_bytes(src.encode("utf-8"))
    shutil.copy2(ref, mut)
    t31_arm.apply_hunks(mut, hunks)
    assert mut.read_bytes().decode("utf-8") == mutated
    assert t31_arm.restore(mut, ref) is True and t31_arm.sha(mut) == t31_arm.sha(ref)


def test_classify():
    c = t31_arm.classify
    assert c(0, "")[0] == "survived" and c(1, "R16 harness differs")[0] == "killed"
    assert c(2, "run_arm: INVALID scope not imported")[0] == "incompetent"
    assert c(124, "run_arm: deadline exceeded")[0] == "killed" and c("timeout", "")[0] == "killed"
    assert c(3, "replay fresh=False")[0] == "apparatus-fault"
