"""P5: behaviours observed by generated sequences but by no seed scenario.

Abstraction (defined for this analysis, not preregistered in detail): each
observation maps to a signature of
  op kind, op fault flags (bad_audit, trigger installed before the op),
  outcome class (ok / err message / panic message),
  active_sessions row-count delta and whether any session row changed,
  session_breadcrumbs row-count delta and the tool names of new rows,
  event types of new audit lines and their sha/ts validity,
  resume specifics (found_prior_session, last_harness == "none",
  task_id origin: literal / generated ordinal / inherited),
  checkpoint stored_chars class relative to the cap (0, <cap, 5999, 6000, >cap).
Signatures present in generated scenarios and absent from all seed scenarios
are reported as novel. A second, coarser view uses (op kind, outcome class).
"""
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
g = json.load(open(os.path.join(HERE, "golden-trace.json"), encoding="utf-8"))


def stored_class(n):
    if n == 0:
        return "0"
    if n in (5999, 6000):
        return str(n)
    return "<cap" if n < 5999 else ">cap"


def task_origin(t, op_task):
    if op_task not in (None, "<last-resumed>") and t == op_task:
        return "literal"
    if re.search(r"-T\d+$", t or ""):
        return "generated-ordinal"
    return "inherited-or-last"


def signatures(sc):
    out, prev_s, prev_c, trig = [], [], [], False
    for o in sc["observations"]:
        op = o["op"]
        kind = op["op"]
        ret = o["ret"]
        outcome = "ok" if "ok" in ret else ("err:" + ret["err"] if "err" in ret else "panic:" + ret["panic"].split(";")[0])
        s, c = o["active_sessions"], o["session_breadcrumbs"]
        new_tools = tuple(sorted(r["tool_name"] for r in c[len(prev_c):]))
        audit = tuple((a.get("event_type"), a.get("record_sha256"), a.get("ts")) for a in o["audit_new"])
        extra = ()
        if kind == "resume" and "ok" in ret:
            v = ret["ok"]
            extra = (v["found_prior_session"], v["last_harness"] == "none", task_origin(v["task_id"], op.get("task")),
                     len(v["breadcrumbs"]))
        if kind == "checkpoint" and "ok" in ret:
            extra = (stored_class(ret["ok"]["stored_chars"]), ret["ok"]["client_usage"] is not None)
        sig = (kind, op.get("bad_audit", False), trig, outcome, len(s) - len(prev_s), s != prev_s,
               len(c) - len(prev_c), new_tools, audit, extra)
        out.append(sig)
        if kind.startswith("install"):
            trig = True
        if kind.startswith("drop"):
            trig = False
        prev_s, prev_c = s, c
    return out


seed, gen = {}, {}
for sc in g["scenarios"]:
    target = seed if sc["name"].startswith("seed:") else gen
    for i, sig in enumerate(signatures(sc)):
        target.setdefault(sig, f"{sc['name']} op {i}")
novel = {k: v for k, v in gen.items() if k not in seed}
coarse_seed = {(k[0], k[3]) for k in seed}
coarse_novel = sorted({(k[0], k[3]) for k in gen} - coarse_seed)
res = {
    "seed_signatures": len(seed),
    "generated_signatures": len(gen),
    "generated_only_signatures": len(novel),
    "coarse_generated_only_op_outcomes": [list(x) for x in coarse_novel],
    "examples": [{"signature": [str(x) for x in k], "first_seen": v} for k, v in list(novel.items())[:40]],
}
json.dump(res, open(os.path.join(HERE, "p5-novel-behaviours.json"), "w", encoding="utf-8"), indent=1)
print(json.dumps({k: res[k] for k in res if k != "examples"}, indent=1))
for e in res["examples"][:12]:
    print(e["first_seen"], "|", " ; ".join(e["signature"]))
