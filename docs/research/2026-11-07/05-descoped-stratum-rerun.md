# R31: Descoped replacement stratum — attempt-2 preregistration and re-run

- Report date/time and timezone: 2026-11-07, Europe/Berlin.
- Execution interval: starts after this preregistration is committed; results
  amend below with dated amendments.
- Status: planned (attempt 2 of 2, owner-approved descope).
- Spec: [.kiro/specs/deconvolution-native-core/prototype-plan.md](../../../.kiro/specs/deconvolution-native-core/prototype-plan.md)
  §9 (Amendment 2); gate [prototype-gate.md](../../../.kiro/specs/deconvolution-native-core/prototype-gate.md).
- Hypothesis ID and predecessor: H9 revision 2; predecessor
  [R30](04-s10-shared-core.md) (verdict revise).
- Type: preregistered.

## Decision

Owner chose the bounded descope recorded in R30 amendment 3: drop the Rust
digest module from the measured stratum (Python owns digests; Rust keeps
sidecar admission only). This entry preregisters attempt 2 before any code
change or live run. If U1–U7 pass on attempt 2 and the independent evidence
review approves, the gate may move to *proceed*; if any frozen target fails
again, the verdict is *stop* (attempt limit 2 of 2 exhausted).

## Hypothesis and falsification

Attempt 1 carried a Rust byte-parity twin of the Python engine (`pyjson.rs`
478, `turncheck.rs` 214, `bin/turncheck.rs` 26, shared hooks; ~740 canonical
lines). Revision: remove the twin from the tree and the claim. The claimed
replacement is now the shared Python engine (762 canonical:
`proto_core` 372, `scope_e3` 164, `scope_e4` 207, `proto_replay_test` 19) plus
Rust sidecar admission (~305 + hooks), so C_app ≈ 1,072 against frozen targets:
C_app ≤ 1,364; B_app − C_app ≈ 633 ≥ 200; C_total ≈ 2,570 ≤ 3,003;
C_seeds = B_seeds = 644; strictly smaller per-scope support (E3 173.5 < 280,
E4 216.5 < 358). Falsified if any frozen target fails under the descoped
inventory, or any replay/control diverges from the frozen identities. No target
is lowered; the attempt-1 failure is retained.

## Frozen inputs and method

Unchanged from the frozen plan §2/§3/§6 and R30: the 14-file baseline at
`103e1aa`; artifact roots and committed `.sha256` sidecars under
`.runs/{e3-facdrone,e4-lifecycle}` and `docs/experiments/deconvolution`;
specimen venv `.runs/e2-facdrone/venv` (CPython 3.13.16); ratified contract B +
four R26 changes + R25 rules; `ruff==0.17.3` / `rustfmt` canonical method;
`cargo +1.98.1 --locked` with nextest; PowerShell, one heavy job at a time.

Attempt-2 delta only: delete `native/src/pyjson.rs`, `native/src/turncheck.rs`,
`native/src/bin/turncheck.rs`, `native/tests/{pyjson_parity,turncheck}.rs`, the
`lesscode-turncheck` `[[bin]]` entry and module registrations; keep
`native/src/admission.rs` + tests and the shared Python engine untouched.
Receipt: `results/s10-u5-measurement-2.json` (same canonical method).

## Expected outcomes (preregistered; failures reported as failed)

1. E3 double-record via the unchanged shared engine: two fresh recordings,
   LF-body digest `92a48f77f66f2c9d0d0ebaf6e9e4b37f747816d2831dd577b72e183b7bf07671`;
   E1b/E1c/E1c-2 replay goldens zero divergence.
2. E4 double-record: identical pair, LF-body digest
   `0b2e383dd6fbbc243215e24a7844674404331308b3a547239594461c5bea5584`
   (5,479,067 bytes); `golden-a.json` replay zero divergence.
3. Candidate replays with the R30 preregistered decisions: E3 `cand` ACCEPT,
   E4 `cand-turn3` ACCEPT (level2_ops = 0, exit 0 each).
4. Adverse controls 9/9 fail closed, including missing-artifact via the kept
   Rust admission CLI.
5. U6: existing CLI contracts green; turn evidence cited from recorded
   attempt-1 receipts (R30 amd 2, nextest 23b9da3e), identified as such.
6. U5 re-measurement (3-stratum, physical/canonical/separated-form) and U7
   attempt-2 ledger within ceiling 1,400; then independent evidence review.

## Stop conditions

Any frozen target failing → record as failed, no post-hoc target change;
attempt limit exhausted → verdict *stop*; frozen inputs missing or sidecar
mismatch → fail closed before execution.

## Limits and interpretation

The descope narrows the claim, not the evidence bar: the replacement no longer
claims the pure turn-input transformation subset (~240 canonical baseline
lines, `regen_turn.py`) nor Rust-side digest emission. Savings are reported
against frozen B_app (primary) and like-for-like (B_app − 240 = 1,465;
savings ≈ 393) alongside. Known-scope reuse only (E3+E4); no unseen-scope
generalization.

## Tool/spec consequence and next gate

Pass + approved review → `solution_unlock` may move to *proceed*; only the
demonstrated slice may seek production design approval. Fail → *stop*;
R27–R31 stand as the usefulness finding with no production native core.

## COBESY check

Preregistration precedes execution; the descope is owner-decided before
attempt-2 numbers exist, not post-hoc tuning; disproof explicit (any frozen
target failure stops the gate); negative results safe to report; facts and
interpretation separated.

## Evidence and results

Execution begins 2026-11-07; amendments below with dated results.
