# Testability, one step at a time

How `lesscode-core` makes a grown codebase safe to change without a big test-writing project first. This
is the reasoning behind the `step` commands; the commands themselves are in [steps.md](steps.md).

## The short version

You do not need a well-tested codebase to refactor safely. You need the few lines you are about to change
to be protected, and you need to know that they are. The tooling gives you three things for that:

1. A map of where your tests would notice a mistake and where they would not, per function.
2. A way to tell whether a new test protects anything, before you rely on it.
3. One answer after each change: do the tests still pass, and do they still notice mistakes in the changed
   code at least as often as before.

Everything else in this document explains why these three are enough for one step, and what they do not
give you.

## The situation this is for

A codebase has grown for years. Some of it is tested well, some of it is tested by accident, and some of it
is not tested at all. Nobody is sure which is which. Coverage reports say which lines ran during the tests.
They do not say whether a test would fail if a line were wrong.

In that situation a refactoring is a bet. The tests are green before and green after, and you still do not
know whether green means "nothing changed" or "nothing was looking".

The usual advice is to write tests first. That is correct and it does not help much, because it does not
say which tests, how many, or when you may stop. Teams answer it with a test-writing initiative that takes
a quarter, or they skip it and refactor on trust.

There is a third way: make the question small. Not "is this codebase tested" but "is this one function
protected well enough to change it today".

## The idea: ask the tests, with planted faults

To find out whether a test would notice a mistake, make the mistake and run the test.

The driver does this mechanically. It takes a function, changes one small thing (negates a condition, drops
a statement, swaps a comparison), runs your own test command against that changed copy, and records what
happened. Then it restores the copy and tries the next change. Your source tree is never touched; all of
this happens in copies.

Each planted fault ends in one of two useful states:

- noticed: at least one test failed. A real mistake of this kind, at this place, would be caught.
- unnoticed: every test passed. A real mistake of this kind, at this place, would go through.

An unnoticed fault is the precise form of "nothing was looking". It names a line and a kind of change that
your tests accept. If a refactoring made that change by accident, you would not hear about it.

This is mutation testing, and it is not new. What is specific here is what it is used for: not a score for
the whole codebase, but a yes-or-no for a single step.

## Three pieces

### 1. A plan: where would a step be safe

`step plan` runs the faults for the files you name and lists every function with its noticed and unnoticed
faults.

For one file of this repository, with one of its test files, the plan found 191 places to plant a fault.
The tests noticed 91 and let 100 through. Twelve functions had faults planted and none noticed. One
function, `verdict`, had 11 faults and 10 of them noticed.

That list changes the conversation. The twelve are not candidates for a quick cleanup, whatever they look
like. `verdict` is one test away from being fully protected. So the first step went there.

### 2. Pins: tests that are proven to protect

A pin is a test written to fix what the code does today, so that a later change cannot alter it unseen.

The difficulty with such tests is that a test can pass for the wrong reason. It can assert something that
is always true. It can exercise the code and check nothing. A reviewer can miss that, and a model writing
tests at volume will produce some of each.

So a pin has to earn its place. `step pin` takes a test and one named unnoticed fault and checks two
things in a copy that has the old code and the new test:

- the tests pass on the code as it is, and
- the tests fail once the fault is planted.

If both hold, the pin is admitted. If the tests fail on the unchanged code, the pin is wrong. If they pass
with the fault in place, the pin protects nothing, and the tool says so. We tried all three on real code:
the real pin was admitted, a test that asserted nothing was rejected, and a failing test was rejected.

Who writes the pin does not matter. A person can, a model can. The tool does not write tests. It decides
whether a test counts.

Pins are committed by themselves, before the change. A pin that arrives in the same commit as the change
was never shown against the old code, and the check refuses the step for that.

### 3. Seams: what stands in the way

Some functions cannot be pinned as they are. They start a process, read the clock, write a file, or call
five other functions that do. No small test can hold them still.

For every function in which no fault is noticed, the plan lists what it reaches out to: the names it calls
and the state it writes. That list is the work order for making the function testable. Each entry is a
place where a test needs a stand-in, or a reason to split the function, or a reason to leave it alone for
now.

The tooling does not intercept calls at run time. Standing something in for a dependency is done with the
means your language and test framework already have: an interface, a parameter, a fake. What the tool adds
is the list, and the honesty to mark such a function as out of reach in the step's declaration, with a
written reason, instead of pretending it is covered.

## One answer per step

A step is one small change on one branch. `step check` judges it.

First it looks at the commits, without running anything. Did the step touch files outside what it
declared? Did a pin and a change share a commit? Did the code grow beyond what the step allowed? Does it
write state where it said it would not? Any of these refuses the step in seconds.

Then it runs your tests on a copy of the code before the step and on a copy after it, and plants faults in
both. Only in the functions the step changed and in the functions that call them, which is what keeps
this to minutes: for `verdict`, 11 faults in about two minutes instead of 191 in about seventeen.

The rule is short. If in any function more faults go unnoticed after the step than before, the step is
refused, and the verdict names the function and the faults. Protection may go up. It may not go down.

The verdict is a file: accepted, refused, or not judged. "Not judged" is its own answer, for the case that
a run says nothing (a test command that could not start, a machine out of memory). It is never rounded to
accepted.

## Why an exit code and not a guideline

Rules that live in a document, a review checklist or an agent's instructions hold until someone is in a
hurry. That includes AI coding agents: a skill that tells an agent to check its work is advice, and advice
can be argued with.

So the boundary is an exit code. The check exits with 0, 1 or 2, and a hook or a pipeline acts on that
number. The first real step shows what this is worth. The change to `verdict` behaved identically on 480
input combinations and passed format and lint. It was refused anyway, by a size limit this repository had
set for that file long before. Nobody had to remember the limit.

The verdict is bound to the exact committed content of the code, the tests and the step's declaration. If
any of them changes afterwards, the verdict no longer matches and the step has to be judged again.

## What this is not

It helps to be plain about the limits.

- It does not generate tests. Tools that explore a function's paths and write tests for them exist; this
  is not one. Here a person or a model writes the test and the tool judges it.
- It does not detour calls at run time. It tells you where a stand-in is needed and leaves the how to your
  language.
- It does not prove that two versions of a function are equivalent. It shows what your tests and the
  planted faults show, no more. A fault the driver cannot plant is a gap the plan does not see.
- A function can look unprotected only because the configured test command does not run the tests that
  cover it. Configure the tests you rely on.
- Functions are matched by name. Rename a function and the check compares the file's totals instead, and
  says that it had to.
- Compiled languages are slow per fault. Limiting runs to the touched functions is what makes this usable,
  and a whole-file plan is still a coffee break.

We would be wrong about the approach if steps that the check accepts turned out to break behaviour that
the tests of the project were supposed to hold, or if developers found the plan's ranking no better than
their own guess at where to start. Both can be observed, and both would be worth a bug report.

## How to start, small

1. Pick one file you want to change and the test command that covers it. Run `step plan`. Read the list;
   do not act on all of it.
2. Pick one function with few unnoticed faults. Write one test for one of them. Run `step pin`. Seeing a
   pin admitted or rejected teaches more about your tests than the documentation can.
3. Commit the pins. Make the change in a second commit. Run `step check`.
4. If it is refused, read the reasons. A refusal is the tool doing its job, and it costs a few minutes
   instead of an incident.

After a handful of steps the plan for that file looks different, and that difference is the record of what
you made safer. Nothing in this asks you to stop feature work or to reach a coverage number first.

## A note on names

This work was sketched with two older tools in mind: Pex, which generated tests by exploring a program's
paths, and Moles, which let tests replace calls that could not otherwise be isolated. We kept their two
questions (what tests do I need, and what stands in the way of testing this) and answered them
differently: tests are judged instead of generated, and obstacles are listed instead of detoured. When
this project says "Pex and Moles" it means that pair of questions, not those tools.

## Status

`step plan`, `step pin` and `step check` exist and are tested. The guard that refuses a push without a
matching verdict, and the ledger that records steps release by release, are being built. See
[steps.md](steps.md) for the commands and the spec under
[.kiro/specs/gradual-refactoring-steps](../.kiro/specs/gradual-refactoring-steps/requirements.md) for the
requirements.
