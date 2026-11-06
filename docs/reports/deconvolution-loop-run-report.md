# The deconvolution loop after two scopes: run report and prototype résumé

Written 2026-11-06, Europe/Berlin, after task 6.7 of the deconvolution-loop spec. Composition recorded in
`docs/reports/blueprints/2026-11-06-run-report-blueprint.md`.

- Project: Lesscode, research on making AI-generated code smaller and less entangled without changing its
  behaviour.
- Period covered: 4 to 6 November 2026. Two full runs of the loop on two scopes of one Python codebase
  (E3, research entry R25; E4, research entry R26) and the spec's task batch between them.
- Audience: the owner, who must decide whether to spend implementation effort; then a future Rust
  implementer, and an outside reviewer. The reader knows software verification and SMT solving. Each
  verification concept gets a one-sentence reminder where it first matters.
- Status: research results on two Python scopes of one specimen. Nothing has been adopted. Every number
  names the file or entry it comes from.

The claim this report argues: after two scopes, the loop's language-agnostic parts (acceptance rule,
recorded-trace oracle contract, turn rule, measures, verifier gates, residue statement, receipts) behaved as
specified and can be written down as a contract for a native implementation. Its language-bound parts
(surface derivation, recorder, measures, mutation, solver) must be rebuilt per language and have evidence
only for Python, plus one Rust recorder on an in-process store. The mature artefact today is the contract,
not the tools.

## 1. Executive summary: what held, what failed, what it costs

Two preregistered predictions failed, one per scope. On the second scope the line prediction failed: the
accepted candidate has 1,405 canonical lines against the original's 1,495, 6.0 percent fewer, where the bar
was at most 1,345 (R26, results table, row T2). On the first scope the sensitivity prediction failed: the
acceptance arm killed 18 of 28 competent mutants on the candidate, 0.643 against the required 0.70 (R25,
results table, row T4). Beside the two misses, one sub-step of the second run is void: the solver
differential's planted-mutant control on the `window` pair did not reach a difference that 164 of 300 random
inputs expose, so under kill criterion 4 nothing is credited to that step (R26, kill criteria). And one
declaration was wrong before the run: the residue said `lifecycle_root.lifecycle_store` was unreached, and a
seed test reaches it (R26, residue section).

Everything else in the two prediction tables held. Where a cell below says "failed", the entry says so in the
same word.

| Quantity | E3, first scope (R25, with Amendments 1 to 3) | E4, second scope (R26) |
| --- | --- | --- |
| Scope | 5 evidence-publication files, 1,274 lines | 7 lifecycle monitor files, 1,495 lines |
| Regenerator | interactive agent, one rewrite | recorded program, task 4 |
| Rounds or turns to acceptance (4 allowed) | 1 | 3 (turn 1 thinking only, turn 2 a surface gap) |
| Acceptance corpus | 64 sequences, 311 operations, zero divergence | 50 sequences, 317 operations, zero divergence |
| Lines, canonical formatting | 1,274 to 1,188, 6.8 percent fewer (Amendment 1; the as-written 1,074 met the bar) | 1,495 to 1,405, 6.0 percent fewer, failed (bar 1,345) |
| Functions | 67 to 51 | 69 to 52, in the band 49 to 55 |
| Intra-scope call edges | 84 to 60 | 89 to 63, in the band 58 to 66 |
| Write sites, all in declared shells | 11 in 15 shells | 11 in 25 shells |
| Pure share by functions | 36 of 51, 0.706 | 27 of 52, 0.519 |
| Kill rate on the candidate | 18 of 28, 0.643, failed (bar 0.70) | 19 of 28, 0.679, held (band 0.60 to 0.75); bar 0.70 unmet |
| Kill rate on the original, same arm | 17 of 26, 0.654 (Amendment 2) | 12 of 29, 0.414, a sample-composition result stated in advance |
| Hand-written faults | M1 to M3 caught | M2 to M4 caught on both trees, M1 missed as predicted |
| Oracle-blind differences, concretely replayed | 1 (`capture_payload` absences after an exception) | 7 (F1 to F7), all on error or malformed-input paths |
| Solver differential | controls held; one counterexample, two pairs inconclusive | void: `window` control failed; two counterexamples kept as evidence |
| Held-out corpus after acceptance | not recorded | 74 sequences, 424 operations, zero divergence at both levels |
| Research code against budget | 545 of 400 lines | 771 of 900 lines |
| Model usage | not metered (interactive agent) | 109,499 input, 325,973 output tokens; about $6.96 at list price, an estimate |
| Wall clock of the run | 09:28 to 14:53 Europe/Berlin | 10:08 to 16:28 UTC, first turn to last hash check |

A reminder of what acceptance means. A recorded trace oracle is a corpus of operation sequences recorded from
the original with every observation needed to compare outcomes, replayed against a candidate; zero divergence
is a statement about those sequences and nothing else. The protection that goes with acceptance is the
measured mutation kill rate, the differences found by review and solver that the oracle cannot see, and the
declared residue. E4's protection statement is `e4-lifecycle/residue.md`, section 1: on every recorded input
and every seed test on this host the candidate does what the original does; off those inputs, about one
competent fault in three at exercised sites survives the arm, and seven concrete differences exist on error
paths.

What a scope costs, from the record: E4 ran from 10:08 to 16:28 UTC between the first regenerator turn and
the last held-out hash check, after a morning of apparatus and planning from 08:49 UTC (R26, header); about
$6.96 of model time by estimate (R26, turn record); 671 lines of scope apparatus and 771 lines of experiment
apparatus (tasks.md notes 5.2 and 6.7). E3 ran from 09:28 to 14:53 (R25, header). A scope is a working day of
a shared host.

Recommendation, argued in section 9: run a third scope first, under the four amendments of section 6, and
write no Rust tooling until its held-out row is in; the agnostic contract of section 7 is frozen for that run.
The recommendation, and the claim under it that the prototype is mature enough for a Rust implementation of
the agnostic parts, is wrong if a third scope under the same protocol fails T7 (any held-out divergence) or
exhausts four turns, or if the Rust recorder of R20 cannot be made byte-deterministic for a store-backed scope.

## 2. The process, as run

The owner develops in the Kiro style: requirements, design, tasks, implementation, each gated by review. The
deconvolution-loop spec went through that flow on 2026-11-04, with tasks reviewed twice by an independent
agent and 18 findings applied (handover, "Spec written after E3"). The owner approved all three documents that
day and said "continue the process as planned" on 2026-11-05. The tasks were bounded apparatus revisions, one
prototype (the regenerator program) and one preregistered experiment (E4); no production task was generated
(tasks.md, scope note).

Each task ran the same way: an implementer subagent with a fresh context did the work; a reviewer subagent
with a fresh context ran the kiro-review protocol and returned APPROVED or NEEDS_FIXES; the controller verified
with fresh evidence, applied the fixes and committed the task alone (handover, "Executed 2026-11-05").
"Independent" means a second model context that did not write the artefact and reran what it could (measures,
the probe harness, the solver gates), not a second person or host. Verdicts: 5.2, 6.1, 6.4, 6.5 and 6.7 came
back NEEDS_FIXES then APPROVED; 3.2, 4, 5.1, 6.2 and 6.6 APPROVED with notes applied; 6.3 was itself the fresh
review (tasks.md notes).

The batch, in commit order (tasks.md Implementation Notes; `git log`):

| Date | Task | Commit | What it produced |
| --- | --- | --- | --- |
| 2026-11-05 | 1.2 host guard, atomic receipt | 3a14745 | `guard.py` (56 lines), `receipt_atomic.py` (153) |
| 2026-11-05 | 1.1 canonical-format measures | 3094ec5 | `measures2.py` (46); E3 canonical lines 1,188, above the 1,083 bar |
| 2026-11-05 | 2.2 test-infrastructure detection | a461ee8 | `testinfra.py` (626), `compare.py` (107) |
| 2026-11-05 | 2.1 missing-test report and usefulness check | e60c3a0 | `missing_tests.py` (147), `missing_tests_check.py` (175); 3 of 3 gaps killed by a fresh writer's tests |
| 2026-11-05 | 3.1 sensitivity arm on the original, dry run | 33180c5 | `t31_arm.py` (259) |
| 2026-11-05 | 1.3 separated-form statements | 606e37d | `measures3.py` (69); E3 candidate 627 against the 561 bar |
| 2026-11-06 | 3.2 full original sample | eb80fc4 | original 17 of 26 = 0.654; R25 Amendment 2 |
| 2026-11-06 | handover | 12cc5c9 | records 3.2 and the lifted pause |
| 2026-11-06 | 4 regenerator program | 576e072 | `regen_turn.py` (337); three recorded turns on the E3 scope |
| 2026-11-06 | 5.1 integration check of that candidate | d5cbb2f | `editscope.py` (65), `pureshare.py` (37); behaviour untested |
| 2026-11-06 | 5.2 second scope apparatus and round 0 | be6226d | `surface4.py` (60), `e4_oracle.py` (534), `run_e4.py` (77); round 0 exit 0 in 75.7 s |
| 2026-11-06 | 6.1 E4 preregistration | 1d9c59c | `plan.md` (454 lines), sample, M1 to M4, reviewer brief |
| 2026-11-06 | handover | 121ddaf | records 4, 5.1, 5.2, 6.1 |
| 2026-11-06 | owner go | 46355f6 | go for 6.2 to 6.7 recorded; plan frozen |
| 2026-11-06 | 6.2 regeneration and acceptance | 9232a49 | accepted at turn 3 |
| 2026-11-06 | 6.3 independent review | f415a38 | 128 probes, F1 to F6 |
| 2026-11-06 | 6.4 sensitivity on both trees | 2f5bb0d | 0.679 and 0.414 |
| 2026-11-06 | 6.5 differential | 20b2977 | void under criterion 4; evidence recorded |
| 2026-11-06 | 6.6 held-out corpus | a44e79f | 74 sequences, zero divergence |
| 2026-11-06 | 6.7 residue, R26, index, handover | 380c629 | 28-entry missing-test report; four amendments proposed |

### The batch between the scopes: tasks 3.2, 4 and 5.1

Task 3.2 made E3's failed sensitivity number readable. The same arm, seed and draw rule on the original's
publication module killed 17 of 26 competent mutants (0.654), one kill apart from the candidate's 0.643;
neither meets 0.70 (R25 Amendment 2). The series was cut off after mutant 24 when the session ended; that arm
had been terminated with exit 1073807364, which `classify` reads as "killed", so the resume voided it by
receipt note and reran mutants 24 to 29 (e3b README, "Full sample"). T4's failure measures the arm on this
module, not damage done by the regeneration.

Task 4 replaced the interactive regenerator with a program. `regen_turn.py` takes a `TurnRequest` (scope
files, surface, definitions, targets, contract, acceptance command, and a feedback field that is `null`,
`failed` or `accepted_scored` and refuses any other key), makes one streamed request, and writes the returned
files into a fresh copy of the frozen tree after refusing any path outside the scope (regenerator README,
"Signature"). The receipt records per turn the sha256 of both prompts, the request body and the response, the
model sent and returned, usage, stop reason and `seed: null`. The smoke on the E3 scope took three turns
(section 4). The program is 337 lines against 300, reported (tasks.md note 4).

Task 5.1 checked the smoke's candidate without acceptance: 1,965 files hashed per side, exactly the five
scope files differing; 53 defs declared, 12 write sites in shells; 1,214 lines (1,210 canonical), 549
statements (610 separated), 53 functions, 63 edges, pure share 0.679 (t51 README). The line bar and the
separated-statement bar were missed as round 1 had missed them. Nothing was asserted about behaviour.

## 3. The run in detail: E4 on the lifecycle scope

### 3.1 Scope, apparatus and round 0 (task 5.2)

The owner chose the lifecycle monitor group from a survey after four problems were put to them: patching by
name across lazy imports, a wall-clock read in `lifecycle_review`, two commands no test exercises on this host,
and repository gates that name these modules (scope-rationale, "Recommendation and decision"). Conditions:
four unreached functions declared residue, `lifecycle_review` excluded from the alphabet, the patching risk
carried as a kill criterion. The scope is seven files, 1,495 lines, 759 statements, 69 functions, 89 edges and
12 write-site matches, three of them writes (scope-rationale, table 1).

The apparatus was three files, `surface4.py` (60 lines), `e4_oracle.py` (534) and `run_e4.py` (77), 671
against 600 with the overrun reported (E4 README, "Files"). The recorder imports the generic parts of E3's
oracle unchanged; its 24-operation alphabet is specific to this scope. The deriver scanned 1,754 files and found
37 names with 0 unresolved. Detection found 15 direct test files, 12 kept, with one node deselection (tasks.md
note 5.2). Two recordings from the frozen original were byte-identical: 50 sequences (26 seed, 24 generated
with seed 20261106), 317 operations, LF-body identity `0b2e383d…` (E4 README, result table). Round 0, the arm
on a byte-identical copy, exited 0 in 75.7 s with 108 passed, 1 skipped, 1 deselected and 1 research test
passed. Its deadline came from a probe printed and not receipted; from 6.2 on every deadline probe is
receipted. The reviewer returned NEEDS_FIXES because the corpus identity hashed the LF body while the file is
CRLF, then APPROVED (tasks.md note 5.2).

### 3.2 Preregistration and original-side apparatus (task 6.1)

The plan transcribed the applied report's section 8 into absolute bars on this scope: acceptance within 3
turns; canonical lines at most 1,345 (fails at 1,346); functions at most 55 (fails at 59); edges at most 66
(fails at 72); kill rate on the candidate's own 30-draw sample in 0.60 to 0.75 (fails below 0.55 or above
0.85), the original's rate beside it and the 0.70 bar stated for both; at least one oracle-blind difference on
an error path; zero divergence on a held-out corpus with seed 20261107; and S, the structural contract: write
sites at most 12 and all in shells, pure share at least 0.50, twelve patched names and six instance
attributes looked up at call time (E4 plan, "Predictions"). Eight kill criteria void a candidate, a sub-step
or the run; criterion 4 is "a verifier control of 6.5 fails: 6.5 is void". The research-code budget was 900
lines (E4 plan, "Budgets").

On the original side the task drew 30 mutants with seed 20261106 from a population of 1,468 and read the
composition first: 8 annotation-only, 1 incompetent, 4 in residue, 2 in `study_baseline`, so the original's
rate caps at 15 of 29 = 0.517 by construction (tasks.md note 6.1). It wrote four semantic mutants with their
catchers predicted: M1, a flag overdue one day early, survives as an alphabet gap; M2, the revision bump
dropped, caught by a seed test and at S01 op 1; M3, not-run records before readings, caught by the corpus only
at S09 op 2; M4, the census taken from any authority, caught by the corpus only at S02 op 7 (mutants README).
The task used 97 of 140 lines. The owner's go was recorded in commit 46355f6 after the cost was stated: up to
4 turns at about 37k input and 85k output tokens each, about 2 hours of host time for sensitivity (E4 plan,
"Owner go").

### 3.3 Regeneration and acceptance (task 6.2)

The program ran at `--max-tokens 128000 --effort medium --thinking-display summarized` against
`claude-opus-5-5`, one fresh candidate root per turn (E4 README, 6.2).

| Turn | Stop reason | Tokens in / out (thinking) | Elapsed s | Outcome |
| --- | --- | ---: | ---: | --- |
| 1 | `max_tokens` | 36,347 / 128,000 (128,000) | 1,116.9 | no text block, no candidate; a failed turn that counts; turn 2's request is turn 1's, byte-identical |
| 2 | `end_turn` | 36,347 / 95,292 (64,819) | 781.5 | seven files, 1,447 lines; edit scope, structure (0.50) and patch check pass; arm exit 1 at collection in 19.2 s: `ImportError: cannot import name 'labeled_reason'`; 0 seed tests ran |
| 3 | `end_turn` | 36,805 / 102,681 (73,922) | 836.0 | request = turn 1's plus `feedback = {kind: failed, first_divergence, measure_gaps}` and nothing else; arm exit 0 in 49.6 s; accepted |

Source: R26, turn record; E4 README, 6.2 per-turn table.

Turn 2's failure was a surface finding. `labeled_reason` is a name the recorder imports from
`lifecycle_projection`; the deriver did not list it because it scans the specimen tree, where only the module
itself calls it. The candidate had kept every surface name and was rejected at collection. Neither the
surface nor the oracle was edited (criterion 5); the arm is the authority, so the turn counts as failed with
that node as its divergence (E4 README, 6.2 findings). Turn 3's feedback carried the node and the measure gaps
(canonical lines 1,449, +104 over the bar; functions 52 and edges 61 inside; write sites 12 and pure share
0.50 at the bar), verified by prompt diff.

The accepted candidate: edit scope pass over 2,798 files; 52 of 52 functions declared, 11 write sites in the
25 declared shells, pure share 0.519; patch check pass; arm exit 0 with 108 passed, 1 skipped, 1 deselected,
1 research test passed, replay fresh at 50 of 50, 0 Level 2 pairs (tasks.md note 6.2). Readings: T1 held; T2
failed at 1,405 canonical lines (1,397 as written; the formatter adds 8 for four lines over 100 characters);
T3 held at 52; T4 held at 63; S held (E4 README, 6.2 readings). The patching sentinel `patchcheck.py` (60
lines) was new here; its first 89-line version misread typer's `SystemExit` on the frozen original, was fixed
before any candidate was judged, and both runs stay in the receipt.

### 3.4 Independent review (task 6.3)

A fresh agent with the brief, the redacted plan, both trees, the diff and the measures, and without the full
plan, the mutants or the turn texts, reviewed the candidate in about 976 s (review-e4, header). It recomputed
the whole-tree walk (2,798 files per side, 2,791 identical outside the scope), the scope hashes, all 37 surface
names by kind and signature, the measures byte for byte, and the structure and patch checks on both trees
(review-e4, section 1). One apparatus row disagreed, the reviewer brief's hash in the plan's table; plan
Amendment 1 records it.

The reviewer stated thirteen hypotheses and executed 128 probes on both trees: 102 agreed, 26 differed, in six
findings (tasks.md note 6.3). F1: a `None` evidence callable is accepted silently by the candidate, where the
original raises and reports the signal unavailable. F2: `bind_writers` lambdas accept a sixth positional
argument, where the original's closures raise `TypeError`. F3: `_compare` echoes after the loop, so lines
before a malformed stored row are lost. F4: `TransitionRefused` is raised `from None`. F5: three seam names
resolve when the runner asks rather than when the seams bind. F6: importing the seams module alone no longer
loads two kernel modules. The controller reran the probe harness on both trees under the receipt,
byte-identical. The review claimed no equivalence.

### 3.5 Sensitivity on both trees (task 6.4)

A mutation kill rate is the share of competent injected faults the arm catches, survivors in the denominator,
incompetent excluded, equivalent kept (the R22 convention). The arm was ported to both trees (`e4_arm.py`, 253
lines; `e4_rates.py`, 55; 308 against 260, reported). The candidate's 30 draws with the same seed from a
population of 1,488 read 2 syntax errors, 5 in residue, 1 in `study_baseline` before any run (tasks.md note
6.4). 68 mutant arms ran after two unmutated bases of 138.6 s and 81.1 s, serial, guarded, at a deadline of
max(120, 5 times the base) scaled by a receipted probe, with restore and seven-hash verification around every
mutant; no timeout, no rerun, longest arm 93.1 s (R26, budget table).

| Tree | Killed | Survived | Incompetent | Rate | Bar 0.70 | Post hoc, labelled, not the result |
| --- | ---: | ---: | ---: | ---: | --- | --- |
| original | 12 | 17 | 1 | 12 of 29 = 0.414 | not met | 0.571 without 8 annotation-only; 0.800 without those and 6 residue-sited |
| candidate | 19 | 9 | 2 | 19 of 28 = 0.679 | not met | 0.679 (no equivalent survivor); 0.864 without 6 residue-sited |

Source: `results/e4-rates.json` as read in residue.md, section 4.

T5 held on the candidate. The original's 0.414 is what the plan said its sample would show: 13 of 14 expected
survivors survived and one (#28) was killed against expectation (residue.md, section 4). The hand-written
mutants went as predicted on both trees: M1 survived, M2 fell to a binding seed test, M3 to the corpus at S09
op 2, M4 at S02 op 7 (R26, hand-written mutants table). Two residue corrections: `lifecycle_root.lifecycle_store`
is reached by a seed test, so the declaration was wrong there; `cppi/lifecycle_seams.bind_lifecycle` is reached
by nothing and had not been declared (residue.md, section 2). A Level 2 observation: the original's survivor
`e4o-m15`, a widened `scope <=` predicate, differs in recorded SQL text in 9 operations although the rows
agree; the arm scores Level 1 only, by the plan. Two apparatus facts: `e4c-m20` was first read as killed
because the arm captures stdout only, and was reclassified incompetent by a receipted syntax probe; labels
`m3` and `M3` collided on the case-insensitive filesystem (tasks.md note 6.4).

### 3.6 Differential behind the four gates (task 6.5)

The differential takes pairs of pure functions, original and candidate, and runs CrossHair's `diffbehavior`, a
concolic engine over Python with Z3 underneath: it runs the code on concrete values while tracking symbolic path
constraints and asks the solver for values that flip a branch. No verdict counts until four gates pass per
pair: symbolic-ness, agreement with a generated corpus, a planted mutant found and an equivalent renaming
exhausted, and concrete replay of every counterexample (tooling note, "Solver-encoding verifier").

Three pairs were preregistered in `pairs.md` before any gate ran: `compare` (H1, F3), `standing` (H8, H7),
`window` (H13) (differential README, "The pairs"). `e5_drivers.py` is 192 lines against 149. Gates: smoke held
on all three; the 300-input corpus differed 24 of 300 on `compare` in the F3 shape, 2 of 300 on `standing` in
the predicted error ordering, and agreed 300 of 300 on `window`; the planted mutant was found by the corpus
on all three; renaming was clean on all three. The solver, nine runs and 3,626.2 s: `compare` and `standing`
each gave plant counterexample, rename exhausted, pair counterexample; `window` plant inconclusive after 5,559
iterations at 990.1 s, pair inconclusive after 5,719 (differential README, gate table).

The `window` control therefore failed in its solver form although 164 of 300 random inputs expose the planted
difference; the search spends its paths on the early `ValueError` branches of `_daily`. Criterion 4 is
literal, so 6.5 is void as a whole and contributes "not found" to T6 (differential README, "T6 reading"). The
two realised counterexamples still replay concretely on both trees as evidence credited to no step: `compare`
prints a line before `KeyError: 'monitor'` on the original and nothing on the candidate (F3 by a second
route); `standing` with `standing = None` and a string baseline raises `AttributeError` at different places on
the two trees (F7, predicted before the gates ran). Both inputs are outside the alphabet, so neither was
replayed as a sequence. Two host facts: a parent process that has loaded the trees exits `0xE0000001` when a
child imports them; the harness reaped the first serial phase-2 script under memory pressure, and the remaining
eight runs went one job each (differential README, header).

### 3.7 Held-out corpus after acceptance (task 6.6)

A held-out corpus is recorded from the original after acceptance, from a seed that did not exist before, so
acceptance cannot have been tuned to it; it is the row that decides trust. `e4_record2.py` (19 lines) rebinds
the seed to 20261107 and the generated count to 48 and calls the recorder and replayer unchanged. Two
recordings were byte-identical (sha256 `27a2cc9a…`, 5,633,158 bytes): 74 sequences, 26 seed plus 48
generated, 424 operations (tasks.md note 6.6). Replay on the frozen original: Level 1 none, Level 2 0. Replay
on `cand-turn3`: Level 1 none, Level 2 0. T7 held. Five guarded runs, longest 26.5 s. One of the 48 generated
sequences duplicates an acceptance sequence, so 47 are new (R26, T7 row).

### 3.8 Residue, research entry and index (task 6.7)

`missing_tests4.py` (80 lines, cut from a first 100-line version to meet its bound, the first run kept)
produced `results/missing-tests-e4.md` with 28 entries: 12 closed assumptions, 7 real survivors including M1,
7 review findings F1 to F7, 2 solver counterexamples, each with its site on both trees, input class, nearest
seed test and the one test that would kill it, described and not written (tasks.md note 6.7). The owner's
stance stands: tests belong to the TDD workflow; the loop adds no test to the specimen.

`residue.md` states the protection level in the form of requirement 10.2 and nowhere stronger, with the
corrected residue (four unreached functions, the unpatched bodies of `_pod_of` and `study_baseline`, the
deselected node, the skipped operator-tier test, a breaching window through the real retained-evidence join,
threaded races), the unresolved `window` pair, and the seven differences with input classes and a reachability
reading; F1 is the one a production fault could produce (residue.md, sections 2 and 3). R26 reports T2 failed
in its title. Research code closed at 771 of 900 lines, with the overruns of 6.4 (48) and 6.5 (43) reported
(R26, budget table).

## 4. The regenerator program: six turns, what the model did

The program has made six recorded requests: three on the E3 scope as task 4's smoke, three on E4 under
acceptance. The turn rule of requirement 6.7 (a turn after a failed turn gets the unchanged inputs plus only
the previous turn's first divergence and measure gaps) was exercised once, on E4 turn 3, and produced the
accepted candidate.

| Label | Scope | Parameters | Stop reason | Tokens in / out (thinking) | Elapsed s | What happened |
| --- | --- | --- | --- | ---: | ---: | --- |
| `t4-turn1` | E3 (5 files) | 64,000 out, effort high, thinking display off | none | none | 310.2 | connection reset by the remote host after a silent stream; no response |
| `t4-turn1-retry` | E3 | 64,000, high, display on | `max_tokens` | 36,557 / 64,000 (64,000) | 556.4 | one thinking block, no text, no JSON |
| `t4-turn2` | E3 | 128,000, medium, display on | `end_turn` | 36,557 / 84,992 (63,504) | 668.4 | five files, 1,214 lines; behaviour never tested under an arm |
| `e4-turn1` | E4 (7 files) | 128,000, medium, display on | `max_tokens` | 36,347 / 128,000 (128,000) | 1,116.9 | thinking only, no candidate |
| `e4-turn2` | E4 | same, request byte-identical to turn 1 | `end_turn` | 36,347 / 95,292 (64,819) | 781.5 | seven files; arm failed at collection on a surface gap |
| `e4-turn3` | E4 | same, plus the feedback section | `end_turn` | 36,805 / 102,681 (73,922) | 836.0 | accepted |

Sources: regenerator README, smoke table; R26, turn record.

Three things the record says about the model. It needs the full 128,000-token output ceiling at medium effort
for a scope of this size: at 64,000 and high it spent the whole budget thinking on E3, and at 128,000 and
medium it still did so once on E4 (regenerator README; R26, limits). A silent stream over about five minutes
gets cut, so the thinking display stays on; the cause of the first reset is a hypothesis. Of E4's three turns
only the third carried feedback about behaviour. E4's totals: 109,499 input tokens, 325,973 output of which
266,741 thinking, 2,734.4 s of turn time, about $2.71, $2.05 and $2.20 per turn, $6.96 in all, an estimate from
usage at list price and not a billed amount (R26, turn record). The endpoint exposes no seed.

What the model did to the E4 code: closures folded into module functions, six projection helpers inlined, the
store's `_applied` inlined, seven docstrings dropped, three seam lookups moved from bind time to ask time
(R26, limits; review-e4, sections 2 and 3). It removed functions and edges and kept most of the text. On E3
the agent had folded two sealing paths and six validators into one each on a scope that duplicated them, and
lines fell 16 percent as written (R25); this scope carried three copies of one ledger lookup and two
projections of one baseline row (scope-rationale, section 1), and lines fell 6 percent. Where a scope carries
fewer verbatim repeats, the loop removes structure before it removes lines (R26, limits).

## 5. Prediction table after one of three scopes

The applied report's section 8 preregistered seven quantities as medians over three Python scopes of 800 to
2,500 lines with one scattered responsibility, a relational store and a test suite that executes most of the
scope. E4 is the first scope under that table; E3 predates it and is shown for information only.

| Quantity | Predicted per scope | Wrong if (three scopes) | E4, scope 1 of 3 (R26) | E3, before the table (R25) |
| --- | --- | --- | --- | --- |
| Lines, canonical | 10 to 20 percent fewer | median under 10 | 6.0 percent fewer: outside the band, failed | 6.8 percent fewer (Amendment 1) |
| Functions | 20 to 30 percent fewer | median under 15 | 24.6 percent fewer: held | 23.9 percent fewer |
| Intra-scope call edges | 25 to 35 percent fewer | median under 20 | 29.2 percent fewer: held | 28.6 percent fewer |
| Rounds to acceptance | 1 on two of three, at most 3 on the third | any scope exhausts 4 | 3: held as the "third" case | 1 |
| Kill rate on the candidate | 0.60 to 0.75 | below 0.55, or above 0.85 unchanged | 0.679: held | 0.643 |
| Differences invisible to the recording | at least one, on an error path | none by review, random, differential together | 7: held, by review probes (F1, F3) and solver replays (F3, F7) | 1 |
| Divergence on a held-out corpus | zero | any | 0 on 74 sequences: held | not recorded |

Two rows constrain the next scopes. The rounds row has spent its "at most 3" allowance on E4, so the second
and third scopes must each accept at turn 1 for the row to hold as written (E4 plan, T1). The lines row needs
a median of at least 10 percent, so with E4 at 6.0 both remaining scopes must reach 10 or the falsifier fires.
T7's falsifier is any divergence on any scope.

## 6. Apparatus findings worth carrying

Each was observed on this run and is proposed as a dated amendment to the deconvolution-loop requirements
(R26, "Tool/spec consequence"; handover, "Open items after E4"). None has been adopted.

1. Surface equals tree imports plus recorder imports. A name only the oracle imports (`labeled_reason`) is
   invisible to a deriver that scans the specimen tree, and a surface-preserving candidate can fail at the
   arm's collection. Requirement 1 should say so and the deriver should take the recorder as an input.
   Observed on E4 turn 2.
2. Turn parameters are part of the preregistered turn: output ceiling, effort and thinking display, with a
   thinking-only turn counted as failed, as the E4 plan did. Requirement 6.8 should say so. Observed on
   `t4-turn1-retry` and `e4-turn1`.
3. Content hashes are over bytes as stored. CRLF bit three times before 6.6 wrote LF: the task 4 sidecars, the
   5.2 corpus identity, the turn files read by 5.1 (regenerator README, "Hash semantics"; tasks.md note 5.2;
   t51 README). Requirement 11.2 should name the bytes hashed.
4. Receipt labels distinct beyond letter case. `m3` and `M3` shared log files on the case-insensitive
   filesystem (tasks.md note 6.4). Requirement 11.3 needs it.
5. Three run-discipline facts: receipt the probe that sets a deadline; the harness reaps long background
   shells under memory pressure, so a long serial series runs one job per launch with a receipted stop and
   resume note; an arm that captures stdout only loses plugin-load errors, so a classification from a bare
   exit code needs a receipted probe first (`e4c-m20`).
6. The patching check. Requirement 6.6 already says patched names stay the names the candidate's functions
   look up; E4 made that a two-tier sentinel step on both trees before the arm's verdict counts
   (`patchcheck.py`), confirmed by the review's table of 12 names and 6 instance attributes (review-e4,
   section 3). The check tests a Python mechanism; the rule is not Python's.

Two rules R25 proposed were executed by design in E4 and should be adopted as written: counts reported in
canonical formatting of both trees, and sensitivity sampled from both trees with each sample's composition
stated before the run (R26, "Tool/spec consequence").

## 7. Prototype résumé: what exists, what is proven, what is not

This section is the contract for an implementer. Status values: "proven on two scopes" means the component ran
under a receipt on E3 and E4 and produced what its requirement asks; "proven on one scope" the same on one;
"built, not exercised" the file exists with tests or controls but never ran on a real case; "unbuilt" nothing
exists. Lines are physical lines of the committed files under `docs/experiments/deconvolution/`, counted
2026-11-06; the task notes give the same figures where they state them.

| Component | Prototype file(s) | Lines | Evidence | Status |
| --- | --- | ---: | --- | --- |
| Surface derivation (req 1) | `e3-evidence-regeneration/surface.py`; `e4-lifecycle/surface4.py` | 60; 60 | R25 (62 names, 15 beyond the hand list); R26 (37 names, 0 unresolved; the recorder-import gap of finding 1) | proven on two scopes |
| Recorder and replayer, Python with PostgreSQL (req 2, 3) | `e1b-facdrone-evidence-publication/e1b_oracle.py`, `scope.py`; `e4-lifecycle/e4_oracle.py`, `test_e4_replay.py` | 406, 78; 534, 32 | R22 Q5 byte-identical recordings; R25 T1; R26 T1, T7 | proven on two scopes |
| Recorder, Rust in-process over SQLite (req 2) | `tests/e1_trace_oracle.rs` in the mmhmem copy, 429 research lines (R20); `e1-mmhmem-session-bus/results/rec.py` | 429 | R20: golden trace replayed four times identically; every hand-written mutant caught | proven on one scope |
| Acceptance arm, fail closed (req 3) | `e1b-facdrone-evidence-publication/run_arm.py`; `e3-evidence-regeneration/run_e3.py`, `test_e3_replay.py`; `e4-lifecycle/run_e4.py` | 221; 44, 33; 77 | R25 T1 (64 sequences); R26 round 0 and turn 3 (50 sequences) | proven on two scopes |
| Query differential for Level 2 pairs (req 3.2) | `e3-evidence-regeneration/qdiff.py` | 108 | controls recorded in E3; no Level 2 pair occurred on E3 or E4 (R25, R26) | built, not exercised |
| Measures: as written, canonical, separated (req 4) | `measures.py`, `measures2.py`, `measures3.py`; `e4-lifecycle/measures4.py` | 54, 46, 69; 44 | R25 Amendments 1 and 3; R26 T2 to T4; reviewers reproduced byte for byte on both runs | proven on two scopes |
| Structure check and pure share (req 5) | check inside `measures.py`; `t51-integration/pureshare.py`; `e4-lifecycle/pureshare4.py` | 37; 15 | R25 T3 (36 of 51); R26 S (27 of 52) | proven on two scopes |
| Edit-scope hash check (req 6.4, 6.5) | `t51-integration/editscope.py` | 65 | t51 README (1,965 files, E3 scope); R26 (2,798 files per turn) | proven on two scopes |
| Patching sentinel check (req 6.6) | `e4-lifecycle/patchcheck.py` | 60 | R26: both tiers, both trees, every turn; the first version's misread kept in the receipt | proven on one scope |
| Regenerator program (req 6.1, 6.8) | `regenerator/regen_turn.py` | 337 | task 4 smoke on the E3 scope without acceptance; R26 turns 1 to 3 under acceptance | proven on one scope |
| Turn rule (req 6.7) | `build_prompt` in `regen_turn.py`; the controller's prompt diff | in 337 | R26 turn 3: exercised once, candidate accepted | proven on one scope |
| Round control: four-turn cap, void rules, turn record (req 6.2, 6.3) | controller procedure in the E4 plan and README; receipt notes `e4-turn<k>-verdict` | n/a | R25 one round; R26 three turns, one failed and one apparatus-failed | proven on two scopes |
| Independent review protocol (req 7) | `e3-evidence-regeneration/results/review-step2.md`; `e4-lifecycle/reviewer-brief.md`, `plan-for-reviewer.md`, `review_probes.py` | 626 (probes) | R25 (76 probes, H4 later confirmed); R26 (128 probes, F1 to F6) | proven on two scopes |
| Sensitivity: sample, arm, classify, restore (req 8.1 to 8.6) | `e3-evidence-regeneration/t4_arm.py`, `e1b-.../sample_mutants.py`; `e3b-original-sensitivity/t31_arm.py`, `t32_summary.py`; `e4-lifecycle/e4_sample.py`, `e4_arm.py`, `e4_rates.py` | 70, 74; 259, 82; 53, 253, 55 | R25 T4 and Amendment 2; R26 T5 on both trees | proven on two scopes |
| Hand-written semantic mutants as controls (req 8.4) | `e4-lifecycle/mutants/M1..M4.patch` and `-cand` ports; E3 `t4-m3-port.patch` | n/a | R25 M1 to M3; R26 M1 to M4 with predictions fixed first | proven on two scopes |
| Solver differential behind four gates (req 9) | `e3-evidence-regeneration/t5_drivers.py`; `e4-lifecycle/differential/e5_drivers.py`; E1c-2 launcher `e1c2-concolic-rerun/drivers2.py`, `ctl2.py` | 149; 192; 130, 33 | R24 verifier; R25 T5 with both controls held; R26 void on a failed control, counterexamples kept as evidence | proven on one scope |
| Held-out corpus after acceptance (req 2.3, 2.7) | `e4-lifecycle/e4_record2.py` | 19 | R26 T7: 74 sequences, zero divergence | proven on one scope |
| Missing-test report generator (req 8.7) | `e3-evidence-regeneration/missing_tests.py`; `e4-lifecycle/missing_tests4.py` | 147; 80 | task 2.1 report on E3; R26 report with 28 entries | proven on two scopes |
| Missing-test usefulness check | `e3-evidence-regeneration/missing_tests_check.py` | 175 | task 2.1: a fresh writer's 17 tests killed 3 of 3 gaps and exposed the exception path; not run on E4 | proven on one scope |
| Residue and protection statement (req 10) | R25 sections; `e4-lifecycle/residue.md` | n/a | R25; R26 with two corrections found by 6.4 | proven on two scopes |
| Receipt, append-only and atomic (req 11.2, 11.3) | `receipt_atomic.py` (earlier `receipt3.py`, `step3.py`) | 153 | task 1.2 (20 of 20 concurrent entries); every E3b and E4 step | proven on two scopes |
| Host guard (req 11.5) | `guard.py` | 56 | task 1.2; R26 (70 pre-notes, every heavy step) | proven on two scopes |
| Adaptive deadlines (req 11.4) | `adaptive.py` | 126 | R25 and R26 budget tables; no fixed cap anywhere | proven on two scopes |
| Test-infrastructure detection (req 12.4) | `testinfra/testinfra.py`, `compare.py` | 626, 107 | task 2.2 on E3: importing files, 236 and 24 deselections and the marker reproduced, the 191 coverage seeds not; R26 5.2: the seed source actually used | proven on one scope |
| Scope selection | none; the edge census was rejected (R21) | 0 | every scope chosen by hand from a survey | unbuilt |
| Rust: surface, measures, mutation sample application, arm, query differential, solver drivers | none; cargo-mutants and nextest exist as tools (R20) | 0 | design, "What a native implementation would replace" | unbuilt |
| Rust recorder with Level 2 capture against an external store | none | 0 | R20 recorded SQLite state and audit lines in-process; no query-text capture, no external store | unbuilt |
| Multi-scope bootstrap of regenerator examples; two programs on one frozen scope | none | 0 | applied report, section 9 | unbuilt |

### Copy, rebuild, do not build

The design's split, updated with what E4 proved (design, "What a native implementation would replace"; R26).

Copy as is (agnostic, two scopes of evidence unless noted): the artefact formats (surface entry, corpus,
acceptance result, measures with structure declaration, mutant sample and outcomes, differential run, receipt,
missing-test report); the acceptance rule (exit 0 only on a fresh replay file covering every sequence with
every seed group passing); the kill criteria in three groups; the kill-rate convention; the four verifier
gates with "a failed control voids the session", now exercised in the void direction; the protection statement
in the form of requirement 10.2; receipt and adaptive-budget logic; the edit-scope and hash rules. New since
the design, with one scope of evidence: the turn rule and turn record; the patching sentinel as a kill
criterion; the held-out corpus procedure (new seed, same alphabet, recorded twice, replayed on the original
first); measures read three ways; sensitivity on both trees with each sample's composition stated first;
per-scope absolute bars derived from the section 8 bands.

Rebuild per language (bound): surface derivation (now with the recorder's imports as an input); recorder and
replayer, with the alphabet written per scope (`e4_oracle.py` was 534 new lines for the second Python scope
and reused only the generic parts of the first); measures over the language's AST with a pinned formatter;
mutation engine and sample application; solver drivers or another counterexample finder; query differential
against the store; the patching check, which tests a Python mechanism and has no counterpart where runtime
patching does not exist. For Rust the in-process recorder exists (R20); nothing else does.

Do not build yet: scope selection; a generic rewrite engine; MCP or plugin surfaces; a Level 2 acceptance
condition, which would raise protection on query shape (the `e4o-m15` observation) at the price of turning
every harmless query rewrite into a divergence (residue.md, section 2); automatic adoption.

### What a Rust implementation would need first

1. A recorder for a store-backed Rust scope that is byte-identical on two recordings and captures Level 2
   (statement text and parameters) as well as Level 1. R20 proved Level 1 determinism over SQLite state and
   audit lines in-process; the Facdrone recorders proved both levels over PostgreSQL through SQLAlchemy event
   hooks (design, technology stack). A hook under sqlx or rusqlite has no evidence. This is the item the
   disproof condition names.
2. A surface deriver over Rust items plus the recorder's own uses, finding 1 applied from the first day.
3. Measures over the Rust AST with `rustfmt` as the canonical formatter and the three readings kept.
4. Mutation sampling from the `cargo-mutants` list with seeded draws and composition read before the run.
5. A counterexample finder that satisfies the four gates. Kani is Linux or macOS only, so CI only under the
   host rule; natively, cargo-fuzz and proptest are the coverage-guided and random rungs (tooling note). The
   symbolic-ness gate has no native Rust instrument; the design must say so.
6. The agnostic core as a program over the JSON artefacts above, checked by replaying E3's and E4's committed
   artefacts and receipts byte for byte. That regression is the only test of the core the record can supply.
7. The regenerator's signature and turn rule as an HTTP client, with the receipt fields of requirement 6.8
   and the parameters of finding 2.

## 8. Maturity assessment against the owner's question

The owner asked whether the loop is ready to become reusable, mature tools. The usual argument says two
accepted candidates mean the method is ready to be productised. Against it: one language, two scopes of one
specimen, both hand-picked; the solver failed its own control once; the line prediction failed on the second
scope; the original's sensitivity sample was composition-limited; a scope costs a working day. The answer this
report gives: the agnostic parts are specified and reproducible (receipts, reviews, a held-out corpus) and the
bound parts are Python-only, so the mature artefact is the contract. Five criteria, each with a verdict.

Stable contract. The artefact formats did not change between E3 and E4: the second recorder imported the
first's generic parts unchanged, the measurers were rebound to seven files, the receipt format carried every
task (E4 README, "Files"). But four amendments are proposed and none adopted. Verdict: stable in substance,
not yet in text; adopt the amendments before freezing anything for an implementer.

Two independent runs. E3 and E4 differ in scope, regenerator and turn count; each was preregistered, reviewed
at every step and recorded under receipt. They share the language, the specimen, the host and the owner, and
because the regenerator changed between them the pair does not separate the loop from the model (applied
report, section 9). Verdict: two runs, independent in scope and apparatus, not in specimen or language.

Reproducible receipts. The E4 receipt holds 208 commands, 35 facts and 427 notes through 6.6 (R26, evidence
list). Reviewers reproduced the measures byte for byte on both runs, the probe harness on E4, and every solver
gate in-process (tasks.md notes 6.3, 6.5). Three defects are on record: hashes over LF text while files were
CRLF, label collisions, one deadline from an unreceipted probe. Verdict: reproducible, with three recorded
defects whose fixes are pending in section 6.

Known failure modes. The record names each with its first occurrence: a thinking-only turn (twice); a surface
gap from the recorder's imports (once); a solver control failing on a date-heavy predicate (R24, R26); a
terminated arm reading as killed (3.2); an arm losing plugin-load errors (6.4); the harness reaping a long
script (6.5); an out-of-memory kill (E3). Verdict: known and recorded; two have automatic detectors (the patch
check, the receipted probe), the rest rely on the controller reading the receipt.

Per-language rebuild cost. Not measured. The record gives the per-scope cost on Python: 671 lines of scope
apparatus for E4, of which the recorder is 534 and scope-specific, plus 771 lines of experiment apparatus
(tasks.md notes 5.2 and 6.7). For Rust the only datum is R20's 429-line harness for one in-process scope.
Verdict: unknown for any second language; the recorder will set it, and it is also per scope.

Where the loop is not mature: no scope selection; a recorder written per scope; a solver rung that voids
itself on predicates of the kind both Facdrone scopes carry; two runs on one specimen. Where it is mature: the
decision rules, artefacts and budget discipline survived a change of regenerator and of scope without a change
of text, and the second run was audited by fresh agents at every step and by a held-out corpus that found
nothing.

## 9. Options and the recommended order

| Option | Cost, from the record | What it would disprove | What it leaves open |
| --- | --- | --- | --- |
| Third scope first, same protocol, the four amendments adopted | one working day of host time (E4: 10:08 to 16:28 UTC plus the morning's apparatus); about $7 of model time (E4 estimate); about 1,400 lines of apparatus (E4: 671 plus 771), most of it a new recorder alphabet | the lines row (a median under 10 fails it), the rounds row (any turn-4 acceptance or exhaustion), the trust row T7 (any held-out divergence) | other languages; scope selection |
| Rust core now, agnostic parts first, recorder pilot on a store-backed scope | unmeasured; the agnostic core is JSON in and out and can be regression-tested against the committed E3 and E4 artefacts; the recorder has one in-process data point (R20) | whether the contract survives a second language; the design's revalidation trigger says every bound component is unproven again on a new language | the table stays at one row of three; a failure in Rust would not say whether the contract or the recorder was at fault |
| Stop the retrospective track; carry the oracle, measures, gates and turn rule into prevention at generation time (R25 and R26, option 3) | none beyond the writing already done | nothing; the table stays at one row and the method stays Python-only | everything above |

**Recommendation: run a third scope first, under the four amendments of section 6, and write no Rust tooling
until its held-out row is in; the agnostic contract of section 7 is frozen for that run.** It costs what E4
cost and it is the only option that can falsify the preregistered table: a second line figure under 10 percent
ends the compression claim as written, a fourth turn ends the rounds claim, and a held-out divergence ends the
trust claim and sends the alphabet back for strengthening (applied report, section 8). If the third scope
holds T7, the contract of section 7 goes to an implementer with three scopes behind every agnostic row, and
the Rust recorder pilot on a store-backed scope is that implementation's first task. The recommendation, and
the claim under it that the prototype is mature enough for a Rust implementation of the agnostic parts, is
wrong if the third scope under the same protocol fails T7 or exhausts four turns, or if the Rust recorder of
R20 cannot be made byte-deterministic for a store-backed scope. Adoption of `cand-turn3` into Facdrone is a
separate decision outside the loop, on `residue.md` and the seven differences, with `missing-tests-e4.md` as
the tests to write first (handover, "Open items after E4").

## 10. References

Research entries (each with plan, receipts and logs in its experiment directory):

- R19, Dynamic deconvolution direction. `docs/research/2026-11-03/01-dynamic-deconvolution-direction.md`
- R20, Session-bus trace oracle (Rust). `docs/research/2026-11-03/02-e1a-session-bus-oracle.md`
- R21, Facdrone edge census. `docs/research/2026-11-03/03-e2-facdrone-edge-census.md`
- R22, Evidence-publication oracle (Python). `docs/research/2026-11-03/04-e1b-evidence-publication-oracle.md`
- R23, Concolic rung as run. `docs/research/2026-11-03/05-e1c-concolic-rung.md`
- R24, Corrected solver rung with verifier. `docs/research/2026-11-04/01-e1c2-corrected-solver-rung.md`
- R25, E3 evidence regeneration, with Amendments 1 to 3. `docs/research/2026-11-04/02-e3-evidence-regeneration.md`
- R26, E4 lifecycle regeneration. `docs/research/2026-11-06/01-e4-lifecycle-regeneration.md`
- Research index and H8 lineage. `docs/research/README.md`

Specification and handover:

- Deconvolution-loop requirements, design and tasks with Implementation Notes.
  `.kiro/specs/deconvolution-loop/requirements.md`, `design.md`, `tasks.md`
- Handover, state at 2026-11-06 evening. `.kiro/handover.md`
- Symbolic-execution tooling per language and the verifier rules. `docs/symbolic-execution-tooling.md`

Experiment directories under `docs/experiments/deconvolution/`:

- E4: `e4-lifecycle/plan.md`, `scope-rationale.md`, `README.md`, `review-e4.md`, `residue.md`,
  `differential/README.md`, `differential/pairs.md`, `mutants/README.md`, `results/e4-rates.json`,
  `results/missing-tests-e4.md`, `results/receipt.json`
- Regenerator program: `regenerator/README.md`, `regen_turn.py`, `results/receipt.json`
- Task 5.1: `t51-integration/README.md`
- Task 3.2: `e3b-original-sensitivity/README.md`
- Tasks 2.1 and 2.2: `e3-evidence-regeneration/results/missing-tests-experiment.md`, `testinfra/`
- E3: `e3-evidence-regeneration/`
- E1b, E1c-2, E1a: `e1b-facdrone-evidence-publication/`, `e1c2-concolic-rerun/`, `e1-mmhmem-session-bus/`

The applied paper for third parties, updated 2026-11-06: `docs/reports/deconvolution-loop-applied.md`.
Composition blueprint for this report: `docs/reports/blueprints/2026-11-06-run-report-blueprint.md`.
