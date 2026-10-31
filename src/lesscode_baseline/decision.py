"""Metric compatibility and numerical comparison, independent of acceptance.

Structured measurements retain their producer-defined shape. Their compatibility
is distinct from availability of a scalar delta; no aggregate score is inferred.
"""

from dataclasses import dataclass
import math
from typing import Literal

from .contracts import MetricDelta, MetricEvidence, PopulationPayload, Threshold


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
    elif threshold.operator == "absolute_maximum":
        observed = delta.after
        reason = delta.reason if observed is None else None
    elif threshold.operator == "delta_maximum":
        observed = delta.absolute_delta
        reason = delta.reason if observed is None else None
    else:
        observed = delta.percentage_delta
        reason = delta.percentage_reason
    if observed is None:
        return ThresholdResult(threshold, "unavailable", None, reason or "threshold measurement unavailable")
    if observed > threshold.value:
        return ThresholdResult(threshold, "failed", observed,
                               f"{threshold.operator}: {observed} exceeds {threshold.value} {threshold.unit}")
    return ThresholdResult(threshold, "passed", observed, None)
