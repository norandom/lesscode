---
name: untangle-state
description: Take one type or one subsystem of grown code through the loop of state, faults, tests and rework - count where state is held and in how many ways it can be written, find what the code reaches and what the tests see, write the missing tests, rework so that what is never reached cannot be written, and let `lesscode-core step` judge it. Use when a person wants scattered state made simpler in real code with numbers before and after. The skill calls commands and reads their output; it cannot accept a rework itself.
---

# Untangling state: measure, test, rework, judge

One turn of this loop takes a type whose state is spread over several fields and ends with a branch
that holds new tests, a rework, and a verdict from `lesscode-core step check`. It was first run on a
type of a remote desktop client on 23 November 2026; the records are R164 to R173 under
[docs/research/2026-11-23/](../../../docs/research/2026-11-23/). The step commands are described in
[refactor-in-steps](../refactor-in-steps/SKILL.md) and [docs/steps.md](../../../docs/steps.md); this
skill says what to measure before them and in what order.

**What is released and what is not.** `lesscode-core` (`deconvolute measure-scope`, `step plan`,
`step check`, `step record`, `step guard`) is released. The four programs that count and read state
(`state-holders`, `walk-gen`, `scope-walk`, `scope-watch`) are research programs in this repository,
Rust only, with rules for Rust only. Say so when you report. Build them with

```sh
cargo build --release --manifest-path docs/experiments/state-consolidation/e5-bound/Cargo.toml --bin <name>
```

and keep a copy of each binary you used, with its hash, beside the outputs.

## Rules that hold in every phase

- **Write down what you expect before you run anything**, with what would reject it. Then report
  each expectation as held or failed. A failed expectation is a result; do not rerun until it holds.
- **Never report a check as done unless it ran and you saw its output.** Take times from the clock.
- **Do not edit the person's checkout.** Work on a new branch in a separate worktree, and run
  anything that builds in copies outside the repository, not on a drive root. Remove build output
  after each run and say what you kept.
- **A refusal is an answer.** If `step check` refuses, fix the step. Do not raise the allowed
  growth, narrow the tests or move a function out of reach; that is the person's decision.
- **Rust tooling only.** If a command cannot do what a phase needs, stop and say which, before
  using anything else. A shell loop that starts the programs and collects their output is fine.
- Say things in words. "The tests notice 34 of 42 planted faults", not a record number.

## Phase 1: where the tests are sharp (`step plan`)

Run the project's own tests against planted faults for the files of the subsystem.

```sh
lesscode-core step plan --config run.json --work <work> --out plan.json
```

Read per file how many faults the tests notice. Before you trust a share:

- Leave out faults in code the platform does not compile (a module under `cfg` for another system).
  The tool plants them and counts them as unnoticed; they say nothing.
- Look at the list of workers in the run receipt. A copy in which the unchanged tests did not pass
  at the start runs no fault, and the summary does not say so.

A fault costs about one rebuild of the project. Three copies of a project that builds to 4 GiB take
12 GiB until you remove them.

## Phase 2: where the state is (`state-holders`)

```sh
state-holders rust.holders.json <root> <file>...
```

Per struct it lists the fields that hold state, which of them are changed after the value is built,
and in how many shapes the type can be written. Pick a type with several changed holders. Do not
pick one whose holders are all set once: that is a record, not a machine.

## Phase 3: what is reached, and what the tests see

Three sets, in this order of trust:

| Set | How | Means |
| --- | --- | --- |
| can be written | Phase 2 | every combination of the holders |
| reached | `walk-gen` for one type: a generated test calls its methods in every order | what the code gets into, for the values tried; a lower bound |
| seen by the tests | `scope-watch`: one statement, in test builds only, per function that takes the type by reference; then the project's own tests | what the tests cover |

`walk-gen` only works where the type's own functions change it. Where the state is changed from
outside the subsystem or sits behind a device, use `scope-watch` alone. Check that the tests give
the same result with the watching statements as without; if not, do not trust the readings.

For a record that a test writes by hand and passes in, a shape seen in a test is what the test
chose, not what the program gets into. Name the function at which each shape was seen.

Read the two gaps. Written and never reached: what a rework can remove. Reached and never seen:
where a test is missing.

## Phase 4: tests, from two sources, before any change

Commit tests by themselves, never together with code under test.

1. **From shapes no test saw.** Put the value into each such shape through its public calls and
   assert what a caller can observe there. Watch again: every reached shape should now be seen.
2. **From faults the tests still miss.** Run Phase 1 again on the file. Read each unnoticed fault
   and write down which of three it is, before writing a test:
   - a test is missing: write it;
   - the fault changes nothing a caller can see: no test can notice it, write none;
   - the fault changes something the interface does not show on one thread, such as the order of
     releasing a lock and waking a thread: note it for the rework.

Covering a shape and noticing a fault are different things. On the first run, four tests from
shapes covered every reached shape and noticed one fault in eight; two tests from faults noticed
three more. Use both.

## Phase 5: declare the step, then rework

1. Commit the declaration by itself. Take the commit **after** the tests as `base`: tests that sit
   in the same file as the code count as scope. Set `max_token_growth` now.
2. Rework in a commit that touches no test. Aim at the two gaps of Phase 3 and the notes of
   Phase 4: one value for what several fields said together, no case for what was never reached,
   no field that is set and never read, an order that the shape of the code enforces instead of a
   statement someone must remember.
3. Keep the public functions as they are unless the person decides otherwise. Say what the rework
   does not close.
4. Run the project's formatter, lints and tests yourself before the check.

## Phase 6: judge (`step check`, `step record`, `step guard`)

```sh
lesscode-core step check --declaration .kiro/specs/<name>/step.json --work <work>
lesscode-core step record --declaration .kiro/specs/<name>/step.json
lesscode-core step guard --branch <branch>
```

Accepted means exit 0 from the check and from the guard, and nothing else. Keep a refused verdict
as evidence, do not commit it, and fix the step. Where `kiro-cli` is installed, also run
`kiro-cli code check --changed <base>` and report what rose.

## Phase 7: the table

Report one table, before and after, with every row filled or marked not measured:

| Lens | Measure | Expected direction |
| --- | --- | --- |
| Size | tokens of the scope (from the verdict) | may rise a little; within what was declared |
| Structure | functions and call edges (`deconvolute measure-scope`) | not judged for one type |
| State | ways the type can be written; reached; seen by the tests; fields set and never read | falls |
| Local complexity | routines that rose (`kiro-cli`) | none above its limit gets worse |
| Behaviour kept | the project's tests, unchanged | same count passes |
| Protection | planted faults noticed and unnoticed, per function | no function worse |

Do not claim a gain in size or decisions unless the table shows one. On the first run the state
went from 16 ways to 6 and the file grew by 57 tokens.

## When to stop and ask

- **No test notices anything in the functions you want to change**, and they need a device, a
  window or a network to run. Tests cannot come first there. The code needs a cut first that
  separates what it decides from what it does to the device, and that cut cannot be checked by
  tests that do not exist. Show the person the functions and their fault counts and let them decide.
- The rework would change a public function.
- A copy's unchanged tests fail and you cannot say why.
- Storage falls below the project's limit.

## What this skill cannot show

- That two versions behave the same. Planted faults measure how sharp the tests are.
- Anything about two threads. The walks and the tests here run on one.
- That a tool can write the tests or the rework. On the first run both were written by hand.
