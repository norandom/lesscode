# R100: What GLM's admitted packs lack of the shipped pack's findings: almost only the call-argument kind, which the guide does not describe

- Report date/time and timezone: 2026-11-12, 16:03, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-12, 15:58 to 16:03; no model call, no test run
- Status: complete
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md),
  Slice B, reopened for cost by owner decision of 2026-11-12
- Hypothesis ID and predecessor: open point of [R99](06-repair-gate-result.md) (only 4 and 2 of the admitted
  packs give the shipped pack's mutants); [R81](../2026-11-11/08-driver-three-changes-and-call-arguments.md)
  (the call-argument kind and its receipts)
- Type: analysis of stored data. The three expectations were written into commit `9bd5cc9` before the join
  ran on these packs; the operators named in the judge's feedback had been read before that, the classes
  had not

## Decision

An admitted pack is not yet a usable pack. The 39 packs GLM got admitted in R99 lose findings in one place:
the call-argument kind. The guide (version 3, 2026-11-10) is older than that kind (R81, 2026-11-11) and
does not mention it; the model sees it only in the example packs and writes it in part or not at all.
The next step is a guide that describes the kind, judged again by this measure. Until then a model-written
Python or TypeScript pack must not replace a shipped pack.

## Hypothesis and falsification

A finding is a mutant of the shipped pack that the project's tests did not notice ("survived" in the
driver's receipts of R81). A pack that does not produce that mutant no longer reports the finding.

| ID | Expectation | Measured | Verdict |
| --- | --- | --- | --- |
| E1 | At least half of the admitted packs lack at least one finding | 18 of 39 | refuted, by two packs |
| E2 | At least 80% of the findings lacked are call-argument sites | 193 of 198 | holds |
| E3 | No Java pack lacks a finding | 0 of 10 | holds |

## Frozen inputs and method

- Tool: `tools/guide-optimizer --gate-effect` at `9bd5cc9`. For each admitted pack it computes the pack's
  mutants on the gate scopes and joins them, by site, operator and hash of the changed text, with the class
  recorded for the shipped pack's mutants.
- Packs: the admitted final replies of R99, arms B3 (19) and B4 (20).
- Classes: `docs/experiments/annotation-driver/results/native-args/*-receipt.json` (R81), five gate scopes.
  Every mutant of the shipped packs was found in the receipts.
- Output: `docs/experiments/model-diagnosis/results/gate-effect/gate-effect.jsonl`.

## Evidence and results

Findings of the shipped pack on the gate scopes, and how many of them an admitted pack lacks:

| Language | Findings (of these call-argument) | Packs | Packs lacking any | Lacked per such pack |
| --- | --- | --- | --- | --- |
| Java | 38 (3) | 10 | 0 | none |
| Python | 27 (17) | 10 | 10 | 16 to 18 |
| TypeScript | 42 (8) | 10 | 7 | 4 or 8 |
| Rust | 21 (0) | 9 | 1 | 2 |

- Python: every pack loses 16 or 17 of the 17 call-argument findings, which is about 60% of all findings
  on that scope. Three packs also lose one state-write finding each.
- TypeScript: the three packs that lose nothing wrote the call-argument operators as the shipped pack has
  them; the others have no operator for swapping two arguments.
- Rust: one pack loses two error-order findings. The Rust packs differ from the shipped pack mostly by
  extra mutants, 4 to 34 per pack.
- Java: three packs of B4 lack four killed mutants (a negation and `<` against `<=`), no finding.
- Beside findings, the packs lack killed mutants (up to 31 per pack, again mostly call arguments). Those
  change a gate's kill rate, not what it reports as unprotected.
- The two arms do not differ in this: the node kind list does not bring the call-argument operators.

## Limits and interpretation

- Extra mutants are counted, not classed. Whether the 4 to 34 extra mutants of a Rust pack are findings,
  duplicates or changes that do not build is not known without running them against the tests.
- "Finding" rests on one recorded run per scope (R81); a time limit under load can change a class (R81
  reports one such case).
- Five gate scopes, one per language except TypeScript with two. What a pack lacks on another project is
  not measured.
- E1 was a guess about a number and failed narrowly; it says nothing more than the table does.

## Tool/spec consequence and next gate

- "Same mutants as the shipped pack" stays the strict measure. The join gives the graded one: findings kept.
  It belongs in the judge's verdict for every future pack-authoring run.
- Admission cannot see a missing optional kind. Either the controls get a case for call arguments in every
  language, or the kind stops being optional for a pack that is meant to replace a shipped one.
- Next gate, to preregister: a guide version that describes the call-argument kind in words, as the other
  kinds are described; GLM 5.3 at "high" with repair and the node kind list; claim on findings kept for
  Python and TypeScript.

## COBESY check

Expectations fixed in a commit before the join ran, one of them refuted and reported as such. The measure
is defined before the numbers. What the join cannot say (extra mutants) is stated.
