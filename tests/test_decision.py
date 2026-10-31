"""Independent arithmetic and provenance cases for metric comparison."""

from dataclasses import replace
import unittest

from lesscode_baseline.contracts import (
    ComplexityPayload, DuplicationPayload, FindingsPayload, MetricDelta, MetricEvidence,
    PopulationPayload, Threshold,
)
from lesscode_baseline.decision import compare_metric, compare_metrics, evaluate_threshold


def metric(value=100, **changes):
    return replace(MetricEvidence("size", "available", value, "bytes", "size-v1",
                                  "counter", "1", "a" * 64, None, None), **changes)


class MetricComparisonTests(unittest.TestCase):
    def test_signed_deltas_and_zero_baseline(self):
        for before, after, delta, percentage in [(100, 75, -25, -25), (40, 50, 10, 25),
                                                  (-20, -10, 10, -50), (0, 5, 5, None)]:
            with self.subTest(before=before):
                result = compare_metric(metric(before), metric(after))
                self.assertEqual(result.status, "available")
                self.assertEqual((result.delta.before, result.delta.after), (before, after))
                self.assertEqual(result.delta.absolute_delta, delta)
                self.assertEqual(result.delta.percentage_delta, percentage)
                self.assertEqual(result.delta.percentage_reason is not None, before == 0)
                self.assertEqual(MetricDelta.from_dict(result.delta.to_dict()), result.delta)

    def test_every_metadata_difference_prevents_delta_and_identifies_values(self):
        for field, value in [("scope_hash", "b" * 64), ("producer", "other"),
                             ("producer_version", "2"), ("definition_id", "v2"), ("unit", "lines")]:
            with self.subTest(field=field):
                result = compare_metric(metric(), metric(50, **{field: value}))
                self.assertEqual(result.status, "non_comparable")
                self.assertIsNone(result.delta.absolute_delta)
                self.assertIn(field, result.delta.reason)
                self.assertIn(str(value), result.delta.reason)
                self.assertEqual(result.delta.before, 100)
                self.assertEqual(result.delta.after, 50)

    def test_all_differences_retained(self):
        result = compare_metric(metric(), metric(50, unit="lines", producer_version="2"))
        self.assertEqual(len(result.differences), 2)
        self.assertIn("unit", result.delta.reason)
        self.assertIn("producer_version", result.delta.reason)

    def test_unavailable_and_missing_are_not_zero(self):
        missing = metric(None, status="unavailable", reason="analyzer absent")
        for before, after in [(missing, metric()), (metric(), missing), (None, metric()), (metric(), None)]:
            with self.subTest(before=before):
                result = compare_metric(before, after)
                self.assertEqual(result.status, "unavailable")
                self.assertIsNone(result.delta.absolute_delta)
                self.assertIsNone(result.delta.percentage_delta)
                self.assertTrue(result.delta.reason)
                self.assertTrue(result.delta.percentage_reason)
        self.assertIn("analyzer absent", compare_metric(missing, metric()).delta.reason)
        with self.assertRaises(ValueError):
            compare_metric(None, None)

    def test_nonfinite_direct_values_and_arithmetic_overflow_have_reasons(self):
        for value in [float("nan"), float("inf"), -float("inf")]:
            result = compare_metric(metric(value), metric())
            self.assertEqual(result.status, "unavailable")
            self.assertIn("finite", result.delta.reason)
            self.assertIsNone(result.delta.before)
        result = compare_metric(metric(-1e308), metric(1e308))
        self.assertEqual(result.delta.status, "unavailable")
        self.assertIn("finite", result.delta.reason)
        result = compare_metric(metric(1e-308), metric(1e308))
        self.assertEqual(result.delta.status, "available")
        self.assertIsNone(result.delta.percentage_delta)
        self.assertIn("finite", result.delta.percentage_reason)

    def test_already_noncomparable_evidence_retains_reason(self):
        result = compare_metric(metric(None, status="non_comparable", reason="old definition"), metric())
        self.assertEqual(result.status, "non_comparable")
        self.assertIn("old definition", result.delta.reason)

    def test_related_analyzers_remain_separately_identified(self):
        before = (metric(key="lizard"), metric(20, key="other", producer="another"))
        after = (metric(10, key="other", producer="another"), metric(80, key="lizard"))
        results = compare_metrics(before, after)
        self.assertEqual([r.delta.key for r in results], ["lizard", "other"])
        self.assertEqual([r.delta.absolute_delta for r in results], [-20, -10])
        self.assertEqual(len(compare_metrics(before, after[:1])), 2)
        with self.assertRaises(ValueError):
            compare_metrics((before[0], before[0]), after)
        with self.assertRaises(ValueError):
            compare_metric(before[0], before[1])

    def test_structured_measurements_are_preserved_without_projection(self):
        values = [ComplexityPayload("complexity", (), (), 0, None, None, None, None),
                  DuplicationPayload("duplication", 10, 100, .1, None, "strict", 10, (), (), ()),
                  FindingsPayload("findings", ())]
        for value in values:
            result = compare_metric(metric(value), metric(value))
            self.assertEqual(result.status, "available")
            self.assertEqual(result.before.value, value)
            self.assertEqual(result.after.value, value)
            self.assertEqual(result.delta.status, "unavailable")
            self.assertIn("scalar", result.delta.reason)
        result = compare_metric(metric(values[0]), metric(values[1]))
        self.assertEqual(result.status, "non_comparable")

    def test_scalar_to_structured_change_is_incompatible(self):
        result = compare_metric(metric(10), metric(FindingsPayload("findings", ())))
        self.assertEqual(result.status, "non_comparable")
        self.assertIn("payload type", result.delta.reason)

    def test_available_side_survives_missing_evidence_and_new_metric_is_retained(self):
        results = compare_metrics((), (metric(7),))
        self.assertEqual(len(results), 1)
        self.assertIsNone(results[0].delta.before)
        self.assertEqual(results[0].delta.after, 7)
        self.assertEqual(results[0].status, "unavailable")
        self.assertEqual(compare_metrics((), ()), ())

    def test_population_changes_disclose_ids_hash_denominator_and_explanation(self):
        before = PopulationPayload("population", ("a", "b"), "a" * 64, 1, 2, None)
        after = PopulationPayload("population", ("a", "c", "d"), "b" * 64, 2, 3, None)
        result = compare_metric(metric(before), metric(after))
        self.assertEqual(result.status, "non_comparable")
        self.assertIn("population_ids", result.delta.reason)
        self.assertIn("population_hash", result.delta.reason)
        self.assertIn("denominator", result.delta.reason)
        explained = replace(after, explanation="Added branches and replaced obsolete branch b")
        result = compare_metric(metric(before), metric(explained))
        self.assertEqual(result.status, "available")
        self.assertEqual(result.after.value.explanation, explained.explanation)
        self.assertTrue(result.differences)
        self.assertEqual(result.delta.status, "unavailable")
        self.assertIsNone(result.delta.absolute_delta)

    def test_population_numerator_and_order_are_not_population_changes(self):
        before = PopulationPayload("population", ("a", "b"), "a" * 64, 1, 2, None)
        after = replace(before, population_ids=("b", "a"), numerator=2)
        result = compare_metric(metric(before), metric(after))
        self.assertEqual(result.status, "available")
        self.assertEqual(result.before.value.numerator, 1)
        self.assertEqual(result.after.value.numerator, 2)

    def test_old_or_empty_population_explanation_does_not_explain_candidate(self):
        before = PopulationPayload("population", ("a",), "a" * 64, 1, 1, "Previous change")
        for explanation in [None, "", "  "]:
            after = replace(before, denominator=2, explanation=explanation)
            result = compare_metric(metric(before), metric(after))
            self.assertEqual(result.status, "non_comparable")

    def test_population_explanation_never_overrides_scope_or_analyzer_difference(self):
        before = PopulationPayload("population", ("a",), "a" * 64, 1, 1, None)
        after = replace(before, denominator=2, explanation="Added branch")
        result = compare_metric(metric(before), metric(after, scope_hash="b" * 64))
        self.assertEqual(result.status, "non_comparable")
        self.assertIn("scope_hash", result.delta.reason)

    def test_threshold_operators_boundaries_units_and_unavailable(self):
        comparison = compare_metric(metric(100), metric(80))
        cases = [("absolute_maximum", 80, "bytes", "passed", 80),
                 ("absolute_maximum", 79, "bytes", "failed", 80),
                 ("delta_maximum", -20, "bytes", "passed", -20),
                 ("delta_maximum", -21, "bytes", "failed", -20),
                 ("percentage_delta_maximum", -20, "percent", "passed", -20),
                 ("percentage_delta_maximum", -21, "percent", "failed", -20),
                 ("delta_maximum", 0, "lines", "unavailable", None),
                 ("percentage_delta_maximum", 0, "bytes", "unavailable", None)]
        for operator, limit, unit, status, observed in cases:
            with self.subTest(operator=operator, limit=limit, unit=unit):
                outcome = evaluate_threshold(comparison, Threshold("size", operator, limit, unit))
                self.assertEqual((outcome.status, outcome.observed), (status, observed))
                self.assertEqual(outcome.reason is not None, status != "passed")
        zero = compare_metric(metric(0), metric(5))
        result = evaluate_threshold(zero, Threshold("size", "percentage_delta_maximum", 10, "percent"))
        self.assertEqual(result.status, "unavailable")
        incompatible = compare_metric(metric(), metric(10, producer_version="2"))
        result = evaluate_threshold(incompatible, Threshold("size", "absolute_maximum", 20, "bytes"))
        self.assertEqual(result.status, "unavailable")

    def test_absolute_threshold_needs_no_delta_arithmetic(self):
        comparison = compare_metric(metric(-1e308), metric(1e308))
        result = evaluate_threshold(comparison, Threshold("size", "absolute_maximum", 1e308, "bytes"))
        self.assertEqual(result.status, "passed")
        result = evaluate_threshold(comparison, Threshold("size", "delta_maximum", 1e308, "bytes"))
        self.assertEqual(result.status, "unavailable")

    def test_threshold_wrong_key_or_structured_measurement_is_unavailable(self):
        result = evaluate_threshold(compare_metric(metric(), metric()),
                                    Threshold("unrelated", "delta_maximum", 0, "bytes"))
        self.assertEqual(result.status, "unavailable")
        self.assertIn("key", result.reason)
        value = FindingsPayload("findings", ())
        result = evaluate_threshold(compare_metric(metric(value), metric(value)),
                                    Threshold("size", "absolute_maximum", 0, "bytes"))
        self.assertEqual(result.status, "unavailable")
        self.assertIn("scalar", result.reason)

    def test_percentage_avoids_spurious_intermediate_overflow(self):
        result = compare_metric(metric(1e308), metric(1.1e308))
        self.assertAlmostEqual(result.delta.percentage_delta, 10)

    def test_percentage_uses_declared_formula_order(self):
        import math
        before, after = 0.1, math.nextafter(0.1, math.inf)
        result = compare_metric(metric(before), metric(after))
        self.assertEqual(result.delta.percentage_delta, 100 * (after - before) / before)


class AcceptanceTests(unittest.TestCase):
    def setUp(self):
        from lesscode_baseline import contracts as c
        from tests.test_contracts import profile, observation
        self.c = c
        self.profile = c.Profile.from_dict(profile())
        self.baseline = c.Observation.from_dict(observation())
        self.baseline = replace(self.baseline, metrics=tuple(
            metric(100, key=key) for key in self.profile.required_metrics))
        self.candidate = replace(self.baseline, metrics=tuple(
            replace(value, value=80) for value in self.baseline.metrics))
        self.integrity = c.IntegrityResult('unchanged', (), c.Protection('local_detection', None))

    def compare(self, baseline=None, candidate=None, profile=None, integrity=None):
        from lesscode_baseline.decision import compare
        return compare(baseline or self.baseline, candidate or self.candidate,
                       profile or self.profile, integrity or self.integrity)

    def check_status(self, observation, status):
        return replace(observation, checks=(replace(observation.checks[0], status=status),))

    def test_every_primary_state_and_priority(self):
        cases = [({}, 'accepted'),
                 ({'integrity': replace(self.integrity, status='changed', changed_resources=('oracle.py',))}, 'invalid'),
                 ({'baseline': self.check_status(self.baseline, 'failed')}, 'baseline_ineligible'),
                 ({'candidate': self.check_status(self.candidate, 'failed')}, 'rejected'),
                 ({'candidate': replace(self.candidate, metrics=())}, 'incomplete')]
        for arguments, expected in cases:
            with self.subTest(expected=expected):
                result = self.compare(**arguments)
                self.assertEqual(result.decision, expected)
                self.assertEqual(self.c.Comparison.from_dict(result.to_dict()), result)
        result = self.compare(baseline=self.check_status(self.baseline, 'failed'),
                              candidate=self.check_status(self.candidate, 'failed'),
                              integrity=replace(self.integrity, status='changed', changed_resources=('oracle.py',)))
        self.assertEqual(result.decision, 'invalid')
        for text in ['oracle.py', 'baseline', 'candidate', 'failed']:
            self.assertIn(text, ' '.join(result.reasons))

    def test_all_declared_baseline_checks_even_optional_must_pass(self):
        optional = replace(self.profile.checks[0], required=False, check_id='optional',
                           expected_ids=('optional.api',), command=replace(self.profile.checks[0].command, command_id='optional'))
        profile = replace(self.profile, checks=(*self.profile.checks, optional))
        for status in ['failed', 'skipped', 'missing', 'error', None]:
            for required in [True, False]:
                with self.subTest(status=status, required=required):
                    spec = replace(optional, required=required)
                    declared = replace(profile, checks=(*self.profile.checks, spec))
                    extra = () if status is None else (replace(self.baseline.checks[0], check_id='optional.api', required=required, status=status),)
                    before = replace(self.baseline, checks=(*self.baseline.checks, *extra))
                    after = replace(self.candidate, checks=(*self.candidate.checks,
                        replace(self.candidate.checks[0], check_id='optional.api', required=required)))
                    self.assertEqual(self.compare(before, after, declared).decision, 'baseline_ineligible')

    def test_shorter_candidate_failure_cannot_be_compensated(self):
        for kind in ['native', 'golden', 'invariant', 'public_api']:
            profile = replace(self.profile, checks=(replace(self.profile.checks[0], kind=kind),))
            before = replace(self.baseline, checks=(replace(self.baseline.checks[0], kind=kind),))
            after = replace(self.candidate, checks=(replace(self.candidate.checks[0], kind=kind, status='failed'),),
                            metrics=tuple(replace(m, value=1) for m in self.candidate.metrics))
            self.assertEqual(self.compare(before, after, profile).decision, 'rejected')

    def test_candidate_required_skipped_missing_error_and_absent_incomplete(self):
        for status in ['skipped', 'missing', 'error', None]:
            after = replace(self.candidate, checks=()) if status is None else self.check_status(self.candidate, status)
            self.assertEqual(self.compare(candidate=after).decision, 'incomplete')

    def test_optional_candidate_failure_rejects_but_absence_is_disclosed(self):
        profile = replace(self.profile, checks=(replace(self.profile.checks[0], required=False),))
        before = replace(self.baseline, checks=(replace(self.baseline.checks[0], required=False),))
        for status, expected in [('failed', 'rejected'), ('missing', 'accepted'), ('skipped', 'accepted'), ('error', 'accepted')]:
            after = replace(self.candidate, checks=(replace(self.candidate.checks[0], required=False, status=status),))
            result = self.compare(before, after, profile)
            self.assertEqual(result.decision, expected)
            self.assertIn(status, ' '.join(result.reasons))

    def test_required_check_identity_kind_and_flag_are_frozen(self):
        for change in [dict(required=False), dict(kind='golden'), dict(check_id='replacement')]:
            after = replace(self.candidate, checks=(replace(self.candidate.checks[0], **change),))
            self.assertNotEqual(self.compare(candidate=after).decision, 'accepted')

    def test_noncomparable_required_metric_and_missing_both_sides(self):
        for before, after in [(self.baseline, replace(self.candidate, metrics=())),
                              (replace(self.baseline, metrics=()), replace(self.candidate, metrics=())),
                              (self.baseline, replace(self.candidate, metrics=tuple(replace(m, producer_version='2') for m in self.candidate.metrics)))]:
            self.assertEqual(self.compare(before, after).decision, 'incomplete')

    def test_structured_required_measurement_satisfies_availability_without_scalar(self):
        value = FindingsPayload('findings', ())
        before = replace(self.baseline, metrics=tuple(replace(m, value=value) for m in self.baseline.metrics))
        after = replace(self.candidate, metrics=before.metrics)
        profile = replace(self.profile, thresholds=())
        result = self.compare(before, after, profile)
        self.assertEqual(result.decision, 'accepted')
        self.assertTrue(all(d.status == 'unavailable' for d in result.metric_deltas))
        self.assertEqual(self.compare(before, after).decision, 'incomplete')

    def test_optional_metrics_do_not_block_but_limitations_remain(self):
        for evidence in [metric(None, key='coverage', status='unavailable', reason='producer absent'),
                         metric(12, key='coverage', producer_version='2')]:
            before = replace(self.baseline, metrics=(*self.baseline.metrics, metric(key='coverage')))
            after = replace(self.candidate, metrics=(*self.candidate.metrics, evidence))
            result = self.compare(before, after)
            self.assertEqual(result.decision, 'accepted')
            self.assertIn('coverage', ' '.join(result.reasons))
            self.assertEqual(self.compare(before, after, replace(self.profile, required_metrics=(*self.profile.required_metrics, 'coverage'))).decision, 'incomplete')

    def test_threshold_violation_overrides_missing_required_metric_and_all_reasons_retained(self):
        after = replace(self.candidate, metrics=(metric(120, key='source_bytes'),))
        result = self.compare(candidate=after)
        self.assertEqual(result.decision, 'rejected')
        for text in ['source_bytes', 'complexity', 'duplication']:
            self.assertIn(text, ' '.join(result.reasons))
        self.assertEqual(self.compare(profile=replace(self.profile, thresholds=(Threshold('absent', 'absolute_maximum', 0, 'bytes'),))).decision, 'incomplete')
        zero = replace(self.baseline, metrics=tuple(replace(m, value=0) for m in self.baseline.metrics))
        self.assertEqual(self.compare(zero, profile=replace(self.profile, thresholds=(Threshold('source_bytes', 'percentage_delta_maximum', 0, 'percent'),))).decision, 'incomplete')

    def test_integrity_unavailable_never_accepts_and_protection_is_disclosed(self):
        self.assertNotEqual(self.compare(integrity=replace(self.integrity, status='unavailable')).decision, 'accepted')
        for protection in [self.integrity.protection, self.c.Protection('external_isolation_claimed', 'provider:job123')]:
            result = self.compare(integrity=replace(self.integrity, protection=protection))
            self.assertIn(protection.mode, ' '.join(result.reasons))
            if protection.provider_evidence:
                self.assertIn(protection.provider_evidence, ' '.join(result.reasons))
            self.assertIn('unchecked behavior', ' '.join(result.reasons))
            self.assertIn('not proof', ' '.join(result.reasons))

    def test_existing_failures_are_retained_without_optional_metric_blocking(self):
        failure = self.c.Failure('optional_evidence_missing', 'coverage producer absent', 'coverage.json', None, None, False, None)
        after = replace(self.candidate, failures=(failure,))
        result = self.compare(candidate=after)
        self.assertEqual(result.decision, 'accepted')
        self.assertIn(failure.code, ' '.join(result.reasons))

    def test_unfinished_or_nonzero_declared_execution_overrides_passed_report(self):
        from tests.test_contracts import plan
        command = self.c.ResolvedCommand.from_dict(plan()['resolved_commands'][0])
        for status, exit_code in [('timeout', None), ('log_limit', None), ('error', None), ('completed', 1)]:
            execution = self.c.ExecutionResult(command, exit_code, status, 1, 'stdout.log', 'stderr.log', '2026-10-31T08:00:00Z', 'a'*64)
            for side, expected in [('baseline', 'baseline_ineligible'), ('candidate', 'incomplete')]:
                changed = replace(getattr(self, side), executions=(execution,))
                result = self.compare(**{side: changed})
                self.assertEqual(result.decision, expected)
                self.assertIn(status, ' '.join(result.reasons))

    def test_foreign_or_duplicate_check_evidence_is_invalid(self):
        for checks in [(replace(self.candidate.checks[0], source_hash='b'*64),),
                       (replace(self.candidate.checks[0], plan_hash='b'*64),),
                       (self.candidate.checks[0], self.candidate.checks[0])]:
            with self.subTest(checks=checks):
                self.assertEqual(self.compare(candidate=replace(self.candidate, checks=checks)).decision, 'invalid')

    def test_baseline_ineligibility_precedes_rejection_and_incompleteness(self):
        before = self.check_status(self.baseline, 'skipped')
        after = replace(self.check_status(self.candidate, 'failed'), metrics=())
        result = self.compare(before, after)
        self.assertEqual(result.decision, 'baseline_ineligible')
        for detail in ['baseline', 'skipped', 'candidate', 'failed', 'required metric']:
            self.assertIn(detail, ' '.join(result.reasons))

    def test_declared_command_failure_record_cannot_hide_behind_passed_report(self):
        failure = self.c.Failure('check_report_invalid', 'fresh report malformed', 'report.json',
                                 self.profile.checks[0].check_id, None, False, 'bad.json')
        before = replace(self.baseline, failures=(failure,))
        after = replace(self.candidate, failures=(failure,))
        self.assertEqual(self.compare(baseline=before).decision, 'baseline_ineligible')
        self.assertEqual(self.compare(candidate=after).decision, 'incomplete')


if __name__ == "__main__":
    unittest.main()
