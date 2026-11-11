"""Prototype of a step verdict: compare two `deconvolute run` receipts of one file, before and after a step.

Research tooling for the prototype gate of `.kiro/specs/gradual-refactoring-steps`. Standard library only.

    python step_verdict.py BEFORE_RUN.json BEFORE_SOURCE AFTER_RUN.json AFTER_SOURCE [--function-pattern REGEX]

Mutants cannot be matched one to one across a change, because sites move. They are grouped by the function
they lie in, and functions are matched by name. A step is refused when either run is void, or when in any
function the number of unnoticed faults (survived) rises or the number of noticed faults (killed) falls
without the function having changed its number of faults.
"""

import json
import re
import sys
from pathlib import Path

DEFAULT_PATTERN = r"^\s*(?:pub(?:\([a-z]+\))?\s+)?(?:async\s+)?(?:fn|def|function)\s+([A-Za-z_][A-Za-z0-9_]*)"


def functions(source: Path, pattern: str) -> list[tuple[int, str]]:
    """(first line, name) of each function header, in order."""
    found = []
    for number, line in enumerate(source.read_text(encoding="utf-8").splitlines(), start=1):
        match = re.match(pattern, line)
        if match:
            found.append((number, match.group(1)))
    return found


def by_function(run: dict, heads: list[tuple[int, str]]) -> dict[str, dict[str, int]]:
    table: dict[str, dict[str, int]] = {}
    for mutant in run["mutants"]:
        line = mutant["site"]["start_line"]
        name = next((n for start, n in reversed(heads) if start <= line), "(before the first function)")
        row = table.setdefault(name, {})
        row[mutant["class"]] = row.get(mutant["class"], 0) + 1
    return table


def main() -> int:
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    pattern = next((a.split("=", 1)[1] for a in sys.argv[1:] if a.startswith("--function-pattern=")), DEFAULT_PATTERN)
    before_run, after_run = (json.loads(Path(p).read_text(encoding="utf-8-sig")) for p in (args[0], args[2]))
    before = by_function(before_run, functions(Path(args[1]), pattern))
    after = by_function(after_run, functions(Path(args[3]), pattern))
    reasons, rows = [], []
    for label, run in (("before", before_run), ("after", after_run)):
        if run.get("void"):
            reasons.append(f"the run {label} the step is void: {run['void']}")
    for name in sorted(set(before) | set(after)):
        b, a = before.get(name, {}), after.get(name, {})
        if b == a:
            continue
        rows.append({"function": name, "before": b, "after": a})
        more_unnoticed = a.get("survived", 0) > b.get("survived", 0)
        fewer_noticed = a.get("killed", 0) < b.get("killed", 0) and sum(a.values()) >= sum(b.values())
        if more_unnoticed or fewer_noticed:
            reasons.append(f"{name}: killed {b.get('killed', 0)} -> {a.get('killed', 0)}, "
                           f"survived {b.get('survived', 0)} -> {a.get('survived', 0)}")
    total = lambda table, key: sum(row.get(key, 0) for row in table.values())  # noqa: E731
    verdict = {"receipt": "step-verdict-prototype-v1", "accepted": not reasons, "reasons": reasons,
               "killed": [total(before, "killed"), total(after, "killed")],
               "survived": [total(before, "survived"), total(after, "survived")],
               "functions_that_differ": rows}
    print(json.dumps(verdict, indent=1))
    return 0 if not reasons else 1


if __name__ == "__main__":
    raise SystemExit(main())
