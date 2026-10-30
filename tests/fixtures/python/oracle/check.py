"""Independent fixture check. Usage: check.py SOURCE RESULT FORMAT PLAN_HASH.

Run against disposable source copies. This is a trusted check, not a sandbox.
Normalized output is a JSON array of the frozen CheckEvidence records.
"""

import hashlib
import json
from pathlib import Path
import runpy
import sys
import xml.etree.ElementTree as ET


def check(source, plan_hash):
    golden = json.loads(Path(__file__).with_name("golden.json").read_text())
    clamp = runpy.run_path(str(source))["clamp"]
    records = []
    source_hash = hashlib.sha256(source.read_bytes()).hexdigest()
    for case in golden:
        records.append({"check_id": case["id"], "status": "passed" if
                        clamp(*case["args"]) == case["expected"] else "failed",
                        "kind": "golden", "required": True, "source_hash": source_hash,
                        "plan_hash": plan_hash, "diagnostic_artifact": None})
    bounded = all(-2 <= clamp(value, -2, 3) <= 3 for value in range(-5, 7))
    records.append({"check_id": "clamp.bounds", "status": "passed" if bounded else "failed",
                    "kind": "invariant", "required": True, "source_hash": source_hash,
                    "plan_hash": plan_hash, "diagnostic_artifact": None})
    return records


def main():
    source, output, fmt, plan_hash = sys.argv[1:]
    records = check(Path(source), plan_hash)
    if fmt == "normalized":
        Path(output).write_text(json.dumps(records, indent=2) + "\n", encoding="utf-8", newline="\n")
    elif fmt == "junit":
        suite = ET.Element("testsuite", name="independent", tests=str(len(records)),
                           failures=str(sum(r["status"] == "failed" for r in records)))
        for record in records:
            case = ET.SubElement(suite, "testcase", name=record["check_id"])
            if record["status"] == "failed":
                ET.SubElement(case, "failure", message="Independent expected behavior mismatch")
        ET.ElementTree(suite).write(output, encoding="utf-8", xml_declaration=True)
    else:
        raise ValueError("format must be normalized or junit")
    return int(any(r["status"] != "passed" for r in records))


if __name__ == "__main__":
    raise SystemExit(main())
