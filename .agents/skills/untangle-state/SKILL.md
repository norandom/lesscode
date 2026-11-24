---
name: untangle-state
description: Take one type or one subsystem of grown code through the loop of state, faults, tests and rework. Count where state is held and in how many ways it can be written, find what the code reaches and what the tests see, write the missing tests, rework so that what is never reached cannot be written, and let `lesscode-core step` judge it. Also covers code that cannot be tested as it stands (a device, a window, a network) and how to cut it into what it decides and what it does before any test exists. Use when a person wants scattered state made simpler in real code, with numbers before and after. The tool measures and judges, the agent writes, the person decides; the skill cannot accept a rework itself.
---

# Untangling state: measure, test, rework, judge

One turn of this loop takes a type whose state is spread over several fields. It ends with a branch
that holds new tests, a rework, and a verdict from `lesscode-core step check`. The loop was first run
on a type of a remote desktop client on 23 November 2026; the records are R164 to R173 under
[docs/research/2026-11-23/](../../../docs/research/2026-11-23/). The step commands are described in
[refactor-in-steps](../refactor-in-steps/SKILL.md) and [docs/steps.md](../../../docs/steps.md). This
skill says what to measure before them and in what order.

## Who does what

lesscode has no model inside it. A project adopts it through an agent that follows this skill.

| Who | Does | Does not |
| --- | --- | --- |
| The tool (`lesscode-core` and the programs named below) | measures, plants faults, judges by exit code | write a test or a change; call a model |
| The agent | reads the measurements, writes the tests and the rework, runs the commands, reports | accept its own work; change a limit; decide for the person |
| The person | chooses the scope, sets and changes limits, decides on public functions and merges, runs the real program where a device is needed | |

Bring the binary, this skill and the project's own test command. Nothing else depends on the
project the loop was first run on.

`lesscode-core` is released, with `deconvolute measure-scope`, `step plan`, `step check`,
`step record` and `step guard`. The four programs that count and read state (`state-holders`,
`walk-gen`, `scope-walk`, `scope-watch`) are research programs in this repository. They are written
in Rust and have rules for Rust only. Say so when you report. Build one with

```sh
cargo build --release --manifest-path docs/experiments/state-consolidation/e5-bound/Cargo.toml --bin <name>
```

and keep a copy of each binary you used, with its hash, beside the outputs.

## Rules for every phase

- Write down what you expect before you run anything, and what would reject it. Afterwards report
  each expectation as held or failed. A failed expectation is a result. Do not rerun until it holds.
- Report a check as done only if it ran and you saw its output. Take times from the clock.
- Leave the person's checkout alone. Work on a new branch in a separate worktree. Run anything that
  builds in copies outside the repository, never on a drive root. Remove build output after each
  run and say what you kept.
- When `step check` refuses, fix the step. Raising the allowed growth, narrowing the tests or
  moving a function out of reach is the person's decision.
- Use the Rust tooling. If a command cannot do what a phase needs, stop and say which one, before
  you use anything else. A shell loop that starts the programs and collects their output is fine.
- Say things in words: "the tests notice 34 of 42 planted faults". A record number tells the
  person nothing.

## Phase 1: where the tests are sharp (`step plan`)

Run the project's own tests against planted faults for the files of the subsystem.

```sh
lesscode-core step plan --config run.json --work <work> --out plan.json
```

When a person is at the screen, add `--open`: it opens a page in their browser that follows the run
and ends as the report. Tell them it is there and that no file of their project is changed. Without a
person watching, add `--page` and give them the place it prints. If a copy stops at its first run, the
tool says whether a build ended or a test failed and names the file with the whole output
(`first-run-w<copy>-<attempt>.log` under the work directory); read that file before anything else.

Read per file how many faults the tests notice. Two things can spoil that share:

- The tool plants faults in code the platform does not compile, such as a module under `cfg` for
  another system, and counts them as unnoticed. Leave them out.
- A copy in which the unchanged tests did not pass at the start runs no fault. The summary does not
  say so; the list of workers in the run receipt does.

A fault costs about one rebuild of the project. Three copies of a project that builds to 4 GiB take
12 GiB until you remove them.

## Phase 2: where the state is (`state-holders`)

```sh
state-holders rust.holders.json <root> <file>...
```

Per struct it lists the fields that hold state, which of them are changed after the value is built,
and in how many shapes the type can be written. Pick a type with several changed holders. A type
whose holders are all set once is a record of findings and has no machine in it.

## Phase 3: what is reached, and what the tests see

| Set | How | What it means |
| --- | --- | --- |
| can be written | Phase 2 | every combination of the holders |
| reached | `walk-gen` for one type: a generated test calls its methods in every order | what the code gets into, for the values tried; a lower bound |
| seen by the tests | `scope-watch`: one statement, in test builds only, per function that takes the type by reference; then the project's own tests | what the tests cover |

`walk-gen` only works where the type's own functions change it. Where the state is changed from
outside the subsystem, or sits behind a device, use `scope-watch` alone. Check that the tests give
the same result with the watching statements as without. If they do not, the readings are worthless.

A record that a test writes by hand and passes in shows the shape the test chose. That is what the
tests feed in, which may differ from what the program gets into. Name the function at which each
shape was seen, so a reader can tell.

There are two gaps to read. Shapes that can be written and are never reached are what a rework can
remove. Shapes that are reached and never seen are where a test is missing.

## Phase 4: tests, from two sources, before any change

Commit tests by themselves, never together with code under test.

First, tests for the shapes no test saw. Put the value into each such shape through its public
calls and assert what a caller can observe there. Watch again; every reached shape should now be
seen.

Second, tests for the faults the tests still miss. Run Phase 1 again on the file. Read each
unnoticed fault and write down which of three it is, before you write a test:

- A test is missing. Write it.
- The fault changes nothing a caller can see. No test can notice it, so write none.
- The fault changes something the interface does not show on one thread, such as the order of
  releasing a lock and waking a thread. Note it for the rework.

Use both sources. On the first run, four tests from shapes covered every reached shape and noticed
one fault in eight. Two tests from faults then noticed three more.

## Phase 5: declare the step, then rework

1. Commit the declaration by itself. Take the commit after the tests as `base`, because tests that
   sit in the same file as the code count as scope. Set `max_token_growth` now.
2. Rework in a commit that touches no test. Aim at the two gaps of Phase 3 and the notes of
   Phase 4: one value for what several fields said together; no case for what was never reached; no
   field that is set and never read; an order that the shape of the code enforces, where today a
   statement has to be remembered.
3. Keep the public functions as they are unless the person decides otherwise. Say what the rework
   does not close.
4. Run the project's formatter, lints and tests yourself before the check.

## Phase 6: judge (`step check`, `step record`, `step guard`)

```sh
lesscode-core step check --declaration .kiro/specs/<name>/step.json --work <work>
lesscode-core step record --declaration .kiro/specs/<name>/step.json
lesscode-core step guard --branch <branch>
```

The step is accepted when the check and the guard both exit 0. Keep a refused verdict as evidence,
leave it uncommitted, and fix the step. Where `kiro-cli` is installed, also run
`kiro-cli code check --changed <base>` and report what rose.

## Phase 7: the table

Report one table, before and after, with every row filled or marked as not measured.

| Lens | Measure | Expected direction |
| --- | --- | --- |
| Size | tokens of the scope, from the verdict | may rise a little, within what was declared |
| Structure | functions and call edges, from `deconvolute measure-scope` | not judged for one type |
| State | ways the type can be written; reached; seen by the tests; fields set and never read | falls |
| Local complexity | routines that rose, from `kiro-cli` | none above its limit gets worse |
| Behaviour kept | the project's tests, unchanged | the same count passes |
| Protection | planted faults noticed and unnoticed, per function | no function worse |

Claim a gain in size or decisions only if the table shows one. On the first run the state went from
16 ways to 6, and the file grew by 57 tokens.

## When no test can come first: the cut

Some code cannot be tested as it stands, because it draws, opens a window, or talks to a network
or a process. Phase 1 shows it as functions in which no fault is noticed, and the plan lists for
each what it calls and writes outside itself. Tests cannot come first there. The code is first cut
into what it decides and what it does to the device, and then the loop runs on the deciding part.

No test protects the cut. It is the only step in this skill of which that is true, so it gets more
care than the others.

1. Ask the person before you start. Show them the functions, their fault counts and what they
   reach out to. Tell them the cut needs one run of the real program by them. Go on only when they
   agree.
2. Sort every statement of the function into three kinds. It decides: it reads plain values and
   chooses. It does: it calls the device. It remembers: it writes a field or a local that the next
   turn of a loop reads.
3. Name what a decision reads as a plain value, such as sizes, flags and what was remembered. Keep
   every handle of the device out of it.
4. Move the decision, word for word, into a function from that value to a plain answer that names
   what to do. Keep the conditions and their order. The old function then gathers the values, asks,
   and does what the answer says.
5. Where a loop carries state in locals, make the locals one value, with one function that takes
   what the turn observed and returns what to do and what to remember.
6. Change nothing else in these commits. Better names, merged conditions and removed cases come
   later, under tests. Each commit should be small enough to read in a minute, build, and pass the
   tests there are.
7. Say what checked the cut and what did not. The compiler, the formatter, the lints, the existing
   tests and a diff that only moves text checked it. Planted faults did not: `step check` accepts
   such a step because nothing was noticed before or after. Declare the functions under
   `out_of_reach` with the reason, so the verdict says they were unprotected on purpose.
8. The person runs the real program once and looks at what you name. For a renderer that is: it
   draws, it redraws on a change, it survives a resize, and it falls back when it should. Record
   their answer as theirs.

After the cut the deciding functions take plain values, and Phases 1 to 7 apply to them as to any
other code. The part that still calls the device stays thin. Report its faults apart, as behind the
device and not testable here. They are not missing tests.

If the person does not want the cut, leave the file and take a type whose state the tests already
reach.

## When to stop and ask

- Before the cut above.
- The rework would change a public function.
- A copy's unchanged tests fail and you cannot say why.
- Storage falls below the project's limit.

## What this skill cannot show

Planted faults measure how sharp the tests are. They do not show that two versions behave the same.
The walks and the tests here run on one thread and say nothing about two. And on the first run the
tests and the rework were written by hand, so it is not shown that a tool can write them.
