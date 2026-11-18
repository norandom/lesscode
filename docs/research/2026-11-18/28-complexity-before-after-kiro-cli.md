# R162: Measured with kiro-cli, untangling state leaves the number of decisions as it was and moves them

- Report date/time and timezone: 2026-11-18, 20:45, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-18, 20:15 to 20:40, from the clock
- Status: completed; exploratory, not preregistered
- Spec: [state-consolidation](../../../.kiro/specs/state-consolidation/brief.md); brief only
- Hypothesis ID and predecessor: the owner's question after [R161](27-batch-progress-dogfood.md): is there a heuristic for when a target is untangled far enough, given that codebases differ
- Type: measurement of four finished examples with an existing tool. No bar was set beforehand, so nothing here passed or failed a registered test

## Decision

The owner's direction: the engineering metrics are declared with `openidea-kiro`, the owner's other
project, and complexity is accounted before and after, by its command-line tool.
`kiro-cli` 1.1.0 (released binary, checksum verified) measured the four examples before and after.

| Example | Routines | Cyclomatic, sum | Decisions (sum minus routines) | Cyclomatic, largest routine | Nesting, deepest |
| --- | --- | --- | --- | --- | --- |
| Order flags (made up) | 4 to 2 | 15 to 13 | 11 to 11 | 5 to 8 | 2 to 1 |
| Batch progress | 10 to 12 | 34 to 36 | 24 to 24 | 7 to 6 | 2 to 2 |
| Reply side channel | 26 to 26 | 56 to 57 | 30 to 31 | 7 to 9 | 2 to 2 |
| Clipboard lease | 11 to 15 | 25 to 27 | 14 to 12 | 8 to 8 | 3 to 3 |

Three things follow.

1. Classical complexity does not credit this kind of change. The sum of cyclomatic complexity rose by
   one or two in every real example, because each new method starts at 1.
2. The number of decisions stays where it was: 11 and 11, 24 and 24, 30 and 31, and 14 to 12 only
   where two of them were copies. This is the floor named after R161, now measured by a second tool
   with its own definition.
3. The decisions move into one routine, and that routine's number goes up: `with_repairs` 7 to 9, the
   opener's `apply` 8 where the largest was 5. Untangling state trades several small deciding
   routines for one larger one. A limit per routine is what keeps that trade honest.

So the two kinds of measure have different jobs. What a state step is meant to improve is counted per
machine (places that hold the state, routines that decide on it). The engineering metrics are the
guard: no routine above a limit may get worse.

The guard found something. `kiro-cli code check --changed v2.1.0` on this repository blocked the step
that our own `step check` had accepted: `run_batch` went from 60 to 61 code lines, with a limit of
60. `step check` counts tokens of the file, which fell. The routine is 56 lines now.

## Hypothesis and falsification

None registered. The question was what the engineering metrics say about four changes already made.

## Frozen inputs and method

- `kiro-cli` 1.1.0 for Windows from the release page of `norandom/openidea-kiro`, SHA-256 of the
  archive `7edf626d…713771d2bc`, equal to the published sum. The repository itself was only read.
- Built-in limits, configuration digest `1d6f83eeec57440e`: per routine cyclomatic 10, nesting 3,
  code lines 60, statements 40, parameters 5.
- Each side of each example in a scratch project; `kiro-cli code baseline` gives every routine's
  values; the after side judged against the before baseline with `kiro-cli code check --all`.
- The batch runner's row is the step as accepted in R161 (commit `165555d`).
- On this repository: `kiro-cli code check --changed v2.1.0`.

## Evidence and results

[Run directory](D:/BuildArtifacts/lesscode/state-consolidation/r162): per example `before/` and
`after/` with their baselines, `*-check.txt`, `lesscode-changed-since-v2.1.0.txt`,
`lesscode-changed-after-fix.txt`.

- Batch progress: `worker` 7 to 6, new `next` 2, new `progress` 1.
- Reply side channel: `Authoring::one` 7 to 6 and 89 to 82 code lines; `with_repairs` 7 to 9, 36 to
  44 code lines. Three ratchet warnings on `with_repairs`, all within the limits.
- Clipboard lease: `reserve`, `claim`, `release` 2, 3, 2 to 1, 1, 1; the four methods of `Owner` 1,
  2, 1, 2; `guard` 3 and 3.
- Ratchet on this repository since 2.1.0, after the fix: 8 files, no blocking finding, six
  pre-existing breaches, twelve warnings within limits.

## Limits and interpretation

- Four examples, one of them made up, all Rust, all small.
- "Decisions" is my subtraction (sum minus routines), not a metric the tool prints.
- The scattered-state rule of `kiro-cli` looks for one subject compared with literal values in more
  than three files. None of the four examples has that shape, and the rule reported nothing.
- No baseline of this repository is committed. Capturing one hides every regression before it and is
  the owner's decision.

## Tool/spec consequence and next gate

1. `.kiro/code-rules.toml` declares the adopted limits for this repository: the built-in ones, with
   `docs/examples/` not judged.
2. The guide gets one more step: after `step check`, run `kiro-cli code check --changed <base>`.
3. A stopping rule to test, not yet tested: a target is far enough when every machine found has one
   place and one deciding routine, the decisions have not fallen below the rows of its transition
   table, and the ratchet passes. Registering that test is the next experiment if the owner wants it.
4. `step check` and the ratchet disagreed on one step. Whether `step check` should call the ratchet,
   or the two stay separate commands, is the owner's decision.

## COBESY check

The table shows the numbers that do not flatter the change: sums up, largest routine up twice. The
status says "not preregistered". The blocked step is in the decision. Lines of code appear only as
the limit the other tool applies, not as a measure of success.
