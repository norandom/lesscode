# R99: Result. GLM 5.3 with repair turns reaches 19 and 20 of 20 admitted packs; the Mistral arms were stopped by the owner

- Report date/time and timezone: 2026-11-12, 15:51, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-12, 13:32 to 15:11 (GLM arms); Mistral attempts 13:32 to 15:50, none complete
- Status: complete for GLM 5.3; not run to the end for Mistral
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md),
  Slice B, reopened for cost by owner decision of 2026-11-12
- Hypothesis ID and predecessor: [R98](05-repair-gate-preregistration.md) with its two amendments;
  [R97](04-model-diagnosis-result.md); [R78](../2026-11-11/04-slice-b-node-kinds-result.md)
- Type: result of a preregistered run; one claim judged, five not judged

## Decision

Owner decision, 2026-11-12, after the GLM arms: GLM only. The affordable model for the model-driven loops
is `zai-glm-5-3` at reasoning level "high". The loop that goes with it: a first call, then at most two
fresh repair calls that show the rejected pack and the admission's reasons. With that loop GLM is admitted
as often as Opus was in R78 (20 of 20). The grammar's node kind names go into the request for this model:
that is a result of this run for GLM, and it overturns R78's "not adopted" for this setting only.

Separate outcomes: pack authoring only. No project's tests were run against these packs, and admitted is
not the same as equal to the shipped pack (below).

## Hypothesis and falsification

| ID | Claim | Measured | Verdict |
| --- | --- | --- | --- |
| Q5 | GLM 5.3 without the list ends with at least 16 of 20 admitted | 19 of 20 | holds |
| Q1 to Q4, Q6 | The claims about Mistral | no complete arm | not judged |

The Mistral arms did not finish:

- 13:32, `mistral-large-4`: stopped after five calls when the owner named `mistral-large-latest` as the
  correct model (first amendment of R98).
- 13:50, `mistral-large-latest`: void, the endpoint answered HTTP 400 to 109 of 114 calls (second amendment).
- 14:12, `mistral-large-latest`: 32 calls answered in 98 minutes, none failed, 392,808 output tokens; arm
  B1 had not ended and no trial was written when the owner decided for GLM. Stopped at 15:50. The ledger
  is in `results/b1-stopped-mistral-large-latest/`.

Nothing in this entry says how Mistral does with repair at "high". What was seen is its pace: about three
minutes and 12,000 output tokens per call on this endpoint, against 5,000 for GLM.

## Evidence and results

`docs/experiments/model-diagnosis/results/b3-glm-repair/` and `b4-glm-repair-kinds/`. Tool at `9a55007`,
"high", 64,000 tokens, at most two repairs, five trials per language.

| | Without the list (B3) | With the list (B4) |
| --- | --- | --- |
| First replies admitted | 7 of 20 | 13 of 20 |
| Admitted in the end | 19 of 20 | 20 of 20 |
| admitted at the first, second, third call | 7, 11, 1 | 13, 6, 1 |
| First replies naming a node kind the grammar lacks | 13 | 0 |
| First replies rejected for another reason | 0 | 7 (6 controls, 1 size) |
| Calls | 35, one failed | 28 |
| Input and output tokens | 90,371 and 170,900 | 87,240 and 129,246 |
| Trials that ended without a pack | 0 | 0 |
| Admitted packs with the shipped pack's mutants | 4 | 2 |

Admitted in the end, of five, with first replies in brackets:

| Arm | Rust | Python | TypeScript | Java |
| --- | --- | --- | --- | --- |
| B3, without the list | 4 (0) | 5 (4) | 5 (3) | 5 (0) |
| B4, with the list | 5 (0) | 5 (5) | 5 (4) | 5 (4) |

What the trials show:

1. The repair call does the work. Without the list every first failure was a wrong node kind name, and one
   repair call fixed 11 of 13. Java went from no first reply to five of five.
2. The list moves Java into the first reply (0 to 4) and saves a fifth of the calls and a quarter of the
   output tokens. It does not help Rust on the first reply; there the failures are control cases, as in
   R78, and repair fixes them.
3. The one trial that stayed rejected is a Rust pack in B3: still a wrong name (`let_pattern`) and a failed
   control after two repairs.
4. GLM's first replies agree with R97: 7 of 20 here, 8 of 20 there, same settings.
5. Few admitted packs equal the shipped pack: 4 of 19 and 2 of 20. Python and TypeScript never do; the
   differences reported are a missing `arg_swap` operator and one extra operator. R69 and R72 saw the same
   on Opus and Mistral for TypeScript.

## Limits and interpretation

- Twenty trials per arm, one afternoon, one endpoint in front of the vendors' APIs.
- No price for GLM is known here, so the cost claim that motivated this work is not shown. What is shown
  is tokens: about 300,000 for 20 admitted packs with the list. At Mistral's list price that would be
  about 0.33 USD; the Opus runs of R78 cost 1.41 USD. That is an illustration with another vendor's price,
  not a measurement.
- Opus's 20 of 20 is from another day, the Python runner and repair inside a conversation.
- The comparison between the two GLM arms had no preregistered claim. The difference in the end result is
  one trial; the differences in first replies (7 against 13) and calls are larger than repetition gave in
  R68 and R69 but rest on one run each.
- Admission is the barrier. Point 5 says the admitted packs are mostly not the shipped pack; whether they
  are worse, equal or better at finding untested behaviour was not measured.
- The decision for GLM was made on GLM's result and Mistral's pace, not on a comparison of the two.

## Tool/spec consequence and next gate

- Loop for a model-driven step with an affordable model: propose, judge with the tool, repair with the
  tool's reasons in a fresh call, at most twice. Facts the tool has (the grammar's names) go into the
  request. This is the shape to carry into the spec in which a model proposes a simplification and the
  step flow judges it.
- Endpoint file: model `zai-glm-5-3`, reasoning level "high". Output limit 64,000 tokens.
- The tool keeps: waiting out a rate limit, the endpoint's message in a failed call, repair turns and the
  list as options of the judge.
- Open: what admitted packs that differ from the shipped pack do to a gate's result (point 5). That is the
  next thing to measure before a model-written pack is used on a project.
- Open: the guide optimiser's GEPA and MIPROv2 loops have not been run on GLM.

## Amendment: cost at the owner's prices (2026-11-12, 15:55)

The owner gave the prices to calculate with for GLM 5.3: 1.19 EUR per million input tokens, 0.119 EUR per
million cached input tokens, 3.74 EUR per million output tokens. On the local endpoint the cost itself is
not relevant; the figures are for comparison.

| | Without the list (B3) | With the list (B4) |
| --- | --- | --- |
| Input, 90,371 and 87,240 tokens | 0.108 EUR | 0.104 EUR |
| Output, 170,900 and 129,246 tokens | 0.639 EUR | 0.483 EUR |
| Total for 20 trials | 0.747 EUR | 0.587 EUR |
| Per admitted pack (19 and 20) | 0.039 EUR | 0.029 EUR |

- The tool does not record cached input tokens, so all input is counted at the full price; with caching
  the totals are lower by at most the input share (about 0.10 EUR per arm).
- Opus in R78: 1.41 USD for 20 admitted packs, 0.07 USD per pack. The currencies differ and no rate is
  applied here; GLM with the list is at well under half of that figure at any rate near parity.
- The illustration under "Limits" (0.33 USD at Mistral's list price) is replaced by this table. "No price
  for GLM is known here" and "Not shown: cost" described the state before this amendment.
- Q6 was a claim about Mistral and stays not judged.

## COBESY check

The one claim that could be judged is judged against its threshold. The five that could not are listed as
not judged, with every attempt and why it ended. The comparison without a claim is marked as such, and the
cost is stated as not shown.
