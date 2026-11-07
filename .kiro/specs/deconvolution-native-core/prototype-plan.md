# Frozen prototype plan: S2 / S9 / S10 (solution unlock)

Written and frozen 2026-11-07, Europe/Berlin. Owner approved execution on
2026-11-07 (opencode session decision: "approve execution now — freeze the plan
in-repo, then execute under U1–U7 without another pause"). This file freezes the
inputs, observations, numerical targets, code budget and attempt limit required
by requirements.md 1.2 and prototype-gate.md. It authorizes only the disposable
prototype; no production design/tasks/code, no third scope, no adoption.

## 1. Boundary (frozen)

Reuse the existing `native/` crate (`lesscode-core`) and its committed
`Cargo.lock` pins: `ast-grep-core 0.51.0`, `serde 1.0.229`, `serde_json
1.0.154`, `sha2 0.11.0`, `clap 4.6.0` (transitive: `ignore 0.4.34`, `walkdir
2.5.0`, `regex 1.13.1`, `toml_edit 0.25.17`, `windows-sys 0.61.2`). No new
dependencies except what is already locked; explicitly no `serde_jcs`, `fs4`,
`quick-junit`, `gix`, `pg_query`, `postgresql_embedded`.

Deliverables (all charged to U7):
1. Artifact admission: verify `.runs/` artifact bytes against committed
   `.sha256` sidecars with explicit `--artifact-root` bindings.
2. Receipt/digest parity: Python-compatible canonical JSON emission in Rust
   (`serde_json` + custom formatter preserving `-0.0`/`0.0`, `ensure_ascii`,
   indentation of the upstream prototype, byte for byte).
3. Acceptance/residue decision core (pure turn-input transformation:
   `build_prompt` / feedback validation) shared across scopes.
4. One shared Python subprocess recorder/replayer + arm runner using the
   specimen's pinned interpreter and the existing SQLAlchemy
   `before_cursor_execute` hook, with explicit per-scope adapters.

Amendment 2 (2026-11-07, attempt 2; §9): deliverables 2 and 3 leave the slice —
Python owns digest/canonical emission (the shared engine) and turn-level
protocol validation returns to the unchanged frozen baseline; Rust keeps
artifact admission only.

Out: live model calls (frozen recorded turns only), candidate regeneration,
adoption, new languages, proxy/indexers, new engines, five-target release work.

## 2. Frozen inputs and artifact roots

- Apparatus inventory (14 files, baseline = committed tree at `103e1aa`):
  `docs/experiments/deconvolution/e1b-facdrone-evidence-publication/{e1b_oracle,run_arm,scope}.py`,
  `docs/experiments/deconvolution/e3-evidence-regeneration/{run_e3,test_e3_replay}.py`,
  `docs/experiments/deconvolution/e4-lifecycle/{e4_oracle,run_e4,test_e4_replay,patchcheck}.py`,
  `docs/experiments/deconvolution/t51-integration/editscope.py`,
  `docs/experiments/deconvolution/{receipt_atomic,adaptive,guard}.py`,
  `docs/experiments/deconvolution/regenerator/regen_turn.py`.
- E3 scope: the five Facdrone evidence modules frozen per R25; frozen original
  copy `.runs/e2-facdrone/source` (venv `.runs/e2-facdrone/venv`, CPython
  3.13.16); accepted candidate `.runs/e3-facdrone/cand` (only the five files
  differ); mutant copies `.runs/e3-facdrone/{mut,orig-mut,orig-check}`.
- E4 scope: the seven lifecycle monitor files frozen per R26 §6.1; frozen
  original and specimen under `.runs/e2-facdrone/source`; accepted candidate
  `.runs/e4-lifecycle/cand-turn3`; mutable copies `orig-mut`/`cand-mut`
  (verified identical to references at E4 end).
- Artifact roots and bindings (sidecars committed, bytes local):
  - `--artifact-root .runs/e4-lifecycle`: `golden-a.json`, `golden-b.json`,
    `held-out-a.json`, `held-out-b.json`, `smoke.json`, `e4-orig.sqlite`,
    `e4-cand.sqlite` against committed sidecars
    `docs/experiments/deconvolution/e4-lifecycle/results/golden-e4.json.sha256`,
    `held-out-e4.json.sha256` and the on-disk `.runs/e4-lifecycle/*.sha256`.
  - `--artifact-root .runs/e3-facdrone`: `t31.sqlite`, `t4.sqlite`, `cand/`,
    `orig-check/` against R25 records and sidecars.
  - `--artifact-root docs/experiments/deconvolution`: committed results and
    `.sha256` sidecars (e1b/e1c/e1c2/e2/e3/e4/regenerator).
  Missing local bytes or digest mismatch = unavailable/invalid evidence; a
  sidecar without verified bytes never admits a claim (U1, requirement 2.2).
- Frozen turn material: `docs/experiments/deconvolution/regenerator/results/turns/`
  (`e4-turn1..3`, `t4-turn1`, `t4-turn1-retry`, `t4-turn2`) with committed
  sidecars; receipts `request-e4-turn*.json`; recorded expected decisions in
  R25/R26 (E3 accepted at round 1; E4 accepted at turn 3 with recorded residue
  and known limitations). Replays must reproduce the recorded verdicts,
  including the known differences (`labeled_reason`, `bind_lifecycle`
  unreached, T4/T5 band results); divergence from recorded outcomes is a
  finding, not noise.
- Identities: E3/E4 revisions, dirty hashes and file inventories as recorded in
  R25/R26 and their committed results JSON (`receipt.json`, `seeds.json`,
  `structure-*`, `editscope-*`, `patchcheck-*`). U1 re-records raw-byte hashes
  of every frozen input at execution time into the research entry.

## 3. Observations, normalization, protocol (frozen)

- Observation levels: L1 = recorded trace observations (operation sequence and
  payloads after normalization); L2 = store-level SQL divergences. L2
  differences without query-differential evidence (upstream requirement 3.2)
  refuse acceptance; never fall back to L1-only acceptance.
- Normalization: preregistered `Norm` helpers of the frozen oracles only; two
  recordings of the same original must be byte-identical after normalization
  (U2); mismatch refuses.
- Seeds/corpus: E3/E4 `SEEDS` + `generated(n)` corpora are immutable
  (`C_seeds = B_seeds`); baseline deselections stay unchanged; seed tests and
  reference harnesses must pass for acceptance.
- Adverse controls (U3): planted L1 difference detected; stale/missing/partial
  results refused; protected edit voids candidate; L2 gap without query
  evidence refused; interrupted arm is reported as interruption, not a kill.
- Turn transformations (U6): pure turn-input transformation checked against
  the frozen protocol examples for accepted AND failed turns (e4-turn2
  `labeled_reason` failure, t4-turn1 retry), including exact permitted
  feedback and retained failed attempts.

## 4. Numerical baselines and targets (frozen; never lowered post hoc)

Canonical formatting: `ruff==0.17.3 format --isolated --line-length 100
--target-version py313` (Python), `rustfmt` (Rust). Baselines from the
2026-11-07 empirical audit (R27):

| Stratum | Baseline | Ceiling / target |
| --- | --- | --- |
| `B_app` replaceable apparatus & wiring | 1,705 | `C_app <= 1,364` AND `B_app - C_app >= 200` |
| `B_ops` specimen domain ops + fixtures | 854 | preserved behavior; verbatim parity |
| `B_seeds` corpus seeds + generators | 644 | `C_seeds = B_seeds` exactly |
| `B_total = app+ops+seeds` | 3,203 | `C_total <= 3,003` |
| E3 per-scope support (run_e3 63 + test_e3_replay 58 + e1b runner 159) | 280 | strictly fewer E3 support lines |
| E4 per-scope support (run_e4 146 + test_e4_replay 62 + e4 runner 150) | 358 | strictly fewer E4 support lines |

Report physical, canonical and separated-form statements together; count
shared code once on each side; tests/fixtures separate; relocated logic is not
deletion. These are preregistered targets, not measured results.

## 5. U7 effort budget and attempt rules (frozen; amended 2026-11-07)

Amendment 2026-11-07 (owner decision after R28): the cumulative U7 ceiling is raised
from `<= 600` to `<= 1,400` added/modified maintained canonical lines. Reason: S2
proved byte-parity requires an order/token-preserving JSON model (774 charged lines
against the 600 estimate, which assumed a thin `serde_json` formatter). All other
targets — `<= 2` attempts, strata ceilings, U1-U6 — are unchanged and never lowered.

- `<= 1,400` added/modified maintained canonical lines of shared + adapter-wiring
  code cumulatively across `<= 2` attempts; restarts consume attempts;
  deletions do not refund budget.
- Charge new/modified shared Rust and Python runner/shim files in full at each
  file's largest observed canonical size.
- Pre-existing `native/src/lib.rs` (618) and `main.rs` (225): prefer a
  dedicated binary/submodule; registration-only touches `<= 20` diff lines
  charge the diff; broader edits charge the whole file.
- Verbatim-migrated `op_*`/`SEEDS` blocks verified by AST/diff parity count
  into `C_ops`/`C_seeds`; wrapper glue charges U7.
- Shared-core changes after E3 freeze fail U4; a second attempt requires a
  dated hypothesis revision, new preregistration and replay of both scopes.
- Amendment 2 (2026-11-07): attempt 2 preregisters the descoped replacement
  stratum (§9). The attempt-2 U7 ledger charges the §9 inventory only, at each
  file's largest observed canonical size; ceiling 1,400 unchanged; attempt 2 of
  2. The attempt-1 ledger (774 at S2, 1,808 at close) is retained as the
  recorded failed first attempt; deleting attempt-1 Rust neither refunds nor
  erases it.

## 6. Tools, locks, commands, host rules (frozen)

Native Windows, plain user, PowerShell (never cmd.exe, never bare `bash`).
Rust `+1.98.1` with `--locked`; tests `cargo +1.98.1 nextest run --locked
--manifest-path native/Cargo.toml --test-threads 4 --no-fail-fast` + separate
`--doc`; repo Python tests via pinned uv interpreter and pytest-xdist
(docs/testing.md). Python specimen runs use the recorded venvs
(`.runs/e2-facdrone/venv`, extras under `.runs/e1*-facdrone/extras`);
`PYTHONDONTWRITEBYTECODE=1` for recording runs; `uv.exe` invoked directly.
Attribution-relevant runs serial with numeric threads = 1; one heavy job at a
time; adaptive deadlines via `docs/experiments/deconvolution/adaptive.py`;
receipted commands (stdout+stderr+exit+duration) for every evidence step.
CI/cross-platform verification only through the existing Dagger configuration
(`.dagger/`, `native-core.yml`); WSL never used for development tasks.

## 7. Execution sequence (frozen)

1. **S2** — artifact admission and digest parity. Verify all declared
   artifact-root bytes against sidecars; implement and prove the
   Python-compatible digest/receipt emission in Rust against the frozen
   goldens (byte-identical digests, `-0.0`/`0.0`, `ensure_ascii`, indent).
   Evidence recorded before anything depends on it.
2. **S9** — recorder-import surface comparison (non-adopting). Compare the
   two frozen surface contracts (tree-only imports vs tree+recorder imports)
   using existing apparatus under this approved bounded plan; evidence only,
   no ratification, no amendment adoption.
3. **S10** — shared core. Build bounded shared recording/acceptance support on
   E3 (attempt 1), freeze it (hashes + inventories), then onboard E4 with
   scope support only. Live double recording + original replay on both scopes;
   actual candidate replay with preregistered expected decisions; adverse
   controls; turn transformation checks.
4. **U1–U7 assessment** — measure against §4/§5, record 3-stratum reports,
   then independent evidence review presented to the owner (inline review,
   identified as such under the 2026-11-07 no-subagent rule). Only U1–U7 pass
   + approved review unlocks normal production design eligibility.

## 8. Stop conditions and recording

Fail-closed: any U-criterion failed or unavailable keeps the feature locked
with verdict revise/stop/not-run; retain failures, controls and overruns.
Research entries under `docs/research/` (dated, execution dates distinct from
writing dates) with reciprocal index/spec links; `spec.json.solution_unlock`
evidence fields updated per step; receipts for every command. Effort/attempt
exhaustion or observations-defeat-usefulness → stop.

## 9. Amendment 2 (2026-11-07, attempt 2): descoped replacement stratum

Owner decision (2026-11-07, opencode chat, after R30 amendment 3 verdict
*revise*): execute the bounded descope — drop the Rust digest module from the
measured replacement stratum. Python owns digest/canonical emission (the shared
engine emits the byte-identical recordings; `proto_core.py` `canon` /
`write_golden` plus `verdict` already carry digests and acceptance); Rust keeps
sidecar admission only (~305 canonical lines + registration hooks). This is a
deliverable/accounting restructure, not a target change: every §4 ceiling and
the U7 ceiling 1,400 stay frozen; nothing is lowered post hoc. Attempt-1
results (U5/U7 FAIL at C_app 1,808) are retained as recorded failures (R30
amendment 3).

### Hypothesis revision (falsifiable; preregistered before execution)
Attempt 1 kept a Rust byte-parity twin of the Python engine — `pyjson.rs`
(478), `turncheck.rs` (214) + `bin/turncheck.rs` (26) plus shared hooks, ~740
canonical lines whose purpose was to re-prove in Rust what the Python engine
already emits. Attempt 2 removes that twin from the prototype tree and the
claim. Revised hypothesis: the claimed replacement = shared Python engine
(`proto_core` 372 + `scope_e3` 164 + `scope_e4` 207 + `proto_replay_test` 19 =
762) + Rust sidecar admission (~305 + hooks), so C_app ≈ 1,072 satisfies every
frozen §4 target: C_app ≤ 1,364; B_app − C_app ≈ 633 ≥ 200; C_total ≈ 2,570 ≤
3,003; C_seeds = B_seeds; strictly smaller per-scope support (E3 173.5 < 280,
E4 216.5 < 358). Falsified if any frozen target fails under the descoped
inventory, or any replay/control diverges from the frozen identities.

### Descoped inventory and deletions (preregistered)
- Kept and charged: `native-prototype/{proto_core,scope_e3,scope_e4,proto_replay_test}.py`,
  `native/src/admission.rs`, admission registration hooks (Cargo.toml / lib.rs /
  main.rs as measured by diff after the deletions).
- Deleted from the prototype: `native/src/pyjson.rs`, `native/src/turncheck.rs`,
  `native/src/bin/turncheck.rs`, `native/tests/{pyjson_parity,turncheck}.rs`,
  the `lesscode-turncheck` `[[bin]]` entry and their module registrations.
  R28 (Rust digest parity) and R30 amendment 2 (turn checks, nextest 23b9da3e)
  remain recorded evidence, re-derivable from the frozen attempt-1 tree
  (commit `f6bb6e2`).
- Deliverables 2 (Rust receipt/digest parity) and 3 (Rust turn core) leave the
  slice. Turn-level protocol validation is served by the unchanged frozen
  baseline (`regen_turn.py` subset); U6 cites the recorded attempt-1 turn
  evidence as such, identified per the gate.

### Attempt-2 replay obligations (both scopes, per §5)
1. E3 double-record via the unchanged shared engine: two fresh recordings,
   LF-body digest `92a48f77f66f2c9d0d0ebaf6e9e4b37f747816d2831dd577b72e183b7bf07671`;
   the three frozen replay goldens (E1b, E1c, E1c-2) replay with zero divergence.
2. E4 double-record: identical pair, LF-body digest
   `0b2e383dd6fbbc243215e24a7844674404331308b3a547239594461c5bea5584`
   (5,479,067 bytes); `golden-a.json` replays with zero divergence.
3. Candidate replays with the preregistered decisions of R30: E3 `cand`
   ACCEPT, E4 `cand-turn3` ACCEPT (each level2_ops = 0, exit 0).
4. Adverse controls: 9/9 fail closed, including missing-artifact via the kept
   Rust admission CLI.
5. U6: existing CLI contracts green; recorded turn evidence identified as
   attempt-1 receipts; no new turn claims.
6. Re-measurement receipt `results/s10-u5-measurement-2.json` under the frozen
   ruff 0.17.3 / rustfmt method; U1–U7 re-assessed on attempt 2 alone, then
   independent evidence review.

### U7 attempt-2 ledger (preregistered)
Charges the §9 inventory at each file's largest observed canonical size, plus
any deletion-and-re-addition within attempt 2. Ceiling 1,400 unchanged; this is
attempt 2 of 2. The attempt-1 ledger is retained in R30 / results as the failed
first attempt.

### Sensitivity (reporting only; no new pass criteria)
The descoped claim no longer replaces the pure turn-input transformation
subset (~240 canonical baseline lines inside B_app 1,705, `regen_turn.py`);
report savings against frozen B_app (primary, §4) and like-for-like
(B_app − 240 = 1,465; savings ≈ 393) alongside, both ≥ 200. The attempt-1
exclusions (census/deadline/coverage machinery ~180, HTTP client ~262) stay
excluded.

[Gate](prototype-gate.md) · [Slice proposal](slice-proposal.md) ·
[Spikes](spikes.md) · [R27](../../../docs/research/2026-11-07/01-solution-first-native-slice.md)
