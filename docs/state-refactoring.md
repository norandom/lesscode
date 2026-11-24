# Untangling scattered state, one checked step at a time

Status: a way of working, written down after doing it on this repository with release 2.1.0. It uses
the `step` commands ([reference](steps.md)) and `deconvolute analyze`. No released command finds the
scattered state for you; that part is reading, and a research program that helps with it is described
at the end.

The running example is real: the batch runner of `lesscode-core` itself, step `batch-progress`. Its
declaration, verdict and both versions are in [examples/batch-progress](examples/batch-progress/README.md).
If the idea is new to you, read the [made-up opener](examples/README.md) first.

## What you are looking for

State is scattered when one fact about where the program stands is kept in several places, and
different functions set and ask them. The usual signs:

- Two or three flags, locks or counters that sit next to each other in one struct or class.
- One function sets a value, another function asks it, and the two never mention each other.
- A rule you can only state by naming two of them together: "take a job only while the run has not
  stopped".

In the batch runner it looked like this. Workers shared three locked fields:

```rust
struct Shared<'a> {
    queue: Mutex<VecDeque<Job>>,
    results: Mutex<Vec<MutantResult>>,
    /// A failed restoration: the run stops and is void.
    fatal: Mutex<Option<String>>,
    // ...
}
```

and each worker asked two of them in turn before every job:

```rust
loop {
    if self.fatal.lock().unwrap().is_some() {
        break;
    }
    let Some(job) = self.queue.lock().unwrap().pop_front() else {
        break;
    };
    // ...
}
```

## Step 1: write the machine down before you touch the code

Say in words what the states are and what moves between them. If you cannot, you have not found the
machine yet.

For the batch runner: a batch is running or stopped. While it runs, a worker takes the next job, and
puts it back if it could not run it. Once it has stopped, no worker takes another job. At the end the
runner asks what is left, what was done, and why it stopped.

That is one value with three parts and one question, "the next job, unless the run has stopped".

## Step 2: one machine, one step

Take one machine per step. Make a branch `step/<name>` and a small spec that says what stays the same
and what gets simpler. Keep the change small enough to review in a minute.

## Step 3: declare the step, and build what the tests need

The declaration names the file, the functions and the project's own test command
([fields](steps.md#2-declare-the-step)).

- Put every function that sets or asks the state into `focus`. Their planted faults are run even
  where your change does not touch them. Here: `one`, `worker`, `run_batch`.
- Set `max_token_growth` before you write the change, and do not raise it afterwards.
- Name everything the tests start. The tests run in a fresh copy of your repository, and only what
  the commands in the declaration build exists there.

The last point is the one that goes wrong. Our first declaration named two test suites:

```json
"test": ["cargo", "nextest", "run", "--locked", "--manifest-path", "native/Cargo.toml",
         "--test", "driver_run", "--test", "driver_interrupt"]
```

With a `--test` filter Cargo builds those two test programs and nothing else. The suites start a
small example program, and it was not there. The fix is a `build` command:

```json
"build": ["cargo", "build", "--locked", "--manifest-path", "native/Cargo.toml", "--examples"]
```

`build` runs before the tests for every planted fault, so the example is rebuilt with the fault in
it. Use `setup` only for things that do not depend on the code under test, such as installing
dependencies; `setup` runs once per copy.

## Step 4: let the tool run the unchanged code first

Before any fault is planted, the tests run once on the base. If they do not pass there, the step is
refused and nothing else runs:

```
step batch-progress: refused
  the project's tests do not pass on the base
```

That took one minute and was right. Release 2.1.0 does not say why the tests failed; run the test
command yourself in the copy under the work directory (`base-run/w1/tree`) to see it. After 2.1.0 the
reason names the step that failed and the end of its output.

With 2.1.0, do not commit a refused verdict. We did, by adding the whole spec folder, and
`step record` then answered "not recorded: the verdict is refused" for the accepted verdict on disk,
until that one was committed. After 2.1.0 `step record` judges the verdict on disk, which is the one
it writes down.

## Step 5: read what your tests do not notice

The check plants small faults in the functions of the step and counts which ones the tests catch.
Before our change:

| Function | Faults noticed | Faults unnoticed |
| --- | --- | --- |
| `one` | 1 | 5 |
| `worker` | 8 | 5 |
| `run_batch` | 0 | 3 |

An unnoticed fault is a behaviour that a change can alter without any test failing. One of ours is a
transition of the machine itself: dropping the statement that puts a job back into the queue after a
failed run goes unnoticed.

This is where a pin belongs: a test that fails on that fault, committed by itself before the change
([`step pin`](steps.md#3-pin-before-you-cut-step-pin)). We did not write one for this step, and say
so: the put-back is as unprotected after the step as it was before.

## Step 6: make the change, in a commit that touches no test

```rust
/// How far the batch is.
struct Progress {
    queue: VecDeque<Job>,
    results: Vec<MutantResult>,
    /// A failed restoration or a stop from outside: the run ends and is void.
    fatal: Option<String>,
}

impl Progress {
    /// The next mutant to run; none once the run has stopped.
    fn next(&mut self) -> Option<Job> {
        if self.fatal.is_some() {
            return None;
        }
        self.queue.pop_front()
    }
}
```

The worker's loop asks once:

```rust
loop {
    let Some(job) = self.progress().next() else {
        break;
    };
    // ...
}
```

Run your formatter, lints and tests yourself first. The check runs the tests again in copies.

## Step 7: judge it

```sh
lesscode-core step check --declaration .kiro/specs/batch-progress/step.json --work ../work
```

```
step batch-progress: accepted
```

It took 14 minutes here. What the verdict says:

| Check | Result |
| --- | --- |
| Changes stay in scope | pass |
| Size | 2,761 tokens before, 2,757 after; allowed growth was 80 |
| Tests on the base and on the step | pass |
| Planted faults | in no function are more faults unnoticed after the step than before |
| Commands tied to the spec's criteria | pass |

| Function | Noticed before | Noticed after | Unnoticed before | Unnoticed after |
| --- | --- | --- | --- | --- |
| `one` | 1 | 1 | 5 | 5 |
| `worker` | 8 | 7 | 5 | 5 |
| `run_batch` | 0 | 0 | 3 | 3 |
| `next` (new) | | 1 | | 0 |

The one fault that left `worker` is the one now noticed in `next`: the question moved, and its
protection moved with it.

If the check refuses, read the reasons and fix the step. Do not raise the limit, narrow the tests or
move a function out of reach to get past it; that is the owner's decision, not the author's.

## Step 8: check that no routine got worse

`step check` counts the tokens of the file. It does not look at single routines. For that, this
repository uses `kiro-cli` from [OpenIDEA Kiro](https://github.com/norandom/openidea-kiro), with the
limits declared in `.kiro/code-rules.toml`:

```sh
kiro-cli code check --changed <base>
```

It compares every changed routine with itself at the base and blocks one that is above a limit and
got worse. On our step it did:

```
runner.rs: ERROR CODE_RATCHET routine `run_batch`: code_lines rose from 60 to 61, limit 60 (default)
```

We had already merged. `run_batch` was shortened afterwards. Run this before the record.

Expect the routine that now holds the decisions to go up. That is the trade: several small deciding
routines for one larger one, and the limit per routine keeps it honest.

## Step 9: record and guard

```sh
lesscode-core step record --declaration .kiro/specs/batch-progress/step.json
git add <verdict> <evidence> <ledger> && git commit
lesscode-core step guard --branch step/batch-progress
```

The step is done when the guard exits 0 on the committed branch. Merging is a person's decision.

## Step 10: count what changed

| Measure | Before | After |
| --- | --- | --- |
| Locked values the workers share | 3 | 1 |
| Questions a worker asks before a job | 2, under two locks | 1, under one lock |
| `deconvolute analyze`: branch conditions in the file | 13 | 13 |
| `deconvolute analyze`: declared gaps in the file | 3 | 1 |
| `kiro-cli`: decisions in the file (cyclomatic sum minus routines) | 24 | 24 |
| `kiro-cli`: cyclomatic of `worker` | 7 | 6 |

The conditions did not get fewer. The check on "stopped" moved from the loop into `next`. What
changed is that "stopped" and "nothing left" can no longer be asked apart.

## The same on a page

The step above was read from the terminal. After release 2.1.0, `step plan` and `step check` can show
a run in the browser: add `--open` to follow it while it goes on, or `--page` to get the report as one
HTML file at the end ([the options](steps.md#watch-the-run-in-a-browser---page-and---open)).

The page asks the question of the command. For `step plan` it is "Which of your functions do your
tests protect?", for `step check` "Does this change keep what your tests protect?". It says first
that no file of your project is changed. Under that the steps of the run stand in one row, with the
one that is going on marked, and the result as the last.

How the tool finds out what your tests protect, as the page tells it in four steps:

1. Your project is copied. Everything that follows happens in the copy.
2. One small thing is changed on purpose, such as a `<` into a `<=`, or a statement is left out.
   That is a planted fault.
3. Your tests run, the ones you already have, as they are.
4. If a test fails, the fault was noticed. If every test passes, it went unnoticed, and nothing
   would have told you about that mistake.

A function in which no fault was noticed is called unprotected. The table of step 5 above is on the
page as one square per function, and `run_batch`, with 0 noticed and 3 unnoticed, would be in the tab
"Unprotected". The page says there what to do with such a function: leave it if you do not plan to
change it; protect it first if you do, with a spec whose tasks are the tests; and where it talks to a
device, a window or a network, separate what it decides from what it does and test the part that
decides.

For `step check` the result lists the outcome, what to do next, and a tile per lens (size, structure,
state, local complexity, behaviour kept, protection) that says whether it moved as expected.

## What no command checked

- That the two versions behave the same. Planted faults measure how sharp the tests are.
- Behaviour under real concurrency. Before, a worker could see "not stopped", and another worker
  could stop the run before the first one took its job. One lock closes that gap. No test here
  observes it, in either version.
- Whether the step was worth taking.

## Finding the state: where research stands

A research program in this repository lists values of the kinds that hold shared state in Rust
(`Mutex`, `Cell`, atomics and a few more), who writes and reads them, and which conditions depend on
them. On this repository it named `queue` and `fatal` as two machines and `results` as shared. On a
remote desktop client it had never read, it found eight of nine machines a person had listed
beforehand. It does not see state in plain fields, and after our step it no longer sees this machine
either, because the state is in plain fields behind one lock now. It is not part of a release. See
the [research log](research/README.md), R156 to R161.
