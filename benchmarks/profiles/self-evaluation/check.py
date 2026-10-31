"""Fixed external checks for the Lesscode threshold-dispatch example."""

import argparse
from dataclasses import replace
import hashlib
import json
from pathlib import Path
import sys
import traceback
import unittest
import xml.etree.ElementTree as ET


MODULES = (
    "tests.test_decision", "tests.test_contracts",
    "tests.test_experiments", "tests.test_experiment_contracts",
)


def cases(suite):
    for item in suite:
        if isinstance(item, unittest.TestSuite):
            yield from cases(item)
        else:
            yield item


def unit_suite(source):
    resources = json.loads(Path(__file__).with_name("test-resources.json").read_text())
    for relative, expected_hash in resources.items():
        if hashlib.sha256((source / relative).read_bytes()).hexdigest() != expected_hash:
            raise AssertionError(f"fixed test resource changed: {relative}")
    suite = unittest.defaultTestLoader.loadTestsFromNames(MODULES)
    expected = json.loads(Path(__file__).with_name("inventory.json").read_text())
    observed = sorted(case.id() for case in cases(suite))
    if observed != expected:
        raise AssertionError("selected test inventory differs from the fixed baseline")
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    if not result.wasSuccessful() or result.skipped or result.testsRun != len(expected):
        raise AssertionError("every fixed test must execute and pass")


def threshold_oracle():
    from lesscode_baseline.contracts import MetricDelta, Threshold
    from lesscode_baseline.decision import MetricComparison, evaluate_threshold

    delta = MetricDelta("size", "available", 20, 30, 10, 50, None, None, "bytes")
    comparison = MetricComparison(None, None, "available", (), delta)
    for operator, observed in (("absolute_maximum", 30), ("delta_maximum", 10),
                               ("percentage_delta_maximum", 50)):
        unit = "percent" if operator == "percentage_delta_maximum" else "bytes"
        for limit in (-1, 0, 10, 30, 50, 100):
            rule = Threshold("size", operator, limit, unit)
            result = evaluate_threshold(comparison, rule)
            expected = "passed" if observed <= limit else "failed"
            assert (result.status, result.observed) == (expected, observed), result
        missing = replace(delta, after=None, absolute_delta=None, percentage_delta=None,
                          reason="no scalar", percentage_reason="zero baseline")
        result = evaluate_threshold(replace(comparison, delta=missing), rule)
        expected_reason = "zero baseline" if unit == "percent" else "no scalar"
        assert (result.status, result.reason) == ("unavailable", expected_reason), result
        wrong_unit = evaluate_threshold(comparison, replace(rule, unit="wrong"))
        assert wrong_unit.status == "unavailable", wrong_unit


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    source = args.source.resolve()
    sys.path[:0] = [str(source / "src"), str(source)]
    suite = ET.Element("testsuite", name="lesscode-self-evaluation", tests="2")
    failures = 0
    for identity, check in (("self.unit_suite", lambda: unit_suite(source)),
                             ("self.threshold_oracle", threshold_oracle)):
        case = ET.SubElement(suite, "testcase", name=identity)
        try:
            check()
        except Exception:
            failures += 1
            ET.SubElement(case, "failure", message="fixed check failed").text = traceback.format_exc()
    suite.set("failures", str(failures))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    ET.ElementTree(suite).write(args.output, encoding="utf-8", xml_declaration=True)
    return int(bool(failures))


if __name__ == "__main__":
    raise SystemExit(main())
