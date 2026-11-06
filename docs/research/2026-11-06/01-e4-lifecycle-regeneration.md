# R26: the loop on a second scope accepts at turn 3 with 25% fewer functions and 29% fewer edges but only 6% fewer canonical lines (T2 failed); sensitivity 0.679 on the candidate against 0.414 on the original's sample, seven oracle-blind differences on error paths, zero divergence on 74 held-out sequences (26 of them the acceptance seed sequences replayed again, 47 new; 1 generated sequence duplicates an acceptance one); H8(c) stands on two scopes with the line prediction failed on the second

- Report date/time and timezone: written 2026-11-06 (evening) and completed 2026-11-06, about 17:50 Europe/Berlin
  (fact `task-6.7-files`, 16:50 UTC); review fixes applied the same evening (note `task-6.7-review-fixes`)
- Execution interval: 2026-11-06 10:08 to 16:28 UTC (11:08 to 17:28 Europe/Berlin), from the first regenerator turn
  (`e4-turn1`) to the last held-out hash check (`e6-hash7-after`), per `results/receipt.json`; the apparatus and round 0
  of 5.2 and the 6.1 plan ran earlier the same day from 08:49 UTC; the 6.7 generator ran as light steps after 16:28 UTC
- Status: completed; T1, T3, T4, T5, T6, T7 and S held; T2 failed; 6.5 void under kill criterion 4; no candidate-voiding
  or run-stopping criterion fired
- Spec: [deconvolution-loop](../../../.kiro/specs/deconvolution-loop/requirements.md), tasks 6.1 to 6.7
  (requirements 1 to 12); [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/brief.md),
  gate remains `revise`
- Hypothesis ID and predecessor: H8(c) regeneration behind the public API, registered in
  [R19](../2026-11-03/01-dynamic-deconvolution-direction.md), first run as E3 in [R25](../2026-11-04/02-e3-evidence-regeneration.md);
  E4 tests the prediction table of [the report's section 8](../../reports/deconvolution-loop-applied.md) on the first of
  its three scopes, and the two pieces that had no evidence: the regenerator program of task 4 and the turn rule
- Type: preregistered, [E4 plan](../../experiments/deconvolution/e4-lifecycle/plan.md) (frozen 2026-11-06, owner go the
  same day, Amendment 1 after the 6.3 review); apparatus, commands and every step in the
  [E4 README](../../experiments/deconvolution/e4-lifecycle/README.md), the [review](../../experiments/deconvolution/e4-lifecycle/review-e4.md),
  the [differential](../../experiments/deconvolution/e4-lifecycle/differential/README.md) and the
  [residue statement](../../experiments/deconvolution/e4-lifecycle/residue.md)

## Decision

The recorded single-turn regenerator program of task 4, given only the seven lifecycle monitor files of Facdrone, their
derived surface, the definitions, the targets and the acceptance command, produced at its third turn a candidate that
the E4 acceptance arm accepts: zero Level 1 divergence on the 50 recorded sequences (317 operations), zero Level 2
pairs, every applicable seed test passing (108 passed, 1 skipped, 1 deselected; research group 1 passed), edit scope
exactly the seven files out of 2,798, every patched name still looked up at call time. The candidate has 52 functions
instead of 69 and 63 intra-scope call edges instead of 89 (24.6% and 29.2% fewer, inside the predicted bands), 11 write
sites all in declared shell functions and 27 of 52 functions in a checked pure core, and 1,405 canonical lines instead
of 1,495. That last number is the failed prediction: 6.0% fewer lines against the 10 to 20% the report's section 8
predicts, with the plan's absolute bar at 1,345. T2 is reported as failed.

The protection that goes with the acceptance is measured, not assumed. The same arm kills 19 of 28 competent sampled
mutants on the candidate (0.679, inside the predicted band 0.60 to 0.75) and 12 of 29 on the original (0.414; a result
about that sample's composition, stated in advance, with 8 annotation-only draws and 6 in residue); the 0.70 bar carried
from E3 is unmet on both trees. Three of four hand-written semantic faults are caught on both trees, at the predicted
sequence and operation for the two the seed tests miss; the fourth (an overdue window one day early) survives on both,
as predicted, an alphabet gap. The independent review's 128 probes found six concrete differences the oracle cannot
see, F1 to F6, every one on an error or malformed-input path or in call-time metadata. The differential step is void:
its `window` planted-mutant control did not reach the planted difference in 5,559 solver iterations, so under the
plan's criterion 4 nothing is credited to 6.5; its corpus gates and solver replays add a seventh difference, F7, and a
second route to F3 as evidence only. Two of the seven (a `None` evidence accepted silently; partial CLI output lost on
a corrupted row) are on inputs the types forbid but a production fault could produce. A corpus of 74 sequences recorded
after acceptance from a seed that did not exist before (26 of them the acceptance seed sequences replayed again, 47
new; 1 generated sequence duplicates an acceptance one) replays on the candidate with zero divergence at both levels:
T7, the row that decides trust, held.

What this permits next: the prediction table of the report now has one of its three rows filled, with lines outside the
band and everything else inside; a third scope would complete the preregistered test, and adoption of this candidate
remains the owner's decision outside the loop. It does not permit calling the candidate equivalent (the residue in
section 2 of the residue statement has never run under the arm on either tree), and it does not permit reading the line
figure as compression of the kind E3 showed.

## Hypothesis and falsification

H8(c), as registered in R19 and run once in E3: an LLM regenerating a scattered responsibility behind its public
surface, accepted by a recorded trace oracle and seed tests, yields fewer functions and edges with equal observed
behaviour. E4 asks the same on a second scope with two differences from E3: the regenerator is the recorded
single-turn program of task 4 with the turn rule of requirement 6.7, and the predictions are the absolute bars of the
report's section 8 read on this scope (plan.md, fixed before any heavy step). Falsifiers as preregistered: T1 fails if
four turns are exhausted or every turn is void; T2 fails at 1,346 canonical lines or more; T3 at 59 functions or more;
T4 at 72 edges or more; T5 below 0.55 or above 0.85 on the candidate's sample; T6 if review, random inputs and the
differential together find no concrete oracle-blind difference; T7 on any Level 1 divergence or a Level 2 pair
failing the query differential; S on any part of the structural contract. Kill criteria void a candidate (an edit
outside the seven files; a patching failure), a sub-step (a stale replay file; a failed verifier control; an unregistered
apparatus edit; a non-identical held-out recording) or the run (a divergence on the frozen copy; four turns exhausted).
What fired: the T2 falsifier (1,405 canonical lines) and kill criterion 4 (6.5 void). Nothing else fired; no
prediction was changed after the go.

## Results against the preregistered predictions

| Id | Prediction (bar on this scope) | Measured | Held |
| --- | --- | --- | --- |
| T1 | acceptance within at most 3 turns (4 allowed; exactly 4 would be "not held, not falsified") | accepted at turn 3: turn 1 `max_tokens` with the whole 128,000-token output spent thinking (no candidate); turn 2 `end_turn`, seven files, edit scope, structure and patch check pass, arm exit 1 at collection (`ImportError: cannot import name 'labeled_reason'`, the replay target's own import); turn 3 `end_turn`, arm exit 0 in 49.6 s | **held** |
| T2 | canonical lines at most 1,345 (original 1,495; band 1,196 to 1,345; fails at 1,346 or more) | 1,405 canonical (1,397 as written; +8 under `ruff format` for four lines over 100 characters), 6.0% below 1,495 | **failed** |
| T3 | functions at most 55 (original 69; band 49 to 55; falsifier at 59 or more) | 52 (24.6% fewer) | **held**, in the band |
| T4 | intra-scope call edges at most 66 (original 89; band 58 to 66; falsifier at 72 or more) | 63 (29.2% fewer); module hops 10 for 12 | **held**, in the band |
| T5 | kill rate on the candidate's own 30-draw sample in 0.60 to 0.75 (falsifier below 0.55 or above 0.85); the original's rate beside it; the 0.70 bar stated met or not met for both | candidate 19 killed, 9 survived, 2 incompetent: 19/28 = **0.679**; original 12 / 17 / 1: 12/29 = **0.414**; bar 0.70 not met on either tree; post hoc readings, labelled and not the result: original 0.571 without 8 annotation-only survivors and 0.800 without those and 6 residue-sited, candidate 0.679 (no equivalent survivor) and 0.864 without 6 residue-sited | **held** on the candidate; the original's figure is the composition result the plan announced (ceiling 15/29 = 0.517) |
| T6 | at least one disagreement on both trees found by the review's probes, by a pair's 300-input corpus gate or by a realised solver counterexample, on an error or malformed-input path, that the acceptance replay did not report, concretely replayed | held on **F3** (partial CLI output lost on a stored-row fault, probes P03, P04, P18) and **F1** (a `None` evidence accepted silently, P62, P126), both executed on both trees in 6.3; 6.5 is void under criterion 4, so the differential contributes "not found"; its `compare` and `standing` solver counterexamples replay concretely on both trees (F3 by a second route; F7, an error-ordering difference) and are recorded as evidence, both inexpressible by the alphabet | **held** |
| T7 | zero Level 1 divergence on every sequence of the held-out corpus (seed 20261107, 26 + 48 sequences) replayed on the accepted candidate; Level 2 pairs, if any, passing the query differential | 0 Level 1 divergences on 74 of 74 sequences (424 operations; 26 of them the acceptance seed sequences replayed again, 47 new; 1 generated sequence duplicates an acceptance one), 0 Level 2 pairs (`qdiff.py` had nothing to judge); the same corpus replays with zero divergence on the frozen original | **held** |
| S | write sites at most 12, every one inside a declared shell function; pure share by functions at least 0.50; every function declared and every pure check passing; the patched names remain module globals looked up at call time and the instance attributes keep their roles | 11 write sites, all in the 25 declared shell functions; 27 of 52 pure (0.519); 52 of 52 declared, 0 pure-check failures, 0 undeclared; `patchcheck.py` both tiers pass on the candidate (and on the original first); the reviewer's patching table agrees on all 12 names and 6 instance attributes | **held**; beside it, without a bar: pure share by statements inside defs 0.40 |
| statements (no target) | reported on both trees as written / canonical / separated (rule 1) | original 759 / 759 / 844; candidate 720 / 720 / 803 | reported |

### The turn record (6.2; receipt notes `e4-turn<k>-turn`, `-verdict`)

| Turn | stop_reason | Tokens in / out (thinking) | Elapsed s | Outcome |
| --- | --- | ---: | ---: | --- |
| 1 | `max_tokens` | 36,347 / 128,000 (128,000) | 1,116.9 | no text block, no candidate; a failed turn that counts (plan 6.2 step 1); the next request is turn 1's, byte-identical (step 4: no feedback section) |
| 2 | `end_turn` | 36,347 / 95,292 (64,819) | 781.5 | seven files written (1,447 lines); edit scope pass, structure pass (0.50), patch check pass; arm exit 1 at collection: the replay target `test_e4_replay.py` imports `labeled_reason` from `lifecycle_projection`, a name the derived surface lacks because only the recorder, outside the tree, imports it; 0 seed tests ran. A failed turn with that node as the first divergence (the arm is the authority under the plan); neither the oracle nor the surface was edited (criterion 5); the gap is a surface finding, not a candidate defect |
| 3 | `end_turn` | 36,805 / 102,681 (73,922) | 836.0 | request = turn 1's plus `feedback = {kind: failed, first_divergence: <that node>, measure_gaps: {canonical lines 1,449 against at most 1,345: +104; functions 52: inside; edges 61: inside; write sites 12: at the bar; pure share 0.50: at the bar; Level 1 not reached}}` and nothing else (requirement 6.7, verified by prompt diff); candidate `cand-turn3` accepted |

Model `claude-opus-5-5` sent and returned on every turn, parameters fixed by the plan (`--max-tokens 128000 --effort
medium --thinking-display summarized`), seed null (the endpoint exposes none), cache reads 0. Totals: 109,499 input
tokens, 325,973 output tokens (266,741 of them thinking), 2,734.4 s of turn time. Cost estimate at $4 per million input
tokens and $20 per million output tokens, an assumption taken from the model's published list price as known to the
writer on 2026-11-06 (not read from the receipt, the endpoint or an invoice): turn 1 about $2.71, turn 2 about $2.05,
turn 3 about $2.20, in all about **$6.96** for the three turns; an estimate from the receipt's usage figures. The one accepted turn therefore cost about a third of the regeneration spend; the other
two thirds bought a thinking-budget exhaustion and a surface gap.

### Kill criteria (plan section "Kill criteria")

None of the candidate-voiding criteria fired: no edit outside the seven files on any turn (2,798 files compared, 0
differing outside the scope, turns 2 and 3), no patching failure (the sentinel check passed on the original first and on
both candidates; no seed test failed for a patching reason). None of the run-stopping criteria fired: the frozen copy
and the identical copy never diverged, four turns were not exhausted. Of the sub-step criteria, criterion 3 (replay not
fresh) did not fire (turn 2 never reached the replay and is a failed turn by the arm's exit, not a void), criterion 5 did
not fire (every new apparatus file was registered by fact before its first run; the plan's Amendment 1 corrects a stale
hash of the reviewer brief, no procedure changed), criterion 6 did not fire (both held-out recordings byte-identical),
and **criterion 4 fired: 6.5 is void** (the `window` pair's planted-mutant control was found by 164 of 300 random inputs
but not by the solver in 5,559 iterations at 990 s; the search spends its paths on the early `ValueError` branches of
`_daily`). 6.2 to 6.4 and 6.6 stand, as the criterion says.

### Hand-written mutants (6.4, predictions fixed in `mutants/README.md` before any candidate existed)

| Id | Fault | Predicted | Original | Candidate (ported) |
| --- | --- | --- | --- | --- |
| M1 | a flag counts as overdue one day early (`overdue_flags`) | survives: alphabet gap, expressible but unrecorded | survived | survived (same construct at candidate line 145) |
| M2 | the revision bump is dropped (`transition`) | caught by a seed test and by the corpus at S01 op 1 | caught by `test_same_bound_runner_uses_only_the_current_census[False]` (`ValueError: ... a revision >= 1`) | caught, same node |
| M3 | not-run records emitted before readings (`_emit`) | seed tests pass; caught by the corpus at S09 op 2 | caught at S09 op 2, `/result/emitted/carry@3[0]/boundary` | caught, identical divergence (port: `_emit` folded into `_records`) |
| M4 | the standing's census taken from any authority (`standing`) | seed tests pass; caught by the corpus at S02 op 7 `result.census` | caught at S02 op 7, `recorded null vs replayed "enrollment:operator"` | caught, identical divergence |

M3 and M4 are the protection the recording adds over the seed tests, and the recording caught them where the plan said
it would, on both trees. M2 fell to the binding test first under `-x`, before the store test the README named; the
corpus was not reached.

### The review's findings (6.3, a fresh agent with the brief, the diff, both trees and the measures; 128 probes, 102 agree, 26 differ) and F7

- **F1** (H1): an evidence callable returning `None` for a window is accepted silently by the candidate's `_cycle`
  (judged with zero readings, nothing inserted, nothing emitted, nothing reported unavailable); the original raises,
  catches, emits `lifecycle.monitor_unavailable` and names the signal under `unavailable`. Outside the declared type;
  the recorded corpus cannot contain it.
- **F2** (H2): `bind_writers` lambdas carry `_t` and `_a` defaults, so a sixth positional argument or a `_t=` keyword
  rebinds the target; the original's five-parameter closures raise `TypeError`. The store's `refusal_reason` still gates
  a mismatched pair.
- **F3** (H1, H11): `_compare` collects its lines and echoes after the loop, so the lines before a malformed stored row
  are lost where the original had printed them; exit code and exception identical.
- **F4** (H3): every `TransitionRefused` is raised `from None`, so `__suppress_context__` and `__context__` differ;
  messages, types, rows, clock reads and receipts agree.
- **F5** (H9): `baseline_view`, `admission_evidence` and `MonitoredSignal` are resolved when the runner asks rather than
  when the seams bind; a patch active only around the bind no longer reaches them. The sentinel check patches around the
  whole run and does not see it.
- **F6** (H9): importing `lifecycle_seams` alone no longer loads two kernel modules; `lifecycle_group` gains a module-level
  `timedelta`. Information.
- **F7** (H8, from 6.5, predicted in `pairs.md` before the gates ran): with `standing = None` and a baseline whose `states`
  holds a string, the original fails on `standing.signal` and the candidate on the baseline view first; same exception
  type, different message. Outside the `Standing` type.

No probe among the 76 of H4 to H8 and H10 to H13 found a difference. The review verified every hash, all 37 surface
names with kind and signature, the measures byte for byte and the patching table, and found one stale apparatus row
(the reviewer brief's hash in the plan's table; plan Amendment 1). The controller reran the reviewer's probe harness on
both trees under the receipt, byte-identical. Every finding, every surviving mutant not judged equivalent and the two
solver counterexamples carry their site on both trees, their input class, the nearest seed test and the one test that
would kill them in the generated missing-test report (`results/missing-tests-e4.md`, 28 entries: 12 closed assumption,
7 real including M1, 7 review findings, 2 solver counterexamples; tests described, not written).

### Residue after E4 (6.4 findings; the full statement is `residue.md`)

The plan's declared residue was four functions. 6.4 showed that `lifecycle_root.lifecycle_store` **is** reached by a
seed test (its mutants at line 23 are killed on both trees by the CLI `readings` command's default store), so the
declaration was wrong on that point, and that `cppi/lifecycle_seams.bind_lifecycle` is unreached by every seed test and
every oracle operation and was **not** declared (`e4c-m11` survived there). The residue after E4 is `lifecycle_admit`,
`lifecycle_review`, `pod_lifecycle_standing`, `bind_lifecycle`, the unpatched bodies of `_pod_of` and `study_baseline`,
the deselected node, the skipped operator-tier test, a breaching window through the real retained-evidence join, and
threaded races. A Level 2 observation sits beside it: the original's survivor `e4o-m15` (a widened `scope <=` predicate)
differs in the recorded SQL text in 9 operations although the dumped rows agree, so the recorded SQL would catch a
query-shape fault that Level 1 cannot; the arm scores Level 1 only, by the plan.

## Frozen inputs, method and budget

Specimen Facdrone `d49f48bd…`, frozen copy `.runs/e2-facdrone/source`, read only throughout (seven-file hashes
`fdd35e1c…`, `35d9f27a…`, `03bfef33…`, `5ae9a370…`, `adb6075c…`, `10e5605f…`, `ffe5ac0b…`, re-verified after every step).
Scope: the lifecycle monitor group, seven files, 1,495 lines, chosen by the owner from the survey after the four
problems of the rationale were put to them (patching by name across lazy imports, a wall-clock read in `review`, two
commands no test exercises on this host, repository gates that name these modules). Surface by `surface4.py` (37 names,
1,754 files scanned, 0 unresolved; the five `lifecycle_group` globals the tests patch explicit). Seeds by detection: 12
files, one deselection. Acceptance corpus `golden-a.json` (LF-body identity `0b2e383d…`, 50 sequences, 317 operations,
two recordings byte-identical, replay on the frozen original clean, round 0 on an identical copy exit 0). Plan
`plan.md` frozen at `baf5ea1c…` (`73ca650f…` after Amendment 1). Regenerator `regenerator/regen_turn.py` at
`038cbad2…`, request `request-e4-turn1.json` at `c2a98af3…` with system `5fab7bf2…` (byte-identical to E3's) and user
`0ffacb8b…` (90,590 characters, the seven files, the surface, definitions, targets, contract and acceptance command);
turn 3's request `7b5394c9…`. One fresh candidate root per turn, never reused. Candidate `cand-turn3` hashes in note
`e4-turn3-candidate` and fact `e4-t3-hash7-after-cand`.

Method per the plan: regeneration and acceptance (6.2) with edit-scope check, structure declaration and measures,
acceptance arm, query differential if any Level 2 pair, patching sentinel check; independent review (6.3); sensitivity
on both trees (6.4: seed 20261106, 30 draws from the sorted cosmic-ray spec keys per tree, populations 1,468 and 1,488,
the arm per mutant with a deadline of max(120, 5 × unmutated seconds) scaled, restore and seven-hash verify before and
after every mutant, 70 pre and 68 restore notes all identical); differential behind the four verifier gates (6.5: three
pairs `compare`, `standing`, `window`, preregistered in `pairs.md`, CrossHair 0.0.112 with z3 through the E1c-2 launcher,
300 s base scaled); held-out corpus (6.6: `e4_record2.py` rebinds the seed to 20261107 and the generated count to 48,
calls the recorder and replayer unchanged). Every heavy step behind `guard.py` and `receipt_atomic.py` with the deadline
factor from a receipted probe (the lesson of round 0), thread counts pinned to one, serial. Native Windows 11, plain user
account, no WSL, no bare `bash`, CPython 3.13.16. Tools: cosmic-ray 8.7.0, ruff 0.17.3, Hypothesis 6.170.14, anthropic
SDK 2.6.0.

| Budget | Preregistered | Used |
| --- | --- | --- |
| Regenerator turns | 4 at 128,000 output tokens, 120,000 counted input | 3 (one accepted); counted input 35,905 to 36,363 |
| Acceptance deadline | 1,200 s base scaled | 1,644 s and 1,512 s scaled (factors 1.37, 1.26); arms took 19.2 s and 49.6 s |
| Sensitivity | 30 + 4 mutants per tree | 68 arms, 11:58 to 13:50 UTC, longest 93.1 s; no timeout, no rerun, no guard stop; factors 1.08 to 4.0 while the review's probes shared the host |
| Differential | at most 3 pairs, 300 s base scaled per solve | 3 pairs, nine solver runs, 3,626.2 s solver time; no timeout; the serial phase-2 script was reaped by the harness under host memory pressure after the first run and the remaining eight ran one job each |
| Recording and replay | 600 s base scaled | five runs, 678 to 864 s scaled, longest 26.5 s |
| Research code | 900 lines (6.1, 6.4, 6.5, the 6.6 shim, the 6.7 shim; tests excluded) | **771** live: 6.1 97 (`measures4.py` 44, `e4_sample.py` 53), 6.2 75 (`patchcheck.py` 60, `pureshare4.py` 15), 6.4 308 (`e4_arm.py` 253, `e4_rates.py` 55), 6.5 192 (`e5_drivers.py`), 6.6 19 (`e4_record2.py`), 6.7 80 (`missing_tests4.py`); the running total recorded through 6.6 was 686 because `e4_rates.py` was counted at 50 before its re-registration at 55; 691 live before 6.7; per-task overruns reported (6.4 by 48, 6.5 by 43), nothing compacted except the 6.7 generator, which was cut from 100 to 80 lines before its rerun to meet the task's bound |

Failed iterations kept in the receipt: the first 89-line `patchcheck.py` whose control misread typer's `SystemExit`
(`patchcheck-frozen`, exit 1, every tier had passed); a PowerShell parse error before the first `e4-t2` arm launch (no
entry, by construction); `e4c-m20` first classified killed because the arm captures stdout only, corrected by a
receipted syntax probe to incompetent; four driver fixes in 6.5 (a parent process that has loaded the trees exits
0xE0000001 when a child imports them; symbolic `timedelta` and `datetime.combine` faults in two drivers; a renaming
that touched an exception message; a summary-mode crash); the first 100-line `missing_tests4.py` and its run.

### The program's first use, on the E3 scope (tasks 4 and 5.1)

Before E4 the regenerator program had been exercised once, on the five E3 files: task 4's smoke took three recorded
turns against the same model (a connection reset after 310 s of silent stream; a retry at 64,000 output tokens and
effort `high` that spent the whole budget thinking; then `t4-turn2` at 128,000 tokens and effort `medium`, `end_turn`,
36,557 in / 84,992 out, 668 s) and produced a candidate of 1,214 lines against 1,274, which task 5.1 measured at 549
statements, 53 functions, 63 edges, 12 write sites and a pure share of 0.679 with the edit-scope and structure checks
passing, and whose behaviour stayed untested (no acceptance run). E4 is the program's first use under an acceptance arm,
and its turn parameters are the ones that smoke settled on.

## Limits and interpretation

- One scope, one regenerator program, one model, one host; thirty draws per tree; nothing about other languages. The
  report's section 8 predicts medians over three scopes, so E4 fills one row and decides nothing about the medians.
- The line prediction failed by a margin that is not a formatting artefact: the candidate gains 8 lines under the
  formatter, the original none; the regenerator removed functions and edges (closures folded into module functions,
  six projection helpers inlined, the store's `_applied` inlined) and kept most of the text. E3's candidate lost 16% of
  its lines on five files that duplicated sealing and validation paths; this scope had no such duplication of that size
  (the rationale named three copies of one ledger lookup and two projections of one baseline row). Where the scope
  carries fewer verbatim repeats, the loop removes structure before it removes lines.
- "Accepted at turn 3" is a statement about the program, the surface and the model together: one turn was lost to the
  model's thinking budget at the preregistered parameters, one to a surface gap in the apparatus (a name only the
  recorder imports), and only the third turn carried feedback about behaviour. The turn rule itself (turn 1's request
  plus the first divergence and the measure gaps, nothing else) was exercised once and produced an accepted candidate.
- The original's kill rate on its own sample has a ceiling of 0.517 by construction (14 expected survivors of 29); the
  candidate's sample had 6 expected survivors of 28. The two numbers are comparable in procedure, not draw for draw,
  and a one-sided reading of either against 0.70 says nothing about the regeneration. What they share is the finding of
  E3b: the real survivors are corpus gaps on boundary and malformed inputs that both trees carry alike.
- The alphabet reaches what it reaches: 24 operations, sequences of 3 to 8 over at most two starting states, no
  malformed stored rows, no `None` evidence, no extra writer arguments, no patching around the bind only. T7's zero on
  47 new sequences is evidence about that domain and no other; the seven oracle-blind differences all lie outside it.
- No production reachability is claimed for any difference: the readings in the residue statement say which call sites
  the review and the differential read and what the types forbid, and F1 is named as the one a production fault could
  produce. Whether a corrupted row or a `None` evidence occurs in Facdrone's operation is not something this experiment
  observed.
- The differential's void is a verifier result, not a result about the candidate: the solver did not reach a difference
  that 164 of 300 random inputs expose, which repeats R24's lesson about CrossHair's reach on date-heavy predicates.
  On this scope, as on E3's, the solver added no difference that cheaper means had not already found; its one new
  finding (F7) was predicted by reading the two bodies before the gates ran.

## Tool/spec consequence and next gate

H8(c) stands on two scopes. On the second, the loop accepted within the turn budget, reduced functions and edges in the
predicted bands, kept the structural contract, held its sensitivity band and its held-out row, and missed the line
prediction. The protection claim that goes with the accepted candidate is stated in the form of requirement 10.2 in
`residue.md` and nowhere stronger.

Apparatus findings worth carrying into the spec as dated amendments, each observed on this run:

1. **Surface = tree imports plus recorder imports.** The deriver scans the specimen tree, so a name only the oracle
   imports (`labeled_reason`) is invisible to it and a surface-preserving candidate can still fail at the arm's
   collection. Requirement 1.5 should say that the recorder's own imports are part of the surface the candidate must
   keep, and the deriver should take the recorder as an input.
2. **Turn parameters.** At the preregistered 128,000 output tokens and effort `medium` the model still spent one whole
   turn thinking; at `high` and 64,000 it had done the same on the E3 scope. Requirement 6.8 should record the output
   ceiling and effort as part of the preregistered turn, with the thinking display kept on so a silent stream is not
   cut, and should count a thinking-only turn as a failed turn, as the plan did.
3. **Bytes as stored.** Sidecar hashes must be over the file's bytes as written (CRLF on this host bit three times
   before 6.6 wrote LF); requirement 11.2's content hashes should name the bytes hashed.
4. **Collision-safe labels.** Receipt labels that differ only by letter case (`m3`, `M3`) share log and result files on
   a case-insensitive filesystem; requirement 11.3's keep-everything rule needs labels distinct beyond case.
5. **Receipt the probe that sets a deadline** (already the practice from 6.2 on); **the harness reaps long background
   shells under memory pressure**, so long serial series run one job per launch with a receipted stop and resume note;
   **an arm that captures stdout only** loses pytest's plugin-load errors, so a classification from a bare exit code needs
   a receipted probe before it stands.

The two rules R25 proposed (statement counts in the original's formatting; sensitivity on original and candidate alike)
were executed by design in E4 and should be adopted as written.

Options for the owner:

1. A third scope under the same protocol, to fill the prediction table's remaining rows and to see whether the line
   figure recovers where the scope carries verbatim repeats, with the four amendments above adopted first.
2. Adoption of `cand-turn3` into Facdrone, outside the loop: a decision on the residue statement and the seven
   differences, with the missing-test report as the list of tests to write first (the owner's stance: tests belong to
   the TDD workflow).
3. Stop the retrospective track and carry the oracle, the measures, the verifier gates and the turn rule into prevention
   at generation time.

Evidence: [experiment directory](../../experiments/deconvolution/e4-lifecycle/): `plan.md`, `scope-rationale.md`,
`README.md`, `review-e4.md`, `residue.md`, `differential/README.md`, `differential/pairs.md`, `mutants/README.md`,
`results/receipt.json` (208 commands, 35 facts and 427 notes through 6.6; 6.7 adds facts `task-6.7-apparatus`, `-2`,
`task-6.7-files`, `task-6.7-files-2`, the note `task-6.7-review-fixes` and the commands `missing-tests-e4`,
`missing-tests-e4-2`, `missing-tests-e4-3`, the last being the report after the review's site correction), `results/logs/`, `results/e4-rates.json`,
`results/e4-survivor-readings.json`, `results/e5-gates.json`, `results/sql-divergences-e6-cand.json`,
`results/missing-tests-e4.{md,json}`, `results/missing-tests-e4-inputs.json`; the turn texts under
`regenerator/results/turns/e4-turn<k>/` (sidecars committed, texts on disk); goldens on disk under `.runs/e4-lifecycle/`
with committed sidecars.

## COBESY check

Outcome first, with the failed prediction in the title and in the decision; T2 is reported as failed next to the
predictions that held; the void differential is reported as void with its evidence kept separate from its credit; the
original's kill rate is reported with the composition reading the plan announced before the run, and the post hoc
readings are labelled; facts (counts, hashes, verdicts, token usage) are separated from readings (reachable, corpus
gap, closed assumption); the cost figure is labelled an estimate; no equivalence and no adoption is claimed; the
decision is the owner's; an expert reader can replay every step from the receipt and the commands in the README.
