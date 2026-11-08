"""Golden-output oracle (recorder, frozen outside candidate scope)."""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from cart import bulk_discount, cart_total, format_receipt, line_total, save_receipt, unit_price

CASES = []


def case(name):
    def deco(fn):
        CASES.append((name, fn))
        return fn
    return deco


@case("unit_price pear")
def _c1():
    assert unit_price("pear") == 1.10


@case("line_total fig x3")
def _c2():
    assert line_total("fig", 3) == 7.20


@case("bulk_discount below threshold")
def _c3():
    assert bulk_discount(2, 5.0) == 5.0


@case("bulk_discount at threshold")
def _c4():
    assert bulk_discount(10, 20.0) == 18.0


@case("cart_total mixed")
def _c5():
    assert cart_total([("apple", 4), ("pear", 2)]) == 7.2


@case("format_receipt single line")
def _c6():
    assert format_receipt([("apple", 2)]) == "apple x2: 2.50; total: 2.50"



def main():
    failures = []
    for i, (name, fn) in enumerate(CASES):
        try:
            fn()
        except Exception as e:  # any failure is a divergence
            failures.append({"case": i, "name": name, "error": f"{type(e).__name__}: {e}"})
    total = len(CASES)
    passed = total - len(failures)
    print(json.dumps({"ok": not failures, "passed": passed, "total": total, "failures": failures}))


main()
