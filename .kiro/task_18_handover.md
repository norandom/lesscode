# Task group 18 handover: Report envelope adoption & dual-specimen maturity gate certification

Written 2026-11-09. Group scope: execute the formal **Self-Application Maturity Gate & Capability Adoption** for the deconvoluted Facdrone **Report Envelope & Snapshot Store** group (6 files, 1,492 lines vs 1,938 baseline), certify dual-specimen replication, emit adoption artifacts, author formal research log R46, and update spec metadata.
All execution, measurement, diff generation, and verification ran in the foreground. No subagents were dispatched, per owner rule.

---

## What was done

1. **Adoption Target Establishment:**
   - Established the clean, isolated adopted specimen tree at `.runs/e6-adopted-report-envelope/`.
   - Verified that `.runs/e6-adopted-report-envelope/` differs from the frozen baseline `.runs/e2-facdrone/source/` in exactly the 6 scope files (0 extraneous diffs).
   - Preserved `.runs/e2-facdrone/source/` read-only as the canonical baseline reference snapshot.

2. **Unified Diff & Hash Integrity Generation:**
   - Computed baseline and adopted SHA-256 digests for all 6 scope files.
   - Generated the 1,755-line unified diff at [`docs/experiments/deconvolution/e6-report-envelope/results/e6-adopted-report-envelope.diff`](../docs/experiments/deconvolution/e6-report-envelope/results/e6-adopted-report-envelope.diff) (SHA-256: `9b8b5456f28806915a689f2343969ff62414a15fcafc6387a3a4dc5f7d242756`).
   - Recorded full metrics and evidence in [`docs/experiments/deconvolution/e6-report-envelope/results/e6-adoption-report.json`](../docs/experiments/deconvolution/e6-report-envelope/results/e6-adoption-report.json) (SHA-256: `9463d936616db4118f705ace31f0aa019165b9d94db071d6218b91430c9c4d9b`) and its `.sha256` sidecar.

3. **End-to-End Acceptance & Verification on Adopted Tree:**
   - Executed `docs/experiments/deconvolution/e6-report-envelope/run_e6.py .runs/e6-adopted-report-envelope`:
     - `packages/facdrone-reporting`: exit 0 in 25.76s.
     - `packages/facdrone-kernel`: exit 0 in 9.77s.
     - Root unit suites: exit 0 in 12.12s.
     - Final exit: 0 (46/46 passed with 0 regressions).
   - Executed `docs/experiments/deconvolution/e6-report-envelope/e6_evaluator.py --candidate .runs/e6-adopted-report-envelope`:
     - 4/4 adverse controls killed (`E6_C1_MissingFailureState`, `E6_C2_BufferTruncation`, `E6_C3_OutOfDomainPurity`, `E6_C4_ErrorOrderInversion`).
     - 0 bounded pure-core counterexamples over $[-10, 10]$ and edge cases.
     - Rust line accounting: 7,173 canonical lines (0 overage against R41 ceiling).
     - Outcome: `structurally_consolidated=True, compression_met=True`.

4. **Dual-Specimen Replication & Research Synthesis:**
   - Certified the **Dual-Specimen Self-Application Maturity Gate**, confirming that State-Boundary Deconvolution breaks the Density Trap systematically across differing application domains (lifecycle event loop vs report persistence & cryptographic serialization).
   - Authored formal research log [R46](../docs/research/2026-11-09/04-e6-report-envelope-adoption-maturity-gate.md).
   - Updated [`docs/research/README.md`](../docs/research/README.md) indexing R46 as Latest.
   - Updated [`.kiro/specs/state-boundary-deconvolution/spec.json`](state-boundary-deconvolution/spec.json).

---

## Fresh Evidence

### 1. Dual-Specimen Empirical Replication

| Specimen Subsystem | File Count | Baseline Lines | Adopted Lines | Line Reduction (%) | Statement Reduction (%) | Function Reduction (%) | Call Edge Reduction (%) | Confined Writes |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **Specimen 1: Lifecycle Monitors (E5)** | 7 | 1,495 | 1,124 | **−24.82%** | **−12.12%** | **−27.54%** | **−31.46%** | 11 |
| **Specimen 2: Report Envelope (E6)** | 6 | 1,938 | 1,492 | **−23.01%** | **−10.49%** | **−38.24%** | **−30.34%** | 4 |
| **Combined Subsystems Total** | **13** | **3,433** | **2,616** | **−23.80%** | **−11.20%** | **−33.92%** | **−30.93%** | **15** |

### 2. Verification Battery Status on Adopted Tree

- **Specimen Acceptance Suites:** 46/46 passed with zero regressions.
- **Adverse Mutation Sensitivity:** 100% kill rate across all 4 controls (missing failure state, buffer truncation, out-of-domain purity, error order inversion).
- **Bounded Pure Equivalence:** 0 counterexamples across $[-10, 10]$ and edge cases.
- **State Write Sites:** Confined strictly to 4 imperative shell locations in `reporting_request_store.py`.
- **Rust Substrate Volume:** Counted Rust substrate = 7,173 canonical lines (Ceiling = 7,173, Overage = 0).

---

## Untouched Owner WIP

All 4 preserved owner WIP files remain strictly unstaged and untouched:
- `docs/experiments/deconvolution/e3-evidence-regeneration/results/sql-divergences-round21.json`
- `docs/experiments/deconvolution/e4-lifecycle/results/sql-divergences-round21.json`
- `src/lesscode_baseline/rules/evaluate.py`
- `tests/test_rule_evaluation.py`

---

## Current State and Next Steps

1. **Commit Task Group 18 Artifacts:** Stage and commit adoption report, diff, research log R46, spec update, and handover to `main`.
2. **Context Reset:** In accordance with owner rule (6), ask the user to clear the conversation context before the next task group.
