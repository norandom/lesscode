"""Pure policy interpretation of selected, validated evidence."""

from decimal import Decimal, localcontext

from lesscode_baseline.contracts import ContractError
from .contracts import EntityTreatment, NumericRule, PolicyFinding, RuleSelection

def evaluate_numeric(rule: NumericRule, selection: RuleSelection,
                     treatments: tuple[EntityTreatment, ...] = ()) -> tuple[PolicyFinding, ...]:
    """Evaluate one rule; continuity records must come from establish_continuity.

    Selection establishes measurement compatibility, not entity continuity.
    Aggregate scalars compare their selected populations directly; function
    references require an exact, established treatment. No IO or new analysis
    occurs here. Observation artifact addresses remain in each original side;
    indexed EvidenceReferences are attached by the assessment coordinator.
    """
    rule = NumericRule.from_dict(rule.to_dict())
    selection = RuleSelection.from_dict(selection.to_dict())
    if selection.rule_id != rule.rule_id:
        raise ContractError("$selection.rule_id: differs from numeric rule")
    if selection.candidate.status != "complete":
        return (_finding(rule, None, None, "insufficient", "unavailable",
                         "; ".join(dict.fromkeys((selection.candidate.reason, *selection.comparison_reasons)))),)

    needs_reference = rule.mode == "ratchet" or rule.change_budget is not None
    reasons = list(selection.comparison_reasons) if needs_reference else []
    if needs_reference and selection.comparison_status == "not_required":
        reasons.append("Required reference comparison was not established")
    before = {e.inventory_id: e for e in selection.baseline.entities} if selection.baseline else {}
    by_candidate = {t.candidate_id: t for t in treatments if t.candidate_id is not None}
    by_baseline = {t.baseline_id: t for t in treatments if t.baseline_id is not None}
    findings, consumed = [], set()
    for candidate in sorted(selection.candidate.entities, key=lambda e: e.inventory_id):
        treatment = by_candidate.get(candidate.inventory_id)
        reference = None
        kind = "unresolved" if needs_reference else "new"
        claim = ""
        if candidate.selector.kind != "function_ccn" and needs_reference and len(before) == 1:
            reference = next(iter(before.values()))
            kind = "matched"
        elif treatment is not None:
            kind = treatment.kind
            reference = before.get(treatment.baseline_id)
            claim = f" Treatment is a producer claim by {treatment.producer_id}: {treatment.rationale}; no semantic proof."
        if reference is not None:
            consumed.add(reference.inventory_id)
        findings.append(_evaluate_entity(rule, reference, candidate, kind, reasons, claim))

    if needs_reference:
        for identity in sorted(before.keys() - consumed):
            treatment = by_baseline.get(identity)
            if treatment is not None and treatment.candidate_id is not None:
                # A paired treatment outside this selection cannot silently
                # establish a removal from the selected population.
                reason = "Reference treatment points outside the selected candidate population"
                status, disposition = "insufficient", "unresolved"
            elif treatment is not None and treatment.kind == "removed":
                reason = (f"Removed by producer claim {treatment.producer_id}: {treatment.rationale}. "
                          "Removal is not proof of behavior or API preservation.")
                status, disposition = "satisfied", "removed"
            else:
                reason = "Reference treatment unresolved; absence is not established removal or preservation proof"
                status, disposition = "insufficient", "unresolved"
            if reasons:
                status = "insufficient"
                reason += "; " + "; ".join(reasons)
            findings.append(_finding(rule, before[identity], None, status, disposition, reason))
    if not findings:
        reason = "Confirmed complete population: no applicable functions"
        findings.append(_finding(rule, None, None, "insufficient" if reasons else "satisfied",
                                 "unavailable" if reasons else "unchanged",
                                 reason + ("; " + "; ".join(reasons) if reasons else "")))
    return tuple(sorted(findings, key=lambda f: (f.entity_id, f.basis)))


def _debt_disposition(reference, candidate, maximum):
    if maximum is not None and reference > maximum:
        if candidate <= maximum:
            return "resolved_debt"
        if candidate == reference:
            return "inherited_debt"
        return "improved_residual_debt" if candidate < reference else "worsened_debt"
    if maximum is not None and candidate > maximum:
        return "new_debt"
    if candidate == reference:
        return "unchanged"
    return "improved" if candidate < reference else "worsened"


def _evaluate_entity(rule, reference, candidate, kind, comparison_reasons, claim):
    value = candidate.metric.value
    baseline_value = reference.metric.value if reference is not None else None
    reasons = [f"candidate={value}; reference={baseline_value}; inclusive maximum={rule.absolute_maximum}"]
    unavailable = list(comparison_reasons)
    violation = False
    disposition = kind
    if candidate.metric.status != "available" or value is None:
        return _finding(rule, reference, candidate, "insufficient", "unavailable",
                        candidate.metric.reason + claim)
    if reference is not None and reference.metric.status != "available":
        unavailable.append(reference.metric.reason)
        baseline_value = None
    if kind == "matched" and baseline_value is not None:
        disposition = _debt_disposition(baseline_value, value, rule.absolute_maximum)
    elif kind != "new":
        disposition = "unresolved"

    maximum = rule.absolute_maximum
    if maximum is not None and value > maximum:
        if rule.mode == "absolute" or kind == "new":
            violation = True
        elif kind == "matched" and baseline_value is not None and not comparison_reasons:
            violation = value > baseline_value
        else:
            unavailable.append("potential limit violation; reference treatment is unresolved or comparison unavailable")
    if rule.mode == "ratchet" and kind == "unresolved" and maximum is not None and value <= maximum:
        reasons.append("Absolute limit satisfied without an inherited allowance; continuity remains unresolved")
    budget = rule.change_budget
    if budget is not None:
        if kind != "matched" or baseline_value is None:
            unavailable.append(f"{budget.kind} unavailable: no established entity reference value")
        elif comparison_reasons:
            unavailable.append(f"{budget.kind} unavailable: reference comparison unavailable")
        elif budget.kind == "percentage_increase" and baseline_value == 0:
            unavailable.append("percentage_increase unavailable: reference value is zero")
        else:
            increase, exceeds_budget = _compare_budget(baseline_value, value, budget)
            reasons.append(f"{budget.kind}={increase}; inclusive maximum={budget.maximum}")
            violation = violation or exceeds_budget
    status = "violation" if violation else "insufficient" if unavailable else "satisfied"
    reasons.extend(unavailable)
    return _finding(rule, reference, candidate, status, disposition, "; ".join(reasons) + claim)


def _compare_budget(reference, candidate, budget):
    """Compare declared decimal values exactly, without binary rounding slack."""
    baseline, observed, maximum = (Decimal(str(value)) for value in
                                   (reference, candidate, budget.maximum))
    with localcontext() as context:
        # Cover every input's decimal places and coefficient digits, plus
        # multiplication by 100. Subtraction and cross-products are exact;
        # only the displayed repeating percentage may require rounding.
        context.prec = 4 + sum(len(value.as_tuple().digits) + abs(value.as_tuple().exponent)
                               for value in (baseline, observed, maximum))
        delta = observed - baseline
        if budget.kind == "absolute_increase":
            return delta, delta > maximum
        numerator = 100 * delta
        return numerator / baseline, numerator > maximum * baseline


def _finding(rule, reference, candidate, status, disposition, reason):
    observations = tuple(e for e in (reference, candidate) if e is not None)
    locations = tuple(dict.fromkeys(e.location for e in observations if e.location is not None))
    identity = candidate.inventory_id if candidate else reference.inventory_id if reference else "selection:" + rule.rule_id
    return PolicyFinding(rule.rule_id, rule.scope, identity, "numeric", rule.enforcement,
                         status, disposition, reference, candidate, rule, locations, (), reason, ())
