"""Behavior-first acceptance and metric compatibility under a frozen profile.

Structured measurements retain their producer-defined shape. Their compatibility
is distinct from availability of a scalar delta; no aggregate score is inferred.
"""

from dataclasses import dataclass
import math
from typing import Literal

from .contracts import (Comparison, FrozenProfile, IntegrityResult, MetricDelta,
                        MetricEvidence, Observation, PopulationPayload, Threshold)


def compare(baseline: Observation, candidate: Observation, profile: FrozenProfile,
            integrity: IntegrityResult) -> Comparison:
    """Apply the frozen profile without allowing metrics to compensate for behavior.

    Completed normalized check evidence is authoritative when raw executions are
    not supplied. When executions are supplied, unsuccessful command outcomes
    cannot be overridden by a passed report. Integrity verification itself is the
    caller's responsibility; this function performs no execution or filesystem IO.
    """
    problems = {state: [] for state in
                ("invalid", "baseline_ineligible", "rejected", "incomplete")}
    disclosures = []
    if integrity.status != "unchanged" or integrity.changed_resources:
        problems["invalid"].append(
            f"integrity {integrity.status}; changed resources: {integrity.changed_resources!r}")

    declared = {}
    commands = {}
    for spec in profile.checks:
        commands[spec.command.command_id] = spec
        for identity in spec.expected_ids:
            if identity in declared:
                raise ValueError(f"duplicate declared check identity: {identity}")
            declared[identity] = spec

    for side, observation in (("baseline", baseline), ("candidate", candidate)):
        checks = {}
        for evidence in observation.checks:
            if evidence.check_id in checks:
                problems["invalid"].append(f"{side} duplicate check {evidence.check_id}")
            checks[evidence.check_id] = evidence
            spec = declared.get(evidence.check_id)
            if spec is None:
                problems["invalid"].append(f"{side} undeclared check {evidence.check_id}")
            elif (evidence.kind, evidence.required) != (spec.kind, spec.required):
                problems["invalid"].append(
                    f"{side} check {evidence.check_id} kind/required differs from frozen profile")
            if evidence.source_hash != observation.snapshot.content_hash:
                problems["invalid"].append(f"{side} check {evidence.check_id} source identity mismatch")
        for identity, spec in declared.items():
            evidence = checks.get(identity)
            status = evidence.status if evidence else "missing"
            if status == "passed":
                continue
            reason = f"{side} check {identity}: {status}"
            if side == "baseline":
                problems["baseline_ineligible"].append(reason)
            elif status == "failed":
                problems["rejected"].append(reason)
            elif spec.required:
                problems["incomplete"].append(reason)
            else:
                disclosures.append(reason + " (optional)")
        for execution in observation.executions:
            spec = commands.get(execution.command.command_id)
            if spec is None or (execution.status == "completed" and execution.exit_code == 0):
                continue
            reason = (f"{side} check command {spec.check_id}: {execution.status}; "
                      f"exit {execution.exit_code}")
            if side == "baseline":
                problems["baseline_ineligible"].append(reason)
            elif spec.required:
                problems["incomplete"].append(reason)
            else:
                disclosures.append(reason + " (optional)")
        for failure in observation.failures:
            reason = f"{side} {failure.code}: {failure.message}"
            disclosures.append(reason)
            # Adapter failures for optional measurements are limitations, whereas
            # a declared behavior command failure still invalidates its evidence.
            spec = declared.get(failure.check_id) or next(
                (item for item in profile.checks if item.check_id == failure.check_id), None)
            if spec:
                if side == "baseline":
                    problems["baseline_ineligible"].append(reason)
                elif spec.required:
                    problems["incomplete"].append(reason)

    baseline_plan_ids = {item.plan_hash for item in baseline.checks}
    candidate_plan_ids = {item.plan_hash for item in candidate.checks}
    if len(baseline_plan_ids | candidate_plan_ids) > 1:
        problems["invalid"].append("check plan identities differ across observations")

    metrics = compare_metrics(baseline.metrics, candidate.metrics)
    by_key = {item.delta.key: item for item in metrics}
    required = set(profile.required_metrics)
    for key in profile.required_metrics:
        comparison = by_key.get(key)
        if comparison is None or comparison.status != "available":
            detail = comparison.delta.reason if comparison else "missing on both sides"
            problems["incomplete"].append(f"required metric {key}: {detail}")
    for comparison in metrics:
        if comparison.delta.key not in required and comparison.status != "available":
            disclosures.append(f"optional metric {comparison.delta.key}: {comparison.delta.reason}")
    for threshold in profile.thresholds:
        comparison = by_key.get(threshold.metric_key)
        if comparison is None:
            problems["incomplete"].append(f"threshold metric {threshold.metric_key}: missing")
            continue
        result = evaluate_threshold(comparison, threshold)
        if result.status != "passed":
            state = "rejected" if result.status == "failed" else "incomplete"
            problems[state].append(f"threshold {threshold.metric_key}: {result.reason}")

    decision = next((state for state, reasons in problems.items() if reasons), "accepted")
    reasons = [reason for group in problems.values() for reason in group]
    if decision == "accepted":
        reasons.append("Candidate accepted under the frozen profile; required declared checks passed")
    reasons.extend(disclosures)
    reasons.append("Results cover only declared checks; unchecked behavior remains; not proof of equivalence")
    protection = integrity.protection
    reasons.append(f"Protection: {protection.mode}; provider evidence: {protection.provider_evidence!r}")
    return Comparison(decision, tuple(reasons), tuple(item.delta for item in metrics))


@dataclass(frozen=True)
class MetricComparison:
    before: MetricEvidence | None
    after: MetricEvidence | None
    status: Literal["available", "unavailable", "non_comparable"]
    differences: tuple[str, ...]
    delta: MetricDelta


@dataclass(frozen=True)
class ThresholdResult:
    threshold: Threshold
    status: Literal["passed", "failed", "unavailable"]
    observed: float | None
    reason: str | None


def _finite_number(value) -> bool:
    if type(value) not in (int, float):
        return False
    try:
        return math.isfinite(value)
    except OverflowError:
        return False


def _scalar(evidence: MetricEvidence | None):
    if evidence is not None and evidence.status == "available" and _finite_number(evidence.value):
        return evidence.value
    return None


def compare_metric(before: MetricEvidence | None, after: MetricEvidence | None) -> MetricComparison:
    """Check identity and population before calculating after-minus-before.

    ``status`` describes evidence compatibility; ``delta.status`` describes scalar
    arithmetic. Changed populations require the candidate's recorded explanation
    of the change. The original records preserve denominators and explanations.
    """
    if before is None and after is None:
        raise ValueError("at least one metric record is required")
    if before is not None and after is not None and before.key != after.key:
        raise ValueError("metric keys must match; keep distinct analyzers separately identified")
    exemplar = before if before is not None else after
    differences = []
    blockers = []
    unavailable = []
    for side, evidence in (("baseline", before), ("candidate", after)):
        if evidence is None:
            unavailable.append(f"{side} metric missing")
        elif evidence.status != "available":
            message = f"{side} metric {evidence.status}: {evidence.reason}"
            (blockers if evidence.status == "non_comparable" else unavailable).append(message)
        elif evidence.value is None or type(evidence.value) is bool:
            unavailable.append(f"{side} metric has no valid measurement")
        elif type(evidence.value) in (int, float) and not _finite_number(evidence.value):
            unavailable.append(f"{side} metric must be finite")
    if before is not None and after is not None:
        for field in ("scope_hash", "producer", "producer_version", "definition_id", "unit"):
            old, new = getattr(before, field), getattr(after, field)
            if old != new:
                difference = f"{field}: {old!r} -> {new!r}"
                differences.append(difference)
                blockers.append(difference)
        if before.status == after.status == "available":
            old, new = before.value, after.value
            old_scalar, new_scalar = type(old) in (int, float), type(new) in (int, float)
            if old_scalar != new_scalar or (not old_scalar and type(old) is not type(new)):
                difference = f"payload type: {type(old).__name__} -> {type(new).__name__}"
                differences.append(difference)
                blockers.append(difference)
            if isinstance(old, PopulationPayload) and isinstance(new, PopulationPayload):
                population_changes = []
                for field in ("population_ids", "population_hash", "denominator"):
                    previous, current = getattr(old, field), getattr(new, field)
                    changed = set(previous) != set(current) if field == "population_ids" else previous != current
                    if changed:
                        population_changes.append(f"{field}: {previous!r} -> {current!r}")
                differences.extend(population_changes)
                if population_changes and not (new.explanation and new.explanation.strip()):
                    blockers.extend(population_changes)
                    blockers.append("candidate population-change explanation missing")
    status = "non_comparable" if blockers else "unavailable" if unavailable else "available"
    numeric_before, numeric_after = _scalar(before), _scalar(after)
    reason = "; ".join(blockers + unavailable) or None
    absolute = percentage = None
    percentage_reason = reason
    delta_status = status
    if status == "available":
        if numeric_before is None or numeric_after is None:
            delta_status = "unavailable"
            reason = "structured measurement has no declared scalar projection"
            percentage_reason = reason
        else:
            absolute = numeric_after - numeric_before
            if not _finite_number(absolute):
                absolute = None
                delta_status = "unavailable"
                reason = "absolute delta is not finite"
                percentage_reason = reason
            elif numeric_before == 0:
                percentage_reason = "baseline value is zero"
            else:
                # Use the declared formula order unless scaling would overflow.
                scaled = 100 * absolute
                percentage = (scaled / numeric_before if _finite_number(scaled)
                              else (absolute / numeric_before) * 100)
                if not _finite_number(percentage):
                    percentage = None
                    percentage_reason = "percentage delta is not finite"
    delta = MetricDelta(exemplar.key, delta_status, numeric_before, numeric_after,
                        absolute, percentage, reason, percentage_reason, exemplar.unit)
    return MetricComparison(before, after, status, tuple(differences), delta)


def compare_metrics(before: tuple[MetricEvidence, ...], after: tuple[MetricEvidence, ...]) -> tuple[MetricComparison, ...]:
    """Pair by declared metric key, retaining one-sided and distinct measurements."""
    def index(records):
        result = {}
        for evidence in records:
            if evidence.key in result:
                raise ValueError(f"duplicate metric key: {evidence.key}")
            result[evidence.key] = evidence
        return result

    baseline, candidate = index(before), index(after)
    keys = dict.fromkeys((*baseline, *candidate))
    return tuple(compare_metric(baseline.get(key), candidate.get(key)) for key in keys)


def evaluate_threshold(comparison: MetricComparison, threshold: Threshold) -> ThresholdResult:
    """Apply an explicit maximum inclusively, with no unit conversion or default."""
    # Validate rule shape even when constructed directly rather than read from JSON.
    threshold = Threshold.from_dict(threshold.to_dict())
    delta = comparison.delta
    reason = None
    observed = None
    expected_unit = "percent" if threshold.operator == "percentage_delta_maximum" else delta.unit
    if threshold.metric_key != delta.key:
        reason = f"threshold key {threshold.metric_key!r} differs from metric {delta.key!r}"
    elif threshold.unit != expected_unit:
        reason = f"threshold unit {threshold.unit!r} differs from expected {expected_unit!r}"
    elif comparison.status != "available":
        reason = delta.reason or "metric comparison unavailable"
    else:
        observed, reason = {
            "absolute_maximum": (delta.after, delta.reason),
            "delta_maximum": (delta.absolute_delta, delta.reason),
            "percentage_delta_maximum": (delta.percentage_delta, delta.percentage_reason),
        }[threshold.operator]
    if observed is None:
        return ThresholdResult(threshold, "unavailable", None, reason or "threshold measurement unavailable")
    if observed > threshold.value:
        return ThresholdResult(threshold, "failed", observed,
                               f"{threshold.operator}: {observed} exceeds {threshold.value} {threshold.unit}")
    return ThresholdResult(threshold, "passed", observed, None)
