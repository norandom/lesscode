# R164: planted faults on the renderer of drift-rdp, before any rework

- Report date/time and timezone: 2026-11-23, 09:20 Europe/Berlin (registration; the time of commit `ec4ca6a`. I first wrote 09:40 here without reading the clock; corrected at 09:30, from the clock)
- Execution interval: not run yet
- Status: planned
- Spec: [lesscode-metrics brief](../../../.kiro/specs/lesscode-metrics/brief.md) (a brief, not approved requirements)
- Hypothesis ID and predecessor: follows the census of 22 November (below) and [R161](../2026-11-18/27-batch-progress-dogfood.md)
- Type: preregistered

## Decision

Open. The run gives the "before" side of the protection lens for the first whole-subsystem example,
and says whether the released `step plan` can carry a subsystem of several files at all.

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

- It plants faults inside a test module that sits in the same file as the code. Those 17 are
  reported apart and are not part of expectation 3.
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

Not run yet.

## Limits and interpretation

A planted fault that the tests notice shows the tests are sharp at that place. It does not show
that a rework keeps behaviour. The run is on Windows only.

## Tool/spec consequence and next gate

After the run: the "before" side of the faults-and-tests view of the page, and a measured cost for
a fault run on a subsystem, which the brief names as an open question.

## COBESY check

Expectations and their rejection are written before the run; the correction of my own count is
stated; the tool's gaps are named.
