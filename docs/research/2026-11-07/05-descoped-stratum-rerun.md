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

## Amendment 1: attempt-2 execution results (2026-11-07, both scopes)

- Descope executed as preregistered (commit `abb71c3`): `admission.rs` self-contained
  (it had imported `pyjson` — the R30 ~1,072 estimate assumed it stood alone);
  `pyjson.rs`, `turncheck.rs`, `bin/turncheck.rs`, parity/turn tests and fixtures
  deleted; `lib.rs`/`Cargo.toml` hooks removed. fmt/clippy/`cargo doc` green;
  nextest 106/106; `cargo test --doc` ok. Admission smoke on the frozen
  `s2-bindings.json` reproduces R28 exactly (21 total / 11 verified / 5 mismatch /
  1 missing / 4 no_bound_sidecar, exit 1 fail-closed).
- E3 double-record (`.runs/native-prototype/proto2-e3-{a,b}.json`): 51 sequences,
  290 operations, LF-body `92a48f77f66f2c9d0d0ebaf6e9e4b37f747816d2831dd577b72e183b7bf07671`
  on both runs = the committed `golden-trace.json` stored-bytes sidecar; the three
  frozen replay goldens (E1b, E1c, E1c-2) replay with zero divergence.
- E4 double-record (`.runs/native-prototype/proto2-e4-{a,b}.json`): 50 sequences,
  317 operations, LF-body `0b2e383dd6fbbc243215e24a7844674404331308b3a547239594461c5bea5584`
  at exactly 5,479,067 bytes on both runs = the committed `golden-e4.json.sha256`
  declared LF-body; `golden-a.json` replays with zero divergence.
- Candidate arms reproduce the preregistered decisions: E3 `cand` ACCEPT
  (fresh 64/64 sequences, 311 operations, level2_ops=0, exit 0, 462.5 s);
  E4 `cand-turn3` ACCEPT (fresh 50/50, 317 operations, level2_ops=0, exit 0, 58.1 s).
- Adverse controls 9/9 fail closed (`results/s10-controls-2.json`), including
  missing-artifact via the descoped admission binary and planted-L1 detected on
  the attempt-2 recording.
- Measurement (`results/s10-u5-measurement-2.json`): C_app = **1,193** = Python 762
  + Rust 431 (admission 415 + registration hooks 16 by live git diff vs `103e1aa`:
  lib.rs 1, main.rs 15, Cargo.toml 0). **U5 PASS**: ratio 0.6997 ≤ 0.80,
  1,193 ≤ 1,364; saved 512 ≥ 200; C_total 2,691 ≤ 3,003; E3 173.5 < 280;
  E4 216.5 < 358; C_seeds = B_seeds. Like-for-like (B_app − 240 = 1,465):
  saved 272 ≥ 200, recorded as sensitivity.
- **U7 attempt-2 ledger**: 1,193 ≤ 1,400; attempt 2 of 2; the attempt-1 charge
  (1,808) is retained in the receipt and nothing was refunded. The measured 1,193
  replaces R30's ~1,072 estimate, which had not accounted for admission's pyjson
  dependency (self-containment grew admission 305 → 415 canonical lines).
- **U1–U7 re-assessment (attempt 2)**: U1 PASS (frozen boundary; sidecar-verified
  bytes; admission re-proven on frozen bindings); U2 PASS (double-record + replay +
  preregistered candidate decisions, both scopes); U3 PASS (9/9); U4 PASS (the
  shared Python core is untouched by the descope — attempt 2 changed Rust only —
  and both scopes re-produce the frozen identities through it); U5 PASS (above);
  U6 PASS (existing CLI contracts green, 106/106 nextest; turn-level evidence
  cited from attempt-1 receipts and identified as such; the pure-turn subset is
  excluded from the claim and declared); U7 PASS (1,193 ≤ 1,400, receipts
  committed, re-derivable).
- Run receipts with exits/durations: `results/s10-attempt2-runs.json` (+ sidecar).
  **Gate verdict: proceed**, subject to the evidence review recorded in
  `results/s10-evidence-review-2.json` (inline, identified as such per the
  2026-11-07 no-subagent rule).

