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
- [x] 1.1 (P) Canonical-format size measures beside the frozen measurer
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

- [x] 1.3 Separated-form statement count as a third size reading (added 2026-11-05 by owner decision after 1.1)
  - Add a versioned counter beside the frozen measurer and `measures2.py`
    that counts statements in separated form: every multi-name import,
    tuple or multiple-target assignment, chained comparison inside one
    statement, and similar packing is counted as the single-name
    statements it replaces, under a declared rule recorded in the result
    file with a rule version; the as-written count must still equal the
    frozen measures (dry-run equality check).
  - Re-read the E3 baseline and candidate under the rule; add a dated
    amendment to R25 and the E3 plan adding the separated-form figures as a
    third labelled reading next to the original T2 verdict and the
    canonical reading. Targets and verdicts are not changed.
  - Done when both E3 trees have separated-form counts in a result file
    with the rule version, the as-written counts match the frozen files,
    and R25's amendment names the separated-form statement totals and says
    whether the candidate's figure sits above or below the 561 bar.
  - _Boundary: Measurer_
  - _Depends: 1.1_
  - _Requirements: 4.1, 4.2, 4.6, 10.3, 11.1, 11.7_

- [ ] 2. Core: turn the E3 residue into artefacts the workflow can consume
- [x] 2.1 (P) Missing-test report from survivors and realised counterexamples
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
- [x] 2.2 (P) Test-infrastructure detection for a Python repository and scope
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
- [x] 2.3 Detection on the Rust specimen's runner
  - Optional follow-up outside the gate's Python scope: run detection on
    the Rust specimen and record what it finds against the known runner.
  - Done when a recorded comparison exists; no claim about Rust
    regeneration follows from it.
  - _Boundary: TestInfraDetector_
  - _Requirements: 12.4_

- [ ] 3. Make sensitivity comparable between original and candidate
- [x] 3.1 (P) Mutant sampling and arm on the original's module, built the same way as on the candidate
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
- [x] 3.2 Run the original's full sample and report both rates (owner go)
  - Report killed, survived, incompetent and the kill rate for the original
    and the candidate on populations drawn the same way; label the post
    hoc equivalent-excluded reading separately; the 0.70 bar is unchanged.
  - Done when a dated amendment to R25, written after the amendment of
    1.1, carries the two comparable rates.
  - _Depends: 1.1, 3.1_
  - _Requirements: 8.2, 8.3, 8.6, 10.2_

- [x] 4. Regenerator as a recorded single-turn program with a turn rule (owner go for model budget)
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

- [x] 5. Integration: the new pieces on the E3 scope and round 0 on a second scope
- [x] 5.1 Integration check of the regenerator program's output on the E3 scope
  - Run the edit-scope hash check and the structure check on the tree from
    task 4; measure it with the measurer from 1.1. Acceptance is not run,
    and this task does not assert the structural prediction; a failing
    verdict is a valid recorded result.
  - Done when the receipt shows the hash check verdict, the structure check
    verdict and the measures for that tree.
  - _Boundary: RoundController, Measurer (integration)_
  - _Depends: 1.1, 4_
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 6.4, 6.5_
- [x] 5.2 Second scope: surface, seeds, corpus and round 0 on an identical copy (owner go, owner-approved scope)
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

- [x] 6. E4: preregister and run the loop on the second scope
- [x] 6.1 Preregistration and original-side apparatus (owner go for 6.2 to 6.7 answered here)
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
- [x] 6.2 Regeneration and acceptance
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
- [x] 6.3 Independent review of the accepted candidate
  - A fresh agent with the reviewer brief verifies hashes, surface
    signatures and measures, states hypotheses, probes them on both trees
    and reports formatting effects.
  - Done when the review file lists every probe with its agreement and
    every hypothesis is carried forward by name.
  - _Depends: 6.2_
  - _Requirements: 7.2, 7.3, 7.4, 7.5_
- [x] 6.4 Sensitivity on both trees
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
- [x] 6.5 Differential behind the four gates
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
- [x] 6.6 Held-out corpus after acceptance
  - Record a new corpus version from the original after acceptance, with
    its own identity and the same validation as 5.2, and replay it against
    the accepted candidate.
  - Done when the new corpus identity and the replay verdict (zero
    divergence or the first divergence) are receipted.
  - _Boundary: Recorder, Replayer_
  - _Depends: 6.2_
  - _Requirements: 2.3, 2.4, 2.7_
- [x] 6.7 Residue, research entry and index (integration)
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
- 1.1 (2026-11-05): `e3-evidence-regeneration/measures2.py` (46 lines)
  imports the frozen measurer and formats copies of the five files with
  ruff format 0.17.3 (`--isolated --line-length 100 --target-version py313`)
  under `.runs/e3-facdrone/fmt/`. Canonical reading of E3: original 660
  statements / 1,274 lines (already formatted); candidate 546 / 1,188. The
  candidate's canonical line count is above the T2 bar of 1,083 (6.8%
  below the original against 15% required); the preregistered verdict is
  unchanged and both amendments say so. Spec finding for the owner: a
  formatter never changes `ast.stmt` counts, so requirement 4.2 as written
  cannot expose statement packing (multi-name imports, tuple assignments;
  the step-2 reviewer's separated-form estimate is about 572). A third
  reading, a separated-form statement count, would need its own
  normaliser; not built, not in any task. E3 research code now 591 lines
  against 400; the plan.md hash in the receipt facts is stale by design
  (amendment).
- 2.2 (2026-11-05): `docs/experiments/deconvolution/testinfra/testinfra.py`
  (626 lines, static only; never imports the specimen) with `compare.py`
  (107) and `test_testinfra.py` (144). CLI `testinfra.py <root> --scope
  <file>... [--baseline-failures <json>] [--seeds-file <txt>]
  [--site-packages <dir>] [--stem <name>] [--out <dir>]`. On Facdrone it
  reproduces the 19 importing files, the 236 resolved baseline failures
  with 24 applicable, and the marker expression `not port_baseline and not
  live_broker` exactly. The 191 coverage seeds are NOT reproduced
  statically: 101 of 191 are found, 90 reach the scope only through
  production modules (coverage-only), 15 recommended files are outside the
  191 (1 marker-deselected, 3 plugin-skipped, 11 unexplained statically).
  `--seeds-file` only applies deselections to a supplied list. The Windows
  shim hazards need `--site-packages`. Known limit: relative imports inside
  `__init__.py` resolve one level too high (not exercised by Facdrone).
- 2.1 (2026-11-05): `e3-evidence-regeneration/missing_tests.py` (147 lines)
  emits `results/missing-tests.{md,json}` from the T4 sample, the receipt
  facts, the T5 replay log and the hand-encoded readings
  (`results/t4-survivor-readings.json`); `missing_tests_check.py` (175)
  scores an external test file on the original copy, the unmodified
  candidate copy and each real gap's mutant, with guard and atomic receipt
  (`results/receipt-missing-tests.json`). E3 report: 4 real gaps (T4-17,
  T4-21, T4-29, T5-capture), 5 equivalent (post hoc), 2 closed-assumption.
  Usefulness check (preregistered in `results/missing-tests-experiment.md`,
  frozen hashes, then a dated Result section): a fresh sonnet writer with
  only the report and the original wrote 17 tests; 3 of 3 gaps killed
  (1, 2 and 8 tests), exception path exposed by 3 tests failing only on
  the candidate; prediction held; 5 of 8 runs used; tests discarded, hash
  5bee7d0a… in the receipt. Attribution rule fixed before any test: a
  killing test must pass on the candidate, because the exception-path
  difference is in every mutant copy. `.runs/e3-facdrone/orig-check` is a
  copy of the original kept for checks.
- 3.1 (2026-11-05): `docs/experiments/deconvolution/e3b-original-sensitivity/t31_arm.py`
  (259 lines) reproduces E3's T4 procedure on the original: population 307
  specs from the original publication module alone (E3 candidate 317),
  seed 20261104, 30 draws from sorted keys (re-draw verified identical),
  same 19 seed files and 6 deselections, same arm stages and
  classification, deadline max(120, 5 × unmutated arm) scaled by the
  probed factor, guard before every heavy step, restore by copy and
  five-file hash check. Mutable copy `.runs/e3-facdrone/orig-mut`
  (git-ignored), sqlite `.runs/e3-facdrone/t31.sqlite`. Dry run: base 125.6 s
  (factor 1.87); m0 AddNot line 103 killed by the R16 harness in 4.8 s; m1
  AddNot line 192 killed by a contracts test in 25.1 s; both restores
  identical. No rate is computed from two mutants; 3.2 (owner go) runs the
  full sample. The frozen replay writes its Level 2 file into E3's results;
  the arm moves it out in a finally block (verified no leftover). Deadline
  arithmetic goes into `<label>-deadline` notes because the receipt is
  append-only.
- 1.3 (2026-11-05): `e3-evidence-regeneration/measures3.py` (69 lines)
  counts statements in separated form under rule version 1 (multi-name
  import and from-import per name; tuple assignment with tuple value per
  target; multiple targets per target; chained comparison per comparator;
  del, global/nonlocal and multi-item with per item; `a, b = f()`,
  starred targets, boolean operators, comprehensions and loop targets are
  not unpacked). E3 third reading: original 718 (from-import 56, tuple 2),
  candidate 627 (import 7, from-import 58, tuple 16); candidate above the
  561 bar; 12.7% below the original's separated figure against 15%
  required. The from-import rule inflates both trees almost equally;
  without it 662 vs 569, matching the step-2 reviewer's 662 vs 572 within
  3. Verdicts unchanged; amendments R25 #3 and E3 plan #5.
- 3.2 (2026-11-05 and 2026-11-06): full sample on the original,
  `t31_arm.py dry 0..29`; the series was cut off after mutant 24 on
  2026-11-05 (07:01 UTC, session ended; m24's arm terminated with exit
  1073807364, classification void) and completed on 2026-11-06 with
  `dry 24..29` under receipt notes `t31-resume-2026-11-06` and its
  `-correction`; the last `t31-m<k>-class` note per k is authoritative.
  Original: killed 17, survived 9, incompetent 4, 17/26 = 0.654 (post hoc
  equivalent-excluded 17/22 = 0.773, labelled); candidate from E3's receipt:
  18/10/2, 0.643 (18/23 = 0.783). Neither meets 0.70; the bar is unchanged.
  Survivors: four annotation-only, five real corpus gaps (lines 184, 393,
  140, 91, 125), two at the sites of the candidate's real survivors; T4's
  failure measures the arm on this module, not a loss from regeneration.
  `t32_summary.py` (82 lines, test 26) computes both arms from the receipts
  only and `--check` reproduces `results/t32-summary.json`; readings in
  `results/t32-survivor-readings.json`. Apparatus fact: a terminated arm
  reads as "killed: exit <code>" in `classify`; a resume must void such a
  classification by hand (done here by receipt note). Amendments R25 #2
  (comparability per requirement 8.6 stated) and E3 plan #4. Reviewed by
  an independent sonnet agent: 5 should-fix (table cells, factor range,
  note count, comparability sentence, time zone) applied, then APPROVED.
- 4 (2026-11-06): `docs/experiments/deconvolution/regenerator/regen_turn.py`
  (337 lines, budget 300, overrun reported) is the regenerator as a
  fixed-signature program: `TurnRequest` (scope root and files, surface
  path, definitions, targets, contract, acceptance command, feedback
  None | failed{first_divergence, measure_gaps} |
  accepted_scored{+surviving_faults, counterexamples}; other keys refused)
  → system brief + user prompt (inputs verbatim, feedback section only when
  present) → `client.messages.stream` on the endpoint from
  `~/.lesscode/model.env` (style, base URL, model, key; key only via
  `api_key()`, redacted everywhere) with `output_config.effort` and a JSON
  schema output → `write_candidate` into a verified copy of the frozen
  tree, refusing any path outside the scope before writing. Receipt holds
  per turn the sha256 of system, user, request body (rebuildable from the
  saved prompts and recorded parameters) and response, model sent and
  returned, seed null (endpoint has none), usage, stop reason. Smoke on the
  E3 scope took three recorded turns against claude-opus-5-5: t4-turn1
  connection reset after 310 s of silent stream; t4-turn1-retry (thinking
  display on) stop `max_tokens`, 64,000 output tokens all thinking;
  t4-turn2 (`--max-tokens 128000 --effort medium`, controller decision
  noted) `end_turn`, 36,557 in / 84,992 out (63,504 thinking), 668 s,
  five files 1,214 lines (vs 1,274), all compile, 36 pure / 17 shell
  declared; exactly the five scope files differ from the frozen copy
  (reviewer: 0 added, 0 other differing). Apparatus facts: the model needs
  the 128k output ceiling and medium effort for a 1,274-line scope in one
  turn; a silent stream over five minutes gets cut, so thinking display
  stays on; sidecars hash the text as sent, and `_save` now writes LF so
  bytes match (review should-fix, recorded in fact
  `t4-apparatus-after-review`). Prompt and response texts stay on disk
  (specimen source); sidecars committed. Reviewed by an independent sonnet
  agent: APPROVED with one should-fix (applied). Behaviour of the
  candidate is unknown until 5.1 (structure, measures) and an acceptance
  run (6.x); nothing is claimed beyond the smoke.
- 5.1 (2026-11-06): `docs/experiments/deconvolution/t51-integration/`
  (`editscope.py` 65 lines, `pureshare.py` 37, tests 67, 6 cases red then
  green). Edit-scope check on `.runs/e4-regen/cand-t4` against the frozen
  copy: 1,965 files per side over root files, src, tests, packages (the
  reviewer's full-tree walk, 2,798 files, agrees), nothing added or
  removed, exactly the five scope files differ, hashes equal the
  regenerator's `t4-turn2-candidate` note: pass. Structure check with the
  declaration from `result.json` (36 pure / 17 shell): 53 defs all
  declared, 0 pure-check failures, 12 write sites all in declared shell
  functions: pass. Measures of the t4-turn2 candidate (original / E3 round
  1 / t4-turn2): lines 1,274 / 1,074 / 1,214 (canonical 1,274 / 1,188 /
  1,210), statements 660 / 546 / 549 (separated 718 / 627 / 610),
  functions 67 / 51 / 53, edges 84 / 60 / 63, write sites 13 / 11 / 12,
  pure share by functions 0.706 / 0.679 (by statements in defs 0.628).
  Against the E3 targets, for information only: statements, functions,
  edges, write sites and pure share inside; lines (bar 1,083) and
  separated statements (bar 561) outside, as for round 1. Nothing
  asserted (5.5), no acceptance, behaviour unknown. Reviewer reran every
  measurer byte-identically; APPROVED; the removed-file test case was
  added after the review (receipt `test-editscope-green-2`).
- 5.2 (2026-11-06): scope = lifecycle monitor group, seven files, 1,495
  lines (`e4-lifecycle/scope-rationale.md`; owner chose it from the survey
  and confirmed it after the rationale's four problems were put to them:
  the full-loop db test imports fcntl through the research app, lazy-import
  patching on the CLI and evidence modules, the import-linter and fan-out
  gates name these modules, `lifecycle_review` reads the wall clock;
  conditions: four functions residue, `lifecycle_review` out of the
  alphabet, patching as an E4 kill criterion). Apparatus `surface4.py` 60,
  `e4_oracle.py` 534, `run_e4.py` 77 lines (671 against 600, overrun
  reported), tests 7 + replay target. Surface derived (0 unresolved; five
  patched `lifecycle_group` globals explicit; instance-attribute patches
  declared, not derivable). Seeds by detection: 15 direct test files, 12
  kept (full-loop test gate-ignored, two fcntl collection errors dropped at
  file level), one node deselection from E1b's 236+8 baseline list, marker
  `not port_baseline and not live_broker`. Corpus: 24-operation alphabet
  (store writers and readers, tick, scripted runs, evidence, projection,
  seams, three CLI commands), injected clock and declared sort orders, 26
  seed + 24 generated sequences (Hypothesis seed 20261106), 50 sequences /
  317 operations; two recordings byte-identical (raw sha256 62791b32…,
  5,643,360 bytes CRLF; LF-body identity 0b2e383d…, 5,479,067 bytes, the
  value `record()` hashes; goldens on disk under `.runs/e4-lifecycle`,
  sidecars committed); replay against the frozen original: Level 1 no
  divergence, Level 2 0. Round 0 on the identical copy: exit 0 in 75.7 s
  (root group 108 passed, 1 skipped, 1 deselected; research group 1
  passed; replay fresh 50/50, level2_ops 0), deadline 2400 s scaled from
  an unreceipted probe (factor 2.0; the guard's receipted probe said 1.47;
  lesson noted: receipt the probe that sets the deadline), thread pins in
  the recorded env, host facts (Windows 11 26200, CPython 3.13.16, plain
  user, no admin token, no symlink privilege, serial). Scope files hashed
  identical in both trees before and after. Reviewed by an independent
  sonnet agent: NEEDS_FIXES (corpus identity hashed the LF body while the
  file is CRLF; deadline provenance) then APPROVED after the sidecars,
  README, receipt notes and a `newline="\n"` write in `record()` for
  future recordings.
- 6.1 (2026-11-06): `e4-lifecycle/plan.md` (454 lines, fixed, dated;
  edits after review marked and made before the go) preregisters E4 on the
  lifecycle scope: frozen identities (seven files, surface, seeds, corpus
  LF-body 0b2e383d… / raw 62791b32…, every apparatus file hashed in
  receipt facts `task-6.1-apparatus` and `-2`), baseline measures of the
  original scope by `measures4.py` (E3 measurers rebound to seven files:
  1,495 lines = canonical, 759 statements, 844 separated, 69 functions, 89
  edges, 12 write sites), predictions T1 to T7 transcribed from the report's
  section 8 with absolute bars (T1 accepted at turn ≤ 3, turn 4 = not held
  not falsified; T2 canonical lines ≤ 1,345, ≥ 1,346 fails; T3 functions
  ≤ 55, ≥ 59 fails; T4 edges ≤ 66, ≥ 72 fails; T5 candidate kill rate
  0.60 to 0.75 on its own 30-draw sample, < 0.55 or > 0.85 fails, the E3
  T4 bar 0.70 stated for both trees; T6 ≥ 1 oracle-blind difference; T7
  zero divergence on a held-out corpus, seed 20261107) and the structural
  contract S (write sites ≤ 12 all in shell, pure ≥ 0.50, the patched
  names and instance attributes stay as looked up). Kill criteria grouped
  (void the candidate: edit outside the seven files, patching failure per
  a new `patchcheck.py` sentinel step; void a sub-step; stop the run),
  adaptive budgets, research-code budget 900 lines for 6.1/6.4/6.5 (788
  estimated). Original-side apparatus: `cosmic-ray-e4.toml` (seven
  modules), population 1,468 (init wrote nothing into the frozen tree,
  snapshot hashes equal), 30-draw sample seed 20261106 (`e4_sample.py`,
  redraw byte-identical; composition: 8 annotation-only, 1 incompetent, 4
  in residue, 2 in `study_baseline`, so the original's sample caps at
  15/29 = 0.517 by construction, stated in advance), four hand-written
  mutants (`mutants/M1..M4.patch`, apply cleanly; M1 overdue window off by
  a day, predicted survivor: alphabet gap; M2 revision bump dropped,
  caught by a store seed test and S01 op 1; M3 not_run before readings,
  seeds pass, S09 op 2 catches; M4 census ref from any authority, seeds
  pass, S02 op 7 catches), reviewer brief with a redacted plan copy
  (`plan-for-reviewer.md`, 15 redactions, leak check receipted), frozen
  turn-1 request `results/request-e4-turn1.json` and its dry prompt hashes.
  Research code 97 lines against 140. Reviewed by an independent sonnet
  agent: NEEDS_FIXES (T1 gap, ceiling 0.55 → 0.517, untestable patching
  branch, brief vs plan disclosure, missing apparatus hashes) then
  APPROVED. Owner go for 6.2 to 6.7: asked 2026-11-06, see plan.md.
- 6.2 (2026-11-06): E4 regeneration on the lifecycle scope with the task-4
  program, fixed parameters (128k output, effort medium, thinking display
  summarized), one candidate root per turn under `.runs/e4-lifecycle/`.
  `patchcheck.py` (60 lines; a first 89-line version misread the CLI
  control's SystemExit, fixed and re-registered before any candidate was
  judged, both runs kept) passed on the frozen original first. Turn 1:
  stop `max_tokens`, all 128,000 output tokens thinking, no candidate,
  counted as a failed turn. Turn 2 (request byte-identical to turn 1):
  `end_turn`, seven files, edit-scope, structure and patchcheck pass,
  arm exit 1 at collection: the replay target could not import
  `labeled_reason` from the projection module, a name the recorder uses
  but the derived surface lacks because only the recorder, outside the
  tree, imports it (apparatus finding; neither the oracle nor the surface
  was edited, criterion 5). Turn 3 (turn 1 + feedback{failed, that
  divergence, measure gaps} and nothing else, verified by prompt diff):
  `end_turn`, 36,805 in / 102,681 out (73,922 thinking), 836 s; candidate
  `cand-turn3` accepted: edit-scope pass over 2,798 files, structure pass
  (52/52 declared, 11 write sites in shells, pure share 0.519), patchcheck
  pass, arm exit 0 in 49.6 s (root 108 passed / 1 skipped / 1 deselected,
  research 1 passed, replay fresh 50/50, 317 operations, level2_ops 0; no
  query differential needed). Readings: T1 held (turn 3); T2 failed
  (canonical 1,405 against ≤ 1,345; 6.0% below 1,495); T3 held (52
  functions, band 49 to 55); T4 held (63 edges, band 58 to 66); S held.
  Deadlines from receipted probes (1,644 s and 1,512 s), far from reached.
  Budget 172 of 900 lines. Reviewed by an independent sonnet agent:
  APPROVED (three low notes: first patchcheck version kept only by hash,
  no negative control, README wording). Prompt and response texts of the
  three turns stay on disk under `regenerator/results/turns/e4-turn*`;
  sidecars committed.
- 6.3 (2026-11-06): `e4-lifecycle/review-e4.md` by a fresh sonnet agent
  under `reviewer-brief.md` and the redacted plan (forbidden: plan.md,
  mutants, samples, turn requests, the regenerator's turn texts and change
  log). Verification: 2,791 files outside the scope identical by hash, the
  seven scope hashes equal the receipt, all 37 surface names present with
  the same kind and signature, measures reproduce byte for byte, patchcheck
  passes on both trees; one apparatus inconsistency (the plan's table
  lists the pre-review brief hash; recorded as plan Amendment 1).
  Formatting: candidate 1,397 as written / 1,405 canonical (four lines
  over 100 characters), no multi-statement lines, seven docstrings
  dropped. Patching table: all 12 names and six instance attributes still
  looked up at call time; three seam lookups moved from bind time to ask
  time. Thirteen hypotheses, 128 probes executed on both trees: 102 agree,
  26 differ, grouped as F1 (error path: an evidence callable returning
  None is accepted silently by the candidate, the original raises and
  reports the signal unavailable), F2 (bind_writers lambdas accept extra
  arguments; the original raises TypeError), F3 (CLI lines printed before
  a malformed stored row are lost), F4 (TransitionRefused raised `from
  None`, context differs), F5 (seam patch timing), F6 (import side
  effects). Carry-forward: survivors in H1 to H3 and H9 to be read as
  outside the alphabet; differential pairs named per hypothesis. The
  controller reran the reviewer's probe harness (`review_probes.py`,
  copied into the directory) on both trees under the receipt: results
  byte-identical to the reviewer's (fact `task-6.3-review`). The review
  claims no equivalence; T6 candidates are H1, H2, H3, H9 pending 6.5.
- 6.4 (2026-11-06): `e4-lifecycle/e4_arm.py` (253 lines, t31_arm's
  logic parameterised over both trees; `e4_rates.py` 55; budget 308
  against 260, overrun reported; running total 480 of 900 before 6.5).
  Candidate-side cosmic-ray configuration on `cand-turn3`: population
  1,488 (init wrote nothing, snapshot hashes equal), 30 draws seed
  20261106 (redraw byte-identical; composition noted before any run: 2
  syntax errors, 5 in residue, 1 in `study_baseline`). M1 to M4 ported to
  the candidate (`mutants/M*-cand.patch`, predictions noted before the
  runs). 70 arms (two bases, 30 sampled plus 4 hand-written per tree),
  serial, guard before each, deadline from a receipted probe
  (max(120, 5 × base) scaled; bases 138.6 s original, 81.1 s candidate),
  no timeout, no rerun, every restore hash-identical, both references
  unchanged, CRLF preserved. Rates (R22 convention): original 12 killed /
  17 survived / 1 incompetent = 0.414 (bar 0.70 not met, as the plan
  stated in advance from the sample's composition; post hoc 0.571 and
  0.800 labelled); candidate 19 / 9 / 2 = 0.679 (bar not met; post hoc
  0.679 and 0.864). T5 held (0.60 to 0.75). Hand-written: M1 survived on
  both trees (predicted alphabet gap), M2, M3, M4 caught on both
  (predicted; M2 by a binding seed test that precedes the store test
  under -x, M3 at S09 op 2, M4 at S02 op 7). Survivor readings for all
  26 survivors with H1 to H13 addressed by name: original 8 equivalent /
  6 closed_assumption / 3 real (#9 `_compare` >=, #15 scope predicate,
  which the Level 2 view distinguishes with 9 SQL divergences although
  Level 1 scores it survived, #17 past-cutoff day); candidate 0 / 6 / 3
  (#9, #12 fetch-margin day, #21 two failing targets). Residue findings:
  `lifecycle_root.lifecycle_store` is reached by a seed test (declared
  residue wrong on that point), `bind_lifecycle` is unreached and
  undeclared. Apparatus facts: `e4c-m20` first read "killed" because the
  arm captures stdout only, reclassified incompetent by a receipted
  syntax probe; labels m1 to m4 and M1 to M4 collide on the
  case-insensitive filesystem (logs overwritten, classifications intact
  by timestamp). Reviewed by an independent sonnet agent: NEEDS_FIXES
  (#12 misclassified, collision list, m15 Level 2 observation, README
  text) then APPROVED.
- 6.5 (2026-11-06): `e4-lifecycle/differential/` (`e5_drivers.py` 192
  lines against the plan's 149, overrun reported; `pairs.md`
  preregistered before any gate ran, hash in the receipt). Three pairs,
  the plan's cap: `compare` (H1/F3, H11; `_compare` echoes vs returns
  lines), `standing` (H8, H7; `signal_standing` with six helpers vs one
  body), `window` (H13; `for_window` with `_lookback` vs inlined). Gates:
  smoke (a path-count signal only), generated corpus of 300 with seed
  20261106 (compare differs 24/300 in the F3 shape, standing 2/300 in the
  predicted error ordering, window agrees 300/300), planted mutant found
  by the corpus on all three, renaming clean on all three. Solver (nine
  runs, 300 s base scaled by receipted probes, 3,626 s in total, no
  timeout; the first serial script was reaped by the harness under memory
  pressure after one run and the rest ran one job each): compare plant
  counterexample / rename exhausted / pair counterexample; standing the
  same; window plant inconclusive at 990 s and pair inconclusive, so the
  solver failed a control. Under the plan's criterion 4 that voids 6.5 as
  a whole (reported as void; 6.2 to 6.4 and 6.6 stand). Evidence
  reported anyway: the compare and standing counterexamples replay
  concretely on both trees (original prints partial lines before
  `KeyError: 'monitor'`, the candidate prints nothing; original and
  candidate raise different AttributeErrors on `standing=None` with a
  string baseline), both inexpressible by the alphabet (`readings`
  returns whole rows; `standing` never yields None), the standing one a
  new error-ordering difference proposed as F7. T6 reading: held, on F3
  and F1 found by the 6.3 review's probes on both trees; the solver
  evidence is not credited to 6.5. Reviewed by an independent sonnet
  agent (every gate rerun in-process, byte-identical): NEEDS_FIXES
  (factors quoted from the guard probe instead of the deadline note,
  per-pair void wording, a counterexample decoding, timeouts not
  recorded) then APPROVED.
- 6.6 (2026-11-06): `e4-lifecycle/e4_record2.py` (19 lines, the plan's
  shim: rebinds the seed to 20261107 and the generated count to 48,
  calls the recorder and replayer unchanged; registered before any run).
  Held-out corpus recorded twice from the frozen original:
  `.runs/e4-lifecycle/held-out-a.json` and `-b.json` byte-identical, raw
  sha256 = LF-body sha256 27a2cc9a… (5,633,158 bytes, LF after the 5.2
  fix), 74 sequences (26 seed + 48 generated), 424 operations, alphabet
  and seed sequences equal to the acceptance corpus; sidecar committed.
  Replay against the frozen original: Level 1 none, Level 2 0 (kill
  criterion 7 not triggered). Replay against the accepted candidate
  `cand-turn3`: Level 1 none, Level 2 0, Level 2 file fresh inside the
  command window. T7 held. Five guarded runs with receipted probes
  (600 s base scaled, 678 to 864 s), longest 26.5 s, no timeout, no
  rerun, both trees and the acceptance goldens unchanged by hash.
  Overlap (informational): 1 of 48 generated sequences duplicates an
  acceptance sequence; the rest share no first operation under the
  receipt's identity. Budget 686 as recorded (691 live). Reviewed by an
  independent sonnet agent: APPROVED (two low notes applied to the
  README: the live budget figure and the overlap definition).
- 6.7 (2026-11-06): `e4-lifecycle/missing_tests4.py` (80 lines; adapts
  the E3 generator's functions by import, E3 file unchanged) produced
  `results/missing-tests-e4.md` and `.json`: 28 entries (12 closed
  assumptions, 7 real survivors incl. M1, 7 review findings F1 to F7, 2
  solver counterexamples), each with the site on both trees, input class,
  nearest seed test and the killing test described, not written (owner's
  stance: tests belong to the TDD workflow). `residue.md`: protection
  level, coverage, the declared residue corrected (`lifecycle_store` is
  reached, `bind_lifecycle` is unreached and undeclared), the Level 2
  observation on original #15, F1 to F7 with input classes and
  reachability as a reading, the unresolved `window` pair, what nobody may
  read E4 as protecting. Research entry R26
  `docs/research/2026-11-06/01-e4-lifecycle-regeneration.md` (310 lines,
  conclusion first, hypothesis and falsification, predictions table T1 to
  T7 and S each held or failed, turn record with usage 109,499 in /
  325,973 out and an estimated $6.96 at list price, kill criteria status,
  apparatus findings, limits, four spec amendments proposed, budget 771 of
  900, COBESY check). Index row, direction paragraph and H8 lineage
  updated; both spec research files and the handover updated. Reviewed by
  an independent sonnet agent (every figure recomputed, generator rerun
  byte-identical): NEEDS_FIXES (a false completion date, the 74-sequence
  qualifier, credit to the void differential, five wording points) then
  APPROVED. All tasks of the spec are done; the owner's next decision is
  recorded in the handover.
- 2.3 (2026-11-07, owner go via opencode): detection run on the E1 frozen
  mmhmem copy (`.runs/e1-mmhmem/original`, read only) with the unchanged 2.2
  apparatus (`testinfra.py` from a461ee8, sha256 1cbb55e2…): exit 0,
  `runner: unknown`, 0 test files, 0 seeds, 0 parse errors, empty marker
  suggestion; known runner cargo test (ci.yml lines 110/143/176) with
  cargo-nextest via `.config/nextest.toml` (E1 toolchain: nextest 0.9.150,
  rustc/cargo 1.98.1 MSVC); corpus 22 `tests/*.rs` + 18 inline `#[cfg(test)]`
  modules + 128 `#[test]`; the nested `ci/` uv sub-project (pytest ≥8.0.2 in
  `uv.lock`, nine `ci/scripts/test_*.py`) is invisible to root-config
  detection; one cross-language positive: the `docker-compose.yml`
  external-store file fact. Comparison
  `testinfra/results/mmhmem-rust-comparison.md`, receipt `receipt-2.3.json`,
  log `logs/mmhmem-rust-scope.log`; research entry R32
  `docs/research/2026-11-07/06-rust-specimen-testinfra-detection.md` with
  index row. No claim about Rust regeneration (task text); with the optional
  2.3 done, every task of the spec including it is closed.
