# Validation report: brownfield-test-stage

Written 2026-11-25, about 12:30 Europe/Berlin, for task 5.3. The agent that implemented the spec
validated it inline; there was no independent reviewer. Code at `7f0324a` (task 6.5); `7f9c889`
after it changed records and documentation only, and is the commit the runs below name.

## Validation Report

- DECISION: **GO**, with what stands against it named under "What stands against it".
- MECHANICAL_RESULTS:
  - **Tests, Windows: PASS.** `cargo +1.98.1 nextest run --locked --manifest-path native/Cargo.toml
    --test-threads 4 --no-fail-fast`, exit 0, 508 of 508, 1 skipped (the replay of a recorded run).
    Doctests: exit 0.
  - **Tests, Linux: PASS.**
    - The run: `dagger call evidence --source . --revision 7f9c889… export --path
      D:\BuildArtifacts\lesscode\native-evidence\linux-7f9c889`. Exit 0, each of its 23 steps 0.
    - Core suite 507 of 507, 1 skipped; model suite 85 of 85. Lints, format, doctests and the model
      budgets also ran inside it.
  - **Lints and format: PASS.** Clippy with `-D warnings` on all targets, exit 0; `fmt --check`, exit 0.
  - **Size budgets: PASS.** 103 files: 96 within, 7 disposed, none exceeded or unlisted. Two files
    stand close to their limits after this spec:
    - `operators.rs`: 3,022 of 3,024 tokens;
    - `protect_begin.rs`: 2,668 of 2,800.
  - **`tools\bin\kiro-cli.exe code check --changed 43ec698^`: PASS** ("fail-on error: pass"), exit
    0, over 21 files.
    - The 4 errors are pre-existing and unchanged: nesting in `token_swaps`, six parameters of
      `compare`, and two long test routines.
    - Of the 24 warnings, the ones in this spec's own code:
      - `pack.rs` holds 13 types and `protect_stage.rs` 10, against a default of 8;
      - `of_write_off` and `chosen` have slightly over 4 lines per statement;
      - `kind_names`, `compare` and `step::run` grew within their limits.

      The rest are similarities to the old experiment copies under `docs/experiments`.
    - All 21 files are mapped by no spec's File Structure Plan (INFO), the same as for earlier
      specs.
  - **TBD/TODO grep** over the 22 files of `native/` the spec changed: CLEAN.
  - **Secrets grep** over the same files: CLEAN.
  - **Smoke runs on real projects: PASS.** Each of the three stages ended with `step protect result`
    exit 0 after the owner's approvals:
    - `yuv.rs` of drift-rdp: one test, eleven labels written off
      ([R188](../../../docs/research/2026-11-24/09-protect-stage-yuv.md));
    - `shared_frame.rs` of drift-rdp, over the whole scope: three tests, four faults written off,
      one fix of the tool on the way ([R189](../../../docs/research/2026-11-24/10-protect-stage-shared-frame.md));
    - `cycles.py` of scitools-hook, Python, its tests in a Linux container: one test, one fault
      written off ([R190](../../../docs/research/2026-11-25/01-protect-stage-python.md)).
- INTEGRATION:
  - **Cross-task contracts: hold.**
    - `stage.json` is written by `begin` and read by `result`. A file of the earlier form, with
      `left_alone`, still reads, which a test shows.
    - The verdict's `unnoticed_before` is additive. `result` uses it only where it is present, and
      a verdict without it behaves as before (tested).
    - `write-offs.json` is read as committed on the branch only.
  - **Shared state: holds.** The commands keep no state of their own; everything they need is in
    committed files and the run configuration.
  - **Boundary audit: two crossings, both named in the design's amendments.**
    - The Rust pack's `skip_strings_in` belongs to `agnostic-annotation-driver` (6.2).
    - The verdict's `unnoticed_before` belongs to `step check` (6.5).

    Neither changes what an existing caller reads.
- COVERAGE:
  - **Requirements mapped:** all seven, with the amendments of 2026-11-24 (1.1, 1.2, 5.10, 7.1)
    and 2026-11-25, are met by ticked tasks.
  - **Coverage gaps:**
    - 3.8 (every language for which a plan can be made): shown for Rust and Python. TypeScript and
      Java were not run.
    - 5.9 (how long the faults take): the figure ran low.
      - Rust: 1.32 to 1.40 times on drift-rdp, now with a stated allowance of 1.4.
      - Python with a container: 1.73 and 2.7 times, because the plan's seconds per fault leave
        out a test command's own start (R190).
    - 7.3 (the owner's approval): it rests on two fields anyone can write. The owner asked why
      approvals come by a script, and stronger ways were offered and are not decided.
- DESIGN:
  - **Architecture drift: none beyond the dated amendments.** The design has three: 13:15 (the
    prototype), 18:40 of 2026-11-24, and 2026-11-25.
  - **Dependency direction: no violation.** The stage's modules sit under `step`, and the pack's
    rule under `adapters/driver`.
  - **File Structure Plan vs actual:**
    - the plan names `protect_begin.rs`, `protect_result.rs`, the skill and the guide;
    - added since: `protect_stage.rs`, `protect_write.rs`, `protect_says.rs`,
      `protect_numbers.rs` and `protect_receipt.rs`, split to stay within the budgets (records of
      groups 1 and 2);
    - not reflected in the plan.
- OWNERSHIP: LOCAL
- UPSTREAM_SPEC: N/A
- BLOCKED_TASKS: none. Every task of groups 1 to 6 is ticked; 5.3 is this report and the handover.
- REMEDIATION: none required for GO.

## The Linux run

From 12:06:16 to 12:18:10, 11 min 54 s, the first run on Docker Desktop after the owner's switch
from Podman, with cold images. Dagger CLI 0.21.11 (engine `registry.dagger.io/engine:v0.21.11`);
the module and Actions stay pinned to 0.21.8, as `docs/ci-validation.md` says. Kernel
`6.18.58.1-microsoft-standard-WSL2`, Docker Desktop's VM; no WSL distribution was used. The core
suite counts one test fewer than on Windows (507 against 508). Several test files carry tests for
one system only (`cfg(windows)` in `cli.rs`, `driver_interrupt.rs`, `recording.rs`,
`run_page_not_built.rs`); which test makes the difference was not matched. The change
of `.dagger/main.go` (`38d337e`) that the owner has not seen yet was in effect: it gives the
core's files the time of the run.

## What stands against it

None of these is a failed check; they are what the checks did not cover, or what they found.

1. **Two languages, one machine.** TypeScript and Java were not walked, and neither was macOS. The
   Python run needed its test command in a Linux container, which the project's suite demanded,
   not the stage.
2. **The time figure of `begin` runs low** where a test command has its own start-up. That has
   been measured, and nothing was changed for it.
3. **Approvals are two fields in a file.** The tool prints every approver and date, and the skill
   forbids an agent to fill them. That holds only as long as agents follow the skill.
4. **Noise of the fault packs costs the owner's reviews.**
   - Rust labels passed through a helper are still planted (4 in `yuv.rs`).
   - The Python pack swaps keyword arguments, which is always equivalent.

   Both were written off by hand; a rule for the second is proposed and not decided.
5. **A flaky test of a project can make the plan count a fault noticed.** The base run catches the
   case where they disagree (6.5). A false kill that repeats in both is not caught.
6. **The reviews of every task were inline**, by the agent that wrote the code.
7. **The design's File Structure Plan** does not list the five modules that were split off.

## Evidence kept

| Place | Size | What |
| --- | --- | --- |
| `D:\BuildArtifacts\lesscode\brownfield-test-stage\yuv` | 0.03 GiB | R188: plan, logs, receipts, frozen binary |
| `D:\BuildArtifacts\lesscode\brownfield-test-stage\stage-6` | 0.05 GiB | 6.2's plan of `yuv.rs`, R189: plans, pins, checks, results, three frozen binaries |
| `D:\BuildArtifacts\lesscode\brownfield-test-stage\stage-5.2` | 0.04 GiB | R190: the test image's Dockerfile and script, plans (one void), pins, check, results |
| `D:\BuildArtifacts\lesscode\brownfield-test-stage\validation-5.3` | small | the logs and receipts of this report |
| `D:\BuildArtifacts\lesscode\native-evidence\linux-7f9c889` | small | the Linux run's exported evidence |

The Docker image `lesscode-stage/scitools-hook-tests:2026-11-25` is kept on Docker Desktop for a
repeat of R190. It is lesscode's, and `docker image rm` removes it.
