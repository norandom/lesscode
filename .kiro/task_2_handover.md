# Task-group handover 2: bounded descope executed — gate verdict proceed (2026-11-07)

Written at the end of the second task group under owner rule 6 (AGENTS.md). Start
the next group from this file plus the mmhmem checkpoint and `.kiro/handover.md`.

## What was done (commits on main)

1. `b9c0cd0` — attempt-2 preregistration BEFORE any code change: prototype-plan.md
   §9 (Amendment 2: descoped stratum, hypothesis revision, preregistered
   deletions, replay obligations, U7 attempt-2 ledger, sensitivity reporting),
   R31 entry `docs/research/2026-11-07/05-descoped-stratum-rerun.md`, README
   index, spec.json owner-decision record. Owner decision: bounded descope
   (drop the Rust digest module from the measured stratum; Python owns digests;
   Rust keeps sidecar admission).
2. `abb71c3` — descope code: `native/src/{pyjson,turncheck}.rs`,
   `src/bin/turncheck.rs`, `tests/{pyjson_parity,turncheck}.rs` + fixtures
   deleted; `admission.rs` made self-contained (it had imported pyjson — R30's
   ~1,072 estimate missed this; admission grew 305 → 415 canonical): local
   Python-`json.dumps(indent=1)` receipt writer (ensure_ascii, DEL, surrogate
   pairs), `serde_json` bindings parse, `sha2` direct hashing. fmt/clippy/doc
   green, nextest 106/106, doc-tests ok. Smoke: frozen `s2-bindings.json`
   reproduces R28 exactly (21/11/5/1/4, exit 1 fail-closed).
3. `8537c8e` — attempt-2 re-run, all receipts committed: E3 double-record
   `92a48f77…` ×2 (51 seq/290 ops) + E1b/E1c/E1c-2 goldens zero divergence;
   E4 double-record `0b2e383d…` ×2 at exactly 5,479,067 bytes + golden-a zero
   divergence; E3 cand + E4 cand-turn3 ACCEPT as preregistered (level2_ops=0,
   exit 0); 9/9 controls fail closed (`s10-controls-2.json`; missing-artifact
   via the descoped admission binary, planted-L1 on the attempt-2 recording);
   measurement `s10-u5-measurement-2.json`: C_app 1,193 (py 762 + admission 415
   + hooks 16 by live git diff vs 103e1aa), U5 all pass (0.6997 ≤ 0.80, saved
   512 ≥ 200, C_total 2,691 ≤ 3,003, e3 173.5 < 280, e4 216.5 < 358;
   like-for-like 272 ≥ 200); U7 1,193 ≤ 1,400, attempt-1 1,808 retained.
   Run receipts `s10-attempt2-runs.json`. Drivers `s10_controls2.py`,
   `s10_measure2.py` (attempt-1 drivers/receipts untouched).
4. `8698bd5` — R31 amendment 1: full results + U1–U7 re-assessment, all pass;
   verdict proceed pending review.
5. `2c82762` — inline evidence review receipt `s10-evidence-review-2.json`
   (11 checks pass; residual risks: inline reviewer not independent in fact;
   U6 turn evidence is attempt-1, re-derivable from f6bb6e2; production locked);
   spec.json: `solution_unlock.status=proceed`, `evidence_review=approved`;
   README index updated.

## Evidence pointers

- Research: R31 (`docs/research/2026-11-07/05-descoped-stratum-rerun.md`) + README
  index; predecessors R27–R30 unchanged.
- Receipts: `docs/experiments/deconvolution/native-prototype/results/
  {s10-attempt2-runs,s10-controls-2,s10-u5-measurement-2,s10-evidence-review-2}.json`
  (+ `.sha256` sidecars; the attempt-2 runs sidecar is
  `32715d90c6c058c9c998e7d03b6e276b3bcf0a70f2ccb55711eead48b841b2e8`).
- Recordings (local, reproducible): `.runs/native-prototype/proto2-e{3,4}-{a,b}.json`
  + attempt2-*.log receipts; round-20 L2 outputs in e3/e4 results (untracked).
- State: `.kiro/specs/deconvolution-native-core/spec.json` (status=proceed,
  evidence_review=approved, 9 evidence entries; JSON validated).
- Attempt-1 evidence stands untouched: R30 + `s10-u5-measurement.json`,
  `s10-controls.json`, turn receipts `s10-turncheck.json` (tool deleted, tree at
  `f6bb6e2` re-derivable).

## Current state

- The gate is passed on attempt 2 of 2 under the frozen targets: verdict
  **proceed** with an approved INLINE evidence review. No target was lowered;
  the descope shrank the claimed stratum, not the ceilings; attempt-1's failure
  and its 1,808-line charge are retained.
- Known-scope reuse only (E3+E4 demonstrated; never unseen-scope generalization).
- Production remains locked: requirements/design/tasks unapproved. The
  demonstrated slice may seek normal production design approval next.
- Owner's uncommitted WIP (`src/lesscode_baseline/rules/evaluate.py`,
  `tests/test_rule_evaluation.py`, ~248 insertions) was deliberately left
  untouched/uncommitted, as preserved at the task-1 handover.

## Next steps (owner decision)

1. Production design phase for the demonstrated slice (normal gates:
   design generation → owner review → tasks), OR
2. commission a truly independent evidence review first (residual risk noted),
   OR
3. stop here: the unlock stands as recorded.

## Environment / gotchas

- Same as task group 1: venv `.runs/e2-facdrone/venv`, PYTHONDONTWRITEBYTECODE=1,
  one heavy job at a time, PowerShell, `cargo +1.98.1 --locked`, nextest.
- E3 arm ≈ 8 min serial; E4 arm ≈ 1 min; records ≈ 20-25 s each.
- `Get-FileHash` was unavailable in this session; hash via the venv python.
- Attempt-2 round artifacts use `--round 20` (round 10 = attempt 1); do not
  reuse round numbers across attempts.
