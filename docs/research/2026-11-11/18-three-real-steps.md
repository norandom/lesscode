# R92: Three real steps through the step tooling, in Rust, Java and TypeScript: all accepted; no refusal and no pin occurred

- Report date/time and timezone: 2026-11-11, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-11, 18:45 to 19:16 local (times from command output)
- Status: completed
- Spec: [gradual-refactoring-steps](../../../.kiro/specs/gradual-refactoring-steps/tasks.md), tasks 6.1 and
  6.2, Requirements 11.2 to 11.5
- Hypothesis ID and predecessor: [R89](15-first-tracked-step.md) (the same change, by hand, refused by a
  line ceiling) and [R91](17-gradual-steps-three-assumptions.md) (the gate of the design)
- Type: three real steps, each from declaration to guard, with the commands of task groups 3 to 5

## Decision

The step commands carry a real step from start to finish in three languages. Each of the three steps was
declared, committed, judged by `step check`, put on a ledger by `step record` and passed by `step guard`,
all by exit code 0.

| Step | Project, language | Function: noticed / unnoticed before | after | Tokens of the file | Wall time of `step check` |
| --- | --- | --- | --- | --- | --- |
| `one-exit` | this repository, Rust | `verdict` 11 / 0 | 12 / 0 | 7,307 to 7,306 | 9 min 57 s |
| `comment-start` | commons-csv, Java | `printComment` 19 / 2 | 17 / 2; new `printCommentStart` 2 / 0 | 1,420 to 1,415 | 25 min 23 s |
| `slug-separator` | OpenUltraCode, TypeScript | `slugify` 9 / 1 | 8 / 0 | 1,283 to 1,277 | 4 min 3 s |

What this does not show: no step was refused and no pin was needed, so `step pin` and the refusals were not
exercised on a real step here. They are covered by the integration tests on fixture repositories (task 4.4)
and by R91. Requirement 11.2 asks for the verdicts "including any refusal"; there was none to record.

Separate outcomes: tooling works end to end (yes, three of three); behaviour of the changed code (tests pass,
fault sensitivity not lower; not an equivalence proof); size (5, 6 and 1 tokens fewer, none of them a
reduction worth the name); adoption (`one-exit` waits for the owner's decision on merging; the foreign steps
are on local clones and go nowhere). No model was called.

## Hypothesis and falsification

Stated before the runs: the `verdict()` change, refused in R89 by a ceiling in lines, is accepted by the
tooling now that budgets count tokens (R90) and the pin is on main. It would have been refuted by a refusal
on size (growth above 0 tokens), on planted faults, or by a run that is not judged. It was accepted.

For the foreign steps nothing was predicted. The Java step was declared with `max_token_growth` 0 although
it adds a method; a refusal on size was possible and would have been recorded as such. The file became five
tokens smaller.

## Frozen inputs and method

- `lesscode-core` built from main at `9ee2fc3` (debug build). Machine shared by the three runs, which
  overlapped; times are wall time under that load.
- `one-exit`: branch `step/one-exit` of this repository from `9ee2fc3`. Commits `08f4a05` (spec and
  declaration), `e64b7b3` (the change), `0d54533` (verdict, evidence, ledger). Test command
  `cargo nextest run --locked --manifest-path native/Cargo.toml --test recording`, that is the tests of
  `tests/recording.rs` only, with `CARGO_TARGET_DIR=target`; criteria 1.1 (the `verdict_` tests) and 1.2
  (the whole suite) as tied commands. Focus on `verdict`. The patch of R89 did not apply with `git apply`
  (context offset), so the change was made by hand; the resulting file has the blob hash `da63888`, the one
  the patch names.
- `comment-start`: clone of the copy of commons-csv at `2bd66bf`, branch `step/comment-start`, commits
  `fe8723b`, `5c0cb73`, `e79ae9e`. Maven offline, `-Dtest=CSVPrinterTest`. Focus on `printComment`.
- `slug-separator`: clone of the copy of OpenUltraCode at `37a4e55`, branch `step/slug-separator`, commits
  `f2ee9e0`, `752b8eb`, `aec1e39`. `node --import tsx --test tests/degradation.test.ts`. Focus on `slugify`.
- The clones are under `D:\BuildArtifacts\lesscode\steps-foreign`; the copies under
  `annotation-driver\scopes` and the upstream checkouts were not touched.
- Each step has a spec of its own with three criteria (`requirements.md`, `spec.json`); `kiro-cli check` on
  `one-exit`: 0 errors, 0 warnings.

## Evidence and results

`docs/experiments/gradual-steps/steps/<step>/`: declaration, verdict, evidence, ledger; for the foreign steps
also the requirements and the change as a patch.

- All eight checks pass in each verdict. Guard: "accepted", exit 0, on the commit that holds the verdict.
- `one-exit`: one fault more is planted in the new shape of `verdict` and it is noticed. Functions run: only
  `verdict` (no caller in scope).
- `comment-start`: `printCommentStart` has no counterpart on the base, so the totals of the file were
  compared too. The two statements that moved took their two noticed faults with them. The constructor
  `CSVPrinter` calls `printComment` and was run as a caller: 8 / 3 before and after.
- `slug-separator`: the one unnoticed fault of `slugify` on the base was in `slug.length > 0`, the half of
  the condition that cannot be false; no test could have noticed it. Three callers were run; no fault is
  planted in them.
- The ledgers have one entry each, with an empty `previous`.

## Limits and interpretation

- Three small, easy steps chosen by the author of the tooling. None needed a fake, a frame specification or
  an out-of-reach mark.
- In the TypeScript step the noticed faults went from 9 to 8. The rule looks at unnoticed faults only, and
  that is right here (the planted fault disappeared with its code), but a step that removes well-tested code
  also passes it. The verdict shows the numbers; docs/steps.md says so under "What no command checks".
- The `one-exit` test command runs one test file. The whole suite ran once as criterion 1.2, not against
  every fault.
- The guard was called by hand. No step branch was pushed through the hook in this entry; that path is
  covered by the test of task 5.3.
- A Java step of two moved statements took 25 minutes because a caller with 13 faults and `printComment`
  with 21 were run before and after, each fault a Maven build.

## Tool/spec consequence and next gate

- Tasks 6.1 and 6.2 are done. docs/steps.md carries the three steps as examples.
- For the owner: merge `step/one-exit` or not. It is local, not pushed.
- Checks that could not be made mechanical are listed in docs/steps.md ("What no command checks") and named
  in the skill `refactor-in-steps`, which does not decide them (Requirements 11.4, 11.5).
- Not done and worth a real case next: a step that needs a pin, and one that is refused, in a foreign
  project.

## COBESY check

Outcome first: three accepted, and what was not exercised is in the title. Replay: commits, commands and
files named. Disproof: the prediction for `one-exit` could have failed on three named checks. Facts and
reading are apart; the size gains are called what they are.
