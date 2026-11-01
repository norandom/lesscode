# Lesscode documentation

Start with the [native workflow and scoped readiness report](reports/native-core-readiness.md).
Supply the baseline, candidate, reviewed ownership and externally run checks;
inspect JSON and Markdown before adoption. Python and TypeScript are supported
for the exercised scopes. Native Linux, Windows and macOS checks passed in
[Actions](ci-validation.md#rust-core-dagger-and-actions).
The retained Python evaluator guides below remain usable reference workflows.

## Run your first comparison

| Goal | Guide |
| --- | --- |
| Inspect local Rust/Python/TypeScript baselines and a rejected library shortcut | [mmhmem/mvibe measurements, versions, chart and COBESY experiment track](experiments/local-adoption/README.md) |
| Run Rust and Python tests in parallel | [cargo-nextest and pytest-xdist](testing.md) |
| Run the same Linux checks in WSL and Actions | [Dagger pipeline and native platform evidence](ci-validation.md#rust-core-dagger-and-actions) |
| Build and run the native comparison CLI | [Native setup](../native/README.md) and [working paired example](reports/native-core-readiness.md) |
| Inspect the native decision and maturity boundary | [Readiness gate](../.kiro/specs/capability-compression-core/readiness.md) and [32-criterion coverage](../.kiro/specs/capability-compression-core/requirement-coverage.md) |
| Inspect the completed Python self-trial and exact reviewed adoption | [Self-application record, failures and provenance](experiments/native-self-apply/trial/record.md) |
| Inspect the real typed capability trial | [TypeScript results and limits](experiments/native-replay/typescript-run/results.md) |
| Understand the original native stack proposal | [Historical recommendation](reports/native-compression-direction.md) |
| Understand earlier trials and the current SMT-augmented LLM experiment | [Compression trial progress](reports/compression-trial-progress.md) |
| Coordinate framework knowledge without dependency proliferation | [Repository capability driver](repository-capability-driver.md) and [next experiment protocol](experiments/capability-guided/plan.md) |
| Compare local discovery with a shared capability inventory | [Completed pilot, paired results and chart](experiments/capability-guided/run-01/README.md) |
| Compare local and connected-responsibility rewrites | [Graph trial: behavior preserved, compression rejected](experiments/report-graph/README.md) |
| Test whether a feature path contains redundant representations | [Analyzer-to-report trace and rejected deletion premises](experiments/feature-path/README.md) |
| Measure compression across a capability rather than individual functions | [ATS bilingual renderer: scoped replacement, chart and checks](experiments/ats-report/README.md) |
| Repeat scope-based compression on active typed code | [CPU/NPU backend trial](experiments/npu-backends/README.md) and [two-trial synthesis](reports/capability-compression.md) |
| Install the Python package with uv | [Runtime setup](runtime-setup.md) |
| Provision and probe the pinned analyzers | [Analyzer setup](analyzer-setup.md) |
| Run the included unchanged fixture | [README example](../README.md#try-an-unchanged-pair-first) |
| Compare your own snapshots | [Profile configuration](profiles.md), then [CLI reference](cli.md) |
| Try the process with a reviewer | [Adoption guide](adoption.md) |
| Drive compression trials from real code opportunities | [Reusable procedure and agent skill](compression-procedure.md) |
| Understand the input and run a real family prototype | [Reader-family trial plan and results](experiments/reader-family/README.md) |
| Inspect a smaller classifier and source-derived SMT check | [Outcome hotspot and next LLM compression pass](experiments/outcome-hotspot/README.md) |
| Inspect a real simplification of Lesscode | [Self-evaluation example and charts](examples/self-evaluation.md) |
| Inspect solver-assisted compression, its negative control and next measurements | [Code-flow synthesis report and charts](reports/code-flow-synthesis.md) |
| See selected Facdrone parts and the first self-application rejection | [Measured boundaries and test impact](experiments/facdrone/selected-parts.md) |
| Inspect actual Lesscode synthesis coverage and whole-replacement statistics | [Lesscode self-application report](reports/lesscode-self-application.md) |
| Research substantial simplification after modularization | [Facdrone empirical roadmap](experiments/facdrone/README.md) |
| Inspect a measured architectural obstruction to simplification | [Facdrone solution review and boundary probe](experiments/facdrone/solution-review.md) |

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

The [measurement guide](measurement-guide.md) distinguishes LLOC, complexity,
convolution, role-revealing names and actual maintenance effort. Scoped
[package instructions](../src/lesscode_baseline/AGENTS.md) guide future coding
agent edits before a refactoring pass is needed.

## Inspect the evidence

- [Current native scoped readiness](reports/native-core-readiness.md) and [fresh Windows receipt](evidence/native-core-readiness/receipt.json)
- [ATS replay](experiments/native-replay/ats-run/results.md), [NPU replay and manipulations](experiments/native-replay/npu-run/results.md), [TypeScript trial](experiments/native-replay/typescript-run/results.md)
- [Python self-trial](experiments/native-self-apply/trial/record.md) and [adopted exact file](experiments/native-self-apply/trial/adoption.json)

The following Python baseline receipts are historical evidence for their recorded revisions.

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
