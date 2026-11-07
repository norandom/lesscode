# R27: demonstrate apparatus reuse before native expansion
- Report date: 2026-11-07, Europe/Berlin (amended 2026-11-07 after library and baseline complexity audit).
- Execution interval: prototype not run; read-only canonical baseline formatting audit run 2026-11-07T09:13–09:14+01:00.
- Status: planned (`solution_unlock = not-run`).
- Spec: [requirements](../../../.kiro/specs/deconvolution-native-core/requirements.md),
  [gate](../../../.kiro/specs/deconvolution-native-core/prototype-gate.md),
  [slice proposal](../../../.kiro/specs/deconvolution-native-core/slice-proposal.md),
  [research notes](../../../.kiro/specs/deconvolution-native-core/research.md).
- Hypothesis: H9 apparatus reuse, following H8/R25/R26.
- Type: exploratory proposal + empirical baseline complexity calibration; execution preregistration still required.

## Decision
Owner requested a smaller native slice, an explicit evidence unlock before substantial
code, and a validation of the proposed Rust libraries and complexity estimations. No
phase approval or prototype execution follows from this documentation amendment.

## Hypothesis and falsification
Shared E3/E4 apparatus preserves observations with `>= 20%` and `>= 200` canonical
physical lines removed on the replaceable apparatus-and-wiring stratum
($C_{\text{app}} \le 0.80 \cdot B_{\text{app}}, B_{\text{app}} - C_{\text{app}} \ge 200$),
`>= 200` canonical lines removed across the whole replacement ($C_{\text{total}} \le B_{\text{total}} - 200$),
unchanged seed/generator coverage ($C_{\text{seeds}} = B_{\text{seeds}}$), and strictly
less scope-specific support on each scope. Proposed prototype effort budget (`U7`):
`<= 600` added/modified maintained canonical lines of shared + adapter-wiring code
cumulatively across `<= 2` attempts. Failure of parity, controls, frozen-core reuse
or savings rejects `proceed`; stop at effort/attempt cap.

## Frozen inputs and method
Pending before S10 execution: frozen file/caller inventories, exact artifact/tool
identities (including `--artifact-root` bindings to untracked `.runs/` golden traces
verified against committed `.sha256` sidecars) and owner-approved bounded plan. Use E3,
freeze shared core, onboard E4. Count all support across three strata, report
tests/fixtures separately. `U1–U7` specify mandatory evidence.

## Evidence and results (Amendment 2: 2026-11-07 library and complexity audit)
No prototype experiment (`S9`, `S2`, `S10`) has run. R26's 6% canonical-line reduction,
seven oracle-blind differences, void differential and 47 new sequences within 74
held-out remain unchanged. Task 6.7 was already committed as `380c629`, correcting
stale `OP-4`.

A read-only empirical audit on 2026-11-07 measured the 14 E3/E4 baseline apparatus
files under the pinned canonical formatter (`ruff==0.17.3 format --isolated --line-length 100 --target-version py313`)
and inspected `native/src/` and `native/Cargo.lock`:

1. **Canonical formatting expands the baseline apparatus by +78.4% (`2,222` as-written $\to$ `3,963` canonical lines)**:
   - `e1b_oracle.py`: `406` written $\to$ `659` canonical (`145` shared `Norm`/helpers, `175` E3 `Env` ops, `174` `SEEDS`+generator, `159` generic runner).
   - `e4_oracle.py`: `534` written $\to$ `1,258` canonical (`160` E4 fixtures/helpers, `472` E4 `Env` ops, `470` `SEEDS`+generator, `150` generic runner).
   - Arm and replay scripts (`run_arm.py` 221/381, `run_e3.py` 44/63, `test_e3_replay.py` 33/58, `run_e4.py` 77/146, `test_e4_replay.py` 32/62): `407` written $\to$ `710` canonical.
   - Shared/sentinel checks (`receipt_atomic.py` 153/192, `editscope.py` 65/111, `patchcheck.py` 60/203, `scope.py` 78/105): `356` written $\to$ `611` canonical.
   - `regen_turn.py`: `337` written $\to$ `502` canonical (`~240` pure prompt/feedback/parse/write; `~262` HTTP streaming client).
   - `adaptive.py` (126/154) + `guard.py` (56/69): `182` written $\to$ `223` canonical.
2. **Three-stratum denominator refinement (`U5`)**:
   - In `e1b_oracle.py` and `e4_oracle.py`, `1,498` canonical lines (`83.3%` of `e4_oracle.py`) are scope-specific domain operation handlers (`op_*` + fixtures: `854` canonical lines) and transcribed test corpora (`SEEDS` + `generated`: `644` canonical lines), while the duplicated generic recorder/arm scaffolding across E3 and E4 is `~390` canonical lines (`~224` as-written).
   - Pooling `1,498` lines of irreducible domain/seed definitions into a single unstratified denominator would cap two-scope whole-replacement savings at `~11%` even if 100% of duplicated runner code vanished at zero replacement cost, or would create a perverse incentive to shrink `SEEDS`.
   - `U5` therefore partitions accounting into $B_{\text{app}}$ (replaceable apparatus & wiring, `~1,705` canonical lines), $B_{\text{ops}}$ (`~854` canonical lines) and $B_{\text{seeds}}$ (`~644` canonical lines), requiring $C_{\text{app}} \le 0.80 \cdot B_{\text{app}}$, $B_{\text{app}} - C_{\text{app}} \ge 200$, $C_{\text{total}} \le B_{\text{total}} - 200$, and $C_{\text{seeds}} = B_{\text{seeds}}$.
3. **Brownfield `native/src/` and verbatim-migration charging fix (`U7` & `complexity.md`)**:
   - Existing `native/src/` totals `6,149` physical lines across 6 files (`compare.rs` 1,667, `scope.rs` 1,598, `metrics.rs` 1,530, `lib.rs` 618, `model.rs` 511, `main.rs` 225). Charging `lib.rs` (618 lines) or verbatim-migrated `op_*`/`SEEDS` blocks (1,498 canonical lines) in full as "new prototype effort" would exceed the 600-line `U7` budget before writing a single new line of logic.
   - `U7` now charges all new/modified shared modules and adapter wiring in full, charges `<= 20`-line registration hooks in pre-existing `lib.rs`/`main.rs` by diff lines (or uses a dedicated binary/submodule), and verifies verbatim-migrated `op_*`/`SEEDS` blocks by AST/diff parity.
   - Full-suite component ceilings in `complexity-budgets.json` remain capped at `3,500` / `4,600` lines total, with internal ceilings rebalanced to match `rustfmt` reality (`reports` `100 -> 350`, `surface_measures_code` `500 -> 850`, `cli_config` `350 -> 450`; offset by `driver_shims` `900 -> 600`, `mutation_engine_readers` `600 -> 450`, `agnostic_core` `900 -> 750`, `host_guard` `300 -> 250`, `tree_hash_edit_scope` `300 -> 250`).
4. **Rust library audit corrections**:
   - **Retained / promoted from `native/Cargo.lock` (zero new deps)**: `serde 1.0.229`, `serde_json 1.0.154`, `sha2 0.11.0`, `clap 4.6.0`, `ast-grep-core 0.51.0`, `ignore 0.4.34`, `walkdir 2.5.0`, `globset 0.4.21`, `regex 1.13.1`, `thiserror 2.0.21`, `indexmap 2.14.2`, `toml_edit 0.25.17`, `syn 2.0.119/3.0.9`, `proc-macro2 1.0.107`, `windows-sys 0.61.2`.
   - **Dropped / replaced**: `serde_jcs 0.2.0` (RFC 8785 turns `-0.0`/`0.0` into `0` and emits raw UTF-8, breaking `S2` parity with Python `json.dumps`); `fs4 1.1.0` (`std::fs::File::lock` is in Rust 1.89+ `std`, and `receipt_atomic.py` uses `.lock` sidecars); `quick-junit 0.8.0` (XML writer only; replaced by `junit-parser` / `quick-xml`); `gix 0.90.0` (150+ crates); `pg_query 18.1.0` (MSVC C build failures; use pure-Rust `sqlparser`); `postgresql_embedded 0.22.0` (duplicates Facdrone's embedded PostgreSQL).
   - **Added to reuse map for checker & deferred phases**: `syn` + `proc-macro2` (`rust-syntax-units-v1` checker + Rust AST walks), `toml_edit` (TOML descriptors), `ruff_python_parser`/`ruff_python_ast` (exact Python `ast` node parity without C builds), `oxc` (`oxc_parser`/`oxc_semantic` for TS/JS), `rusqlite` (`bundled`, `trace`, `session` changesets for SQLite L1/L2 capture), and `arbtest`/`bolero`.

## Limits and interpretation
Known-scope reuse is not unseen-scope efficacy, language parity or equivalence.
Baseline canonical line counts are measured facts; replacement savings targets remain
proposals to freeze before `S10` execution and never lower post hoc.

## Tool/spec consequence and next gate
Approve bounded plan; S9 evidence is separate from ratification; execute S2/S10 only
as authorized. U1-U7 and independent review unlock normal design eligibility.
No candidate adoption or third regeneration scope.

## COBESY check
Target behavior: agents verify dependency semantics and baseline arithmetic before
freezing gates or expanding infrastructure. Challenge the assumption that proposed
crates or unstratified line percentages work without checking actual baseline code.
