# R62: Batches now run in parallel worker copies with scaled time limits: Java 1,875 s to 178 s, TypeScript 412 s to 70 s, Python 97 s to 32 s, every mutant classified as in the serial run; Rust stays serial

- Report date/time and timezone: 2026-11-10, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-10, about 13:45 to 15:15 local
- Status: completed
- Spec: [.kiro/specs/agnostic-annotation-driver/prototype-gate.md](../../../.kiro/specs/agnostic-annotation-driver/prototype-gate.md), Amendment 9
- Hypothesis ID and predecessor: operational change after [R61](07-annotation-driver-gate-result.md); no hypothesis about the mechanism
- Type: exploratory (apparatus only; verified against the committed serial receipts)

## Decision

The batching strategy of attempt 1 is replaced. Mutants of a scope now run
concurrently in independent worker copies with time limits scaled from the
unmutated run, long-lived test containers, and one shared outcome store. On
Python, TypeScript and Java every mutant gets the same class as in the
committed serial receipts of R60. Rust is kept at one worker, because
concurrent builds of that scope do not fit in the machine's memory.

This changes how long a batch takes and nothing else. It is a setup result:
no new evidence about the mechanism, no change to operators, packs, targets
or classification of real test runs. The gate outcome of R61 stands.

## Hypothesis and falsification

The owner's observation: about 20% of the CPU was in use, so the batches
were slow because they were serial. Checked against the receipts of R60,
that was right, with two further causes: fixed time limits of 180 to 900 s
(nine timeouts cost 20 of the 38 minutes spent on Java and TypeScript), and
a fresh Maven container per Java mutant.

Acceptance, fixed before the runs: with outcome reuse switched off, a
parallel batch must give every mutant the class it has in
`results/d1/<scope>.json`. Any difference other than a timeout caused by the
new limits is a defect.

## Frozen inputs and method

Changes, all in `docs/experiments/annotation-driver/`:

| File | Lines | SHA-256 |
| --- | --- | --- |
| `harness.py` | 339 | `92429b593de0905f54633c963f4a160f49fbf1bc7aee43220bb8214676bd4694` |
| `pool.py` (new) | 265 | `55f687ce2aa179d807c7dbc7a484f1d91139e756e4e90b8faabbb537d9d2f8a4` |
| `run_arm.py` | 147 | `bacdc08f4fe79a0da942b46d3817be73b3f4a2e3632350ec441b959d25e5042c` |
| `scopes.json` | 45 | `6c60a49cd7f072caf461691938ecaed9f0e83cde974321434ebc3ab7698c2d33` |
| `compose.yaml` | 55 | `6fdbb76187b3cfdf0c981740b3da4e6ff6d97f78f9f1649062f9d2a0c5a36708` |
| `java-fast-test.sh` (new) | 22 | `7c01a56be5c83aa5a0acc986806349a7154c497ea5ec78be919dd72d8913947c` |

What changed:

1. **Worker copies.** A worker is a full replica of the scopes directory
   (`...\annotation-driver\workers\wK\<repository>`), cloned from the scope
   clone at the pinned revision, set up once (`pnpm install`, `uv sync`) and
   warmed with one unmutated run. Workers pull mutants from a shared queue. A
   worker whose baseline fails stops and its mutants go back to the queue.
2. **Scaled time limits.** A mutant gets five times what the unmutated run
   needed on that worker, with a floor per scope (Python 20 s, TypeScript and
   Java 30 s, Rust 120 s) and the old fixed limit as the ceiling.
3. **Java.** One long-lived container per worker. Each run compiles only the
   scope file with `javac` and runs the one test class with surefire
   (`java-fast-test.sh`), instead of Maven's `test` phase, which recompiles
   the module every time.
4. **One outcome store** shared by all workers and the serial path, written
   atomically.
5. **Errors are not results.** A test command that fails to start, or a
   build that runs out of memory, is class `error`: retried once, never
   stored, and reported in the batch summary. Before this, any non-zero exit
   counted as killed.
6. **Rust:** `max_workers` is 1.

The pool and the error handling were written by a delegated Opus agent at
the highest effort setting from a written specification. The session agent
reviewed the results, made the Java fast path, the Rust limit and the
resource-failure rule, and ran the final verification below.

Command: `run_arm.py --scope NAME --scopes-dir ... --all-sites --workers N`
with `AD_RESULT_CACHE=off`, same seed and cap as R60.

## Evidence and results

Final verification, with nothing reused (receipts under
[results/pool](../../experiments/annotation-driver/results/pool/)):

| Scope | Workers | Mutants | Serial run in R60 | Now | Class differences against R60 |
| --- | --- | --- | --- | --- | --- |
| Python | 6 | 40 | 97 s | 32 s | 0 |
| TypeScript | 6 | 28 | 412 s | 70 s | 0 |
| Java | 6 | 40 | 1,875 s | 178 s | 0 |
| Rust | 1 | 16 | 217 s | 207 s | 0 |

A second Java run with reuse on took 34 s with all 40 outcomes taken from
the store; the remaining time is each worker's baseline before and after.

Model-free checks after the changes: validator controls 76 of 76; engine
checks all passing, including the harness smoke test on the four scopes;
the two tool test suites pass (17 and 23 tests).

Intermediate measurements, kept because they show what did and did not help:

| Step | Java, 40 mutants |
| --- | --- |
| Attempt 1: serial, fresh container per run, 180 s limit | 1,875 s |
| Serial, long-lived container, scaled limit | 1,384 s |
| 4 workers, Maven `test` phase | 628 s |
| 6 workers, compile one file and run one test class, including first-time preparation of two workers | 286 s |
| Same, workers already prepared | 178 s |

Failed attempts and defects found:

- **Rust in parallel.** With four workers, 5 of 16 mutants came out
  `invalid`; with two workers, 2 of 16. Every such log shows
  `memory allocation of N bytes failed` or corrupt crate metadata: the
  concurrent test builds of `mmhmem` exhausted memory, and the failed build
  was recorded as a mutant that does not compile. Preparing four Rust
  workers also took 368 s. Rust is therefore serial, and those messages now
  classify as `error`.
- **A launch failure counted as a kill.** In the serial Java rerun one run
  ended after 0.12 s with "Access is denied." and exit code 5, and was
  recorded as killed. All 240 kills in the attempt 1 receipts were then
  checked against their logs: none is such a case, so R60 and R61 are not
  affected. The rule in point 5 above prevents it from now on.
- **A long-lived container alone barely helped Java** (1,384 s): Maven needs
  about 19 s per run even without the container start.

Cost of the workers: 2.3 GB of disk as measured after the unused Rust build
directories of four workers were removed (23 GB); eight idle test containers
were stopped at the end.

## Limits and interpretation

- Equality with the serial run was shown for the D1 mutants of these four
  scopes, once. Timing-sensitive tests could behave differently under
  parallel load on other scopes; the scaled limit is measured on each worker
  under that load, which mitigates but does not exclude it.
- The Java fast path compiles one file against classes built earlier. A
  mutant that changes a signature used by other classes would not be
  recompiled against them. The operators of version 0 do not change
  signatures; a later operator that does would need the full build.
- The evidence rule for kills applies to failing exits only. A test command
  that wrongly exits 0 without running anything would still count as
  survived; the baseline run before and after each batch is the guard
  against that.
- Rust remains the slowest scope at about 13 s per mutant. Limiting build
  jobs per worker might allow two workers; that was not tried.
- Changing the Rust and Java commands changed their store keys, so outcomes
  stored before this change are not reused for those two scopes.

Not part of this entry's result, recorded for attempt 2: the annotations
rejected in arm A (retained in `results/a`) were read against the syntax
tree. On TypeScript they are mostly calls that write state
(`mutationQueues.set(...)`, `await rm(...)`) marked as `state_write`, and
boolean expressions returned by predicate functions marked as
`branch_condition`; neither is covered by the taxonomy's node kinds. On Java
three `throw` statements were rejected because the guide says to leave off
the statement terminator while the Java statement node includes it. A call
statement as a write site, with "drop the call" as its operator, would also
give TypeScript a third kind of landed mutant.

## Tool/spec consequence and next gate

- Gate Amendment 9 records the new batching and the `error` class.
- A full D1 pass over the four scopes now takes about 8 minutes instead of
  43, and a repeat with reuse about one minute apart from Rust.
- Attempt 2 is still not started and still needs the owner's go; the
  proposal in R61 gains the three observations above.

## COBESY check

Outcome first: times before and after and zero class differences, in the
title. Replay: hashes, command and receipts given. Disproof: the acceptance
rule was fixed first; the Rust failure and the two defects are reported.
Facts are in the tables; the reading of the rejected annotations is marked
as outside the result. No claim about the mechanism is made.
