# R35: U5 disposition — accept the Python stage-1 negative; Rust re-measure gate before Slice B adoption

- Report date/time and timezone: 2026-11-08, decision recorded ~08:15Z (writing
  date = decision date; no separate execution interval)
- Execution interval: not run — owner disposition on retained R34 measurements
- Status: completed (decision recorded); Rust re-measurement pending
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md)
  `Req 7.8`, `7.10`; recorded as Amendment 3 (requirements.md) and the U5
  disposition amendment ([prototype-gate.md](../../../.kiro/specs/constraint-guided-compression/prototype-gate.md));
  tasks.md Deferred section updated
- Hypothesis ID and predecessor: [R34](01-g1r-gate-and-live-recovery.md)
  (U5 fails as measured: 3,783 vs 2,270 canonical lines)
- Type: retrospective (owner decision on retained evidence; no new measurement)

## Decision

The owner accepts U5 **as failing on the Python stage-1 prototype substrate**.
The frozen ceiling (2,270 canonical lines, 20% reduction on the expanded
baseline apparatus stratum $B_{\text{app}}^{\text{expanded}} = 2,838$) is
unchanged; the `Req 7.8` negative is retained without weakening. No slimming
pass and no ceiling amendment is authorized at this stage.

What this permits next: the Rust port becomes the substrate on which U5 is
re-measured. **Slice B production adoption (`Req 7.10`) stays withheld until a
Rust-port re-measurement of U5 under the same expanded-baseline accounting
passes the frozen ceiling**, or a further dated owner decision records a
different disposition. Compression adoption is decided separately from
candidate validity; nothing in this decision converts the Python prototype's
3,783-line result into a savings claim.

## Hypothesis and falsification

Working frame (owner, 2026-11-07/08): Python is the stage-1 prototype; Rust is
the target substrate. The Python apparatus overage (3,783 vs 2,270) does not
by itself falsify the mechanism — it is measured on the prototype substrate,
where the loop apparatus (regenerator, receipts, admission) is expressed in
the same Python it measures.

Falsifiable prediction recorded for the next gate: if the Rust port's
consolidated replaceable apparatus — measured under the same canonical-line
accounting against the same expanded baseline — still exceeds 2,270 canonical
lines, then the "smaller replaceable representation" claim fails on the target
substrate too, and Slice B adoption requires a new dated owner decision (slim
or amend); it must not proceed on a weakened ceiling. Amendment dated
2026-11-08; targets and budgets unchanged from the frozen checklist.

## Frozen inputs and method

No new execution. Inputs are the retained R34 measurements: canonical-line
count 3,783 for the consolidated `src/lesscode_baseline/{recording,surface,
measures,regenerator}.py` + `native/src/{admission,receipt}.rs` apparatus;
frozen ceiling 2,270 from the 2026-11-07 checklist (prototype-gate.md); all
measurement commands, receipts and admission checks as recorded in R34
(commits `1878947`, `9d9cf72`, `227e12b`).

## Evidence and results

- R34 (same day): U5 measured 3,783 vs 2,270 canonical lines — failed; all
  deterministic G1-R checklist items passed; live weaker-model recovery (B-2)
  reached R=1.0 within frozen budgets.
- This entry adds no measurement. The disposition itself is the recorded
  outcome: accept negative on Python; re-measure on Rust before adoption.
- Apparatus note (defect 6 of the G1-R live program, found in 7.1, fixed in
  this change set): the concise-output recovery directive emitted by
  `next_rollout_params` on score-0.00 rollouts was filtered out by
  `run_refine_loop` (`k in base_params`) and never dispatched to the model.
  Fixed by prompt-level dispatch (appended to the user prompt on the next
  attempt, recorded in `preregistered_params`), with the per-attempt params
  view frozen in receipts; dated comments in `regenerator.py`; regression
  test in `tests/test_regenerator.py`. No retained run or receipt was altered.

## Limits and interpretation

The accepted negative is substrate-specific: it bounds the Python prototype
apparatus, not the Rust target. It is not proof that the mechanism cannot meet
the ceiling, and equally not evidence that it can — the Rust re-measurement is
the deciding observation. Unknown is not pass: no transfer of the Python line
count to Rust is assumed in either direction.

## Tool/spec consequence and next gate

Recorded: requirements.md Amendment 3; prototype-gate.md U5 disposition
amendment (amendments newest-first); tasks.md Deferred section now names the
Rust-port U5 re-measurement gate as the Slice B precondition. Next gate: the
Rust port re-measures U5 under the same accounting; pass unlocks Slice B
adoption planning, fail returns to the owner with slim/amend options. The
concise-output directive fix is apparatus maintenance inside Slice A
boundaries; it does not alter any frozen target or the G1-R outcome.

## COBESY check

Outcome first (accept negative, gate Rust re-measure); replayable from R34
receipts and this entry's links; disproof explicit (Rust overage falsifies
transfer); negative result safe to report; facts (measurements) separated from
interpretation (substrate rationale); no inflated claims — no savings adopted.
