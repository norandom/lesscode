# R80: Final validation of the agnostic driver: implemented as specified, one criterion in part

- Report date/time and timezone: 2026-11-11, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-11, 08:20 to 08:35 local
- Status: completed
- Spec: [agnostic-annotation-driver](../../../.kiro/specs/agnostic-annotation-driver/tasks.md), task 8.3
  (Requirements 9.4, 9.6); report:
  [validation-report.md](../../../.kiro/specs/agnostic-annotation-driver/validation-report.md)
- Hypothesis ID and predecessor: closes the work of [R65](../2026-11-10/11-annotation-driver-rust-sizing-spike.md),
  [R67](../2026-11-10/13-native-driver-engine-parity.md),
  [R73](../2026-11-10/19-native-driver-batches-on-gate-scopes.md) and
  [R75](01-specimen-controls-rerun-through-driver.md)
- Type: validation record

## Decision

The driver is implemented as the approved specification describes, in the one binary, for Rust, Python,
TypeScript and Java, and the specimen-bound rules are gone. One acceptance criterion is met only in part:
a driver that is killed from outside can leave a project command running (5.7), because it has no signal
handler. The validation was done by the implementer in the main conversation; it is not an independent
review. No release contains the driver.

Separate outcomes: this is a statement about the implementation against its specification. Test sensitivity
of any project, and equivalence of any rewrite, are not claimed.

## Hypothesis and falsification

Claim: every requirement of the spec has evidence in the test suite or in a dated research entry, and the
checks of the validation protocol pass. Refuted by a failing check or a requirement without evidence. Result:
the checks pass; requirement 5.7 has evidence only for normal ends and time limits and is reported as met in
part.

## Frozen inputs and method

Working tree at commit `c2a0a1b` plus the documents of this task. Protocol of
`.agents/skills/kiro-validate-impl`, run inline: test suite, lints, budgets, searches for leftover markers and
secrets, smoke runs of the commands, then a reading of requirements, design and boundaries against the code.

## Evidence and results

- 382 of 382 tests pass with cargo-nextest; format and lint checks are clean; the budgets check passes
  (twenty driver files, the largest 348 lines).
- No leftover marker and no secret pattern in the driver; no endpoint key in the committed tree.
- Smoke: `analyze` and seeded `mutate` on a Python fixture succeed; an unsupported language exits 3; the
  fixture is unchanged.
- Requirement by requirement: see the table in the validation report. Evidence entries: R67 (sites and
  mutants equal the prototype's), R73 (556 of 556 classes), R75 (specimen controls, losses recorded).
- Slice B (9.6): dated status in `.kiro/steering/roadmap.md` on 2026-11-11, with R76 to R79 run today.
- CI: recorded in the validation report.

## Limits and interpretation

- Not an independent review.
- Two things were found only because real batches were run today: a void run that did not say why, and a
  build directory shared between copies. Both are in R75. More such findings should be expected on projects
  the driver has not seen.
- The driver's operators cover five kinds of change. Seven of twelve hand-made controls of earlier gates were
  of other kinds.

## Tool/spec consequence and next gate

- The spec's tasks are complete. Open for the owner: a signal handler (5.7), the two library functions that
  report success without running anything, operators for changed arguments and literals in calls, a CI job
  for the guide optimiser, and a release.
- Slice B continues under `constraint-guided-compression`; its next step is the owner's choice between an
  annotation-layer baseline and closing the pack-authoring question.

## COBESY check

Outcome first: specification met, one criterion in part, named. Replay: commit, protocol and report named.
Disproof: a failing check or an uncovered requirement; one partial coverage found and stated. The review's
lack of independence is stated.
