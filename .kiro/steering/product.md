# Lesscode / Deconvolute

## Authoritative scope: less code, same behavior

Owner clarification, 2026-11-01: hardware execution, accelerator validation and performance optimization/benchmarking are outside Lesscode's scope. Do not make throughput, latency, memory consumption or hardware access acceptance gates for this research. Preserve the declared software-visible behavior, including interaction contracts and safeguards. Process timeouts and cross-platform correctness are operational concerns, not performance targets. Measure maintained implementation and structure; bytes/gzip are not benefit metrics. Earlier trial limitations remain historical records and do not expand this scope.

Owner clarification, 2026-11-01: this project is about code. Its objective is to help agents express the same behavior with substantially less, clearer implementation. Agents remain the means of producing code. Requirements engineering, feature reduction and application-specific domain redesign are outside this work.

Study a complete implementation responsibility, including its helpers and callers. Look for custom machinery already supplied by the language or installed libraries; repeated conversion, copying, validation and serialization; forwarding layers without meaningful behavior; procedural scaffolding around a simpler algorithm or representation; excessive fragmentation; and duplication that can share an implementation without creating a generic framework.

Ask: what would an experienced programmer use to implement this same operation, and which existing code would that make unnecessary? Compare concrete code alternatives against the original. Preserve observable behavior, public contracts, safeguards, useful types and role-revealing names. Count the whole replacement, including adapters, configuration and dependency integration. Complexity metrics locate candidates; empirical replacement results establish compression. Smaller files, relocated code, denser formatting or a better isolated function score do not establish less maintained implementation.

Facdrone is an example case for empirical research, not Lesscode's key deliverable. Local code experiments and behavior-preserving refactoring trials on Facdrone are allowed when they test a general Lesscode hypothesis; use isolated experimental copies and preserve the user's working state. Do not commit or push changes upstream to Facdrone from this work for now. The deliverable is a reusable solution for producing less code with the same behavior, supported by measured examples, rather than delivery of a Facdrone refactor. Each trial should state the general code-level hypothesis, compare concrete implementations, record behavior and whole-replacement size results, and explain what can generalize. Facdrone-specific details supply evidence and constraints, not a separate application-development agenda. Do not change financial behavior or other application semantics to manufacture reductions. The earlier schema/regime proposal is superseded as the project's agenda; it is not a blanket prohibition on experiments using this example. Requirements engineering remains outside the project's focus.

Derive reusable agent practices and tools from demonstrated behavior-preserving code reductions. Keep production code, tests, fixtures and experiment support separate in measurements; tests protect real behavior and are not a reduction target by default. Do not expand infrastructure before empirical results justify it.

Establish framework/library knowledge once per repository snapshot through a global capability discovery pass before distributing local simplification work. Distinguish runtime, build, test, analyzer and research dependencies; local agents reuse the shared inventory and cannot independently introduce competing frameworks. New dependencies or upgrades require a repository-level decision within the user's authorized scope. Refresh affected inventory entries when manifests, runtime constraints or cited source contracts change. The initial controlled experiment freezes dependencies; this workflow is a design direction, not a claim of implemented orchestration or MCP features. See `docs/repository-capability-driver.md` and `docs/experiments/capability-guided/plan.md`.

Compression discovery starts with a complete responsibility and a concrete deletion/replacement hypothesis. Select repeated behavior families or substantial custom operations with a plausible existing or simpler replacement; only then choose verification tools. A small isolated classifier forced into a table is a rejection control, not evidence for or against family compression. Solver-interface coverage is not compression potential. The experimental repo-local `lesscode-discovery` skill records this procedure without changing formal Kiro specification status.

Current empirical pass (owner clarification, 2026-11-01): select and measure appropriate Facdrone examples read-only; do not change Facdrone. Apply candidate simplifications to Lesscode only when useful, with the existing tests preserving the functionality boundary. Retain required features and useful structure rather than minimizing the project indiscriminately. Record rejected replacements and test impact alongside successful reductions.

## Capability-scoped compression

Self-application is a maturity gate after the first working implementation:
exercise Lesscode on a complete capability of its own code, with an isolated
candidate, frozen evaluator/checks and independent expected outcomes when changing
the evaluator. Preserve existing functionality and require project regression
checks before adoption. Report workflow maturity separately from actual reduction;
a justified rejection is preferable to forced deletion, while missing evidence
keeps the maturity claim open. This must not block bootstrap or ongoing development.
Unsupported languages remain explicit exclusions, not a reason to expand the stack.

Compression acceptance is scoped to a complete capability, not individual
functions. Freeze its entry points, outputs and transitive owned implementation
before proposing changes. Include new helpers, wrappers, tables, configuration
and changed callers even when moved outside the original files. Splitting code
into atomic functions must not improve the acceptance score. Report aggregate
source and decisions alongside function count, call layers and responsibility
distribution; reduced per-function complexity is not evidence of compression.
Use a wider module/project accounting boundary to detect exported complexity.

## Product correction, 2026-11-01

Develop through empirical replacement trials before expanding infrastructure. Count production implementation separately from automated tests, fixtures, experiments and dependencies. Tests remain valuable behavioral evidence; their size is not production feature implementation. Start with Lesscode, establish repeatable substantial feature-preserving reductions, then evaluate larger grown projects around 150k lines with explicit population definitions. Failed or marginal replacement trials are retained as evidence and must not be reported as successful simplification.

The primary outcome is to remove unnecessary implementation: replace custom machinery with existing framework, dependency or standard-library capabilities, and express required behavior with experienced, idiomatic code. Complexity and size measurements support this judgment; building a general evidence platform is not the outcome.

The owner identified Lesscode itself as disproportionate to its delivered features. Before extending the current implementation queue, assess its feature-to-code footprint, identify unnecessary custom implementation and existing capabilities that replace it, and prove a substantial simplification on Lesscode itself. Preserve features, required behavior, safeguards and public contracts. Move implementation responsibilities into suitable existing framework, library or language capabilities; use framework shorthands and idiomatic code to remove boilerplate and supporting layers. Moving the same custom code elsewhere, introducing a wrapper, or removing a feature does not establish simplification. Account for dependency, configuration and integration burden when evaluating replacements. Do not infer reduced maintenance effort from line counts. Preserve current uncommitted work until its disposition is decided.

Build a local-first system that measures and reduces convolution in AI-generated codebases and lets developers set measurable coding rules for agents to prevent regressions. A Codex skill is an optional workflow interface, not the product's complete solution.

The user clarified on 2026-10-31 that prevention and retrospective improvement both matter. Agent rules must use observable criteria and executable checks, with configurable thresholds and change budgets. The Anthropic code-simplifier agent is a side reference for policy wording, not a replacement for the measured evaluator or its quality gates.

The priority is preserved behavior, security and public APIs, followed by reduced complexity, duplication and unnecessary code. Prefer deletion, inlining, reuse and native features before introducing abstractions. GoF patterns require demonstrated benefit.

The core workflow must require no SciTools installation, commercial analyzer, license key or hosted analysis account. Analysis should run independently of Codex; LLM usage costs remain separate. Code style does not reliably establish AI authorship.

Status: efficiency-baseline is complete with final validation GO; its evidence remains historical and valid for the recorded revisions. Agent-quality-rules retains its approved historical plan; tasks.md marks 9 of 25 executable tasks complete, and remaining work is retired from the active delivery plan. Preserve uncommitted implementation. The experimental lesscode-discovery skill exists; measured skill efficacy remains unestablished. ATS01 and NPU01 demonstrate capability-scoped compression in two manually selected trials. The active capability-compression-core requirements, Rust design and 22-task implementation plan are approved through explicit task-generation -y. All 22 tasks are implemented and independently reviewed. The native CLI is verified on Windows with accepted external Python/TypeScript trials; the supported Python self-trial was adopted with 29 fewer statements and 32 fewer physical lines. Full cross-platform readiness remains pending current Linux/macOS integrated evidence. Use nextest and pytest-xdist for the respective test suites; keep frozen execution provenance explicit. The broad rule engine, generic transactions, MCP/plugins, separate pattern adviser and solver integration are removed from this delivery scope, not queued prerequisites. Reintroduce only after a concrete unmet need and a new scope decision. Python and TypeScript are target languages; English is the documentation language.
