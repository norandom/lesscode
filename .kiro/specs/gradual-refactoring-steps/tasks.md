# Implementation Plan: gradual refactoring in checked steps

Generated 2026-11-11. **Approved by the owner on 2026-11-11.**
Basis: [requirements](requirements.md) (approved 2026-11-11) and [design](design.md) (approved 2026-11-11);
gate `proceed` for the demonstrated subset
([R88](../../../docs/research/2026-11-11/14-gradual-steps-prototype-gate.md),
[R89](../../../docs/research/2026-11-11/15-first-tracked-step.md)).

Rules for every task group (owner rules of 2026-11-07 and 2026-11-10):

- Foreground execution; one commit per completed task on main; Rust tests with cargo-nextest; format check
  and lints before every push.
- Each group ends with the budgets check passing for every new file at the default ceilings (2,800 tokens,
  300 units, 560 tokens per function), a dated research entry where a result or a deviation exists, and
  `.kiro/task_N_handover.md`. A file over budget is split, not exempted.
- Group 1 can fail. If one of its tasks does, the design is revised and re-approved before group 2.
- Runs happen in copies under `D:\BuildArtifacts\lesscode\`; copies set `CARGO_TARGET_DIR` of their own.
- No model is called.

## Tasks

- [x] 1. Show the three assumptions the design rests on
- [x] 1.1 Limit a fault run to named functions and compare with a whole-file run
  - Run configuration accepts `functions` per file; sites outside them are not mutated and are counted as
    not selected in the receipt. Without the field the receipt is unchanged.
  - Compare, for `verdict` in `native/src/recording.rs`, the classes of the filtered run with those of the
    same faults in the whole-file run of R88.
  - Done when unit tests cover the filter and the comparison shows identical classes for every selected
    fault; a difference is recorded and stops the group.
  - _Boundary: DriverChanges_
  - _Requirements: 5.2_
- [x] 1.2 Copies from `git archive` build and test like a working tree
  - `step/revisions.rs`: resolve a revision, unpack `git archive` of a root into a work directory outside
    the repository, reject a work directory inside it or at a drive root.
  - Try it on this repository at two revisions and on the Java and TypeScript projects of R88: tests in the
    copy give the same result as in a working tree. Record what differs (line endings, ignored files,
    submodules).
  - Done when the comparison is recorded in a research entry and the unit tests pass.
  - _Boundary: Revisions_
  - _Requirements: 10.1, 10.4_
- [x] 1.3 Admit a pin against one fault through the driver
  - Apply one numbered fault of a base run to a copy that has the base code and the step's test paths; run
    the tests on the copy with and without the fault.
  - Repeat the pin of R89 this way: admitted. A test that does not fail on the fault: rejected, with the
    reason; a test that fails on the unchanged code: rejected, with the other reason.
  - Done when the three outcomes are reproduced and recorded.
  - _Boundary: Pin_
  - _Requirements: 3.1, 3.2_
- [x] 1.4 Record the gate result
  - Research entry with the three results, index updated, handover written.
  - _Requirements: 11.3_

- [x] 2. Driver: stored outcomes depend on the tests
- [x] 2.1 Put the test files into the identity of a stored outcome
  - Hash over the files under `test_paths` (relative path and content, sorted) joins the configuration
    identity. Old stores no longer match and are left alone.
  - Done when a test shows that a changed test file makes the affected mutants run again without
    `--no-reuse`, and an unchanged one reuses.
  - _Boundary: DriverChanges_
  - _Requirements: 9.1, 9.2_
- [x] 2.2 Update the driver's documents
  - `docs/driver.md`: the hazard note becomes the new behaviour. Amendment in the design of
    agnostic-annotation-driver.
  - _Requirements: 9.1_

- [x] 3. Declaration, functions and plan
- [x] 3.1 Read and validate the step declaration
  - `step/declaration.rs` and the `step` command group with its exit codes. Unknown fields, a base that is
    not an ancestor, an output path outside the repository: bad input.
  - _Boundary: Declaration, CommandLine_
  - _Requirements: 1.1, 1.3, 1.4, 4.2, 8.1_
- [x] 3.2 Functions: which changed, which call them, which fault belongs where
  - `step/functions.rs` from the pack's function boundaries: changed by token sequence, callers in scope by
    callee name, faults assigned by position; calls and state writes that leave a function.
  - The name scan that keeps language names out of the driver covers `native/src/step/` too.
  - _Boundary: Functions_
  - _Requirements: 1.5, 4.1, 5.2, 5.10_
- [x] 3.3 `step plan`
  - Per function: noticed and unnoticed faults, writes outside the frame, tokens; ranked; functions with no
    noticed fault marked and their outward calls listed. Runs in copies only.
  - Done when an integration test on a fixture repository checks the table and that the repository is
    unchanged.
  - _Boundary: Plan_
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 4.1, 4.3_

- [x] 4. Pins and the step check
- [x] 4.1 `step pin`
  - The product path of task 1.3, with the list of unnoticed faults that remain. No threshold on them; no
    test file is written.
  - _Boundary: Pin_
  - _Requirements: 3.1, 3.2, 3.3, 3.5, 3.6, 3.7_
- [x] 4.2 `step check`: the checks before any fault run
  - Checks 1 to 4 of the design: declaration and revisions, files outside the scope, pins and change in one
    commit, tests on base and step.
  - _Boundary: Check_
  - _Requirements: 1.2, 3.4, 5.1, 5.9_
- [x] 4.3 `step check`: fault runs and comparison
  - Checks 5 to 10: functions run, faults on base and step, comparison per function and with the ledger's
    number, unmatched functions, size, frame; functions out of reach reported as unprotected.
  - _Boundary: Check_
  - _Requirements: 4.4, 5.2, 5.3, 5.4, 5.5, 5.6, 5.10, 5.11, 5.12, 7.5_
- [x] 4.4 Verdict and evidence
  - `step/verdict.rs`: `step-verdict-v1` with sidecar, written for accepted, refused and not judged;
    `step-evidence-provisional-v0` from the commands tied to criteria.
  - Done when integration tests on fixture repositories produce an accepted verdict and one refusal for
    each of checks 2, 3, 4, 7, 8, 9 and 10, and a void run gives "not judged".
  - _Boundary: Verdict_
  - _Requirements: 5.7, 5.8, 8.2, 8.3, 8.4_

- [x] 5. Guard and ledger
- [x] 5.1 Ledger: append, chain, ratchet, release
  - `step/ledger.rs` and `step record`: entries with the hash of the line before; a broken chain is
    reported; last accepted number per function; `--release`; `--override --reason`.
  - _Boundary: Ledger_
  - _Requirements: 6.4, 7.1, 7.2, 7.3, 7.4_
- [x] 5.2 `step guard`
  - Recompute revisions and hashes, compare with the verdict; stale on a change to code, tests or
    declaration; overridden reported as overridden; a branch no declaration names is said not to be a step
    branch.
  - _Boundary: Guard_
  - _Requirements: 6.1, 6.2, 6.4, 6.5, 6.7_
- [x] 5.3 Hook and pipeline
  - `step install-hook` beside the budgets call; `kiro-cli check` in the hook where `tools/bin` has it;
    `docs/steps.md` with the pipeline job.
  - _Boundary: Guard_
  - _Requirements: 6.3, 6.6, 6.7_
- [x] 5.4 Protection tests
  - The repository tree and refs before and after every command differ only in the three declared outputs;
    the git arguments used contain no writing command.
  - _Requirements: 10.2, 10.3, 10.5_

- [ ] 6. Real steps and delivery
- [x] 6.1 The `verdict()` change of R89 as the first step through the tooling
  - Spec, declaration, branch, check, record, guard, on this repository. Owner decision 2026-11-11.
  - Done 2026-11-11: branch `step/one-exit` (`08f4a05` spec and declaration, `e64b7b3` the change, `0d54533`
    verdict, evidence, ledger). Accepted; guard exit 0. Copies of the outputs:
    `docs/experiments/gradual-steps/steps/one-exit/`. The branch is not merged; that is the owner's decision.
  - _Requirements: 11.2, 11.3_
- [ ] 6.2 One step in a foreign project in each of two languages
  - Copies of the Java and TypeScript projects used in R88; verdicts recorded, including any refusal.
  - _Requirements: 11.2, 11.3_
- [ ] 6.3 Documents and the soft layer
  - README section, `docs/steps.md` complete, a skill that walks the phases and calls the commands; a check
    that could not be made mechanical is listed, not moved into the skill.
  - _Requirements: 4.3, 6.6, 11.4, 11.5_
- [ ] 6.4 Final validation
  - `kiro-validate-impl`, `kiro-cli check`, research entry, handover; the evidence format proposed to
    openidea-kiro as an issue text for the owner to file.
  - _Requirements: 8.3, 11.3_
