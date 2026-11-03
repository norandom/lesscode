# R17 — The provider load algorithm is already largely shared

Recorded and executed 2026-11-02. **Do not build another orchestration abstraction
from this lead.** The three provider load methods together contain 37 AST statements
inside a five-module scope of 693 statements. Their substantive common operations
already live in shared helpers. This is a static admission result, not an executed
replacement, proof of equivalence or evidence that Facdrone cannot be compressed.

## What the screen establishes

Following [R16](16-evidence-publication-trial.md), PO01 inspected R15's ECB, FRED
and licensed S&P adapters. The [plan](../../experiments/constraint-modeler/provider-orchestration/plan.md)
froze a 10% whole-scope statement-reduction target before measurement: 70 statements.
Existing CodeQL case evidence and source/dependency manifests were reused. Source
inspection, not a new CodeQL run or automated semantic model, supplied the distinctions.

| Enumerated edit region | AST statements | Maximum gross removal as fraction of scope |
| --- | ---: | ---: |
| All three `load` methods | 37 | 5.34% |
| Load methods plus all four series/segment builders | 60 | 8.66% |
| Complete five-file accounting scope | 693 | 100% |

The first two rows overlap and must not be added. The deliberately unrealistic
zero-replacement bound counts every statement in those functions as removable,
including definitions, validation and returns. Even that bound is below this
trial's threshold. Real helpers, retained provider rules and callers cost code.
This bound applies only to these named edit regions; it says nothing about
rewriting other regions or changing the broader representation.

## Shared steps do not imply duplicated implementations

| Stage | Existing common operation | Required differences retained in the adapters |
| --- | --- | --- |
| Admission | `validate_source_snapshot` | Approved series/status/version; S&P entitlement; ECB/FRED historical start checks |
| Row interpretation | Positive-level/date helpers; ECB/S&P share `row_times` | FRED missing-publication marker and acquisition availability; ECB header/dialect/status rules; S&P licensed schema and error order |
| Selection | `select_level_window` | Time zone and configured maximum age |
| Construction | Typed series/segment models | Currency, methodology, calendar, quote convention, publication policy and provenance |
| Final evidence | `evidence_from_snapshot` | Provider metadata; strict compounded-index validation only where applicable |

These differences come from the frozen provider implementations. They are observable
contracts, not proposed finance changes. No entitlement or source validation should
be removed to manufacture compression. The apparent repeated outer sequence mostly
calls common code already; another dispatcher would retain much of that sequencing
as parameters, callbacks or wrappers.

## A metric limitation remains open

The constructor alternative repeats named fields across `SeriesSegment` and
`VersionedIndexSeries`. Python AST statement counts treat a large keyword constructor
as one expression. Thus the 10% statement gate cannot fairly determine whether a
data-oriented construction replacement would substantially reduce explicit field
plumbing. Physical lines alone can also be gamed by formatting.

We have **not** tested or rejected that alternative on behavioral/usefulness grounds.
Its disposition is deferred under an inadequate representation metric. A separate
trial would need a declared field-mapping/representation measure alongside physical
lines, statement non-regression and whole-scope accounting. It must preserve useful
named fields and types and count all configuration/adapters. Do not change PO01's
threshold retrospectively or claim its failure proves the constructors are optimal.

## Decision and reproducibility

No candidate rewrite was admitted, no specimen tests executed, no source changed,
and adopted reduction is zero. All 781 upstream source hashes still match R15.
The original test boundary was located at
`packages/facdrone-reporting/tests/test_index_sources_and_alignment.py`; locating it
does not mean running it. No new dependencies, SMT, Cairn or AALpy were used.

Reproduce from Lesscode with the existing uv-managed Windows environment:

```powershell
uv.exe run --no-project --python .runs/evidence-publication-venv/Scripts/python.exe python docs/experiments/constraint-modeler/provider-orchestration/screen.py
```

The script uses only the standard library. [Raw screen receipt](../../experiments/constraint-modeler/provider-orchestration/screen.json)
records Python/platform, hashes, every measured function span and shared-helper call
location. [Script](../../experiments/constraint-modeler/provider-orchestration/screen.py)
is research apparatus, not a new production discovery subsystem.

COBESY handoff: the owner and continuing agents should distinguish a map of related
code from evidence of duplicated maintained behavior. The local orchestration
hypothesis fails admission; the broader representation hypothesis remains open.
Keep the [Kiro gate](../../../.kiro/specs/constraint-guided-compression/prototype-gate.md)
at `revise`. Neither the bounded rejection nor locating a metric blind spot qualifies
the proposed multi-pass constraint modeler for production implementation.
