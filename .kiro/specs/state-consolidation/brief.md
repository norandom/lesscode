# Brief: state consolidation

Written 2026-11-17, 20:55 Europe/Berlin (from the clock), from the owner's words of the same evening.
Discovery only. Not approved; there are no requirements, no design and no tasks yet.
It replaces the spec [model-simplification](../model-simplification/brief.md), which is superseded
([R133](../../../docs/research/2026-11-17/20-model-simplification-closed.md)).

## What the owner asked for

> "the hidden state machine / finding of scattered state across n files or code path"
>
> "the idea was scattered state consolidation building up on the solver approach"
>
> "I am aware that SAT is not always the same as a minimalized solution formally, but we just use SMT.
> we don't need to proof it"

## The problem in plain words

Some code behaves like a state machine that nobody wrote down. What an operation may do depends on what
happened before, and the pieces of that rule sit in different files: one place stores the state, another
reads it in a condition, a third changes it. To change the rule a maintainer has to find all the pieces
and keep them in agreement. That is scattered state.

Two things are wanted, in this order:

1. **Find it.** For a part of a code base, show the machine: what the thing is, which states it has,
   which events move it, which conditions guard a move, and the file and line of every piece.
2. **Consolidate it.** Propose a version in which the rule lives in one place, and check that the
   consolidated version behaves like the scattered one.

## The solver approach, and what "no proof" means here

This builds on what the project already tried under
[constraint-guided-compression](../constraint-guided-compression/brief.md): a model writes down the
behaviour as constraints, an SMT solver looks for an input or a history on which two versions differ,
and a counterexample goes back to the model. Earlier evidence that belongs to it:

- [R14](../../../docs/research/2026-11-02/14-aalpy-state-learning.md): automata learning recovered a
  four-state lifecycle from real Lesscode code, and concrete runs rejected every merge of two states.
- [R24](../../../docs/research/2026-11-04/01-e1c2-corrected-solver-rung.md): 7 of 10 planted
  changes were told apart from the original, 3 of them only by inputs the solver found; none was
  proved equivalent.

The owner's rule for this spec:

- SMT is the working tool. It is used to find differences and to check a consolidated machine against
  the scattered one within stated bounds (for example, all histories up to a length).
- No proof of equivalence is required, and none is claimed. "The solver found no difference within these
  bounds, and the project's tests pass" is the result that counts.
- The smallest possible machine is not required either. A machine that is in one place and correct
  within the bounds is good enough.
- That a result is not a proof is stated once, in the report. It is not a reason to stop, to defer, or
  to refuse a result.

Earlier decisions that set the solver aside (2026-11-03 and 2026-11-07, as an authority for equivalence)
are about proof. They do not forbid using a solver as a check, and this brief asks the owner to say so
explicitly when requirements are written.

## What can be reused

Checked on 2026-11-17 in [R134](../../../docs/research/2026-11-17/21-audit-of-port-and-reduction-claims.md).
Do not build on "the Rust verifier" or on the port `native-deconvolution-closure`: no command reaches
that code and its equivalence and mutation claims were never run on a specimen.

- Z3 itself: a binary is on this machine (`.runs/e1c-facdrone/extras/bin/z3.exe`) and seven queries of
  the first week replay with their recorded verdicts.
- Two regenerated candidates that pass their specimen's tests (E5, and E6 by its record), about 8% to
  10% smaller in syntax, as real before-and-after pairs of a state-boundary rewrite:
  `.runs/e2-facdrone/source` against `.runs/e5-state-boundary/cand-e5` and `.runs/e6-report-envelope/cand-e6`.

- The per-file facts the released analyzer already gives: write sites, branch conditions, function
  boundaries, declared gaps (`deconvolute analyze`, four languages).
- `lesscode-model` for every model call.
- The step flow (`step check`, `step guard`) to judge a consolidated version on the project's own tests.
- Two specimens, already prepared on 2026-11-17 ([R130](../../../docs/research/2026-11-17/17-hidden-state-machine-fitness.md)):
  - a small job machine written twice, once scattered over two files and once as one table, with a
    checker over all 31 histories up to length four;
  - the stale-worker fault of the mutation driver: the source before the fix (`60f79f3`) and the fix
    (`aa9d7d7`).

## What is deliberately left out

- Counting write sites or writers as the measure of success. On the job machine it gave the same
  numbers for the scattered and the consolidated version.
- Bookkeeping about how often a model's proposals are useful. First one machine found and one
  consolidated.
- Small local models for the reading. Use the configured endpoints through `lesscode-model`.
- Onboarding, release questions, and anything about 2.0.0. The release is published and stays.

## Owner's answers, 2026-11-17 (by menu)

1. **Find first, then consolidate.** Two stages in one spec. Stage 1 delivers the machine as a report
   and is usable alone. Stage 2 proposes the consolidated version and checks it with the solver, and
   starts only after stage 1 works on a real example.
2. **Solver: "the one we already use?"** That is Z3. `docs/experiments/cegis/` calls it directly, and
   the concolic experiments used it behind CrossHair. Both were Python programs, and the project now
   allows no Python tooling. To be settled in the design: Z3 run as an external program on SMT-LIB text
   written by the Rust tool, so that `lesscode-core` links nothing new. Z3 is not installed on this
   machine's search path today.
3. **Python first, then Rust.** The small job machine, where the right answer is known, then the
   stale-worker case in Lesscode's own driver.
4. **Where to look: "../Source has many options".** Read as: the owner names the place, and the real
   examples after the two specimens come from the owner's repositories under `D:\Source` (48
   directories on 2026-11-17, among them `mmhmem`, `mvibe`, `openidea-kiro`, `drift-rdp`). Those
   repositories are only read; work happens on copies under `D:\BuildArtifacts`. Not yet confirmed by
   the owner: whether the tool should also propose candidates across those repositories by itself.

## First check, proposed

On the job machine: the tool, given only the scattered version, returns the three states and their
moves with file and line; the solver finds no difference between the scattered version and the table
version on all histories up to length four; and it does find the difference when one guard is made
wrong on purpose. No model call is needed for the solver half of that check.

## 18 November · Prototype runtime decision

The owner approved Rust writing SMT-LIB and invoking the pinned Z3 executable, with a counted adapter to call the specimen’s CPython, for corrective 2.3. The E5 experiment found agreement over 81 inputs and reproduced a planted stale reading from a solver witness. The feature’s first discovery test remains the scattered job machine above. Production requirements, design and adoption remain open. [Experiment gate](prototype-gate.md) · [R139](../../../docs/research/2026-11-18/05-e5-rust-z3-check.md)

The proposed first discovery check subsequently passed on the prepared Python job machine. Rust recovered the scattered rules, identified the central table in the comparison package and reproduced a planted guard difference on the real fixture. The next discovery check is the pre-fix Rust stale-worker case. [R140](../../../docs/research/2026-11-18/06-rust-job-machine-discovery.md)

On the real stale-worker case, the historical CLI reproduced the wrong result and the fixed CLI ran the changed tests. The source-only model returned no real-case prediction at either output allowance. The discovery gate remains open. The next proposed revision extracts concrete facts about input identity and the working copy before asking for a state-machine interpretation. [R141](../../../docs/research/2026-11-18/07-real-rust-worker-discovery.md)

Rust subsequently traced the project test identity, worker acquisition, successful test-file copy and test process directory on the historical driver. Its reconstructed file hash matched all 69 launches, including the disabled-refresh control. The next proposed discovery test withholds the changed-test history and asks for a prediction from earlier observations and source facts. [R142](../../../docs/research/2026-11-18/08-native-worker-byte-trace.md)

The forecast passed on both historical revisions. From their own Rust source and pre-edit observations, separate model calls correctly predicted the test file, result reuse and selected mutation outcome for reopened and fresh workers. The investigator supplied the relevant code locations and the proposed edit. The next gate moves to a separately selected real case to test discovery. [R143](../../../docs/research/2026-11-18/09-worker-forecast-before-edit.md)

On Apache Commons CSV, Rust followed five public operations into five production files without a list of internal helper names. A separately recorded delivery attempt on an identical frozen request produced 36 correct behavioral predictions and 20 exact locations in 22 citations. The original request received no answer. The delivery result and the two missing initializer defaults support a revision to extraction before further consolidation work. [R144](../../../docs/research/2026-11-18/10-commons-csv-iterator-discovery.md)

Rust recovered both builder defaults. Applying the known iterator behavior with those values predicted all 108 JVM observations across the baseline, a different counter and a one-row limit. The citation check found unique locations for the two misplaced quotations. Deriving a transition from the Java source and replaying a solver-selected history is the next proposed test. [R145](../../../docs/research/2026-11-18/11-commons-csv-builder-defaults.md)

Rust lowered the iterator's closure guard from the Java source. Z3 found no difference for a parentheses rewrite and selected `hasNext, close, next` as the shortest history separating a changed guard. Fresh JVM replays and a census of all 121 bounded histories agreed with the solver. The other transitions still come from the hand-written R144 model. [R146](../../../docs/research/2026-11-18/12-commons-csv-closure-guard.md)
