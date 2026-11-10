# R73: The native driver runs the five gate scopes end to end; 554 of 556 mutants get the class the prototype gave

- Report date/time and timezone: 2026-11-10, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-10, about 22:05 to 22:37 local (Python and TypeScript first, then Rust and Java
  side by side)
- Status: completed
- Spec: [agnostic-annotation-driver](../../../.kiro/specs/agnostic-annotation-driver/tasks.md), task 8.1
  (Requirements 4, 5, 9.1, 9.2, 9.4)
- Hypothesis ID and predecessor: follows the engine parity of [R67](13-native-driver-engine-parity.md)
- Type: confirmatory

## Decision

`lesscode-core deconvolute run` does what the Python prototype's harness did, on the same five files and
with the projects' own tests: of the 556 mutants both ran, 554 received the same class. The two that differ
are Java mutants classed as timeouts under a tighter time limit in this run's configuration. No source tree
was changed. This is the first evidence that the single binary can replace the prototype for whole batches,
not only for listing sites and mutants.

Separate outcomes: behaviour of the native runner against the prototype's receipts. Test strength of the
five projects is what the classes describe; it was already reported in R64 and is not re-interpreted here.

## Hypothesis and falsification

Claim (Requirement 9.1 for batches): for every mutant that both the prototype (gate attempt 2, arm D) and
the native driver run, the class is the same. Refuted by any difference that the configuration does not
explain. Mutants only one side has must be the declared ones: repeats of a negation dropped by the driver
(R67), and the swaps of the two Python pairs adopted after the gate (R66 Amendment 3).

## Frozen inputs and method

- Binary: `lesscode-core` built from commits `50f80b6` (Python, TypeScript, Rust) and `2682b64` (Java, which
  needed the exit codes for an inner time limit added in `28f386b`). Shipped packs as embedded.
- Source trees: the gate's isolated clones under `D:\BuildArtifacts\lesscode\annotation-driver\scopes`, at
  the gate commits; the scope files' hashes equal the fixtures before and after each run.
- Configurations and receipts: `docs/experiments/annotation-driver/results/native/` (`*-config.json`,
  `*-receipt.json`). Commands are the gate's: `pytest` for Python, `tsc` and Node's test runner for
  TypeScript, `cargo nextest` for Rust, Maven in a container for Java.
- Reference: `docs/experiments/annotation-driver/results/attempt2/d1/*.json`. Mutants are matched by site
  position, kind and operator.

## Evidence and results

| Scope | Copies | Mutants, native | Same class as prototype | Only in prototype | Only native | Wall time |
| --- | --- | --- | --- | --- | --- | --- |
| Python `ratelimit.py` | 6 | 197 | 196 of 196 | 3 dropped repeats | 1 new swap (`is`) | 118 s |
| TypeScript `state.ts` | 6 | 71 | 71 of 71 | 3 dropped repeats | 0 | 107 s |
| TypeScript `verification.ts` | 6 | 55 | 55 of 55 | 11 dropped repeats | 0 | 88 s |
| Rust `update.rs` | 1 | 64 | 64 of 64 | 2 dropped repeats | 0 | about 23 min |
| Java `Lexer.java` | 4 | 170 | 168 of 170 | 19 dropped repeats | 0 | 814 s |
| Total | | 557 | 554 of 556 | 38 | 1 | |

Classes in the native runs: Python 181 killed, 10 survived, 5 invalid, 1 timeout; TypeScript 28, 18, 24, 1
and 30, 16, 9, 0; Rust 43 killed, 21 survived; Java 106 killed, 35 survived, 15 invalid, 14 timeouts. No
run was void, none had an error class, every closing baseline passed.

The two Java differences (a negated condition at line 232, a dropped statement at line 446): the prototype
classed them as killed, this run as timeouts. Here every mutant starts its own container and carries a
fixed inner limit of 30 seconds, while the unmutated test takes about 16 seconds that way; the prototype
used a long-lived container, where the test took a few seconds, and scaled its limit from that baseline
with a floor of 30 seconds. Relative to the unmutated run the limit was therefore much tighter here. That explanation was not tested by rerunning the two mutants
with a longer limit.

Safety: after all five runs the five scope files have the gate hashes, the clones show no change, and no
container of the Java run was left.

## Limits and interpretation

- Five files; the same ones the prototype was developed and judged on.
- The Java configuration differs from the prototype's (one container per mutant, fixed limit). It shows the
  driver can run a containerised project, and it shows that a fixed inner limit is a poor substitute for a
  limit scaled from the baseline.
- On this machine the container command is a wrapper script; the configuration calls it through PowerShell.
  Such configurations are not portable as they stand.
- Rust still runs serially at about 20 seconds per mutant; the driver did not change that.
- The run receipts name the driver as version 1.0.2, the version of the last release; no release contains
  the driver yet.
- Times are wall time on a shared machine that was also running two other batches.

## Tool/spec consequence and next gate

- Task 8.1 is done for all five scopes. Open from group 8: the five CI platforms with the driver are green
  for the test suite (8.2 partly; binary size and build time not yet recorded), and the final validation
  (8.3).
- Added while doing this: `timeout_exit_codes` in the run configuration, and a setup failure now reports
  its exit code and the end of its output (the first Java attempt failed because the container command is
  not an executable on this machine, and the message did not say so).
- Worth doing: a time limit for containerised commands that scales with the baseline, by letting the
  configuration refer to the computed limit.
- Group 7 (removing the old adapters) now has what it was waiting for: the driver reproduces the prototype
  on real projects.

## Addition, 2026-11-10, about 22:45 to 23:00: the Java explanation tested, 556 of 556

The explanation given above for the two Java differences was tested. The driver now hands a step's scaled
time limit to a command that asks for it (`{limit_s}` in an argument, commit `643de44`), and the Java
configuration passes it to the inner `timeout` (`java-scaled-config.json`). The whole Java scope was run
again in fresh copies: 170 of 170 mutants have the prototype's class (108 killed, 35 survived, 15 invalid,
12 timeouts; limits of 59 to 62 seconds from baselines of about 12 seconds). With that, every one of the 556
mutants that both sides ran has the same class. The first Java receipt with its fixed limit is kept beside
the second.

## Addition, 2026-11-10: platforms, binary size and build time (task 8.2)

- The existing CI matrix (Linux x86_64 through Dagger, Linux aarch64, Windows, macOS on Intel and on Apple
  silicon) is green with the driver and its tests at commit `2682b64`.
- Size of the Windows binary, release profile, built on this machine: 10,992,128 bytes at the tag `v1.0.2`
  and 13,256,704 bytes at `2682b64`, an increase of 2.26 MB (21%) for the driver and the two added
  grammars. The sizing spike (R65) had measured 1.5 MB for the grammars alone.
- Clean release build on this machine: 54 seconds at `v1.0.2`, 55 seconds at `2682b64`.

## COBESY check

Outcome first: 554 of 556, with the two exceptions named and their likely cause marked as untested. Replay:
binaries by commit, configurations and receipts in the repository, reference receipts named. Disproof: any
unexplained class difference; none was found beyond the two stated.
