# Implementation Plan: constraint-guided-compression (Slice A production + G1-R bounded spike)

Scope note. Production tasks cover **Slice A** (the demonstrated apparatus:
surface derivation, scope measures, test-infrastructure detection, and the
stdlib single-turn/two-tier regenerator controller with `n_rollouts_per_turn=1`
default, plus parameterized rollout width so task 6 can execute the `G1-R`
spike directly against the module). Slice B production adoption claims and the
optional `DspyRefineAdapter` (`--backend dspy`) remain withheld pending the
`G1-R` spike per `Req 7.10` and `prototype-gate.md`; task 6 is that bounded
spike, nothing more. Every language-specific or domain-specific analysis step
stays behind the per-language extractor seam (`design.md` "Deferred Target
Languages & Rust Ecosystem Foundation", `requirements.md` Amendment 2,
2026-11-07): `regenerator.py` never imports `ast`, prompt labels and code
fences derive from `TurnRequest.language`, and adding TypeScript or Rust is a
new sibling extractor module plus tests (leveraging `ast-grep-core` /
`tree-sitter-typescript` and `syn` already locked in `native/Cargo.lock`, or
`oxc_semantic` / `ruff_python_ast`), never a rewrite of controllers, reward
ladder, receipts, or regenerator. Tasks marked "owner go" spend model budget
and start only on the owner's explicit answer. Frozen apparatus
(`recording.py`, `native/src/*.rs`, frozen experiment drivers) is never edited.

- [x] 1. Foundation: create module skeletons and register package exports
  - Create minimal stdlib-only skeletons for `surface.py`, `measures.py`,
    `testinfra.py`, and `regenerator.py` in `src/lesscode_baseline/`, and
    export all four modules from `src/lesscode_baseline/__init__.py` (`cli.py`
    untouched because its subcommands and exits are an evidence contract).
  - Wire the `python -m lesscode_baseline.regenerator` CLI entry point skeleton
    with the preserved exit codes 0/2/3/4/5/6 and keep top-level imports
    stdlib-only; lazy external tools (`ruff` via `uv`, endpoint SDK, `dspy`)
    stay inside their owning call sites.
  - Done when `python -m lesscode_baseline.regenerator --help` prints usage
    with exit 0, all four modules import without side effects or a specimen
    environment, and the existing package test collection is unchanged.
  - _Boundary: SurfaceDeriver, ScopeMeasurer, TestInfraDetector, ProbabilisticRegenerator_
  - _Requirements: 2.1_

- [ ] 2. Surface derivation and candidate verification (Contract B)
  - [x] 2.1 (P) Contract-B surface derivation and syntax check behind the extractor seam
    - Implement `derive_surface` and `check_syntax` in
      `src/lesscode_baseline/surface.py` from explicit `scope_files` plus
      `recorder_files` (Contract B: symbols the recorder oracle imports are
      part of the contract), reporting callable kinds, unresolved targets,
      source anchors, and extractor version; keep all AST parsing inside this
      seam (`check_syntax`) so `regenerator.py` never imports `ast`, and carry
      no module-global scope state.
    - Done when the derived report for the retained E4 material includes the
      recorder-imported symbol (`labeled_reason`), marks every unresolvable
      target as `unresolved` rather than absent, and `check_syntax` reports
      line-anchored syntax errors without importing candidate code.
    - _Boundary: SurfaceDeriver_
    - _Depends: 1_
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 3.1, 3.3, 7.1_
  - [x] 2.2 Candidate surface verification for Tier-1 rejection
    - Implement `verify_candidate_surface` in `src/lesscode_baseline/surface.py`
      to check a candidate tree against the surface report: missing symbols,
      kind mismatches, and local rebinding of `patched` module globals
      (parameter shadowing, local assignment, default-arg capture), returning
      the structured `SurfaceVerification` pass/fail result the static gate
      consumes.
    - Done when each violation class is reported with the offending symbol
      and location, and a clean candidate passes.
    - _Boundary: SurfaceDeriver_
    - _Depends: 2.1_
    - _Requirements: 2.5, 5.7_
  - [x] 2.3 Surface unit suite (offline)
    - Create `tests/test_surface.py` covering Contract A vs Contract B
      (recorder-imported symbol case), `check_syntax`, the three
      candidate-verification violation classes, and the `patched` rebind modes.
    - Done when the suite runs green offline on synthetic trees with zero
      specimen dependencies.
    - _Depends: 2.1, 2.2_
    - _Requirements: 7.1_

- [ ] 3. Scope measures: three readings, structure, pure share, and edit scope
  - [x] 3.1 (P) As-written measures, pluggable EffectProfile structure check, and pure share
    - Implement `measure_written` and statement-level `measure_pure_share` in
      `src/lesscode_baseline/measures.py`, separating the default
      SQLAlchemy/database `EffectProfile` constants (`WRITE`, `BAD`, `SQL`)
      from the AST walker: count `lines`, `stmts`, `funcs`, `qualnames`,
      `writes`, intra-scope call `edges`, `module_hops`, and statement-level
      pure/shell ratios (`pureshare`), and validate declared `structure` (write
      sites confined to `shell`, no forbidden DB parameters or SQL attribute
      calls in `pure` functions, undeclared functions rejected).
    - Done when `measure_written` and `measure_pure_share` for the retained
      E3/E4 scope files reproduce the frozen readings and a structure violation
      is reported with its site.
    - _Boundary: ScopeMeasurer_
    - _Depends: 1_
    - _Requirements: 2.2, 2.3, 3.1, 6.2_
  - [x] 3.2 Separated-form v1 and canonical reading
    - Implement `measure_separated` (the nine separated-form statement rules
      with the exact `separated - sum(extra) == written` identity) and
      `measure_canonical` (isolated formatting via pinned `ruff==0.17.3`) in
      `src/lesscode_baseline/measures.py`; when the formatter is unavailable,
      report the canonical reading as `None`, never `0`.
    - Done when all nine rules hold on the rule table cases and the canonical
      reading is `None` (not `0`) with the formatter unavailable.
    - _Boundary: ScopeMeasurer_
    - _Depends: 3.1_
    - _Requirements: 6.2, 6.3_
  - [x] 3.3 Composition entry point, language-parameterized edit scope, and measure gaps
    - Implement `measure_all`, `check_edit_scope` (enforcing SHA-256 byte
      identity outside `scope_files` plus scanning `mentions_files` for
      out-of-scope source file tokens, parameterized by `source_extensions` and
      `ignore_dirs` so non-Python extensions require no rewrite), and
      `compute_measure_gaps` in `src/lesscode_baseline/measures.py`.
    - Done when one `measure_all` call reproduces the three readings plus gaps
      for the frozen scope, and `check_edit_scope` refuses both out-of-scope
      file modifications and out-of-scope file mentions with the offending path.
    - _Boundary: ScopeMeasurer_
    - _Depends: 3.1, 3.2_
    - _Requirements: 1.5, 2.6, 3.6, 6.1, 6.4_
  - [x] 3.4 Measures unit suite (offline)
    - Create `tests/test_measures_scope.py` covering `measure_written`,
      `EffectProfile` structure validation, `measure_pure_share`, all nine
      separated-form rules with the base-plus-extra identity, canonical
      unavailability (`None`), `check_edit_scope` (hash mismatch and
      `mentions_files` token refusal), and `compute_measure_gaps`.
    - Done when the suite runs green offline and exercises every rule with
      its own case.
    - _Depends: 3.1, 3.2, 3.3_
    - _Requirements: 7.1_

- [ ] 4. Test-infrastructure detection as a package module
  - [x] 4.1 (P) Versioned detector relocation and cross-language envelope
    - Relocate the demonstrated detector into `src/lesscode_baseline/testinfra.py`
      as a versioned module (the frozen experiment original stays untouched),
      preserving the cross-language top-level report envelope (`runner`,
      `fixtures`, `external_store`, `scope_references`, `seeds`,
      `marker_expression`, `deselections`, `rule_version`) while keeping
      pytest-specific keys scoped as runner subfields; non-Python repositories
      return `runner.name = "unknown"` with zero seeds cleanly.
    - Done when the relocated module's report on the retained validation
      inputs equals the demonstrated output and rule versions are recorded.
    - _Boundary: TestInfraDetector_
    - _Depends: 1_
    - _Requirements: 1.3, 3.5, 7.6_
  - [x] 4.2 Detector unit suite (offline)
    - Create `tests/test_testinfra_unit.py` covering config discovery across
      `pyproject.toml`, `pytest.ini`, and `setup.cfg`, fixture and plugin
      resolution, marker exclusion rules, store-fact and POSIX-hazard
      detection, and clean `runner.name = "unknown"` behavior on a non-Python
      layout.
    - Done when the suite runs green offline on synthetic repositories,
      including one non-Python layout case.
    - _Depends: 4.1_
    - _Requirements: 7.1_

- [ ] 5. Probabilistic regenerator: single-turn base with two-tier reward
  - [x] 5.1 Language-parameterized turn contracts, endpoints, and admission-ready receipts
    - Implement `TurnRequest` (with `language: str = "python"` driving prompt
      language labels and markdown code fences), `TurnResult`, `Endpoint`
      (`anthropic` streaming and `openai` JSON-schema HTTP with key redaction
      everywhere), `build_prompt`, `prepare_candidate`, `write_candidate`, and
      `hash_set` in `src/lesscode_baseline/regenerator.py`, plus the four `R26`
      amendments (Contract-B surface, preregistered parameters, LF-byte hashes,
      case-distinct labels) and an internalized `.lock` atomic receipt writer
      emitting `receipt.json.sha256` and `bindings.json` so `lesscode-core
      admission` verifies runs directly.
    - Done when a recorded turn round-trips through `receipt.json` +
      `bindings.json` with all five hash fields and `.sha256` sidecars over raw
      stored LF bytes, and no key material appears in any serialized output.
    - _Boundary: ProbabilisticRegenerator_
    - _Depends: 2.2_
    - _Requirements: 1.1, 1.2, 3.2, 3.3, 7.3, 8.5_
  - [x] 5.2 Tier-1 static gate and adaptive schedule
    - Implement `evaluate_tier1_static` (rungs `0.00` transport/stop, `0.10`
      JSON schema, `0.20` scope refusal, `0.30` syntax via
      `surface.check_syntax`, `0.40` Contract-B surface and `patched` rebinding
      via `surface.verify_candidate_surface`, `0.45` `pure`/`shell` structure,
      `0.49` static targets, `0.50` pass) and `next_rollout_params`
      (thinking-budget recovery, bounded temperature steps, missing-symbol
      injection into feedback) in `src/lesscode_baseline/regenerator.py` so
      static failures never launch a runtime oracle and `regenerator.py` never
      imports `ast`.
    - Done when every ladder rung reports its score and diagnostics in under
      two seconds on the frozen failure artifacts with zero runtime calls.
    - _Boundary: ProbabilisticRegenerator_
    - _Depends: 5.1, 2.2, 3.3_
    - _Requirements: 4.2_
  - [x] 5.3 Tier-2 runtime gate wrapping recording.py with pinned partial credit
    - Implement `evaluate_tier2_runtime` in
      `src/lesscode_baseline/regenerator.py`, wrapping the frozen
      `src/lesscode_baseline/recording.py` functions (`probe`, `run_groups`,
      `replay_engine`, `replay_to_l2`, `verdict`) to compute
      `seed_group_pass_fraction`, `sequence_pass_fraction`, and
      `first_divergence` without modifying `recording.py`; apply the pinned
      partial-credit formula `0.60 + 0.25 * (0.5 * g + 0.5 * s)` with
      fail-closed missing keys, keep `runtime_infrastructure` (`exit 124`/`2`)
      separate from behavioral kills or equivalence, and enforce the feedback
      content boundary (divergence identities and measure gaps only, never
      held-out sequence names).
    - Done when the formula reproduces the documented scores on synthetic
      fractions, a missing fraction key scores as zero, and a payload
      containing a held-out name is rejected before dispatch.
    - _Boundary: ProbabilisticRegenerator_
    - _Depends: 5.2_
    - _Requirements: 4.2, 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 6.5_
  - [x] 5.4 Stdlib Refine/BestOfN controller: budgets, stops, and outcome separation
    - Implement `run_refine_loop` and `verify_holdout_disjoint` in
      `src/lesscode_baseline/regenerator.py`: outer turns, parameterized
      `n_rollouts_per_turn` (`1` default for Slice A; `>1` exercised by `G1-R`
      while `DspyRefineAdapter` remains withheld), inner static retries,
      budget and `no_progress_limit` stops on outer turns, best trajectory
      retention on exhaustion, candidate-tree restoration from `frozen_root`
      before each write, resume identity re-verification, and `RefineOutcome`
      keeping validity, compression, and adoption (`adopted=False`) separate.
    - Done when the simulated four-turn failure-and-recovery sequence reaches
      acceptance, exhaustion retains the best trajectory, a hash mismatch
      aborts with exit 4, and outcome fields never conflate.
    - _Boundary: ProbabilisticRegenerator_
    - _Depends: 5.3_
    - _Requirements: 1.4, 1.5, 3.4, 4.1, 4.3, 4.4, 4.5, 4.6, 4.7, 6.6, 6.7, 7.4, 7.11_
  - [x] 5.5 Regenerator unit suite (offline)
    - Create `tests/test_regenerator.py` covering key redaction and payload
      formatting (`anthropic` and `openai`), `TurnRequest.language` prompt
      fences, the six adverse controls (five frozen plus `ctrl_feedback_leak`)
      returning $R < 0.50$ with zero runtime evaluator invocations, simulated
      multi-turn recovery, exhaustion stops, `bindings.json` + `.sha256`
      emission, and `verify_holdout_disjoint` rejection.
    - Done when the suite runs green offline with every control failing
      closed and no runtime oracle call recorded for Tier-1 controls.
    - _Depends: 5.4_
    - _Requirements: 5.8, 7.5_

- [ ] 6. G1-R bounded spike (Slice B gate; owner go for 6.2)
  - [ ] 6.1 Deterministic G1-R verification driver
    - Build `docs/experiments/deconvolution/probabilistic-refine/g1r_spike.py`
      discharging the deterministic `prototype-gate.md` checklist on frozen
      E3/E4 artifacts: all six Tier-1 controls fail closed without runtime
      invocation, the recorded three-turn transition replay (E4 Turn 1
      `max_tokens`, Turn 2 missing `labeled_reason` under Contract B, Turn 3
      acceptance) passes Tier 1 in under two seconds per check, `k=2`
      no-progress exhaustion halts with the best trajectory, replaceable
      apparatus (`recording.py`, `surface.py`, `measures.py`, `regenerator.py`
      + `admission.rs`, `receipt.rs`, hooks) stays within the 20% ceiling
      ($\le 2,270$ canonical lines) against the frozen expanded baseline
      stratum ($B_{\text{app}}^{\text{expanded}} = 2,838$ canonical lines, with
      `testinfra.py` at $1,081$ canonical lines accounted under `U7` verbatim
      relocation), and `lesscode-core admission` verifies every emitted receipt
      and `.sha256` sidecar byte-exactly via `bindings.json`; retain negative
      results in the gate record.
    - Done when the driver runs the full deterministic checklist end-to-end
      and writes the gate evidence record with every item pass or fail.
    - _Boundary: ProbabilisticRegenerator, SurfaceDeriver, ScopeMeasurer_
    - _Depends: 2.3, 3.4, 4.2, 5.5_
    - _Requirements: 5.8, 7.2, 7.8, 7.9_
  - [ ] 6.2 Live weaker-model recovery run (owner go)
    - Execute at least one live non-deterministic or weaker-model run that
      recovers from a Tier-1 or Tier-2 failure to full acceptance ($R = 1.0$)
      within the frozen turn (`max_turns <= 4`) and retry
      (`max_static_retries <= 3`) budgets, appending receipts and recording
      cost.
    - Done when the run's receipts show a recorded failure followed by
      acceptance within budget and the gate record cites it.
    - _Boundary: ProbabilisticRegenerator_
    - _Depends: 6.1_
    - _Requirements: 7.9_

- [ ] 7. Final verification and evidence records
  - [ ] 7.1 Full-suite verification, self-trial checks, and record updates
    - Run the four new offline suites (`tests/test_surface.py`,
      `tests/test_measures_scope.py`, `tests/test_testinfra_unit.py`,
      `tests/test_regenerator.py`) together with the existing package suites
      (`pytest -n 4`) and native self-checks (`cargo nextest`, `lesscode-core
      budgets`); confirm the evaluator-package evidence rules (stable records,
      exits, no duplicate representations) hold; update the spec records and
      `docs/research/` with the dated verification evidence and reciprocal
      research-log links.
    - Done when every suite is green in one fresh run, no frozen file or
      receipt changed (hash check), and the spec and research records carry
      the dated evidence with planned, executed, and measured states kept
      distinct.
    - _Depends: 6.1_
    - _Requirements: 7.7, 8.1, 8.2, 8.3, 8.4, 8.6_

## Deferred (documented, not scheduled)

- TypeScript extraction evaluation and further target languages (Rust, JS):
  deferred by `requirements.md` Amendment 2 (2026-11-07); the language
  extractor seam (tasks 2.1, 3.1, 3.3, 4.1, 5.1 keep all AST and extension
  rules behind versioned extractors) makes each addition a new sibling module
  plus suite, reusing `ast-grep-core` / `tree-sitter-typescript` and `syn` +
  `proc-macro2` already compiled in `native/Cargo.lock` (or `oxc_semantic`,
  `ruff_python_ast`, and `junit-parser` when deep scope/test analysis is
  needed). G0-era TypeScript fixtures are preserved as that stage's first
  material, subject to revalidation.
- Slice B production adoption tasks (multi-rollout default adoption, optional
  `DspyRefineAdapter` `--backend dspy` and offline `MIPROv2` compilation):
  withheld pending `G1-R` outcome per `Req 7.10` and `prototype-gate.md`.
- _Requirements: 7.1, 7.10_

## Implementation Notes

- Validation must run from `D:\Source\lesscode`: `C:\Users\mariu\Source` is a junction onto `D:\Source`, and the efficiency-baseline integrity layer refuses reparse-point traversal (67 apparent failures vanish when run from the D: root).
- Pre-existing on this machine, unrelated to this spec: 6 Windows process/timeout tests fail deterministically from the D: root (`tests/test_process.py` x3-5, `test_adverse_evaluation.py::test_timeout_of_child_spawning_check_retains_diagnostics`, `test_checks.py::test_timeout_after_report_preserves_raw_but_cannot_pass`); that code was last touched at ac68307 (efficiency-baseline). Owner WIP suites may fail independently (mid-work).
- Test venv: `.runs/e2-facdrone/venv` (pytest 9.0.3, xdist 3.8.0); direct module runs need `PYTHONPATH=src`.
