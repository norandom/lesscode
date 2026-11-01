"""Fresh ATS replay: immutable historical harness functions, isolated full modules."""
import argparse
import ast
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
ATS = ROOT / "docs/experiments/ats-report"
_spec = importlib.util.spec_from_file_location("prepare", HERE.parent / "prepare.py")
prepare = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(prepare)
NATIVE = prepare.NATIVE


def read(path):
    return json.loads(path.read_text(encoding="utf-8"))


def write(path, value):
    prepare.write(path, value)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def harness():
    # Execute definitions only: never execute the historical collector's main block.
    tree = ast.parse((ATS / "trial.py").read_text(encoding="utf-8"))
    selected = [n for n in tree.body if isinstance(n, (ast.Import, ast.ImportFrom, ast.FunctionDef))]
    namespace = {}
    exec(compile(ast.Module(body=selected, type_ignores=[]), "frozen-ATS-harness", "exec"), namespace)
    return namespace


def extract_bytes(raw, arm):
    tree = ast.parse(raw.decode("utf-8"))
    agent = next(n for n in tree.body if isinstance(n, ast.ClassDef))
    fragment = ATS / {"baseline": "original.py.txt", "candidate": "candidate.py", "mutant": "mutant.py"}[arm]
    frozen = fragment.read_text(encoding="utf-8")
    expected = ast.parse(frozen)
    fixture = next(n for n in expected.body if isinstance(n, ast.ClassDef))
    names = {n.name for n in fixture.body if isinstance(n, ast.FunctionDef)}
    actual_class = copy.deepcopy(agent)
    actual_class.name = "ReportFixture"
    actual_class.body = [n for n in actual_class.body if getattr(n, "name", None) in names]
    actual = ast.Module(body=[n for n in expected.body if isinstance(n, ast.ImportFrom)] +
        [n for n in tree.body if isinstance(n, ast.Assign)] + [actual_class], type_ignores=[])
    if ast.dump(actual) != ast.dump(expected):
        raise ValueError("full-module reporting implementation differs from frozen fragment: " + arm)
    return frozen


def extract(path, arm):
    return extract_bytes(path.read_bytes(), arm)


def observe(fragment, original):
    h = harness()
    oracle = read(ATS / "oracle.json")
    mismatches, witnesses, observed = 0, [], 0
    outcomes = []
    for number, case in enumerate(h["corpus"]()):
        actual = h["outcome"](h["load"](fragment), *case)
        value = h["digest"](actual)
        outcomes.append(value)
        observed += 1
        if value != oracle[number]:
            mismatches += 1
            if len(witnesses) < 8:
                witnesses.append(dict(case=number, input=case, expected=h["outcome"](h["load"](original), *case), actual=actual))
    dispatch = []
    for source in (original, fragment):
        instance = h["load"](source)
        instance._generate_german_report = lambda *args: "override-de"
        instance._generate_english_report = lambda *args: "override-en"
        dispatch.append([instance.generate_report(dict(success=True, ats_analysis={}, parsing_summary={}), language)
                         for language in ("de", "en", None)])
    return dict(cases=observed, expected=len(oracle), mismatches=mismatches, witnesses=witnesses,
                outcome_digests=outcomes, dispatch=dispatch, dispatch_preserved=dispatch[0] == dispatch[1])


def check_result(raw, source, trial, definition, path, artifact_sha, check):
    passed = raw["mismatches"] == 0 and raw["cases"] == 8820 if check == "differential" else raw["dispatch_preserved"]
    summary = dict(kind="case_counts", expected=8820, observed=raw["cases"], mismatches=raw["mismatches"]) if check == "differential" else dict(kind="exit_status", code=0 if passed else 1)
    return dict(schema_version=1, source_identity=source, frozen_trial_digest=trial, definition_digest=definition,
                status="passed" if passed else "failed", artifacts={"raw": dict(schema_version=1, path=path, sha256=artifact_sha)},
                tested_domain="8820 unchanged fictional ATS corpus cases, exact return or exception type/message; existing subclass dispatch seam",
                limitations=["Finite reporting observations only; external services and non-reporting agent behavior are not executed.",
                             "Raw artifacts and hashes establish consistency, not authenticated execution."], summary=summary)


def cli(args, allowed=(0,)):
    result = subprocess.run([str(NATIVE), *map(str, args)], capture_output=True, text=True, timeout=120)
    if result.returncode not in allowed:
        raise RuntimeError(result.stderr + result.stdout)
    return result.returncode


def relationships(source, ids):
    # Every review retains its exact call expression and either an owned full-module
    # target or a named language/dependency API contract. Unknown shapes stop replay.
    raw = source.read_bytes()
    tree = ast.parse(raw.decode("utf-8"))
    starts = [0]
    for line in raw.splitlines(keepends=True):
        starts.append(starts[-1] + len(line))
    calls = {starts[n.lineno - 1] + n.col_offset: n for n in ast.walk(tree) if isinstance(n, ast.Call)}
    result = {}
    for identity in ids:
        offset = int(identity.rsplit(":", 1)[1])
        call = calls[offset]
        expression = ast.unparse(call.func)
        owned = (expression.split(".")[1] if expression.startswith("self.") else expression) in {n.name for n in ast.walk(tree) if isinstance(n, ast.FunctionDef)}
        contract = contract_for(expression) if not owned else None
        result[identity] = dict(schema_version=1, owned_targets=["ats_agent.py"] if owned else [], external_contract=contract,
            rationale=f"Reviewed call {expression} at UTF-8 byte {offset}: " + ("implementation retained in whole owned module; runtime dispatch remains observable." if owned else contract + "; unchanged surrounding call retained and counted; reporting-only replay does not validate service behavior."))
    return result


def contract_for(expression):
    named = {"OpenAI": "openai.OpenAI client construction", "extract_text_from_file": "cv_parser.extract_text_from_file", "analyze_cv_structure": "cv_parser.analyze_cv_structure", "detect_language": "cv_parser.detect_language", "get_german_ats_tips": "german_cv_knowledge.get_german_ats_tips", "get_german_section_requirements": "german_cv_knowledge.get_german_section_requirements"}
    if expression in named:
        return named[expression]
    if expression in ("str", "int", "float", "bool", "len", "enumerate", "range", "isinstance", "max", "min", "sum", "print", "ValueError", "Exception", "dict", "list"):
        return "Python builtins." + expression
    if expression.startswith(("os.", "json.")):
        return "Python standard library " + expression
    if expression.startswith("self.client."):
        return "OpenAI client API " + expression
    if expression.rsplit(".", 1)[-1] in ("get", "append", "join", "items", "strip", "lower", "upper", "split", "replace", "extend", "startswith", "endswith"):
        return "Python built-in container/string protocol " + expression
    raise ValueError("unreviewed call contract: " + expression)


def run(output, inputs, python):
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    before = prepare.original_hashes()
    for arm in ("baseline", "candidate", "mutant"):
        shutil.copytree(inputs / arm, output / arm)
        extract(output / arm / "ats_agent.py", arm)
    definition = dict(schema_version=1, runner_sha256=sha(Path(__file__)), harness_sha256=sha(ATS / "trial.py"),
                      oracle_sha256=sha(ATS / "oracle.json"), corpus="unchanged corpus/outcome/digest/load definitions from archived trial.py",
                      python=str(python.resolve()), source_fragments={n: sha(ATS / n) for n in ("original.py.txt", "candidate.py", "mutant.py")})
    write(output / "check-definition.json", definition)
    manifest = read(inputs / "manifest.json")
    manifest["trial_id"] = "ATS01-fresh-native-6.2"
    manifest["target"]["definitions"] = {key: value for key, value in manifest["target"]["definitions"].items() if key != "internal_call_layers"}
    manifest["target"]["permitted_structural_tradeoffs"].append("Separately declared 6.2 source-reduction target: unresolved internal call layers remain contextual; does not satisfy the stricter 6.1 preparation target.")
    for check in manifest["checks"].values():
        check["definition_digest"] = sha(output / "check-definition.json")
        check["artifact_expectations"] = []
    # Discover located findings through native analysis; no results are admitted here.
    write(output / "manifest-probe.json", manifest)
    cli(["prepare", "--manifest", output / "manifest-probe.json", "--baseline", output / "baseline", "--out", output / "probe-frozen.json"])
    template = read(inputs / "candidate-evidence.json")
    for arm in ("candidate", "mutant"):
        cli(["compare", "--frozen", output / "probe-frozen.json", "--baseline", output / "baseline", "--candidate", output / arm,
             "--evidence", inputs / "candidate-evidence.json", "--out", output / (arm + "-probe")], (1, 2, 3))
    probe = read(output / "candidate-probe/result.json")["comparison"]
    manifest["scope_review"]["relationships"] = relationships(output / "baseline/ats_agent.py", [key.removeprefix("baseline/") for key in probe["accounting"]["unresolved"] if key.startswith("baseline/")])
    write(output / "manifest.json", manifest)
    cli(["prepare", "--manifest", output / "manifest.json", "--baseline", output / "baseline", "--out", output / "frozen.json"])
    frozen = read(output / "frozen.json")
    identities = {}
    for arm in ("baseline", "candidate", "mutant"):
        cli(["prepare", "--manifest", output / "manifest.json", "--baseline", output / arm, "--out", output / (arm + "-identity.json")])
        identities[arm] = read(output / (arm + "-identity.json"))["baseline_identity"]
    # serde records preserve declaration order while keyed maps are sorted on read.
    trial_digest = hashlib.sha256(json.dumps(frozen, ensure_ascii=False, separators=(",", ":")).encode()).hexdigest()
    for arm in ("baseline", "candidate", "mutant"):
        rawdir = output / (arm + "-raw")
        rawdir.mkdir()
        subprocess.run(["uv.exe", "run", "--no-project", "--no-python-downloads", "--python", str(python.resolve()), "python", str(Path(__file__).resolve()), "observe", "--module", str(output / arm / "ats_agent.py"), "--arm", arm, "--out", str(rawdir / "raw.json")], check=True, timeout=180)
    for arm in ("candidate", "mutant"):
        evidence = copy.deepcopy(template)
        probe = read(output / (arm + "-probe/result.json"))["comparison"]
        evidence["candidate_ownership"]["review"]["relationships"] = relationships(output / arm / "ats_agent.py", [key.removeprefix("candidate/") for key in probe["accounting"]["unresolved"] if key.startswith("candidate/")])
        for check, check_definition in manifest["checks"].items():
            for side, actual in (("baseline", "baseline"), ("candidate", arm)):
                rawpath = output / (actual + "-raw/raw.json")
                evidence["checks"][check][side] = check_result(read(rawpath), identities[actual], trial_digest, check_definition["definition_digest"], actual + "-raw/raw.json", sha(rawpath), check)
        write(output / (arm + "-evidence.json"), evidence)
        cli(["compare", "--frozen", output / "frozen.json", "--baseline", output / "baseline", "--candidate", output / arm,
             "--evidence", output / (arm + "-evidence.json"), "--artifact-root", output, "--out", output / (arm + "-comparison")], (0, 1, 2, 3))
    after = prepare.original_hashes()
    if before != after:
        raise ValueError("historical originals changed")
    write(output / "receipt.json", dict(originals_before_sha256=before, originals_after_sha256=after, source_identities=identities,
          runner_sha256=sha(Path(__file__)), native_executable_sha256=sha(NATIVE), python=str(python.resolve())))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    replay = sub.add_parser("run")
    replay.add_argument("--out", type=Path, required=True)
    replay.add_argument("--inputs", type=Path, default=ROOT / ".runs/native-replay/task-6.1/ats")
    replay.add_argument("--python", type=Path, default=ROOT / ".venv-task11/Scripts/python.exe")
    observation = sub.add_parser("observe")
    observation.add_argument("--module", type=Path, required=True)
    observation.add_argument("--arm", choices=("baseline", "candidate", "mutant"), required=True)
    observation.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    if args.command == "observe":
        write(args.out, observe(extract(args.module, args.arm), (ATS / "original.py.txt").read_text(encoding="utf-8")))
    else:
        run(args.out, args.inputs, args.python)




