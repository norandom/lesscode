# Task group 15 handover: Lifecycle deconvolution adoption & self-application maturity gate qualification

Written 2026-11-09. Group scope: execute the formal **Self-Application Maturity Gate & Capability Adoption** for the deconvoluted Facdrone `lifecycle` monitor group (7 files, 1,124 lines vs 1,495 baseline), following owner selection of Option A1.
All execution, measurement, diff generation, and verification ran in the foreground. No subagents were dispatched, per owner rule.

---

## What was done

1. **Adoption Target Establishment:**
   - Established the dedicated adopted specimen tree at `.runs/e5-adopted-lifecycle/`.
   - Verified that `.runs/e5-adopted-lifecycle/` differs from the frozen baseline `.runs/e2-facdrone/source/` in exactly the 7 scope files (0 extraneous diffs).
   - Preserved `.runs/e2-facdrone/source/` read-only as the canonical baseline reference snapshot.

2. **Unified Diff & Hash Integrity Generation:**
   - Computed baseline and adopted SHA-256 digests for all 7 scope files.
   - Generated the complete 1,504-line unified diff at `docs/experiments/deconvolution/e5-state-boundary-lifecycle/results/e5-adopted-lifecycle.diff`.
   - Recorded full metrics and evidence in `docs/experiments/deconvolution/e5-state-boundary-lifecycle/results/e5-adoption-report.json` and generated its SHA-256 sidecar.

3. **End-to-End Regression & Acceptance Verification:**
   - Executed `run_e5.py .runs/e5-adopted-lifecycle --round 3`:
     - Specimen root tests: 108 passed, 1 skipped, 1 deselected in 9.46s (exit 0).
     - Specimen research package tests: 1 passed in 1.53s (exit 0).
     - Golden replay oracle: 50/50 sequences replayed with 317/317 operations and 0 Level 2 divergences (`sql-divergences-round3.json`).
   - Executed `e5_evaluator.py --candidate .runs/e5-adopted-lifecycle`:
     - 4/4 adverse controls killed (E3 `AuditedAbsence`, E4 F1, F3, F7).
     - 0 bounded pure-core counterexamples over $[-10, 10]$ and edge cases.
     - DensityGate scored: `structurally_consolidated: True`, `compression_met: True`, `disposition: PASS`.

4. **Research Documentation & Spec Synchronization:**
   - Authored formal research log entry `docs/research/2026-11-09/01-e5-lifecycle-adoption-maturity-gate.md` ([R43](../docs/research/2026-11-09/01-e5-lifecycle-adoption-maturity-gate.md)).
   - Updated `docs/research/README.md`.
   - Updated `.kiro/specs/state-boundary-deconvolution/spec.json`.

---

## Fresh Evidence

### 1. Adopted Scope Reductions

| Metric | Baseline | Adopted | Absolute Delta | Reduction | Target Bar | Status |
|---|---:|---:|---:|---:|---|---|
| Canonical lines | 1,495 | 1,124 | −371 | **−24.82%** | $\ge 20\%$ | **PASS** |
| AST statements | 759 | 667 | −92 | **−12.12%** | $\ge 10\%$ | **PASS** |
| Function count | 69 | 50 | −19 | **−27.54%** | $\ge 20\%$ | **PASS** |
| Call edges | 89 | 61 | −28 | **−31.46%** | $\ge 25\%$ | **PASS** |
| Write sites | 12 | 11 | −1 | **−8.33%** | $\le 12$ | **PASS** |
| Pure share | 0.0% | 51.9% | +51.9% | — | $\ge 50\%$ | **PASS** |

### 2. Verification Battery Status

- **Specimen Test Suite:** 108 passed, 1 skipped, 1 deselected in root; 1 passed in `packages/facdrone-research`.
- **Golden Replay Oracle:** 50/50 sequences, 317/317 operations, 0 Level 2 divergences (`sql-divergences-round3.json`).
- **Adverse Mutation Sensitivity:** 100% kill rate across all 4 controls.
- **Bounded Pure Equivalence:** 0 counterexamples.
- **Python State Boundary Tests:** 32 passed in 10.87s (`tests/test_state_boundary.py`).
- **Rust Substrate Nextest:** 241 passed in 225.9s (`cargo nextest run`).
- **Rust Apparatus Line Accounting:** Counted Rust lines = 7,173 (Ceiling = 7,173, Overage = 0).

---

## Untouched Owner WIP

All 4 preserved owner WIP files remain strictly unstaged and untouched:
- `docs/experiments/deconvolution/e3-evidence-regeneration/results/sql-divergences-round21.json`
- `docs/experiments/deconvolution/e4-lifecycle/results/sql-divergences-round21.json`
- `src/lesscode_baseline/rules/evaluate.py`
- `tests/test_rule_evaluation.py`

---

## Current State and Next Steps

1. **Self-Application Maturity Gate Certified:** The state-boundary deconvolution loop has achieved the Self-Application Maturity Gate on real-code scattered state, delivering sound −24.8% line compression and −12.1% statement compression.
2. **Ready for User Review & Context Reset:**
   - Commit adoption artifacts cleanly to `main`.
   - Ask user to clear conversation context before the next task group.
