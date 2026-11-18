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

## Amendment, 16:30: the transfer test passed, and the reports of 17 November are amended

[R159](../docs/research/2026-11-18/25-finder-transfer-drift-rdp.md). Frozen revision 3 on `D:\Source\drift-rdp` at `7eb659e`, census sealed in `b62ac40` before the run: 21 of 24 cells with exact lines, 8 of 9 machines, 0 of 11 reported machines wrong. Misses: `try_lock` not in the rules (`operation`, `pending`), a `static` (`REGISTRY`). Outside the finder: `state: SessionState` in `rdp/session.rs`, a plain field. Evidence: `D:\BuildArtifacts\lesscode\state-consolidation\r159` (6 MiB; `census.md`, `scoring.md`).

On the owner's word the entries of 17 November (R129, R130, R133, R134), the state-boundary report and the docs index carry dated amendments saying what 18 November changed (`b1e8f7a`). The release claim stays deferred there: research binary, no released command.

Owner's phase decision now open: requirements and design for the find step (gate assessed `proceed` on the demonstrated subset). Follow-up experiments: state in plain enum fields; `try_lock` and statics with a rerun of all three censuses; `fatal` and `queue` as second stage-2 case; `match`-arm prototype. Next research ID: R160.

## Amendment, 17:46: owner's goal, first example, Linux still unverified

Owner, 17:35: the goal of the state work is simpler code and visible before-and-after examples, smaller code bases among them; `drift-rdp` is a candidate. Decisions: first case the clipboard lease of `drift-rdp`; examples under `docs/examples`, one folder each; delivery as a branch in `drift-rdp` (read-only rule lifted for a branch); Linux exit 137 deferred again.

[R160](../docs/research/2026-11-18/26-clipboard-lease-consolidated.md): `native_clipboard.rs`, three cells to one value, `if` conditions 5 to 3, 416 of 416 library tests before and after, clippy clean; registered point 4 missed (finder reports one machine after, not none). Branch `lesscode/clipboard-lease-machine` at `1a49d70` in `D:\Source\drift-rdp`, local, made in a worktree that is removed; its working tree stayed on `master`. Example: `docs/examples/clipboard-lease`. Evidence: `D:\BuildArtifacts\lesscode\state-consolidation\r160` (isolated target removed after the run).

Linux: a fourth Dagger run at `a096080` was killed with exit 137 after core test 203, 24 s in, with 4.7 GiB free in the Podman machine and no memory kill in its kernel log. Memory does not look like the cause. Not finished at the kill: `driver_run` tests `time_limits_invalid_mutants_and_errors_are_told_apart` and `unsafe_work_directories_are_refused_and_interrupted_copies_recovered`. A `kill -KILL` on a missing process group in the image does not take the shell down. Log: `D:\BuildArtifacts\lesscode\native-evidence\linux-a096080` (the memory sampler there failed and holds no numbers).

Next: more examples (`fatal` and `queue` in the runner; plain enum fields after a finder experiment). README link to the examples needs the humanizer pass before a push. Next research ID: R161.

## Amendment, 18:10: Linux kill found and fixed; 2.1.0 prepared

Cause of the exit 137: `kill -KILL -<group>` in `native/src/adapters/driver/process.rs` without `--`. procps 4.0.2 (Debian 12) reads a group number that starts with 1 as `-1` and signals every process. Confirmed in the image, fixed in `6912855`. Releases 1.1.0 to 2.0.0 have the fault on Linux. Linux through Dagger passed at `6912855` and again at the release preparation `964209d`: overall 0, 408 of 408 core, 85 of 85 model. Windows at the prepared version: fmt, clippy clean, 409 of 409. `48010b5` and `3db7caf` are thereby verified on Linux.

Owner agreed to a 2.1.0 described as it is (fix, `measure-scope`, `bound-check`, stricter `lesscode-model` check; state work as examples, no command finds a machine). Prepared in `964209d`: version in `native/Cargo.toml` and three locks, `CITATION.cff`, `.zenodo.json`, README badge, `docs/README.md`, `docs/releases/2.1.0.md`. Humanizer pass done on the release notes and the two example pages. Opener example: `docs/examples/README.md` and `docs/examples/order-flags` (made up, both versions compared on 6,015 steps). Main README intro is left for later, on the owner's word.

Not done, owner's: push of main, CI on five platforms, tag `v2.1.0`, approval of the release environment, signing keys. Evidence: `D:\BuildArtifacts\lesscode\native-evidence` (`linux-a096080` killed run, `linux-6912855`, `prep-2.1.0`). Slice B unchanged (R79, owner's choice). Next research ID R161; next handover 77.

## Amendment, 18:57: main pushed, v2.1.0 tagged, release waits for approval

On the owner's word: main pushed (`a5df503..ebeae80`, 77 commits). Hosted pipeline 44356412424 on `ebeae80` green on all six jobs (step guard, Linux through Dagger, Windows, macOS Intel and ARM, Linux ARM). Tag `v2.1.0` at `ebeae80` pushed. Release workflow 44358080653: all five builds succeeded; `Sign and publish release` waits for the owner's approval of the `release` environment. No GitHub release exists yet.

Owner's after publication: signatures, checksums and attestation of the ten archives, Zenodo; optionally a note on the 1.1.0 to 2.0.0 release pages for Linux users.

Next, asked by the owner: more real, obvious cases under `docs/examples`, found and judged with the project's own tools (dogfooding).
