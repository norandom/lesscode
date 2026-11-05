# Implementation Plan: deconvolution-loop (bounded revision and prototype tasks)

Scope note. The prototype gate (`prototype-gate.md`) is `proceed` only for
the loop as a research instrument on one Python scope and `revise` for the
size claim, the sensitivity target, the missing-test report and
test-infrastructure detection. Under `.kiro/steering/prototype-first.md`
these tasks are bounded apparatus revisions, one prototype (the
regenerator program, requirements 6.7 and 6.8, whose evidence is E4) and
one preregistered experiment. No native implementation task is generated.
The sensitivity bar of 0.70 is unchanged; tasks 3.2 and 6.4 re-measure it
and never lower it. Frozen E3 apparatus files are never edited; new
behaviour goes into new versioned files beside them. Tasks marked
"owner go" start only on the owner's explicit answer, because they spend
heavy machine time or model budget, or they fix a scope. The go answered
in 6.1 covers 6.2 to 6.7. "The report" is
`docs/reports/deconvolution-loop-applied.md`; its section 8 holds the
predictions E4 reuses. The research-code line budget in 6.1 covers every
apparatus file authored in 6.1, 6.4 and 6.5.

- [ ] 1. Foundation: measurement and run discipline the later tasks rely on
- [ ] 1.1 (P) Canonical-format size measures beside the frozen measurer
  - Add a new versioned measurer that reports statements and lines twice:
    as written, and after both trees are formatted with one declared
    formatter and settings recorded in the result file. The frozen E3
    measurer stays untouched.
  - Dry-run check: the new measurer's as-written counts equal the frozen
    measures for the E3 baseline and candidate.
  - Re-measure the E3 baseline and candidate; add a dated amendment to R25
    and to the E3 plan that adds the canonical figures as a separate
    labelled reading next to the original T2 verdict. Targets are not
    changed.
  - Done when both E3 measures files carry canonical counts for both trees,
    the as-written counts match the frozen files, and R25's amendment
    names the canonical statement and line totals.
  - _Boundary: Measurer_
  - _Requirements: 4.1, 4.2, 4.6, 10.3, 11.1, 11.7_
- [x] 1.2 (P) Host guard and atomic receipt as apparatus
  - Promote the idle-wait and memory-floor guard from a scratch script to
    a versioned apparatus file with its thresholds recorded, used by every
    later heavy step.
  - Make receipt writes append-only and atomic so concurrent steps cannot
    lose entries; keep the existing receipt format readable.
  - Done when two concurrent dummy steps each write ten receipt entries and
    all twenty are present, and a guard note precedes a dummy heavy step
    in the receipt.
  - _Boundary: Receipt, HostGuard_
  - _Requirements: 11.2, 11.3, 11.4, 11.5, 12.3_

- [ ] 2. Core: turn the E3 residue into artefacts the workflow can consume
- [ ] 2.1 (P) Missing-test report from survivors and realised counterexamples
  - Read the recorded sensitivity outcomes and the differential replay
    records (both existing E3 result files) and emit one report that names,
    per item, the function, line, mutated or differing condition and the
    input class that would distinguish it; survivors judged equivalent are
    listed separately with the reason.
  - Run it on the E3 results and keep the report beside them.
  - Preregister, then run, a usefulness check outside the Loop: a fresh
    agent given only the report and the original tree writes tests; the
    tests must pass on the original and, applied in the isolated mutant
    copy, kill each listed real gap (prediction: the three real T4 gaps and
    the exception-path difference). The written tests are an experiment
    artefact and are discarded afterwards; the Loop never adds tests to
    the specimen.
  - Done when the report exists for E3, a receipt shows which listed gaps
    the experiment's tests killed, and the prediction is reported as held
    or failed.
  - _Boundary: ResidueReport_
  - _Depends: 1.2_
  - _Requirements: 8.7, 10.1, 10.3, 11.1, 11.2_
- [ ] 2.2 (P) Test-infrastructure detection for a Python repository and scope
  - Given a repository root and a scope, detect the test runner, its
    configuration and markers, fixture and plugin files, external-store
    setup, and the test files that import or reference the scope; emit one
    JSON description and a readable summary. Baseline-failing nodes are
    taken from a supplied recorded list (the E1b baseline-failure file for
    Facdrone), not from running the suite.
  - Validate against the hand-built E3 lists (191 seed files, 19 importing
    files, 236 baseline failures of which 24 applicable, the marker
    expression); every difference is explained or fixed.
  - Done when detection output for the Facdrone specimen matches the hand
    lists or every difference is recorded with its cause.
  - _Boundary: TestInfraDetector_
  - _Depends: 1.2_
  - _Requirements: 12.2, 12.4, 3.1_
- [ ]* 2.3 Detection on the Rust specimen's runner
  - Optional follow-up outside the gate's Python scope: run detection on
    the Rust specimen and record what it finds against the known runner.
  - Done when a recorded comparison exists; no claim about Rust
    regeneration follows from it.
  - _Boundary: TestInfraDetector_
  - _Requirements: 12.4_

- [ ] 3. Make sensitivity comparable between original and candidate
- [ ] 3.1 (P) Mutant sampling and arm on the original's module, built the same way as on the candidate
  - Same seed, population enumeration, sample size, acceptance arm,
    deadline rule, classification, restore-and-verify as in E3, on an
    isolated copy of the original, with the guard from 1.2.
  - Dry run of two mutants on the original: both classified, the copy
    restored and hash-verified after each.
  - Done when the dry run's two receipt entries show classification and a
    verified restore.
  - _Boundary: Sensitivity_
  - _Depends: 1.2_
  - _Requirements: 8.1, 8.2, 8.5, 8.6, 11.4, 11.5_
- [ ] 3.2 Run the original's full sample and report both rates (owner go)
  - Report killed, survived, incompetent and the kill rate for the original
    and the candidate on populations drawn the same way; label the post
    hoc equivalent-excluded reading separately; the 0.70 bar is unchanged.
  - Done when a dated amendment to R25, written after the amendment of
    1.1, carries the two comparable rates.
  - _Depends: 1.1, 3.1_
  - _Requirements: 8.2, 8.3, 8.6, 10.2_

- [ ] 4. Regenerator as a recorded single-turn program with a turn rule (owner go for model budget)
  - Define the regenerator brief as a fixed signature: inputs (scope files,
    derived surface, definitions, targets, structural contract, acceptance
    command, and for a turn after a failed turn only the previous turn's
    first divergence and measure gaps; after an accepted, sensitivity-scored
    turn also its surviving faults and realised counterexamples) and
    outputs (candidate files, structure declaration, change log).
  - Make the step runnable against a model endpoint configured from the
    environment; credentials never stored; every prompt, response, model
    identifier and seed recorded in the receipt by content hash.
  - Smoke on the frozen E3 scope: one logged turn produces a candidate
    tree in a fresh isolated copy. This task ends there; the edit-scope and
    structure checks on that tree are run by task 5.1 as integration.
  - Done when one logged turn on the E3 scope yields a candidate tree and
    the receipt holds prompt, response, model identifier and seed hashes.
  - _Boundary: Regenerator_
  - _Depends: 1.2_
  - _Requirements: 6.1, 6.7, 6.8, 11.2_

- [ ] 5. Integration: the new pieces on the E3 scope and round 0 on a second scope
- [ ] 5.1 Integration check of the regenerator program's output on the E3 scope
  - Run the edit-scope hash check and the structure check on the tree from
    task 4; measure it with the measurer from 1.1. Acceptance is not run,
    and this task does not assert the structural prediction; a failing
    verdict is a valid recorded result.
  - Done when the receipt shows the hash check verdict, the structure check
    verdict and the measures for that tree.
  - _Boundary: RoundController, Measurer (integration)_
  - _Depends: 1.1, 4_
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 6.4, 6.5_
- [ ] 5.2 Second scope: surface, seeds, corpus and round 0 on an identical copy (owner go, owner-approved scope)
  - The owner approves the second Facdrone scope from a written rationale
    (800 to 2,500 lines, one scattered responsibility, store behind it,
    tests executing most of it); this task does not pick it.
  - Derive the surface with the existing deriver; run detection from 2.2
    for seeds and deselections; record a corpus from the original with the
    existing recorder and validate it (two byte-identical recordings, zero
    divergence on replay); run the acceptance arm on an identical copy.
  - Record in the receipt that the run was native, under a plain user
    account, serial, with numeric thread counts pinned to one.
  - Done when round 0 on the identical copy exits 0 with a fresh, complete
    replay file, and the corpus identity, seed list, deselections and the
    host facts are receipted.
  - _Depends: 1.2, 2.2_
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 3.1, 3.3, 3.4, 11.7, 12.1, 12.2, 12.3_

- [ ] 6. E4: preregister and run the loop on the second scope
- [ ] 6.1 Preregistration and original-side apparatus (owner go for 6.2 to 6.7 answered here)
  - Write the plan with frozen input identities, the predictions of the
    report's section 8 (canonical lines, functions, edges, rounds, kill
    rate on both trees, at least one oracle-blind difference, zero
    divergence on a corpus recorded after acceptance), kill criteria,
    adaptive budgets and the research-code line budget covering 6.1, 6.4
    and 6.5.
  - Author what needs only the original tree: the mutation configuration
    and seed files for the original, at least three hand-written semantic
    mutants against the original with their catching evidence predicted,
    and the reviewer brief (plan, diff, both trees, no regenerator
    rationale).
  - Done when the plan exists with predictions fixed, the apparatus files
    are hashed in the receipt, and the owner has answered the go question.
  - _Depends: 5.2_
  - _Requirements: 7.1, 8.4, 11.1, 11.6_
- [ ] 6.2 Regeneration and acceptance
  - Run the regenerator program from task 4 for at most four turns under
    the turn rule; after each turn run the edit-scope check, the structure
    check, the measures and the acceptance arm; arbitrate any Level 2 pair
    with the query differential.
  - Done when a turn is accepted or four turns are exhausted, with each
    turn's first divergence, measures and verdict receipted, and the
    receipt stating whether a Level 2 divergence occurred and, if so, the
    query-differential verdict.
  - _Depends: 6.1_
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 4.3, 4.4, 4.5, 5.5, 6.2, 6.3, 6.6, 6.7_
- [ ] 6.3 Independent review of the accepted candidate
  - A fresh agent with the reviewer brief verifies hashes, surface
    signatures and measures, states hypotheses, probes them on both trees
    and reports formatting effects.
  - Done when the review file lists every probe with its agreement and
    every hypothesis is carried forward by name.
  - _Depends: 6.2_
  - _Requirements: 7.2, 7.3, 7.4, 7.5_
- [ ] 6.4 Sensitivity on both trees
  - Author the candidate-side mutation configuration, port the semantic
    mutants to where the code moved, and draw the seeded samples for both
    trees the same way.
  - Run the sampled and the hand-written mutants on the candidate and the
    original with restore-and-verify; report both rates; the 0.70 bar is
    unchanged.
  - Done when both rate tables and the hand-written mutant outcomes are
    receipted and every reviewer hypothesis that names a fault class is
    addressed by name in the survivor classification.
  - _Depends: 6.3_
  - _Requirements: 7.5, 8.1, 8.2, 8.3, 8.4, 8.5, 8.6_
- [ ] 6.5 Differential behind the four gates
  - Select at most the preregistered number of pure-function pairs between
    original and candidate, each tied to a named reviewer hypothesis, and
    author their drivers with recorded domains.
  - For each pair: symbolic-ness smoke, generated-corpus agreement, planted
    mutant and renaming controls, solver run at the scaled base deadline,
    concrete replay of every counterexample, and oracle replay where the
    alphabet allows it.
  - Done when each pair has a verdict and names the hypothesis it tested,
    each gate has a recorded result, and every counterexample has a replay
    record.
  - _Depends: 6.3_
  - _Requirements: 7.5, 9.1, 9.2, 9.3, 9.4, 9.5, 9.6_
- [ ] 6.6 Held-out corpus after acceptance
  - Record a new corpus version from the original after acceptance, with
    its own identity and the same validation as 5.2, and replay it against
    the accepted candidate.
  - Done when the new corpus identity and the replay verdict (zero
    divergence or the first divergence) are receipted.
  - _Boundary: Recorder, Replayer_
  - _Depends: 6.2_
  - _Requirements: 2.3, 2.4, 2.7_
- [ ] 6.7 Residue, research entry and index (integration)
  - Produce the missing-test report with the generator from 2.1 and the
    residue; write the research entry with every prediction reported as
    held or failed, including the held-out divergence row; update the
    research index, the spec's research links and the handover.
  - Done when the research entry exists with the held-out row answered and
    the index links it.
  - _Boundary: ResidueReport (integration)_
  - _Depends: 2.1, 6.4, 6.5, 6.6_
  - _Requirements: 8.7, 10.1, 10.2, 10.3, 11.3_

## Implementation Notes

- 1.2 (2026-11-05): `docs/experiments/deconvolution/guard.py` (56 lines) and
  `receipt_atomic.py` (153 lines) are the apparatus for every later heavy
  step. `receipt_atomic.py RECEIPT run LABEL [--cwd] [--env K=V] [--timeout]
  [--no-probe] -- CMD` takes the receipt path first; `run` probes the host by
  default (a few seconds), pass `--no-probe` for light steps. Deadline
  scaling and the idle rerun stay with the caller via `adaptive.scaled`.
  Thread pinning (BLAS/OpenMP variables to 1) is not set by `run`; heavy-step
  callers pass it with `--env`. Lock files older than 120 s are broken; the
  lock is held only for one read-append-replace, never for a whole step.
  The guard writes `<label>-guard` (and `-guard-2` on a second wait) with
  the probe and all four thresholds; exit 0 proceed, 3 stop.
