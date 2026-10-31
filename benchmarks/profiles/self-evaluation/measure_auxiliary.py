"""Measure auxiliary Radon evidence on the exact product bytes in a run report."""

import argparse
import ast
import hashlib
import importlib.metadata
import json
from pathlib import Path

from radon.metrics import mi_visit
from radon.raw import analyze


parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--report", type=Path, required=True)
parser.add_argument("--baseline", type=Path, required=True)
parser.add_argument("--candidate", type=Path, required=True)
parser.add_argument("--out", type=Path, required=True)
args = parser.parse_args()
if args.out.exists():
    parser.error("use a new auxiliary output file")
report = json.loads(args.report.read_text(encoding="utf-8"))
evidence = {
    "schema_version": 1, "producer": "radon",
    "version": importlib.metadata.version("radon"),
    "definition": "radon.raw.analyze LLOC; mi_visit(multi=False), descriptive only",
    "plan_hash": report["plan_hash"],
    "dependencies": {name: importlib.metadata.version(name)
                     for name in ("radon", "mando", "colorama", "six")},
    "sides": {},
}
for side, source in (("baseline", args.baseline), ("candidate", args.candidate)):
    files = {}
    for record in report[side]["snapshot"]["files"]:
        if record["category"] != "product":
            continue
        content = (source / record["path"]).read_bytes()
        if hashlib.sha256(content).hexdigest() != record["content_hash"]:
            raise ValueError(f"source differs from recorded evidence: {side} {record['path']}")
        code = content.decode("utf-8")
        files[record["path"]] = {"sha256": record["content_hash"],
                                 "raw": analyze(code)._asdict(),
                                 "maintainability_index": mi_visit(code, multi=False)}
    code = (source / "src/lesscode_baseline/decision.py").read_text(encoding="utf-8")
    function = next(node for node in ast.parse(code).body
                    if isinstance(node, ast.FunctionDef) and node.name == "evaluate_threshold")
    evidence["sides"][side] = {
        "source_hash": report[side]["snapshot"]["content_hash"], "files": files,
        "totals": {key: sum(item["raw"][key] for item in files.values())
                   for key in ("loc", "lloc", "sloc", "comments", "multi", "blank")},
        "target_function": analyze(ast.get_source_segment(code, function))._asdict(),
        "target_module_mi": files["src/lesscode_baseline/decision.py"]["maintainability_index"],
    }
args.out.write_text(json.dumps(evidence, indent=2) + "\n", encoding="utf-8")
print("Wrote auxiliary evidence with verified source hashes to", args.out)
