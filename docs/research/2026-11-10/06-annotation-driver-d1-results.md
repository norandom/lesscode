# R60: Arm D1 (generic operators at every eligible site, no model) meets the gated rows on Rust and Python, misses kind coverage on TypeScript and, on provisional labels, meaningfulness on Java

- Report date/time and timezone: 2026-11-10, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-10, about 11:10 to 12:15 local; Rust, Python and TypeScript batches 11:10 to 11:22, Java batch 11:43 to 12:15 after two unusable attempts
- Status: completed for arm D1; arms A and D2 not yet run
- Spec: [.kiro/specs/agnostic-annotation-driver/prototype-gate.md](../../../.kiro/specs/agnostic-annotation-driver/prototype-gate.md), Amendments 3 to 5
- Hypothesis ID and predecessor: H-AD1, [R59](05-annotation-driver-strategy-and-harness.md)
- Type: preregistered (targets and classification fixed in the gate before the batches)

## Decision

D1 is a first measured result, not a gate decision. On the gated rows it
passes for Rust and Python and fails one row each for TypeScript and Java.
The gate needs three of four languages including one held-out language, so
D1 with hand-written packs does not meet it as it stands. Arms A and D2 are
still to run; the gate decision is taken after them.

Separate outcomes: setup held (baselines passed before and after every
batch, every file restored byte for byte). Extraction and fidelity are not
applicable to this arm. Behavioral evidence: each mutant was classified by
the scope's own tests. Compression: not applicable. Adoption: none.

## Hypothesis and falsification

For this arm the prediction was that a model-free engine with one data pack
per language reaches, on each scope, at least 4 landed mutants in at least 3
kinds (yield) and at least 70% of landed mutants killed or labelled a real
test gap (meaningfulness). Both targets were fixed before the runs. The
prediction failed on yield for TypeScript and, with the labels below, on
meaningfulness for Java.

## Frozen inputs and method

Inputs as frozen in R59 (engine, harness, validator, packs, scopes) with the
Java time limit of gate Amendment 5. Command per scope:

```text
uv run python run_arm.py --scope NAME --scopes-dir D:\BuildArtifacts\lesscode\annotation-driver\scopes --out-dir OUT --all-sites
```

Selection: up to 40 mutants among the sites that have an operator, seeded
(20261110) and balanced across kinds. Classification as in gate Amendment 3e.
No model was involved. Receipts:
[results/d1](../../experiments/annotation-driver/results/d1/); test logs are
kept locally and not committed.

"Landed" below means every mutant that was not classified invalid, so
timeouts are in the denominator. That is the stricter reading of the
meaningfulness row and it is the one used for the verdicts.

## Evidence and results

| Scope | Role | Run | Killed | Survived | Timeout | Invalid | Kinds landed |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Rust `update.rs` | development | 16 | 11 | 5 | 0 | 0 | 4 |
| Python `ratelimit.py` | development | 40 | 35 | 5 | 0 | 0 | 4 |
| TypeScript `state.ts` | held-out | 28 | 11 | 6 | 1 | 10 | 2 |
| Java `Lexer.java` | held-out | 40 | 17 | 14 | 5 | 4 | 4 |

Review of the 30 survivors, by hand, by the same agent that designed the
mechanism. The labels are provisional until someone else has checked them.

| Scope | Real test gap | Equivalent | Undetermined |
| --- | --- | --- | --- |
| Rust | 4 | 0 | 1 |
| Python | 4 | 1 | 0 |
| TypeScript | 6 | 0 | 0 |
| Java | 7 | 4 | 3 |

Gated rows for D1:

| Scope | Yield (at least 4 landed in at least 3 kinds) | Meaningful share of landed (target 70%) |
| --- | --- | --- |
| Rust | met: 16 landed, 4 kinds | 15 of 16, 94% |
| Python | met: 40 landed, 4 kinds | 39 of 40, 98% |
| TypeScript | **not met**: 18 landed, 2 kinds | 17 of 18, 94% |
| Java | met: 36 landed, 4 kinds | **24 of 36, 67%**; 24 of 31 (77%) if timeouts are left out |

What the survivors are:

- Rust: dropping `deps.restart.command = Command::Update` and negating two
  conditions that decide what is printed are not noticed by the tests; one
  swap of two checks changes whether the user is asked before a refusal is
  reported. One swap could not be judged without knowing whether both
  conditions can hold together.
- Python: all five survivors are swaps of two adjacent checks. Four change
  which error is raised or which state is written when both conditions hold;
  one is equivalent because the empty string cannot match the second check.
- TypeScript: three safety checks in `state.ts` can be removed with all
  tests passing: the check that the state directory resolves inside the
  project root, the symlink check on the state path, and the rethrow that
  the symlink error passes through. Two more concern errors while backing up
  a corrupt state file, one a queue clean-up condition.
- Java: seven concern the recorded first end-of-line string, the escaped and
  ready flags of a token, and the order of an end-of-line check and trailing
  data. Four swaps are equivalent because the two conditions exclude each
  other. Three depend on a side effect of `isDelimiter` and were not resolved.

What did not land:

- TypeScript, 10 invalid: every one failed the type check. Dropping an
  assignment leaves a variable unassigned; negating a type guard breaks the
  narrowing the following code relies on; dropping a `throw` leaves a path
  without a return value. All three `state_write` mutants were among them,
  which is why only two kinds landed.
- Java, 4 invalid: three dropped assignments to `final` fields in the
  constructor and the negation of `while (true)`; the compiler rejects each.
- Java, 5 timeouts and TypeScript, 1: the mutant makes a loop run forever.

Failed attempts, kept on record: the first Java batch could not start
because the container engine's API pipe was missing; the second was stopped
by hand at a mutant that looped under a 900 s limit (gate Amendment 5). No
mutant result from either is used. During the second attempt two test
containers were left running and were stopped by hand.

## Limits and interpretation

- The survivor labels decide the meaningfulness row and were made by the
  mechanism's designer. For Java the verdict flips with three undetermined
  labels or with the reading of "landed". It should be read as "close to the
  target, not shown to meet it".
- A surviving mutant is a gap relative to the tests selected for the scope.
  For Java only `LexerTest` (37 tests) was run; the parser tests of the same
  project exercise the lexer much more and were not part of the scope. The
  Java survivor count therefore overstates gaps in the project.
- TypeScript's yield failure is partly a property of the scope: it has three
  kinds at all (no adjacent checks to swap) and three statement-level writes.
  It also shows a real limit of syntax-only operators in a typed language:
  10 of 28 mutants did not type-check.
- The TypeScript pack was edited after seeing the scope (Amendment 4d), so
  TypeScript D1 is not a clean held-out result.
- 40 mutants are a seeded sample, not all mutable sites: Python has 161 and
  Java 115.
- Nothing here measures what model annotation adds; that is arm A.

## Tool/spec consequence and next gate

- D1 shows the shared engine and data packs work end to end on four
  languages with a harness that restores its scopes, and that most landed
  mutants are either killed or point at behavior the tests do not check.
- It also shows where version 0 of the operators is too blunt: statement
  drops in typed languages, and `return` statements, for which there is no
  operator (17 of 33 Rust sites, 40 of 157 Java sites).
- Next: the counted runs with frozen guide version 1 (arm A twice per scope)
  and the model-written packs for TypeScript and Java (D2), then the gate
  decision. The survivor labels should be checked by the owner or an
  independent reviewer before that decision relies on them.

## COBESY check

Outcome first: two languages pass, two fail one row each, stated in the
title. Replay: frozen inputs, command, seed and receipts are given.
Disproof: the two failed rows are reported against targets fixed beforehand,
with the stricter reading used. Negative results and failed attempts are
kept. Facts (counts, classes) are in the tables; the survivor labels are
marked as judgement by an interested party. No claim is made beyond these
four scopes.
