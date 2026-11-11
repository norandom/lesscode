# R88: Prototype gate for gradual refactoring: a step can be judged from two fault runs, and pinning first makes it pass

- Report date/time and timezone: 2026-11-11, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-11, 12:20 to 13:05 local (three batches 12:20 to 12:44, a fourth to 13:00; times
  from command output)
- Status: completed; the gate passes
- Spec: [gradual-refactoring-steps](../../../.kiro/specs/gradual-refactoring-steps/brief.md) (brief, not yet
  approved as requirements)
- Hypothesis ID and predecessor: the owner's question of 2026-11-11 after
  [R87](13-density-by-syntax-specimens-restated.md): is the code healthy and working after a change, and can
  refactoring be supported one step at a time
- Type: prototype gate, done by hand with the released commands and one comparison script

## Decision

A small refactoring step can be judged mechanically from what the tool already produces. Two runs of
`deconvolute run`, one before the step and one after, compared function by function, gave a verdict for
five steps without a judgement call, and refused the step that should be refused, for the right reason.

The gate also corrected its own expectation. The step that was expected to pass, replacing one function by
its simpler version, was refused: the new version has a second place where the order of two checks is not
fixed by any test. Two assertions pin that order; they pass on the code as it is and fail on the two planted
faults. Written first, they turn the refusal into an acceptance. That is the working order for a step: pin,
then change.

This is the answer to "healthy and working" that the tool can support today: after a step the project's
tests pass, and planted faults in the changed code are noticed at least as often as before. It is not a
proof that behaviour is the same.

Separate outcomes: feasibility of a step verdict. No model was called. Nothing was merged into this
repository's code.

## Hypothesis and falsification

From the brief, fixed before the runs: the gate passes if the verdicts can be produced from receipts without
judgement calls and step B is refused for the right reason; it fails if comparing fault runs across a step
needs manual matching. Expected in the brief: step A accepted with 10 of 11 faults noticed before and 10 of
12 after. That expectation was wrong in its conclusion (see step A).

## Frozen inputs and method

- Binary: `lesscode-core` 1.1.1 (release build at `65db66b`). Trees: exports of `native/` at `65db66b` under
  `D:\BuildArtifacts\lesscode\steps`, each building into itself.
- Runs: `deconvolute run` on `src/recording.rs` with `cargo nextest run --test recording`, two copies each.
  Configurations, run receipts and verdicts: `docs/experiments/gradual-steps/prototype/`.
- Verdict: `docs/experiments/gradual-steps/step_verdict.py`. Mutants are grouped by the function they lie in
  and functions are matched by name. A step is refused when a run is void, or when in any function the
  number of unnoticed faults rises, or the number of noticed faults falls while the function has no fewer
  faults than before.
- Steps: (A) `verdict()` replaced by the candidate's version; (B) code untouched, the two tests of
  `verdict()` removed; (pin) code untouched, one test added that fixes the order of two reasons; (C) the
  TypeScript and Java specimen candidates against their originals, from the receipts of R75.
- Not run: the "not harder to change" check of openidea-kiro, because `kiro-cli` is not installed here.

## Evidence and results

| Step | Tests on the branch | Noticed faults | Unnoticed faults | In the changed function | Verdict |
| --- | --- | --- | --- | --- | --- |
| A: simpler `verdict()` | 358 of 358 pass | 90 to 90 | 101 to 102 | 10 noticed, 1 unnoticed to 10 and 2 | refused |
| B: two tests removed | pass | 90 to 80 | 101 to 111 | 10 and 1 to 0 and 11 | refused |
| Pin: one test added | pass | 90 to 91 | 101 to 100 | 10 and 1 to 11 and 0 | accepted |
| C: TypeScript `degradation.ts` | pass (R75) | 15 to 5 | 1 to 0 | functions renamed or removed | accepted |
| C: Java `CSVPrinter.java` | pass (R75) | 78 to 78 | 12 to 11 | two functions differ by one each | accepted |

- Step A. The rewritten function checks three things in a row. Swapping the first two, or the second and
  third, changes only the order of the reasons it returns, and no test looked at that order. The original
  had one such gap, the rewrite has two. Size by syntax: 7,307 to 7,306 tokens, so the step simplifies the
  control flow and removes next to nothing. State writes: 62 to 63.
- Pinning. Two assertions on the order of the reasons were added to step A's tests: they pass on the
  rewritten code and fail on each of the two planted faults (checked by planting the two mutants by hand).
  With them the rewritten function has no unnoticed fault left.
- Step B is what a careless or over-eager change looks like: everything still passes. The fault run is the
  only check here that sees it.
- Step C, TypeScript. The rule accepts, and the table of functions shows its limit: the candidate renamed
  and merged functions, so the comparison by name has nothing to compare for most of them, and a function
  with ten faults in the original has no counterpart. The totals still say nothing got worse.

## What the gate showed about the design

- A step verdict needs no matching of individual faults. Per function is enough when names are stable; when
  a step renames, only the totals are comparable, and the verdict should say so.
- "Unnoticed faults must not rise" is a strict rule and a useful one: it refused a step a reviewer would
  have waved through, and pointed at the exact missing test.
- The order "pin first, then change" follows from the numbers, not from taste.
- A hazard in the driver was found: stored outcomes are keyed by the source file, the mutated text and the
  configuration, not by the tests. After tests change, a rerun in the same work directory would reuse stale
  classes. Until the key includes the tests, use a fresh work directory or `--no-reuse` after changing tests.
- Each run of this file took about twenty minutes with two copies. A step verdict that reruns a whole file
  is too slow for small steps in Rust; it should run only the functions a step touches, plus those that
  call them.

## Limits and interpretation

- One file of this repository and two foreign files; the foreign steps came from receipts made earlier with
  an older binary, the same on both sides of each comparison.
- Function matching is a line pattern in the prototype script; the product would use the pack's function
  boundaries.
- Three of the six functions of this scope (`probe`, `run_groups`, `arm_env`) have no noticed fault before
  or after any step, because they start processes and no test isolates them. A step verdict cannot protect
  code that nothing tests; it can only show that this is so. That is where isolation is needed.

## Tool/spec consequence and next gate

- The gate passes. The brief can go to requirements when the owner approves.
- For requirements: the step verdict as a command over two Git revisions; its strict rule; totals when
  functions are renamed; running only what a step touches; the outcome store keyed by the tests as well; a
  receipt keyed by the criteria of the step's spec, as executed evidence for openidea-kiro.
- The pinning tests of step A are not merged; whether to take step A into this repository, pins first, is a
  small real first case for the workflow.

## COBESY check

Outcome first: the gate passes, and the one expectation that was wrong is named with what the tool found
instead. Replay: binary, trees, configurations, receipts, verdicts and the script are in the repository.
Disproof: the gate's failure condition did not occur; step B was refused for the stated reason. What the
verdict cannot do (untested code, renamed functions, speed) is stated.
