"""G1-R live weaker-model recovery runner (constraint-guided-compression task 6.2).

Prototype-gate.md L27 live check: at least one non-deterministic or weaker-model
multi-turn run recovers from a Tier-1 or Tier-2 failure to R = 1.0 within
max_turns <= 4 and max_static_retries <= 3. This runner executes one live run per
invocation against a small frozen task, appends its receipt under results/live/
and merges a run entry into results/g1r-live-record.json (cost included).

Honesty rules (Req 7.8): every invocation is recorded whether or not it helps;
the escalation ladder (task A small, task B two-module with cross imports) is
declared here and executed by the operator in labeled runs; no run is deleted.
No failure is scripted: the weaker model must produce any recorded failure
itself. The runtime oracle is deterministic and frozen outside candidate scope
(scope_files list only the module under regeneration); the live element is the
model call, exactly as in the deterministic checklist item 2 vs item 3 split.

Run:  venv-python g1r_live.py TASK LABEL    # TASK in {A, B}
"""
from __future__ import annotations

import json
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
RESULTS = HERE / "results"
LIVE = RESULTS / "live"
LESSCODE = HERE.parents[3]
sys.path.insert(0, str(LESSCODE / "src"))

from lesscode_baseline import regenerator as rg  # noqa: E402
from lesscode_baseline import surface  # noqa: E402

WEAKER_MODEL = "claude-haiku-4-5"  # probed available 2026-11-08; opus-5-5 recorded E4
RECORDER_REL = "oracle/check.py"

# ---------------------------------------------------------------------------
# Frozen task fixtures. The oracle embeds golden expected values; the runner
# self-tests oracle vs frozen tree before any model spend (abort on mismatch).
# ---------------------------------------------------------------------------

TASK_A_MAIN = '''\
"""Cart pricing helpers (live-run task A)."""


def unit_price(item):
    table = {"apple": 1.25, "pear": 1.10, "fig": 2.40}
    return table[item]


def line_total(item, qty):
    return round(unit_price(item) * qty, 2)


def bulk_discount(qty, total):
    if qty >= 10:
        return round(total * 0.90, 2)
    return total


def cart_total(lines):
    raw = sum(line_total(item, qty) for item, qty in lines)
    return bulk_discount(sum(qty for _, qty in lines), raw)


def format_receipt(lines):
    parts = [f"{item} x{qty}: {line_total(item, qty):.2f}" for item, qty in lines]
    parts.append(f"total: {cart_total(lines):.2f}")
    return "; ".join(parts)


def save_receipt(conn, text):
    conn.write(("receipts", text))
'''

TASK_B_MAIN = '''\
"""Cart pricing with regional tax (live-run task B)."""
from tax import tax_rate


def unit_price(item):
    table = {"apple": 1.25, "pear": 1.10, "fig": 2.40}
    return table[item]


def line_total_with_tax(item, qty, region):
    rate = tax_rate(region)
    return round(unit_price(item) * qty * (1 + rate), 2)


def bulk_discount(qty, total):
    if qty >= 10:
        return round(total * 0.90, 2)
    return total


def cart_total(lines, region):
    raw = sum(line_total_with_tax(item, qty, region) for item, qty in lines)
    return bulk_discount(sum(qty for _, qty in lines), raw)


def format_receipt(lines, region):
    parts = [f"{item} x{qty}: {line_total_with_tax(item, qty, region):.2f}" for item, qty in lines]
    parts.append(f"total: {cart_total(lines, region):.2f}")
    return "; ".join(parts)


def save_receipt(conn, text):
    conn.write(("receipts", text))
'''

TASK_B_TAX = '''\
"""Regional tax table (live-run task B)."""


def tax_rate(region):
    rates = {"eu": 0.20, "us": 0.07, "row": 0.00}
    return rates[region]


def with_tax(amount, region):
    return round(amount * (1 + tax_rate(region)), 2)
'''

ORACLE_TEMPLATE = '''\
"""Golden-output oracle (recorder, frozen outside candidate scope)."""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

{imports}

CASES = []


def case(name):
    def deco(fn):
        CASES.append((name, fn))
        return fn
    return deco


{cases}


def main():
    failures = []
    for i, (name, fn) in enumerate(CASES):
        try:
            fn()
        except Exception as e:  # any failure is a divergence
            failures.append({{"case": i, "name": name, "error": f"{{type(e).__name__}}: {{e}}"}})
    total = len(CASES)
    passed = total - len(failures)
    print(json.dumps({{"ok": not failures, "passed": passed, "total": total, "failures": failures}}))


main()
'''

# Direct from-imports so derive_surface binds every task symbol with a
# recorder tag (the e4_oracle/labeled_reason convention).
ORACLE_A_IMPORTS = "from cart import bulk_discount, cart_total, format_receipt, line_total, save_receipt, unit_price"
ORACLE_A_CASES = '''\
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
'''

ORACLE_B_IMPORTS = ("from cart import bulk_discount, cart_total, format_receipt, line_total_with_tax, "
                    "save_receipt, unit_price\nfrom tax import tax_rate, with_tax")
ORACLE_B_CASES = '''\
@case("unit_price pear")
def _c1():
    assert unit_price("pear") == 1.10


@case("line_total_with_tax apple x2 eu")
def _c2():
    assert line_total_with_tax("apple", 2, "eu") == 3.0


@case("bulk_discount below threshold")
def _c3():
    assert bulk_discount(2, 5.0) == 5.0


@case("bulk_discount at threshold")
def _c4():
    assert bulk_discount(10, 20.0) == 18.0


@case("cart_total mixed eu")
def _c5():
    assert cart_total([("apple", 4), ("pear", 2)], "eu") == 8.64


@case("tax_rate us")
def _c6():
    assert tax_rate("us") == 0.07


@case("with_tax row")
def _c7():
    assert with_tax(1.0, "row") == 1.0
'''


def _write(root: Path, rel: str, text: str) -> None:
    p = root / rel
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(text, encoding="utf-8", newline="\n")


def build_task(run_dir: Path, task: str) -> tuple[rg.TurnRequest, Path]:
    frozen = run_dir / "frozen"
    if task == "A":
        _write(frozen, "src/cart.py", TASK_A_MAIN)
        _write(frozen, "src/user.py", "from cart import cart_total, format_receipt\n\nprint(format_receipt([('apple', 2)]))\n")
        scope = ["src/cart.py"]
        defs = ("unit_price/line_total/bulk_discount/cart_total/format_receipt are pure pricing "
                "helpers; save_receipt is the only shell (writes through conn).")
        contract = ("Consolidate the duplicated rounding logic without changing behavior. Every "
                    "public symbol in public-surface.json must remain defined in src/cart.py with "
                    "the same import surface. All pricing helpers stay pure: no I/O, no conn use; "
                    "only save_receipt may touch conn. Stdlib only.")
    elif task == "B":
        _write(frozen, "src/cart.py", TASK_B_MAIN)
        _write(frozen, "src/tax.py", TASK_B_TAX)
        _write(frozen, "src/user.py", "from cart import cart_total, format_receipt\nfrom tax import with_tax\n\nprint(format_receipt([('apple', 2)], 'eu'), with_tax(1.0, 'us'))\n")
        scope = ["src/cart.py", "src/tax.py"]
        defs = ("tax.py holds tax_rate/with_tax; cart.py composes them. save_receipt is the only "
                "shell function; everything else is pure.")
        contract = ("Consolidate the duplicated rounding/multiplication logic across cart.py and "
                    "tax.py without changing behavior. Every public symbol in public-surface.json "
                    "must remain defined with the same home module and import surface. All helpers "
                    "except save_receipt stay pure: no I/O, no conn use. Stdlib only; no new files.")
    else:
        raise SystemExit(f"unknown task {task!r} (A or B)")
    if task == "A":
        oracle_text = ORACLE_TEMPLATE.format(imports=ORACLE_A_IMPORTS, cases=ORACLE_A_CASES)
    else:
        oracle_text = ORACLE_TEMPLATE.format(imports=ORACLE_B_IMPORTS, cases=ORACLE_B_CASES)
    _write(frozen, RECORDER_REL, oracle_text)
    report = surface.derive_surface(frozen, scope, recorder_files=(RECORDER_REL,))
    _write(frozen, "public-surface.json", json.dumps(report, indent=1))
    req = rg.TurnRequest(
        scope_root=str(frozen), scope_files=scope,
        surface_path=str(frozen / "public-surface.json"), definitions=defs,
        targets={}, contract=contract,
        acceptance_command=f"{sys.executable} {RECORDER_REL} (run by the loop, not by you)",
    )
    return req, frozen


def self_test_oracle(frozen: Path) -> dict:
    """Oracle must be green on the frozen tree before any model spend."""
    proc = subprocess.run([sys.executable, RECORDER_REL], cwd=frozen,
                          capture_output=True, text=True, timeout=120)
    payload = json.loads(proc.stdout.strip().splitlines()[-1])
    if proc.returncode != 0 or not payload.get("ok"):
        raise SystemExit(f"oracle self-test failed on frozen tree: {payload} / {proc.stderr[-400:]}")
    return payload


def oracle_evaluator(timeout_s: float = 120.0):
    def evaluate(cand_root: Path, _results: dict):
        try:
            proc = subprocess.run([sys.executable, RECORDER_REL], cwd=cand_root,
                                  capture_output=True, text=True, timeout=timeout_s)
        except subprocess.TimeoutExpired:
            return 124, "oracle timeout", {}
        try:
            payload = json.loads(proc.stdout.strip().splitlines()[-1])
        except (ValueError, IndexError):
            return 2, f"oracle output unparsable: {proc.stdout[-200:]}", {}
        if payload.get("ok"):
            return 0, None, {}
        failures = payload.get("failures") or []
        total = max(int(payload.get("total", 0)), 1)
        passed = int(payload.get("passed", 0))
        first = failures[0] if failures else {}
        divergence = f"case {first.get('case')} ({first.get('name')}): {first.get('error')}"
        fractions = {"seed_group_pass_fraction": round(passed / total, 4),
                     "sequence_pass_fraction": round(passed / total, 4)}
        return 1, divergence[:400], fractions

    return evaluate


def weaker_call_fn():
    """Live endpoint from the owner's model.env with the model id swapped to the weaker probe.

    Weaker-model capability mapping (live findings, 2026-11-08): claude-haiku-4-5
    rejects output_config.effort, so every preregistered effort value maps to
    None (omitted); temperature is dropped by the anthropic arm of rg.call.
    """
    base = rg.load_endpoint()
    ep = rg.Endpoint(base.style, base.base_url, WEAKER_MODEL, base.api_key())

    def call_fn(system: str, user: str, params: dict):
        return rg.call(ep, system, user, max_tokens=params.get("max_tokens", 64000),
                       effort=None, temperature=params.get("temperature"),
                       display=params.get("display"))

    return call_fn


def evaluate_criterion(history: list[dict], status: str, best_score: float) -> dict:
    """Failure-then-acceptance: a recorded sub-1.0 attempt strictly before the accepted best."""
    failure_idx = [i for i, rec in enumerate(history) if rec["score"] < 1.0]
    accepted = status == "accepted" and best_score >= 1.0
    recovered = bool(failure_idx) and accepted
    return {"recorded_failure": bool(failure_idx),
            "first_failure": history[failure_idx[0]] if failure_idx else None,
            "accepted": accepted,
            "failure_then_acceptance": recovered,
            "turns_used": max((rec["turn"] for rec in history), default=0),
            "max_attempt_seen": max((rec["attempt"] for rec in history), default=0)}


def main() -> int:
    if len(sys.argv) != 3:
        print(__doc__)
        return 2
    task, label = sys.argv[1].upper(), sys.argv[2]
    run_dir = LIVE / label
    run_dir.mkdir(parents=True, exist_ok=True)
    req, frozen = build_task(run_dir, task)
    self_test = self_test_oracle(frozen)
    budget = rg.RefineBudget()  # frozen budgets: max_turns=4, max_static_retries=3, k=2
    t0 = time.time()
    outcome = rg.run_refine_loop(
        req, run_dir / "cand", frozen, run_dir / "receipt.json", label,
        budget, runtime_evaluator=oracle_evaluator(), call_fn=weaker_call_fn(),
    )
    criterion = evaluate_criterion(outcome.rollout_history, outcome.status, outcome.best.score)
    entry = {
        "label": label, "task": task, "model": WEAKER_MODEL,
        "status": outcome.status, "best_score": outcome.best.score,
        "best_failure_kind": outcome.best.failure_kind,
        "criterion": criterion, "usage": outcome.usage,
        "elapsed_s": round(time.time() - t0, 1),
        "oracle_self_test": self_test,
        "budget": {"max_turns": budget.max_turns, "max_static_retries": budget.max_static_retries,
                   "n_rollouts_per_turn": budget.n_rollouts_per_turn,
                   "no_progress_limit": budget.no_progress_limit, "threshold": budget.threshold},
        "receipt": str((run_dir / "receipt.json").relative_to(LESSCODE)),
        "written_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    }
    record_path = RESULTS / "g1r-live-record.json"
    record = json.loads(record_path.read_text(encoding="utf-8")) if record_path.exists() else {"runs": []}
    record["runs"] = [r for r in record["runs"] if r["label"] != label] + [entry]
    record["criterion_met"] = any(r["criterion"]["failure_then_acceptance"] for r in record["runs"])
    record_path.write_text(json.dumps(record, indent=1), encoding="utf-8")
    print(json.dumps(entry, indent=1))
    return 0 if entry["criterion"]["failure_then_acceptance"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
