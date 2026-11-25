# R191: `begin`'s time figure, counted as a run spends its time (task 7.2)

- Report date/time and timezone: 2026-11-25, written at about 13:00 Europe/Berlin, before the code
  of task 7.2. The section "After the code" is added once `begin` prints the new figures.
- Execution interval: the measured runs are those of R188 (2026-11-24, 16:30 to 17:17), R189
  (2026-11-24, 20:07 to 21:05, and 2026-11-25, 08:50 to 09:02) and R190 (2026-11-25, 10:54 to
  11:53). Nothing was run again for this entry.
- Status: running
- Spec: [brownfield-test-stage](../../../.kiro/specs/brownfield-test-stage/tasks.md), task 7.2, from
  the amendment of the afternoon of 2026-11-25 in the
  [design](../../../.kiro/specs/brownfield-test-stage/design.md)
- Hypothesis ID and predecessor: [R190](01-protect-stage-python.md), whose expectation E3 failed:
  the base run took 1.73 times `begin`'s figure and the check 2.7 times
- Type: retrospective (the model was found on R190's numbers and then checked on R188 and R189)

## Decision

`begin`'s figure was low because it counted one run of the tests without a fault, where a run
makes three, and because it let the first fault run beside the others, where it runs alone. It also
gave the check the same figure as the base run, where the check runs the base and the step. R190
read the gap as the container's start and copy. That reading is revised here: the container's start
is part of every run of the tests, and the tests' seconds already hold it.

Counted as a run spends its time, the three stages' base runs took 0.91 to 0.99 of the figure and
their checks 0.45 to 1.05. The room of 1.4 covers all of them. This permits task 7.2 to print the
new figure, one for the base run and one for the check.

## Hypothesis and falsification

A run of the faults (`runner.rs`, `first_run.rs`) does this in each copy: it runs the tests without
a fault, then faults until none is left, then the tests without a fault once more. The first run of
each kind goes alone and is measured; the others start only when it is done. So with `c` copies,
`T` the plan's run of the tests without a fault, `S` the plan's seconds of the faults to run again
and `n` their number:

- the first copy's run of the tests goes alone (`T`), the other copies' runs follow (`T`), the
  first fault goes alone (`S/n`), the other faults are shared out (`(S - S/n)/c`), and the tests
  run once more at the end (`T`): the base run takes about `3T + S/n + (S - S/n)/c`;
- with one copy nothing waits for a turn: `2T + S`;
- the check runs the base and then the step, in work directories beside the base run's. On the
  base it takes the faults' results the base run stored and runs the tests only (`3T`). On the step
  it runs the faults again where the tests changed. It takes about the base run's figure plus `3T`.

The model is rejected if a base run of the three stages took more than 1.4 times its new figure,
or less than 0.8 times (then it counts something twice), or if a check took more than 1.4 times
its figure.

## Frozen inputs and method

- `S`, `n` and `T` as `begin` printed them for each stage (`begin.log` of R188 and R190, the
  `sf-begin.log` of R189), rounded to the second. `T` is the longer of the plan's two copies.
  `c` is 2 in all three.
- The measured times are those the research entries give, from the `.times` files under
  `D:\BuildArtifacts\lesscode\brownfield-test-stage\`: `yuv\`, `stage-6\` and `stage-5.2\`.
- That the check's base side takes the base run's results: the receipts `base-run.json` of the
  three stages' work directories list every fault as reused (28 of 28, 30 of 30, 4 of 4).

## Evidence and results

| Stage | `S` | `n` | `T` | old figure | new, base run | new, check |
|---|---|---|---|---|---|---|
| R188 `yuv.rs` | 1,096 s | 28 | 183 s | 731 s | 1,117 s | 1,666 s |
| R189 `shared_frame.rs` | 1,138 s | 30 | 158 s | 727 s | 1,062 s | 1,536 s |
| R190 `cycles.py` | 717 s | 4 | 140 s | 499 s | 868 s | 1,288 s |

| Run | took | of the old figure | of the new figure |
|---|---|---|---|
| R188 base run | 1,026 s | 1.40 | 0.92 |
| R188 check | 968 s | 1.32 | 0.58 |
| R189 base run | 962 s | 1.32 | 0.91 |
| R189 first check | 1,009 s | 1.39 | 0.66 |
| R189 second check | 687 s | 0.95 | 0.45 |
| R190 base run | 863 s | 1.73 | 0.99 |
| R190 check | 1,348 s | 2.70 | 1.05 |

No base run is under 0.8 or over 1.4 of the new figure, and no check is over 1.4: the model is
not rejected.

## Limits and interpretation

- The model was found on R190's numbers. R188 and R189 test it, but they are two runs of one
  Rust project on one machine.
- The checks of the Rust project took well under their figure: 0.45 to 0.66. On the base side
  cargo found most of the build done: in both Rust stages one copy's run of the tests without a fault
  took 11 to 15 s there, against about 155 s in the plan. In R189's second check the step's tests were unchanged, so its faults were stored too.
  The check's figure is an upper bound for such runs, not an estimate.
- The order in which copies take their turns depends on timing, so `3T` is the most a run spends on
  the tests alone, not a fixed sum.
- A machine that is busy with other work makes every run longer. The room of 1.4 stays for that,
  although no run of the three needed more than 1.05.

## Tool/spec consequence and next gate

Task 7.2 prints both figures with the new count. Its test checks the wording; the frozen plans of
the three stages check the numbers. The next stage run checks the figure again.

## After the code

(Added when task 7.2 is done.)

## COBESY check

The outcome stands first, and the table gives every number needed to replay it. The rejection
bounds were set before task 7.2 prints a figure, but after the model was fitted to R190, so this
entry is retrospective and says so. R190's earlier reading of the gap is kept there and revised
here.
