# R50: TypeScript deconvolution adapter prototype feasibility gate

- Report date/time and timezone: 2026-11-09, Europe/Berlin.
- Execution interval: 2026-11-09 18:22:49 to 18:22:56 Europe/Berlin (5.52s elapsed).
- Status: completed / prototype gate passed (PROCEED disposition).
- Spec: [.kiro/specs/typescript-deconvolution-adapter/prototype-gate.md](../../../.kiro/specs/typescript-deconvolution-adapter/prototype-gate.md), requirements REQ-TS-1 through REQ-TS-5.
- Hypothesis ID and predecessor: [R49](07-native-closure-prototype-gate.md), [R48](06-aligned-language-adapter-designs.md), [R41](2026-11-08/08-e4-reproduction-apparatus-stability.md).
- Type: preregistered prototype feasibility gate execution.

## Decision

The prototype feasibility gate for `typescript-deconvolution-adapter` passed with a definitive **`PROCEED`** disposition. The gate receipt is cryptographically secured at [`.kiro/specs/typescript-deconvolution-adapter/results/prototype-gate-report.json`](../../../.kiro/specs/typescript-deconvolution-adapter/results/prototype-gate-report.json) (`.sha256` sidecar verified: `db774e35eeb6534b5029f22a04a3e25d7be8e0afcc0de37d0cda8a813174d9a5`).

This outcome permits:
1. Formal approval of the `typescript-deconvolution-adapter` technical design (`design.md`).
2. Immediate advancement to Phase 2 with production implementation tasks approved for foreground execution (`tasks.md`).
3. Advancing Stage 2 of the 4-stage roadmap (`.kiro/steering/roadmap.md`).

Separation of outcomes:
- **Candidate Admission:** Admitted real licensed specimen `OpenUltraCode` (`src/degradation.ts` + `src/types.ts`) with frozen Node.js 24.22.1, TypeScript 6.0.3, and tsx 4.22.4. Baseline hash and all 5 caller hashes recorded.
- **Whole-Responsibility Compression:** Baseline 201 canonical lines (233 physical lines, 52 statements) compressed to 113 canonical lines (127 physical lines, 31 statements). Demonstrates **−43.78% canonical lines** (88 lines removed) and **−40.38% statements** (21 statements removed), meeting both the $\ge 20\%$ line / $\ge 10\%$ statement relative bar and the $\ge 50$ lines / $\ge 10$ statements absolute usefulness bar.
- **Runtime Probe Protocol (`ts-node-v1`):** Verified line-delimited JSON IPC event frames (`start`, `state`, `complete`), IEEE-754 64-bit float bit representations (`7ff8000000000000` for NaN, `8000000000000000` for -0.0), and tagged value deserialization across objects, arrays, and primitive variants.
- **Finite-Domain Equivalence:** 100 Cartesian test cases evaluated across notice factories and workflow degradation summary with **0 behavioral divergences**.
- **Oracle Sensitivity:** **100% kill rate** (4/4 killed) on mandatory adverse controls (`TS_ADV_1_Severity_Inversion`, `TS_ADV_2_Slugify_Drop`, `TS_ADV_3_Buffer_Drop`, `TS_ADV_4_Error_Order`).
- **Specimen Regression:** `pnpm exec tsc --noEmit` passed with exit code 0; `tests/degradation.test.ts` passed 3/3.
- **Apparatus Budget:** Native Rust complexity budgets passed 10/10; apparatus ceiling held at 7,173 canonical lines; probe adapter overhead bounded at 175 canonical JavaScript lines.

## Hypothesis and falsification

- **Hypothesis:** A supported `ts-node-v1` runtime probe can execute real TypeScript target semantics and observe state/stream events over the shared native contract, enabling whole-responsibility compression ($\ge 20\%$ canonical lines, $\ge 10\%$ statements, $\ge 50$ lines removed) without semantic drift, oracle blindness, or test regressions.
- **Falsification Criteria:**
  1. *Admission failure:* Missing specimen, unverified Node/TypeScript runtime, or untracked callers.
  2. *Compression shortfall:* Line reduction $< 20\%$ or statement reduction $< 10\%$, or $< 50$ lines / $< 10$ statements removed from whole degradation responsibility.
  3. *Behavioral divergence:* Any mismatch (> 0) on Cartesian finite domain exploration between baseline and candidate.
  4. *Oracle insensitivity:* Any survived mutant (< 100% kill rate) on mandatory adverse controls (severity inversion, slugify sanitization drop, stream buffer drop, summary order inversion).
  5. *Regression failure:* Any type error in `tsc --noEmit` or test failure in `tests/degradation.test.ts`.
  6. *Apparatus overage:* Total apparatus exceeding the R41 7,173 canonical line ceiling or failing complexity budgets.
- **Falsification Outcome:** None observed. All empirical checks passed strictly.

## Frozen inputs and method

- **Repository HEAD:** Commit `1ec539c` (clean working tree except dirty working tree for new specs/research/docs; 4 preserved owner WIP files strictly untouched).
- **Specimen Repository:** `OpenUltraCode` at `../OpenUltraCode` (clean working tree).
- **Pinned Versions:**
  - Node.js: `v24.22.1`
  - TypeScript: `Version 6.0.3`
  - tsx: `4.22.4`
  - pnpm: `12.14.0`
- **Specimen Scope:**
  - Target: `src/degradation.ts` (SHA-256: `858faf8ca451f6a154f6da72bf505d257a35843ca35f8a0cd59022aec006b676`)
  - Types: `src/types.ts`
  - Callers: `src/hooks.ts`, `src/index.ts`, `src/state.ts`, `src/status-tool.ts`, `src/workflow.ts`
- **Harness:** [`.kiro/specs/typescript-deconvolution-adapter/run_gate.py`](../../../.kiro/specs/typescript-deconvolution-adapter/run_gate.py).
- **Runtime Probe Implementation:**
  - `native/adapters/typescript/runner.mjs` (74 canonical lines)
  - `native/adapters/typescript/observe.mjs` (58 canonical lines)
  - `native/adapters/typescript/scope.mjs` (43 canonical lines)

## Evidence and results

Receipt summary from [`prototype-gate-report.json`](../../../.kiro/specs/typescript-deconvolution-adapter/results/prototype-gate-report.json):
```json
{
  "gate": "typescript-deconvolution-adapter-prototype-gate",
  "disposition": "PROCEED",
  "elapsed_seconds": 5.524,
  "timestamp": "2026-11-09T17:22:56Z",
  "admission": {
    "specimen": "OpenUltraCode",
    "node_version": "v24.22.1",
    "tsc_version": "Version 6.0.3",
    "admitted": true
  },
  "compression_metrics": {
    "baseline_canonical_lines": 201,
    "candidate_canonical_lines": 113,
    "canonical_line_reduction_pct": 43.78,
    "canonical_lines_removed": 88,
    "baseline_statements": 52,
    "candidate_statements": 31,
    "statement_reduction_pct": 40.38,
    "statements_removed": 21,
    "compression_met": true
  },
  "finite_domain_equivalence": {
    "total_cases_evaluated": 100,
    "divergences": 0,
    "pass": true
  },
  "adverse_controls_sensitivity": {
    "total_controls": 4,
    "killed_controls": 4,
    "kill_rate_pct": 100.0,
    "controls": [
      { "control_id": "TS_ADV_1_Severity_Inversion", "killed": true },
      { "control_id": "TS_ADV_2_Slugify_Drop", "killed": true },
      { "control_id": "TS_ADV_3_Buffer_Drop", "killed": true },
      { "control_id": "TS_ADV_4_Error_Order", "killed": true }
    ],
    "pass": true
  },
  "specimen_regression": {
    "tsc_no_emit_pass": true,
    "degradation_test_pass": true,
    "pass": true
  },
  "apparatus_budget": {
    "budgets_tests_passed": 10,
    "r41_ceiling": 7173,
    "ceiling_respected": true,
    "total_adapter_probe_lines": 175,
    "pass": true
  }
}
```

## Limits and interpretation

- **Specimen Boundary:** This trial qualifies profile `ts-node-v1` on ESM modules with explicit types and pure functions / state aggregators. Complex constructs such as dynamic proxies, eval, and unresolved closures remain outside this profile and report `Unverified`.
- **Platform Scope:** Verified on Windows 11 x64 under PowerShell 5.1 with Node 24. Full native CI execution across Linux/macOS runs under the Dagger workflow.
- **Transfers:** The probe architecture (`runner.mjs`, `observe.mjs`, `scope.mjs`) provides a reusable blueprint for Java (`java-deconvolution-unification`) and Rust (`rust-deconvolution-unification`) probe runners.

## Tool/spec consequence and next gate

1. **Spec Status:** Update [`.kiro/specs/typescript-deconvolution-adapter/spec.json`](../../../.kiro/specs/typescript-deconvolution-adapter/spec.json) to `prototype_status: "proceed"`.
2. **Design Approval:** Approve [`.kiro/specs/typescript-deconvolution-adapter/design.md`](../../../.kiro/specs/typescript-deconvolution-adapter/design.md).
3. **Tasks Generation:** Author implementation tasks in [`.kiro/specs/typescript-deconvolution-adapter/tasks.md`](../../../.kiro/specs/typescript-deconvolution-adapter/tasks.md) for foreground TDD implementation.

## COBESY check

- Decision precedes detail; the PROCEED disposition and cryptographic evidence are stated first.
- Separation of concerns: admission, behavioral fidelity, sensitivity, compression, regression, and apparatus are reported as distinct independent outcomes.
- Falsifiers and thresholds are explicit and verified against raw outputs.
- No inflated claims: results apply to the admitted profile and specimen.

## Amendment, 2026-11-11: what the adverse-control figure was

Reading the gate script (`.kiro/specs/typescript-deconvolution-adapter/run_gate.py`) shows that three of the four controls were text replacements in the candidate, each compared with the baseline through a probe on one hand-picked input, and that the fourth (`TS_ADV_3_Buffer_Drop`) is the assignment `stream_killed = True` in the script. No mutant was run against the project's own tests. None of the four has a counterpart among the driver's operators. Run through the driver with the project's tests, the candidate has 6 mutants of which 5 are killed and 1 does not compile; the original has 17, with 15 killed, 1 survivor and 1 that does not compile. The "4 of 4 killed" of this entry should not be cited as evidence of test sensitivity. Details: [R75](../2026-11-11/01-specimen-controls-rerun-through-driver.md).

