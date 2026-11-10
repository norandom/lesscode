# R70: Preregistration of a bounded, verified optimisation of the pack guide in the style of MIPROv2, in Rust (not run)

- Report date/time and timezone: 2026-11-10, Europe/Berlin (UTC+1)
- Execution interval: none. This entry fixes the protocol before any call is made.
- Status: preregistered, not run
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md), Slice B
- Hypothesis ID and predecessor: follows [R69](15-slice-b-baseline-rerun-stronger-admission.md); owner
  direction of 2026-11-10 ("mipro makes sense"; and, on doing it with DSPy in Python: "no, you can find a
  rust equivalent here")
- Type: confirmatory (one optimisation run with a fixed acceptance rule)

## Decision

Nothing is decided here except the protocol. The optimiser is written in Rust as research tooling inside the
native crate (`native/examples/guide_optimizer.rs`), on what the crate already has: the endpoint loader and
HTTP transport of the regenerator, and the driver's admission, control replay and mutant listing. No Python
and no DSPy are used. It is not part of the shipped binary.

## What is taken from MIPROv2 and what is not

Taken: instruction candidates are proposed by a model that is grounded in the current instruction, in
observations from earlier runs and in one "tip" per candidate; candidates are scored by the task's own
metric on small batches; the best are re-scored on more data; the result is judged on data kept apart.

Not taken, because the budget is small: there is no search over demonstrations (the example packs shown stay
as fixed in R66), and no Bayesian surrogate chooses the next candidate; selection is two rounds of successive
halving. This is therefore "in the style of" MIPROv2, not an implementation of it, and it is named so.

## Hypothesis and falsification

Baseline (R69, guide version 3, first replies): Rust 0 of 5 admitted; Java 5 of 5 admitted but 2 of 5 with
the shipped pack's mutants; Python and TypeScript 5 of 5 on both.

Claim: an optimised guide exists within this budget that, on fresh runs,

- H1: reaches at least 4 of 5 first replies admitted for Rust and for Java,
- H2: reaches at least 4 of 5 first replies that are admitted and give the shipped pack's mutants, for Rust
  and for Java,
- H3: does not regress on the languages kept apart: at least 4 of 5 on both measures for Python and for
  TypeScript.

The optimised guide is adopted only if all three hold. If selection ends with the baseline guide as the best
candidate, the result is "no improvement found" and nothing is adopted. A refusal counts as a failed trial
and is never reworded.

## Frozen inputs and method

- Model: `claude-opus-5-5` through the owner's endpoint, for proposals and for trials. Sampling settings are
  the endpoint's defaults.
- Baseline instruction: `pack-guide-v3.md`
  (SHA-256 `dc49e14bb0d62d1560724b828e54e1c7d216ca5bd47cb37580b618b4c7d02bbb`).
- Languages used for optimisation: Rust and Java. Kept apart for the final judgement: Python and
  TypeScript. Example packs shown per language as in R66.
- Observations given to the proposer: `docs/experiments/annotation-driver/slice-b/observations.md`, written
  from the Rust and Java results of R68 and R69 only, in general terms without node kind names.
- Proposals: four candidates, one per tip (completeness checklist; decision rules for ambiguous cases; say
  why each field matters; state what must never be listed). Each proposal is one call that returns a full
  revised guide.
- Verification of every candidate before it is tried (a candidate that fails is discarded, not repaired):
  - its first paragraph, the statement of purpose, is that of version 3, unchanged;
  - it still has the sections "Fields", "Operators", "Rules" and "Output";
  - it is at most 1.5 times as long as version 3;
  - it contains no node kind name used in any shipped pack that version 3 does not already contain, and no
    control label. This keeps a candidate from carrying the answers.
- Trial: one fresh conversation (system message: the candidate guide; user message: the example packs and
  the language). The first reply is checked by the driver's admission with control replay on every gate
  scope of the language and the hand-written control sets; an admitted pack's mutants are compared with the
  shipped pack's on the gate scopes. No repair turns. Score of a trial: 0 not admitted, 0.5 admitted, 1
  admitted with the shipped pack's mutants.
- Selection: round 1 scores the baseline and every verified candidate on Rust and Java, two trials each;
  the best two by mean score go to round 2 with three more trials per language. The best by mean score over
  its five trials per language is the selected guide; ties go to the shorter text.
- Final judgement: the selected guide, if it is not the baseline, gets five fresh trials on each of the four
  languages. Only these count for H1 to H3.
- Outputs: `docs/experiments/annotation-driver/results/slice-b/optimise1/` (candidates, every request and
  reply, every trial, `summary.json`).

## Cost bound

At most 56 calls: 4 proposals, 20 trials in round 1, 12 in round 2, 20 in the final judgement. At the list
price of this model the two baseline runs cost about 0.05 USD per trial; proposals return a whole guide and
cost more. Expected total: under 4 USD. Hard stop: no call is made when the spend of this run would pass
6 USD or recorded spend would pass 20 USD (5.89 USD recorded before this run, at list prices).

## Evidence and results

None. Nothing was run.

## Limits and interpretation

- The languages kept apart were already at the ceiling in R69, so H3 can show absence of regression, not
  transfer of an improvement. A real test of transfer needs a language whose grammar is not yet compiled
  into the crate.
- Two languages for optimisation and five trials per cell; an improvement of one or two trials is within
  what repetition alone can produce. The acceptance rule is deliberately coarse.
- The proposer sees observations from the same two languages that selection is scored on. The verification
  rule limits, but does not exclude, a guide tuned to them.
- "Same mutants as the shipped pack" treats the hand packs as the target, which R68 showed can be wrong.
- One endpoint, one evening. Drift over time and across models is not measured.

## Tool/spec consequence and next gate

- Results go into a new dated entry that links back here; changes to this protocol before the run are dated
  amendments below.
- If the guide is adopted it becomes `pack-guide-v4.md`; the shipped packs are not regenerated from it
  without a separate decision.

## Amendment 1, 2026-11-10 (before any call and before any optimiser code; owner directions)

Three directions from the owner arrived while this entry was being written. They replace parts of it.

- Optimiser: "https://github.com/getlatentic/dsrust for example". The hand-rolled selection described above
  under "What is taken from MIPROv2 and what is not" and "Selection" is dropped; it was never written. The
  optimiser is the `MIPROv2` of the `dsrust` crate, a Rust port of DSPy. What was established about it:
  version `0.1.0-alpha.3` is published and resolves on the pinned toolchain; licence MIT or Apache-2.0; it
  ships `MIPROv2`, `GEPA`, `COPRO` and bootstrap optimisers, a `ChatModel` trait for custom models, an
  OpenAI-compatible provider with a configurable base URL and an Anthropic provider bound to Anthropic's own
  URL. It is an alpha by one author and pulls about 200 crates, so it goes into a separate research package
  (`tools/guide-optimizer/`) with its own lock file and is never a dependency of `lesscode-core`.
  Not yet established: its `MIPROv2` parameters and metric interface, and whether its results on this task
  are sound. Those are the first things to check, on a run without model calls.
- Budget: "I'd simply assume the turns or other parameters get set by the user consciously, our defaults
  should be smart". The fixed 6 USD stop of this entry is replaced by parameters the user sets (trials,
  candidates, runs per language), with defaults chosen for a small run and printed with their expected cost
  before the first call. The project-wide 20 USD stop remains until the owner changes it.
- Second endpoint: the owner supplied a Mistral configuration (`mistral-large-4`, OpenAI-compatible). It is
  in the git-ignored `.env`; `.env.sample` documents it. This is the second model arm that R66 left open.

Unchanged: the metric (first-reply admission, and mutants equal to the shipped pack's), the languages used
for optimisation and kept apart, the verification of candidate guides, and the acceptance rule H1 to H3 on
fresh runs. Still to be written and hashed here before the first call: the observations file, the default
parameters, and the commit of the optimiser package.

## COBESY check

Outcome first: a protocol, nothing run. Replay: model, inputs, counts, scoring, selection and acceptance are
fixed. Disproof: three conditions on fresh runs; "no improvement found" is a stated outcome. What differs
from MIPROv2 is said plainly, and the weakness of the held-out comparison is named.
