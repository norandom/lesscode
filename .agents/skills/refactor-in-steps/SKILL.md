---
name: refactor-in-steps
description: Walk one small refactoring through lesscode's checked steps (plan, declare, pin, change, check, record, guard). Use when a person wants to change grown code a piece at a time and know by exit code whether the tests still protect it. The skill calls `lesscode-core step` commands; it cannot accept a step itself.
---

# Refactoring in checked steps

A step is one small change, on one branch, with one small spec. `lesscode-core step` judges it: do the
project's tests still pass, and do they still notice planted faults in the changed code at least as often as
before. The reference is [docs/steps.md](../../../docs/steps.md); the reasons are in
[docs/testability.md](../../../docs/testability.md).

**What this skill is and is not.** It is a walk through the phases with the commands to call. It is not the
boundary. A step is accepted when `step check` exits 0 and `step guard` exits 0, and by nothing else. Do not
report a step as accepted, safe or equivalent on your own reading of the code. If a command refuses, say
what it refused and stop or go back; do not argue the refusal away, and do not weaken the declaration to get
past it (a larger `max_token_growth`, a narrower test command, a function moved to `out_of_reach`) without
the person deciding that.

Exit codes of every `step` command: 0 accepted or done, 1 refused or rejected, 2 not judged, 3 bad input or
a void run. Gate every next action on the exit code.

## Before you start

- `lesscode-core` on `PATH`, a Git repository, a language with a pack (Rust, Python, TypeScript, Java).
- A work directory outside the repository, not a drive root. Copies and stored outcomes live there.
- The project's own test command, runnable from the repository root in a fresh copy of the repository.
  With Cargo and a global `CARGO_TARGET_DIR`, set `"env": {"CARGO_TARGET_DIR": "target"}`.
- No model is needed and the commands contact none.

## Phase 1: choose the step (`step plan`)

```sh
lesscode-core step plan --config run.json --work <work> --out plan.json
```

Read the table. Prefer a function with few unnoticed faults. A function marked "no fault noticed" is not
protected by the configured tests at all: either configure the tests that cover it, or write tests first, or
leave it out. For such a function the receipt lists what it calls and writes; that is where a test needs a
fake. Propose one function and one change to the person; keep it small enough to review in a minute.

## Phase 2: declare it

1. Create the branch `step/<name>` from the base.
2. Write the step's spec under `.kiro/specs/<name>/`: `requirements.md` with the few criteria the step must
   meet (what stays the same, what gets simpler), and `spec.json`.
3. Write `.kiro/specs/<name>/step.json` (fields in docs/steps.md, section 2). Name the base as a commit hash
   or a tag, never a branch. Put the function in `focus`. Tie criteria to commands in `criteria` where a
   command can show them.
4. Commit spec and declaration by themselves.

## Phase 3: pin before you cut (`step pin`)

For each unnoticed fault in the function that the change could hide behind, write a test that fails on it,
commit the test without touching the code under test, and ask:

```sh
lesscode-core step pin --declaration .kiro/specs/<name>/step.json --fault <n> --work <work>
```

Exit 0: the pin is admitted. Exit 1: it either fails on the unchanged code (the test is wrong) or passes on
the fault (the test does not pin it); the output says which. Fix the test, not the code. Not every unnoticed
fault must be pinned; say which remain.

## Phase 4: make the change

Change the code in scope only, in a commit that touches no test file. Run the project's formatter, lints and
tests yourself first; the check will run the tests again in copies.

## Phase 5: judge it (`step check`)

Everything must be committed. Then:

```sh
lesscode-core step check --declaration .kiro/specs/<name>/step.json --work <work>
```

- Exit 0: accepted. Go on.
- Exit 1: refused. Read the reasons in the verdict file. More unnoticed faults in a function means the
  change removed protection: add a pin (phase 3) or take the change back. Growth over the limit, a file out
  of scope, a pin in the same commit as the change: fix the step, not the limit.
- Exit 2: not judged. A run was void or incomplete; find out why (a command that could not start, a shared
  build cache, a timeout) and run again. It is not a pass.
- Exit 3: bad input, for example uncommitted work.

## Phase 6: record and guard (`step record`, `step guard`)

```sh
lesscode-core step record --declaration .kiro/specs/<name>/step.json
git add <verdict> <evidence> <ledger> && git commit
lesscode-core step guard --branch step/<name>
```

The guard must exit 0 on the committed branch before you call the step done. An override
(`step record --override --reason "..."`) is the person's decision, never yours; it is reported as
"overridden" everywhere.

Merging is the person's decision. Report: the verdict, the numbers per function before and after, the size
before and after, and what remains unnoticed.

## What no command checks

Say these to the person; do not fill them with your own judgement:

- Whether the change behaves the same. Planted faults measure how sensitive the tests are, not equivalence.
- Whether a pin asserts the right behaviour. A pin is admitted when it fails on the fault; it can still fix
  a behaviour nobody wants.
- Whether the step is worth taking, is small enough, or reads better.
- Whether an `out_of_reach` reason is true.
- Calls through values, and two methods of one name in a file, which the function matching does not tell
  apart.
