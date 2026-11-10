# R58: The annotation validator rejects every hand-made wrong annotation and accepts the correct ones on all four scopes (71 of 71 controls); arm D is in force and the hard-coded operators are to be removed completely

- Report date/time and timezone: 2026-11-10, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-10, about 10:25 to 11:00 local, one session
- Status: completed (gate checklist step 2, model-free)
- Spec: [.kiro/specs/agnostic-annotation-driver/prototype-gate.md](../../../.kiro/specs/agnostic-annotation-driver/prototype-gate.md), Amendment 2
- Hypothesis ID and predecessor: H-AD1, [R57](03-annotation-driver-scope-admission.md)
- Type: preregistered (checker validation; no hypothesis test in this entry)

## Decision

The gate's validator-integrity row is met. This permits the next steps: the
comparison arm and the baseline for mutation runs (step 3), then the guide
and the rule packs. It establishes that the checker behaves as specified on
these four scopes. It says nothing about whether a model's annotations or
syntactic rules produce useful mutants.

Two owner decisions of the same day are recorded in gate Amendment 2:

- Arm D (rules as data, with the model as an offline rule author for the
  held-out languages) is approved and in force.
- The hard-coded mutation operators must be removed completely. They are not
  kept as a control set. In this gate they appear only as a measurement taken
  with the released 1.0.1 binary.

No model call was made and nothing was spent.

## Hypothesis and falsification

Prediction for this step, fixed in the gate: 100% of hand-made wrong
annotations are rejected and no-op mutants are classified invalid. Added
before the run, because a validator that rejects everything would pass that
prediction trivially: every hand-made correct annotation must be accepted,
and a real change must be classified as a change. Any deviation would have
stopped the gate until fixed.

## Frozen inputs and method

Prototype in [docs/experiments/annotation-driver/](../../experiments/annotation-driver/README.md),
run with CPython through uv and `ast-grep-py` 0.39.5 (locked in `uv.lock`).
Scopes as admitted in R57, unchanged.

| Input | SHA-256 |
| --- | --- |
| `validator.py` | `d1d11ac7841ca48d7f8ef3635cce28da2414f0fe2b816ba118965c3c3f313775` |
| `run_controls.py` | `6e3851a80babd3295ef35ce4206570fe86c50a0cf8c0ec28758f8b159702bb49` |
| `controls.json` | `2d1d4fb9fb4bd2cdb758f5912b6f381c87e1e0171d62991284b3d1f8ef27550d` |
| `scopes.json` | `c1ea2e9bb827a053ff46445c2c6a8713a90ab235da57ab76c7909a24a234f2e0` |
| `tables/java.json` | `e8769f905d6aba313b5a5b677890f46114f8187cc64718a79ae42217f8c69fc7` |
| `tables/python.json` | `0be340697d93a3433a272942cfc51715b5811bbeea4158d1ecf0ac286a9a4b4a` |
| `tables/rust.json` | `b519961dfe34d2f37aa90cf838d3519a295f7504814ef7ee879fe1271316ff54` |
| `tables/typescript.json` | `f90933c604c8c2a3b81d20acfd9ea0d121666faedd4e9ed1170380642b5dcef3` |

The files are stored with LF line endings (`.gitattributes`), so the hashes
hold in a fresh clone.

Controls, per scope: 2 or 3 correct seed annotations written by hand from the
source (11 in total, covering `branch_condition`, `state_write` and
`failure_state`); ten wrong annotations derived from the first seed or from
locations named in `controls.json` (wrong line, altered anchor, partial
anchor, a comment as the site, a test file, a file outside the scope, a kind
the node cannot have, a duplicate, a missing kind, inconsistent lines); five
mutant classifications (identical, trailing whitespace only, comment only, a
real one-token change, an unparseable change).

Command:

```text
uv run python run_controls.py --scopes-dir D:\BuildArtifacts\lesscode\annotation-driver\scopes --out results/controls-receipt.json
```

## Evidence and results

[Receipt](../../experiments/annotation-driver/results/controls-receipt.json):
71 controls, 0 failed. Every control group behaved as expected on all four
scopes: 11 of 11 seeds accepted, 40 of 40 wrong annotations rejected with the
expected reason, 20 of 20 mutant classifications as expected.

Failed attempt, kept on record: the first run gave 70 of 71. The failing
control was mine, not the validator's. The whitespace-only mutant doubled the
first three newlines of the file, which in the Python scope fall inside the
module docstring, so the string content really changed and the validator
correctly said `changed`. The control now only adds trailing blank lines.
The incident is the one place in this step where the checker was exercised on
an input I had not predicted correctly, and it was right.

Observations made while building the tables, relevant to later steps:

- The Rust scope has few branch sites: 4 `if` expressions, 3 `match`
  expressions, 1 binary expression, no `return` and no `?`. The gate's yield
  row (at least 4 landed mutants over at least 3 kinds) may be hard to meet
  there for syntactic reasons alone.
- TypeScript and Java wrap conditions in parentheses that belong to the
  statement. An annotation on the inner expression is the natural thing to
  write, so the tables name the wrapper and the validator accepts the inner
  node as the condition.
- The per-language tables are 14 to 17 lines, against the gate's limit of 40.
- Prototype size so far: `validator.py` 210 lines, `run_controls.py` 128
  lines, tables 59 lines in total.

## Limits and interpretation

- The controls are hand-made and few. They show the rejection paths work on
  these files; they are not a proof that no wrong annotation can pass.
- `failure_state` is checked syntactically only. A `return` that is not a
  failure passes the kind check if the model labels it one; the Java seed
  uses such a return on purpose. Whether such sites give meaningful mutants
  is what the meaningfulness row measures later.
- `error_order` and `pure_shell_boundary` have table entries but no seed, so
  those two kinds are untested by controls.
- The parse check relies on error nodes from the tree-sitter grammars; a
  mutant that parses but does not compile is caught only later, by the build.
- One reading of the owner's removal requirement is assumed: it concerns the
  hard-coded operators of arm B. The hand-written rule packs of arm D1 are
  generic syntax rules stored as data and are not treated as hard-coded.

## Tool/spec consequence and next gate

- Gate Amendment 2: arm D in force; removal of the hard-coded operators is a
  required consequence of any `proceed` design and supersedes the
  "frozen control set" condition in constraint-guided-compression
  Amendment 5; validator rules made precise and frozen; integrity row met.
- Next (step 3, still model-free): run the comparison arm (one existing
  mutation tool per language, restricted to the scope file) and establish the
  mutation baseline in each clone. Then the generic operators, the rule packs
  for Rust and Python (D1), and guide version 0. A further entry freezes the
  guide and packs before the first model call.

## COBESY check

Outcome first: checker validated, mechanism untested. Replay: inputs hashed,
command and receipt given. Disproof: the prediction and its added positive
half are stated; the one failure is reported with its cause. Negative
observations that may hurt the hypothesis later (few Rust sites) are stated
now. Facts are separated from the assumed reading of the owner's requirement.
