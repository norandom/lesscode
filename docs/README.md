# Lesscode documentation

## Start here

| You want to | Read |
| --- | --- |
| Install the binary and run it on one change | [README](../README.md#install) |
| Know what the released commands check and what they do not | [README: what is not checked](../README.md#what-is-not-checked) and [R55](research/2026-11-10/01-released-binary-adopter-dogfood.md) |
| Try it with one reviewer before making it a gate | [Adoption guide](adoption.md) |
| Build from source, check prerequisites and license notices | [Native core](../native/README.md) |
| Understand the architecture or add a language | [Multi-language deconvolution](multi-language-deconvolution.md) |
| See how checks and releases are built | [CI and release workflows](ci-validation.md) |
| Verify a download or set up the signing key | [Release signing](release-signing.md) |
| Follow the experiments, including the failed ones | [Dated research log](research/README.md) |
| Judge whether the agnostic mutation driver can become part of the Rust binary | [Feasibility report](reports/agnostic-driver-feasibility.md) |
| Use the frozen-manifest comparison (`prepare`, `compare`) | [Native workflow and scoped readiness report](reports/native-core-readiness.md) |

The native core runs on Linux x86_64 and aarch64, Windows x86_64, and macOS on
Intel and Apple Silicon; each push to `main` is checked on all five in
[Actions](ci-validation.md#multi-platform-and-multi-architecture-ci-matrix).

The Python prototype (`lesscode-baseline`) was retired at commit `fd0b5b9`.
Its guides ([CLI](cli.md), [runtime setup](runtime-setup.md),
[analyzer setup](analyzer-setup.md), [profiles](profiles.md),
[technical overview](technical-overview.md)) are kept as archived reference
and carry a notice at the top.

## Experiments, reports and earlier guides

Rows that mention the Python package, uv or profiles belong to the retired
prototype. Some rows point to experiment records that are kept in the owner's
working tree and may not be present in a fresh clone.

| Goal | Guide |
| --- | --- |
| Inspect the unified 4-language state-boundary deconvolution architecture | [Multi-language deconvolution architecture and extension guide](multi-language-deconvolution.md) |
| Inspect preserved Python baseline history and commit references | [Preserved Python baseline index](archive/preserved-python-baseline.md) |
| Inspect the first CodeQL query prototype and its failed callback check | [CG-G0: extraction, frozen oracle and replay](experiments/constraint-modeler/README.md) |
| Review the proposed graph-driven, multi-pass constraint-modeling research | [Research blueprint and staged experiments](../.kiro/specs/constraint-guided-compression/research.md) |
| Inspect real paired replacements, including a marginal Facdrone result | [mvibe: 23 lines removed](experiments/proxy-routing-trial/README.md) and [Facdrone CSV: rejected](experiments/facdrone-csv-trial/README.md) |
| Inspect local Rust/Python/TypeScript baselines and a rejected library shortcut | [mmhmem/mvibe measurements, versions, chart and COBESY experiment track](experiments/local-adoption/README.md) |
| Run Rust and Python tests in parallel | [cargo-nextest and pytest-xdist](testing.md) |
| Find decision sites, write mutants and run a project's tests against them (from release 1.1.0) | [The agnostic driver](driver.md) |
| See what a release contains and what changed | [Release 1.1.1](releases/1.1.1.md), [release 1.1.0](releases/1.1.0.md) |
| Decide how far a model may be trusted to write a language pack or choose mutation sites | [Slice B closing summary](reports/slice-b-drift-hardening.md) |
| Run the same Linux checks locally (native Dagger CLI with Podman) and in Actions | [Dagger pipeline and native platform evidence](ci-validation.md#rust-core-dagger-and-actions) |
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
| Choose symbolic-execution and guided-input tooling per language | [Symbolic execution tooling assessment](symbolic-execution-tooling.md) |
| Understand the input and run a real family prototype | [Reader-family trial plan and results](experiments/reader-family/README.md) |
| Inspect a smaller classifier and source-derived SMT check | [Outcome hotspot and next LLM compression pass](experiments/outcome-hotspot/README.md) |
| Inspect a real simplification of Lesscode | [Self-evaluation example and charts](examples/self-evaluation.md) |
| Inspect solver-assisted compression, its negative control and next measurements | [Code-flow synthesis report and charts](reports/code-flow-synthesis.md) |
| See selected Facdrone parts and the first self-application rejection | [Measured boundaries and test impact](experiments/facdrone/selected-parts.md) |
| Inspect actual Lesscode synthesis coverage and whole-replacement statistics | [Lesscode self-application report](reports/lesscode-self-application.md) |
| Research substantial simplification after modularization | [Facdrone empirical roadmap](experiments/facdrone/README.md) |
| Inspect a measured architectural obstruction to simplification | [Facdrone solution review and boundary probe](experiments/facdrone/solution-review.md) |

## Understand the retired Python evaluator

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
