"""Apply one target's cosmic-ray mutation (the exact line from E1b's cr-arm-b.jsonl diff) to a copy, or restore it.
  mutate.py apply COPY LINE | restore COPY      prints scope.record(COPY)["changed"] afterwards"""
import json, shutil, sys
from pathlib import Path

E1B = Path(__file__).resolve().parents[1] / "e1b-facdrone-evidence-publication"
sys.path.insert(0, str(E1B))
import scope  # noqa: E402

REL = "src/facdrone/shells/cppi/state_repository.py"
OPS = {150: "Eq_LtE|4", 166: "ReplaceAndWithOr|0", 169: "Eq_LtE|8", 186: "Eq_IsNot|11", 200: "Eq_GtE|16",
       418: "Eq_GtE|33", 584: "Eq_Is|37", 642: "NumberReplacer|20", 240: "Eq_Is|20", 241: "Eq_LtE|23"}


def diff_lines(line: int):
    op, occ = OPS[line].split("|")
    for raw in open(E1B / "results" / "cr-arm-b.jsonl", encoding="utf-8"):
        job, res = json.loads(raw)
        m = job["mutations"][0]
        if "state_repository" in m["module_path"] and m["operator_name"].endswith(op) and m["occurrence"] == int(occ):
            assert m["start_pos"][0] == line, m
            d = res["diff"].splitlines()
            return ([x[1:] for x in d if x.startswith("-") and not x.startswith("---")],
                    [x[1:] for x in d if x.startswith("+") and not x.startswith("+++")], m)
    raise KeyError(line)


if __name__ == "__main__":
    copy = Path(sys.argv[2])
    shutil.copy2(scope.FROZEN / REL, copy / REL)  # both modes start from the frozen file
    if sys.argv[1] == "apply":
        line = int(sys.argv[3])
        (old,), (new,), m = diff_lines(line)
        text = (copy / REL).read_bytes().decode("utf-8").split("\n")  # raw bytes: CRLF kept
        assert text[line - 1].rstrip("\r") == old, (text[line - 1], old)
        text[line - 1] = new + ("\r" if text[line - 1].endswith("\r") else "")
        (copy / REL).write_bytes("\n".join(text).encode("utf-8"))
        print(json.dumps({"mutation": m, "old": old, "new": new}))
    print(json.dumps({"changed": scope.record(copy)["changed"], "lines": scope.changed_lines(
        (scope.FROZEN / REL).read_text(encoding="utf-8"), (copy / REL).read_text(encoding="utf-8"))}))
