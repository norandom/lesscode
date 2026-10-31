# Lesscode documentation

Start with a supplied baseline and candidate. Establish the checks and scope,
run the comparison, then inspect the evidence before accepting the change.

## Run your first comparison

| Goal | Guide |
| --- | --- |
| Install the Python package with uv | [Runtime setup](runtime-setup.md) |
| Provision and probe the pinned analyzers | [Analyzer setup](analyzer-setup.md) |
| Run the included unchanged fixture | [README example](../README.md#try-an-unchanged-pair-first) |
| Compare your own snapshots | [Profile configuration](profiles.md), then [CLI reference](cli.md) |
| Try the process with a reviewer | [Adoption guide](adoption.md) |

## Understand the evaluator

Read the [technical overview](technical-overview.md) for the detailed writeup:

- [Architecture and execution](technical-overview.md#architecture-and-execution)
- [Measurement definitions](technical-overview.md#measurement-definitions)
- [Decision rules](technical-overview.md#decision-rules)
- [Artifact storage and recovery](technical-overview.md#artifact-storage-and-recovery)
- [Controlled experiments](technical-overview.md#controlled-experiments)
- [Trust boundaries and limits](technical-overview.md#trust-boundaries-and-limits)

The [CLI reference](cli.md) specifies invocation and exit semantics. The
[profile guide](profiles.md) explains what you must configure and which rules
the current implementation can execute.

## Inspect the evidence

- [Native CI procedure and artifact access](ci-validation.md)
- [Native audit receipt](evidence/native-41198054658.json)
- [Fresh suite and installed-package receipt](evidence/readiness-2026-10-31.json)
- [Final feature validation](../.kiro/specs/efficiency-baseline/validation.md)
- [All 56 acceptance criteria and their coverage](../.kiro/specs/efficiency-baseline/requirement-coverage.md)

The receipts describe specific tested revisions. Raw CI artifacts have finite
retention; the CI guide explains how to reproduce them.

## Maintain or extend Lesscode

The versioned records and serializers in
[`contracts.py`](../src/lesscode_baseline/contracts.py) define the JSON boundary.
The [approved design](../.kiro/specs/efficiency-baseline/design.md) and
[requirements](../.kiro/specs/efficiency-baseline/requirements.md) record the
baseline contract. The [roadmap](../.kiro/steering/roadmap.md) separates delivered
evaluation from proposed agent rules, skills and rewrites.

[Analyzer dependency licenses](../tools/licenses.json) map locked packages to
the original notices retained in `tools/notices/`; dependencies do not all share
one license.

[Documentation design notes](maintainers/documentation-design.md) record the
COBESY framing, composition plan and adoption assumptions behind these guides.
