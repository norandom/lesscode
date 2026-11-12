# R97: Result. The reasoning level helps the second model but does not close the gap; what remains is node kinds the model has to recall

- Report date/time and timezone: 2026-11-12, 13:20, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-12, 10:24 to 13:06
- Status: complete
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md),
  Slice B, reopened for cost by owner decision of 2026-11-12
- Hypothesis ID and predecessor: [R96](03-model-diagnosis-preregistration.md) (preregistration),
  [R72](../2026-11-10/18-slice-b-baseline-mistral.md), [R69](../2026-11-10/15-slice-b-baseline-rerun-stronger-admission.md)
- Type: result of a preregistered run, with one amendment made during the run (below)

## Decision

The authoring loop must stop asking a model to recall the node kind names of a grammar. The request gets
the grammar's list of node kinds for the target language, and a repair turn names the invalid kinds. The
reasoning level "high" becomes the setting for Mistral. No model is chosen yet: the same run has to be
repeated with the kind list in the request before Mistral, GLM and Opus can be compared on what is left.

Amendment, 2026-11-12, 13:50: this decision was written without reading
[R78](../2026-11-11/04-slice-b-node-kinds-result.md), which had already tested the node kind list on this
model and did not adopt it. The decision is withdrawn in the form above; see "Amendment after reading R78"
at the end. The measurements and verdicts of this entry are not affected.

## Hypothesis and falsification

| ID | Claim | Threshold | Measured | Verdict |
| --- | --- | --- | --- | --- |
| H1 | Replication of R72 at the default level and 16,000 tokens | 3 to 7 admitted of 20 | 3 of 20; 6 replies cut off (R72: 5 admitted, 6 empty) | holds, at the lower edge |
| H2 | The empty replies were the output limit | at most 1 of 20 empty or cut off at 64,000 | 0 of 20 | holds |
| H3 | Reasoning level "high" lifts Mistral | at least 10 of 20 | 8 of 20 | refuted |
| H4 | What remains is knowledge of the grammar | at least half of the rejections at "high" name a node kind the grammar lacks | 12 of 12 | holds |
| H5 | GLM 5.3 at "high" | at least 10 of 20 | 8 of 20 | refuted |

## Amendment during the run

The endpoint refused calls with HTTP 429 (too many requests): 6 trials in A1, 7 in A2, 4 in A3, none in
A4. The tool retried a failure once after ten seconds and then recorded the trial as rejected. That is a
fault of the apparatus, not an answer of the model.

- Fix: commit `a913de7`. A 429 is asked again after 10, 30, 60, 120 and 240 seconds; any other failure is
  still retried once.
- The 17 refused trials were run again, same language, same arm settings, with `--judge-list`. All 17 were
  answered. Results are in the `-rerun` folders beside each arm.
- The numbers above and below count, per arm, the 20 trials the model answered: the answered trials of the
  first run plus the reruns. Five per language in every arm.
- This was decided after seeing the refusals and before seeing any rerun reply. Without the amendment the
  counts are 2, 2, 7 and 8 admitted of 20; no verdict changes except H1 (2 would be below the range).

## Evidence and results

`docs/experiments/model-diagnosis/results/<arm>/` and `<arm>-rerun/`: `trials.jsonl` (one line per trial
with the reply), `summary.json`, `ledger.jsonl`.

| Arm | Model, level, limit | Admitted | Cut off | Rejected, complete | of these: unknown node kinds | Output tokens |
| --- | --- | --- | --- | --- | --- | --- |
| A1 | Mistral Large 4, default, 16,000 | 3 | 6 | 11 | 10 | 219,721 |
| A2 | Mistral Large 4, default, 64,000 | 2 | 0 | 18 | 17 | 281,052 |
| A3 | Mistral Large 4, high, 64,000 | 8 | 0 | 12 | 12 | 227,308 |
| A4 | GLM 5.3, high, 64,000 | 8 | 0 | 12 | 10 | 69,517 |

The other rejections: packs over the size limit (one each in A1 and A4) and failed controls with valid
kinds (one each in A2 and A4).

Admitted first replies by language, of five each. Opus is R69, not run here:

| Arm | Rust | Python | TypeScript | Java |
| --- | --- | --- | --- | --- |
| A1 | 0 | 1 | 2 | 0 |
| A2 | 0 | 0 | 2 | 0 |
| A3 | 0 | 3 | 5 | 0 |
| A4 | 0 | 5 | 3 | 0 |
| Opus (R69) | 0 | 5 | 5 | 5 |

What the replies show:

1. Raising the output limit removed every cut-off reply and admitted nothing more (A1 against A2). The
   replies that were cut off before now complete and fail on node kinds.
2. The reasoning level is the only setting that moved admission: 2 to 8 of 20 (A2 against A3), all of it
   in Python and TypeScript.
3. No arm has an admitted Rust or Java pack. Rust fails for Opus too (R69, the stronger controls). Java is
   where the cheaper models differ from Opus: 9 of 10 Java replies of A3 and A4 name `text_block`, which
   the grammar does not have. In Rust the recurring names are `method_call_expression`,
   `compound_assignment_expression` and `let_expression`; in Python `binary_expression` and
   `comparison_expression`.
4. The guide demands exact node type names (line 19) and gives no list. The names the models write are
   plausible names from other grammars or from the language's own vocabulary. The model is guessing a
   fact that the tool has.
5. Mistral at the default level already reasons: about 14,000 output tokens per trial for a pack of about
   2,000 characters, against about 11,400 at "high". "High" does not cost more output here. GLM needs
   about 3,500 per trial, a third of Mistral at "high", and lost no call to the rate limit.

## Limits and interpretation

- Twenty trials per arm; a difference of 8 against 10 is within what five trials per language can move. The
  verdicts on H3 and H5 are against the stated thresholds, not a claim that the models are equal or unequal.
- One day, one endpoint in front of the vendors' APIs. Whether the endpoint passes the reasoning level on
  unchanged was not checked beyond the reply's token report before the run.
- No repair turn. Opus in R69 repaired two of five Rust packs; nothing here says how the cheaper models
  repair.
- H1 compares the Rust judge with the Python prototype of R72. 3 against 5 admitted and 6 against 6 empty
  replies is agreement at this sample size, not proof that the two judges are the same.
- No cost is stated: no list price for GLM is known here, and the endpoint's price for Mistral was not
  recorded. Tokens are in the table.
- The rerun trials ran later in the day than the first run of their arm.

## Tool/spec consequence and next gate

What the DSPy-style loops change for an affordable model:

1. Facts go into the request, not into the model's memory. The authoring request carries the node kind
   names of the target grammar, taken from the grammar the tool ships. This is generic: the same code for
   every language pack, no per-language text.
2. The admission's feedback is already precise ("node kinds not in the java grammar: text_block"). A repair
   turn that returns it, with the valid names near the invalid one, is the cheap second step.
3. The optimiser (GEPA, MIPROv2 style) should not spend calls on teaching a guide to convey kind names.
   Guide text is for judgement the tool cannot check; names are data.
4. Mistral runs at "high"; the output limit is 64,000 tokens. The tool waits out a rate limit.

Next gate, to be preregistered before it runs: the same four languages and twenty trials with the kind
list in the request, for Mistral at "high" and GLM at "high". Threshold to state then: Java reaches the
Opus level, and the remaining rejections are controls, not names. Rust stays a separate question, since
Opus fails it as well.

## Amendment after reading R78 (2026-11-12, 13:50)

The consequence above proposes the node kind list as if it were new. It is not. R78 (2026-11-11) gave
Mistral Large 4 the list: first replies naming a kind the grammar lacks fell from 10 to 1 of 20, first
replies admitted stayed at 5 and 6 of 20, and the list was not adopted. With the names right, the replies
failed control cases, and more of them ended at the output limit of 16,000 tokens. The same entry shows
what did work without the list: the repair turn with precise feedback, 15 of 20 admitted in the end.
The preregistration R96 and this entry should have started from R78; neither cites it.

What this entry adds to R78, and what it does not:

- New: the reasoning level moves first replies from 2 to 8 of 20, and 64,000 tokens remove every cut-off
  reply. In R78 the runs that ended empty were the ones repair could not help (5 of 20 without the list,
  11 with it), so the limit matters for the repair path even though it admits no first reply by itself.
- Confirmed, not new: the commonest first failure is a wrong node kind name (R72, R78).
- Open in R78 and still open: whether the list helps once the limit no longer binds. R78 named that as a
  separate question. Its own result says to expect failed controls behind the names, so point 1 of the
  consequence is a hypothesis with evidence against it, not a decision.
- Withdrawn: "Java reaches the Opus level" as the threshold to state for the next gate. In R78 the list
  moved Java from 1 to 2 of 5.
- Point 3 (no optimiser calls for names) agrees with R74 and stands. Point 4 stands.

Next gate, replacing the one above: the measure is packs admitted in the end and what they cost, not first
replies. Mistral and GLM at "high" and 64,000 tokens with at most two repair turns, with and without the
list. The Rust judge has no repair turn yet; R72 and R78 used the Python runner, which is not to be
extended. To be preregistered before it runs.

## COBESY check

Thresholds were fixed before the first call and are judged as written, including the two that failed. The
amendment is dated, its reason and its effect on the counts are stated, and the numbers without it are
given. What was not measured (cost, repair, the endpoint's handling of the level) is named.
