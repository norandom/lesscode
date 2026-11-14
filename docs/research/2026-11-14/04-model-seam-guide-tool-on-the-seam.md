# R111: The guide tool calls models through the seam: its dry runs give the same summaries as before, and a live judgement of four trials on GLM 5.3 completes with three of four packs admitted

- Report date/time and timezone: 2026-11-14, 10:58, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-14, 10:21 to 10:54 (dry runs before the change from 10:21, after it until 10:48; the live run: 10:51:26 to 10:53:43)
- Status: completed
- Spec: [model-seam](../../../.kiro/specs/model-seam/requirements.md), requirement 8.2, [task 6](../../../.kiro/specs/model-seam/tasks.md)
- Hypothesis ID and predecessor: the migration step 3 of the design ("move `tools/guide-optimizer` onto the seam; delete its client"); the seam's repair loop, task 3
- Type: migration check with one live run; the pass condition is the task's (written 2026-11-12)

## Decision

Task 6 is done by its three conditions: the tool's tests pass, both dry runs give byte-identical
summaries to those of the commit before, and one live judgement of four trials on GLM 5.3 completed.
`tools/guide-optimizer` has no client of its own any more. Every call goes through
`lesscode_model::call`, and the repair turns are the seam's `with_repairs`.

Four behaviours of the tool changed with the move. They are the seam's rules and are listed under
"Limits and interpretation" for the owner.

## Hypothesis and falsification

The tool's request building, reply reading, retry wait and repair loop can be replaced by the seam
without a change to what a dry run reports, and a live judgement then runs to its end. Rejected when a
summary of a dry run differs from the one before, when a test fails, or when the live run stops before
its four trials are judged.

## Frozen inputs and method

- Before: main at `d7902e7`. The tool built to its own target directory; two dry runs from the
  repository root: the default optimisation (`--dry-run`), and a judgement
  (`--dry-run --judge-only docs/experiments/annotation-driver/pack-guide-v3.md --repairs 2`). In the
  second the stand-in's first Rust reply is rejected, so the repair path runs.
- After: the same two runs on the changed tool, compared file by file.
- Live: `docs/experiments/model-seam/run-guide-tool.ps1`. The repository's `.env`, the local endpoint at
  `127.0.0.1:8765` (container `mvibe-proxy-proxy-1`, started by the owner), model `zai-glm-5-3`.
  `--judge-only` with guide version 3, `--judge-list rust,java,python,typescript`, `--repairs 2`, output
  limit 4,000 tokens per authoring call. The seam's limits at their defaults. Nothing else of ours ran
  against the endpoint. The check is the tool's `judge` (admission and the control sets of each language).
- The implementation was written by a subagent on the owner's instruction of 2026-11-14 for this group;
  the decisions, the review of the diff, the comparison and the live run were made in the main context.

## Evidence and results

What went from the tool: its `Reply`, `Live::request`, `Live::read`, `text_of`, `retried`, `WAITS`, the
use of `ureq_post`, `Authoring::attempt`, the hand-written loop in `Authoring::one`, and its
`repair_request`. What came: the `Model` trait takes the seam's `CallRequest` and returns the seam's
`Reply` or `CallError`; `Live::chat` is one call of `lesscode_model::call`; `Authoring::one` runs
`with_repairs` with the tool's `judge` as the check.

| Check | Result |
| --- | --- |
| `cargo nextest run` in `tools/guide-optimizer` | 3 run, 3 passed |
| `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` | clean |
| Dry run, optimisation: `summary.json`, `ledger.jsonl` (83 lines), candidates, log | identical to before |
| Dry run, judgement with two repairs: `summary.json`, `ledger.jsonl` (25 lines), log | identical to before |
| `trials.jsonl` of both dry runs | differ only in the stop word: `end_turn` became `complete` |

The crate has no library, so it has no doc tests. The three tests are new: the stand-in answers the
seam's repair request with the shipped pack; a cut-off or refused reply is never admitted; a call the
budget stops is recorded as `not_run`. The four tests of the deleted client went with it.

The live judgement (kept in `docs/experiments/model-seam/guide-tool/`), 137 s, exit 0, 9 calls:

| Language | Calls | First reply | In the end | Input tokens | Output tokens |
| --- | --- | --- | --- | --- | --- |
| rust | 3 | rejected (node kinds, controls) | not admitted: both further replies cut off at 4,000 | 7,698 | 10,773 |
| java | 3 | rejected (node kinds) | admitted on the second repair | 8,652 | 5,283 |
| python | 2 | cut off at 4,000 | admitted when asked again | 4,884 | 7,175 |
| typescript | 1 | admitted | admitted | 2,467 | 2,567 |

- First replies admitted: 1 of 4. Admitted within two repairs: 3 of 4. No call failed, none was refused,
  none was stopped by the budget.
- Tokens: 23,701 in, 25,798 out. The tool has no list price for `zai-glm-5-3` and counted its fallback
  of 15 and 75 USD per million tokens, which gives 2.29 USD in the ledger. That figure is not what the
  calls cost.
- One notice of the seam appeared: "still waiting for the answer of 127.0.0.1:8765: 45 s so far".
- The stop words in the records are the seam's (`complete`, `cut_off`). No kept file contains the key.

## Limits and interpretation

- Four trials say nothing about the guide or the model. The run shows that the path works live: a first
  call, a repair with the reasons, a cut-off reply asked again, and the notice of a slow answer.
- Changed with the move, each the seam's rule:
  1. The repair text says "answer" where the tool said "pack", and shows the whole rejected reply where
     the tool showed the pack taken out of it.
  2. A call that fails ends the trial with `error`. The seam waits out a refusal for a rate limit (up to
     ten minutes); the old tool also sent any failed call once more, which is gone.
  3. A refused reply is asked again as it was, within the repairs allowed. The old loop ended the trial
     at a refusal. On an endpoint that keeps refusing this costs up to `repairs` calls per trial. The
     client library also reports a content filter as a refusal, so such a reply is now rejected unjudged.
  4. The records use the seam's stop words. Trial files written before use the endpoints' words
     (`end_turn`, `stop`, `max_tokens`, `length`); the tool reads both, an analysis that mixes them sees
     two vocabularies.
- The Rust trial shows a property of rule "not complete, ask again as it was": after a repair reply is
  cut off, the next call is the first request and not the repair request, so the reasons of the first
  rejection are not shown again. Both further Rust replies ran into the output limit of 4,000.
- The dry runs use the engine `gepa`. The `mipro` path was compiled and linted, not run.
- A budget stop is carried as `CallError::Config` with a fixed text, since the seam has no error for a
  call its caller did not make. Its feedback line now starts "no reply: configuration: not run".
- The seam's waits, seconds and reasoning tokens are in neither the trials nor the ledger; the shape of
  the records was kept.
- The tool's lock file gained the client library's packages. Three versions differ from the seam's own
  lock file (`mio`, `bumpalo`, `webpki-roots`, each by one patch version).

## Tool/spec consequence and next gate

- Requirement 8.2 is met for the research tool. The core's own request (`native/src/regenerator.rs`) is
  the last hand-written client; it leaves in task 7.2.
- Nothing in the design changes.
- For the owner, not acted on: whether a refusal should end a trial again (point 3), and whether a reply
  cut off after a repair should be asked again with the repair request.
- Next: group 7, the core on the seam (the launcher, then the receipt that states only what was sent).

## COBESY check

The title states what was shown and how much: four trials, three admitted. The cost figure is named as
the tool's fallback count and not as a price paid. The behaviours that changed are listed and not
folded into "same as before"; the summaries are the same, the tool is not.
