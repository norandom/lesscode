# Prototype gate: agnostic annotation-guided driver

Status: **not-run**, 2026-11-10. Research preparation only. No production
tasks exist or are authorized. Execution needs the owner's go, because it
selects external scopes and spends model tokens. See the [brief](brief.md) and
[R56](../../../docs/research/2026-11-10/02-agnostic-annotation-driver-direction.md).

## Hypothesis

H-AD1: given one frozen annotation guide and a scope the guide was not written
for, a model returns located annotations that a deterministic validator
accepts, and generic operators applied at those sites produce mutants that
land and that the scope's own tests classify. The same guide works, unchanged,
on languages it was not developed on.

What could disappear if this holds: the specimen-bound operator bodies in
`native/src/adapters/{rust,java,typescript}/mutation.rs` (about 450 lines) and
the need to write such operators for each new language and codebase. What must
replace them: an annotation contract and validator, a generic operator set,
one node-kind table per language, and the guide. The saving is an estimate
until measured.

## Arms

| Arm | What it does | Role |
| --- | --- | --- |
| A | Annotation-guided: model annotates, validator checks, generic operators mutate | Mechanism under test |
| B | Existing fixed operators (`lesscode-core deconvolute mutate`) | Frozen control; expected to yield no mutants on these scopes |
| C | One existing mutation tool per language (cargo-mutants, cosmic-ray or mutmut, StrykerJS, PIT), restricted to the same files | Comparison; shows what a non-model tool already gives |

Arm A uses the existing endpoint seam (`LESSCODE_MODEL_ENV`, Opus). The model
identity, request parameters and raw responses are retained. No prompt
optimisation, no multi-turn repair and no human edit of annotations is
allowed inside a counted run.

## Scopes and admission

Four scopes, one each in Rust, Python, TypeScript and Java. Each must:

- be outside the experiment specimens (not the files or responsibilities used
  in R49 to R53, and not containing the substrings the fixed operators match);
- be a complete responsibility of 150 to 800 canonical lines with its own
  tests that pass in an isolated copy;
- have a recorded revision, license and dirty-tree hash.

Candidates to screen (not yet admitted): a module of `mmhmem` (Rust), a
Facdrone or `mvibe` subsystem other than the R53 specimen (Python), an
`OpenUltraCode` module other than the R53 specimen (TypeScript), a
`commons-csv` class other than `CSVPrinter` (Java). If a language has no
admissible scope, that language is reported as not-run; it is not replaced by
a synthetic fixture.

Development and held-out split, frozen before any model call: the guide is
written and adjusted using the Rust and Python scopes only. It is then frozen
by hash and run unchanged on the TypeScript and Java scopes. Only the
node-kind table may differ per language.

## Annotation contract, version 0

One JSON object per annotation: `file`, `start_line`, `end_line`, `kind`,
`anchor` (the exact source text of the annotated node), `reason`. Kinds:
`failure_state`, `branch_condition`, `error_order`, `state_write`,
`pure_shell_boundary`.

The validator accepts an annotation only if all of these hold, and records
the rejection reason otherwise:

1. the file is inside the frozen scope and is not a test or generated file;
2. `anchor` occurs at the stated lines, byte for byte;
3. the smallest syntax node covering the anchor has a node kind that the
   language's table allows for the annotation kind;
4. the site is not inside a comment or string literal;
5. it is not a duplicate of an accepted annotation.

## Research checklist

1. Screen and admit scopes; freeze inputs, the split, the taxonomy, the
   validator rules, the node-kind tables and the targets below. Record them in
   a dated research entry before the first model call.
2. Validator controls first, without a model: feed hand-made wrong
   annotations (wrong line, altered anchor, site in a comment, site in a test
   file, kind and node mismatch, duplicate) and a no-op mutant. All must be
   rejected or classified invalid.
3. Establish a healthy baseline: the scope's tests pass in the isolated copy
   used for mutation. Run arm B and arm C.
4. Write guide version 0 against the Rust and Python scopes. Freeze it.
5. Run arm A twice per scope with identical settings on all four scopes.
6. Review every surviving mutant by hand and label it equivalent, real test
   gap or invalid. This review is a manual intervention and is reported as
   such.
7. Count the prototype's lines by part (contract and validator, operators,
   per-language tables, guide, glue) and estimate what would enter
   `native/src`.

## Targets, falsifiers and decision

All targets are fixed here before execution. Changing one needs a dated
amendment made before the run it applies to.

| Dimension | Proceed condition | Otherwise |
| --- | --- | --- |
| Validator integrity | 100% of the hand-made wrong annotations rejected; the no-op mutant classified invalid | stop until fixed; no model run counts |
| Validity | On each scope, at least 80% of returned annotations pass validation, and no rejected annotation is ever applied | revise the guide (development scopes only) or the contract |
| Yield where the control has none | On each scope, at least 4 landed mutants covering at least 3 kinds, while arm B produces 0 | revise or stop |
| Oracle | Every landed mutant is classified by the real tests against a healthy baseline in the same copy; harness errors and timeouts are not kills | gaps block |
| Meaningfulness | At least 70% of landed mutants are killed or are labelled a real test gap in review | revise; equivalent-mutant rate reported |
| Agnosticism | The frozen guide meets the validity and yield targets on both held-out languages; no scope-specific string in guide, operators or tables; each language table is at most 40 lines | revise; report which language failed and why |
| Comparison | Arm C results reported for the same files; if arm C alone gives equal or better meaningful mutants at lower cost on all four scopes, the recommendation is to adopt the existing tools and stop this mechanism | stop is a valid outcome |
| Apparatus | Estimated production size fits the R41 ceiling of 7,173 canonical lines together with the removal of the fixed operator bodies | owner-reviewed amendment before production, not an automatic pass |

Reported, not gated (baseline for the second priority): agreement between the
two runs of arm A per scope, as the Jaccard index over validated annotation
sites and over kinds; token use and cost per scope.

Outcome is `proceed`, `revise`, `stop` or `not-run`. `proceed` needs every
gated row on at least three of the four languages, including at least one
held-out language, and no failing validator control. Only `proceed` followed
by ordinary design approval unlocks production task generation.

## Bounds

At most two attempts. Per attempt: two runs of arm A per scope, at most two
model calls per run, 64k output tokens per call, a total spend cap of 20 USD
(a dated pre-run amendment is needed to raise it). Per-stage limits: 1,800
seconds for a build or test run, 900 seconds for a mutation batch. Work in
isolated copies under `D:\BuildArtifacts\lesscode\annotation-driver`; keep
prototype scripts and retained results under
`docs/experiments/annotation-driver/`. Linux-only tools run through a Compose
file in that directory. Nothing is adopted into `native/src` during the gate,
and the released commands are not changed.

## Evidence record

No result exists yet. Execution begins with a dated research entry that
freezes the inputs, and ends with one that reports each row above, the raw
annotations, validator decisions, mutants, test logs and the review labels.
A validator control validates the checker, not the mechanism.
