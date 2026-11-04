# R25: one regeneration round reaches the oracle's bar with 16% fewer lines and 29% fewer edges; the candidate's mutation sensitivity misses its prediction and one oracle-blind difference is real; H8(c) survives on one scope with a lowered protection claim

- Report date/time and timezone: 2026-11-04, Europe/Berlin
- Execution interval: 2026-11-04 09:28 to 14:53 Europe/Berlin (receipt, UTC+1)
- Status: completed; T1, T2, T3 held; T4 failed on its kill-rate clause and held on the hand-written mutants; T5 held on its prediction with a verifier-gate caveat; no kill criterion fired
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/brief.md); gate remains `revise`
- Hypothesis ID and predecessor: H8(c) regeneration behind the public API, registered in [R19](../2026-11-03/01-dynamic-deconvolution-direction.md); admitted after [R24](01-e1c2-corrected-solver-rung.md) by owner decision of 2026-11-04
- Type: preregistered, [E3 plan with amendments 1 and 2](../../experiments/deconvolution/e3-evidence-regeneration/plan.md); apparatus, commands and validation in the [E3 README](../../experiments/deconvolution/e3-evidence-regeneration/README.md)

## Decision

An LLM regenerated the five Facdrone evidence-publication modules behind
their mechanically derived public surface and passed the acceptance oracle
in the first of four allowed rounds: no Level 1 divergence on the 64
recorded sequences, identical R16 output, every applicable seed test passing
in all three test groups, and no Level 2 query-text divergence at all. The
candidate has 1,074 lines instead of 1,274, 546 AST statements instead of
660, 51 functions instead of 67 and 60 intra-set call edges instead of 84,
with all 11 write sites inside declared shell functions and 36 of 51
functions in a statically checked pure core. T1, T2 and T3 held as
measured and neither kill criterion fired. H8(c) is not rejected on this
scope.

The result is weaker than the predictions in three places and the reader
should hold all three. First, the candidate's sensitivity to mutation is
lower than preregistered: 18 of 28 competent cosmic-ray mutants were killed
(0.643 against 0.70 required), although all three hand-written mutants of
E1b, including the stored-representation change that only the trace replay
sees, were caught. Second, the independent reviewer showed that part of the
statement reduction is syntactic packing: reformatted into the original's
style the candidate has about 572 statements, above the 561 bar, and the
line bar was met by nine lines. The edge reduction, the function count and
the pure-core split do not depend on formatting. Third, T5 found a real,
concretely replayed difference the oracle cannot see: after an exception
inside `capture_payload` the original leaves an `AuditedAbsence` in the
capture's bookkeeping and the candidate leaves nothing, because the
candidate calls the route after deduplication instead of before. That holds
T5's prediction, but with two honest qualifications: the reviewer had
already named this difference by reading (H4), and the 300-input random
soundness corpus exposed it on 151 inputs before the solver ran; and on the
plan's strict wording of verifier check 2 the pair that produced the
counterexample fails the gate, which makes the solver's own verdict
inconclusive under that reading. The difference is a fact either way.

What this permits next: a second round on this candidate against the known
gaps, or a second scope, with the oracle's protection declared at its
measured level. It does not permit calling the candidate equivalent, and it
does not permit adoption.

## Results against the preregistered predictions

| Prediction | Result | Held |
| --- | --- | --- |
| T1: zero Level 1 divergence on 64 sequences, identical R16 output, all seeds pass, every Level 2 pair passes `qdiff.py`, within 4 rounds | Round 1: 64 sequences, 311 operations, 0 Level 1 divergences, 0 Level 2 pairs (so `qdiff.py` had nothing to judge); root 1,403 passed, 2 skipped, 10 deselected; research 39 passed, 3 skipped, 6 deselected; reporting 92 passed, 8 deselected, 1 xfailed; exit 0 in 768.8 s at factor 1.79 | Yes, in 1 round |
| T2: statements at most 561, lines at most 1,083, functions at most 67 | 546 statements, 1,074 lines, 51 functions | Yes, with the reviewer's packing caveat |
| T3: edges at most 67; writes at most 13, all in shell; pure core at least 50% and static check passes | 60 edges; 11 writes, all in the 15 declared shell functions; 36 of 51 pure (0.706); structure check 0 failures, 0 undeclared | Yes |
| T4: kill rate at least 0.70 on the 30-mutant sample; M1, M2 and ported M3 caught | 18 killed, 10 survived, 2 incompetent: 18/28 = 0.643 (R22 convention, survivors in the denominator); M1, M2 caught by seed tests, M3 port caught by the replay only | No on the rate; yes on M1 to M3 |
| T5: at least one realizable oracle-blind counterexample on the first T1-passing candidate | `capture_payload` pair: counterexample, realizable on concrete replay, reachable through the public function, not catchable by the oracle's alphabet; the other two pairs inconclusive; both controls held; verifier check 2 failed for the capture pair (151 of 300 random inputs differ, the same difference) | Yes on the prediction; inconclusive under a strict reading of check 2 |

### T4 detail

Population 317 mutants of the candidate's `decision_evidence_publication.py`,
sample of 30 with seed 20261104; arm = R16 check, replay of the 64 sequences
and the 19 seed files that import the module (6 deselections); unmutated arm
199.9 s, deadline base 999.5 s scaled. Fifteen mutants fell at the R16
check, one at the replay (line 320, zlib level 1 to 0, seen at
`S09-codecs-compact` op 10), two at a seed test. The two incompetent mutants
are syntax errors on a `*` keyword marker.

| Survivor | Line | Reading |
| --- | --- | --- |
| m4, m11 (`\|` to `+`, `-`) and m8 (`\|` to `%`) | 287, 216 | annotation-only; never evaluated under `from __future__ import annotations` |
| m24 (`!=` to `<` on set length) | 86 | equivalent, since a set is never longer than its source |
| m28 (`strict=True` to `False` in `zip`) | 136 | equivalent for sequences of equal length |
| m15 (`==` to `>=` on `route.kind`) | 267 | equivalent only for the closed set of route kinds |
| m16 (`==` to `is "scored"`) | 256 | survived because the kind arrives as the interned literal |
| m17 (`==` to `is not` on a consumed URI) | 313 | real gap: no malformed `consumed:` reference in the corpus |
| m21 (`!=` to `is` on ledger currency) | 76 | real gap: no sequence carries a ledger without `currency` |
| m29 (pydantic `strict=True` to `False`) | 226 | real gap: lax validation of the captured JSON is never exercised |

Post hoc, labelled: excluding the five mutants equivalent by inspection (m4,
m8, m11, m24, m28) gives 18 of 23 (0.783). This reading was not
preregistered and is not the result. E1b's 0.746 (R22) came from a
different population (the original module plus `state_repository.py`) and is
not a baseline for this number.

### Reviewer findings (step 2, independent sonnet agent)

Within the rules: 2,798 Python files in each tree, none added or removed,
exactly the five files differ; 58 surface names with the same kind and
signature; apparatus and golden hashes unchanged; measures reproduce
text-identically. Caveats: 26 statements of the reduction are packing
(candidate at about 572 statements in the original style; pure share by
statements 0.62 rather than 0.706 by functions); maximum line length rose
from 100 to 128 characters; character count fell 4.7%. Of nine behavioural
hypotheses, 76 probes agreed on 74 and differed on 2, both the generator
case of `deduplicated_inputs` (outside its `Sequence` contract). Hypothesis
H4, the reordered route call in `capture_payload`, was named by reading and
later confirmed by T5.

## T5: oracle blindness

| Pair | Verdict | Solver seconds |
| --- | --- | ---: |
| `deduplicated_inputs` (list domain; the reviewer's generator case is excluded by construction) | inconclusive: no differences in 1,379 iterations, not exhausted | 351.3 |
| `capture_payload` with `capture.absences` observed after the call | counterexample | 353.4 |
| sealing path, `seal_evidence` and `seal_evidence_with_run_slices` with a symbolic namespace switch | inconclusive: no differences in 511 iterations, not exhausted | 342.0 |

The counterexample: an input whose payload cannot be serialised. Both
sides raise the same error (the solver reports a non-finite float, the
concrete replay a `bytes` value; the shape is the same). Afterwards the
original's `capture.absences` holds `AuditedAbsence('scoring', 'policy
returned before scoring')` and the candidate's holds nothing. It is
reachable through the public `capture_payload` with inputs its types allow.
No caller the agent read inspects the capture after the exception.

Verifier: (1) symbolic-ness held, with 41/40 entries for the dedup pair,
8/8 for capture and, after the per-condition timeout was raised from 15 s to
60 s because each path takes about 15 s, 4/4 and 2/2 for the seal pair; (2)
soundness on 300 Hypothesis inputs per pair, seed 20261104, held for dedup
and seal (300 of 300 agree) and failed for capture (151 of 300 differ, all
with the H4 shape; there were no recorded inputs for these drivers, see
Amendment 2); (3a) the planted mutant (line 135, `!=` to `>`) produced a
counterexample, realizable on replay; (3b) the renamed-locals variant
exhausted on a domain shrunk to three booleans; (4) the counterexample
differs on concrete replay. Both controls held, so T5 is not void. The
oracle replay of the counterexample as a new sequence was not executed: the
oracle has no operation that calls `capture_payload`, and adding one would
change the frozen E1b/E1c-2 apparatus, which step 3 may not do. On its
current alphabet the oracle cannot catch this difference, which repeats
R23's alphabet-gap finding rather than demonstrating solver depth.

## Frozen inputs, method and budget

Frozen copy `.runs/e2-facdrone/source` at `d49f48bd…` as in E1b to E1c-2;
candidate `.runs/e3-facdrone/cand` (native copy, 4,851 files identical
before regeneration); mutant copy `.runs/e3-facdrone/mut`, restored and
hash-verified after every mutant and once more after a host out-of-memory
kill during mutant 16 (rerun from scratch; the partial log is kept); final
check 4,851 files identical. Public surface derived by `surface.py`: 62
names, 15 beyond the plan's transcription (recorded in the README).
Acceptance corpus: the three golden traces (51 + 2 + 11 = 64 sequences), the
R16 observations, 191 seed files with the 24 applicable baseline
deselections. Apparatus validated on the frozen copy itself before any
candidate (exit 0 in 546 s, factor 1.24).

Regenerator: one opus agent, one rewrite of all five files; changes listed
in `results/regeneration-log.md` (sealing paths folded into one `_seal`;
six record checks into one `_validate`; resolver walk, rebuild and refusal
into one `_resolve`; several one-call helpers inlined; `SliceResolver`
replaces the two caches). Reviewer: one sonnet agent with the plan, the
diff and the candidate only. T4 and T5: one opus apparatus agent; CrossHair
0.0.112 with z3, cosmic-ray 8.7.0 through `uv tool run`, CPython 3.13.16.

Adaptive budgets applied to every heavy step; factors ranged from 1.04 to
3.6; no timeout and no rerun. A memory guard (idle wait, free-RAM floor)
ran before every T4 and T5 step after the host kill and is recorded per
step in the receipt. Failed gate iterations (a Hypothesis restriction on
default arguments, CrossHair's audit wall on an import-time hostname call,
a PowerShell case-insensitive variable) stay in the receipt as failures.

| Budget | Preregistered | Used |
| --- | ---: | ---: |
| Regeneration rounds | 4 | 1 |
| Acceptance deadline | 2,569 s base, scaled | 768.8 s (round 1); 620.4 / 601.4 / 950.6 s (M1 / M2 / M3) |
| T4 mutants | 30 | 30 (2 incompetent) |
| T5 | 3 pairs × 300 s base plus controls | 3 pairs; 1,560.1 s solver time including controls and gate iterations |
| Research code | 400 lines | 545 lines (326 apparatus at step 0; 70 `t4_arm.py`; 149 `t5_drivers.py`); 145 over, reported, nothing compacted |

Native Windows, plain user account; no WSL and no bare `bash` from scripts.
`D:\Source\facdrone` untouched.

## Limits and interpretation

- The oracle's protection on this scope is what T4 and T5 measured, not
  what T1 shows: a candidate can pass every recorded sequence and still
  differ on inputs the corpus never carried (m17, m21, m29, the
  `capture_payload` exception path). "Zero divergence" is a statement about
  64 sequences.
- Every real gap found is on malformed or failing inputs, where recorded
  traces are thinnest. The reviewer's reading and 300 random inputs found
  the exception-path difference; the solver confirmed it and found nothing
  else in two further pairs. On this scope the solver added no difference
  that cheaper means had not already exposed.
- Statement reduction is partly formatting; edges, functions and the
  pure-core split are the robust part of the compression claim.
- The T4 rate is not comparable with R22's: different population, different
  seed selection. A future sensitivity measure should sample the original
  and the candidate alike.
- One scope, five files, one regenerator, one round, one host. Nothing about
  the other target languages.

## Tool/spec consequence and next gate

H8(c) stands on one scope with these numbers; the protection claim that
goes with it is the acceptance corpus plus 0.643 measured mutation
sensitivity plus one known oracle-blind difference on the exception path,
with the declared residue (E1c-2 targets 166, 169, 200; unreached
functions; malformed-input paths). Two protocol rules are proposed for the
spec from this entry: statement counts are reported in the original's
formatting as well as the candidate's, and sensitivity samples are drawn
from the original and the candidate alike so the rates are comparable.

Options for the owner:

1. E3b: a second regeneration round on this candidate aimed at the three
   real T4 gaps and the exception path, with the corpus extended by the
   realised counterexample as a new oracle operation (an apparatus change,
   so a new preregistration), to see whether the sensitivity number moves.
2. E4: the same protocol on a second Facdrone scope, to test whether one
   round suffices again and whether the packing caveat recurs.
3. Stop the retrospective track and carry the oracle, measures and verifier
   into prevention at generation time.

Evidence: [experiment directory](../../experiments/deconvolution/e3-evidence-regeneration/),
`README.md`, `plan.md`, `results/receipt.json`, `results/logs/`,
`results/regeneration-log.md`, `results/review-step2.md`,
`results/measures-*.json`, `results/t4-mutant-sample.json`,
`results/t5-soundness.json`.

## COBESY check

Outcome first; the failed T4 prediction is reported as failed and the
reviewer's packing caveat sits next to the T2 success; T5's gate caveat is
stated with the result it qualifies; facts (counts, hashes, survivors,
verdicts) are separated from readings (equivalent by inspection, real gap);
the post hoc rate is labelled; no equivalence or adoption is claimed; the
decision is the owner's.
