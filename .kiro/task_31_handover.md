# Task handover 31: agnostic driver, task group 3 (engine with parity)

Written 2026-11-10 (evening, Europe/Berlin). Spec: `.kiro/specs/agnostic-annotation-driver/`.
Predecessor: `.kiro/task_30_handover.md`. Start the next task group from this file.

## What was done

- Task 3.1 (`b98b1ef`): `adapters/driver/sites.rs`, site rules and eligible sites.
- Task 3.3 (`852895c`): `adapters/driver/operators.rs`, operators and mutant classification; packs gain
  `negation_swaps`.
- Task 3.2 (`8724f26`): `adapters/driver/annotations.rs` (validation, ten reasons) and
  `adapters/driver/controls.rs` (control replay); `deconvolute admit-pack --controls K --source-root S` now
  runs the checks `controls` and `seed_mutants`. This closes what task 2.2 left open.
- Task 3.4 (`2009845`): fixtures, references, `native/tests/driver_parity.rs`, the name scan, research entry
  [R67](../docs/research/2026-11-10/13-native-driver-engine-parity.md).

The three commits before `2009845` were staged file by file and were not built or tested individually; the
state at `2009845` is the tested one.

## Evidence

- 388 tests pass with cargo-nextest; `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and the
  budgets check are clean locally. CI for `2009845` was pushed; see "Current state" for its result.
- Parity (R67): sites identical on five scopes (71, 211, 70, 64, 184); mutants identical under the gate packs
  (66, 199, 74, 66, 189, by hash); 98 of 98 controls as recorded; control replay through the command admits
  the shipped packs and rejects a weakened pack.
- Intended difference (Requirement 3.4): with the shipped packs 38 of 594 mutants are dropped as repeats of
  a negation (2, 3, 3, 11, 19 per scope).

Measured lines (total / production / longest function):

| File | Measured | Design |
| --- | --- | --- |
| `sites.rs` | 256 / 256 / 33 | about 320 with validation |
| `annotations.rs` | 203 / 203 / 32 | part of `sites.rs` in the design |
| `operators.rs` | 265 / 265 / 55 | about 330 |
| `controls.rs` | 127 / 127 / 28 | part of admission in the design |
| `admission.rs` | 349 / 284 / 50 | about 180 |
| `cli.rs` | 83 / 83 / 37 | part of about 280 |

Driver production lines so far: 1,725 in eleven files, all within the per-file budgets.

## Deviations and things to know

- New pack key `negation_swaps` (optional, in `operators.branch_condition`): token pairs whose swap repeats
  the negation. The shipped packs list `==`/`!=` (TypeScript also `===`/`!==`). A pack without the key is
  admitted and does not suppress repeats.
- Control file format of the product (`controls.rs`): `{"files", "test_paths", "cases": [{"label",
  "annotations", "expected"}]}`. The prototype's seed-derived `controls.json` is not read by the product;
  the parity test builds control files from the references.
- Validation differs from the prototype only on malformed input that no control covers: a `file` that is not
  a string is treated as an empty path (outcome `outside_scope`), and an annotation that is not an object is
  `malformed` instead of an error.
- `mutants()` reparses the file for every site (about nine seconds for the parity test in a debug build).
  The batch runner should parse once per file.
- Fixtures: `native/tests/fixtures/driver/scopes/**` is `-text` in `.gitattributes`; the files are CRLF as
  the gate ran them. Licence texts are beside them; origins and commits are in the fixture README.
- The owner asked during this group whether the many running containers were needed. Seven idle Java worker
  containers from gate attempt 2 had been left running; they and their seven compose networks were removed.
  The Dagger engines, `mvibe-proxy` and the Maven cache volume were left alone. Rule recorded in memory:
  remove a batch's containers when it ends.

## CI failure found and fixed in this group

The Linux job of the group 2 commits (`0ac7fb3`, `ab3eb60`) and of `2009845` failed while the other four
platforms passed. Cause: the Linux check runs in a Dagger container that mounts only `native/` and `tools/`,
and the admission test read the two model-written packs from `docs/experiments/`. The packs are now copied
byte for byte to `native/tests/fixtures/driver/model-packs/`. Rule for driver tests: read nothing outside
`native/`. I had reported group 2 without checking its CI run.

## Slice B status

Queued second, not started. R66 Amendment 1: the engine-dependent start condition is met. Still open before
the first call: `pack-guide-v3.md` (one sentence each for `test_attribute_kinds` and `negation_swaps`) with
its hash recorded in R66, and the owner's go. The run stops at the 20 USD cap.

## Next steps

1. Task group 4: move the `analyze` and `mutate` handlers out of `cli_deconvolute.rs` into
   `adapters/driver/cli.rs` (4.1), route `analyze` through the driver for four languages with receipt
   `deconvolution-analysis-v2` (4.2), route `mutate` with receipt `deconvolution-mutation-v2`, `--limit` and
   `--seed` (4.3). Existing CLI tests in `native/tests/cli_deconvolute.rs` and the per-language adapter
   tests pin today's behaviour, including the Python refusal (exit 2); they change with 4.2 and 4.3.
2. Ask the owner for the go on the Slice B baseline, or keep it queued behind the driver.

## Standing rules

- Never manage the Podman machine; containers only, and remove them when a batch ends.
- Stage files by name. Build with `CARGO_TARGET_DIR=D:\BuildArtifacts\cargo-target\lesscode-native`.
