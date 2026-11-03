"""Extract session_bus.rs region/line/function coverage from llvm-cov summary JSONs."""
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
out = {}
for name in ("arm-a-summary", "arm-b-summary", "literal-lib-handoff-summary", "literal-lib-handoff-e1-summary"):
    data = json.load(open(os.path.join(HERE, "coverage", name + ".json"), encoding="utf-8"))
    for f in data["data"][0]["files"]:
        fn = f["filename"].replace("\\", "/")
        if fn.endswith("src/services/session_bus.rs") or fn.endswith("src/services/session_bus_tests.rs"):
            s = f["summary"]
            out.setdefault(name, {})[fn.split("/src/")[-1]] = {
                k: {"covered": s[k]["covered"], "count": s[k]["count"], "percent": round(s[k]["percent"], 2)}
                for k in ("regions", "lines", "functions") if k in s
            }
json.dump(out, open(os.path.join(HERE, "coverage", "session_bus-coverage.json"), "w"), indent=2)
print(json.dumps(out, indent=1))
