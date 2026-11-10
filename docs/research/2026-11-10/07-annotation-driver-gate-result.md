# R61: Gate result `revise`: the shared engine and data packs work on Rust, Python and Java, TypeScript fails in every arm, and model-written packs match hand-written ones

- Report date/time and timezone: 2026-11-10, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-10, about 12:25 to 13:15 local (arm A calls 12:25, arm A batches 12:27 to 13:05, D2 trials and packs 12:40 to 12:55, D2 batches 12:55 to 13:12)
- Status: completed (first attempt of the gate; outcome `revise`)
- Spec: [.kiro/specs/agnostic-annotation-driver/prototype-gate.md](../../../.kiro/specs/agnostic-annotation-driver/prototype-gate.md), Amendments 3 to 8
- Hypothesis ID and predecessor: H-AD1, [R60](06-annotation-driver-d1-results.md)
- Type: preregistered (targets, arms, split and classification fixed before the runs; operational amendments dated)

## Decision

Gate outcome for attempt 1: **revise**. Not `proceed`, because TypeScript,
one of the two held-out languages, fails a gated row in every arm, and the
agnosticism row requires both held-out languages. Not `stop`, because the
mechanism meets every row on Rust, Python and Java in at least one arm, with
one engine, the same operators and packs of about 22 lines per language.

This permits a second, bounded attempt after the owner has seen the result.
It does not permit a design approval, production tasks or any change to the
released commands. The requirement to remove the hard-coded operators
stands; what replaces them is not yet demonstrated for all four languages.

Separate outcomes: setup held in every counted batch (baselines before and
after, byte-exact restore). Extraction (annotation validity) met the target
on three languages. Behavioral evidence is the scopes' own tests. No
compression claim. No adoption.

## Hypothesis and falsification

H-AD1 predicted that one frozen guide and generic operators give valid
annotations and landed, test-classified mutants on scopes outside the
specimens, including languages the guide was not developed on. It held on
Java and failed on TypeScript. The companion prediction for arm D, that the
same engine without a run-time model meets the rows, held on Rust and
Python, failed on TypeScript, and on Java depends on which 40 mutants were
drawn (67% and 78% meaningful, target 70%).

## Frozen inputs and method

Engine, validator, packs and scopes as frozen in R59; guide version 1 and
pack guide version 1 as frozen in R59 addition 3 and gate Amendment 7; Java
time limit of Amendment 5; outcome reuse of Amendment 8. Model: the owner's
endpoint, `claude-opus-5-5`, no sampling parameters, no extended thinking.

- Arm A: two annotation calls per scope with guide version 1, each validated
  and then run with `run_arm.py --annotations`.
- Arm D2: packs for TypeScript and Java written by the model from the pack
  guide and the Rust and Python packs, admitted by `pack_check.py`, then run
  with `run_arm.py --all-sites` (seed 20261110, cap 40).
- Survivors were labelled by hand by the session agent, which also designed
  the mechanism. Labels are provisional.

Retained: [results/a](../../experiments/annotation-driver/results/a/),
[results/d2](../../experiments/annotation-driver/results/d2/) (replies,
parsed annotations, call records, packs, admission checks, receipts,
ledgers). Test logs are local only.

Deviations, all recorded in the gate before or when they happened:

- The validator, `run_arm.py` and `annotate.py` were replaced (pack
  directory override, no change in behaviour) while the arm A batches were
  running. TypeScript run 2 ran the old code but its receipt lists the new
  code hashes; every other receipt is consistent. The override is not used
  in arm A, so the results are unaffected.
- The first Java arm A run 2 was stopped at 4 of 24 mutants to restart it
  with outcome reuse; 21 of its 24 outcomes and 29 of the 40 Java D2
  outcomes are reused from earlier batches.
- The TypeScript and Java pack calls each made a second attempt on false
  feedback, because the agent's read-only copy of the scopes was incomplete
  and the control check crashed. Those second replies are discarded. The
  first replies were rechecked properly and are the D2 packs.

## Evidence and results

### Arm A: model-annotated sites

| Scope | Run | Annotations | Valid | Mutants | Killed | Survived | Timeout | Invalid | Kinds landed | Meaningful share of landed |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Rust | 1 | 26 | 92% | 14 | 9 | 5 | 0 | 0 | 3 | 13 of 14, 93% |
| Rust | 2 | 26 | 88% | 13 | 9 | 4 | 0 | 0 | 3 | 12 of 13, 92% |
| Python | 1 | 51 | 90% | 43 | 41 | 2 | 0 | 0 | 4 | 43 of 43, 100% |
| Python | 2 | 59 | 97% | 53 | 50 | 3 | 0 | 0 | 4 | 53 of 53, 100% |
| TypeScript | 1 | 28 | **71%** | 19 | 10 | 3 | 1 | 5 | **2** | 13 of 14, 93% |
| TypeScript | 2 | 29 | **76%** | 20 | 10 | 4 | 1 | 5 | **2** | 14 of 15, 93% |
| Java | 1 | 26 | 88% | 22 | 14 | 7 | 1 | 0 | 3 | 19 of 22, 86% |
| Java | 2 | 26 | 96% | 24 | 16 | 6 | 2 | 0 | 4 | 21 of 24, 88% |

Targets: at least 80% valid; at least 4 landed in at least 3 kinds; at least
70% of landed killed or a real test gap. Rust, Python and Java meet all three
in both runs. TypeScript misses validity and kind coverage in both runs.

Agreement between the two runs of the same scope (Jaccard index over the
sites that have an operator), reported, not gated: Rust 0.93, Python 0.78,
TypeScript 0.95, Java 0.64.

### Arm D2: packs written by the model

| Language | Admission | Eligible sites, model pack against hand pack | Sites with an operator |
| --- | --- | --- | --- |
| TypeScript | admitted, first reply, 7 of 7 checks, 19 of 19 controls | 50 and 50, identical | 28 and 28, identical |
| Java | admitted, first reply, 7 of 7 checks, 19 of 19 controls | 157 and 157, identical | 112 and 115, 97% shared; the model gave no operator for `throw` |

| Scope | Mutants | Killed | Survived | Timeout | Invalid | Kinds landed | Meaningful share of landed |
| --- | --- | --- | --- | --- | --- | --- | --- |
| TypeScript | 28 | 11 | 6 | 1 | 10 | **2** | 17 of 18, 94% |
| Java | 40 | 22 | 13 | 1 | 4 | 3 | 28 of 36, 78% |

TypeScript D2 is mutant for mutant the same as D1. Java D2 meets every row;
Java D1, a different draw of 40 from nearly the same sites, reached 67%.

Leave-one-out trials on the development languages (not counted): with pack
guide version 0 the Python pack was admitted and the Rust pack request was
refused twice by the endpoint (`stop_reason: refusal`, category `cyber`, no
text); with version 1, which states the purpose, both were admitted on the
first reply.

### Rows by language and arm

| Language | Role | D1 | A | D2 |
| --- | --- | --- | --- | --- |
| Rust | development | all met | all met, both runs | not run (hand pack is the D form) |
| Python | development | all met | all met, both runs | not run |
| TypeScript | held-out | kind coverage not met | validity and kind coverage not met | kind coverage not met |
| Java | held-out | meaningfulness 67%, not met | all met, both runs | all met |

Cross-language rows: validator integrity met (76 of 76 controls). Oracle
met: every landed mutant was classified by real tests against passing
baselines. Agnosticism **not met**: the frozen guide does not reach the
validity and yield targets on TypeScript; the other parts hold (no
scope-specific string in guide, engine, operators or packs; packs 22 to 25
lines). Apparatus: estimated only, see below.

### Survivor labels added in this entry

Arm A and D2 produced survivors at six Java sites not reviewed in R60:
dropped writes of `token.isEscaped`, `token.isReady` (two sites),
`token.isQuoted` and `firstEol`, and a negated condition in the
escape-delimiter check. All six are labelled real test gap, by analogy with
the token flags and the first-end-of-line string that `LexerTest` does not
check. Every other survivor carries its R60 label.

### What annotation adds over applying the rules everywhere

| Scope | Killed share of landed, D1 | Killed share of landed, A (two runs) | Invalid share of mutants run, D1 | Invalid share, A |
| --- | --- | --- | --- | --- |
| Rust | 69% | 64%, 69% | 0% | 0% |
| Python | 88% | 95%, 94% | 0% | 0% |
| TypeScript | 61% | 71%, 67% | 36% | 26%, 25% |
| Java | 47% | 64%, 67% | 10% | 0% |

Annotation picks sites that are more often killed and less often invalid,
most clearly on Java, where it avoids the swaps of mutually exclusive checks
that make up most of D1's equivalent mutants. The price is a model call per
run and run-to-run variation in which sites are picked.

### Cost and size

21 model calls in total (12 annotation, 9 pack authoring): 125,390 input and
56,242 output tokens. At an assumed upper bound of 15 and 75 USD per million
tokens that is at most 6.10 USD against the cap of 20 USD. The actual price
of the configured model was not looked up.

Prototype size: validator 246 lines, engine 195, harness 225, batch runner
127, four packs 91 lines in total, annotation guide 54 lines, pack guide 63
lines; model-call and pack-admission tooling 866 lines. The part that would
enter the native core is the engine, the validator and the packs, about 530
lines of Python and data, against 449 lines of specimen-bound operator
bodies they would replace. A Rust port was not sized, so the apparatus row
is an estimate and is not counted as met.

## Limits and interpretation

- **Labels.** The meaningfulness verdicts rest on survivor labels made by
  the mechanism's designer. On Java they decide D1 and D2. They need an
  independent check.
- **Sampling.** Java D1 and D2 drew different sets of 40 from almost the
  same sites and landed on opposite sides of the 70% target. With these
  operators the Java result for arm D is near the threshold, not above it.
- **Scope tests.** For Java only `LexerTest` ran. Survivors are gaps
  relative to that file, not to the project.
- **TypeScript.** Two different failures. Kind coverage: the scope has no
  adjacent checks to swap, and all three statement-level writes fail the
  type check when dropped, so two kinds is the most any arm could land; this
  is partly a property of the scope chosen. Validity: 7 rejections in each run
  were `kind_mismatch`, which is a failure of the frozen guide and
  pack on a language they were not developed on, and is not a scope
  artefact. The rejected annotations were not analysed further in this
  attempt.
- **Typed languages.** Dropping a statement or negating a type guard often
  does not compile (TypeScript 10 of 28, Java 4 of 40 in arm D). Syntax-only
  operators are weaker there.
- **`return` sites.** There is still no operator for failure signalled by a
  return value, which is most failure sites in Rust and Java.
- **Development and held-out are not independent samples of languages.**
  Java and TypeScript are close to each other and to what the model knows
  well. Nothing here speaks to a language the model knows poorly.
- **Endpoint behaviour.** One request was refused until its purpose was
  stated. A driver that depends on a model must treat a refusal as an
  outcome of its own; the prototype recorded it as a failed attempt.
- **Removed arm.** Because arm C was dropped, nothing here shows whether an
  existing per-language mutation tool would do better on any one language.

## Tool/spec consequence and next gate

Established, within these four scopes:

- One engine and one small data pack per language produce test-classified
  mutants in four languages, with no language or project name in the engine.
- A model writes such a pack for a new language from two examples, and the
  deterministic admission checks accept it; the packs it wrote were the same
  as the hand-written ones for all practical purposes. That is the strongest
  result for horizontal application: adding a language was one model call
  and a set of checks, not code.
- Per-scope annotation improves the share of useful mutants and is not
  needed to get mutants at all.

Not established: TypeScript in any arm; arm D on Java above the threshold;
production size.

Proposed for a second attempt (for the owner to accept or change; nothing is
run before that):

1. Add type-preserving operators as pack data, so that typed languages land
   more than conditions: relational and logical operator swaps, and literal
   replacement for returned values, in place of statement drops where a
   drop cannot compile.
2. Give `return`-based failure sites an operator through the same pack
   mechanism.
3. Analyse the TypeScript `kind_mismatch` rejections, revise the annotation
   guide on the development languages only, and test on a second TypeScript
   scope admitted by the existing rule, alongside the current one.
4. Run arm D on all eligible sites where the scope is small enough, to
   remove the sampling effect seen on Java.
5. Have the survivor labels of both attempts checked by someone other than
   the designer before the decision.

The recommendation between the two forms, as far as attempt 1 supports one:
the D form (rules as data, model as offline pack author) as the base,
because it has no inference at run time and its only model output is a
small checked artefact; annotation as an optional layer that ranks sites.

## COBESY check

Outcome first: `revise`, with the failing language named in the title.
Replay: inputs frozen by hash, commands, seed, receipts, replies and packs
retained; deviations listed. Disproof: the failed rows are reported against
targets fixed beforehand, and the Java arm D result is given with both
draws. Failed attempts, the refusal and the agent's own setup error are
kept. Facts are in the tables; labels and the proposal are marked as
judgement. No claim is made beyond four scopes and one model.
