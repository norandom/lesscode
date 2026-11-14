# R110: The indicator on four real cases: every one is "good enough direct". The built-in set does not tell the models apart by its verdict; the cost per accepted answer differs by a factor of eleven

- Report date/time and timezone: 2026-11-14, 09:50, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-14, 09:15:49 to 09:43:48
- Status: completed as a measurement; the question it was to answer (does the set separate the models) is answered with no
- Spec: [model-seam](../../../.kiro/specs/model-seam/requirements.md), requirements 10.2, 10.3, 10.9, 10.10, [task 5.3](../../../.kiro/specs/model-seam/tasks.md)
- Hypothesis ID and predecessor: the indicator of requirements Amendment 2 (2026-11-13); built in task 4.3 against a stub only
- Type: live measurement; the task named beforehand that it can fail (written 2026-11-13)

## Decision

The indicator works on real endpoints: each run gives counts, rates, cost and a verdict, and no trial
was lost to an endpoint failure. As a means to tell models apart the built-in set fails. All four cases
get the same verdict, and the first-reply rates (85 % to 100 %) are too close for twenty trials to
separate. The task's rule for hardening (every model passes every first reply) is not met to the letter,
since three of the four cases had a first reply rejected. I have not hardened the set and not ticked the
task: by the plan's rule a failure goes back to the design, and what the indicator is meant to say is
the owner's decision.

What the run does show is a difference in cost, which the verdict does not use: on the same service, Mistral Large
needs about eleven times the output tokens and twenty times the seconds of GLM 5.3 for an accepted answer.

## Hypothesis and falsification

`measure` on the built-in set gives different verdicts, or clearly different first-reply rates, for
models that differed on harder work (pack authoring: Mistral Large had 5 of 20 first packs admitted in
R72, GLM 5.3 reached 19 and 20 of 20 with repair turns in R99). Rejected when all get the same verdict and the rates lie within what twenty trials cannot
distinguish. The quick mode's outcome agrees with the full verdict; rejected for a case where the quick
mode says something the full run contradicts.

## Frozen inputs and method

- Binary: `lesscode-model`, debug build of main at `4fdcf57`; `measure`, at most 2 repairs.
- Task set: built-in (four small modules: total price, shipping fee, discount, orders). Check: the core's
  parsing and static checks on a copy (`parse_result`, `prepare_candidate`, `evaluate_tier1_static`).
- Per case one quick run (5 trials) and then one full run (20 trials), by
  `docs/experiments/model-seam/run-measure.ps1`. Cases 1, 2 and 5 one after the other; case 3 beside
  them, on another upstream. The Anthropic case is left out, as the task says.
- Endpoint files outside the repository and the repository's `.env`; reasoning level as each file states
  (high for cases 1, 2 and 5, medium for case 3).
- Thresholds as built: direct from 3/4 of first replies, with the repair loop from 9/10 passed.

## Evidence and results

Reports as text and JSON in `docs/experiments/model-seam/measure/`. A is the share of first replies
accepted, R the share accepted within two repairs; task set and check as above.

Full runs, 20 trials each, none not run:

| Case | A | R | Calls | Output tokens per accepted answer | Seconds per accepted answer | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| 1 GLM 5.3, Mistral's API | 19 of 20 (95 %) | 20 of 20 | 21 | 450 | 2.9 | good enough direct |
| 2 Mistral Large, Mistral's API | 17 of 20 (85 %) | 20 of 20 | 24 | 5,207 | 60.8 | good enough direct |
| 3 `gpt-6.1-sol`, OpenAI's API | 20 of 20 (100 %) | 20 of 20 | 20 | 475 | 9.5 | good enough direct |
| 5 GLM 5.3, local endpoint | 19 of 20 (95 %) | 20 of 20 | 21 | 508 | 1.9 | good enough direct |

Quick runs, 5 trials each, against the full verdict:

| Case | A | R | Calls | Quick outcome | Full verdict | Agree |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 5 of 5 | 5 of 5 | 5 | good enough so far | good enough direct | yes |
| 2 | 4 of 5 | 4 of 5 | 7 | undecided | good enough direct | not contradicted |
| 3 | 5 of 5 | 5 of 5 | 5 | good enough so far | good enough direct | yes |
| 5 | 4 of 5 | 5 of 5 | 6 | good enough so far | good enough direct | yes |

- Does the set separate the models: no. One verdict for all. The largest difference in A is 17 against
  20 of 20, which is within what two samples of twenty from the same model can show.
- Not every first reply passes: 5 of the 80 first replies of the full runs were rejected (1, 3, 0, 1),
  and each was repaired. In Mistral Large's quick run one trial was rejected after two repairs; in its
  full run none was.
- Cost: the full run of Mistral Large took 1,216 s and 104,144 output tokens; GLM 5.3 on Mistral's API
  took 58 s and 9,008.
- No call was refused or waited out in any run. No trial was counted as not run.
- The runs printed "still waiting for the answer" notices after 2 s to 3 s on GLM 5.3, because the
  expected time follows the answers already seen in the run. They are correct by the rule and of no use
  at that length.

## Limits and interpretation

- The report does not say which task a rejected reply belonged to or why it was rejected, and `measure`
  keeps no record of its calls. So this run cannot say whether the five rejected first replies fall on
  one task. That is needed before the set can be hardened with purpose.
- One full run per case. The same model on two endpoints (cases 1 and 5) gave the same counts, which is
  the only repeat there is.
- The output tokens include what the endpoint counts as reasoning. Mistral Large's cost at reasoning
  level high is what is measured, not the model at another level.
- The four tasks are small rewrites of one function each. Pack authoring (R72, R99) was far harder than this set. A set that every capable model passes can still say "this
  endpoint and model can do the smallest step", which is what `check` already says with one task.

## Tool/spec consequence and next gate

Findings are above; what follows is proposal, for the owner.

- The indicator as built answers "is this model good enough for small steps", and for these four the
  answer is yes. It does not rank models. If ranking or a warning for weak models is wanted, the set needs
  harder tasks, and a weaker model to show that the verdict can come out otherwise; none of the four
  first cases is weak on this set.
- Two additions would make a hardening measurable, both in `measure`: the counts per task, and the
  reasons of rejected replies in the JSON report.
- Whether cost should enter the verdict or stay a reported number.
- Task 5.3 stays open and group 5 is not closed until the owner has decided.

## COBESY check

The negative answer is in the title. Counts are given with their totals; no rates are pooled across
models. The proposal is separate from the findings. What the report cannot show (per task, reasons) is
named as a limit and not guessed.
