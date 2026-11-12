# R104: Result. The guide that describes the call-argument kind restores Python's findings and not TypeScript's; pack authoring is closed here

- Report date/time and timezone: 2026-11-12, 17:52, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-12, 17:00 to 17:49
- Status: complete; closing note for pack authoring
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md),
  Slice B
- Hypothesis ID and predecessor: [R101](08-guide-v4-preregistration.md), [R100](07-admitted-packs-gate-effect.md)
- Type: result of a preregistered run

## Decision

Guide version 4 is kept as the current guide: it closes most of the gap R100 found. No further work on
pack authoring follows. Owner statement of 2026-11-12, made while this run was under way: the language
packs are published once with the tool and are not regenerated with a cheaper model; what is needed is
that adopters can run another model behind the tool. That work is the spec `model-seam`.

## Hypothesis and falsification

| ID | Claim | Measured | Verdict |
| --- | --- | --- | --- |
| G1 | At least 4 of 5 Python packs keep at least 15 of the 17 call-argument findings | 4 of 5 keep all 17; one keeps 1 | holds |
| G2 | At least 4 of 5 TypeScript packs lack no finding | 0 of 5; each lacks 4 of 42 | refuted |
| G3 | At least 18 of 20 admitted in the end | 19 | holds |
| G4 | No Java or Rust pack lacks more than 2 findings | none lacks any | holds |

## Evidence and results

`docs/experiments/model-diagnosis/results/c1-glm-guide-v4/` and its `gate-effect/` folder. GLM 5.3 at
"high" on the local endpoint, repair turns, node kind list, 29 calls, one failed.

| Language | First reply admitted | Admitted in the end | Same mutants as shipped | Findings lacked per pack |
| --- | --- | --- | --- | --- |
| Java | 2 | 5 | 4 | 0 |
| Python | 5 | 5 | 3 | 0 in four packs, 16 in one |
| TypeScript | 5 | 5 | 0 | 4 in every pack |
| Rust | 0 | 4 | 0 | 0 |

- Against version 3 (R100, arm B4): Python packs lacking findings fall from 5 of 5 to 1 of 5. Packs with
  the shipped pack's mutants rise from 2 of 20 to 7 of 19.
- TypeScript: all five packs leave out the swap of the first two arguments. The new guide row says to
  leave it out "when a compiler checks the types of arguments". TypeScript has such a compiler; the shipped
  pack swaps all the same, because on that scope the tests run without a type check. The row is wrong for
  that case, and five identical packs show the model followed it.
- The run took 47 minutes for 29 calls: the local endpoint was rate limited for this model most of the
  time (see R102).

## Limits and interpretation

- One arm of twenty trials against an arm from earlier the same day.
- G2 failed for a sentence in the guide, not for the model. Whether a reworded row fixes it was not run.
- Extra mutants are counted, not classed.

## Tool/spec consequence and next gate

- If pack authoring is taken up again for a fifth language: reword the row on swapping arguments so that
  it depends on whether the project's test command type-checks, and add a control case for call arguments
  to every language, so that admission sees a missing kind (R100).
- The measure "findings kept" (`--gate-effect`) stays in the tool.
- No next gate on this line.

## COBESY check

Four claims judged as written; the one that failed is traced to its cause in the guide's text and left
unrepaired, with the reason.
