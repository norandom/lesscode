"""Trusted pilot checks. Run only against disposable source copies.

Native mode executes the unmodified upstream doctests. Independent mode uses
fixed external expected cases and invariants; no analyzer or evaluator is here.
"""

import argparse
import doctest
import importlib.util
import json
from pathlib import Path
import traceback
import xml.etree.ElementTree as ET


def load(source):
    spec = importlib.util.spec_from_file_location("pilot_subject", source / "mathutils.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def native(module):
    result = doctest.testmod(module)
    # Prevent deletion of all upstream examples from yielding a passing command.
    if result.attempted != 10 or result.failed:
        raise AssertionError(f"expected 10 upstream examples; observed {result}")


def golden(module):
    cases = json.loads(Path(__file__).with_name("golden.json").read_text(encoding="utf-8"))
    for case in cases:
        function = getattr(module, case["function"])
        if "raises" in case:
            try:
                function(*case["args"])
            except ValueError:
                continue
            raise AssertionError(f"{case['id']}: expected ValueError")
        actual = function(*case["args"])
        if actual != case["expected"]:
            raise AssertionError(f"{case['id']}: {actual!r} != {case['expected']!r}")


def invariant(module):
    for lower, upper in [(-3, 4), (0, 0), (2, 9)]:
        for value in range(-6, 13):
            actual = module.clamp(value, lower, upper)
            if not lower <= actual <= upper:
                raise AssertionError("clamp bounds")
            if lower <= value <= upper and actual != value:
                raise AssertionError("clamp identity within bounds")
    for value in range(256):
        bits = module.Bits.from_int(value, 8)
        if (bits.as_int() != value or len(bits) != 8 or
                bits.as_bin() != format(value, "08b") or
                bits.as_bytes() != bytes([value]) or
                module.Bits.from_bytes(bytes([value])).as_int() != value):
            raise AssertionError(f"Bits 8-bit roundtrip {value}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--mode", choices=("native", "independent"), required=True)
    args = parser.parse_args()
    checks = [("native.doctests", native)] if args.mode == "native" else [
        ("independent.golden", golden), ("independent.invariants", invariant)]
    suite = ET.Element("testsuite", name="boltons-mathutils", tests=str(len(checks)))
    failures = 0
    for identity, function in checks:
        case = ET.SubElement(suite, "testcase", name=identity)
        try:
            function(load(args.source.resolve()))
        except Exception:
            failures += 1
            ET.SubElement(case, "failure", message="declared check failed").text = traceback.format_exc()
    suite.set("failures", str(failures))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    ET.ElementTree(suite).write(args.output, encoding="utf-8", xml_declaration=True)
    return int(failures != 0)


if __name__ == "__main__":
    raise SystemExit(main())
