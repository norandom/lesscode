r"""Gate checklist step 2: validator controls, no model.

Derives wrong annotations from the hand-made seeds in controls.json and checks
that every one is rejected for the expected reason, that the seeds themselves
are accepted, and that no-op or unparseable mutants are classified invalid.
With a pack that has a statement terminator (pack format version 1), a seed
anchor that ends with it is also tried without it and must be accepted with
the terminator appended (reported as ``ok+anchor_completed``).

    uv run python run_controls.py --scopes-dir D:\BuildArtifacts\lesscode\annotation-driver\scopes --out RECEIPT.json
        [--scope NAME ...]

``--scope`` (repeatable) runs the controls of only the named scopes. The packs
are read where ``validator.load_table`` reads them (``AD_TABLES_DIR`` when set).

Exit code 0 only if every control behaves as expected.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
from pathlib import Path

from ast_grep_py import SgRoot

import validator as v

HERE = Path(__file__).parent
COMPLETED = "+anchor_completed"


def comment_node_at(scope: v.Scope, file: str, line: int) -> dict:
    table = v.load_table(scope.language)
    root = SgRoot(scope.source(file), scope.language).root()
    for node in v._walk(root):
        if node.kind() in table["comment_kinds"] and node.range().start.line == line - 1:
            return {"file": file, "start_line": line, "end_line": node.range().end.line + 1,
                    "kind": "branch_condition", "anchor": node.text(), "reason": "control: a comment"}
    raise SystemExit(f"no comment node at {file}:{line}; fix controls.json")


def punctuation_case(scope: v.Scope, seed: dict) -> list[tuple[str, list[dict], list[str]]]:
    """A lone bracket that is unique on the seed's line is a token, not a site."""
    line = scope.source(seed["file"]).splitlines()[seed["start_line"] - 1]
    bracket = next((c for c in "({[" if line.count(c) == 1), None)
    if bracket is None:
        return []
    token = {**seed, "anchor": bracket, "end_line": seed["start_line"]}
    return [("punctuation_anchor", [token], ["anchor_mismatch"])]


def terminator_cases(scope: v.Scope, spec: dict) -> list[tuple[str, list[dict], list[str]]]:
    """Pack format version 1: a seed whose anchor ends with the pack's statement terminator,
    without it, is accepted with the terminator appended (rule 2 extension)."""
    terminator = v.load_table(scope.language).get("statement_terminator") or ""
    if not terminator:
        return []
    return [(f"anchor_completed:{s['kind']}@{s['start_line']}", [{**s, "anchor": s["anchor"][: -len(terminator)]}],
             [f"ok{COMPLETED}"]) for s in spec["seeds"] if s["anchor"].endswith(terminator)]


def outcome(decision: dict) -> str:
    """The reason, marked when the anchor matched only with the statement terminator appended."""
    return decision["reason"] + (COMPLETED if decision.get("anchor_completed") else "")


def annotation_cases(scope: v.Scope, spec: dict) -> list[tuple[str, list[dict], list[str]]]:
    """(label, annotations, expected reason per annotation)."""
    seeds = spec["seeds"]
    first = seeds[0]
    cases = [(f"seed:{s['kind']}@{s['start_line']}", [s], ["ok"]) for s in seeds]

    def variant(**changes):
        changed = copy.deepcopy(first)
        changed.update(changes)
        return changed

    other_kind = next(k for k in v.KINDS if k not in {s["kind"] for s in seeds if s["anchor"] == first["anchor"]}
                      and k != "error_order")
    cases += [
        ("wrong_line", [variant(start_line=first["start_line"] + 3, end_line=first["end_line"] + 3)], ["anchor_mismatch"]),
        ("altered_anchor", [variant(anchor=first["anchor"] + " x")], ["anchor_mismatch"]),
        ("partial_anchor", [variant(anchor=first["anchor"][: max(3, len(first["anchor"]) // 2)])], ["anchor_mismatch"]),
        *punctuation_case(scope, first),
        # A scope file without any comment has no such control (comment_line is null).
        *([("in_comment", [comment_node_at(scope, first["file"], spec["comment_line"])], ["in_comment_or_string"])]
          if spec.get("comment_line") else []),
        ("test_file", [variant(file=spec["test_file"])], ["test_file"]),
        ("outside_scope", [variant(file=spec["outside_file"])], ["outside_scope"]),
        ("kind_mismatch", [variant(kind=other_kind if other_kind != first["kind"] else "pure_shell_boundary")], ["kind_mismatch"]),
        ("duplicate", [first, copy.deepcopy(first)], ["ok", "duplicate"]),
        ("malformed_no_kind", [{k: val for k, val in first.items() if k != "kind"}], ["malformed"]),
        ("malformed_lines", [variant(start_line=first["end_line"] + 1)], ["malformed"]),
        *terminator_cases(scope, spec),
    ]
    return cases


def mutant_cases(scope: v.Scope, spec: dict) -> list[tuple[str, str, str]]:
    """(label, mutated source, expected class)."""
    source = scope.source(spec["seeds"][0]["file"])
    old, new = spec["real_change"]
    if source.count(old) < 1:
        raise SystemExit(f"real_change text not found in {scope.name}; fix controls.json")
    return [
        ("identical", source, "no_op"),
        ("whitespace_only", source.rstrip("\n") + "\n\n\n", "no_op"),
        ("comment_only", source + "\n" + spec["comment_text"] + "\n", "no_op"),
        ("real_change", source.replace(old, new, 1), "changed"),
        ("unparseable", source.replace(old, old + " ((((", 1), "parse_error"),
    ]


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--scopes-dir", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--scope", action="append", metavar="NAME",
                        help="run only this scope's controls; repeatable")
    args = parser.parse_args()

    scopes = v.load_scopes(args.scopes_dir)
    controls = json.loads((HERE / "controls.json").read_text(encoding="utf-8"))["scopes"]
    if args.scope:
        unknown = sorted(set(args.scope) - set(controls))
        if unknown:
            parser.error(f"no controls for scope: {', '.join(unknown)}")
        controls = {name: spec for name, spec in controls.items() if name in args.scope}
    results, failures = [], 0
    for name, spec in controls.items():
        scope = scopes[name]
        for label, annotations, expected in annotation_cases(scope, spec):
            got = [outcome(d) for d in v.validate(scope, annotations)]
            ok = got == expected
            failures += not ok
            results.append({"scope": name, "control": label, "expected": expected, "got": got, "pass": ok})
        original = scope.source(spec["seeds"][0]["file"])
        for label, mutated, expected in mutant_cases(scope, spec):
            got = v.classify_mutant(scope.language, original, mutated)
            ok = got == expected
            failures += not ok
            results.append({"scope": name, "control": f"mutant:{label}", "expected": expected, "got": got, "pass": ok})

    frozen = ["validator.py", "run_controls.py", "controls.json", "scopes.json",
              *sorted(f"tables/{p.name}" for p in (HERE / "tables").glob("*.json"))]
    receipt = {
        "receipt": "annotation-driver-validator-controls-v0",
        "controls": len(results),
        "failed": failures,
        "passed": failures == 0,
        "inputs_sha256": {name: sha256(HERE / name) for name in frozen},
        "scope_files_sha256": {f"{s.name}:{f}": sha256(s.root / f) for s in scopes.values() for f in s.files},
        "results": results,
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(receipt, indent=1) + "\n", encoding="utf-8")
    for r in results:
        if not r["pass"]:
            print(f"FAIL {r['scope']} {r['control']}: expected {r['expected']}, got {r['got']}")
    print(f"{len(results) - failures} of {len(results)} controls behaved as expected")
    return 0 if failures == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
