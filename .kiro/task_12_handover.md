# Task group 12 handover: refinement evidence integrity implementation and qualification

Written 2026-11-08. Group scope: execute the structured Kiro repair workflow for
`.kiro/specs/refinement-evidence-integrity`, covering requirements 1 through 6.
All specification, implementation, and verification ran in the foreground. No
subagents were dispatched, per owner rule.

## What was done

1. **Specification phase (commit `a263a61`):**
   - Requirements approved in `spec.json`.
   - Discovery research log written to `.kiro/specs/refinement-evidence-integrity/research.md`.
   - Complete technical design document generated at `.kiro/specs/refinement-evidence-integrity/design.md`.
   - Task plan generated at `.kiro/specs/refinement-evidence-integrity/tasks.md`.

2. **Task 1: Oracle process execution and verdict consistency in Rust (commit `0657bc1`):**
   - `native/src/regenerator.rs`: `RuntimeOracle::evaluate` now checks process return code from `recording::call`. Non-zero/non-1 codes (such as exit 2 or timeout 124) return infrastructure failure (`(code, Some("oracle process failed or timed out"), ...)`), preventing premature `ok: true` verdict files from masking execution failures.
   - Validated verdict payload schema and internal consistency: if `ok: true` but `failures` is non-empty or `passed < total`, the outcome is rejected as contradictory (`code 1`, divergence noted, pass fraction calculated).
   - Added unit test cases to `native/tests/regenerator.rs` (`runtime_oracle_maps_verdict_files`) covering exit 2, process timeout, contradictory counts, and positive control.

3. **Task 2: Candidate retention and filesystem delivery in Rust and Python (commit `681e95a`):**
   - `native/src/regenerator.rs`: added candidate file snapshotting (`best_files`, `turn_best_files`) on `cand_root` when rollouts improve `best`. Restores the exact selected candidate files back to `cand_root` before returning from `run_refine_loop` upon acceptance or exhaustion.
   - `src/lesscode_baseline/regenerator.py`: synchronized candidate snapshotting and filesystem restoration across rollouts and turns.
   - Verified that multi-rollout turns deliver the passing candidate on disk rather than subsequent failing candidate bytes. Added `loop_best_candidate_files_are_delivered` in `native/tests/regenerator.rs`.

4. **Task 3: Turn feedback propagation and obligation tracking in Rust and Python (commit `d99237e`):**
   - `native/src/regenerator.rs`: scoped `turn_feedback` across outer turns (moved outside the `for turn` loop) and populated it from `best` at turn completion; audited via `enforce_feedback_boundary`.
   - `src/lesscode_baseline/regenerator.py`: scoped `turn_feedback` across outer turns, removing the bug where turn 2 erased feedback set by turn 1.
   - Obligation tracking: corrected `discharged_obligations` to return empty set on failing records (`score < 1.0`). Discovered failures are active obligations and are never marked discharged upon discovery.
   - Added tests `loop_feedback_reaches_next_turn_prompt` and `loop_unresolved_obligation_retained_on_failure` in `native/tests/regenerator.rs`.

5. **Task 4: Independent outcome decisions for validity and compression in Rust and Python (commit `d923ea5`):**
   - Decoupled search stopping threshold from `validity`. `validity` requires runtime evaluator execution, tier 2 verification, `score >= 1.0`, zero failures, and zero unresolved obligations.
   - Fixed `compression_met`: requires explicit non-empty static targets and zero target gaps; returns false (unmeasured) when targets are absent.
   - Added tests `loop_low_threshold_cannot_establish_validity` and `loop_missing_targets_cannot_establish_compression` in `native/tests/regenerator.rs`.

6. **Task 5: End-to-end verification and qualification (commit `37f21c4`):**
   - Executed full R39 adversarial probe battery in Rust and Python.
   - Executed full native and reference regenerator test batteries.
   - Verified canonical line count limits. Spec `refinement-evidence-integrity` marked completed.

## Fresh evidence

- **R39 probe runner (`docs/experiments/deconvolution/refinement-correctness/run.ps1`):**
  - Native Rust probe crate: **9 passed, 0 skipped** in 7.285s (was 1 pass, 8 failures).
  - Reference Python suite: **5 passed** in 7.45s (was 5 failures).
  - Exit code: **0** (all 14 baseline probe defects now pass).
- **Existing regenerator suites:**
  - Native Rust (`native/tests/regenerator.rs`): **50 passed, 0 skipped** in 22.504s (expanded from 45 with 5 new regression tests).
  - Python baseline (`tests/test_regenerator.py`): **59 passed** in 6.41s.
- **Apparatus ceiling:**
  - `native/src/regenerator.rs`: 2,752 physical lines (net +38 lines). Well within the R38 7,004 canonical line ceiling.
- **Untouched owner WIP:**
  - `docs/experiments/deconvolution/e3-evidence-regeneration/results/sql-divergences-round21.json`
  - `docs/experiments/deconvolution/e4-lifecycle/results/sql-divergences-round21.json`
  - `src/lesscode_baseline/rules/evaluate.py`
  - `tests/test_rule_evaluation.py`
  All four remain unstaged and unchanged.

## Current state and next steps

The refinement evidence integrity defects identified in R39 are resolved and verified in both Rust and Python. The controller now:
- Binds acceptance and delivery to the exact code evaluated.
- Rejects process timeouts, failed processes, and contradictory verdicts.
- Carries runtime counterexamples causally to subsequent turn prompts.
- Retains discovered counterexamples as unresolved obligations.
- Separates search thresholds, behavioral validity, and compression measurements.

As emphasized in concept.md, a truthful and trustworthy evaluator provides honest decisions, but does not by itself prove useful compression on new workloads. Future compression efficacy trials must begin with a concrete removal hypothesis, a frozen observation matrix, and whole-replacement measurements.

Slice B adoption remains unscheduled.

Per owner rule, please clear the conversation context before starting the next task group.
