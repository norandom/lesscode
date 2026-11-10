# Implementation Plan: agnostic driver in `lesscode-core`

Generated 2026-11-10. **Approved by the owner on 2026-11-10.** Owner decision 2026-11-10 ("3.4 - yes"): the five gate scope files may be added to the repository as test fixtures.
Basis: [requirements](requirements.md) and [design](design.md), both approved 2026-11-10; gate `proceed`;
sizing spike [R65](../../../docs/research/2026-11-10/11-annotation-driver-rust-sizing-spike.md).

Rules for every task group (owner rules of 2026-11-07 and 2026-11-10):

- Foreground execution; one commit per completed task on main; Rust tests with cargo-nextest.
- Each group ends with measured lines per new file against the design's table (design C8), a dated research
  entry where a result or a deviation exists, and `.kiro/task_N_handover.md`. A file over budget is split,
  not exempted.
- Nothing is deleted from the existing adapters before group 7.
- The reference for ported code is the spike under `docs/experiments/annotation-driver/native-spike/`; it is
  ported into the crate's structure, not copied wholesale (admission is rewritten on the typed pack).
- Slice B: the handover of group 2 must contain the preregistration of the drift baseline (see task 2.4).

## Tasks

- [x] 1. Foundation: grammars, packs and the read-only helpers
- [x] 1.1 Switch on the Rust and Java grammar features and record the dependencies
  - Add the two features of the pinned parser crate; update the lock file, `LICENSES.md` and
    `THIRD_PARTY_NOTICES.txt`.
  - Done when the crate builds with `--locked` on the local machine and the existing suite passes unchanged.
  - _Requirements: 8.1, 8.4_
- [x] 1.2 Add the four packs and the typed pack
  - Copy the four version 1 packs to `native/packs/`; move the attribute marker of the test-region rule into
    the pack as `test_attribute_kinds`.
  - `driver/pack.rs`: typed pack, embedded lookup by language, loading from a directory; an unknown language
    is a distinct "not supported" result.
  - Done when unit tests load all four embedded packs and reject an unknown language and a mistyped pack.
  - _Requirements: 1.1, 1.2, 1.6_
- [x] 1.3 (P) Add the tree and source helpers
  - `driver/tree.rs` (parse without panicking, wrapper chains, child and token helpers) and
    `driver/source.rs` (file reading that keeps line endings, the test-path rule).
  - Done when unit tests cover CRLF input, non-ASCII columns and a file that does not parse.
  - _Boundary: driver/tree.rs, driver/source.rs_
  - _Requirements: 2.5, 2.6_
- [x] 1.4 Register the driver files in the complexity budgets
  - Add each new file to `native/complexity-budgets.json` at the default ceilings; the budgets test passes.
  - _Requirements: 8.2, 8.3_

- [x] 2. Pack admission and the `admit-pack` command
- [x] 2.1 Implement admission on the typed pack
  - Size limits, typed shape, every node kind accepted by the grammar, placeholders limited to the three
    allowed names; results cached by pack hash; failed checks named as in the prototype (`size`, `shape`,
    `kinds_valid`, `templates`).
  - Done when the four shipped packs and the two model-written packs of attempt 2 are admitted and four
    deliberately wrong packs are rejected for the recorded reasons.
  - _Requirements: 1.3, 1.4, 1.5_
- [x] 2.2 Add control replay to admission
  - With a control file and a source root, each control must behave as recorded and each control site with an
    operator must yield a parseable change. Depends on the site rules of 3.1 for the replay itself; the
    command accepts the options and reports "controls not checked" until 3.1 lands.
  - State after group 2: the options are accepted and a pack checked with them is not admitted (fail closed,
    exit 1, check `controls` reported as not run). The replay itself is part of task 3.2.
  - _Depends: 3.1_
  - _Requirements: 1.3, 9.3_
- [x] 2.3 Add `deconvolute admit-pack`
  - New handler file `cli_driver.rs`; exit 0 admitted, 1 rejected, 3 bad input; nothing written on failure.
  - Done when CLI tests cover the three exit codes and an embedded-pack test admits every shipped pack
    against the crate's grammars.
  - _Requirements: 1.5, 1.6, 8.4_
- [x] 2.4 Preregister the Slice B drift baseline
  - Write the preregistration entry for pack-authoring drift (five authoring runs per language for four
    languages with the frozen pack guide, current endpoint first, Mistral second; measures: admission rate,
    repair attempts, identity of the mutants from admitted packs; cost bound inside the 20 USD cap) in
    `docs/research/` and link it from the roadmap and `constraint-guided-compression`.
  - No model call is made by this task. Running the baseline is Slice B work and needs the owner's go.
  - Done when the entry exists, is indexed, and the group 2 handover names it.
  - _Requirements: 9.6_

- [ ] 3. Engine: sites, annotations and operators
- [x] 3.1 Port the site rules and eligible sites
  - `driver/sites.rs`: the rules of the gate as amended (2c, 10a, 12), same order and de-duplication.
  - Done when the driver lists exactly the retained prototype sites for the five scopes.
  - _Requirements: 2.1, 2.2, 2.6, 9.1_
- [ ] 3.2 Port annotation validation
  - The five rules with the ten reason strings; terminator completion and statement unwrapping.
  - Done when all 98 controls give the recorded outcome.
  - _Requirements: 6.1, 6.2, 6.3, 9.3_
- [ ] 3.3 Port the operators and mutant classification
  - `driver/operators.rs`: operators in fixed order, replacement by exact offsets, classification as
    changed, no-op or parse error; a swap of the site's own equality operator is dropped when the negation
    is present; seeded selection with a generator defined in the code.
  - Done when mutants equal the retained receipts by site, operator and text hash for the five scopes, with
    the dropped duplicates listed and explained in a research entry (an intended difference under 9.2).
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.7, 9.1, 9.2_
- [ ] 3.4 Add the parity tests and the name scan
  - Integration tests over the retained receipts and controls, needing no Python. The five scope files are added as
    fixtures at their gate commits (owner decision 2026-11-10), each with its origin, commit and licence noted,
    with line endings pinned; a missing reference is reported as "not run", never as passed.
  - A test scans the engine sources and packs for language names, repository names and the gate's file
    names.
  - _Requirements: 1.7, 9.1, 9.3, 9.5_

- [ ] 4. Switch `analyze` and `mutate` to the driver
- [ ] 4.1 Move the handlers out of `cli_deconvolute.rs`
  - `analyze` and `mutate` move to `cli_driver.rs` unchanged in behaviour; `cli_deconvolute.rs` drops well
    below its ceiling. Existing CLI tests pass.
  - _Requirements: 8.3_
- [ ] 4.2 Route `analyze` through the driver for four languages
  - Receipt `deconvolution-analysis-v2` with sites by kind, write sites, function boundaries, declared gaps
    and the pack hash; frame specification applied as before; exit 0, 1 or 3 as designed. A Python adapter
    module is added with the same shape as the others.
  - Done when the Python "not implemented" refusal is gone and replaced by a real result, and CLI tests cover
    each exit code per language.
  - _Requirements: 2.1, 2.3, 2.4, 2.5, 7.1_
- [ ] 4.3 Route `mutate` through the driver for four languages
  - Receipt `deconvolution-mutation-v2`; mutant files written to the output directory only; exit 1 with no
    mutants; `--limit` and `--seed`.
  - Done when CLI tests show mutants for all four languages and the source file is byte-identical afterwards.
  - _Requirements: 3.5, 3.6, 3.7, 5.1, 7.2_

- [ ] 5. Workspace and safety
- [ ] 5.1 Implement copies and the refusal rules
  - `driver/workspace.rs`: recursive copy without version-control metadata and configured exclusions; setup
    commands; refusal when the work directory is the source root, inside it, or contains it.
  - _Requirements: 5.1, 5.2_
- [ ] 5.2 Implement the restore journal
  - Save the original and a journal entry before a mutant is written; restore, verify the hash and clear the
    entry afterwards; recover found entries at start and report them; stop with a failure when verification
    fails.
  - Done when tests show a source tree left byte-identical (run against a read-only tree), recovery after a
    run killed in the middle, and a stop on a forced hash mismatch.
  - _Requirements: 5.3, 5.4, 5.5, 5.6_

- [ ] 6. Batch runner, outcome store and `deconvolute run`
- [ ] 6.1 Add the run configuration
  - `driver/config.rs`: commands as argument vectors, environment, limit floor, ceiling and factor, exit
    codes and output patterns for the classes, setup commands, exclusions, workers, `serial_only`.
  - _Requirements: 4.1, 4.4_
- [ ] 6.2 Implement the single-worker protocol and classification
  - Baseline before and after; scaled limit; classes killed, survived, invalid, timeout, error; one retry
    for errors; process trees killed on every exit path; reuse of the crate's existing process supervision.
  - Done when toy-project tests produce each class, including a command that cannot start and a
    resource-failure pattern, and a failing baseline voids the run.
  - _Requirements: 4.2, 4.3, 4.5, 4.6, 5.7_
- [ ] 6.3 Add the worker pool
  - Threads with one copy each on a shared queue; a worker with a failed baseline returns its mutants;
    void when a mutant could not be run or a closing baseline fails.
  - Done when a parallel run gives the same classes as a serial run on the toy projects.
  - _Requirements: 4.7_
- [ ] 6.4 (P) Add the outcome store
  - `driver/store.rs`: key from original hash, mutated-text hash and configuration; atomic write; errors and
    void runs never stored; `--no-reuse`.
  - _Boundary: driver/store.rs_
  - _Requirements: 4.8_
- [ ] 6.5 Add `deconvolute run` and its receipt
  - Receipt `deconvolution-run-v1`; `--annotations` restricts the run to validated sites; exit 0 complete,
    2 incomplete, 3 void or bad input.
  - Done when CLI tests cover the exit codes and an annotated run mutates only accepted sites.
  - _Requirements: 4.9, 6.4, 6.5_

- [ ] 7. Migration: remove the specimen-bound rules
- [ ] 7.1 Map the specimen mutation kinds onto driver operators
  - Five kinds mapped as designed; `TruncatedStream` has no counterpart and the loss is recorded.
  - _Requirements: 7.3, 7.5_
- [ ] 7.2 Rerun the specimen experiments through the driver
  - The experiments that cited "four of four adverse controls killed" (R49 to R53) are rerun; new figures are
    recorded beside the old ones in a dated research entry and as amendments to the cited entries.
  - _Requirements: 7.4, 9.4_
- [ ] 7.3 Delete the six adapter files and their substring tests
  - Remove `adapters/{rust,java,typescript}/{analysis,mutation}.rs` and the tests that pin their text
    matching; keep the specimens as fixtures; update the budgets file.
  - Done when the suite passes and a search finds no fixed specimen substring in `native/src`.
  - _Requirements: 7.1, 7.2, 7.3_
- [ ] 7.4 Update the documentation
  - README, native README, adoption guide and the deconvolute skill: four languages, the new commands, and
    the narrowed statement that only `deconvolute run` executes a project's commands, in copies.
  - _Requirements: 7.5, 8.6_

- [ ] 8. Validation and close
- [ ] 8.1 Run one full batch per language with the native driver
  - The five gate scopes through `deconvolute run`; classes compared with the attempt 2 receipts; every
    difference explained. Linux-only runs use containers; the Podman machine is not managed by the agent.
  - _Requirements: 9.1, 9.2, 9.4_
- [ ] 8.2 Verify the five platforms
  - The existing CI matrix through the Dagger configuration is green with the driver; binary size and build
    time recorded against R65.
  - _Requirements: 8.5, 8.6_
- [ ] 8.3 Final validation and Slice B check
  - `$kiro-validate-impl`; a research entry with the outcome; confirm that the Slice B baseline has a dated
    status (run, scheduled with a date, or stopped by the owner). The spec is not reported complete without
    it.
  - _Requirements: 9.4, 9.6_

## Not in this plan

- Fewer non-compiling mutants in typed languages, non-literal failure returns, faster Rust scopes.
- Model calls of any kind, pack authoring, prompt optimisation (Slice B).
- A fifth language. It needs a grammar feature and an admitted pack, and is a separate decision.
