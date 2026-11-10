# R64: Gate attempt 2: the rules-as-data form meets every mechanism row on all four languages, with packs the model wrote for the held-out ones; the apparatus ceiling is the open row

- Report date/time and timezone: 2026-11-10, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-10, about 16:45 to 17:40 local (counted calls 16:45 to 16:50, D1 batches 16:45 to 17:10, arm A and D2 batches 17:00 to 17:15, labels 17:05 to 17:35)
- Status: completed (second and last attempt of the gate)
- Spec: [.kiro/specs/agnostic-annotation-driver/prototype-gate.md](../../../.kiro/specs/agnostic-annotation-driver/prototype-gate.md), Amendments 9 to 12
- Hypothesis ID and predecessor: H-AD1, [R63](09-annotation-driver-attempt2-setup.md)
- Type: preregistered (inputs frozen in R63 and Amendment 12; targets and `proceed` rule unchanged since attempt 1)

## Decision

Arm D, the rules-as-data form with no model at run time, meets every
mechanism row of the gate on all four languages and all five scopes: with
hand-written packs (D1) everywhere, and with packs the model wrote for
TypeScript and Java (D2). Arm A, model-annotated sites, meets them on Rust,
Python and Java in both runs and misses the kind-coverage row in one of its
four TypeScript runs.

By the gate's decision rule the recommended form is D. The mechanism result
is `proceed`. One gated row is not a mechanism row and is not met by
measurement: the apparatus ceiling. The counted native apparatus stands
exactly at its no-growth cap of 7,173 canonical lines (R53), so a production
port needs either an owner-reviewed amendment of that cap or an equal
removal elsewhere. The gate text makes that an owner decision and not an
automatic pass. Until it is taken, and until an ordinary design approval,
no production tasks follow.

Separate outcomes: setup held in every batch (baselines, byte-exact
restore, no `error` mutants). Extraction: annotation validity at or above
80% on every scope and run. Behavioral evidence: each scope's own tests.
Compression: none measured, none claimed. Adoption: none.

## Hypothesis and falsification

H-AD1 held in attempt 2. The two predictions specific to the revision (R63)
both held: TypeScript lands mutants in three kinds on both of its scopes in
arm D, and the frozen guide reaches 80% valid annotations there. The
companion prediction for arm D held on every language. What could have
rejected it and did not: fewer than three kinds landed, a meaningful share
below 70%, a model-written pack not admitted, or packs that needed
scope-specific strings.

## Frozen inputs and method

As frozen in R63 and gate Amendment 12: engine, validator, harness, pool,
version 1 packs, five scopes, `guide-v2.md`, `pack-guide-v2.md`. Model: the
owner's endpoint, `claude-opus-5-5`, no sampling parameters, no extended
thinking. Batches ran in the worker pool with outcome reuse; arm A and D2
reuse D1 outcomes for identical mutants.

- D1: every mutant of every eligible site (no scope exceeds 200).
- D2: packs for TypeScript and Java written by the model from the pack
  guide and the Rust and Python packs; admission by `pack_check.py`.
- A: two annotation calls per scope, validated, all mutants of accepted
  sites.
- Labels: every survivor (106) was labelled by an independent Opus agent
  per language group from the code, the selected tests and the change only.
  Those labels decide the meaningfulness row. The designer labelled a seeded
  sample of 30 as a cross-check.

Retained under
[results/attempt2](../../experiments/annotation-driver/results/attempt2/):
receipts, replies, parsed annotations, call records, model-written packs
and admission checks, survivor lists, independent and designer labels,
computed rows, ledgers.

## Evidence and results

### Arm D1 (hand-written packs, no model)

| Scope | Role | Mutants | Killed | Survived | Timeout | Invalid | Kinds landed | Real gap / equivalent / undetermined | Meaningful share of landed |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Rust `update.rs` | development | 66 | 45 | 21 | 0 | 0 | 4 | 20 / 1 / 0 | 65 of 66, 98% |
| Python `ratelimit.py` | development | 199 | 183 | 10 | 1 | 5 | 4 | 7 / 2 / 1 | 190 of 194, 98% |
| TypeScript `state.ts` | held-out | 74 | 28 | 19 | 1 | 26 | 3 | 19 / 0 / 0 | 47 of 48, 98% |
| TypeScript `verification.ts` | held-out | 66 | 33 | 17 | 0 | 16 | 3 | 12 / 5 / 0 | 45 of 50, 90% |
| Java `Lexer.java` | held-out | 189 | 123 | 39 | 12 | 15 | 4 | 32 / 7 / 0 | 155 of 174, 89% |

Targets: at least 4 landed in at least 3 kinds; at least 70% of landed
killed or a real test gap. All five scopes meet both.

### Arm D2 (packs written by the model)

Both packs were admitted on the first reply (7 of 7 checks; controls 19 of
19 for TypeScript, 21 of 21 for Java; 30 and 28 lines). On all three held-out scopes the
model-written packs make exactly the same sites eligible and yield exactly
the same mutants as the hand-written packs (70 / 74, 64 / 66 and 184 / 189
sites / mutants). The D2 batches therefore equal the D1 rows above for
TypeScript and Java, mutant for mutant.

### Arm A (model-annotated sites, two runs per scope)

| Scope | Run | Annotations | Valid | Mutants | Killed | Survived | Timeout | Invalid | Kinds landed | Meaningful share of landed |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Rust | 1 | 26 | 88% | 23 | 17 | 6 | 0 | 0 | 4 | 22 of 23, 96% |
| Rust | 2 | 24 | 88% | 23 | 17 | 6 | 0 | 0 | 4 | 22 of 23, 96% |
| Python | 1 | 68 | 93% | 83 | 75 | 8 | 0 | 0 | 4 | 81 of 83, 98% |
| Python | 2 | 71 | 92% | 86 | 78 | 8 | 0 | 0 | 4 | 84 of 86, 98% |
| TypeScript `state.ts` | 1 | 34 | 97% | 56 | 25 | 11 | 1 | 19 | 3 | 36 of 37, 97% |
| TypeScript `state.ts` | 2 | 32 | 100% | 56 | 24 | 12 | 1 | 19 | 3 | 36 of 37, 97% |
| TypeScript `verification.ts` | 1 | 21 | 81% | 28 | 19 | 6 | 0 | 3 | **2** | 25 of 25, 100% |
| TypeScript `verification.ts` | 2 | 23 | 83% | 29 | 19 | 4 | 0 | 6 | 3 | 23 of 23, 100% |
| Java | 1 | 32 | 100% | 54 | 40 | 11 | 3 | 0 | 4 | 50 of 54, 93% |
| Java | 2 | 31 | 100% | 53 | 40 | 10 | 3 | 0 | 4 | 48 of 53, 91% |

Validity meets the target everywhere. One run on `verification.ts` landed
mutants in two kinds only, so arm A does not meet the yield row for
TypeScript under the rule that all scopes and both runs must meet it.

Agreement between the two runs of a scope (Jaccard index over sites with
mutants), reported, not gated: Rust 0.71, Python 0.83, TypeScript 0.94 and
0.60, Java 0.82.

### Rows

| Row | Arm D | Arm A |
| --- | --- | --- |
| Validator integrity | met: 98 of 98 controls | same |
| Validity (80%) | not applicable (no annotations); packs admitted first reply | met on all five scopes, both runs |
| Yield | met on all five scopes | met on Rust, Python, Java, `state.ts`; not met in one of two runs on `verification.ts` |
| Oracle | met: every landed mutant classified by real tests, healthy baselines, no errors | same |
| Meaningfulness (70%) | met: 89% to 98% | met: 91% to 100% |
| Agnosticism | met: frozen pack guide yields admitted packs for both held-out languages, identical to the hand-written ones; no scope-specific string in engine, operators or packs; packs 26 to 33 lines | validity met on both held-out languages; yield missed once on TypeScript |
| Apparatus | **open**: production size not measured, and the counted apparatus is already at its cap | same |

### Duplicate mutants

Negating a condition that is a single equality test and swapping its
operator give the same program written two ways (`!(a == b)` and `a != b`).
The independent reviewers pointed this out. Counting such pairs once
changes no verdict:

| Scope, D1 | Raw | Pairs counted once |
| --- | --- | --- |
| Rust | 65 of 66, 98% | 63 of 64, 98% |
| Python | 190 of 194, 98% | 187 of 191, 98% |
| TypeScript `state.ts` | 47 of 48, 98% | 46 of 47, 98% |
| TypeScript `verification.ts` | 45 of 50, 90% | 41 of 46, 89% |
| Java | 155 of 174, 89% | 136 of 155, 88% |

### Labels

Independent labels: 90 real gaps, 15 equivalent, 1 undetermined of 106
survivors. The designer's blind cross-check covers 18 survivors (TypeScript,
Python, Rust); 17 agree. The one difference: the designer called swapping
the refusal check and the "Update now?" prompt in the Rust scope a real gap,
and the reviewer showed that the refusal cannot fire at that point in a real
run. The 12 Java survivors in the designer's sample were seen after the Java
reviewer's summary had arrived, so they are not counted as a cross-check.

What the reviewers found behind the gaps:

- Rust: most are terminal text the tests check only with `contains` or
  `ends_with`; one changes the exit code when no home folder is found, one
  restarts as `setup` instead of `update`. The integration test named in
  the scope does not call `update.rs` at all, so only the unit tests
  constrain it.
- TypeScript `state.ts`: all 19 are real, among them the safety checks
  already seen in attempt 1.
- Java: token flags and the first end-of-line string that `LexerTest` never
  reads, and format configurations no test exercises.

### What annotation adds, now that arm D runs every mutant

Every arm A mutant is also an arm D mutant, so annotation no longer finds
anything arm D does not run. It selects a subset that is killed somewhat
more often and is invalid less often:

| Scope | Killed share of landed, D1 | A, two runs | Invalid share of mutants, D1 | A, two runs |
| --- | --- | --- | --- | --- |
| Rust | 68% | 74%, 74% | 0% | 0%, 0% |
| Python | 94% | 90%, 91% | 3% | 0%, 0% |
| TypeScript `state.ts` | 58% | 68%, 65% | 35% | 34%, 34% |
| TypeScript `verification.ts` | 66% | 76%, 83% | 24% | 11%, 21% |
| Java | 71% | 74%, 75% | 8% | 0%, 0% |

### Cost and time

Attempt 2: 12 counted calls (83,458 input and 43,418 output tokens) and 4
development calls (24,090 and 15,054). Both attempts together: 232,938
input and 114,714 output tokens, at most 12.10 USD at the assumed upper
bound of 15 and 75 USD per million tokens, against the cap of 20 USD. The
actual price was not looked up. The independent label agents are separate
from the endpoint and are not in this figure.

Batch times in the pool: Python 92 s, TypeScript 70 s and 94 s, Java 701 s,
Rust 1,395 s (serial, 66 mutants).

## Limits and interpretation

- **Typed languages.** 35% and 24% of the TypeScript mutants and 8% of the
  Java mutants do not compile. The token swaps added valid mutants, but
  negations of type guards and dropped statements still fail often. The
  rows are met in spite of this, not because it is solved.
- **Kind names are loose.** Twelve of the 14 Rust "write" survivors drop a
  line printed to the terminal. Counting kinds as a measure of breadth is
  weaker than it looks: the third kind on TypeScript comes largely from
  dropped calls.
- **Arm A's TypeScript miss** is one run landing in two kinds because the
  model did not mark any failure or order site that yields a mutant in that
  file; the other run landed three. With run-to-run agreement of 0.60 on
  that scope, a pass in both runs would also have been partly luck.
- **Scopes.** Five files from four repositories, three of them the owner's.
  The two TypeScript scopes share a repository. The hand-written TypeScript
  and Java packs were shaped with knowledge of those scopes; the clean
  evidence for agnosticism is D2, where the model's packs came out
  identical, which also means D2 did not test a pack that differs.
- **The model knows these grammars well.** A language whose tree-sitter
  grammar the model knows poorly was not tried; the admission checks would
  catch wrong node kinds, but a repair was never needed here, so the repair
  path is untested on real failures.
- **Labels.** Independent of the designer, but made by the same model
  family, and for Java against one test class. One of 106 stayed
  undetermined.
- **Mutation, not compression.** This gate concerns the mechanism that
  measures test strength. It says nothing about how much code can be
  removed.
- **Production size.** The parts that would enter the native core are
  about 569 lines of Python (engine 273, validator 296) and 115 lines of
  packs, against 449 lines of specimen-bound operator bodies. A Rust port
  was not written or sized.

## Tool/spec consequence and next gate

- Recommended form for the design: one engine, one data pack per language,
  packs written offline by a model and admitted by deterministic checks, no
  model at run time. Annotation is optional and only ranks sites.
- The hard-coded operators can be replaced by this form in a design; that
  design must also settle the two things this gate leaves open: the
  apparatus ceiling, and whether the engine is ported to Rust or kept as a
  tool beside the binary.
- Operator work that would raise the value of the mechanism, none of it
  required by the gate: fewer non-compiling mutants in typed languages,
  an operator for failures returned as non-literal values, and removal of
  the duplicate negate and equality-swap pair.
- Next step in the spec workflow: requirements for
  `agnostic-annotation-driver`, then a design for the owner's approval. The
  second priority, hardening against inference drift, now applies to one
  small artefact per language (the pack) and to the optional annotation
  layer, with the agreement figures above as its baseline.

## COBESY check

Outcome first: arm D meets the mechanism rows everywhere, one row is open
and is the owner's. Replay: frozen inputs, receipts, replies, packs and
labels retained. Disproof: targets were fixed before attempt 1; the one
failed run of arm A and the open apparatus row are stated. Negative
findings (invalid rates, loose kinds, duplicates, untested repair path) are
in the limits. Facts are in the tables; the recommendation is marked as
such. No claim beyond five scopes, four languages and one model.
