# R155: The cell is gone from the repository, and the rule against replies that are not whole is in the loop for every caller

- Report date/time and timezone: 2026-11-18, 15:25, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-18, 15:05 to 15:20 (from the clock)
- Status: completed
- Spec: [state-consolidation](../../../.kiro/specs/state-consolidation/brief.md), stage 2 adopted on one real case; brief only, no requirements yet
- Hypothesis ID and predecessor: the two owner decisions that [R154](20-reply-machine-one-place.md) asked for
- Type: confirmatory; the expected differences were stated in the working session before the run, not in a file; no model call

## Decision

The owner said to proceed with both steps. Both are in the repository, each as its own commit.

1. `48010b5`: the check of `with_repairs` receives the whole reply. The guide adapter's cell, its write
   and its read are removed. This is R154's patch, unchanged.
2. `3db7caf`: `with_repairs` itself rejects a reply that was cut off or refused, without showing it to
   the check. The adapter's own match on the stop reason is removed. The rule is in one place.

Step 2 changes behaviour, as intended, and was compared before and after on all 125 histories:

| Pair, before and after step 2 | Solver | Modelled results that differ | Real records that differ | Verdict of the command |
| --- | --- | ---: | ---: | --- |
| Loop with a text-only check | sat | 62 of 125 | 62 of 125 | `different`; witness "refused, failed, rejected" reproduced |
| Guide adapter | unsat | 0 of 125 | 31 of 125 | `inconsistent` |

Both rows are what was expected. The 62 are the histories in which a text-only check used to accept a
reply that was not whole; after the step it accepts none. The 31 adapter records are the histories in
which a cut-off reply is reached, and they differ in one sentence only: "the reply was cut off before
the pack was complete" now reads "the reply was cut off before it was complete". With that sentence
replaced, 0 of 125 records differ.

## Hypothesis and falsification

Stated before the run: the text-only pair differs on exactly the histories where a reply that is not
whole would have passed; the adapter pair has the same endings and call counts and differs only in the
reworded sentence. Rejected by a text-only history that still accepts such a reply, by an adapter
record that differs in anything else, or by a failing project test other than the three that state the
old behaviour.

## Frozen inputs and method

- Before: `48010b5` with the [harness](../../experiments/state-consolidation/reply-machine/harness.rs).
  After: the same with step 2. Two test binaries, pinned by SHA-256 in the target file.
- [Target generator](../../experiments/state-consolidation/reply-machine/make-target-whole.ps1). The
  loop's model is now in one file shared with R154's generator; the regenerated R154 target is
  identical in bytes to the one R154 ran.
- `lesscode-core deconvolute bound-check` of R152, Z3 5.3.3. The sentence check is a separate script
  step over the two arms' kept outputs.

## Evidence and results

- `run/result.json`: the table; exit 2 because of the adapter pair.
- `adapter-wording.json`: 31 differing records, the same 31 histories that reach a cut-off reply, 0
  differing once the sentence is replaced. The old sentence occurs 43 times before and never after.
- Three tests stated the old behaviour and were changed: the check no longer sees a refusal
  (`a_refused_reply_ends_the_loop_as_rejected_without_another_call`), a cut-off attempt carries the
  loop's reason (`every_attempt_is_recorded_with_its_verdict_its_stop_and_its_tokens`), and the adapter
  test expects the new sentence. One test was added:
  `a_reply_that_is_not_whole_is_never_accepted_and_never_judged`.
- Windows after step 2: `lesscode-model` 85 of 85 and doctests, the guide tool 3 of 3, clippy clean on
  both. After step 1 alone: 84 of 84 and 3 of 3.
- Linux: not established at `3db7caf`. The Dagger run was killed with exit 137 while the core suite ran (203 of 408 passed, none failed; the core is unchanged since `c1c76e8`, which passed). A second call replayed the kept failure in 20 seconds. The cause was not found; the Podman machine is shared and was restarted 42 minutes earlier. A run at the commit of this entry is recorded in the amendment below.

[Run directory](D:/BuildArtifacts/lesscode/state-consolidation/r155)

## Limits and interpretation

- `inconsistent` is the command's word for "the records differ where the model says nothing changes".
  Here the cause is known and counted. The command cannot say that itself.
- What `lesscode-model check` and `measure` now report for a cut-off or refused reply was not run
  against an endpoint. Their own judges were not in the comparison; the stand-in check was.
- A reply with another stop reason than complete, cut off or refused is still judged, as before.
- Kept records of earlier guide trials carry the old sentence. Nothing reads it back.
- The machine was found by people. No difference within these histories is not a proof.

## Tool/spec consequence and next gate

One hidden machine of this repository is consolidated and adopted, with a bounded comparison and the
project's tests behind each step. For the next release notes: `lesscode-model` no longer accepts a
reply that was cut off or refused, whatever its text, and the check passed to `with_repairs` takes the
reply. The open gate is unchanged: a find step that returns such a machine from the files alone.

## COBESY check

Both rows of the table were predicted, including the one the command calls inconsistent, and the
prediction is checked by count. The behaviour change is named as one. What was not run is listed.
