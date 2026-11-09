# Research and Design Decisions: TypeScript / JavaScript

Date: 2026-11-09. Discovery: complex integration. No experimental execution.
Skills: kiro-spec-design and its discovery/synthesis/review rules; unified-memory.
Review is inline under the owner foreground rule.

## Summary

Use the shared native protocol and outcome policy with explicit TypeScript / JavaScript runtime
bindings. Syntax analysis, behavioral probes and regression reports remain distinct.
The design introduces no runtime/dependency changes and claims no measured parity.

## Research Log

Local evidence: [Cargo manifest](../../../native/Cargo.toml),
[specimen/source context](../../../docs/experiments/native-replay/typescript-run/results.md),
[shared design](../native-deconvolution-closure/design.md), and R27/R41/R47.
Current Cargo enables Python/TypeScript grammars and syn, not Java.
Legacy Language variants do not yet constitute a general adapter registry.

- [Oxc semantic analysis](https://docs.rs/oxc/latest/oxc/semantic/index.html) — consulted for component role; exact selected-runtime compatibility still requires a pinned prototype.
- [Node subprocess stdio](https://nodejs.org/api/child_process.html) — consulted for component role; exact selected-runtime compatibility still requires a pinned prototype.

Reuse ast-grep 0.51.0 and locked TypeScript grammar for syntax/metrics. Compare Oxc's symbol/scope graph with the specimen's installed TypeScript compiler API during the prototype; choose one source of binding identities. No Oxc version is approved by this design.

## Architecture Pattern Evaluation

| Alternative | Decision / reason |
|---|---|
| Native reimplementation of target semantics | Reject; use actual pinned target runtime |
| Treat test runner output as behavioral oracle | Reject; explicit source-bound trace hooks required |
| Reuse existing parser and closed effect profile | Selected provisional approach; fail closed on unresolved effects |
| Broader semantic framework or automatic instrumentation | Deferred until a concrete coverage gap and measured benefit justify it |

## Design Decisions

- Generalize only the shared request/outcome/trace contract. Keep language semantics
  and runtime binding local; do not create a second comparator or controller.
- Node 24.22.1 where compatible with the specimen lock; the specimen's exact TypeScript compiler/module mode and existing test runner are frozen at admission. A runtime mismatch requires a new profile, never a silent upgrade.
- Reuse native logical_statements.v1 and its grammar identity. Freeze a specimen-installed formatter and exact flags for canonical physical lines; missing formatter means admission stops. Add packing controls for semicolons, declarations, expressions and multiline literals. Preserve existing TS/TSX comparison receipts. JS files require explicit grammar/profile tests; JSX/TSX dynamic execution is excluded from v1 unless separately qualified.
- Start admission screening from the prior TS01 repository and its complete degradation responsibility plus callers. TS01 removed only 19 physical lines and is not a qualifying 20% canonical reduction result. If this family has insufficient remaining redundancy or no observable state/stream boundary, reject it and select a different licensed TypeScript family before freezing the trial.
- Full profiling/semantic support is not inferred from AST setup or past Python
  successes. Unsupported constructs become explicit gaps.
- Prototype selection must demonstrate a deletion hypothesis and whole-support
  accounting before infrastructure expansion.

## Risks and mitigations

Unresolved effects: reject unsupported profiles rather than infer purity.
Runtime interception: independently challenge every required hook.
Budget growth: expanded R41 inventory, no hidden adapter/generator exclusions.
Metric drift: pinned definitions and packing controls.
Specimen/dependency uncertainty: admission blocks execution until recorded.
See [design](design.md), [gate](prototype-gate.md) and
[R48](../../../docs/research/2026-11-09/06-aligned-language-adapter-designs.md).
