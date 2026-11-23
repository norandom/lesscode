---
name: untangle-state
description: Take one type or one subsystem of grown code through the loop of state, faults, tests and rework - count where state is held and in how many ways it can be written, find what the code reaches and what the tests see, write the missing tests, rework so that what is never reached cannot be written, and let `lesscode-core step` judge it. Also covers code that cannot be tested as it stands (a device, a window, a network): how to cut it into what it decides and what it does, before any test. Use when a person wants scattered state made simpler in real code with numbers before and after. The tool measures and judges, the agent writes, the person decides; the skill cannot accept a rework itself.
---

# Untangling state: measure, test, rework, judge

One turn of this loop takes a type whose state is spread over several fields and ends with a branch
that holds new tests, a rework, and a verdict from `lesscode-core step check`. It was first run on a
type of a remote desktop client on 23 November 2026; the records are R164 to R173 under
[docs/research/2026-11-23/](../../../docs/research/2026-11-23/). The step commands are described in
[refactor-in-steps](../refactor-in-steps/SKILL.md) and [docs/steps.md](../../../docs/steps.md); this
skill says what to measure before them and in what order.

## Who does what

lesscode is adopted through an agent, not through a model inside the tool.

| Who | Does | Does not |
| --- | --- | --- |
| The tool (`lesscode-core` and the programs named below) | measures, plants faults, judges by exit code | write a test or a change; call a model |
| The agent, following this skill | reads the measurements, writes the tests and the rework, runs the commands, reports | accept its own work; change a limit; decide for the person |
| The person | chooses the scope, sets and changes limits, decides on public functions and merges, runs the real program where a device is needed | |

The same split holds in every project: bring the binary and this skill, and the project's own test
command. Nothing here depends on the project the loop was first run on.

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

## When no test can come first: the cut

Some code cannot be tested as it stands: it draws, opens a window, talks to a network or a
process. Phase 1 shows it as functions in which no fault is noticed, and the plan lists for each
what it calls and writes outside itself. Tests cannot come first there. First the code is cut into
what it **decides** and what it **does** to the device; then the loop runs on the deciding part.

The cut is the one step in this skill that no test protects. Treat it accordingly.

1. **Ask first.** Show the person the functions, their fault counts and what they reach out to,
   and say that the cut needs one run of the real program by them. Go on only when they agree.
2. **Sort every statement** of the function into three kinds: it decides (reads plain values and
   chooses), it does (calls the device), it remembers (writes a field or a local that the next
   turn of a loop reads).
3. **Name what a decision reads** as a plain value: sizes, flags, what was remembered. No handle of
   the device in it.
4. **Move the decision, word for word,** into a function from that value to a plain answer that
   names what to do. Same conditions, same order. The old function becomes: gather the values, ask,
   do what the answer says.
5. **For a loop that carries state in locals,** make the locals one value with one function that
   takes what the turn observed and returns what to do and what to remember.
6. **Change nothing else in these commits.** No better names, no merged conditions, no removed
   case. Each commit is small enough to read in a minute, builds, and passes the tests there are.
7. **Say what checked it, and what did not.** The compiler, the formatter, the lints, the existing
   tests, and a diff that only moves text. Not planted faults: `step check` accepts such a step
   because nothing was noticed before or after. Declare the functions under `out_of_reach` with the
   reason, so that the verdict says they were unprotected on purpose.
8. **The person runs the real program once** and looks at what you name: for a renderer, that it
   draws, redraws on a change, survives a resize, and falls back when it should. Their word is the
   check. Record it as theirs.

After the cut the deciding functions take plain values, and Phases 1 to 7 apply to them as to any
other code: plant faults, write the tests, rework, judge. The part that still calls the device stays
thin. Report its faults apart, as behind the device and not testable here; do not count them as
missing tests.

If the person does not want the cut, leave the file and take a type whose state the tests already
reach.

## When to stop and ask

- Before the cut above.
- The rework would change a public function.
- A copy's unchanged tests fail and you cannot say why.
- Storage falls below the project's limit.

## What this skill cannot show

- That two versions behave the same. Planted faults measure how sharp the tests are.
- Anything about two threads. The walks and the tests here run on one.
- That a tool can write the tests or the rework. On the first run both were written by hand.
