# R48: Aligned provisional language adapter designs

- Report date: 2026-11-09, Europe/Berlin.
- Execution interval: not run; source/document/API review only.
- Status: completed provisional design generation; feasibility untested.
- Specs: [native closure](../../../.kiro/specs/native-deconvolution-closure/design.md),
  [TypeScript/JS](../../../.kiro/specs/typescript-deconvolution-adapter/design.md),
  [Java](../../../.kiro/specs/java-deconvolution-unification/design.md),
  [Rust](../../../.kiro/specs/rust-deconvolution-unification/design.md).
- Predecessor: [R47](05-deconvolution-spec-review-amendment.md).
- Type: prospective design and bounded prototype plans.

## Decision

The owner requested designs based on the revised requirements. Record that as
requirements approval for this design phase; designs remain provisional,
prototype gates not-run and production tasks absent.
Native closure owns a concrete [shared contract](../../../.kiro/specs/native-deconvolution-closure/adapter-contract.md);
all language adapters consume it with explicit supported profiles and runtime
instrumentation. No Java-to-Rust dependency.

## Hypothesis and falsification

Each language can exercise real target semantics through a counted adapter and
enable useful whole-responsibility compression without weakening the oracle.
Falsifiers: missing hooks, unresolved semantics, incomplete finite domains,
surviving controls, incompatible metric definitions or apparatus overage.
The per-language gates preregister 20% lines/10% statements and absolute
50 canonical lines/10 statements removed. Native closure retains its own
separate migrated-apparatus target and gate.

## Frozen inputs and method

No trial manifest has been frozen and no source execution was performed.
Review inputs: revised specs/steering, native Cargo.toml, metrics.rs, measures.rs,
recording.rs, TS01 historical result and official Node/Oxc/JUnit/JDK/Syn/nextest
documentation cited in the per-spec research files. Installed versions are not
inferred from latest web documentation. Specimen admission pins exact versions.

## Evidence and results

The manifest enables Python/TypeScript grammars, not Java. Existing Language
variants do not implement a general adapter registry. Use a separate internal
profile registry and explicit compatibility review for public schema changes.
TS01 is a candidate-admission lead, not evidence for the new reduction bar.
Nextest/JUnit test results are distinct from source-bound state/stream traces;
dedicated probes carry the latter. Rust interior mutability and Java mutable
referents prevent purity conclusions from surface syntax.
No measured performance, semantic parity or successful compression is claimed.

## Limits and interpretation

TypeScript and Java specimen admission remains open; the Rust recording-lifecycle
responsibility is a screening candidate, not a certified feasible target.
Runtime hooks, parser alternatives, formatter identities and whole-budget fit
must be demonstrated. All new hand-maintained probes/wrappers are counted.

## Tool/spec consequence and next gate

Review documentary alignment, then perform bounded admission/prototypes.
Disposable implementations may exercise the contract before production components
exist, avoiding a bootstrap dependency. Production plans require proceed and
ordinary design approval. Preserve unrelated WIP and historical research.

## COBESY check

Decisions precede detail; falsifiers and absent evidence are explicit. Negative
results remain valid outcomes. Documentary alignment is not experimental success.
