"""R5: every captured call of an original is reproduced by its lifted function (mut=0).
  check_r5.py CAPTURE.jsonl [...]   prints per-function counts and the first mismatches as JSON."""
import json, sys
from collections import Counter

import lifted
from facdrone_wire.instructions import InstructionPayload

J = lambda v: json.loads(json.dumps(v))  # tuples to lists, as the capture stored them


def lifted_out(c):
    fn, rows, ident = c["fn"], [tuple(r) for r in c.get("rows", [])], c.get("ident")
    if fn in ("opening_tickets", "resting_orders"):
        return getattr(lifted, fn)(rows, ident, [tuple(h) for h in c["history"]])
    if fn == "placed_under":
        return lifted.placed_under(tuple(c["rest"]), c["body"])
    if fn == "instruction_history":  # recorded conversion: validate the lifted bodies as the original does
        return [InstructionPayload.model_validate(b).model_dump(mode="json") for b in lifted.instruction_history(rows, ident)]
    if fn == "intention_ids":
        return lifted.intention_ids(c["candidates"])
    return lifted.decision_served(rows, ident, c["decision_id"])


def main(paths):
    seen, bad, mism, unjson, distinct = Counter(), Counter(), [], 0, set()
    for p in paths:
        for line in open(p, encoding="utf-8"):
            c = json.loads(line)
            unjson += "__unjson__" in line
            seen[c["fn"]] += 1
            distinct.add(json.dumps({k: v for k, v in c.items() if k != "out"}, sort_keys=True))
            try:
                got = J(lifted_out(c))
            except Exception as e:  # noqa: BLE001
                got = {"lifted_raised": f"{type(e).__name__}: {e}"}
            if got != c["out"]:
                bad[c["fn"]] += 1
                mism.append({"fn": c["fn"], "recorded": c["out"], "lifted": got})
    print(json.dumps({"calls": dict(seen), "mismatches": dict(bad), "unjson_lines": unjson,
                      "distinct_inputs": len(distinct), "first_mismatches": mism[:5]}, indent=1, default=str))
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
