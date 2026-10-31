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


if __name__ == "__main__":
    unittest.main()
