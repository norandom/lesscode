# R16 — Sharing the remaining binder adds code; reject this candidate

Recorded and executed 2026-11-02. **EP01 did not compress the selected responsibility.**
The isolated candidate matched 128 differential observations but increased the
eight-module scope from 1,256 to 1,259 AST statements. Adopted reduction is zero.
Facdrone's 781 inventoried source files still match their frozen hashes.

This revises [R15's preferred case](15-codeql-cross-module-cases.md), not the wider
hypothesis that grown repositories contain substantial removable implementation.
CodeQL helped locate a connected responsibility. The candidate proposed here only
unified its remaining legacy/run-slice binding path; it did not model or simplify
the complete stateful lifecycle. Those are different levels of evidence.

## What was tested

The [plan](../../experiments/constraint-modeler/evidence-publication/plan.md) was
written after source inspection and before candidate editing or differential
execution. The useful reduction threshold was 126 AST statements, approximately
10% of the complete eight-file accounting scope. Selection was exploratory.

The candidate replaced two binding sequences with one internal `_seal_payload`
operation while retaining both public APIs. It preserved the distinction between
embedded legacy slices and namespaced run-slice references/schema version 2.
No production dependency or caller was changed. All added helper code remained
in the measurement. [Complete patch](../../experiments/constraint-modeler/evidence-publication/results/candidate.patch).

| Whole responsibility metric | Original | Candidate | Change |
| --- | ---: | ---: | ---: |
| Physical lines | 2,682 | 2,681 | −1 |
| Python AST statements | 1,256 | 1,259 | +3 |
| Named functions | 123 | 124 | +1 |
| `if` statements | 137 | 138 | +1 |

The one-line physical difference is not useful compression. Shared implementation
needed a new helper and mode branches; public wrappers still remained. These counts
do not measure maintenance hours or establish a complexity-score improvement.

## Behavior evidence is bounded

A frozen external harness ran against original, candidate and wrong-schema mutant
in separate native Windows Python 3.13.16 processes using uv. It exercised 16 fixtures
across two scopes, two namespaces and two binding modes: 128 observations, comprising
72 successful results and 56 exceptions. Candidate mismatches: zero. The wrong-schema
mutant differed in 20 observations. These combinations are not independent experiments.

Checks compare sealed payloads, canonical/stored serialization, identities, slice
references, stored envelopes, error types/messages and input immutability. They cover
missing/null capture, empty/multiple routes, duplicate and forward slice references,
missing/malformed references, future availability and malformed encodings/timestamps.

The harness constructs private captured records directly; it does not exercise the
upstream policy capture producer. It uses empty signal/stage/IC records, so this is
not broad financial evidence coverage. No original Facdrone tests, PostgreSQL lifecycle
checks, live-writer commit/retry tests, simulation retry checks or complete caller
integration tests ran. No SMT proof or learned lifecycle model was produced. Passing
these checks alone would not qualify a useful rewrite for adoption.

The frozen plan stops on insufficient reduction before paying for database setup.
That condition fired. It would be misleading to complete only codec checks and call
the whole responsibility equivalent. [Raw receipt and observations](../../experiments/constraint-modeler/evidence-publication/results/receipt.json)
retain the failures, metric definitions and evaluator/source hashes.

## What changed in the discovery hypothesis

Source inspection showed that live and simulation writers already call the same
low-level slice publisher. Their remaining differences include transaction ownership,
ledger adoption after commit, identity checks and retry-coverage validation. Treating
those as interchangeable wrapper boilerplate would remove real behavior.

This candidate tests ordinary factoring, not the stronger proposed constraint-driven
state compression. Its failure cannot disprove that stronger approach. It does show
that shared callees, connected modules and similar outer sequences do not by themselves
identify duplicated maintained behavior. R15's preference for this case was premature
as a compression forecast; it remains a state-modeling example.

Next experiment: inspect R15's provider-orchestration case for repeated complete
algorithms, with an explicit removal map and retained provider differences before
editing. If common behavior already lives in shared helpers, stop that mechanism
early. For the stateful case, require a concrete redundant state/transition and its
observable distinction before building a translator. Neither route warrants a new
general framework or more synthetic examples at this point.

## Evidence and continuation

[Reproduction, pinned dependencies and limitations](../../experiments/constraint-modeler/evidence-publication/README.md).
The harness is 77 physical lines; the runner is 138 research lines, accounted separately
from production. No test code was deleted. No upstream changes, commits or adoption
occurred. No WSL runtime or database was used.

COBESY handoff: the owner and continuing agents should reject this candidate without
treating a negative trial as evidence of a hopeless repository. The falsified local
claim is that consolidating these binders materially reduces the whole responsibility.
The next decision must be based on removable behavior, not graph size. See the
[Kiro feasibility gate](../../../.kiro/specs/constraint-guided-compression/prototype-gate.md),
which remains `revise`; formal design/tasks approval has not advanced.
