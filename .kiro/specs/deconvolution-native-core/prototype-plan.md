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

## 5. U7 effort budget and attempt rules (frozen)

- `<= 600` added/modified maintained canonical lines of shared + adapter-wiring
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

[Gate](prototype-gate.md) · [Slice proposal](slice-proposal.md) ·
[Spikes](spikes.md) · [R27](../../../docs/research/2026-11-07/01-solution-first-native-slice.md)
