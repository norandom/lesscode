"""Run the local pilot with a clean installed wheel and real provisioned analyzers.

Run this script using the clean environment's interpreter from outside the
checkout. Outputs are retained, including CLI stdout/stderr and frozen inputs.
No network access, model calls, source changes or tool installation occur here.
"""

import argparse
from copy import deepcopy
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8", newline="\n")


def fingerprints(root):
    return {p.relative_to(root).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted(root.rglob("*")) if p.is_file()}


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def resume_completed(output, tool_paths):
    """Read-only recovery; reject any changed binding before writes or workloads.

    Partial validations deliberately require a new output directory. This avoids
    repairing a supplied source or blessing changed input as a resumed attempt.
    """
    from lesscode_baseline import contracts as c
    from lesscode_baseline.config import verify_integrity
    from lesscode_baseline.report import recover, recover_experiment
    from lesscode_baseline.snapshot import capture_scope

    validation = json.loads((output / "validation.json").read_bytes())
    require(validation["status"] == "passed", "only completed validations can resume")
    require(str(tool_paths.resolve()) == validation["tool_paths"] and
            hashlib.sha256(tool_paths.read_bytes()).hexdigest() == validation["tool_paths_sha256"],
            "tool configuration changed")
    for group in ("source_fingerprints", "checkout_fingerprints"):
        for root, hashes in validation[group].items():
            require(fingerprints(Path(root)) == hashes, f"frozen input differs: {root}")
    reports = {}
    for name, decision in validation["run_decisions"].items():
        require(Path(name).name == name, "invalid run name")
        directory = output / "runs" / name
        recovered = recover(directory)
        require(recovered.state == "complete", f"{name}: incomplete indexed run")
        outer = c.loads(c.PreparedPlan, (output / (name + "-plan.json")).read_bytes())
        frozen = c.loads(c.PreparedPlan, (directory / "plan.json").read_bytes())
        canonical = c.dumps(outer).encode("utf-8")
        require(canonical == c.dumps(frozen).encode("utf-8"), f"{name}: outer/frozen plan mismatch")
        require(recovered.report.plan_hash == hashlib.sha256(canonical).hexdigest(),
                f"{name}: report is not bound to requested canonical PreparedPlan")
        require(verify_integrity(outer).status == "unchanged", f"{name}: frozen evaluator/tools changed")
        for original, observation in ((outer.plan.baseline, recovered.report.baseline),
                                      (outer.plan.candidate, recovered.report.candidate)):
            snapshot = capture_scope(Path(original.root), outer.plan.profile.scope)
            require(isinstance(snapshot, c.Snapshot) and snapshot.content_hash == original.content_hash
                    == observation.snapshot.content_hash, f"{name}: selected source changed")
        require(recovered.report.decision == decision, f"{name}: recorded decision differs")
        reports[name] = recovered.report
    summary = recover_experiment(output / "summary")
    require(summary.state == "complete", "completed summary missing or changed")
    manifest = c.loads(c.ExperimentManifest, (output / "inputs/experiment.json").read_bytes())
    require(summary.report.manifest == manifest, "experiment manifest changed")
    for record in (*summary.report.trials, *summary.report.extra_trials):
        require(Path(record.trial_id).name == record.trial_id, "invalid trial identity")
        stored = c.loads(c.TrialRecord, (output / "inputs" / (record.trial_id + ".json")).read_bytes())
        require(stored == record, "producer record changed")
    # Reconcile current completed runs against the exact strict producer inputs;
    # a summary's own index alone does not bind it to today's run artifacts.
    from lesscode_baseline.experiments import aggregate
    current = aggregate(manifest, summary.report.trials + summary.report.extra_trials,
                        artifact_root=output / "runs")
    require(current.report == summary.report, "completed summary differs from current run evidence")
    return reports, summary.report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--checkout", type=Path, required=True)
    parser.add_argument("--tool-paths", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--resume", action="store_true", help="read-only verification of a completed pilot; partial pilots require a new output")
    args = parser.parse_args()
    checkout, output = args.checkout.resolve(), args.out.resolve()
    require("PYTHONPATH" not in os.environ, "remove PYTHONPATH for installed validation")
    require(not Path.cwd().resolve().is_relative_to(checkout), "run outside the checkout")
    require(not output.is_relative_to(checkout), "output must be outside checkout")
    import lesscode_baseline
    from lesscode_baseline import contracts as c
    from lesscode_baseline.report import recover, recover_experiment
    from lesscode_baseline.snapshot import capture_scope
    package = Path(lesscode_baseline.__file__).resolve()
    require(not package.is_relative_to(checkout), "validation requires an installed wheel")
    if args.resume:
        resume_completed(output, args.tool_paths)
        print(f"Completed pilot recovered read-only: {output / 'validation.json'}")
        return
    output.mkdir(parents=True, exist_ok=False)
    console = Path(sys.executable).with_name("lesscode-baseline.exe" if os.name == "nt" else "lesscode-baseline")
    require(console.is_file(), "installed console entry point missing")
    inputs, runs = output / "inputs", output / "runs"
    inputs.mkdir()
    runs.mkdir()
    commands = []

    def command(argv, name, expected=0):
        result = subprocess.run([str(x) for x in argv], cwd=output, capture_output=True,
                                timeout=240, env=dict(os.environ))
        (output / (name + ".stdout.txt")).write_bytes(result.stdout)
        (output / (name + ".stderr.txt")).write_bytes(result.stderr)
        commands.append({"argv": [str(x) for x in argv], "exit": result.returncode,
                         "stdout": name + ".stdout.txt", "stderr": name + ".stderr.txt"})
        write_json(output / "commands.json", commands)
        require(result.returncode == expected,
                f"{name}: expected exit {expected}, got {result.returncode}; inspect retained logs")
        return result

    profile_path = checkout / "benchmarks/profiles/python.json"
    profile = c.loads(c.Profile, profile_path.read_bytes())
    real = checkout / "tests/fixtures/real/boltons_mathutils"
    synthetic = checkout / "tests/fixtures/python"
    originals = {str(root): fingerprints(root) for root in (real, synthetic, profile_path.parent)}
    manifest = json.loads((checkout / "benchmarks/corpus.json").read_bytes())
    # The recorded experiment declares exactly the three evaluations executed
    # below. These are supplied-snapshot evaluator trials, not model generations.
    manifest["experiment_id"] = "installed-local-reproducibility-v1"
    manifest["arms"][1]["interventions"]["transformation"] = "shorter broken supplied snapshot"
    manifest["planned_trials"] = [p for p in manifest["planned_trials"]
                                  if p["trial_id"] in ("control-1", "control-2", "candidate-1")]
    manifest["planned_trials"].sort(key=lambda p: ("control-1", "control-2", "candidate-1").index(p["trial_id"]))
    manifest["controls"]["settings"]["order"] = "control-1, control-2, candidate-1"
    manifest["controls"]["settings"]["scope_hash"] = capture_scope(real / "source", profile.scope).scope_hash
    c.ExperimentManifest.from_dict(manifest)
    write_json(inputs / "experiment.json", manifest)  # Before any evaluations.
    manifest_before = fingerprints(inputs)
    broken = inputs / "real-broken"
    broken.mkdir()
    broken_bytes = b"def clamp(x, lower=0, upper=1): return x\n"
    (broken / "mathutils.py").write_bytes(broken_bytes)
    reports = {}
    source_before = {}

    def evaluate(name, baseline, candidate, selected_profile, expected=0, module=False):
        before = {str(root): fingerprints(root) for root in (baseline, candidate)}
        source_before.update(before)
        prefix = [sys.executable, "-m", "lesscode_baseline"] if module else [console]
        plan_path = output / (name + "-plan.json")
        command(prefix + ["plan", "--profile", selected_profile, "--baseline", baseline,
                "--candidate", candidate, "--tool-paths", args.tool_paths.resolve(), "--out", plan_path], name + "-plan")
        prepared = c.loads(c.PreparedPlan, plan_path.read_bytes())
        require(all(t.status == "available" for t in prepared.tools), "all pinned tools must be available")
        command(prefix + ["run", "--plan", plan_path, "--out", runs / name], name + "-run", expected)
        recovery = recover(runs / name)
        require(recovery.state == "complete", f"{name}: incomplete recovery {recovery.diagnostics}")
        report = recovery.report
        require(report.plan_hash == hashlib.sha256(c.dumps(prepared).encode("utf-8")).hexdigest(),
                f"{name}: report is not bound to requested canonical PreparedPlan")
        require(c.loads(c.PreparedPlan, (runs / name / "plan.json").read_bytes()) == prepared,
                f"{name}: outer/frozen plan mismatch")
        require(report.integrity.status == "unchanged", f"{name}: integrity changed")
        for root, observation in ((baseline, report.baseline), (candidate, report.candidate)):
            require(capture_scope(root, prepared.plan.profile.scope).content_hash == observation.snapshot.content_hash,
                    f"{name}: recovered source differs from current input")
        require(all(m.status == "available" for obs in ((report.baseline,) if expected else (report.baseline, report.candidate))
                    for m in obs.metrics if m.key in profile.required_metrics), f"{name}: unavailable core evidence")
        require(report.decision == ("rejected" if expected == 1 else "accepted"), f"{name}: unexpected decision")
        require(before == {str(root): fingerprints(root) for root in (baseline, candidate)}, f"{name}: originals changed")
        reports[name] = report
        return report

    first = evaluate("control-1", real / "source", real / "source", profile_path)
    second = evaluate("control-2", real / "source", real / "source", profile_path, module=True)
    rejected = evaluate("candidate-1", real / "source", broken, profile_path, expected=1)

    def structural(observation):
        # Only volatile raw artifact locations are removed. Values, definitions,
        # scope and complete producer/version metadata must match exactly.
        return [dict(m.to_dict(), raw_artifact=None) for m in observation.metrics]

    require(structural(first.baseline) == structural(first.candidate) == structural(second.baseline)
            == structural(second.candidate), "identical installed structural evidence differs")
    metrics = {m.key: m for m in first.baseline.metrics}
    require((metrics["complexity"].value.count, metrics["complexity"].value.maximum) == (23, 15), "boltons known complexity changed")
    highest = max(metrics["complexity"].value.functions, key=lambda f: f.ccn)
    require((highest.path, highest.name, highest.start_line, highest.end_line) ==
            ("mathutils.py", "__init__", 140, 167), "known highest-complexity location changed")
    require(first.decision == second.decision == "accepted" and rejected.decision == "rejected", "real outcome mismatch")
    require(any(check.status in ("failed", "error") for check in rejected.candidate.checks), "shorter source must fail independent checks")
    require(next(m.value for m in rejected.candidate.metrics if m.key == "source_bytes") < metrics["source_bytes"].value, "broken candidate is not shorter")

    # Freeze the synthetic evaluator outside selected sources. JUnit binds to
    # current source/plan at import; the oracle's positional hash is unused there.
    evaluator = inputs / "synthetic-evaluator"
    shutil.copytree(synthetic / "oracle", evaluator / "oracle")
    synth_profile = profile.to_dict()
    synth_profile["checks"] = [{"check_id": "oracle", "kind": "invariant", "required": True,
        "command": {"command_id": "oracle", "argv": ["python", "oracle/check.py", "subject.py", "checks.xml", "junit", "0" * 64],
                    "cwd": ".", "environment": {"PYTHONDONTWRITEBYTECODE": "1"}, "limits": profile.limits.to_dict()},
        "result_path": "checks.xml", "result_format": "junit",
        "expected_ids": ["clamp.below", "clamp.inside", "clamp.above", "clamp.bounds"]}]
    synth_profile["scope"]["generated"] = ["generated/**"]
    synth_profile["scope"]["vendor"] = ["vendor/**"]
    synth_profile["evaluator_resources"] = ["profile.json", "oracle/check.py", "oracle/golden.json"]
    write_json(evaluator / "profile.json", synth_profile)
    baseline, candidate = inputs / "synthetic-baseline", inputs / "synthetic-candidate"
    shutil.copytree(synthetic / "pairs/duplicated/baseline", baseline)
    shutil.copytree(synthetic / "pairs/duplicated/candidate", candidate)
    # Preserve fixture originals; expand this supplied candidate to six
    # meaningful lines so pinned jscpd's fixed min-lines scope can measure it.
    (candidate / "subject.py").write_text("def clamp(value, lower, upper):\n    result = min(upper, max(lower, value))\n    result += 0\n    result += 0\n    result += 0\n    return result\n", encoding="utf-8")
    # Git index proves that a selected product file is untracked. No commit,
    # global Git setting, or developer checkout modification is needed.
    command(["git", "init", str(candidate)], "synthetic-git-init")
    command(["git", "-C", candidate, "add", "subject.py"], "synthetic-git-add")
    (candidate / "untracked_extra.py").write_bytes(b"")
    for directory in ("tests", "generated", "vendor"):
        (candidate / directory).mkdir()
        (candidate / directory / "added.py").write_text("# selected non-product input\n", encoding="utf-8")
    untracked = command(["git", "-C", candidate, "ls-files", "--others", "--exclude-standard"], "synthetic-git-untracked")
    require(b"untracked_extra.py" in untracked.stdout, "untracked input setup failed")
    inventory = evaluate("synthetic-inventory", baseline, candidate, evaluator / "profile.json", module=True)
    require(inventory.decision == "accepted", "synthetic duplicate removal not accepted")
    files = {f.path: f.category for f in inventory.candidate.snapshot.files}
    require("redundant.py" in {f.path for f in inventory.baseline.snapshot.files} and "redundant.py" not in files,
            "removed source absent from baseline inventory")
    require(files.get("untracked_extra.py") == "product" and files.get("tests/added.py") == "tests"
            and files.get("generated/added.py") == "generated" and files.get("vendor/added.py") == "vendor", "selected classes/inventory mismatch")
    require(next(m.value for m in inventory.candidate.metrics if m.key == "source_bytes") ==
            sum(f.size_bytes for f in inventory.candidate.snapshot.files if f.category == "product"),
            "non-product files entered product-source totals")
    require(inventory.test_file_delta.added == ("tests/added.py",), "test changes not reported separately")
    broken_synth = inputs / "synthetic-broken"
    shutil.copytree(synthetic / "pairs/shorter_broken/candidate", broken_synth)
    synth_rejected = evaluate("synthetic-broken", baseline, broken_synth, evaluator / "profile.json", expected=1)
    require(synth_rejected.decision == "rejected" and any(x.status == "failed" for x in synth_rejected.candidate.checks), "synthetic shorter behavior failure missing")

    # Known jscpd aggregate pair spans, kept separate from clamp fixture tokens.
    duplicate = inputs / "known-duplicates"
    (duplicate / "nested").mkdir(parents=True)
    code = (b"def classify(value):\n    result = 0\n    if value > 0:\n        result += value\n    for item in range(value):\n        result += item\n"
            + b"    result += 1\n" * 9 + b"    return result\n")
    (duplicate / "nested/a.py").write_bytes(code)
    (duplicate / "b.py").write_bytes(code)
    (duplicate / "empty.py").write_bytes(b"")
    dup_evaluator = inputs / "duplicate-evaluator"
    dup_evaluator.mkdir()
    (dup_evaluator / "check.py").write_text('import runpy\nfrom pathlib import Path\nassert runpy.run_path("b.py")["classify"](2) == 12\nPath("checks.xml").write_text(\'<testsuite tests="1"><testcase name="known.classify"/></testsuite>\')\n', encoding="utf-8")
    dup_profile = deepcopy(synth_profile)
    dup_profile["checks"][0]["command"]["argv"] = ["python", "check.py"]
    dup_profile["checks"][0]["expected_ids"] = ["known.classify"]
    dup_profile["evaluator_resources"] = ["profile.json", "check.py"]
    write_json(dup_evaluator / "profile.json", dup_profile)
    duplicated = evaluate("known-duplicates", duplicate, duplicate, dup_evaluator / "profile.json")
    payload = next(m.value for m in duplicated.baseline.metrics if m.key == "duplication")
    require((payload.total_tokens, payload.duplicated_tokens, payload.ratio) == (236, 118, .5), "known duplication counts changed")
    require(payload.scanned_paths == ("b.py", "nested/a.py") and payload.zero_token_paths == ("empty.py",), "duplicate source inventory mismatch")
    require(len(payload.clones) == 1 and {payload.clones[0].first.path, payload.clones[0].second.path} ==
            {"b.py", "nested/a.py"} and all((span.start_line, span.end_line) == (1, 16)
            for span in (payload.clones[0].first, payload.clones[0].second)), "known clone locations changed")

    unavailable = lambda unit: {"status": "unavailable", "value": None, "unit": unit,
        "source": None, "observed_at": None, "reason": "no agent/model used; not measured"}
    trial_paths = []
    for order, trial in enumerate(manifest["planned_trials"], 1):
        conditions = deepcopy(manifest["controls"])
        conditions["interventions"] = next(a["interventions"] for a in manifest["arms"] if a["arm_id"] == trial["arm_id"])
        record = {k: trial[k] for k in ("trial_id", "arm_id", "corpus_id", "task_id", "repetition")}
        record.update(schema_version=1, starting_source_hash=reports[trial["trial_id"]].baseline.snapshot.content_hash,
            run_artifact=trial["trial_id"], conditions=conditions, execution_order=order, seed=0,
            completion_state="completed", budget_exhausted=False,
            telemetry={"token_usage": unavailable("tokens"), "provider_cost": unavailable("USD"),
                       "agent_elapsed": unavailable("seconds"), "runtime_benchmarks": []})
        c.TrialRecord.from_dict(record)
        path = inputs / (trial["trial_id"] + ".json")
        write_json(path, record)
        trial_paths += ["--trials", path]
    command([console, "summarize", "--experiment", inputs / "experiment.json", "--runs", runs,
             "--out", output / "summary", *trial_paths], "summarize", expected=1)
    summary = recover_experiment(output / "summary")
    require(summary.state == "complete", f"summary recovery incomplete: {summary.diagnostics}")
    require(all(not o.confounded_differences for o in summary.report.outcomes), "pilot conditions falsely confounded")
    require([o.decision for o in summary.report.outcomes].count("accepted") == 2 and
            [o.decision for o in summary.report.outcomes].count("rejected") == 1, "summary counts wrong")
    require(fingerprints(inputs)["experiment.json"] == manifest_before["experiment.json"], "schedule changed after execution")
    require(all(fingerprints(Path(root)) == hashes for root, hashes in originals.items()), "checkout originals changed")
    require(all(fingerprints(Path(root)) == hashes for root, hashes in source_before.items()), "supplied inputs changed")
    write_json(output / "validation.json", {"status": "passed", "package": str(package),
        "python": sys.version, "platform": sys.platform, "tool_paths": str(args.tool_paths.resolve()),
        "tool_paths_sha256": hashlib.sha256(args.tool_paths.read_bytes()).hexdigest(),
        "source_fingerprints": source_before, "checkout_fingerprints": originals,
        "run_decisions": {k: v.decision for k, v in reports.items()},
        "structural_determinism": "full MetricEvidence equality excluding raw_artifact paths; elapsed/timestamps not compared",
        "repeated_structural_evidence": structural(first.baseline),
        "known_values": {"boltons_functions": 23, "boltons_max_ccn": 15, "duplicate_total_tokens": 236,
                         "duplicate_pair_tokens": 118, "duplicate_pair_ratio": .5},
        "recovery": "all six runs and experiment complete with verified artifact indexes",
        "limitations": ["native platform only", "supplied snapshots; no model generations or efficacy claim",
                        "declared checks pass, not universal equivalence", "local change detection, not hostile-code isolation"]})
    print(f"Pilot passed: {output / 'validation.json'}")


if __name__ == "__main__":
    main()
