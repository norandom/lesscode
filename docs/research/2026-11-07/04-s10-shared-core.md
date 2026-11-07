# R30: S10 shared recording/acceptance core — preregistration and C-side inventory
- Report date: 2026-11-07, Europe/Berlin. Execution starts after this entry; results amend below.
- Contract: owner-ratified B + four R26 changes + R25 rules (see R29 amendment 1).
- U7 ledger at start: 774 / 1,400.

## Hypothesis (falsifiable)
One shared engine + two thin adapters covers E3 and E4 recording, replay and acceptance
with unchanged observations, and the replacement is smaller than the frozen baseline:
`C_app <= 1,364` AND `B_app - C_app >= 200` (B_app = 1,705), `C_total <= 3,003`
(B_total = 3,203), `C_seeds = B_seeds = 644`, strictly less scope-specific support on
each scope (E3 baseline 280, E4 baseline 358 canonical).

## C-side inventory (preregistered before coding)
| New/charged unit | Replaces (baseline, canonical) | Est. lines |
| --- | --- | --- |
| `native/src/pyjson.rs` + `admission.rs` + hooks (done, S2) | receipt/sidecar hashing in `receipt_atomic.py` class | 774 (measured) |
| `native-prototype/proto_core.py` — Norm/canon/signatures/first_diff/run_sequence/_attached/replay_engine/L2-replay-driver/record/embedded_engine + deadline call | duplicated generic engines in `e1b_oracle.py` (159) + `e4_oracle.py` (150) + `test_e3_replay.py` (58) + `test_e4_replay.py` (62) + arm/probe/group parts of `run_arm.py`/`run_e3.py`/`run_e4.py` (~230) | ~300 |
| `native-prototype/scope_e3.py`, `scope_e4.py` adapters (import frozen `Env`/`SEEDS`/`generated` = ops/seeds strata, verbatim by import) | `run_e3.py` (63) + `run_e4.py` (146) wiring | ~120 |
| `native/src/turncheck.rs` (+ CLI hook) — ratified-protocol turn validation on admission-verified turn bytes | regen_turn feedback-validation subset | ~150 |

NOT claimed as replaced (explicitly outside the slice, frozen files stay): E1b mutation
census/deadline/coverage machinery (`run_arm.py` `all`/`time`/`seeds`/`report`/cap,
~180 canonical), regen_turn HTTP client (~262), solver/standing controls (e5-*).
U5 sensitivity must quantify the effect of this exclusion on the frozen comparison.

## Expected outcomes (preregistered; failures are reported as failed)
1. E3 double-record via the shared engine, specimen venv: two fresh recordings whose
   LF-body digests are identical to each other and to the frozen E3-relevant golden
   identity (E3 replay goldens are the E1b/E1c/E1c-2 set; the E3 acceptance recorded
   `t31.sqlite`/`t4.sqlite` traces; equality target: mutual identity + golden-vs-replay
   zero L1 divergence; the frozen golden-a bytes belong to E4's recorder).
2. E4 double-record: identical pair, equal to the frozen `.runs/e4-lifecycle/golden-a.json`
   LF-body digest `0b2e383d…e5584` (the decisive reuse proof).
3. Original replays: zero L1 and zero unexplained L2 on both scopes.
4. Candidate replays with preregistered decisions: E3 `cand` — accepted (zero L1,
   fresh seeds/replay); E4 `cand-turn3` — accepted; E4 turn 1 — failed, thinking-only
   (all 128k output tokens spent thinking, no tool result); E4 turn 2 — failed,
   `labeled_reason` missing from the turn-2 candidate surface (recorder-bound name
   absent from contract A, present in ratified B); E4 turn 3 — accepted.
5. Adverse controls (U3): planted L1 difference detected; stale replay refused;
   missing artifact refused; protected edit voids; interrupted arm reported as
   infrastructure, not a kill; L2 divergence without query evidence refuses acceptance.
6. U5 measurement under `ruff==0.17.3 format --line-length 100` / `rustfmt` with the
   frozen 3-stratum method; U7 ledger updated with each file's largest observed size.

## Stop conditions
Any expected outcome failing -> record as failed, no post-hoc target change; U7 ceiling
1,400 or attempt limit 2 exhausted -> gate verdict revise/stop; frozen inputs missing
or sidecar mismatch -> fail closed before execution.

## Evidence and results
Execution begins 2026-11-07; amendments below with dated results.

## COBESY check
Target behavior: preregister the full inventory and every expected decision before the
first live run so reuse claims cannot be tuned to whatever happens. Challenge: the
shared engine is only proven if it reproduces frozen digests, not if it merely runs.

## Amendment 1: execution results (2026-11-07, both scopes)
- E3 record via the shared core: two live recordings identical, LF-body digest
  `92a48f77f66f2c9d0d0ebaf6e9e4b37f747816d2831dd577b72e183b7bf07671` = the frozen
  committed `golden-trace.json` sidecar value (51 sequences, 290 operations, ~31 s each).
  All three frozen replay goldens (E1b, E1c, E1c-2) replay with zero divergence.
- E4 record via the shared core: two live recordings identical, LF-body digest
  `0b2e383dd6fbbc243215e24a7844674404331308b3a547239594461c5bea5584` = the frozen
  `golden-a.json` declared LF-body, 5,479,067 bytes exactly (50 sequences, 317
  operations, ~21 s each). `golden-a.json` replays with zero divergence.
- Artifacts: `.runs/native-prototype/proto-e{3,4}-{a,b}.json` (+ `.sha256`; files over
  1 MB stay out of git per repo convention). The 4.1 MB S2 admission receipt moved to
  `.runs/native-prototype/` the same way (sidecar stays committed in results/).
- Expected-outcome status: (1) PASS E3, (2) PASS E4, (3) PASS both scopes (replay
  level). Candidate arms, adverse controls and measurement still pending; results
  amend here as they complete.

## Amendment 2: turn checks and a recorded protocol fact (2026-11-07)
Rust `turncheck` (dedicated `lesscode-turncheck` binary; `native/src/turncheck.rs`)
validates the frozen, admission-verified E4 turns under the ratified contract: all
checks pass (nextest 23b9da3e, 111/111; receipt `results/s10-turncheck.json`).
- e4-turn1: failed-thinking-only confirmed (`stop_reason=max_tokens`, thinking-only
  content); parameters preregistered; feedback correctly null.
- e4-turn2: produced-candidate; **recorded fact**: its request carries `feedback:
  null` — a thinking-only failure yields no candidate and therefore no feedback. The
  preregistered expectation assumed feedback on every post-failure turn and was
  corrected to match the recorded evidence (descriptor fix, not an evidence change).
- e4-turn3: accepted; feedback present (turn 2's arm failure), prompt carries the
  frozen contract head and acceptance command byte-for-byte.
T4 turns remain excluded (inadmissible bytes, R28).

## Amendment 3: candidate arms, controls, measurement and U1-U7 assessment (2026-11-07)
- **Candidate arms (preregistered decisions, all correct)**: E3 `cand` ACCEPTED
  (all seed groups pass: 1,534 tests, replay fresh 64/64 sequences / 311 operations,
  level2_ops=0, exit 0). E4 `cand-turn3` ACCEPTED (verdict exit 0, fresh 50/50
  sequences / 317 operations, level2_ops=0, `sql-divergences-e6-cand.json` bound as
  query evidence). Rounds written as `sql-divergences-rounds10.json`.
- **U4 freeze**: after the final shared-core changes (class-level listen, boot-rule
  guard, verdict rules), E3 re-recorded twice more: `92a48f77…` both times — the
  frozen final core serves both scopes unchanged (`.runs/native-prototype/proto-e3-c{1,2}.json`).
- **U3 controls: 9/9 fail closed** (`results/s10-controls.json`): accept-clean;
  stale-replay-refused; L2-without-query-evidence-refused (bound evidence admissible);
  invalid-arm-stays-invalid; missing-artifact-refused (Rust admission CLI);
  protected-edit-voids-record; interrupted-arm-not-a-kill (exit 124, infrastructure);
  planted-L1-difference-detected (doctored golden flagged at the exact planted path).
- **U5/U7 measurement FAILED as frozen** (`results/s10-u5-measurement.json`):
  C_app = 1,808 canonical lines (Python 762: proto_core + adapters + replay target;
  Rust 1,046: pyjson 463, admission 293, turncheck ~167, binary + hooks 23) against
  ceiling 1,364; B_app − C_app = −103 (the replacement is larger than the 1,705
  baseline); C_total 3,306 > 3,003; U7 1,808 > 1,400. Per-scope support IS smaller
  (E3 173.5 < 280, E4 216.5 < 358) and C_seeds = B_seeds, but the frozen conjunction
  fails. Cost driver: the Rust byte-parity core (1,046 lines) the slice specified.
- **U1-U7**: U1 pass (plan frozen, inventories, sidecar-verified bytes); U2 pass
  (double-record + replay + preregistered candidate decisions, both scopes); U3 pass
  (9/9); U4 pass (final core frozen, re-proven on E3); U5 FAIL; U6 pass (existing
  CLI contracts green, 111/111 nextest; honest differences retained); U7 FAIL.
- **Gate verdict: revise.** The reuse hypothesis is mechanically demonstrated
  (byte-identical recordings/replays on both scopes through one shared engine), but
  the usefulness targets fail: preserving Python byte parity in Rust costs more than
  the duplicated apparatus it replaces. A concrete bounded revision exists (drop the
  Rust digest module from the measured stratum: C_app would be ~1,072 ≤ 1,364,
  saving 633; or stop here with the negative result). Owner decision required;
  evidence_review stays not-reviewed.
