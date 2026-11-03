# R18 — Fewer repeated fields still produced more maintained code

Executed and recorded 2026-11-02. **Reject PR01's shared snapshot projection.**
It preserves all 32 sampled provider-load outcomes and passes the same 70 selected
source/alignment tests as the original, but increases the complete accounting scope
by 12 physical lines and 10 AST statements. Adopted reduction is zero.

The owner authorized rewriting Facdrone directly as an experimental specimen.
That permission is now in project memory. This candidate was developed in an
experimental copy to preserve a replayable original, not because the checkout is
protected. There is no benefit in installing this larger candidate in the checkout.
No Facdrone commit/push or production change occurred.

## A concrete representation alternative, beyond the admission screen

[R17](17-provider-orchestration-screen.md) left repeated metadata projection open
because AST statements undercount constructor plumbing. PR01 therefore implemented
the alternative rather than rejecting it on that metric alone. The
[plan](../../experiments/constraint-modeler/provider-representation/plan.md) set a
33-line net reduction aim for the complete construction replacement and required
whole-provider accounting. It did not carry forward R17's 70-statement gate.

Three provider builders previously constructed both a segment and its enclosing
series. The candidate replaced those repeated projections with `snapshot_series`,
sharing identity fields through a TypedDict and keeping provider-specific named
metadata at each call site. ECB's separate segment helper was removed. Public
load APIs, parsing, source admission, validation, currencies, timestamps, calendars,
methodologies and provenance remained. No production dependency was introduced.

The [full patch](../../experiments/constraint-modeler/provider-representation/candidate.patch)
includes the new helper, its typed arguments/identity record, all imports and all
three residual provider builders. Ruff 0.14.3 formatted the changed files at width
100 before measurement. No compression is attributed to condensed formatting.

| Complete five-file scope plus new helper | Original | Candidate | Change |
| --- | ---: | ---: | ---: |
| Physical lines | 1,865 | 1,877 | +12 |
| AST statements | 693 | 703 | +10 |
| `if` statements | 72 | 72 | 0 |
| Explicit keyword arguments | 305 | 275 | −30 |

The last row counts Python `ast.keyword` nodes with named arguments across this
scope. It is an auxiliary representation measure, not a maintainability score.
Fewer explicit field mappings did not pay for the new signature, type declaration,
helper and residual calls. We do not infer maintenance savings or a smaller project.

## Behavior evidence and limits

The same unchanged `test_index_sources_and_alignment.py` passed **70 tests** on
the original and candidate. Pytest ran on native Windows through uv Python 3.13.16,
with `not port_baseline and not live_broker`, no concurrent pytest process and no
automatic fixture-plugin loading. `--noconftest` avoided the unrelated global
database fixture; no missing fixtures or skips occurred in this selected file.
This is a scoped test run, not Facdrone's full gate or a live report build.

A separate harness calls the real ECB, FRED and S&P provider `load` methods with
supplied immutable CSV snapshots, covering €STR, FX, SOFR and SPTR. Across 32
combinations, eight produce complete results and 24 produce errors. Entire returned
dataclass trees (including Decimal values, timestamps, metadata and evidence) and
exception types/messages match. Changing the helper's publication policy string
causes eight mismatches, demonstrating that metadata loss is observable.

These are synthetic CSV inputs to real code, not independent real-world datasets.
The variants cover identity/status refusal, missing snapshot, header-only input,
future availability, entitlement and selected bad levels. The FX bad-level mutation
does not alter its decimal-valued CSV, so that combination is a repeated success,
not an invalid-level rejection. No network, broker, database, SMT, CodeQL re-query,
Cairn or AALpy execution was part of PR01. No universal equivalence claim follows.

Baseline setup initially failed on missing `statsmodels`, then released Facdrone
data packages and XLSX support. Those packages were installed only in the research
environment; resolved versions are retained. A harness setup error attempted to
construct an empty snapshot, which the real model correctly rejects immediately.
It was corrected to a header-only payload **after candidate generation**, then the
same final harness ran on both versions. This is an explicit deviation from the
intended baseline-before-edit sequence, not a retrospectively preregistered check.

## What this result changes

The local hypothesis was falsified: repeating a field mapping does not necessarily
justify a new shared abstraction. Required provider variation consumes most of the
remaining construction code. Neither deleting checks nor hiding named fields in
positional tuples would meet the intended readability/behavior boundary.

This still does **not** exercise the originally proposed whole-capability constraint
modeler. It is a manually generated representation candidate grounded in a previously
mapped responsibility. We must not present several small factoring failures as a
test of semantic state compression, or conclude that the repository is incompressible.
The outstanding problem remains selecting a capability with an avoidable internal
representation or duplicated algorithm, rather than merely similar construction.

The [Kiro gate](../../../.kiro/specs/constraint-guided-compression/prototype-gate.md)
remains `revise`. COBESY handoff: reject this concrete candidate; preserve the
working behavioral checks and the failed patch, but do not grow a production
framework around this result. Fewer repeated mappings is evidence about structure,
not success at less maintained code.

## Reproduction and evidence

[Experiment directory](../../experiments/constraint-modeler/provider-representation/),
[metrics/observations receipt](../../experiments/constraint-modeler/provider-representation/results/receipt.json),
[baseline test log](../../experiments/constraint-modeler/provider-representation/results/upstream-tests.txt),
[candidate test log](../../experiments/constraint-modeler/provider-representation/results/candidate-tests.txt),
[test/configuration hashes](../../experiments/constraint-modeler/provider-representation/results/test-input-hashes.json).

Run `check.py SOURCE_ROOT` through the pinned uv environment for provider observations.
`rewrite.py` constructs the candidate from R15's frozen snapshot and refuses an
existing destination. `finalize.py` retains observations and runs/restores the wrong
policy mutant. The original selected tests live in Facdrone; set `PYTHONPATH` to the
chosen source root's package directories plus `src`, and `FACDRONE_PROJECT_ROOT` to
the checkout containing the unchanged reporting configuration. Invoke pytest with
`--noconftest -c packages/facdrone-reporting/pyproject.toml`, the explicit test file
and the safe marker expression. See `requirements.txt` for exact research packages.
The apparatus and new checks are research code, excluded from production savings.
