# R34: G1-R gate discharged — deterministic checklist green, live weaker-model recovery reaches R=1.0, U5 fails as measured

- Report date/time and timezone: 2026-11-08, UTC (evidence written 2026-11-07T19:47Z through 2026-11-08T08:00Z)
- Execution interval: 2026-11-07 18:30Z – 2026-11-08 08:00Z (tasks 6.1, 6.2, 7.1 of `constraint-guided-compression`)
- Status: completed (U5 outcome: fail-as-measured, owner disposition open)
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/tasks.md) tasks 6.1, 6.2, 7.1; requirements 5.8, 7.2, 7.7–7.9, 8.1–8.6
- Hypothesis ID and predecessor: H10 (Slice B) gated by `G1-R`; predecessor [R33](../2026-11-07/07-dspy-probabilistic-turn-refinement-design.md)
- Type: preregistered (gate checklist frozen in [prototype-gate.md](../../../.kiro/specs/constraint-guided-compression/prototype-gate.md) L18–29 before execution)

## Decision

The G1-R spike requirements are discharged for every deterministic checklist item,
and the live/replay recovery check is satisfied twice: deterministically by the
frozen E4 transition replay, and live by run `B-2` in which a weaker model
(`claude-haiku-4-5`) recovers from a recorded Tier-1 `out_of_scope` failure
(R=0.20) to full acceptance (R=1.0, oracle 7/7) within one turn
(`max_turns<=4`, `max_static_retries<=3`), receipts and cost retained.
**U5 fails as measured** (3,783 canonical lines vs the frozen ceiling 2,270);
per Req 7.8 the negative result is retained without weakening targets, and the
disposition (accept the apparatus as stage-1 prototype, slim it, or amend the
ceiling) is an open owner decision. Extraction, detection and loop mechanics
are separated from compression: this entry claims loop-mechanism feasibility
only, not a net-reduction claim. Framing per owner direction: Python is the
stage-1 prototype substrate; Rust remains the target substrate.

## Hypothesis and falsification

Frozen in `prototype-gate.md` L18–29 (2026-11-07, before execution): six adverse
controls must fail closed in Tier 1 with zero runtime invocations; the recorded
E4 transition must pass Tier 1 in under two seconds per check; a live
non-deterministic or weaker-model run must recover from a Tier-1/Tier-2 failure
to R=1.0 inside the frozen budgets; `k=2` no-progress must halt with the best
trajectory; replaceable apparatus must stay within the 20% ceiling
(<=2,270 canonical lines vs `B_app_expanded` = 2,838); every receipt must be
admission-verifiable via `bindings.json`. Any control passing Tier 1, any
runtime invocation during a control, any replay check >= 2 s, any live
acceptance outside budget, or any unverifiable receipt would reject the Slice B
apparatus. U5 was expected to fail at the 6.1 measurement (predicted ~3,783
vs 2,270) and is reported as measured, not excused.

## Frozen inputs and method

- Deterministic driver: `docs/experiments/deconvolution/probabilistic-refine/g1r_spike.py`
  (commit `1878947`) over frozen E4 turns (`e4-turn1..3` bodies untracked by
  convention, sha256 sidecars + `result.json` tracked), Contract B recorder
  oracle `e4_oracle.py`, and the frozen facdrone source copy
  (`.runs/e2-facdrone/source`, 184.5 MB / 4,851 files).
- Live runner: `docs/experiments/deconvolution/probabilistic-refine/g1r_live.py`
  (commit `9d9cf72`); frozen small Contract-B tasks A (single module, 6 symbols,
  6-case golden oracle) and B (two cross-importing modules, 7 cases); weaker
  model `claude-haiku-4-5` served by the same endpoint as the frozen E4 turns
  (`claude-opus-5-5`); SDK `anthropic` 2.6.0 via `PYTHONPATH=.runs/regen-extras`;
  budgets preregistered in each receipt (`max_turns=4`, `max_static_retries=3`,
  `no_progress_limit=2`, `threshold=1.0`).
- Verification (7.1): `pytest -n 4` from `D:\Source\lesscode` with
  `.runs/e2-facdrone/venv` (pytest 9.0.3, xdist 3.8.0); `cargo nextest run` in
  `native/`; `lesscode-core budgets --config native/complexity-budgets.json
  --root native/src`; `lesscode-core admission` over every emitted
  `bindings.json`; sha256 re-verification of every `*.sha256` sidecar under
  `docs/experiments/deconvolution`; `git status` for tracked-file immutability.

## Evidence and results

- Deterministic checklist (task 6.1, record
  `probabilistic-refine/results/g1r-record.json`, written 2026-11-07T19:47:36Z):
  six Tier-1 controls fail closed with zero runtime calls; E4 replay t1 0.001 s
  (score 0.00, stop_reason), t2 0.032 s (0.40, `labeled_reason` caught under
  Contract B recorder tags), t3 accepted with Tier-2 replay 1.00 at 0.06 s;
  `k=2` exhaustion retains best; admission verifies all six receipts
  (`exit 0` each). U5 measured 3,783 vs ceiling 2,270 — recorded FAIL.
- Live runs (task 6.2, record `probabilistic-refine/results/g1r-live-record.json`,
  receipts under `results/live/A-1..A-8, B-1..B-3`): 11 runs, all retained
  including honest negatives. `B-2` is the criterion run: attempt 1 fails
  Tier-1 `out_of_scope` (0.20, paths outside scope), attempt 2 in the same turn
  accepts at R=1.0 (2 calls, 4,811 input / 2,309 output tokens, oracle 7/7).
  Every receipt admission-verified (`exit 0`); fresh 7.1 admission sweep
  re-verified all 11 (`admission-7_1.json` receipts).
- Native self-checks (task 7.1, 2026-11-08): `cargo nextest run` 123/123
  passed; `lesscode-core budgets` exit 0 (2 within_budget, 7 disposed,
  0 exceeded/unlisted/missing; receipt
  `probabilistic-refine/results/budgets-selfcheck/budgets.json`).
- Full suite (task 7.1): `pytest -n 4` — 648 passed, 4 failed, 11 skipped,
  473 subtests, 685 s. The 4 failures (test_config, test_metrics, test_snapshot
  x2) reproduce identically with the 6.1/6.2 changes stashed — pre-existing
  xdist-specific flakes, not regressions. The 6 documented Windows
  process/timeout failures remain listed as non-regressions (implementation
  notes, tasks.md).
- Hash check (task 7.1): `git status` shows zero tracked modifications under
  `docs/experiments/deconvolution` (untracked bodies/raw dirs follow the
  repository convention); 53 sha256 sidecar pairs verify; the 14 mismatches and
  4 missing targets are pre-existing states of older experiment dirs
  (t4 turns, e1b/e1c goldens, native-prototype s10, e4 golden-e4) — enumerated,
  not repaired, because rewriting historical evidence would itself mutate the
  frozen record (Req 8.4).
- Apparatus defects found by the live run and fixed with dated comments
  (commit `9d9cf72`): SDK 2.6.0 `Messages.stream()` accepts no `temperature`
  and the API rejects it beside `output_config` (temperature now dispatchable
  only on openai-compatible endpoints); `claude-haiku-4-5` rejects
  `output_config.effort` (`request_kwargs` omits it on `effort=None`; the
  runner adapter maps effort to None); system-prompt rule 4 taught a
  three-segment structure grammar the checker never accepted (corrected to
  `<basename>::<qualname>`); `_static_feedback` now forwards structure-rung
  `why`/`write_sites`; `measures._check_structure` normalizes
  scope-path-qualified declarations.

## Limits and interpretation

- U5 measures the prototype's replaceable apparatus in Python stage-1 form
  (recording 372 + surface 281 + measures 558 + regenerator 1,239 = ~2,450 py
  plus 1,333 frozen Rust canonical lines); the 3,783 total exceeds the frozen
  2,270 ceiling. This is a property of the measured apparatus, not of the
  method; no target was weakened. The Rust-target port may change the
  accounting materially and must be re-measured, not assumed.
- The live recovery evidence is one criterion run on small synthetic scopes
  with a golden-file oracle; it demonstrates the feedback loop closes, not
  that recovery generalizes to repository-scale scopes or runtime-heavy oracles.
- Known gap (recorded for follow-up): `next_rollout_params`' concise-output
  directive is filtered out by `run_refine_loop` and never dispatched.
- The 4 xdist flakes and 6 process failures bound "every suite green": green
  here means no new failures versus the documented pre-existing set.
- Pre-existing sidecar mismatches in older dirs are outside this spec's
  evidence and were not investigated further.

## Tool/spec consequence and next gate

Slice B's mechanism survived its preregistered spike: detection, bounded
refinement and admission are demonstrated end-to-end, live and deterministic.
Production claims stay withheld pending the owner's U5 disposition (accept as
Python stage-1 prototype with a Rust re-measure gate, slim the apparatus, or
amend the ceiling) and the already-deferred Slice B adoption tasks (Req 7.10).
Next gate: owner decision on U5; if "accept", a Rust-port re-measurement gate
replaces the Python U5 number before any adoption task is scheduled.
Reciprocal spec link: [prototype-gate.md G1-R outcome amendment,
2026-11-08](../../../.kiro/specs/constraint-guided-compression/prototype-gate.md).

## COBESY check

Outcome first (gate discharged, U5 failed as measured); replay enabled by
committed drivers, receipts, sidecars and exact commands; disproof criteria
frozen before execution and each negative retained (A-1..A-7, U5); facts
(verification outputs, token counts) separated from interpretation (loop
feasibility vs compression); no adoption claimed from candidate validity.
