# R112: The core reaches a model only through `lesscode-model`: `ureq` and the hand-written request are gone, a new receipt states only what was sent, and three pipeline checks fail on a planted violation

- Report date/time and timezone: 2026-11-16, 15:52, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-16, 14:20 to 15:51 (tasks 7.1 to 7.3: 14:20 to 15:02; review and lock files: to 15:10; task 7.4 with its Dagger runs: 15:10 to 15:37; the fixture and the last Dagger run: 15:39 to 15:51)
- Status: completed
- Spec: [model-seam](../../../.kiro/specs/model-seam/requirements.md), requirements 7.1, 7.2, 8.1, 8.3 to 8.6, 1.8, 2.8; [tasks 7.1 to 7.4](../../../.kiro/specs/model-seam/tasks.md); [design Amendment 10](../../../.kiro/specs/model-seam/design.md)
- Hypothesis ID and predecessor: migration step 4 of the design; the item carried from the gate, [R107](../2026-11-12/14-model-seam-gate-p4.md) ("the receipt states only what was sent"); [R111](../2026-11-14/04-model-seam-guide-tool-on-the-seam.md)
- Type: implementation check with scripted executables; no live call. The pass conditions are the tasks' (written 2026-11-12)

## Decision

Group 7 is done by the conditions of its four tasks. The item the gate carried forward is met: a new
turn's receipt has `"mode": "seam-1"` and only the settings `lesscode-model` reported as sent, and
receipts written before verify unchanged. Task 7.3 could have failed and did not.

The receipt's new form differs from the design's text in several points. They are recorded as design
Amendment 10 and listed below. One question is left for the owner.

## Hypothesis and falsification

The core's hand-written request can be replaced by a launcher that starts `lesscode-model call`, with
no client library left in the core, and a new receipt can then state one thing about what was sent
without breaking the verification of earlier receipts. Rejected when the core's suite fails, when
`native/Cargo.lock` still names `ureq`, when a new receipt names a setting the executable did not
report, when the recorded three-turn replay or the admission of a stored receipt fails, or when one of
the pipeline's three checks passes with a violation planted.

## Frozen inputs and method

- Start: main at `f7e07b8`. Commits: `bb1bc90` (7.1), `2ea4e26` (7.2), `e6203b3` (7.3), `3bd6496` (lock
  files of `model/` and `tools/guide-optimizer`, the success line), `bfab244` (7.4), `51f7f3c` (fixture).
- No model was called. The executable in every test is `native/examples/scripted_model.rs`: it answers
  from a JSON script named in the environment and records its arguments and the request it read.
- Toolchain 1.98.1. `cargo nextest run --locked`, `cargo clippy --locked --all-targets -- -D warnings`,
  `cargo fmt --check` and the budgets check, per crate, each with its own target directory.
- Pipeline: `dagger call check` with the native Windows CLI against Podman, on `51f7f3c`.
- Tasks 7.1 to 7.3 and task 7.4 were each written by a subagent, which the owner allowed for this group
  on 2026-11-16. The review of the diffs, the reruns of the suites, the two commits `3bd6496` and
  `51f7f3c`, the last Dagger run and this entry were made in the main context.

## Evidence and results

| Check, at `51f7f3c` unless stated | Result |
| --- | --- |
| `native`: nextest | 400 run, 400 passed (389 before the group); the three-turn replay ran, 46 s |
| `native`: clippy, format, budgets | clean, exit 0 |
| `native/Cargo.lock` | no `ureq`; 284 lines shorter |
| `model`: nextest, clippy (at `3bd6496`) | 79 of 79, clean |
| `tools/guide-optimizer`: nextest, clippy (at `3bd6496`) | 3 of 3, clean |
| `dagger call check` | "All native checks passed", 7 min 39 s |

Sizes, in tokens, logical units and the longest function:

| File | Before | After | Ceiling now |
| --- | --- | --- | --- |
| `regenerator.rs` | 20,788 / 858 / 2,292 (ceiling 20,827 / 862 / 2,314) | 19,699 / 830 / 2,303 | 19,699 / 830 / 2,303 |
| `seam.rs` | none | 1,667 / 72 / 367 | 2,800 / 300 / 560 |

The removed request code measures 1,072 tokens; `regenerator.rs` is 1,128 tokens below its old ceiling.

- **7.1.** `native/src/seam.rs` finds the executable by `LESSCODE_MODEL_BIN`, else beside the running
  program, else on the search path. Tests show an answered call, a failed call, four unreadable answers,
  the missing executable with the variable set and unset, and the search path.
- **7.2.** `cmd_run` and `default_call_fn` call the launcher. `HttpTransport`, `ureq_post`, `openai_call`
  and the hand-written `call` are deleted. One code fence around a reply is removed before parsing. A
  failed turn prints its reason to standard error and writes it to the receipt. The two tests of the
  injected transport became four with the scripted executable.
- **7.3.** A test writes new receipts with and without a temperature and finds no setting the scripted
  executable did not report; with unsent settings left in on purpose, the test failed. A second test
  adds a new turn to the stored receipt of gate P4: its earlier facts and notes are unchanged, the
  earlier turn says `openai-json-http`, the new one `seam-1`, and the receipt passes admission.
- **7.4.** Three steps in `tools/check_native.sh`, which the hosted runners and the Dagger container
  share. Each failed on its plant and passed without it, run directly on Windows and in Dagger:

| Check | Plant | Result with the plant |
| --- | --- | --- |
| `separation-suites` (52 tests of the `step_*` and `driver_*` binaries) | a `lesscode-model` on the search path, in the variable, beside `lesscode-core`, beside the test executables | exit 1 before any test ran, naming the file |
| `separation-lock` | the core's manifest and lock file from before `2ea4e26` | exit 1, `848:name = "ureq"` |
| `separation-model-source` (the existing scan in `model/tests/call.rs`) | a model name, then an address, in `model/src/client.rs` | exit 100, `client.rs contains "gpt-"`, `contains "https://"` |

  The plant evidence and the logs of three Dagger runs are in `D:\BuildArtifacts\lesscode\task-7.4\`.

Found on the way and repaired:

- The lock files of `model/` and `tools/guide-optimizer` still named `ureq` through the core and failed
  `--locked` commands. Refreshed in `3bd6496`; both changes are deletions only.
- The 7.3 test read the stored receipt from `docs/`, which the pipeline's container does not hold. A
  copy is now `native/tests/fixtures/model-seam/receipt-p4-3.json` (`51f7f3c`).
- The Linux job had failed since task 6 (run 43874632025: `optimizer-fmt`, `optimizer-clippy`), because
  the container did not hold `model/`, which the guide tool depends on. `bfab244` mounts it.

## Limits and interpretation

- No live call was made in this group. The launcher against the built `lesscode-model` on a real
  endpoint is not shown here; the gate's prototype showed the path on six cases (R107), with another
  executable and the earlier command line.
- macOS and the hosted jobs have not run: nothing is pushed.
- Two of the three Dagger runs on `51f7f3c` were killed with exit 137 in the main test step, at test 195
  of 399 both times. The container host had about 200 MB free of 8.3 GB while the Dagger engine held
  4.7 GB. After a restart of the engine container the run passed. Read as a memory limit of the local
  Podman machine, not shown beyond that.
- Different from the design's text (Amendment 10):
  1. The launcher returns the call fact in the receipt's words, with digests of the system text, the
     user text, the request as written and the reply. About 74 tokens were left in `regenerator.rs`
     for the replacement, so the regenerator only passes the fact on. A new turn does not use
     `request_kwargs` and `hash_set`.
  2. In a new turn's note `sent` is nested, `seconds` replaces `elapsed_s`, and `model_sent`,
     `model_returned`, `seed` and `seed_note` are gone. The seam does not report the model name an
     endpoint returns.
  3. Every request carries `"purpose": "regenerate"`; `regenerate` asks for no structure.
- Changed for a user of `regenerate run`:
  1. It needs `lesscode-model`. Without it the command ends with exit 3 and says where it looked.
  2. `--temperature` is sent only when given. Before, 0.0 was always sent.
  3. `--effort` and `--thinking-display` are accepted, not sent and not recorded. They were hashed into
     the receipt before and never sent by the Rust core. The reasoning level is the endpoint file's.
  4. A reply in a code fence is accepted. The stored stop word is `complete`.
  5. A failed call's reason appears twice on the terminal, from `lesscode-model` and from the core.
- The refine loop's token-budget stop now reads real counts; the earlier usage keys did not match what
  it reads.
- The `separation-suites` check looks beside the executables at cargo's default target location. A
  target directory set in a cargo configuration file would be missed; neither the repository nor the
  pipeline has one. The step and driver suites run twice in the pipeline.
- The reused source scan finds addresses only with `http://` or `https://` and reads only the files
  directly in `model/src`.
- `seam.rs` has no digest in the apparatus list of a receipt; it did not fit under the regenerator's
  ceiling.

## Tool/spec consequence and next gate

- Requirements 7.1, 7.2, 8.1 and 8.3 to 8.6 are met with scripted executables. The item of gate P4 is
  closed.
- Design Amendment 10 records the as-built form. Nothing in the requirements changes.
- For the owner, not acted on: the refine loop's schedule still steps an effort to "medium" and records
  effort and display in its attempt notes, although neither is sent. `regenerate refine` is not
  implemented, so no receipt of a user is affected today.
- Next: group 8. Task 8.1 adds format, lints, the full suite and budgets of `model/` to the pipeline and
  can take over the build that `separation-model-source` now does. One live `regenerate run` through the
  built `lesscode-model` belongs in the final validation (8.4).

## COBESY check

The title states what was shown and by what: scripted executables and planted violations, no live
call. The two killed Dagger runs are reported beside the one that passed. The changes for a user and
the differences from the design are listed and not folded into "as designed".
