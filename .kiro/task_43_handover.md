# Task handover 43: release 1.2.0 prepared; a refused and pinned real step; a driver fault found and fixed

Written 2026-11-11, 20:30 (Europe/Berlin, from the clock). Predecessor: `.kiro/task_42_handover.md`.
This work has no task list of its own. It follows owner decisions of 2026-11-11, given in the session after
handover 42.

## Owner decisions this handover rests on

- Prepare release 1.2.0; the owner pushes the tag.
- Before the version bump, take one real step that needs a pin and one that is refused, in a foreign project.
- README pass, release wording, docs audit; run README and docs through the humanizer skill before pushing.
- No issue for openidea-kiro. It is the owner's co-project (`../openidea-kiro`); an alignment note instead,
  with its `RunRecord` as the target of the evidence file.
- The solver note should say what lesscode and openidea-kiro use where, and why, with a link.

## What was done

| Item | Commit | Result |
| --- | --- | --- |
| Solver section with link to openidea-kiro | `3b43570` | `docs/symbolic-execution-tooling.md` |
| Evidence alignment instead of an issue; design Amendment 5 | `60f79f3` | `.kiro/specs/gradual-refactoring-steps/evidence-alignment.md` |
| Driver fix: a working copy that is used again follows its source | `aa9d7d7` | `native/src/adapters/driver/sync.rs`; test `a_changed_test_file_reaches_a_copy_that_is_used_again` fails without the fix |
| Help texts; `step record` message | `9495223` | |
| Real step `shape-checks` in OpenUltraCode: refused, pinned, accepted; R93 | docs commit of this handover | [R93](../docs/research/2026-11-11/19-refused-pinned-accepted.md); files in `docs/experiments/gradual-steps/steps/shape-checks/` |
| README pass, release wording, docs audit, humanizer pass | docs commit of this handover | see below |
| Release preparation | release commit of this handover | version 1.2.0 in `native/Cargo.toml`, both lock files, `CITATION.cff`, `.zenodo.json`, README badge; `docs/releases/1.2.0.md` |

388 tests pass at version 1.2.0; format check, lints, doctests clean; `kiro-cli check` on
`gradual-refactoring-steps` 0 errors, 0 warnings; relative links of the current guides resolve.

## Docs audit: what was wrong and is corrected

- README and the skill said no command makes a network or model request. `regenerate`, a research command
  in the same binary, sends prompts to a model endpoint that the user configures. The statements now name
  the commands they hold for.
- The top-level help of the binary described release 1.0.
- `docs/multi-language-deconvolution.md` described per-language adapters that no longer exist, under a list
  of corrections. It is now a short page about the binary as it is; the old page is in `docs/archive/`.
- `docs/cli.md` had no archive notice although it describes the retired Python prototype.
- `docs/README.md` linked to removed files and to a README anchor that no longer exists.
- `docs/adoption.md` said the binary keeps no baseline; the step ledger is one.
- `docs/ci-validation.md`: `tools/adopter_check.sh` was written for 1.0.x and probes a file that is gone.
  The guide now says so. The script itself was not changed.
- `docs/steps.md` had a heading run together with its paragraph.

Not audited line by line: the archived Python guides, `docs/reports/`, `docs/experiments/`, the research
log entries before R92, `docs/release-signing.md`, `docs/testing.md` (read, no change needed).

## Humanizer pass

Applied to README, `docs/steps.md`, `docs/testability.md`, `docs/ci-validation.md`, the new architecture
page, the release notes and R93: bold lead-ins in lists removed, "not X but Y" constructions rewritten, a
signposting sentence removed, the README opening rewritten in plain words, one title-case heading lowered.
A scan for dashes, curly quotes and the usual vocabulary found nothing else in the current guides. Left as
it is: the research index, whose bold lead-ins are that file's convention.

## Current state

- main: everything above, pushed. CI on five platforms: success for `db175e5` (run 43087717445) and for
  `3710db7` (run 43088660633, README reworded for four languages). Later commits change documents only
  (`docs/steps.md` reworded the same way, `.kiro/handover.md` reset for the next session).
- Not tagged. `docs/releases/1.2.0.md` has the steps for the owner.
- Foreign clones under `D:\BuildArtifacts\lesscode\steps-foreign`: `OpenUltraCode` (branches
  `step/slug-separator`, `step/shape-checks`), `commons-csv` (`step/comment-start`), `OpenUltraCode-replay`.
  Work directories there and `step-one-exit` can be deleted. `work-shape`, `work-replay2` hold stored
  outcomes from runs without the fix and must not be reused.

## Open

1. The tag `v1.2.0` (owner). `date-released` in `CITATION.cff` is 2026-11-11.
2. `tools/adopter_check.sh` is stale; bring it up to date or retire it.
3. Per-test results for the evidence file, so that openidea-kiro's `breadth accept` can take it; waits for
   `verification-evidence` to be implemented there.
4. Not shown on a real step: override, release entry on the ledger, frame specification, out-of-reach mark,
   a push of a step branch through the installed hook.
5. Stores written on this machine by builds between task 2.1 and `aa9d7d7` can hold wrong classes if a work
   directory was used twice with changed tests. Use fresh work directories.

## Notes

- A mistake of this session, recorded in R93: three runs reported as made with the fix used a binary built
  with the fix switched off. `cargo build --examples` does not rebuild the executable; build `--bins` too,
  and copy the executable before starting long runs, because a running step check locks it.
- OpenUltraCode's `pnpm` prepare script sets `core.hooksPath` in a clone.
- No model was called. Model spend unchanged: about 32.9 of 40 USD. Slice B is closed. No container of this
  project was started.
