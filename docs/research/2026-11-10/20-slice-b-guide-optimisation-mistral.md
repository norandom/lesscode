# R74: Guide optimisation with the GEPA engine on Mistral Large 4: no improvement found

- Report date/time and timezone: 2026-11-10, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-10, 21:57 to 23:22 local
- Status: completed for `mistral-large-4`
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md), Slice B
- Hypothesis ID and predecessor: runs [R70](16-slice-b-guide-optimisation-preregistration.md) on the second
  model; compare [R71](17-slice-b-guide-optimisation-opus.md) for `claude-opus-5-5` and the baseline
  [R72](18-slice-b-baseline-mistral.md)
- Type: confirmatory (one optimisation run with a fixed acceptance rule)

## Decision

For this model the optimisation found nothing to adopt: after 62 metric calls the starting guide and the one
candidate the engine kept both score zero on the scoring set, so the selected guide is the starting guide
and no final judgement was run. The same engine, protocol and budget of metric calls that fixed admission
for the first model (R71) did not help the second. Rewording the instruction does not give a model the
grammar's node kind names, which is what it lacks (R72).

Separate outcomes: one optimisation run on one endpoint. No project's tests were run. No compression was
measured.

## Hypothesis and falsification

R70's claims H1 to H3 apply to a guide that differs from the starting guide. Selection ended with the
starting guide, which R70 names as the outcome "no improvement found": nothing is adopted and the three
claims are not evaluated.

## Frozen inputs and method

As R70 Amendment 3, with the parameters of the Opus run: starting guide `pack-guide-v3.md`, Rust and Java
for optimisation, 60 metric calls, minibatch 3, seed 0. Tool at commit `419da6f` plus the output-limit
parameters committed in `1d1b339`. Model `mistral-large-4` as author and as reflection model.
Results: `docs/experiments/annotation-driver/results/slice-b/optimise1-mistral/`.

Differences from the Opus run, all stated before the run or forced by the model:

- Output limits of 16,000 tokens for authoring and 24,000 for reflection instead of 4,000 and 8,000,
  because this model counts its reasoning against the limit (R72).
- Budget stop of 2.50 USD for the run instead of 6 USD.
- The shipped packs shown as examples were the same files as in the Opus run; the `gaps` key was added to
  them only after this run had read them.

## Evidence and results

| Guide | Rust admitted | Java admitted | Java equal to shipped | Mean score on the scoring set |
| --- | --- | --- | --- | --- |
| version 3 (start) | 0 of 8 | 0 of 11 | 0 | 0 |
| candidate 1 (7,749 characters) | 0 of 17 | 4 of 17 | 3 | 0 |

- The engine ran 13 iterations and 62 metric calls. It kept one candidate, which did better than its parent
  on a minibatch (some Java packs admitted) but scored zero on the scoring set, like the starting guide.
- Ten proposals were discarded by the barrier: eight for length (9,395 to 11,196 characters against a limit
  of 9,073) and two because they changed the statement of purpose.
- 18 of the 62 authoring calls ended at the output limit without a complete pack.
- No call was refused.
- Cost: 76 calls, 1.99 USD (62 authoring calls with 155,612 input and 744,741 output tokens, 14 reflections
  with 43,670 and 137,820; almost all output is reasoning).

## Limits and interpretation

- One run, one seed, and a small budget for a model whose calls take minutes; the engine made few steps.
- The barrier discarded ten of eleven proposals. For this model the length limit and the fixed statement of
  purpose were binding in a way they were not for the first model; whether a looser barrier would have
  produced an admissible guide is not known.
- Rust was never admitted under either guide, so the engine had no gradient to follow there: every Rust
  trial scored zero, whatever the guide said.
- The two model arms differ in output limits and budget, as stated.

## Tool/spec consequence and next gate

- Reported side by side, the two models show what optimisation of the instruction can and cannot do: it
  removed instability for a model that knows the grammars and did nothing for one that does not.
- The idea noted in R72 stands and is now better supported: give the author the grammar's node kinds (the
  driver can list them) instead of asking the instruction to compensate. That is a change of the task's
  input, to be decided by the owner and preregistered before it is tried.
- The tool now holds back the final judgement's budget by the measured cost per call, since this model's
  calls cost several times the estimate built into the tool.
- Recorded spend at list prices after this run: about 15.2 of 20 USD.

## COBESY check

Outcome first: no improvement, by the rule fixed beforehand. Replay: tool, parameters, model and results
named; the differences from the first model's run are listed. Disproof: the acceptance rule was not reached
because selection ended with the starting guide. What the run does not show is stated, including that the
barrier may have been too tight for this model.
