# Refactoring in checked steps

Status: in development after release 1.1.1 (spec
[gradual-refactoring-steps](../.kiro/specs/gradual-refactoring-steps/design.md)). `step plan` exists; the
commands `pin`, `check`, `guard` and `record` follow.

A step is one small change on one branch with one small spec. The tooling answers by exit code whether the
project's tests still pass after the step and whether they still notice planted faults in the changed code
at least as often as before. It writes no test and no refactoring; it judges what a person or a model wrote.

Exit codes of every `step` command: 0 accepted or done, 1 refused or rejected, 2 not judged because a run
is incomplete, 3 bad input or a void run.

## 1. See where a step would help: `step plan`

```sh
lesscode-core step plan --config run.json --work ../work --out plan.json [--frame-spec frame.json]
```

`run.json` is a run configuration as for [`deconvolute run`](driver.md): the files, the test paths and the
project's own test command. The faults run in copies under `--work`, which must lie outside the source
root; the source tree is only read.

The plan lists every function of the configured files, ranked by unnoticed faults:

| Column | Meaning |
| --- | --- |
| noticed | planted faults the tests caught |
| unnoticed | planted faults the tests let through; each is a behaviour a change could alter unseen |
| tokens | size of the function, outside comments |
| writes | state writes in it; with `--frame-spec`, the receipt also counts those the specification does not allow |
| "no fault noticed" | faults were planted and the tests caught none: nothing in this function is protected |
| "no fault planted" | the pack offers no change for this function; the plan says nothing about it |

For a function with no noticed fault the receipt (`step-plan-v1`) lists what it reaches out to: the names
it calls and its state writes. That is where a test needs a fake, or where the function stays out of the
step.

What to do with a plan: pick a function with few unnoticed faults, write tests that turn them into noticed
ones (pins), and only then change the code.

## Example

A plan for `native/src/recording.rs` of this repository, with the tests of `tests/recording.rs` only
(2026-11-11, 191 faults, 13.8 minutes with three copies; receipt in
`docs/experiments/gradual-steps/gate2/plan-recording.json`): 91 faults noticed, 100 not. Ten functions have
no noticed fault, led by `replay_to_l2` (20 unnoticed) and `assemble_meta` (17); `verdict` has 11 noticed and
none unnoticed, which is why it was the first step. A function can look unprotected because the configured
test command does not run the tests that cover it; configure the tests you rely on.

## Limits

- Functions are matched by name. Two methods of one name in a file are counted together.
- A call is a name directly before an opening parenthesis. Calls through values are not seen.
- A whole file of a compiled language can take many minutes; the store under `--work/outcomes` makes a
  second plan with unchanged code, tests and configuration immediate.
