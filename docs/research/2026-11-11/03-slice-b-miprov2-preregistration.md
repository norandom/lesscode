# R77: Preregistration: the guide optimised with a MIPROv2 engine, beside the GEPA run

- Report date/time and timezone: 2026-11-11, Europe/Berlin (UTC+1)
- Execution interval: not yet run; written before any call
- Status: preregistered
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md), Slice B
- Hypothesis ID and predecessor: second engine for the protocol of
  [R70](../2026-11-10/16-slice-b-guide-optimisation-preregistration.md); compared with the GEPA run
  [R71](../2026-11-10/17-slice-b-guide-optimisation-opus.md)
- Type: preregistration of one confirmatory run

## Decision

Owner decision, 2026-11-11: run the MIPROv2 comparison on `claude-opus-5-5`, with the stop for model spend
raised to 40 USD. This entry fixes what is run and what would count as a result.

## What "MIPROv2" means here

In the `dsrust` repository MIPROv2 is not a separate crate; it is part of the main crate and built on its
program model. The tool therefore has its own module (`tools/guide-optimizer/src/mipro.rs`, commit `3f68560`)
that follows that code where this task has a use for it:

- Kept: the proposer's wording and its six tips; the starting instruction as candidate zero; the trial loop
  (tell the sampler the starting score, then ask, score, tell); the TPE sampler, copied unchanged.
- Left out, because the task has nothing for it: sets of demonstrations to search over (the task is one
  instruction and no examples in the prompt), the model-written descriptions of program and module, and
  trials on subsamples (MIPROv2 turns those on above 50 scoring examples; this scoring set has 4).
- Different: the dataset description is written by hand, and the tip of a proposal is chosen by its number
  and the seed instead of by a random draw.

With one instruction and no demonstrations the search space is one choice among the proposed instructions.
What remains of MIPROv2 is "propose several instructions without looking at failures, then let a sampler
decide which to score again". GEPA, by contrast, writes each new instruction from the judged replies of the
previous one. That difference is what this run can show.

## Hypothesis and falsification

| Claim | Statement | Refuted when |
| --- | --- | --- |
| M1 | The guide chosen after the search differs from the starting guide and has a mean score of at least 0.75 on the scoring set, which is what the GEPA run's kept candidate reached | the starting guide is chosen, or the chosen guide scores below 0.75 |
| M2 | In the final judgement of the chosen guide every first reply is admitted for at least three of the four languages (GEPA's guide: all four) | fewer than three languages with 5 of 5 |
| M3 | The adoption rule of R70 (at least 4 of 5 admitted and equal to the shipped pack, in every language) is not met, because of the Rust pack as in R71 | the rule is met |

Descriptive: proposals discarded by the rules, authoring calls used, cost, and which candidates the sampler
returned to.

What one run cannot show: which engine is better in general. Both runs are a single seed on a scoring set of
four calls; a difference of one admitted reply moves a score by 0.25.

## Frozen inputs and method

- Tool at commit `3f68560`, `--engine mipro --candidates 6` (MIPROv2's light preset), seed 0.
- Everything else as the GEPA run of R71: starting guide `pack-guide-v3.md`, Rust and Java for the search,
  Python and TypeScript held out, scoring set of two calls per training language, at most 60 authoring calls
  for the search, final judgement with five fresh calls per language, output limits 4,000 and 8,000 tokens.
- The rules a proposed guide must pass are the same (purpose unchanged, four sections, at most one and a half
  times the starting length, no node kind or check name). A proposal that fails is discarded and not retried.
- One input differs from R71: the shipped packs shown as examples now carry the `gaps` key.
- Budget: this run stops at 8 USD; expected about 6 USD. Recorded before: about 19.6 USD of 40.
- A refusal is an outcome and is not reworded.

## Evidence and results

None yet. Results go into their own entry.

## COBESY check

Outcome first: nothing measured; three claims fixed. Replay: tool commit, engine option, parameters and
budget named. Disproof: each claim can fail on the numbers of one run. The limits of what a single run on a
four-call scoring set can say are stated before the run.
