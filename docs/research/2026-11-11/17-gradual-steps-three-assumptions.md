# R91: The three assumptions of the step design hold; two details of the design change

- Report date/time and timezone: 2026-11-11, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-11, about 15:05 to 16:10 local (durations below are from command output)
- Status: completed
- Spec: [gradual-refactoring-steps](../../../.kiro/specs/gradual-refactoring-steps/tasks.md), tasks 1.1 to
  1.4 (requirements, design and tasks approved by the owner on 2026-11-11)
- Hypothesis ID and predecessor: the "not yet shown" list in the design's prototype-gate section; follows
  [R88](14-gradual-steps-prototype-gate.md) and [R89](15-first-tracked-step.md)
- Type: bounded checks with product code, on this repository and two foreign projects

## Decision

Gate result: **proceed** to task group 2. All three assumptions held. Two details of the design are amended
(below); neither changes a requirement.

Separate outcomes: mechanism only. No step was taken with the tooling yet. No model was called.

## Hypotheses and how each could have failed

| # | Assumption | Would have failed if | Result |
| --- | --- | --- | --- |
| 1 | A fault run limited to named functions gives the same classes as a whole-file run | any of the selected faults had another class | held: 11 of 11 the same |
| 2 | A copy read from a commit builds and tests like a working tree | tests in the copy gave another result | held for the whole repository; **failed for a subdirectory** |
| 3 | A pin can be judged against one fault through the driver | one of the three outcomes could not be produced | held: all three |

## Method and results

### 1. Function filter (task 1.1, commit `1b6e91a`)

Run configuration field `functions` (per file, names). Functions come from the pack's boundary kind; the
name is the `name` field of the node. Run on `verdict` in `native/src/recording.rs` with the test command of
R89, against the whole-file run of R89 (`step1/pins-run.json`).

| | Whole file (R89) | Filtered |
| --- | --- | --- |
| Sites | 674 | 15 selected, 659 not selected |
| Mutants | 191 | 11 |
| Classes of the 11 in `verdict` | 11 killed | 11 killed |
| Time | about 17 minutes | 2.1 minutes |

The 11 were matched by line, column, kind and operator. Their hashes differ between the two runs because
the R89 clone had CRLF line endings and the working copy here has LF; see finding B.

### 2. Copies from a commit (task 1.2, commit `5543cea`)

`step/revisions.rs` reads a tree with `git ls-tree` and `git cat-file --batch` and writes the files. The
design said `git archive`; that needs a second tool to unpack. Only `rev-parse`, `merge-base`, `ls-tree`,
`cat-file` and `log` can be started.

| Project | Copied | Tests in the copy | Tests in the working tree |
| --- | --- | --- | --- |
| this repository, `native/` only | 134 files | **358 of 363 pass** | 363 of 363 |
| this repository, whole | 4,331 files, 103 MB, 4.5 s | 363 of 363 | 363 of 363 |
| commons-csv (Java), whole | 125 files | `CSVPrinterTest`: 168 run, 0 failures, 7 skipped | the same |
| OpenUltraCode (TypeScript), whole | 118 files | install, type check, 3 of 3 tests | the same |

The five failures in the copy of `native/` alone are tests of `driver_admission` that read fixtures under
`docs/`. No links or submodules occurred in the three repositories; they would be listed as skipped.

### 3. Pin against one fault (task 1.3)

New option `deconvolute run --only N`. The base (`ad75c0f`, whole repository, faults in `verdict` only) has
one unnoticed fault: number 2, line 864, `swap_next`; 10 of 11 are noticed, as in R88.

| Copy | Tests added | Result of `--only 2` | Reading |
| --- | --- | --- | --- |
| base code + tests of `e5dff41` (the pin of R89) | the real pin | killed | admitted |
| base code + a test that asserts 1 = 1 | useless | survived | rejected: passes on the fault |
| base code + a test that asserts 1 = 2 | wrong | void, exit 3: "the unmutated tests do not pass" | rejected: fails on the unchanged code |

Each run took between 1.3 and 2.1 minutes, most of it the first build in a fresh copy.

## Findings that change the design

- **A. The whole repository is copied.** A declaration's `root` cannot mean "the part to copy". Paths in the
  declaration are relative to the repository root and commands run there (for this crate:
  `--manifest-path native/Cargo.toml`).
- **B. Hashes are those of committed content.** The copies hold the bytes of the commit. A working tree with
  converted line endings has other bytes, so the verdict and the guard must hash blobs of the revision, not
  files on disk. This also makes the guard independent of the machine's line-ending setting.
- **C. `git archive` is not used** (see 2).

## Limits

- One function, one repository for assumptions 1 and 3.
- Functions are matched by name; two methods of one name in a file are selected together.
- The fault number of `--only` is only stable for the same configuration and the same file content. The
  step commands will pass the number from a base run they made themselves.
- A fresh copy builds from nothing each time. The step commands should reuse a copy's build between the
  test run and the fault run.

## Evidence

`docs/experiments/gradual-steps/gate2/`: `filtered-config.json`, `filtered-run.json`, and for the pin
`pin-{base,pin,useless,wrong}-{config,run}.json`. Tests: `native/tests/driver_functions.rs`,
`step_revisions.rs`, `driver_run.rs` (`single_mutants_can_be_run_by_number`). 366 tests pass.

## COBESY check

- Claims that can fail: the table above; one of them did fail in one form (subdirectory copy) and the design
  is changed for it.
- What a reader may not conclude: that a step check works end to end. That is task groups 3 to 6.
