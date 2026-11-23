# R164: planted faults on the renderer of drift-rdp, before any rework

- Report date/time and timezone: 2026-11-23, 09:20 Europe/Berlin (registration; the time of commit `ec4ca6a`. I first wrote 09:40 here without reading the clock; corrected at 09:30, from the clock)
- Execution interval: 2026-11-23, 09:21:00 to 11:03:56 Europe/Berlin (from the run's start and end records)
- Status: completed
- Spec: [lesscode-metrics brief](../../../.kiro/specs/lesscode-metrics/brief.md) (a brief, not approved requirements)
- Hypothesis ID and predecessor: follows the census of 22 November (below) and [R161](../2026-11-18/27-batch-progress-dogfood.md)
- Type: preregistered

## Decision

The released `step plan` carries a subsystem of seven files: 268 planted faults in 1 hour 43 minutes.
Of the faults in code that is compiled on this platform, the tests notice 72 of 204. Four of five
rows hold; one fails: in one of the three copies the unchanged tests did not pass at the start, for
a reason the release does not keep. A statement I made before the run was wrong and is corrected
below: the tool plants no fault in test code. It plants faults in code that is compiled only on
another platform, 64 of the 268.

## Hypothesis and falsification

Written before the run. I have not run `step plan` on more than one file of a foreign project.

| # | Expectation | Rejected when |
| --- | --- | --- |
| 1 | `lesscode-core` 2.1.0 `step plan` takes the seven files in one configuration and ends with a plan that is not void | it refuses the configuration, ends void, or exits other than 0 |
| 2 | The unchanged library tests pass in the copies | they do not |
| 3 | Of the planted faults outside test modules, between 25% and 50% are noticed by the tests | the share is below 25% or above 50% |
| 4 | At least 10 functions have faults planted and none noticed | fewer than 10 |
| 5 | With three copies the run takes between one and four hours | shorter or longer |

Why 3 and 4: the subsystem draws through the graphics card and a native window layer. I expect its
tests to cover the arithmetic (colour conversion, frame bookkeeping) and little of the drawing.

No limit, exclusion or test selection is changed after the start to make an expectation hold. If
the run is void it is reported as void, with its reason, and repeated only after the cause is named.

## Frozen inputs and method

- Subject: `D:\Source\drift-rdp` at `09b4ce8` (21 November), as it is on disk; the run records the
  head and the dirty files under `src-tauri` when it starts. The repository is only read; the tool
  works in copies.
- Scope: the seven Rust files of `src-tauri/src/renderer` that hold code (`mod.rs` holds none).
- Tests: the crate's library tests, `cargo nextest run --locked --manifest-path src-tauri/Cargo.toml --lib --no-fail-fast`.
- Tool: `lesscode-core` 2.1.0, the released Windows binary
  (`D:\BuildArtifacts\lesscode\state-consolidation\r161\lesscode-core-2.1.0.exe`), command `step plan`.
- Configuration and start script: [config.json](../../experiments/lesscode-metrics/renderer-baseline/config.json),
  [run.ps1](../../experiments/lesscode-metrics/renderer-baseline/run.ps1). The script starts the Rust
  command and keeps its output; it measures nothing. Settings for recognising a fault that does not
  build are those of the earlier Rust runs in this repository.
- Three copies, each building into its own `target`.
- Output: `D:\BuildArtifacts\lesscode\lesscode-metrics\r164-renderer-baseline`.

Counted beforehand with `deconvolute mutate`, file by file (outputs in
`D:\BuildArtifacts\lesscode\state-consolidation\r164\renderer-mutants`): 268 faults can be planted.
17 of them, all in `display_path.rs`, lie inside that file's test module. My first count of these
said 0, because my script asked for a field the receipt does not have; the second count reads
`site.start_line`.

## Known gaps of the tool, named before the run

- ~~It plants faults inside a test module that sits in the same file as the code. Those 17 are
  reported apart and are not part of expectation 3.~~ Wrong; see the correction under the results
  (amended 2026-11-23, 11:05).
- The plan records how many faults are noticed per function, not which test noticed which.
- Nothing counts state holders or writable states; the state lens stays empty in this run.

## The census this follows (22 November, exploratory, not preregistered)

`deconvolute measure-scope` and the research finder on the subsystems of `drift-rdp` at `09b4ce8`,
test files left out (`D:\BuildArtifacts\lesscode\state-consolidation\r164\drift-rdp-09b4ce8\census.json`):

| Subsystem | Files | Functions | Call edges | Cells | Machines found |
| --- | --- | --- | --- | --- | --- |
| renderer | 8 | 70 | 91 | 13 | 3 |
| commands | 14 | 112 | 94 | 12 | 6 |
| store | 7 | 122 | 250 | 6 | 2 |
| rdp/folder_share | 5 | 87 | 135 | 2 | 0 |
| rdp, top level | 20 | 348 | 1,046 | 36 | 4 |

The same day, `measure-scope` on two of the single-file examples of 18 November showed nothing by
the measures of the subsystem trials: order flags 4 to 2 functions and no call edges; clipboard
lease 5 and 5 functions, call edges 0 to 2. The owner's conclusion: examples are reworks of whole
subsystems.

## Evidence and results

Outputs in `D:\BuildArtifacts\lesscode\lesscode-metrics\r164-renderer-baseline`: `start.json`,
`end.json`, `plan.log`, `plan.json`, `work\plan-run.json` (every fault with its place and class) and
`work\outcomes`. The three working copies, 12.9 GiB, were removed at 11:05.

| # | Expectation | Measured | Holds |
| --- | --- | --- | --- |
| 1 | `step plan` takes seven files and ends with a plan that is not void | exit 0, `incomplete: false`, 70 functions, 268 faults | yes |
| 2 | The unchanged tests pass in the copies | in two of three. Copy 1 stopped before any fault: "the unmutated tests do not pass in this copy" | **no** |
| 3 | 25% to 50% of the faults outside test modules are noticed | 72 of 268, 27%; of the 204 in code compiled here, 35% | yes |
| 4 | At least 10 functions with faults and none noticed | 21 | yes |
| 5 | One to four hours with three copies | 1 hour 43 minutes, on two copies | yes |

**Row 2.** The release keeps no output of the failed first run, so the cause cannot be read. Run by
hand in that copy after the run, the same command passed: 536 of 536. The first runs of the three
copies took 383 to 387 seconds each, side by side, while I also built other programs on the
machine. A test that fails under that load is my guess and is not shown. The other two copies ran
all 268 faults between them, 134 each, and each closed with the unchanged tests passing.

**A correction.** Before the run I wrote that 17 faults lie inside the test module of
`display_path.rs` and called that a gap of the tool. That was wrong. My script took every line after
the start of the test module for test code; the module ends at line 412, and the file goes on.
Counted with the real ends of the modules, no fault of the 268 lies in a test module. The first
count of this I had already got wrong once (0, from a field that does not exist). Two wrong counts
of one thing, both from scripts I did not check against the source.

**What is there instead.** 64 of the 268 faults lie in modules compiled only on Linux
(`native_layer.rs` lines 124 to 385, `display_path.rs` lines 520 to 578). On Windows that code is
not built, so no test can notice a fault in it: 63 are reported unnoticed and 1 as not building.
They cost a quarter of the run and say nothing.

Faults in code compiled on this platform, outside tests:

| File | Faults | Noticed | Share |
| --- | --- | --- | --- |
| `shared_frame.rs` | 42 | 34 | 81% |
| `frame_stores.rs` | 4 | 4 | 100% |
| `native_layer.rs` | 6 | 4 | 67% |
| `yuv.rs` | 30 | 18 | 60% |
| `native_layer/dcomp.rs` | 19 | 3 | 16% |
| `display_path.rs` | 11 | 1 | 9% |
| `gpu.rs` | 92 | 8 | 9% |
| all | 204 | 72 | 35% |

2 of the 204 did not build. The tests are sharp where the code is arithmetic and bookkeeping
(`shared_frame.rs`, `yuv.rs`) and nearly blind where it draws (`gpu.rs`), as expected. The 21
functions with faults and none noticed include Linux-only ones (`attach` alone has 31).

A fault took about 47 seconds, most of it the rebuild of the crate.

## What the run says about the tool

- It plants faults in code that the platform does not compile, and counts them as unnoticed.
- When the unchanged tests fail in one copy at the start, the run goes on with the others and ends
  with `incomplete: false`; that a copy dropped out is only in the list of workers. Release 2.1.0
  does not keep why. (After 2.1.0 a failing first run names the step and the end of its output;
  this run used the released binary.)
- It does not plant faults in a test module that sits in the same file as the code. I had claimed
  the opposite.

## Limits and interpretation

A planted fault that the tests notice shows the tests are sharp at that place. It does not show
that a rework keeps behaviour. The run is on Windows only.

## Tool/spec consequence and next gate

After the run: the "before" side of the faults-and-tests view of the page, and a measured cost for
a fault run on a subsystem, which the brief names as an open question.

## COBESY check

Expectations and their rejection are written before the run; the correction of my own count is
stated; the tool's gaps are named.
