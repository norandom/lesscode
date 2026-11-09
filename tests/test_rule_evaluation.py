"""Numeric bounds retain entity debt, uncertainty and measurement meaning."""

from dataclasses import replace
import unittest

from lesscode_baseline.contracts import ContractError, MetricEvidence, SourceSpan
from lesscode_baseline.rules.contracts import (
    EntityObservation, EntityTreatment, EvidenceInventory,
    NumericRule, PolicyFinding, RuleSelection, Selector,
)
from lesscode_baseline.rules.evaluate import evaluate_numeric
from lesscode_baseline.rules.evaluate import evaluate_check, evaluate_review
from lesscode_baseline.rules import contracts as c
from tests import test_rule_contracts as fixtures
from tests.test_rule_contracts import numeric


def rule(mode="ratchet", budget=None, maximum=10, function=True):
    value = numeric("limit")
    value.update(mode=mode, absolute_maximum=maximum, change_budget=budget)
    if function:
        value["selector"] = {"kind": "function_ccn", "metric_key": None}
    return NumericRule.from_dict(value)


def entity(value, side="candidate", name="f", function=True):
    selector = Selector("function_ccn", None) if function else Selector("scalar_metric", "source_bytes")
    metric = MetricEvidence("complexity" if function else "source_bytes", "available", value,
        "bytes", "raw-source-bytes-v1", "lesscode", "0.1", "c" * 64, None, f"{side}/metric.json")
    return EntityObservation.from_dict(EntityObservation(f"{side}:{name}", ("a" if side == "baseline" else "b") * 64,
        selector, metric, SourceSpan(f"{name}.py", 1, 3) if function else None,
        name if function else None, (f"{side}/metric.json",)).to_dict())


def selection(before, after, *, status="comparable", reasons=()):
    return RuleSelection(1, "limit", None if before is None else EvidenceInventory("a" * 64, "complete", None, tuple(before)),
                         EvidenceInventory("b" * 64, "complete", None, tuple(after)), status, reasons)


def treatment(kind="matched", name="f"):
    return EntityTreatment("d" * 64, "a" * 64, "b" * 64, kind,
        None if kind == "new" else f"baseline:{name}",
        None if kind == "removed" else f"candidate:{name}", "reviewer", "Explicit continuity claim")


class NumericEvaluationTests(unittest.TestCase):
    def evaluate_pair(self, before, after, **options):
        findings = evaluate_numeric(rule(**options), selection([entity(before, "baseline")], [entity(after)]),
                                    (treatment(),))
        self.assertEqual(len(findings), 1)
        finding = findings[0]
        self.assertEqual(PolicyFinding.from_dict(finding.to_dict()), finding)
        self.assertEqual(finding.reference.metric.value, before)
        self.assertEqual(finding.candidate.metric.value, after)
        self.assertEqual(finding.locations, (SourceSpan("f.py", 1, 3),))
        return finding

    def test_ratchet_debt_table(self):
        for before, after, status, disposition in (
            (56, 56, "satisfied", "inherited_debt"), (56, 54, "satisfied", "improved_residual_debt"),
            (56, 57, "violation", "worsened_debt"), (4, 11, "violation", "new_debt"),
            (56, 10, "satisfied", "resolved_debt"), (4, 10, "satisfied", "worsened"),
            (4, 4, "satisfied", "unchanged"), (4, 3, "satisfied", "improved")):
            with self.subTest(before=before, after=after):
                finding = self.evaluate_pair(before, after)
                self.assertEqual((finding.status, finding.disposition), (status, disposition))

    def test_absolute_applies_to_inherited_debt(self):
        self.assertEqual(self.evaluate_pair(56, 56, mode="absolute").status, "violation")

    def test_absolute_equality_needs_no_reference(self):
        finding, = evaluate_numeric(rule(mode="absolute"), selection(None, [entity(10)], status="not_required"))
        self.assertEqual(finding.status, "satisfied")
        self.assertIsNone(finding.reference)
        self.assertIn("10", finding.reason)

    def test_budgets_are_additional_inclusive_bounds(self):
        for kind, maximum, before, after, expected in (
            ("absolute_increase", 1, 4, 6, "violation"), ("absolute_increase", 2, 4, 6, "satisfied"),
            ("percentage_increase", 50, 4, 6, "satisfied"), ("percentage_increase", 49, 4, 6, "violation"),
            ("absolute_increase", 99, 56, 57, "violation")):
            with self.subTest(kind=kind, maximum=maximum):
                finding = self.evaluate_pair(before, after, budget={"kind": kind, "maximum": maximum})
                self.assertEqual(finding.status, expected)
                self.assertIn(kind, finding.reason)

    def test_zero_reference_percentage_is_unavailable(self):
        finding = self.evaluate_pair(0, 0, budget={"kind": "percentage_increase", "maximum": 5})
        self.assertEqual(finding.status, "insufficient")
        self.assertIn("zero", finding.reason)

    def test_decimal_budget_boundaries_and_meaningful_exceedances(self):
        for kind, maximum, candidate, expected in (
            ("absolute_increase", 0.6, 0.9, "satisfied"),
            ("absolute_increase", 0.6, 0.9000000000000001, "violation"),
            ("percentage_increase", 200, 0.9, "satisfied"),
            ("percentage_increase", 200, 0.9000000000000001, "violation"),
        ):
            with self.subTest(kind=kind, candidate=candidate):
                finding = self.evaluate_pair(0.3, candidate,
                    budget={"kind": kind, "maximum": maximum})
                self.assertEqual(finding.status, expected)

    def test_large_finite_percentage_does_not_overflow(self):
        for candidate, expected in ((1.1e308, "satisfied"), (1.12e308, "violation")):
            with self.subTest(candidate=candidate):
                finding = self.evaluate_pair(1e308, candidate, maximum=1.7e308,
                    budget={"kind": "percentage_increase", "maximum": 11})
                self.assertEqual(finding.status, expected)

    def test_new_entities_have_no_inherited_allowance(self):
        for value, expected in ((10, "satisfied"), (56, "violation")):
            finding, = evaluate_numeric(rule(), selection([], [entity(value)]), (treatment("new"),))
            self.assertEqual((finding.status, finding.disposition), (expected, "new"))
            self.assertIn("claim", finding.reason)

    def test_new_delta_only_has_no_reference(self):
        finding, = evaluate_numeric(rule(mode="absolute", maximum=None,
            budget={"kind": "absolute_increase", "maximum": 5}), selection([], [entity(4)]), (treatment("new"),))
        self.assertEqual(finding.status, "insufficient")
        self.assertIn("reference", finding.reason)

    def test_uncertain_matches_do_not_grant_inherited_pass(self):
        for treatments in ((), (treatment("unresolved"),)):
            findings = evaluate_numeric(rule(), selection([entity(56, "baseline")], [entity(56)]), treatments)
            candidate = next(f for f in findings if f.candidate is not None)
            self.assertEqual((candidate.status, candidate.disposition), ("insufficient", "unresolved"))
            self.assertIn("potential", candidate.reason)

    def test_removed_entities_disclaim_behavior_proof(self):
        finding, = evaluate_numeric(rule(), selection([entity(56, "baseline")], []), (treatment("removed"),))
        self.assertEqual((finding.status, finding.disposition), ("satisfied", "removed"))
        self.assertIn("preservation", finding.reason)

    def test_entities_do_not_offset_each_other(self):
        findings = evaluate_numeric(rule(), selection([entity(56, "baseline", "a"), entity(4, "baseline", "b")],
            [entity(10, name="a"), entity(11, name="b")]), (treatment(name="a"), treatment(name="b")))
        self.assertEqual([(f.status, f.disposition) for f in findings],
                         [("satisfied", "resolved_debt"), ("violation", "new_debt")])

    def test_comparable_aggregate_uses_scalar_reference(self):
        finding, = evaluate_numeric(rule(function=False), selection([entity(56, "baseline", function=False)],
                                                                  [entity(54, function=False)]))
        self.assertEqual(finding.disposition, "improved_residual_debt")

    def test_unavailable_comparison_retains_candidate_and_reason(self):
        finding, = evaluate_numeric(rule(), selection([entity(56, "baseline")], [entity(54)],
            status="unavailable", reasons=("producer version differs",)), (treatment(),))
        self.assertEqual(finding.status, "insufficient")
        self.assertEqual(finding.candidate.metric.value, 54)
        self.assertIn("producer version differs", finding.reason)

    def test_advisory_unavailable_is_disclosed(self):
        selected = selection(None, [], status="not_required")
        selected = replace(selected, candidate=EvidenceInventory("b" * 64, "unavailable", "missing CCN", ()))
        finding, = evaluate_numeric(replace(rule(mode="absolute"), enforcement="advisory"), selected)
        self.assertEqual((finding.status, finding.enforcement), ("insufficient", "advisory"))
        self.assertIn("missing CCN", finding.reason)

    def test_confirmed_empty_functions_are_disclosed(self):
        finding, = evaluate_numeric(rule(mode="absolute"), selection(None, [], status="not_required"))
        self.assertEqual(finding.status, "satisfied")
        self.assertIn("no applicable functions", finding.reason)

    def test_missing_baseline_retains_available_candidate(self):
        selected = selection([], [entity(54)], status="unavailable", reasons=("baseline missing",))
        selected = replace(selected, baseline=EvidenceInventory("a" * 64, "unavailable", "missing metric", ()))
        finding, = evaluate_numeric(rule(), selected)
        self.assertEqual((finding.status, finding.candidate.metric.value), ("insufficient", 54))

    def test_absolute_violation_survives_unavailable_budget(self):
        selected = selection([], [entity(11)], status="unavailable", reasons=("baseline missing",))
        finding, = evaluate_numeric(rule(mode="absolute", budget={"kind": "absolute_increase", "maximum": 5}), selected)
        self.assertEqual(finding.status, "violation")
        self.assertIn("baseline missing", finding.reason)

    def test_selection_must_identify_rule(self):
        with self.assertRaises(ContractError):
            evaluate_numeric(rule(), replace(selection([], []), rule_id="other"))

    def test_undefined_scalar_does_not_become_zero(self):
        candidate = entity(4, function=False)
        candidate = replace(candidate, metric=replace(candidate.metric, status="unavailable", value=None,
            reason="pair-span denominator undefined"))
        finding, = evaluate_numeric(rule(mode="absolute", function=False),
            selection(None, [candidate], status="not_required"))
        self.assertEqual((finding.status, finding.disposition), ("insufficient", "unavailable"))
        self.assertIsNone(finding.candidate.metric.value)
        self.assertIn("denominator undefined", finding.reason)

    def test_all_unavailable_selection_reasons_remain_visible(self):
        selected = replace(selection([], [], status="unavailable", reasons=("incompatible scope",)),
            candidate=EvidenceInventory("b" * 64, "unavailable", "missing CCN", ()))
        finding, = evaluate_numeric(rule(), selected)
        self.assertIn("missing CCN", finding.reason)
        self.assertIn("incompatible scope", finding.reason)

    def test_observation_artifact_addresses_and_claims_are_retained(self):
        finding = self.evaluate_pair(56, 54)
        self.assertEqual(finding.reference.evidence_refs, ("baseline/metric.json",))
        self.assertEqual(finding.candidate.evidence_refs, ("candidate/metric.json",))
        self.assertIn("no semantic proof", finding.reason)
        self.assertEqual(finding.exceptions, ())

    def test_order_and_fixed_inputs_replay_identically(self):
        selected = selection([entity(56, "baseline", "a"), entity(4, "baseline", "b")],
            [entity(10, name="a"), entity(11, name="b")])
        treatments = (treatment(name="a"), treatment(name="b"))
        expected = evaluate_numeric(rule(), selected, treatments)
        reordered = replace(selected, candidate=replace(selected.candidate,
            entities=tuple(reversed(selected.candidate.entities))))
        self.assertEqual(expected, evaluate_numeric(rule(), reordered, tuple(reversed(treatments))))

    def test_no_treatment_does_not_assert_removal(self):
        finding, = evaluate_numeric(rule(), selection([entity(56, "baseline")], []))
        self.assertEqual((finding.status, finding.disposition), ("insufficient", "unresolved"))

    def test_established_new_violation_survives_missing_budget_reference(self):
        finding, = evaluate_numeric(rule(budget={"kind": "absolute_increase", "maximum": 5}),
            selection([], [entity(56)]), (treatment("new"),))
        self.assertEqual(finding.status, "violation")
        self.assertIn("unavailable", finding.reason)


class CheckReviewEvaluationTests(unittest.TestCase):
    def setUp(self):
        self.evidence = c.EvidenceBundle.from_dict(fixtures.bundle())
        self.check_rule = c.CheckRule.from_dict(fixtures.check())
        self.policy = c.Policy.from_dict(fixtures.review_policy())
        self.review_rule = self.policy.rules[0]
        self.record = c.ReviewRecord.from_dict(fixtures.review_record())

    def checks(self, checks):
        return replace(self.evidence, report=replace(self.evidence.report,
            candidate=replace(self.evidence.report.candidate, checks=checks)))

    def check(self, evidence=None, rule=None):
        finding, = evaluate_check(rule or self.check_rule, evidence or self.evidence)
        self.assertEqual(c.PolicyFinding.from_dict(finding.to_dict()), finding)
        return finding

    def review(self, records=None, rule=None, policy=None):
        finding, = evaluate_review(rule or self.review_rule, self.evidence,
            policy or self.policy, (self.record,) if records is None else records)
        self.assertEqual(c.PolicyFinding.from_dict(finding.to_dict()), finding)
        return finding

    def test_exact_check_statuses_and_diagnostics(self):
        original, = self.evidence.report.candidate.checks
        for observed, expected in (("passed", "satisfied"), ("failed", "violation"),
                ("skipped", "insufficient"), ("missing", "insufficient"), ("error", "insufficient")):
            with self.subTest(observed=observed):
                finding = self.check(self.checks((replace(original, status=observed),)))
                self.assertEqual((finding.status, finding.disposition), (expected, "check"))
                self.assertIn(observed, finding.reason)
                self.assertIn(original.check_id, finding.reason)
                self.assertIn(original.diagnostic_artifact, finding.reason)

    def test_absent_check_cannot_use_baseline_or_similar_id(self):
        original, = self.evidence.report.candidate.checks
        for checks in ((), (replace(original, check_id=original.check_id + ".extra"),)):
            self.assertEqual(self.check(self.checks(checks)).status, "insufficient")

    def test_check_source_and_plan_tampering_fail_closed(self):
        original, = self.evidence.report.candidate.checks
        for field in ("source_hash", "plan_hash"):
            with self.subTest(field=field), self.assertRaises(ContractError):
                self.check(self.checks((replace(original, **{field: "e" * 64}),)))

    def test_duplicate_check_is_invalid_input(self):
        original, = self.evidence.report.candidate.checks
        with self.assertRaises(ContractError):
            self.check(self.checks((original, original)))

    def test_selection_preserves_other_failed_checks_and_upstream_rejection(self):
        original, = self.evidence.report.candidate.checks
        rejected = self.checks((original, replace(original, check_id="test.behavior", status="failed")))
        rejected = replace(rejected, report=replace(rejected.report, decision="rejected",
            reasons=("candidate test.behavior: failed",)))
        self.assertEqual(self.check(rejected).status, "satisfied")
        self.assertEqual(rejected.report.decision, "rejected")
        self.assertEqual(rejected.report.reasons, ("candidate test.behavior: failed",))
        self.assertEqual(rejected.report.candidate.checks[1].status, "failed")

    def test_bound_review_outcomes_and_provenance_are_distinct(self):
        for outcome, expected in (("satisfied", "satisfied"), ("concern", "violation"),
                                  ("unresolved", "insufficient")):
            with self.subTest(outcome=outcome):
                finding = self.review((replace(self.record, outcome=outcome),))
                self.assertEqual((finding.status, finding.disposition), (expected, "review"))
                self.assertEqual(finding.locations, self.record.locations)
                for text in (outcome, self.record.reviewer_id, self.record.rationale,
                             "producer_supplied", "not authenticated"):
                    self.assertIn(text, finding.reason)
                self.assertEqual(finding.expected, self.review_rule)

    def test_lower_ccn_cannot_supply_missing_semantic_review(self):
        numeric_finding, = evaluate_numeric(rule(), selection([entity(56, "baseline")], [entity(54)]), (treatment(),))
        self.assertEqual(numeric_finding.disposition, "improved_residual_debt")
        self.assertEqual(numeric_finding.status, "satisfied")
        required = replace(self.review_rule, enforcement="blocking")
        finding = self.review((), rule=required, policy=replace(self.policy, rules=(required,)))
        self.assertEqual((finding.status, finding.enforcement), ("insufficient", "blocking"))

    def test_well_formed_review_mismatches_are_insufficient(self):
        changes = ({"policy_hash": "e" * 64}, {"candidate_source_hash": "e" * 64},
            {"rule_id": "other"}, {"criterion_id": "other"},
            {"scope": replace(self.record.scope, exclude=("other/**",))})
        for change in changes:
            with self.subTest(change=change):
                finding = self.review((replace(self.record, **change),))
                self.assertEqual(finding.status, "insufficient")
                self.assertIn("mismatch", finding.reason)

    def test_duplicate_or_conflicting_reviews_are_invalid_even_with_wrong_source(self):
        for record in (self.record, replace(self.record, outcome="concern"),
                       replace(self.record, candidate_source_hash="e" * 64)):
            finding = self.review((self.record, record))
            self.assertEqual(finding.status, "invalid")
            self.assertIn("duplicate", finding.reason.lower())

    def test_external_independence_is_unverified_for_local_claims(self):
        external = replace(self.review_rule, trust_requirement="external_independence")
        policy = replace(self.policy, rules=(external,))
        record = replace(self.record, policy_hash=c.policy_hash(policy), reviewer_id="independent-auditor")
        finding = self.review((record,), rule=external, policy=policy)
        self.assertEqual(finding.status, "insufficient")
        self.assertIn("unverified", finding.reason)

    def test_review_concern_is_retained_when_independence_unverified(self):
        external = replace(self.review_rule, trust_requirement="external_independence")
        policy = replace(self.policy, rules=(external,))
        record = replace(self.record, policy_hash=c.policy_hash(policy), outcome="concern")
        finding = self.review((record,), rule=external, policy=policy)
        self.assertEqual(finding.status, "violation")
        self.assertIn("unverified", finding.reason)

    def test_outside_inventory_or_scope_review_span_is_invalid(self):
        for path in ("missing.py", "vendor/hidden.py"):
            finding = self.review((replace(self.record, locations=(SourceSpan(path, 1, 2),)),))
            self.assertEqual(finding.status, "invalid")
            self.assertIn("span", finding.reason)

    def test_review_cannot_be_evaluated_under_different_policy_rule(self):
        with self.assertRaises(ContractError):
            self.review(rule=replace(self.review_rule, criterion_text="Different criterion"))

    def test_unrelated_review_does_not_satisfy_required_criterion(self):
        unrelated = replace(self.record, rule_id="other", criterion_id="other")
        self.assertEqual(self.review((unrelated,)).status, "insufficient")

    def test_advisory_unavailable_check_and_review_remain_disclosed(self):
        self.assertEqual((self.check(self.checks(()), replace(self.check_rule, enforcement="advisory")).status,
                          self.review(()).enforcement), ("insufficient", "advisory"))

    def test_review_replay_is_independent_of_record_order(self):
        conflict = replace(self.record, outcome="concern")
        self.assertEqual(self.review((self.record, conflict)), self.review((conflict, self.record)))

    def test_duplicate_unmatched_claims_are_still_invalid_review_input(self):
        unrelated = replace(self.record, rule_id="other", criterion_id="other")
        self.assertEqual(self.review((self.record, unrelated, unrelated)).status, "invalid")

    def test_spans_outside_policy_target_are_invalid_even_within_rule(self):
        policy = replace(self.policy, target_scope=replace(self.policy.target_scope,
            include=("different/**",)))
        record = replace(self.record, policy_hash=c.policy_hash(policy))
        self.assertEqual(self.review((record,), policy=policy).status, "invalid")

    def test_malformed_provenance_and_spans_fail_closed(self):
        for record in (replace(self.record, provenance="independent"),
                       replace(self.record, reviewer_id=""),
                       replace(self.record, locations=(SourceSpan("src/main.py", 0, 1),))):
            with self.subTest(record=record), self.assertRaises(ContractError):
                self.review((record,))


if __name__ == "__main__":
    unittest.main()
