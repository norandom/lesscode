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

## Questions for the owner before requirements

1. Is finding (step 1) a useful result by itself, or only together with consolidation?
2. Which solver: Z3, as in the earlier experiments?
3. Which language first? The specimens are Python (job machine) and Rust (mutation driver).
4. How is the place to look chosen: the owner names a directory, or the tool proposes candidates?

## First check, proposed

On the job machine: the tool, given only the scattered version, returns the three states and their
moves with file and line; the solver finds no difference between the scattered version and the table
version on all histories up to length four; and it does find the difference when one guard is made
wrong on purpose. No model call is needed for the solver half of that check.
