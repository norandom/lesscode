# Research and architecture proposal

## Implementation outcome

All 22 tasks are implemented and independently reviewed. The native workflow,
external Python/TypeScript replays and scoped Python self-adoption are recorded
in readiness.md and implementation-log.md. Final validation.md withholds broader
GO pending current Linux/macOS integrated results. Earlier proposal/approval
states below remain historical.

## Current Summary — design generation

Current phase update: explicit `kiro-spec-tasks capability-compression-core -y`
approved requirements, design and tasks. Independent task-graph sanity review
passed after one sizing repair; see [review record](task-plan-review.md).
Implementation has not started. The design-generation notes below retain their
historical approval state.

- Feature: capability-compression-core; discovery scope: new native feature with
  brownfield evidence reuse (full discovery).
- Owner instruction to proceed approves amended requirements, including the
  self-application maturity gate. Design is generated for review; tasks and
  implementation remain pending.
- Selected one synchronous Rust package, existing grammar library and existing
  external check runners. No reinstatement of retired layers.
- Key limits: syntactic scope is not semantic completeness; supplied results
  establish recorded evidence, not authenticated execution; native metrics must
  not inherit incompatible historical names.

This section and the design log below supersede the earlier proposal's pending
requirements/no-design status. Earlier notes are retained as decision history.

See [native compression direction](../../../docs/reports/native-compression-direction.md)
for the evidence, ecosystem comparison, source citations, keep/defer/drop decisions
and proposed implementation sequence. Rust is recommended, not yet an approved
formal design. No design or tasks have been generated.

The requirements intentionally separate user-observable scope and evidence
behavior from the proposed ast-grep implementation. Imported metrics need explicit
definition compatibility; a new native metric cannot inherit a historical name
without reproducing its definition. A skill-driven repository inventory and
candidate selection precede local edits; ASTs alone do not establish semantic
reachability, completeness or equivalent behavior.

## Brownfield findings

A delegated read-only inventory identified the frozen ATS01/NPU01 trials and
adverse contract/evidence fixtures as reusable. It advised against porting Python
runtime bootstrapping, report storage machinery, the unfinished policy platform,
or experiment-specific orchestration. The agent-quality task list marks 9/25
tasks complete; prior steering had become stale. Existing uncommitted source
changes remain untouched.

## Requirements review gate

Inline review completed before finalizing the draft. Four numerically identified
requirement groups cover scope, measurement, behavioral evidence and workflow.
Criteria use EARS statements and observable outcomes. A local repair made the
compression target explicit: a frozen scope-level threshold and allowed structural
tradeoffs are required for successful compression classification; otherwise the
report is descriptive. This avoids inventing a universal reduction percentage.

Boundary review: hardware/performance, feature removal and automatic semantic
proof are excluded; checks and target runtimes remain project responsibilities.
Implementation proposals are held outside acceptance criteria. Missing evidence,
incompatible metrics, omitted implementation and unverified behavior are explicit.
The review is an authoring check, not independent implementation validation or
human approval. Requirements approval remains pending under the Kiro workflow.

## Self-application amendment

Owner requested a maturity gate after implementation. Requirement 5 adds a
fifth group: a bounded self-trial, frozen evaluator and checks, regression checks,
independent fixtures for evaluator changes, and separate workflow/compression
outcomes. Inline boundary review: no bootstrap cycle, compulsory reduction quota,
or Rust-analysis expansion is introduced. Unsupported coverage is declared;
incomplete evidence cannot establish maturity. The earlier four-group review
above describes the original draft. Formal design/tasks remain pending and must
include this gate in final validation.

## Research Log — native design

### Existing code and evidence
Read the existing Python decision logic, steering, discovery skill, ATS candidate
result and NPU execution receipt. Reused the earlier delegated codebase inventory.
The baseline distinguishes integrity, baseline eligibility, failure and missing
evidence; retain those meanings without translating the entire package. The NPU
collector can exit zero while reporting mismatches, so native admission needs
interpreted result evidence rather than a process exit code alone.

### External dependencies and local prerequisites
Verified local cargo/rustc report 1.98.1. This confirms tool availability, not a
successful native build. Read primary library documentation:

- [ast-grep core](https://docs.rs/ast-grep-core/latest/ast_grep_core/): library
  parsing/traversal/matching API; documentation reports 0.51.0.
- [ast-grep language](https://docs.rs/ast-grep-language/latest/ast_grep_language/):
  LanguageExt and Python/TypeScript/TSX support; documentation reports 0.51.0.
- [Serde attributes](https://serde.rs/attributes.html): derived typed records with
  deny_unknown_fields, avoiding a custom serialization framework.
- [Cargo lockfiles](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html):
  exact resolved dependency recording; commit a native package lockfile.
- [sha2](https://docs.rs/sha2/latest/sha2/): SHA-256 implementation; docs report 0.11.1.
- [clap](https://docs.rs/clap/latest/clap/): argument parsing; docs report 4.6.7.

Version-specific node/feature documentation requests were unavailable through the
browser. Grammar feature selection and compilation remain explicit first-build
checks; no feature flags or build success were invented. Design selects release
lines; the first build records exact compatible pins and license notices.
No hardware or performance research was performed because the owner excluded it.

## Architecture Pattern Evaluation

| Option | Assessment |
| --- | --- |
| Port Python evaluator and rule engine | Rejected: duplicates the platform and reinstates removed scope |
| Native library plus thin CLI | Selected: deterministic comparison with one entry path and plain data |
| Native command runner and analyzer adapters | Rejected: existing project checks suffice; avoid a second execution framework |
| Skill alone | Insufficient: cannot supply reproducible fixed accounting and evidence admission by instructions alone |

## Design Decisions — synthesis

1. **Generalize the trial, not the platform.** ATS/NPU and self-application use one
   manifest/compare flow. No special self-refactoring service or language backend
   registry. Maturity remains a recorded review procedure.
2. **Adopt parsing, serialization, hashing and CLI utilities.** Build only the
   scope accounting and decision semantics those libraries do not provide.
3. **Use supplied evidence.** Keep project runtimes outside the native core;
   report trust and missing provenance explicitly. Historical conversion scripts
   are research support, not runtime adapters.
4. **Count expansions conservatively.** Added/moved support must enter replacement
   accounting; full changed-file accounting is safer than automatically guessing
   semantic slice equivalence. This can dilute measured savings and is disclosed.
5. **Prevent circular self-certification.** Freeze evaluator/check identities;
   use independent expected outcomes if their implementation changes. Complete
   rejection evidence may establish workflow maturity but not compression.

## Risks and mitigations

- Semantic omissions: located unresolved relationships and explicit ownership
  review; no universal closure claim from syntax.
- Metric drift: per-language versioned definitions and hand-counted fixtures;
  incompatible historical/native results remain separate.
- Evidence forgery: no adversarial security claim from hashes; maintain externally
  controlled checks when that trust boundary is required.
- Scope inflation: preserve the no-service/no-plugin/no-solver boundary; inspect
  native production and test footprint against delivered behavior.
- Bootstrap blockage: mature only after self-trial evidence exists, but keep
  implementation possible and permit a justified no-change outcome.

## Design review gate

Inline review (not independent): all 32 requirement IDs appear in explicit
traceability rows. Boundary sections, concrete file ownership and allowed
dependencies are populated. Repaired scope/exclusion accounting and explicit
unknown/provenance handling before finalizing. No placeholder components,
automatic semantic proof, hardware gate or extension framework is introduced.
Design is ready for human review; no runtime test or implementation completion
claim follows from this authoring check.
