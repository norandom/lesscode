# R154: The reply-checking machine without its hidden cell behaves the same on all 125 reply histories

- Report date/time and timezone: 2026-11-18, 14:55, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-18, 14:46 to 14:52 (from the clock): three builds, the tests of the changed version, one bounded comparison
- Status: completed
- Spec: [state-consolidation](../../../.kiro/specs/state-consolidation/brief.md), stage 2 (consolidate and check) on a real case; brief only, no requirements yet
- Hypothesis ID and predecessor: the experiment proposed in [R153](19-reply-machine-tool-findings.md); uses the command of [R152](18-bound-check-in-core.md)
- Type: preregistered in R153 (parts b and c); no model call. Part a, a find step, was not built

## Decision

A version of the guide adapter and the repair loop that hands the check the whole reply, and so needs
no cell, was written by hand in a copy of the tree. It is checked three ways and holds in all three.
The change is kept as a patch and is not applied to the repository; adopting it is the owner's decision.

| Pair, on all histories of three replies | Solver | Modelled results that differ | Real records that differ | Verdict |
| --- | --- | ---: | ---: | --- |
| Adapter as it is, adapter without the cell | unsat | 0 of 125 | 0 of 125 | `same` |
| Adapter as it is, changed adapter with its refusal arm removed | sat | 39 of 125 | 39 of 125 | `different`; witness "refused first" reproduced |
| Loop with a text-only check, before and after the change | unsat | 0 of 125 | 0 of 125 | `same` |

The project's tests pass on the changed version: `lesscode-model` 84 of 84 and its doctests, the guide
tool 3 of 3, clippy clean on the model crate.

A second finding came out of the same runs. The rule "a reply that was cut off or refused is never
accepted" exists in the adapter only. The two callers inside `lesscode-model` (`check.rs:71`,
`measure.rs:165`) hand the loop a check that sees the text alone. With such a check the real loop
accepts a cut-off or refused reply in 62 of the 125 histories, 31 of them on a cut-off reply. The
change does not alter that; it makes the stop reason available to those checks.

## Hypothesis and falsification

The one-place version is right if no history of replies gives another record than today's code.
Rejected by one differing record among the 125, by a failing project test, or by a modelled result
that disagrees with a real run. The comparison is trusted only if it also finds a fault planted on
purpose, with a witness that reproduces.

## Frozen inputs and method

- Tree `45aff76` in a worktree under the run directory, with its own Cargo target directory.
- A history is three reply codes: complete and admissible, complete and rejected, cut off, refused,
  failed call. Two repairs are allowed, so at most three calls are made. 5 x 5 x 5 = 125 histories.
- [Harness](../../experiments/state-consolidation/reply-machine/harness.rs): inserted in the test
  module of `adapter.rs`. It runs the real `Authoring::one` with a scripted model and returns the
  whole trial record, and runs the real `with_repairs` with a check that looks at the text only.
- Three test binaries from that tree: as it is, with the
  [one-place change](../../experiments/state-consolidation/reply-machine/one-place.patch), and with
  that change minus the arm for a refused reply. SHA-256 of each is pinned in the target file.
- [Target file generator](../../experiments/state-consolidation/reply-machine/make-target.ps1): the
  modelled result of an arm is written by hand from `repair.rs` lines 113 to 136 and the check's rule:
  ten times the number of the last call, plus 0 accepted, 1 rejected, 2 failed.
- `lesscode-core deconvolute bound-check` of R152 (binary `d8d51d80…`), Z3 5.3.3.

## Evidence and results

- `run/result.json`: the table above, exit 1 because of the planted pair. The witness is the history
  refused, failed, failed: today's adapter ends rejected after one call, the planted one admits.
- `model-against-runs.json`: for each of the five arms the modelled result equals what the real record
  shows (number of calls and ending) on all 125 histories; 625 of 625.
- The same file has the count for the second finding: 62 histories accepted on a reply that was not
  whole, by the text-only check; the adapter admits no such reply.
- Test logs of the changed version: `one-place-model-nextest.log`, `one-place-model-doctest.log`,
  `one-place-guide-nextest.log`, `one-place-model-clippy.log`.
- The patch changes five files: the check's parameter in `repair.rs` and its three callers, and in
  `adapter.rs` the cell, its write and its read go; the check matches on the reply's stop reason.

[Run directory](D:/BuildArtifacts/lesscode/state-consolidation/r154)

## Limits and interpretation

- The machine was found by people (R130, R153). No tool found it. Part a of the proposal is open.
- The modelled results were written by hand from the source, the same text for the first two arms.
  The solver's `unsat` there says little by itself; the weight is on the 125 real records per arm and
  on the model matching every one of them.
- Three replies, two repairs, five kinds of reply. A cut-off reply always carried the admissible pack;
  a cut-off reply with other text was not run.
- The text-only check of the harness stands in for `check.rs` and `measure.rs`; their own judges were
  not run. Whether a cut-off reply can pass them depends on the task's text.
- The worktree and its build output are removed. The three binaries are kept as evidence but read a
  pack from the removed tree, so a rerun starts from the patch and the harness.
- No difference within these histories is not a proof.

## Tool/spec consequence and next gate

Stage 2 worked once on real code with the released command: consolidate, compare over bounded
histories, replay a witness, run the project's tests. Two owner decisions follow. First, whether to
apply the patch (it changes the signature of `with_repairs` in `lesscode-model`). Second, whether the
loop itself should refuse a reply that is not whole, which would give all three callers the rule and
change what `lesscode-model check` and `measure` accept. The open technical gate is the find step:
closures, match arms and writes through a cell (R153).

## COBESY check

The table is first; the planted fault and the model-against-runs count say why the comparison can be
believed. What no tool did is in the first limit. The second finding is stated with what was and was
not run.
