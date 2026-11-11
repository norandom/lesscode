# R79: A MIPROv2-style search does not fix Rust admission on Opus; the GEPA run did

- Report date/time and timezone: 2026-11-11, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-11, 08:07 to 08:20 local (one invocation; times from the run's output)
- Status: completed
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md), Slice B
- Hypothesis ID and predecessor: runs [R77](03-slice-b-miprov2-preregistration.md); compared with the GEPA run
  [R71](../2026-11-10/17-slice-b-guide-optimisation-opus.md)
- Type: confirmatory (one run with claims fixed beforehand)

## Decision

With the same model, starting guide, rules and number of authoring calls, the MIPROv2-style search ends with
a guide that is no better than the one it started from: Rust packs written under it are never admitted on
the first reply (0 of 5 in the final judgement), where the guide found by the GEPA run got 5 of 5. Nothing is
adopted from this run.

The reading that fits both runs: the starting guide's weakness shows only in the judged replies (which
control failed, which operator was missing). GEPA writes its next guide from those; MIPROv2 proposes its
guides from a description of the task and a tip, before any failure is seen, and then only chooses among
them. For this task the first kind of information was the one that mattered. One run of each engine does not
establish that in general.

For hardening against drift the consequence is unchanged from R71: the barrier is what protects, and an
optimised instruction is an improvement worth having only where the model already knows the grammar.

Separate outcomes: pack authoring on one endpoint. No project's tests were run against these packs.

## Hypothesis and falsification

| Claim of R77 | Result | Verdict |
| --- | --- | --- |
| M1: the chosen guide differs from the start and scores at least 0.75 on the scoring set | it differs; it scores 0.50 | refuted |
| M2: in the final judgement every first reply is admitted for at least three of four languages | Java, Python and TypeScript 5 of 5; Rust 0 of 5 | holds |
| M3: the adoption rule of R70 is not met, because of the Rust pack | not met | holds, but for another reason than in R71: Rust is not admitted at all, and Python equals the shipped pack in 2 of 5 |

## Frozen inputs and method

As R77: tool at commit `3f68560`, `--engine mipro --candidates 6`, seed 0, 60 authoring calls for the search,
scoring set of two Rust and two Java calls, final judgement of five fresh calls for each of four languages,
stop at 8 USD. Results: `docs/experiments/annotation-driver/results/slice-b/optimise2-opus-mipro/`.

## Evidence and results

The search:

- Five proposals were requested. Two were discarded by the rules for length (10,252 and 9,236 characters
  against 9,073); three were kept, so the sampler chose among four guides including the starting one.
- 15 scorings of the whole set, 60 authoring calls. Mean score per guide over its scorings: starting guide
  0.46 (three scorings: 0.375, 0.5, 0.5), candidate 1 0.50 (five), candidate 2 0.50 (two), candidate 3 0.35
  (five).
- A score of 0.5 is two Java packs admitted and equal to the shipped pack, and two Rust packs rejected. In
  the search no Rust pack was admitted under any of the four guides (0 of 30 calls); the rejected packs fail
  the hand-written Rust controls, as in R69. Java was admitted in 29 of 30 calls and equal to the shipped
  pack in 24; all differences between the guides' scores are Java packs that were admitted but differed.
- The chosen guide is candidate 1 (7,522 characters; the starting guide has 6,049). Its lead over the
  starting guide, 0.50 against 0.46, is one Java reply of the starting guide's six that differed from the
  shipped pack.

Final judgement, five fresh calls per language:

| Guide | Rust admitted | Java admitted / equal | Python admitted / equal | TypeScript admitted / equal |
| --- | --- | --- | --- | --- |
| version 3, the start (control arm of R78 this morning; Rust on the first reply, "equal" counted after repair) | 0 of 5 | 5 / 3 | 5 / 5 | 5 / 5 |
| GEPA's guide (R71) | 5 of 5, none equal to the hand pack | 5 / 5 | 5 / 5 | 5 / 5 |
| MIPROv2's chosen guide (this run) | 0 of 5 | 5 / 5 | 5 / 2 | 5 / 5 |

- No call was refused.
- Cost: 85 calls (60 search, 5 proposals, 20 final), 5.82 USD at list prices. The GEPA run with its top-up
  cost 6.49 USD. Recorded spend after this run: about 25.5 of 40 USD.

## Limits and interpretation

- One run, one seed. A scoring set of four calls moves in steps of 0.125 with one reply, and the starting
  guide itself scored between 0.375 and 0.5 on it.
- The module is not the whole of MIPROv2: demonstrations in the prompt, which are half of what MIPROv2
  searches over, have no place in this task as it is set up. A version of the task that showed the author an
  admitted pack for the same language as a demonstration would be a different task (and for a new language
  no such pack exists).
- The proposer saw one admitted Java pack as its example of the task and no rejected reply. That is how
  MIPROv2 grounds its proposals; it is also the plain reason it could not know what was wrong with Rust.
- The examples shown to the author carry the `gaps` key, which they did not in R71. The same-day control arm
  of R78 shows the starting guide behaving as in R69 (Rust 0 of 5 on the first reply), so this difference
  does not explain the result.
- The row for version 3 is taken from the baseline runner's control arm of R78, not from this tool's
  final-judgement procedure; the two count "equal to the shipped pack" at slightly different points.

## Tool/spec consequence and next gate

- The tool keeps both engines behind `--engine`. The rules, the budget stop with the held-back final
  judgement and the selection rule are shared.
- No guide is adopted. The shipped packs remain hand-written; the owner decided on 2026-11-11 not to take
  any of the four Rust points of R71 into the shipped pack.
- Slice B state after R66 to R79: drift is measured for pack authoring between runs (R68, R69), between
  models (R72, R78) and under two optimisers (R71, R74, this entry). Not yet measured: the annotation layer,
  and drift of one model over time. Those are the owner's next decision.

## COBESY check

Outcome first: the claim about the score is refuted and the run adopts nothing; the comparison with GEPA is
stated with the numbers of both final judgements. Replay: tool commit, engine option, parameters and result
directory named. Disproof: M1 failed as written. What the module leaves out of MIPROv2, and that one run per
engine cannot rank them in general, is stated.
