# R67: The native engine reproduces the prototype on the five gate scopes; 38 of 594 mutants are dropped on purpose as repeats of a negation

- Report date/time and timezone: 2026-11-10, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-10, about 20:00 to 21:10 local
- Status: completed
- Spec: [agnostic-annotation-driver](../../../.kiro/specs/agnostic-annotation-driver/tasks.md), task group 3
  (Requirements 2.1, 2.2, 3.1 to 3.4, 6.1 to 6.3, 9.1 to 9.3)
- Hypothesis ID and predecessor: follows the sizing spike [R65](11-annotation-driver-rust-sizing-spike.md)
- Type: confirmatory (parity fixed as acceptance in the requirements)

## Decision

The engine inside `lesscode-core` (site rules, annotation validation, operators, mutant classification,
control replay) gives the prototype's results on the five gate scopes. Requirement 9.1 and 9.3 hold for the
code as committed. One difference is intended and bounded (Requirement 9.2): with the shipped packs the
driver leaves out a swap of a condition's own equality operator when the same site is also negated.

Separate outcomes: model fidelity (parity) is shown for the engine. No tests of the examined projects were
run by the native code; that is the batch runner of task group 6.

## Hypothesis and falsification

Claim: the engine, written into the crate's structure with a typed pack, produces the same sites, the same
mutants and the same control outcomes as the Python prototype. Refuted by any difference in a site field, in
a mutant's site, operator or hash of the mutated text, or in a control outcome, other than the declared
suppression.

## Frozen inputs and method

- Scope files: the five gate files, added as fixtures under `native/tests/fixtures/driver/scopes/` with the
  bytes the gate ran on (CRLF); their SHA-256 values equal those in the gate receipts.
- Reference: `native/tests/fixtures/driver/reference/*.json`, written by
  `docs/experiments/annotation-driver/native-spike/make_reference.py` from the frozen prototype run on those
  fixtures. The references record the hashes of the prototype files used.
- Test: `native/tests/driver_parity.rs`, run with cargo-nextest; no Python is needed to run it. A missing or
  changed fixture fails with "not run".
- Packs: the shipped packs. For the exact comparison of mutants the test removes the new key
  `negation_swaps`, which gives the pack the gate used.

## Evidence and results

| Check | Result |
| --- | --- |
| Eligible sites, all fields and order, five scopes | identical: 71, 211, 70, 64, 184 |
| Mutants (site, operator, hash) under the gate packs | identical: 66, 199, 74, 66, 189 |
| Annotation controls and mutant-classification controls | 98 of 98 give the recorded outcome |
| Accepted seed annotations | each is a site; every mutant of it is a real, parseable change |
| `admit-pack` with control replay, per scope | admitted (7 checks); a pack without condition rules is rejected by `controls` |
| Engine sources and packs | name no language, repository or gate file (the registry of shipped packs excepted) |

The Rust pack differs from the gate's pack by one key (`test_attribute_kinds`, replacing a literal in the
prototype's code). The identical sites on the Rust scope show the rule is equivalent there.

### The intended difference

Negating `a == b` and swapping its `==` for `!=` produce two mutants that say the same thing. The gate counted
both (R64 named this as a known flaw). The packs now list the token pairs for which this holds
(`negation_swaps`), and the driver drops such a swap when the token belongs to the condition itself and the
site is also negated.

| Scope | Mutants in the reference | Dropped as repeats | Remaining |
| --- | --- | --- | --- |
| Rust | 66 | 2 | 64 |
| Python | 199 | 3 | 196 |
| TypeScript `state.ts` | 74 | 3 | 71 |
| TypeScript `verification.ts` | 66 | 11 | 55 |
| Java | 189 | 19 | 170 |
| Total | 594 | 38 | 556 |

The test checks that every dropped mutant is such a swap at a site that keeps its negation, and that nothing
else differs.

## Limits and interpretation

- Parity is shown on five files. It says the port is faithful there.
- The suppression removes only exact repeats at the site's own operator. A swap deeper inside a compound
  condition is kept, and other equivalent mutants are not detected.
- Kill rates reported for the gate included the repeats. The share of repeats differs by scope (3% to 17%),
  so gate rates are not directly comparable with future native runs; the native batch of task 8.1 has to
  report both counts.
- The mutant comparison takes about nine seconds in a debug build because each site reparses its file. That
  is acceptable for a test and is a known cost for the batch runner to avoid.
- A model-written pack without `negation_swaps` is still admitted and simply does not suppress repeats.

## Tool/spec consequence and next gate

- Tasks 3.1 to 3.4 are complete. Control replay is now part of `deconvolute admit-pack`, which closes the
  part of task 2.2 that was left open.
- The start condition of the Slice B baseline that depended on the engine is met ([R66](12-slice-b-pack-authoring-drift-preregistration.md));
  the guide amendment and the owner's go remain. The guide must also mention `negation_swaps`, since a pack
  without it yields more mutants than the shipped pack and would fail the "same mutants" claim for a reason
  that is not drift.
- Next: task group 4, switching `analyze` and `mutate` to the driver.

## COBESY check

Outcome first: parity with one declared difference, with numbers per scope. Replay: fixtures, references,
generator and test are in the repository. Disproof: any other difference fails the test. A consequence for
the preregistered Slice B protocol is stated instead of left to be found during the run.
