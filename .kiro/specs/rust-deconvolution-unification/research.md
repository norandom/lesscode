# Research and Design Decisions: Rust

Date: 2026-11-09. Discovery: complex integration. No experimental execution.
Skills: kiro-spec-design and its discovery/synthesis/review rules; unified-memory.
Review is inline under the owner foreground rule.

## Summary

Use the shared native protocol and outcome policy with explicit Rust runtime
bindings. Syntax analysis, behavioral probes and regression reports remain distinct.
The design introduces no runtime/dependency changes and claims no measured parity.

## Research Log

Local evidence: [Cargo manifest](../../../native/Cargo.toml),
[specimen/source context](../../../native/src/recording.rs),
[shared design](../native-deconvolution-closure/design.md), and R27/R41/R47.
Current Cargo enables Python/TypeScript grammars and syn, not Java.
Legacy Language variants do not yet constitute a general adapter registry.

- [Syn syntax API](https://docs.rs/syn/2.0.119/syn/) — consulted for component role; exact selected-runtime compatibility still requires a pinned prototype.
- [Rust interior mutability](https://doc.rust-lang.org/stable/std/cell/) — consulted for component role; exact selected-runtime compatibility still requires a pinned prototype.
- [nextest output formats](https://nexte.st/docs/machine-readable/) — consulted for component role; exact selected-runtime compatibility still requires a pinned prototype.
- [nextest JUnit](https://nexte.st/docs/machine-readable/junit/) — consulted for component role; exact selected-runtime compatibility still requires a pinned prototype.

Reuse syn 2.0.119 with full/visit and proc-macro2 span-locations from Cargo.lock. They provide syntax and source locations, not whole-program effect or trait resolution. V1 supports explicitly resolved calls and an inventoried closed subset; compiler-dependent semantic expansion is a deferred alternative, not a hidden dependency.

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
- Rust 1.98.1 and nextest 0.9.143 per docs/testing.md, frozen target triple, feature set, Cargo.lock, profile, panic strategy, overflow checks and environment. Run reference/candidate with matching configurations.
- Freeze rust-statements-v1 separately from rust-syntax-units-v1 budget units. Define semicolon statements, let bindings, tail expressions, match arms and nested items explicitly with packing controls. Format isolated copies with pinned rustfmt and unchanged edition/config; do not mix physical and canonical counts. Macros whose expanded behavior matters require supported accounting or leave the metric unavailable.
- Screen native/src/recording.rs record/replay process-lifecycle responsibility, including run_shim, callers and transitive helpers, in an isolated source copy. Freeze the complete closure and external evaluator before editing. If private access, required effects or realistic savings cannot be covered, return revise and select another complete native responsibility; do not force deletion or change the evaluator.
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
