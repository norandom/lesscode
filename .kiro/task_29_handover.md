# Task handover 29: agnostic driver, task group 1 (foundation)

Written 2026-11-10 (evening, Europe/Berlin). Spec: `.kiro/specs/agnostic-annotation-driver/`.
Start the next task group from this file.

## What was done

Owner decisions and approvals of this session, all 2026-11-10:

- Requirements approved ("make sure we don't forget slice B this time").
- Apparatus cap: the driver stays outside the counted set of the 7,173-line cap and is governed by the
  per-file budgets ("(1) it is").
- Design approved; tasks approved. The fixture question (may the five gate scope files go into the
  repository) was asked twice and not answered; task 3.4 uses the default (path from an environment variable,
  "not run" when absent).

Work:

- Sizing spike (R65): a Rust port of the model-free core reproduces the prototype exactly on five scopes.
  `docs/experiments/annotation-driver/native-spike/` is the reference for porting.
- Task 1.1 (`8b5e69e`): grammar features for Rust and Java; lock file gains `tree-sitter-java` 0.23.5 and
  `tree-sitter-rust` 0.24.2 only; licence inventory and notices updated.
- Task 1.2 (`af86525`): `native/packs/{java,python,rust,typescript}.json`; typed pack; shipped-pack registry.
- Tasks 1.3 and 1.4 (`8852d11`): tree and source helpers; budgets registered.

## Evidence

- 371 tests pass with cargo-nextest (356 before, 15 new); `cargo fmt --check` and
  `cargo clippy --all-targets -- -D warnings` clean; doctests pass; budgets check exit 0.
- CI for `8852d11` was pushed and not checked in this session.

Measured lines against the design table (total / production / longest function):

| File | Design | Measured |
| --- | --- | --- |
| `adapters/driver/mod.rs` | about 60 | 23 / 23 / 0 |
| `adapters/driver/pack.rs` | 133 | 237 / 176 / 22 |
| `adapters/driver/registry.rs` | not in the design | 82 / 30 / 8 |
| `adapters/driver/tree.rs` | 104 | 184 / 123 / 24 |
| `adapters/driver/source.rs` | 153 | 148 / 86 / 28 |

Production lines are 438 against 450 in the design for these parts; totals are higher because the unit tests
live in the files.

## Deviations from the design (design Amendment 1)

- Location is `native/src/adapters/driver/`, not `native/src/driver/`: `lib.rs` is at its measured ceiling
  (196 of 196 syntax units), so the module is registered in `adapters/mod.rs`. Command handlers will go to
  `adapters/driver/cli.rs`.
- `registry.rs` is new: the only driver file that lists languages, by naming the pack files.
- The typed pack is stricter than the prototype: unknown keys are rejected, and `requires_next_sibling` and
  `swap_next` must be booleans.
- The Rust pack gained `test_attribute_kinds` (`attribute_item`, `inner_attribute_item`), replacing a literal
  in the prototype's code. Its digest therefore differs from the prototype's pack; parity of sites has to
  confirm the rule is equivalent (task 3.1).

## Current state

- main at `8852d11` plus this handover; nothing of the old adapters is changed or removed.
- The driver is not reachable from any command yet.
- `cli_deconvolute.rs` is still at 399 of 400 lines; do not add to it.

## Next steps

1. Task group 2: admission on the typed pack (2.1), control replay options (2.2), `deconvolute admit-pack`
   (2.3), and the Slice B preregistration (2.4). The admission check names must stay `size`, `shape`,
   `kinds_valid`, `templates`. Reference: the spike's `pack_check.rs` and the prototype's `pack_check.py`;
   the two model-written packs are in `docs/experiments/annotation-driver/results/attempt2/d2/packs/`.
2. **Slice B** (hardening against LLM inference drift): queued second, not started. Its dated step is task
   2.4: the preregistration of the pack-authoring drift baseline is due with the handover of group 2. Running
   the baseline needs the owner's go.
3. Ask the owner again about the fixtures before task 3.4.

## Standing rules that bit in this session

- Never manage the Podman machine; containers only.
- Stage files by name; about 200 untracked owner files must stay untracked.
- `cargo update -p` re-resolved an unrelated `windows-sys` edge; check the lock diff after any resolve.
- Build with `CARGO_TARGET_DIR=D:\BuildArtifacts\cargo-target\lesscode-native`.
