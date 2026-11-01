"""Prepare only ATS01/NPU01 native replay inputs; never run historical collectors."""
import argparse
import ast
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NATIVE = ROOT / "native/target/debug/lesscode-core.exe"
ATS = ROOT / "docs/experiments/ats-report"
NPU = ROOT / "docs/experiments/npu-backends"
NPU_UPSTREAM = ROOT.parent / "npu_rag_from_folder"
PROVIDERS = "src/npu_rag/embedding/providers/"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read(path):
    return json.loads(path.read_text(encoding="utf-8"))


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def original_hashes():
    return {str(p.relative_to(ROOT)).replace("\\", "/"): digest(p.read_bytes())
            for folder in (ATS, NPU) for p in sorted(folder.rglob("*"))
            if p.is_file() and "__pycache__" not in p.parts}


def copy(source, target):
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, target)


def ats_mutant_module():
    """Use the same whole-module assembly as ATS01, including all added tables."""
    source = (ATS / "source.py.txt").read_text(encoding="utf-8")
    replacement = (ATS / "mutant.py").read_text(encoding="utf-8")
    tree = ast.parse(source)
    agent = next(n for n in tree.body if isinstance(n, ast.ClassDef))
    entry = next(n for n in agent.body if getattr(n, "name", None) == "generate_report")
    lines = source.splitlines(keepends=True)
    new_tree = ast.parse(replacement)
    fixture = next(n for n in new_tree.body if isinstance(n, ast.ClassDef))
    new_lines = replacement.splitlines(keepends=True)
    constants = "".join("".join(new_lines[n.lineno - 1:n.end_lineno]) + "\n"
                        for n in new_tree.body if isinstance(n, ast.Assign))
    module = ("".join(lines[:agent.lineno - 1]) + constants + "\n" +
              "".join(lines[agent.lineno - 1:entry.lineno - 1]) +
              "".join(new_lines[fixture.lineno:]) + "".join(lines[agent.end_lineno:]))
    compile(module, "ATS01-mutant-module", "exec")
    return module.encode("utf-8")


def manifest(name, files, checks):
    rationale = ("Whole ATS agent module, including retained class members and every translation table/helper. "
                 "External imports are retained unchanged; this replay tests reporting only." if name == "ats" else
                 "Whole CPU and VitisAI modules plus the entire new checked_run module. "
                 "Unchanged source/tests/configuration form the wider copied boundary. Hardware is excluded.")
    owned_paths = ["ats_agent.py"] if name == "ats" else [PROVIDERS + n + ".py" for n in ("cpu", "vitisai")]
    owned = {p: {"kind": "whole_file", "path": p} for p in owned_paths}
    categories = {p: ("tests" if p.startswith("tests/") else "dependency" if p in ("uv.lock", "pyproject.toml")
                      else "production") for p in files}
    if name == "npu":
        categories[PROVIDERS + "checked_run.py"] = "production"
    return dict(schema_version=1, trial_id=name.upper() + "01-native-replay", hypothesis=(
        "Share bilingual report rendering with translation data and preserved dispatch." if name == "ats" else
        "Share typed batch validation/session/output handling across CPU and VitisAI providers."),
        entry_points=owned, owned=owned, wider_boundary=["."], categories=categories, exclusions={},
        scope_review=dict(schema_version=1, rationale=rationale, relationships={}), checks=checks,
        target=None, capability_inventory=None)


def check_definitions(name, out):
    paths = ([ATS / "trial.py", ATS / "oracle.json"] if name == "ats" else
             [NPU / "run.py", NPU / "account.py", NPU_UPSTREAM / "pyproject.toml"])
    if name == "npu":
        prefix = str(NPU_UPSTREAM / "tests") + "\\"
        paths += [Path(p) for p in read(NPU / "frozen.json") if p.startswith(prefix)]
    definition = dict(schema_version=1, input_sha256={str(p.relative_to(ROOT.parent)).replace("\\", "/"):
                     digest(p.read_bytes()) for p in paths},
                     tested_domain="finite frozen software-only corpus; unchanged native checks where supplied",
                     limitations=["Current bytes hashed during preparation; not historical execution provenance.",
                                  "Fresh external runner and raw artifacts required before native evidence admission."])
    write(out / "check-definition.json", definition)
    definition_digest = digest((out / "check-definition.json").read_bytes())
    checks = {"differential": dict(schema_version=1, description="Frozen behavioral observations", required=True,
              definition_digest=definition_digest, artifact_expectations=["differential.json"],
              interpretation=dict(kind="case_counts", expected=8820 if name == "ats" else 3168))}
    for check in (["dispatch"] if name == "ats" else ["native-tests", "types", "shared-safety"]):
        checks[check] = dict(schema_version=1, description=check + " software contract", required=True,
                            definition_digest=definition_digest, artifact_expectations=[check + ".log"],
                            interpretation=dict(kind="exit_status"))
    return checks


def native_prepare(manifest_path, root, output):
    subprocess.run([str(NATIVE), "prepare", "--manifest", str(manifest_path), "--baseline", str(root),
                    "--out", str(output)], check=True, capture_output=True, text=True, timeout=60)
    return read(output)


def historical(name, preserved):
    if name == "ats":
        observations = {arm: {key: read(ATS / (arm + "-result.json"))[key]
                             for key in ("cases", "mismatches", "dispatch_preserved", "scope")}
                        for arm in ("candidate", "mutant")}
    else:
        result = read(NPU / "result.json")
        observations = dict(result["comparisons"])
        observations["historical_measurements"] = result["measurements"]
        observations["historical_tests"] = result["tests"]
    return dict(native_execution_provenance="unavailable", observations=observations,
                current_artifact_sha256={p: h for p, h in preserved.items() if
                                        p.startswith("docs/experiments/" + ("ats-report/" if name == "ats" else "npu-backends/"))},
                metric_names="Historical labels retained: physical, lloc (Radon), decisions (collector AST), ccn_sum (Lizard). These are not native metric definitions.",
                limitation="Original executions did not bind native frozen trial/check/source identities. Collector success is not behavioral acceptance.")


def prepare_inputs(output):
    output = Path(output).resolve()
    if any(output.is_relative_to(p.resolve()) for p in (ATS, NPU, NPU_UPSTREAM, ROOT.parent / "ats-agent")):
        raise ValueError("output must be outside historical artifacts and upstream repositories")
    output.mkdir(parents=True, exist_ok=False)
    preserved = original_hashes()
    # Only paths explicitly recorded by the historical NPU freeze are eligible.
    frozen_paths = read(NPU / "frozen.json")
    npu_sources = {}
    prefix = str(NPU_UPSTREAM) + "\\"
    for filename, expected in frozen_paths.items():
        if filename.startswith(prefix):
            source = Path(filename)
            if digest(source.read_bytes()) != expected:
                raise ValueError("historical NPU input changed: " + filename)
            npu_sources[source.relative_to(NPU_UPSTREAM).as_posix()] = source
    if not npu_sources:
        raise ValueError("historical NPU source inventory unavailable")
    receipt = dict(schema_version=1, task="6.1", trials={}, originals_before_sha256=preserved)
    for name in ("ats", "npu"):
        trial = output / name
        for arm in ("baseline", "candidate", "mutant"):
            root = trial / arm
            root.mkdir(parents=True)
            if name == "ats":
                if arm == "mutant":
                    (root / "ats_agent.py").write_bytes(ats_mutant_module())
                else:
                    copy(ATS / ("source.py.txt" if arm == "baseline" else "candidate-module.py.txt"), root / "ats_agent.py")
            else:
                for path, source in npu_sources.items():
                    copy(source, root / path)
                for provider in ("cpu", "vitisai"):
                    copy(NPU / ("original" if arm == "baseline" else "candidate") / (provider + ".py.txt"), root / (PROVIDERS + provider + ".py"))
                if arm != "baseline":
                    copy(NPU / arm / "checked_run.py.txt", root / (PROVIDERS + "checked_run.py"))
        checks = check_definitions(name, trial)
        files = [p.relative_to(trial / "baseline").as_posix() for p in (trial / "baseline").rglob("*") if p.is_file()]
        record = manifest(name, files, checks)
        manifest_path = trial / "manifest.json"
        write(manifest_path, record)
        initial = native_prepare(manifest_path, trial / "baseline", trial / "definitions-probe.json")
        record["target"] = dict(schema_version=1, scope_identity="production", language="python",
             definitions={v["name"]: v for v in initial["definitions"].values() if v["language"] == "python"},
             minimum_production_statement_reduction=1, count_bounds={}, permitted_structural_tradeoffs=[
                 "Whole modules retained in capability accounting; function/call-layer changes reported without inferred maintenance savings."])
        write(manifest_path, record)
        frozen = native_prepare(manifest_path, trial / "baseline", trial / "frozen.json")
        identities = {}
        for arm in ("baseline", "candidate", "mutant"):
            identities[arm] = native_prepare(manifest_path, trial / arm, trial / (arm + "-identity.json"))["baseline_identity"]
        replacements = {key: [value] for key, value in record["owned"].items()}
        if name == "npu":
            replacements[PROVIDERS + "cpu.py"].append(dict(kind="whole_file", path=PROVIDERS + "checked_run.py"))
        evidence = dict(schema_version=1, candidate_ownership=dict(schema_version=1, replacements=replacements,
                        review=record["scope_review"]), checks={key: dict(schema_version=1, baseline=None, candidate=None)
                        for key in checks}, dependency_changes={})
        for arm in ("candidate", "mutant"):
            write(trial / (arm + "-evidence.json"), evidence)
        receipt["trials"][name] = dict(source_identities=identities, definitions=frozen["definitions"],
             check_definition_digests=frozen["check_definition_digests"], historical=historical(name, preserved),
             coverage="Whole module replacement; finite corpus and external checks pending fresh replay.",
             native_check_outcomes="missing; task 6.2/6.3 must produce new bound raw results")
    receipt["originals_after_sha256"] = original_hashes()
    if receipt["originals_after_sha256"] != preserved:
        raise ValueError("historical artifacts changed during preparation")
    write(output / "preparation-receipt.json", receipt)
    return receipt


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True, help="fresh isolated output, normally .runs/native-replay/NAME")
    args = parser.parse_args()
    prepare_inputs(args.out)
    print("Prepared ATS/NPU inputs; fresh behavioral evidence remains missing.")
