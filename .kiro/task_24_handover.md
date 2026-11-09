# Task Group Handover: Python Baseline Retirement & Standalone Deconvolution Skill Packaging

- **Date:** 2026-11-09
- **Handover Index:** 24
- **Preceding Handover:** `.kiro/task_23_handover.md` (Stage 4: Rust Deconvolution Adapter)
- **Status:** **COMPLETED — PYTHON BASELINE RETIRED — STANDALONE SKILL PACKAGED — PUSHED TO REMOTE**

---

## 1. What Was Done

1. **Owner WIP Files Preserved & Committed (`fd0b5b9`):**
   - Committed the 4 preserved owner WIP files:
     - `src/lesscode_baseline/rules/evaluate.py`
     - `tests/test_rule_evaluation.py`
     - `docs/experiments/deconvolution/e3-evidence-regeneration/results/sql-divergences-round21.json`
     - `docs/experiments/deconvolution/e4-lifecycle/results/sql-divergences-round21.json`
   - Pushed commit `fd0b5b9` to `origin/main`.

2. **Preserved Baseline Documentation Index (`4f0aed4`):**
   - Created [`docs/archive/preserved-python-baseline.md`](../docs/archive/preserved-python-baseline.md) recording commit hashes, file locations, lookup commands (`git show`, `git log`), and recovery procedures (`git checkout fd0b5b9 -- ...`).
   - Linked in [`docs/README.md`](../docs/README.md#L18).

3. **Steering Policy Amendments (`4f0aed4`):**
   - Updated [`.kiro/steering/tech.md`](steering/tech.md): declared the native Rust substrate (`lesscode-core`) the sole production and reference engine; formalized the retirement of `src/lesscode_baseline/`. Python is now supported exclusively as an external target-project specimen runtime.
   - Updated [`.kiro/steering/structure.md`](steering/structure.md): documented retirement of `src/lesscode_baseline/` and transition to `native/`.

4. **Python Baseline & Legacy Tests Retirement (`4f0aed4`):**
   - Removed `src/lesscode_baseline/` and legacy `tests/` (117 files, 23,526 lines removed).
   - Verified that `native/` is 100% self-contained and passes all budget checks.
   - Pushed commit `4f0aed4` to `origin/main`.

5. **Standalone Deconvolution Skill Packaging (`4939f53`):**
   - Created [`.agents/skills/deconvolute/SKILL.md`](../.agents/skills/deconvolute/SKILL.md).
   - Documented the end-to-end deconvolution workflow driving `lesscode-core` natively across Python, TypeScript, Java, and Rust.
   - Pushed commit `4939f53` to `origin/main`.

---

## 2. Evidence & Commit References

- **Preserved Python Baseline Commit:** `fd0b5b9`
- **Retirement Commit:** `4f0aed4`
- **Standalone Skill Packaging Commit:** `4939f53`
- **Documentation Guide:** [`docs/archive/preserved-python-baseline.md`](../docs/archive/preserved-python-baseline.md)
- **Standalone Agent Skill:** [`.agents/skills/deconvolute/SKILL.md`](../.agents/skills/deconvolute/SKILL.md)
- **Substrate Health:** 10/10 budget tests passed; 338/338 nextest tests verified.

---

## 3. Current State

The repository no longer maintains an internal Python execution engine. Lesscode is now distributed and executed as:
1. **A Compiled Native Rust Binary (`lesscode-core`):** Providing CLI commands (`prepare`, `compare`, `admission`, `budgets`, `surface`, `regenerate`).
2. **A Reusable Agent Skill (`deconvolute`):** Driving the native binary for code deconvolution across Python, TypeScript, Java, and Rust codebases.

---

## 4. Next Steps

1. Clear conversation context per owner rule (2026-11-07).
2. Start the next project milestone from `.kiro/task_24_handover.md`.
