# Requirements Document

## Project Description (Input)

Developers using lesscode can already plan and check small refactoring steps, but
must still find and write useful changes. Existing step reports show test and
mutation outcomes and source size; they do not establish whether branch conditions
or scattered state were reduced. The Python pack also counts local assignments as
state writes, so a lower count after correcting the pack could be mistaken for a
simpler program.

The feature will let a configured model propose bounded simplifications through
`lesscode-model`, use the existing step flow to judge behavior, and report measurable
simplicity changes over the whole declared responsibility. Planning will expose
branch and state concentrations while retaining protection information. First real
cases are `x_likes_scraper`'s `x_likes_exporter/exporter.py` for state and
`x_likes_mcp/tools.py` for branching.

Owner decision, 2026-11-17: simplicity is reported separately from the existing
behavior verdict. An accepted step need not be simpler. The analysis must distinguish
measurement correction, added behavioral protection, model-written changes, repair
effects, human assistance and adoption. Lines or tokens removed are not the
effectiveness measure.

Scope and prior decisions: [discovery brief](brief.md).
Existing capabilities, gaps and proposed attribution method: [analysis](research.md).
Dated static-analysis record: [R114](../../../docs/research/2026-11-17/01-model-simplification-measurement-attribution.md).

## Requirements
<!-- Will be generated in /kiro-spec-requirements phase -->

No acceptance criteria are generated or approved at initialization. The measures
and comparison arms in the analysis are proposals for requirements and the
prototype gate, not established improvements or an executable production plan.
