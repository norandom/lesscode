# Task 76 handover · The find step on the reply machine, and what consolidation changed in the measures

Written 18 November 2026, 15:32 Europe/Berlin (from the clock). Start from here and from [handover 75](task_75_handover.md), whose open points still stand. The owner said at the start of this session: Linux verification later; focus on the state machine and its effect on deconvolution.

## Done

**A finder returns the reply machine from the four files** ([R156](../docs/research/2026-11-18/22-find-step-reply-machine.md)). Research binary `find-machine` in the `e5-bound` experiment crate, source and rules under `docs/experiments/state-consolidation/machine-find`. Five of six registered rows exact. The mover's guards came back with two extra lines, `repair.rs:116` and `117`, which are real guards. Registered check: `revise`. Controls passed: head, a renamed copy, a copy whose read feeds no condition.

**Sweep of 97 own Rust files.** One machine not written down before: `fatal` in `native/src/adapters/driver/runner.rs` (set at 187 and 190, read at 261 and 382; the finder misses 382). Two budget cells in `tools/guide-optimizer/src/model.rs`, found only after revision 2 (a read bound to a name that a later condition tests).

**Effect of the consolidation of R154 and R155 on the measures.** Four conditions on the stop reason before and after; two units in two files before, one function after; no cell, no shared closures, no hand-over after. `analyze` and `measure-scope` show five call-argument sites and two calls of difference and nothing else, because `match` arms are not conditions for the Rust pack. The mutation step therefore has no site on this rule, before or after.

## Decisions for the owner

- Accept `repair.rs:116` and `117` as part of the expected answer? Then the find step passes on its first real case.
- Should `match` arms become branch conditions in the Rust pack? The pack is at 40 lines and 2,480 of 2,500 characters.
- Where should the finder's rules live? They are a second data file of 20 lines per language, not pack entries.
- Still open from handover 75: the `measures.rs` callee-in-parentheses fix, and the prototype worktree `corrective-audit\foreground-20261118\prototype`.

## Not done

- Linux is still unverified for `48010b5` and `3db7caf` (Dagger exit 137 three times, cause not found). Deferred by the owner.
- Nothing was ported to `lesscode-core`. No test covers the finder; it was checked by the registered rows and controls only.
- Known misses of the finder: a field read through another name than `self`; captured mutable variables (`judged` in the adapter at head); more than one binding between a read and its condition; `lock` counts as write and read.

## Storage

D had 175.8 GiB free at entry and 175.7 GiB at close. No new target directory; the builds went into the shared one and were incremental. Retained evidence: 0.02 GiB under `D:\BuildArtifacts\lesscode\state-consolidation\r156` (two pinned binaries, the reports, the copies of the four files and the `1eb106f` source tree). Lesscode-owned caches were not measured again; the last figure is near 32 GiB.

## Next

1. The owner's decisions above.
2. The finder on a repository it was not written against, from `D:\Source`, every finding read and counted as right, wrong or missed.
3. `fatal` as the second case for stage 2: one place, then the bounded comparison.
4. Then as in handover 75: R140 as a history form of `bound-check`, the Java lowering, 3.1 with 5.2 and 3.4, 3.2 and 3.3, group 4, the rest of 5.

Local commits stay unpushed (`origin/main` at `a5df503`); a humanizer pass on `README.md` and `docs/driver.md` is due before any push. Slice B drift hardening (MIPROv2/GEPA) is unchanged, measured through R79, and awaits the owner's choice. Next research ID: R157.

## Amendment, 16:02: the owner's four decisions

1. `repair.rs:116` and `117` are accepted into the expected answer; the find step passes on the reply machine.
2. `match` arms as branch conditions: prototype first, in an experiment copy, before the Rust pack or its limit changes.
3. The finder's rules become a second data file per language beside the pack.
4. Next: the finder on a repository it was not written against. Chosen: `D:\Source\mmhmem` at `452fb7c`, preregistered in the prototype gate. Next research ID stays R157.

## Amendment, 16:05: the finder does not transfer to mmhmem

[R157](../docs/research/2026-11-18/23-finder-on-mmhmem.md). Unchanged finder on `D:\Source\mmhmem` at `452fb7c` (236 files, only read). Eight production cells by hand census: all eight declarations listed, accesses complete for one, three machines missed (`state` in `download_line.rs`, `stop` in `release_local.rs`, `max_scroll` in `ui/app.rs`), one machine reported that is none (`mismatch` in `release.rs`). Causes: clone under another name (3), receiver other than `self` (2), cell passed as a parameter (1), write in another file (1), condition on the lock's result (1). Six reports in test doubles are right. Evidence: `D:\BuildArtifacts\lesscode\state-consolidation\r157` (8 MiB, with `census.md`).

Next: revision 3 of the finder with name following, registered before it runs, scored against that census and against R156's rows and controls. Queued behind it: the `match`-arm prototype for the Rust pack, `fatal` as second consolidation case. Next research ID: R158. D had 175.7 GiB free; no build beyond the incremental one of R156.

## Amendment, 16:15: revision 3 matches the mmhmem census

[R158](../docs/research/2026-11-18/24-finder-follows-names.md), registered in `885e54e` before it was written. First scored run, nothing adjusted: three machines found, `mismatch` shared, all eight cells with exactly the census lines, reply machine and controls unchanged, `fatal` with line 382. New in this repository: `queue` (`runner.rs:123`) is a machine; lines 268 and 272 listed for it are wrong (the bound-name rule counts any later condition that mentions the name). Rules file: 30 lines, 2,008 characters. It is a fit, written with the census open.

Next: a third repository under `D:\Source`, revision 3 frozen, census by hand sealed before the finder runs. Then `fatal` and `queue` as the second stage-2 case; the `match`-arm prototype stays queued. Evidence: `D:\BuildArtifacts\lesscode\state-consolidation\r158` (8 MiB). Next research ID: R159. D free 175.7 GiB; builds incremental in the shared target.
