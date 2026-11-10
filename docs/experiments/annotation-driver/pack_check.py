r"""Admission checks for a data pack written by a model. No model, no network.

    uv run --no-sync python pack_check.py --pack FILE --language LANG [--scope NAME --scopes-dir DIR] --out FILE

``check_pack`` evaluates, in this order: json, size, shape, kinds_valid,
templates and, when a scope and its controls are given, controls and
seed_mutants. Every check is evaluated; one that cannot run because of an
earlier failure is reported as failed with "not run" in its detail. The last
two run in subprocesses with AD_TABLES_DIR set to a temporary directory that
holds only the checked pack. Details name keys, node kinds, counts and control
labels, never scope source text.

Exit code 0 when the pack is admitted, 1 when it is not, 2 on a usage error.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
from collections import Counter
from pathlib import Path

from ast_grep_py import SgRoot

import engine as e
import validator as v
from annotate import _reject_constant

HERE = Path(__file__).parent
MAX_LINES, MAX_CHARS = 40, 2500
REQUIRED = {"language": "string", "comment_kinds": "strings", "string_kinds": "strings", "unwrap_kinds": "strings",
            "test_attribute_markers": "strings", "kinds": "object", "noop_statement": "string",
            "statement_wrappers": "strings", "operators": "object"}
KIND_ORDER = ("branch_condition", "failure_state", "error_order", "state_write", "pure_shell_boundary")
RULE_KEYS = {"field_of": "pairs", "nodes": "strings", "nodes_with_prefix": "prefixes", "requires_next_sibling": "boolean"}
OPERATOR_ENTRIES = ("branch_condition", "failure_state", "state_write", "error_order")
DESCRIBE = {"string": "a string", "object": "an object", "strings": "a list of strings", "boolean": "true or false",
            "pairs": "a list of [kind, field] string pairs", "prefixes": "an object of lists of strings"}
PLACEHOLDERS = ("text", "noop", "child0")
PLACEHOLDER = re.compile(r"\{(\w+)\}")  # what engine._fill substitutes
PASSING_SEED_OUTCOMES = ("changed", "no_operator")
SUBPROCESS_TIMEOUT_S = 900
EXCEPTION_LINE = re.compile(r"([A-Za-z_][\w.]*(?:Error|Exception|Exit|Interrupt))\b")
SEED_WORKER = "import sys, pack_check; sys.exit(pack_check._seed_worker())"


def line_count(text: str) -> int:
    """Lines as run_engine_checks counts them: LF, CRLF or CR breaks; a final break adds none."""
    return len(text.encode("utf-8").splitlines())


def parse_pack(text: str) -> tuple[dict | None, str | None]:
    try:
        data = json.loads(text, parse_constant=_reject_constant)
    except (ValueError, RecursionError) as exc:
        return None, f"not valid JSON: {exc}"
    if not isinstance(data, dict):
        names = {list: "an array", str: "a string", bool: "a boolean", type(None): "null"}
        return None, f"the JSON value is {names.get(type(data), 'a number')}, not an object"
    return data, None


def _type_ok(value, expected: str) -> bool:
    if expected == "string":
        return isinstance(value, str)
    if expected == "object":
        return isinstance(value, dict)
    if expected == "strings":
        return isinstance(value, list) and all(isinstance(x, str) for x in value)
    if expected == "boolean":
        return isinstance(value, bool)
    if expected == "pairs":
        return isinstance(value, list) and all(isinstance(p, list) and len(p) == 2 and _type_ok(p, "strings")
                                               for p in value)
    if expected == "prefixes":
        return isinstance(value, dict) and all(_type_ok(x, "strings") for x in value.values())
    raise ValueError(expected)


def shape_problems(pack: dict, language: str) -> tuple[list[str], list[str]]:
    """(problems that keep the validator and engine from reading the pack, other problems)."""
    structural, other = [], []
    for key, expected in REQUIRED.items():
        if key not in pack:
            structural.append(f"missing key {key}")
        elif not _type_ok(pack[key], expected):
            structural.append(f"{key} must be {DESCRIBE[expected]}")
    if isinstance(pack.get("language"), str) and pack["language"] != language:
        other.append(f"language is {json.dumps(pack['language'])}, expected {json.dumps(language)}")
    kinds = pack.get("kinds")
    if isinstance(kinds, dict):
        for kind in KIND_ORDER:
            rule = kinds.get(kind)
            if kind not in kinds:
                structural.append(f"missing kinds.{kind}")
            elif not isinstance(rule, dict):
                structural.append(f"kinds.{kind} must be an object")
            else:
                structural += [f"kinds.{kind}.{key} must be {DESCRIBE[expected]}"
                               for key, expected in RULE_KEYS.items() if key in rule and not _type_ok(rule[key], expected)]
    ops = pack.get("operators")
    if isinstance(ops, dict):
        structural += [f"operators.{name} must be an object"
                       for name in OPERATOR_ENTRIES if name in ops and not isinstance(ops[name], dict)]
        branch, failure = ops.get("branch_condition"), ops.get("failure_state")
        if isinstance(branch, dict) and "exclude_kinds" in branch and not _type_ok(branch["exclude_kinds"], "strings"):
            structural.append("operators.branch_condition.exclude_kinds must be a list of strings")
        if isinstance(failure, dict) and "by_kind" in failure and not isinstance(failure["by_kind"], dict):
            structural.append("operators.failure_state.by_kind must be an object")
    unknown = sorted(set(pack) - set(REQUIRED))
    if unknown:
        other.append(f"unknown top-level keys: {', '.join(unknown)}")
    return structural, other


def _strings(value) -> list[str]:
    return [x for x in value if isinstance(x, str)] if isinstance(value, list) else []


def _entry(mapping, key) -> dict:
    value = mapping.get(key) if isinstance(mapping, dict) else None
    return value if isinstance(value, dict) else {}


def kind_names(pack: dict) -> list[str]:
    """Every node kind name the pack uses, read defensively (shape reports wrong types)."""
    names = [n for key in ("comment_kinds", "string_kinds", "unwrap_kinds", "statement_wrappers")
             for n in _strings(pack.get(key))]
    kinds = pack.get("kinds")
    for rule in (kinds.values() if isinstance(kinds, dict) else ()):
        if not isinstance(rule, dict):
            continue
        names += _strings(rule.get("nodes"))
        pairs = rule.get("field_of")
        names += [p[0] for p in (pairs if isinstance(pairs, list) else ())
                  if isinstance(p, list) and p and isinstance(p[0], str)]
        names += list(_entry(rule, "nodes_with_prefix"))
    ops = pack.get("operators")
    names += _strings(_entry(ops, "branch_condition").get("exclude_kinds"))
    names += list(_entry(_entry(ops, "failure_state"), "by_kind"))
    return sorted(set(names))


def _kind_is_valid(root, name: str) -> bool:
    try:
        root.find(kind=name)
        return True
    except RuntimeError:  # ast-grep: "Invalid Kind"
        return False


def kinds_valid(pack: dict, language: str) -> tuple[bool, str]:
    names = kind_names(pack)
    try:
        root = SgRoot("", language).root()
    except (KeyboardInterrupt, SystemExit):
        raise
    except BaseException:  # ast-grep panics (a BaseException) on a language it does not know
        return False, f"ast-grep has no grammar named {json.dumps(language)}"
    invalid = [n for n in names if not _kind_is_valid(root, n)]
    if invalid:
        return False, f"node kinds not in the {language} grammar: {', '.join(invalid)}"
    return True, f"{len(names)} node kinds, all valid for {language}"


def templates(pack: dict) -> tuple[bool, str]:
    ops = pack.get("operators")
    branch, failure, write = _entry(ops, "branch_condition"), _entry(ops, "failure_state"), _entry(ops, "state_write")
    found = []
    if "template" in branch:
        found.append(("operators.branch_condition.template", branch["template"]))
    found += [(f"operators.failure_state.by_kind.{kind}", t) for kind, t in _entry(failure, "by_kind").items()]
    if "statement" in write:
        found.append(("operators.state_write.statement", write["statement"]))
    problems = []
    if "template" not in branch:
        problems.append("operators.branch_condition.template is missing")
    elif isinstance(branch["template"], str) and "{text}" not in branch["template"]:
        problems.append("operators.branch_condition.template does not contain {text}")
    for path, template in found:
        if not isinstance(template, str):
            problems.append(f"{path} is not a string")
            continue
        unknown = sorted(set(PLACEHOLDER.findall(template)) - set(PLACEHOLDERS))
        if unknown:
            problems.append(f"{path} uses unknown placeholders: {', '.join('{' + u + '}' for u in unknown)}")
    if problems:
        return False, "; ".join(problems)
    return True, f"{len(found)} templates, placeholders only {{text}}, {{noop}}, {{child0}}"


def _subprocess(argv: list[str], tables: Path, stdin: str | None = None) -> subprocess.CompletedProcess:
    return subprocess.run(argv, cwd=HERE, env={**os.environ, "AD_TABLES_DIR": str(tables)}, input=stdin,
                          capture_output=True, text=True, encoding="utf-8", errors="replace",
                          timeout=SUBPROCESS_TIMEOUT_S)


def _stopped(what: str, proc: subprocess.CompletedProcess) -> str:
    """Why a subprocess gave no result, without its output (which could quote source text)."""
    lines = [line for line in proc.stderr.splitlines() if line.strip()]
    last = lines[-1] if lines else ""
    match = EXCEPTION_LINE.match(last)
    hint = f", {match.group(1)}" if match else ""
    if last.startswith("no comment node at "):
        hint = ", no node of comment_kinds at the line of the in_comment control"
    return f"{what} stopped without a result (exit {proc.returncode}{hint})"


def controls_check(scope: v.Scope, tables: Path, work: Path) -> tuple[bool, str]:
    """run_controls.py --scope NAME with the checked pack; the scopes directory is the scope root's parent."""
    receipt = work / "controls-receipt.json"
    argv = [sys.executable, str(HERE / "run_controls.py"), "--scopes-dir", str(scope.root.parent),
            "--out", str(receipt), "--scope", scope.name]
    try:
        proc = _subprocess(argv, tables)
    except subprocess.TimeoutExpired:
        return False, f"run_controls.py did not finish within {SUBPROCESS_TIMEOUT_S} s"
    if not receipt.is_file():
        return False, _stopped("run_controls.py", proc)
    results = json.loads(receipt.read_text(encoding="utf-8"))["results"]
    failing = [r for r in results if not r["pass"]]
    detail = f"{len(results) - len(failing)} of {len(results)} controls behaved as expected"
    if failing:
        detail += "; failing: " + "; ".join(
            f"{r['control']} expected {json.dumps(r['expected'])} got {json.dumps(r['got'])}" for r in failing)
    return proc.returncode == 0 and bool(results) and not failing, detail


def seed_mutant_outcomes(scope: v.Scope, seeds: list[dict]) -> dict[str, int]:
    """Outcome counts for control seeds under the pack ``validator.load_table`` reads.

    Run it with AD_TABLES_DIR set. Outcomes: ``changed``, ``no_op`` or
    ``parse_error`` (``validator.classify_mutant`` of the mutant);
    ``no_operator`` when the pack has no operator for the seed's site;
    ``no_site`` when the seed is not an eligible site under the pack;
    ``error:<type>`` when the engine raised.
    """
    outcomes: Counter = Counter()
    for seed in seeds:
        try:
            if seed.get("kind") not in e.OPERATOR_KINDS:
                outcome = "no_operator"
            elif (site := e.site_from_annotation(scope, seed)) is None:
                outcome = "no_site"
            elif (mutant := e.mutate(scope, site)) is None:
                outcome = "no_operator"
            else:
                outcome = v.classify_mutant(scope.language, e.read_text(scope, site.file), mutant.text)
        except Exception as exc:
            outcome = f"error:{type(exc).__name__}"
        outcomes[outcome] += 1
    return dict(sorted(outcomes.items()))


def _seed_worker() -> int:
    """Subprocess entry: a JSON request on stdin, the outcome counts as one JSON line on stdout."""
    request = json.loads(sys.stdin.read())
    scope = v.Scope(request["name"], request["language"], Path(request["root"]),
                    tuple(request["files"]), tuple(request["test_paths"]))
    print(json.dumps(seed_mutant_outcomes(scope, request["seeds"])))
    return 0


def seed_mutants_check(scope: v.Scope, seeds: list[dict], tables: Path) -> tuple[bool, str]:
    request = json.dumps({"name": scope.name, "language": scope.language, "root": str(scope.root),
                          "files": list(scope.files), "test_paths": list(scope.test_paths), "seeds": seeds})
    try:
        proc = _subprocess([sys.executable, "-c", SEED_WORKER], tables, request)
    except subprocess.TimeoutExpired:
        return False, f"the seed check did not finish within {SUBPROCESS_TIMEOUT_S} s"
    try:
        counts = json.loads(proc.stdout.strip().splitlines()[-1]) if proc.returncode == 0 else None
    except (ValueError, IndexError):
        counts = None
    if not isinstance(counts, dict):
        return False, _stopped("the seed check", proc)
    detail = f"{sum(counts.values())} seeds: " + ", ".join(f"{outcome} {n}" for outcome, n in counts.items())
    return all(outcome in PASSING_SEED_OUTCOMES for outcome in counts), detail


def check_pack(text: str, language: str, scope: v.Scope | None = None, controls: dict | None = None) -> dict:
    """``{"admitted": bool, "checks": [{"name", "pass", "detail"}]}`` for one pack text.

    ``controls`` is the scope's entry in controls.json; the controls check runs
    run_controls.py, which reads controls.json itself.
    """
    if (scope is None) != (controls is None):
        raise ValueError("give both a scope and its controls, or neither")
    if scope is not None and scope.language != language:
        raise ValueError(f"scope {scope.name} is {scope.language}, not {language}")
    checks: list[dict] = []

    def add(name: str, ok: bool, detail: str) -> None:
        checks.append({"name": name, "pass": bool(ok), "detail": detail})

    pack, error = parse_pack(text)
    add("json", pack is not None, error or "one JSON object")
    lines, chars = line_count(text), len(text)
    add("size", lines <= MAX_LINES and chars <= MAX_CHARS,
        f"{lines} lines (at most {MAX_LINES}), {chars} characters (at most {MAX_CHARS})")
    later = ["shape", "kinds_valid", "templates", *(["controls", "seed_mutants"] if scope is not None else [])]
    if pack is None:
        for name in later:
            add(name, False, "not run: the text is not one JSON object")
        return {"admitted": False, "checks": checks}
    structural, other = shape_problems(pack, language)
    add("shape", not structural and not other, "; ".join(structural + other) or "required keys present with the right types")
    add("kinds_valid", *kinds_valid(pack, language))
    add("templates", *templates(pack))
    if scope is not None:
        if structural:
            for name in ("controls", "seed_mutants"):
                add(name, False, "not run: required keys are missing or of the wrong type")
        else:
            with tempfile.TemporaryDirectory(prefix="pack-check-", ignore_cleanup_errors=True) as tmp:
                tables = Path(tmp) / "tables"
                tables.mkdir()
                (tables / f"{language}.json").write_bytes(text.encode("utf-8"))
                add("controls", *controls_check(scope, tables, Path(tmp)))
                add("seed_mutants", *seed_mutants_check(scope, controls["seeds"], tables))
    return {"admitted": all(c["pass"] for c in checks), "checks": checks}


def feedback_for_model(result: dict) -> str:
    """The failed checks and their details as a plain-text list."""
    return "\n".join(f"- {c['name']}: {c['detail']}" for c in result["checks"] if not c["pass"])


def load_scope_controls(name: str, scopes_dir: Path) -> tuple[v.Scope, dict]:
    """The scope from scopes.json and its entry in controls.json; KeyError when either is missing."""
    scopes = v.load_scopes(scopes_dir)
    controls = json.loads((HERE / "controls.json").read_text(encoding="utf-8"))["scopes"]
    if name not in scopes or name not in controls:
        raise KeyError(name)
    return scopes[name], controls[name]


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description="Admission checks for one data pack.")
    parser.add_argument("--pack", type=Path, required=True)
    parser.add_argument("--language", required=True)
    parser.add_argument("--scope")
    parser.add_argument("--scopes-dir", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args(argv)
    if (args.scope is None) != (args.scopes_dir is None):
        parser.error("--scope and --scopes-dir go together")
    try:
        text = args.pack.read_bytes().decode("utf-8")
    except (OSError, UnicodeDecodeError) as exc:
        parser.error(f"cannot read the pack: {type(exc).__name__}: {exc}")
    scope = controls = None
    if args.scope is not None:
        try:
            scope, controls = load_scope_controls(args.scope, args.scopes_dir)
        except KeyError:
            parser.error(f"unknown scope or no controls: {args.scope}")
        if scope.language != args.language:
            parser.error(f"scope {scope.name} is {scope.language}, not {args.language}")
    result = check_pack(text, args.language, scope, controls)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(result, indent=1, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    feedback = feedback_for_model(result)
    print(f"{args.pack.name}: admitted {result['admitted']}" + (f"\n{feedback}" if feedback else ""))
    return 0 if result["admitted"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
