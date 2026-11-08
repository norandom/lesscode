# Research & Discovery Log: Refinement Evidence Integrity

## Summary

This research log captures the discovery and root-cause analysis for correcting the evidence and candidate lifecycle in the refinement loop (`native/src/regenerator.rs` and `src/lesscode_baseline/regenerator.py`).
The existing suites (45/45 Rust, 59/59 Python) passed because parity testing reproduced shared defects rather than testing against contract requirements. Independent adversarial probes ([R39](../../docs/research/2026-11-08/06-refinement-correctness.md)) exposed 8 Rust failures and 5 Python failures across six root-cause groups.

### Key Findings

1. **Oracle Process & Verdict Independence (F1):** The runtime oracle adapter captured the process exit code but ignored nonzero exits and timeouts if a preexisting or prematurely written `oracle-verdict.json` had `"ok": true`. Furthermore, contradictory verdicts (where `ok: true` but `passed < total` or `failures` is non-empty) were accepted blindly. The adapter must validate process exit, timeout, and verdict schema/internal consistency.
2. **Turn Feedback Propagation (F2):** Runtime counterexamples from tier-2 execution were not propagated to subsequent turns because `turn_feedback` was scoped inside the outer turn loop (and reset to `None` at each turn start in both Python and Rust), and Rust omitted the assignment at turn completion. Scoping `turn_feedback` across turns ensures that discovered counterexamples reach the model prompt on subsequent attempts.
3. **Exact Candidate Retention & Delivery (F3):** When multiple rollouts occur within a turn, later rollouts wipe and overwrite `cand_root`. If an earlier rollout was superior, `best` recorded its metadata, but the delivered disk tree contained the later, failing rollout. The controller must snapshot the best candidate files upon selection and restore them to `cand_root` before run completion.
4. **Obligation Discharge Semantics (F4):** The existing obligation tracking marked obligations discharged simply because their string representation differed from the generic `failure_kind` string (`record.failure_kind != o`). Finding a failure was treated as discharging it! An obligation exhibited by a candidate must remain unresolved until a candidate actually passes that check.
5. **Separation of Search Stopping and Validity (F5):** Setting a low search threshold allowed candidates with only tier-1 syntax/surface scores (e.g. 0.50) to be marked `validity = true`. Search stopping is an optimization policy; validity requires full behavioral verification with all required checks passing and zero unresolved obligations.
6. **Compression Target Accounting (F6):** When static targets were omitted (`None`), the controller erroneously reported `compression_met = true`. Absent measurements must be reported as unmeasured/false.

## Architecture Pattern Evaluation

- **Minimal-Footprint In-Place Correction:** No new solver, graph framework, or generic rewrite platform is needed. Correcting the existing state transitions in `regenerator.rs` and `regenerator.py` completely resolves the defects while respecting the 7,004-line ceiling.
- **Dual-Language Parity:** Both Rust native and Python baseline implementations must receive synchronized corrections to maintain parity while ensuring independent tests prevent concealment of future defects.
