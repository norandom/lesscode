"""Differential trial: acceptance and values, not exact diagnostic messages."""
import ast
from copy import deepcopy
import hashlib
import importlib.metadata
import inspect
import json
from pathlib import Path
import random
import candidate
from lesscode_baseline import contracts as reference

ROOT = Path(__file__).resolve().parents[3]
samples = {
    "Limits": {"timeout_seconds": 30, "max_log_bytes": 1000},
    "FileComplexity": {"path": "src/a.py", "nloc": 0},
    "SourceSpan": {"path": "src/a.py", "start_line": 1, "end_line": 3},
    "FunctionComplexity": {"path": "src/a.py", "start_line": 1, "end_line": 3,
                           "name": "f", "long_name": "f(x)", "ccn": 1, "nloc": 0, "token_count": 0},
    "ClonePair": {"first": {"path": "a.py", "start_line": 1, "end_line": 2},
                  "second": {"path": "b.py", "start_line": 1, "end_line": 2}},
}
values = [None, True, False, -1, 0, 1, 1.0, 1.5, float("inf"), float("nan"),
          "", " ", "x", "../x", "/x", "x\\y", "x.", "x\x00", [], {}, 10**1000]
random.seed(0)
mismatches, cases = [], 0


def outcome(reader, value):
    try:
        return True, reader(value)
    except (ValueError, TypeError, OverflowError):
        return False, None


for name, valid in samples.items():
    inputs = [valid, {}, {**valid, "unknown": 1}, None, [], True]
    for key in valid:
        inputs.append({k: v for k, v in valid.items() if k != key})
        inputs.extend({**valid, key: value} for value in values)
    for _ in range(1000):
        value = deepcopy(valid)
        key = random.choice(list(value))
        if isinstance(value[key], dict):
            value[key][random.choice(list(value[key]))] = random.choice(values)
        else:
            value[key] = random.choice(values)
        inputs.append(value)
    for value in inputs:
        cases += 1
        old = outcome(lambda v: getattr(reference, name).from_dict(v).to_dict(), value)
        new = outcome(lambda v: getattr(candidate, name).model_validate(v).model_dump(), value)
        if old != new:
            mismatches.append({"record": name, "input": repr(value), "reference": repr(old), "candidate": repr(new)})


def counts(text):
    return {"physical_lines": len(text.splitlines()), "bytes": len(text.encode()),
            "statements": sum(isinstance(n, ast.stmt) for n in ast.walk(ast.parse(text)))}


original = "\n".join(inspect.getsource(getattr(reference, name)) for name in samples)
source = Path(candidate.__file__).read_text(encoding="utf-8")
result = {"seed": 0, "cases": cases, "mismatch_count": len(mismatches),
          "mismatches": mismatches[:10], "pydantic": importlib.metadata.version("pydantic"),
          "reference_leaf_classes": counts(original), "candidate_including_support": counts(source),
          "trial_test_code": counts(Path(__file__).read_text()),
          "reference_module_sha256": hashlib.sha256(Path(reference.__file__).read_bytes()).hexdigest(),
          "limits": ["No runtime migration", "Exact diagnostics not equivalent or tested",
                     "Reference shared helpers remain needed by other records",
                     "Finite deterministic corpus is not proof of equivalence",
                     "Statement count is not LLOC or maintenance effort"]}
result["checkout_footprint"] = {
    category: {"files": len(paths), **counts("\n".join(p.read_text(encoding="utf-8-sig") for p in paths))}
    for category, paths in {
        "production": sorted((ROOT / "src").rglob("*.py")),
        "automated_tests_excluding_fixtures": sorted((ROOT / "tests").glob("test_*.py")),
    }.items()
}
Path(__file__).with_name("result.json").write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps(result, indent=2))
