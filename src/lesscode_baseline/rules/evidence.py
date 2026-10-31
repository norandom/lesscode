"""Admit completed baseline stores without executing or recapturing source."""

import hashlib
import json
from dataclasses import replace
from pathlib import Path

from lesscode_baseline import contracts as baseline
from lesscode_baseline.report import ArtifactIndex, RunManifest, recover
from lesscode_baseline.decision import compare_metrics
from lesscode_baseline.snapshot import matches
from .contracts import (EvidenceBundle, EvidenceInventory, EntityObservation, EntityTreatment,
                        NumericRule, Policy, PolicyScope, RuleFailure, RuleSelection, Selector,
                        policy_hash, validate_treatments)


def establish_continuity(evidence: EvidenceBundle, policy: Policy,
                         treatments: tuple[EntityTreatment, ...] = ()) -> tuple[EntityTreatment, ...]:
    """Resolve exact inventory addresses without certifying semantic equivalence.

    Supplied treatments are disclosed producer claims. Every omitted address
    remains unresolved unless unchanged product bytes and inventory identity
    establish an automatic match. Missing entities are never assumed removed
    or new: a rename, split or merge may explain either side's absence.
    """
    evidence = EvidenceBundle.from_dict(evidence.to_dict())
    policy = Policy.from_dict(policy.to_dict())
    supplied = validate_treatments(treatments, evidence, policy)
    resolved = list(supplied)
    before_used = {r.baseline_id for r in supplied if r.baseline_id is not None}
    after_used = {r.candidate_id for r in supplied if r.candidate_id is not None}
    before_files, after_files = (
        {f.path: (f.content_hash, f.size_bytes) for f in observation.snapshot.files
         if f.category == "product"}
        for observation in (evidence.report.baseline, evidence.report.candidate))
    binding = (policy_hash(policy), evidence.baseline_inventory.source_hash,
               evidence.candidate_inventory.source_hash)

    def record(kind, before, after, rationale):
        return EntityTreatment(*binding, kind, before, after, "lesscode:entity-continuity-v1", rationale)

    # Snapshot source hashes legitimately differ when another file changes.
    # Exclude those hashes and values from the address; retain the complete
    # measurement identity so incompatible analyzer inventories cannot match.
    candidates = {}
    for entity in evidence.candidate_inventory.entities:
        if entity.inventory_id not in after_used:
            candidates.setdefault(_continuity_address(entity), []).append(entity)
    for before in sorted(evidence.baseline_inventory.entities, key=lambda e: e.inventory_id):
        if before.inventory_id in before_used:
            continue
        possible = candidates.get(_continuity_address(before), ())
        if before.location is None:
            identical_bytes = bool(before_files) and before_files == after_files
        else:
            path = before.location.path
            identical_bytes = path in before_files and before_files[path] == after_files.get(path)
        if identical_bytes and len(possible) == 1 and possible[0].inventory_id not in after_used:
            after = possible[0]
            resolved.append(record("matched", before.inventory_id, after.inventory_id,
                "Automatic continuity: identical product file bytes and inventory identity; no semantic certification."))
            before_used.add(before.inventory_id)
            after_used.add(after.inventory_id)
    for inventory, used, side in ((evidence.baseline_inventory, before_used, "baseline"),
                                  (evidence.candidate_inventory, after_used, "candidate")):
        for entity in inventory.entities:
            if entity.inventory_id not in used:
                resolved.append(record("unresolved", entity.inventory_id if side == "baseline" else None,
                    entity.inventory_id if side == "candidate" else None,
                    "Reference treatment not established; rename, split, merge or other change may require an explicit treatment. No inherited-debt allowance established."))
    # Reuse the public validator for generated and supplied records together.
    resolved.sort(key=lambda r: (r.baseline_id or "", r.candidate_id or "", r.kind))
    return validate_treatments(tuple(resolved), evidence, policy)


def _continuity_address(entity: EntityObservation):
    metric = entity.metric
    return (entity.selector, metric.key, metric.definition_id, metric.producer,
            metric.producer_version, metric.unit, metric.scope_hash,
            entity.location, entity.long_name)


def select_observations(evidence: EvidenceBundle, rule: NumericRule) -> RuleSelection:
    """Select a named quantity without interpreting debt or changing evidence."""
    rule = NumericRule.from_dict(rule.to_dict())
    candidate, after = _select_side(evidence.report.candidate, rule)
    if rule.mode == "absolute" and rule.change_budget is None:
        return RuleSelection(1, rule.rule_id, None, candidate, "not_required", ())
    reference, before = _select_side(evidence.report.baseline, rule)
    reasons = []
    for side, inventory in (("baseline", reference), ("candidate", candidate)):
        if inventory.status != "complete":
            reasons.append(f"{side}: {inventory.reason}")
    # Compare original structured measurements as well as the projected scalar.
    # Structured delta unavailability is expected; compatibility is independent.
    comparisons = compare_metrics((before,) if before else (), (after,) if after else ())
    for comparison in comparisons:
        if comparison.status != "available":
            reasons.append(comparison.delta.reason)
    # Each side covers the whole applicable population. Added/removed product
    # files under unchanged scope rules remain visible in the original report;
    # their changing source addresses do not alone invalidate a comparison.
    if rule.change_budget is not None and rule.change_budget.kind == "percentage_increase":
        if any(entity.metric.value == 0 for entity in reference.entities):
            reasons.append("percentage comparison unavailable: baseline value is zero")
    reasons = tuple(dict.fromkeys(reasons))
    return RuleSelection(1, rule.rule_id, reference, candidate,
                         "unavailable" if reasons else "comparable", reasons)


def _scope_contains(path: str, scope: PolicyScope) -> bool:
    if scope.root != ".":
        prefix = scope.root + "/"
        if not path.startswith(prefix):
            return False
        path = path[len(prefix):]
    return any(matches(path, pattern) for pattern in scope.include) and not any(
        matches(path, pattern) for pattern in scope.exclude)


def _metric_key(selector: Selector) -> str:
    if selector.kind == "scalar_metric":
        return selector.metric_key
    return "complexity" if selector.kind == "function_ccn" else "duplication"


def _select_side(observation: baseline.Observation, rule: NumericRule):
    source = observation.snapshot.content_hash
    def unavailable(reason):
        return EvidenceInventory(source, "unavailable", reason, ()), metric
    metric = next((m for m in observation.metrics if m.key == _metric_key(rule.selector)), None)
    product = {f.path for f in observation.snapshot.files if f.category == "product"}
    selected = {path for path in product if _scope_contains(path, rule.scope)}
    if not selected or not any(f.size_bytes for f in observation.snapshot.files if f.path in selected):
        return unavailable("scope has no matching nonempty product population")
    if rule.selector.kind != "function_ccn" and selected != product:
        return unavailable("aggregate population cannot be narrowed to the rule scope")
    if metric is None:
        return unavailable(f"metric {_metric_key(rule.selector)} missing")
    entities, reason = _metric_entities(observation, metric, rule.selector)
    if reason is not None:
        return unavailable(reason)
    differences = []
    # Units describe the selected quantity; the untouched report retains the
    # analyzer's payload unit, including its pair-span denominator.
    projected_unit = _quantity_unit(metric, rule.selector)
    for field in ("definition_id", "producer", "producer_version", "unit"):
        actual = projected_unit if field == "unit" else getattr(metric, field)
        if actual != getattr(rule.evidence, field):
            differences.append(f"{field}: expected {getattr(rule.evidence, field)!r}, observed {actual!r}")
    if differences:
        return unavailable("; ".join(differences))
    if rule.selector.kind == "function_ccn":
        entities = tuple(e for e in entities if e.location.path in selected)
    return EvidenceInventory(source, "complete", None, entities), metric


def _quantity_unit(metric: baseline.MetricEvidence, selector: Selector) -> str:
    if selector.kind == "duplication_clone_pairs":
        return "clone_pairs"
    if selector.kind in ("duplication_duplicated_tokens", "duplication_total_tokens"):
        return "tokens"
    return metric.unit


def _metric_entities(observation, metric, selector):
    """Project validated payloads at deterministic, rule-independent addresses."""
    if metric.status != "available":
        return (), f"metric {metric.key} {metric.status}: {metric.reason}"
    if metric.scope_hash != observation.snapshot.scope_hash:
        return (), "metric scope_hash differs from snapshot"
    product = {f.path for f in observation.snapshot.files if f.category == "product"}
    nonempty = {f.path for f in observation.snapshot.files if f.category == "product" and f.size_bytes}
    payload = metric.value
    values = []
    if selector.kind == "function_ccn":
        if not isinstance(payload, baseline.ComplexityPayload):
            return (), "function_ccn requires a complexity function inventory"
        inventory = {f.path for f in payload.files}
        if not nonempty <= inventory or not inventory <= product:
            return (), "complexity file inventory differs from product population or omits nonempty source"
        values = [(f.ccn, baseline.SourceSpan(f.path, f.start_line, f.end_line), f.long_name)
                  for f in payload.functions]
    elif selector.kind == "scalar_metric":
        if type(payload) not in (int, float):
            return (), "declared scalar quantity is unavailable for a structured payload"
        if payload < 0:
            return (), "declared scalar quantity must be nonnegative"
        values = [(payload, None, None)]
    else:
        if not isinstance(payload, baseline.DuplicationPayload):
            return (), "duplication selector requires a duplication payload"
        scanned, zero = set(payload.scanned_paths), set(payload.zero_token_paths)
        if not nonempty <= (scanned | zero) or not (scanned | zero) <= product:
            return (), "duplication inventory differs from product population or omits nonempty source"
        if any(span.path not in scanned for clone in payload.clones for span in (clone.first, clone.second)):
            return (), "clone location absent from scanned population"
        quantity = {"duplication_clone_pairs": len(payload.clones),
                    "duplication_duplicated_tokens": payload.duplicated_tokens,
                    "duplication_total_tokens": payload.total_tokens,
                    "duplication_pair_span_ratio": payload.ratio}[selector.kind]
        if quantity is None:
            return (), payload.ratio_reason
        values = [(quantity, None, None)]
    entities = []
    for value, location, name in values:
        address = [observation.snapshot.content_hash, selector.to_dict(), metric.key,
                   metric.definition_id, metric.producer, metric.producer_version,
                   metric.scope_hash, location.to_dict() if location else None, name]
        identity = hashlib.sha256(json.dumps(address, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
        projected = replace(metric, value=value, unit=_quantity_unit(metric, selector))
        # report.json is always indexed, even for supplied optional metrics which
        # have no separate raw artifact. The report preserves the full payload.
        entities.append(EntityObservation(identity, observation.snapshot.content_hash, selector,
            projected, location, name, (metric.raw_artifact or "report.json",)))
    return tuple(sorted(entities, key=lambda e: e.inventory_id)), None


def _address_inventory(observation):
    """All usable evidence addresses, independent of any rule's scope or order.

    Missing metric coverage is expressed by RuleSelection, not by deleting other
    metrics' addresses. Completeness here concerns this address inventory.
    """
    entities = []
    for metric in observation.metrics:
        if isinstance(metric.value, baseline.ComplexityPayload):
            selectors = (Selector("function_ccn", None),)
        elif isinstance(metric.value, baseline.DuplicationPayload):
            selectors = tuple(Selector(kind, None) for kind in ("duplication_clone_pairs",
                "duplication_duplicated_tokens", "duplication_total_tokens", "duplication_pair_span_ratio"))
        elif type(metric.value) in (int, float):
            selectors = (Selector("scalar_metric", metric.key),)
        else:
            selectors = ()
        for selector in selectors:
            selected, reason = _metric_entities(observation, metric, selector)
            if reason is None:
                entities.extend(selected)
    return EvidenceInventory(observation.snapshot.content_hash, "complete", None,
                             tuple(sorted(entities, key=lambda e: e.inventory_id)))

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
    return EvidenceBundle(1, report, prepared.plan.profile_hash,
                          _address_inventory(report.baseline), _address_inventory(report.candidate))


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
