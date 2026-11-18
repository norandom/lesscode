# R161: The released step flow accepts the batch runner's three locks as one value, after refusing a wrong declaration

- Report date/time and timezone: 2026-11-18, 19:25, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-18, 19:00 (preregistration) to 19:17 (guard), from the clock
- Status: completed; passed as registered. Two of four predictions wrong
- Spec: [state-consolidation](../../../.kiro/specs/state-consolidation/brief.md), stage 2 (consolidate); the step's own spec is [batch-progress](../../../.kiro/specs/batch-progress/requirements.md) on the branch `step/batch-progress`
- Hypothesis ID and predecessor: the two machines the finder named in this repository in [R158](24-finder-follows-names.md)
- Type: preregistered in the [prototype gate](../../../.kiro/specs/state-consolidation/prototype-gate.md), commit `6da5952`, before any code was written

## Decision

The owner asked for real, obvious cases under `docs/examples` and for the project's own tool to be
used on them. `queue`, `results` and `fatal`, three `Mutex` fields of the batch runner, are one
`Mutex<Progress>` on the branch `step/batch-progress`. `lesscode-core` 2.1.0 judged the step on its
own source: `step check` exit 0, `step guard` exit 0, recorded on the ledger.

| Registered | Result |
| --- | --- |
| `step check` exits 0 | held, on the second run. The first refused: the tests did not pass on the base, because the declaration built no example program |
| `step guard` exits 0 | held |
| Cells 3 to 1 | held |

| Prediction | Result |
| --- | --- |
| Accepted | held |
| Finder reports one machine after | **wrong**: it reports one shared value and no machine. The accesses go through a helper and the condition is on a plain field |
| `analyze` shows no fewer branch conditions | held: 13 and 13 |
| The scope grows, under 80 tokens | **wrong**: it shrank, 2,761 to 2,757 |

The declaration was changed once after the refusal: a `build` command for the examples was added.
No limit was raised and no test narrowed.

The way of working is written down as a guide, on the owner's word:
[Untangling scattered state](../../state-refactoring.md).

## Hypothesis and falsification

The three locked fields are one machine and can be one value, judged acceptable by the step flow
with a growth limit fixed beforehand. Falsified by a refusal that stands after the declaration is
correct.

## Frozen inputs and method

- Base `6da5952`, step `6b580a5` (change in `165555d`), branch `step/batch-progress`, not merged.
- `lesscode-core` 2.1.0, binary SHA-256 `2c9c23c846a84f0e…`, built from main before the change.
- Declaration: scope `runner.rs`; focus `one`, `worker`, `run_batch`; suites `driver_run` and
  `driver_interrupt`; `max_token_growth` 80; two criteria commands, the second the whole suite.
- Finder revision 3 and `deconvolute analyze` on the file before and after.

## Evidence and results

[Run directory](D:/BuildArtifacts/lesscode/state-consolidation/r161): both versions, the two check
logs, `verdict.json`, `analyze-*.json`, `finder-*.json`, the binary. The work directory of the step
(6.8 GiB) is removed.

| Function | Noticed before | Noticed after | Unnoticed before | Unnoticed after |
| --- | --- | --- | --- | --- |
| `one` | 1 | 1 | 5 | 5 |
| `worker` | 8 | 7 | 5 | 5 |
| `run_batch` | 0 | 0 | 3 | 3 |
| `next` (new) | | 1 | | 0 |

- `analyze`: branch conditions 13 and 13, state writes 16 and 16, gaps 3 to 1, call-argument sites
  181 to 170.
- Among the unnoticed faults is dropping `queue.push_front(job)` after a failed run: a transition of
  the machine that no test protects, before or after.

## What dogfooding found in the tool

1. A failing first run leaves no output in the receipt. "The unmutated tests do not pass in this
   copy" had to be reproduced by hand to learn why.
2. The Rust row of `docs/steps.md` showed a `--test` filter without a build command, which is the
   trap this step fell into. Corrected in the same change as this record.
3. `step record` answered "not recorded: the verdict is refused" while the accepted verdict was on
   disk and the refused one was committed. It reads the committed verdict. The skill's order (record,
   then commit verdict and ledger together) and the comment in `record.rs` ("the verdict on disk")
   do not say that. Not changed here.
4. `run_batch` has no noticed fault at all with these two suites.

Amendment, 2026-11-18, 19:55: findings 1 and 3 are fixed on main on the owner's word, after release
2.1.0. A failing first run now gives its reason as "the unmutated tests do not pass in this copy
(test step, exit …: end of the output)", and `step check` carries that into its refusal. `step record`
judges the verdict on disk, which is the one it writes down; the guard still judges what is
committed. Core suite 410 of 410 on Windows; Linux through the container pipeline at the same commit.

## Limits and interpretation

- No pin was written. The step is as protected as the base, which is the tool's bar, and the base
  leaves 13 faults unnoticed in these functions.
- Concurrency is not tested. One lock closes a gap that two left; no test observes it either way.
- The finder losing the machine after the change is its known blindness to plain fields.
- Windows only for the step runs.

## Tool/spec consequence and next gate

1. The step flow can carry a state consolidation from declaration to guard on real code. The find
   part stays by hand and by a research program.
2. Findings 1 and 3 are candidates for small fixes in the product; the owner decides.
3. Merging `step/batch-progress` is the owner's decision.

## COBESY check

Wrong predictions are in bold. The refusal is told before the acceptance. The count of unnoticed
faults is in the decision's neighbourhood, not hidden. Token counts appear only as the size check the
tool itself applies; the claim of the example is one shared value where there were three.
