# Handover: state machines are found and untangled by hand, in checked steps; next is the kiro-cli extension

Reset 2026-11-18, 20:55 Europe/Berlin (from the clock). This file is the start for the next session.
The version before this one is in Git (`b08adf1`, reset of 17 November); older ones are
[handover-archive-2026-11-17.md](handover-archive-2026-11-17.md),
[handover-archive-2026-11-12.md](handover-archive-2026-11-12.md) and
[handover-archive-2026-11-11.md](handover-archive-2026-11-11.md). The day's detail, in order, is in
[task_76_handover.md](task_76_handover.md).

## Owner's direction of 22 November (replaces "Next" below where they differ)

- `kiro-cli` stays as it is: the spec-driven tool that measures code. The state measurement is built
  here instead, as a command `lesscode-metrics`. Nothing is to be changed in `openidea-kiro`; the
  concept note is the starting point for the command's spec, not for an amendment there.
- More examples are needed, and each must be runnable: a reader goes into an examples folder, runs
  the command and gets the table.
- `drift-rdp` has moved 91 commits past the example's base (`b6fbcf6`; head `09b4ce8` on 21
  November). The clipboard file is unchanged. Scan the repository again.
- `D:\Source\facdrone` (Python, finance code) is checked out only to adjust lesscode against a real
  project. Read only; nothing is delivered to it. It needs Python support in the command first.
- Say things in words. "R163" is the unrun test of a rule that says when a state machine is
  untangled far enough. "Slice B" is the hardening against model drift: closed by the owner, only
  drift of one model over time is unmeasured. The line about it under "State" below is stale.
- Order: first cut of `lesscode-metrics` for Rust (spec and prototype check first; it must give the
  hand-counted numbers of the four examples), runnable examples, rescan of `drift-rdp`, Python and
  facdrone, then the stopping-rule test if the owner still wants it. Nothing of this is begun.

## Start here

1. Read [the guide](../docs/state-refactoring.md) and [the examples](../docs/examples/README.md). They
   are what the owner wanted to be able to show.
2. Read [specs/state-consolidation/kiro-cli-state-metrics-concept.md](specs/state-consolidation/kiro-cli-state-metrics-concept.md).
   The owner's last decision of the day: **the kiro-cli extension comes first**, before the test of
   the stopping rule.
3. For anything about 8 or 9 November, [R134](../docs/research/2026-11-17/21-audit-of-port-and-reduction-claims.md)
   still stands: never report a check as done unless it ran on the real code and you saw its output.

## What the owner wants

Simpler code through the state work, shown in before-and-after examples that are visible in `docs`,
smaller codebases among them. The refactoring lives by the step-by-step flow, and the user guide
says so. Engineering metrics are declared with `openidea-kiro`, the owner's other project, and
complexity is accounted before and after by its tool. No proof is needed; a solver result within
bounds is said once to be no proof and is never a reason to hedge.

## State at the end of 18 November

- **Release 2.1.0 is published** (tag `v2.1.0` at `ebeae80`, published 18:59 Europe/Berlin). Its
  headline is a fix: on Linux a command that ran over its time limit could end all of a user's
  processes (`kill` without `--`; releases 1.1.0 to 2.0.0). Not checked by me after publication:
  signatures, checksums, attestation, Zenodo.
- `origin/main` is at `4b213d4`; the hosted pipeline on it was still running at 20:53. Local main is
  ahead by the preregistration of R163, the concept note and this handover. Pushing them is the
  owner's decision.
- After 2.1.0, on main and for the next release notes: the batch runner's progress in one value
  (step `batch-progress`, merged); a failing first run says which step failed and how its output
  ended; `step record` judges the verdict on disk; `run_batch` shortened.
- Last checks of the code as it is: Linux through Dagger at `4b213d4` overall 0, core 409 of 409;
  Windows core 410 of 410, clippy clean.
- Research today: R156 to R162 under `docs/research/2026-11-18/` (find step, transfer to two
  repositories, three consolidations, complexity before and after).
- Slice B is unchanged: measured through R79, the next step is the owner's choice.

## What exists, and what does not

- A finder for state kept in cell types, in Rust, as a research program
  (`docs/experiments/state-consolidation/machine-find/`). On a repository it had not read it found 8
  of 9 machines and reported none wrongly (R159). It is not a released command, it does not see state
  in plain fields, and after two of the consolidations it no longer sees the machine.
- Four examples under `docs/examples`: a made-up opener, the batch runner and the reply side channel
  from this repository, the clipboard lease from `drift-rdp`. All consolidations were made by hand.
- Measured on all four (R162): the number of decisions stays, the sum of cyclomatic complexity rises
  by one or two, the routine that now decides grows. The per-machine counts are the goal; the limits
  and ratchet of `kiro-cli` are the guard. `.kiro/code-rules.toml` declares the limits; no baseline is
  committed; `step check` and the ratchet stay separate commands (owner's decisions).
- No command says "this target is untangled far enough". A rule for it is registered and not run
  (R163, in the prototype gate of state-consolidation).

## Next

1. **The kiro-cli extension**, in `D:\Source\openidea-kiro`, on a feature branch in a separate
   worktree, not pushed. I had read the code and begun nothing; a worktree and branch made for it
   were removed again, and that repository is as it was (`6f66d5a`, clean).
   - Its owning spec is `code-locality-data` (approved, tasks generated). The repository works by
     requirements, design, tasks, then implementation, with the owner's review of each phase; ask
     whether the concept note is to become an amendment of that spec first.
   - Smallest first cut, as read from the code: two type metrics, `flags` and `cells`, for Rust.
     `crates/kiro-code/src/measure.rs` counts fields in `count_member` into `Members`; the language
     `Profile` would name the type heads that make a field a flag or a cell. Metric names are in
     `crates/kiro-core/src/code/facts.rs`, built-in limits in `code/config.rs` (`built_in`), hint
     texts in `code/evaluate.rs`. Tests that list every metric and the golden outputs will change.
   - Later cuts: `shared_locals`, `deciders`, the rule `CODE_STATE_SPREAD`, net decisions in the
     coverage line; then calibration on that project's five repositories.
   - The released binary for measuring is at `D:\BuildArtifacts\lesscode\tools\kiro-cli-1.1.0\x\kiro-cli.exe`.
     The debug build inside that repository is 0.1.0 and has no `code` command.
2. **R163**, the stopping rule: ten open machines, my judgement of each written and sealed before any
   count is taken, bar 8 of 10 plus the three known examples.
3. Further examples: `held_back` and `state` in the guide optimiser; the session state of
   `drift-rdp`, which needs the finder or `kiro-cli` to see plain enum fields first.

## Open, for the owner

- The branch `lesscode/clipboard-lease-machine` (`1a49d70`) in `D:\Source\drift-rdp` is local and not
  merged. `guard` there takes the owner's lock where it read an atomic; built and tested on Windows
  only.
- In the batch runner, putting a job back after a failed run is not protected by any test, and
  `run_batch` has no noticed fault with the two driver suites. No pin was written.
- One Windows run of the core suite, made while other commands ran on the repository, failed
  `the_installed_hook_stops_a_push_without_a_matching_verdict` once. It passed in four runs after.
  Not explained.
- A note on the release pages of 1.1.0 to 2.0.0 that points Linux users to 2.1.0.
- The main README's introduction with the opener: later, on the owner's word. A humanizer pass is due
  before any further push of README or guides.
- Two untracked files under `native` that I once committed by mistake and took out again before
  pushing: `adapters/java/lesscode/ProbeMain.class` and `budget-report.json.sha256`. Left as they are.
- From earlier handovers and still open: the `measures.rs` parenthesized-callee fix; removal of the
  prototype worktree `corrective-audit\foreground-20261118\prototype`; the R140 history form of
  `bound-check`; Java lowering; `match` arms as branch conditions in the Rust pack (prototype first).

## Storage

D: has 142.6 GiB free, measured at 20:53; it was 175 GiB at the start of the day. Most of the
difference is in the Podman machine's disk after seven Dagger runs, which this project cannot prune,
and in the shared Cargo target. Measure the lesscode-owned caches before the next heavy build; they
were near 32 GiB this morning and the limit is 40. Removed today: the isolated target of the
`drift-rdp` build (4.2 GiB) and the step's work directory (6.8 GiB). Retained evidence, small:
`D:\BuildArtifacts\lesscode\state-consolidation\r156` to `r162`, `native-evidence\`, and the
`kiro-cli` binary.

Next research ID: R163 is taken by the registered test; the one after is R164. Next handover file:
`task_77_handover.md`.
