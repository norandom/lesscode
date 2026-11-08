# Task group 14 handover: Experiment E5 state-boundary deconvolution trial — breaking the density trap

Written 2026-11-08. Group scope: prepare, execute, and qualify **Experiment E5: State-Boundary Deconvolution Trial** on the Facdrone `lifecycle` monitor group (7 files, 1,495 baseline canonical lines), following Option 1 disposition and owner ceiling amendment R41 (`3387902`).
All execution, measurement, candidate refinement, and verification ran in the foreground. No subagents were dispatched, per owner rule.

---

## What was done

1. **E5 Preregistered Plan & Falsifiable Framing:**
   - Preregistered `docs/experiments/deconvolution/e5-state-boundary-lifecycle/plan.md` using COBESY falsifiable structure with fixed targets T1–T7 and structural contract S before candidate execution.
   - Identified the **Density Trap** observed in E4 (structural consolidation of −24.6% functions and −29.2% edges yielded only −6.0% lines) and established hard bars: $\ge 20\%$ line reduction ($\le 1,196$ lines) and $\ge 10\%$ statement reduction ($\le 683$ statements).

2. **Verification & Evaluation Apparatus:**
   - Implemented `docs/experiments/deconvolution/e5-state-boundary-lifecycle/e5_evaluator.py`:
     - Built four adverse control tests: E3 `AuditedAbsence` (failure-side post-conditions), E4 F3 `Buffer_Drop` (stream preservation), E4 F1 `Silent_None` (bounded pure-core equivalence), and E4 F7 `Error_Order` (mutation sensitivity gate).
     - Integrated `measures.measure_written` to measure exact baseline and candidate trees across the 7 scope files.
     - Integrated `DensityGate` logic evaluating `structurally_consolidated` vs `compression_met`.
     - Counted production Rust substrate apparatus canonical lines against the amended R41 ceiling (7,173 lines).
   - Authoring independent acceptance and replay runner:
     - Implemented `run_e5.py` and `test_e5_replay.py` directing output strictly to `e5-state-boundary-lifecycle/results/sql-divergences-round<N>.json`, preserving all prior experiment directories untouched.

3. **Candidate Execution & Refinement Loop:**
   - Round 0: Evaluated frozen specimen baseline (`BASELINE_ROUND_0_PASS`, 1,495 lines, 759 statements, 69 functions, 89 edges, 12 write sites) and confirmed apparatus canonical line accounting (7,173 / 7,173 lines, 0 overage).
   - Round 1: Applied whole-responsibility streamlining across all 7 scope files in `.runs/e5-state-boundary/cand-e5`. Unit tests passed (108 passed, 1 skipped) and golden replay oracle passed (50/50 sequences, 317/317 operations, 0 Level 2 divergences). Lines reached 1,142 (−23.6%) and statements reached 682 (−10.1%), but function count was 66 (target $\le 55$) and call edges 85 (target $\le 66$).
   - Round 2: Consolidated and inlined internal micro-functions (`_finish`, `_recorded`, `_row` into `lifecycle_monitor_runner.py`; `_state`, `_baseline`, `_latest_basis`, `_flag`, `_reading`, `_latest_and_not_run` into `lifecycle_projection.py`; `_lookback` into `lifecycle_evidence.py`; `standing` into `lifecycle_root.py`; `_applied` and `_refuse_act` into `lifecycle_store.py`; `_window` into `cli/lifecycle_group.py`). Preserved test monkeypatch seams (`_cited_baseline`, `_pod_of`).
   - Round 2 Evaluation: Exited 0 with **PASS** on all targets!

4. **Formal Research Documentation:**
   - Authored `docs/research/2026-11-08/09-e5-state-boundary-deconvolution-lifecycle.md` ([R42](../docs/research/2026-11-08/09-e5-state-boundary-deconvolution-lifecycle.md)) with COBESY falsifiable structure and evidence links.
   - Updated research index `docs/research/README.md`.
   - Updated `.kiro/specs/state-boundary-deconvolution/spec.json` with trial results and evidence pointers.

---

## Fresh Evidence

### 1. Target Evaluation Results (All Targets HELD)

| Prediction | Target Bar | Observed Value | Status |
|---|---|---|---|
| **T1: Refinement Convergence** | $\le 3$ turns | Turn 2 | **HELD** |
| **T2: Whole-Responsibility Lines** | $\le 1,196$ lines ($\ge 20\%$) | **1,124 lines (−24.8%)** | **HELD** |
| **T3: AST Statement Reduction** | $\le 683$ stmts ($\ge 10\%$) | **667 stmts (−12.1%)** | **HELD** |
| **T4: Function De-fragmentation** | $\le 55$ funcs ($\ge 20\%$) | **50 funcs (−27.5%)** | **HELD** |
| **T4: Call Edge De-fragmentation**| $\le 66$ edges ($\ge 25\%$) | **61 edges (−31.5%)** | **HELD** |
| **T5: Adverse State Mutation Sensitivity** | 100% kill rate | **100% kill rate (4/4 controls)** | **HELD** |
| **T6: Bounded Pure Core Equivalence** | 0 counterexamples | **0 counterexamples** | **HELD** |
| **T7: Apparatus Volume Stability** | $\le 7,173$ canonical lines | **7,173 lines (0 overage)** | **HELD** |
| **S: Structural Contract** | Write sites $\le 12$, Pure share $\ge 50\%$ | **11 write sites, Pure verified** | **HELD** |

### 2. Full Test Suite Verification

- **Golden Replay Oracle (`run_e5.py`):** Exit 0, 50/50 sequences replayed, 317/317 operations verified, 0 Level 2 divergences (`sql-divergences-round2.json`).
- **Specimen Test Suite:** 108 passed, 1 skipped, 1 deselected in root; 1 passed in `packages/facdrone-research`.
- **State Boundary Verification Suite (`pytest -n auto tests/test_state_boundary.py`):** 32 passed in 10.87s.
- **Rust Substrate Test Suite (`cargo nextest run --manifest-path native/Cargo.toml`):** 241/241 passed in 225.9s (including `e4_three_turn_replay`, `e3_double_record`, `e4_double_record`, and `frozen_traces_replay`).

---

## Untouched Owner WIP

All 4 preserved owner WIP files remain strictly unstaged and untouched:
- `docs/experiments/deconvolution/e3-evidence-regeneration/results/sql-divergences-round21.json`
- `docs/experiments/deconvolution/e4-lifecycle/results/sql-divergences-round21.json`
- `src/lesscode_baseline/rules/evaluate.py`
- `tests/test_rule_evaluation.py`

---

## Current State and Next Steps

1. **Density Trap Conquered:** The combination of state-boundary enforcement (StateFrame failure conditions, stream monitors, PureVerifier AST checks) and whole-responsibility restructuring proved that code compression $\ge 20\%$ is achievable while simultaneously reducing functions by $27.5\%$ and call edges by $31.5\%$, with zero semantic or state divergence.
2. **Ready for User Review & Context Reset:**
   - Commit changes to `main` (preserving the 4 owner WIP files).
   - Ask user to clear context before launching the next task group.
