"""Admit completed baseline stores without executing or recapturing source."""

import hashlib
import json
from pathlib import Path

from lesscode_baseline import contracts as baseline
from lesscode_baseline.report import ArtifactIndex, RunManifest, recover
from .contracts import EvidenceBundle, EvidenceInventory, Policy, RuleFailure

def read_evidence(run_directory: Path, policy: Policy) -> EvidenceBundle | RuleFailure:
    """Bind a recovered report to its indexed plan and explicit same-run reference.

    Admission is independent of evaluation success. Original outcomes, failures,
    limitations and unavailable measurements remain in the unmodified report.
    This does not check current source/evaluator files or authenticate producers.
    """
    recovered = recover(run_directory)
    if recovered.state != "complete":
        # Recovery combines missing storage and corruption. Inspect surviving
        # evidence before reporting insufficiency; absence cannot hide damage.
        try:
            report = _validate_partial_store(recovered.directory, policy)
        except (baseline.ContractError, ValueError, OSError) as exc:
            return RuleFailure(1, "prepare", "invalid",
                               (str(exc), *recovered.diagnostics), None)
        diagnostics = recovered.diagnostics
        if report is not None:
            diagnostics = (*diagnostics, f"upstream outcome: {report.decision}",
                           *report.reasons, *report.limitations)
        return RuleFailure(1, "prepare", "insufficient", diagnostics, None)
    report = recovered.report
    indexed = {artifact.path: artifact for artifact in recovered.index.artifacts}
    diagnostics = (f"upstream outcome: {report.decision}", *report.reasons,
                   *report.limitations)
    if "plan.json" not in indexed:
        try:
            _validate_available_records(None, report, policy)
        except baseline.ContractError as exc:
            return RuleFailure(1, "prepare", "invalid", (str(exc), *diagnostics), None)
        return RuleFailure(1, "prepare", "insufficient",
                           ("required indexed PreparedPlan plan.json is missing", *diagnostics), None)
    try:
        plan_bytes = (recovered.directory / "plan.json").read_bytes()
        artifact = indexed["plan.json"]
        if (hashlib.sha256(plan_bytes).hexdigest(), len(plan_bytes)) != (artifact.content_hash, artifact.size_bytes):
            raise baseline.ContractError("plan.json changed after recovery")
        prepared = baseline.loads(baseline.PreparedPlan, plan_bytes)
        _validate_available_records(prepared, report, policy)
    except (baseline.ContractError, ValueError, OSError) as exc:
        return RuleFailure(1, "prepare", "invalid", (str(exc), *diagnostics), None)
    reason = "Entity quantities have not been selected from the admitted report"
    return EvidenceBundle(1, report, prepared.plan.profile_hash,
                          EvidenceInventory(report.baseline.snapshot.content_hash, "unavailable", reason, ()),
                          EvidenceInventory(report.candidate.snapshot.content_hash, "unavailable", reason, ()))


def _read_present(path):
    """Read surviving regular artifacts, never following a link/reparse point."""
    for part in (path, *path.parents):
        if part.is_symlink() or part.is_junction():
            raise baseline.ContractError(f"artifact path traverses link/reparse point: {path}")
    try:
        return path.read_bytes()
    except FileNotFoundError:
        return None


def _validate_partial_store(directory, policy):
    """Reject detectable corruption without treating missing bytes as valid data."""
    records = {}
    for name, record_type in (("manifest.json", RunManifest), ("artifacts.json", ArtifactIndex),
                              ("report.json", baseline.RunReport), ("plan.json", baseline.PreparedPlan)):
        content = _read_present(directory / name)
        records[name] = None if content is None else baseline.loads(record_type, content)
    manifest, index, report, prepared = (records[name] for name in
                                        ("manifest.json", "artifacts.json", "report.json", "plan.json"))
    identities = {(record.run_id, record.plan_hash) for record in (manifest, index, report)
                  if record is not None}
    if len(identities) > 1:
        raise baseline.ContractError("present report/manifest/index identities differ")
    if index is not None and manifest is not None and manifest.state != "finalized":
        raise baseline.ContractError("completion index conflicts with unfinalized manifest")
    artifacts = {}
    for artifact in (*(manifest.raw_artifacts if manifest else ()), *(index.artifacts if index else ())):
        if artifact.path in artifacts and artifacts[artifact.path] != artifact:
            raise baseline.ContractError(f"index/manifest artifact identities differ: {artifact.path}")
        artifacts[artifact.path] = artifact
    for artifact in artifacts.values():
        artifact_path = directory / artifact.path
        if not artifact_path.resolve().is_relative_to(directory.resolve()):
            raise baseline.ContractError(f"artifact path escapes run directory: {artifact.path}")
        content = _read_present(artifact_path)
        if content is not None and (hashlib.sha256(content).hexdigest(), len(content)) != (
                artifact.content_hash, artifact.size_bytes):
            raise baseline.ContractError(f"artifact identity mismatch: {artifact.path}")
    if index is not None:
        indexed = {artifact.path for artifact in index.artifacts}
        required = {"manifest.json", "report.json", "summary.md"}
        if prepared is not None:
            required.add("plan.json")
        if manifest is not None:
            required.update(artifact.path for artifact in manifest.raw_artifacts)
        if not required.issubset(indexed):
            raise baseline.ContractError("index omits required run artifacts")
        if report is not None:
            references = set()
            for observation in (report.baseline, report.candidate):
                references.update(metric.raw_artifact for metric in observation.metrics)
                references.update(check.diagnostic_artifact for check in observation.checks)
                references.update(failure.raw_artifact for failure in observation.failures)
                for execution in observation.executions:
                    references.update((execution.stdout_artifact, execution.stderr_artifact))
            references.discard(None)
            unindexed = references - indexed
            if unindexed:
                raise baseline.ContractError(f"report references unindexed raw artifacts: {sorted(unindexed)}")
    _validate_available_records(prepared, report, policy)
    if prepared is not None:
        plan_hash = hashlib.sha256(baseline.dumps(prepared).encode()).hexdigest()
        if any(identity[1] != plan_hash for identity in identities):
            raise baseline.ContractError("present plan identity differs from report/manifest/index")
    return report


def _validate_available_records(prepared, report, policy):
    """Validate each identity as soon as its actual prerequisites are present."""
    policy = Policy.from_dict(policy.to_dict())
    if prepared is not None:
        _validate_plan_identity(prepared)
    if report is not None:
        _validate_report_bindings(report)
    if prepared is not None and report is not None:
        _validate_bindings(prepared, report)
    if policy.reference is not None:
        baseline_snapshot = (prepared.plan.baseline if prepared is not None else
                             report.baseline.snapshot if report is not None else None)
        if baseline_snapshot is not None and policy.reference.source_hash != baseline_snapshot.content_hash:
            raise baseline.ContractError("policy.reference.source_hash differs from same-run baseline")
        if prepared is not None and policy.reference.profile_hash != prepared.plan.profile_hash:
            raise baseline.ContractError("policy.reference.profile_hash differs from frozen profile")


def _validate_snapshot_identity(snapshot, side):
    source_bytes = json.dumps([[f.path, f.size_bytes, f.content_hash] for f in snapshot.files],
                             ensure_ascii=False, separators=(",", ":")).encode()
    if hashlib.sha256(source_bytes).hexdigest() != snapshot.content_hash:
        raise baseline.ContractError(f"{side} source identity differs from file inventory")


def _validate_report_bindings(report):
    for side in ("baseline", "candidate"):
        observation = getattr(report, side)
        _validate_snapshot_identity(observation.snapshot, side)
        for check in observation.checks:
            if (check.source_hash, check.plan_hash) != (observation.snapshot.content_hash, report.plan_hash):
                raise baseline.ContractError(f"{side} check {check.check_id} source/plan binding differs")
        for metric in observation.metrics:
            if metric.scope_hash != observation.snapshot.scope_hash:
                raise baseline.ContractError(f"{side} metric {metric.key} scope differs from snapshot")


def _validate_plan_identity(prepared):
    plan = prepared.plan
    if hashlib.sha256(baseline.dumps(plan.profile).encode()).hexdigest() != plan.profile_hash:
        raise baseline.ContractError("plan.profile_hash differs from frozen profile")
    resource_bytes = json.dumps([(r.logical_path, r.path, r.content_hash, r.size_bytes)
                                 for r in prepared.evaluator_resources], separators=(",", ":")).encode()
    if hashlib.sha256(resource_bytes).hexdigest() != plan.evaluator_hash:
        raise baseline.ContractError("plan.evaluator_hash differs from prepared resource inventory")
    scope_hash = hashlib.sha256(json.dumps(plan.profile.scope.to_dict(), ensure_ascii=False,
                               sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    for side in ("baseline", "candidate"):
        snapshot = getattr(plan, side)
        _validate_snapshot_identity(snapshot, side)
        if snapshot.scope_hash != scope_hash:
            raise baseline.ContractError(f"{side} snapshot scope differs from frozen profile")


def _validate_bindings(prepared, report):
    plan = prepared.plan
    if hashlib.sha256(baseline.dumps(prepared).encode()).hexdigest() != report.plan_hash:
        raise baseline.ContractError("plan.json canonical identity differs from report.plan_hash")
    declared = {identity: spec for spec in plan.profile.checks for identity in spec.expected_ids}
    for side in ("baseline", "candidate"):
        observation = getattr(report, side)
        if observation.snapshot != getattr(plan, side):
            raise baseline.ContractError(f"{side} report snapshot differs from prepared plan")
        for check in observation.checks:
            spec = declared.get(check.check_id)
            if spec is None or (spec.kind, spec.required) != (check.kind, check.required):
                raise baseline.ContractError(f"{side} check {check.check_id} differs from frozen check inventory")
        # Absent checks/metrics remain absent evidence, not storage corruption.
