# Research and Design Decisions: Java

Date: 2026-11-09. Discovery: complex integration. No experimental execution.
Skills: kiro-spec-design and its discovery/synthesis/review rules; unified-memory.
Review is inline under the owner foreground rule.

## Summary

Use the shared native protocol and outcome policy with explicit Java runtime
bindings. Syntax analysis, behavioral probes and regression reports remain distinct.
The design introduces no runtime/dependency changes and claims no measured parity.

## Research Log

Local evidence: [Cargo manifest](../../../native/Cargo.toml),
[specimen/source context](../../../docs/architecture/multi-language-deconvolution.md),
[shared design](../native-deconvolution-closure/design.md), and R27/R41/R47.
Current Cargo enables Python/TypeScript grammars and syn, not Java.
Legacy Language variants do not yet constitute a general adapter registry.

- [ast-grep-language dependencies](https://docs.rs/crate/ast-grep-language/0.51.0) — consulted for component role; exact selected-runtime compatibility still requires a pinned prototype.
- [JDK Trees API](https://docs.oracle.com/en/java/javase/25/docs/api/jdk.compiler/com/sun/source/util/Trees.html) — consulted for component role; exact selected-runtime compatibility still requires a pinned prototype.
- [JUnit listeners](https://docs.junit.org/5.12.2/user-guide/) — consulted for component role; exact selected-runtime compatibility still requires a pinned prototype.

Propose enabling tree-sitter-java on the existing ast-grep-language dependency for syntax metrics only. This adds a lockfile dependency and needs a separate version/license/MSVC/platform decision. Resolve symbols via the selected JDK's public compiler Trees API in the counted adapter; missing classpath or unresolved elements means unverified.

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
- One specimen-declared JDK release, one Maven or Gradle wrapper and exact JUnit Platform/engine versions. Freeze classpath/module path, locale, charset, timezone and build options from the chosen specimen; no latest-version substitution.
- Define java-statements-v1 on a frozen Java grammar before measuring: count declarations and executable statements with explicit nested-body and multi-declarator rules. Keep expression/decision and physical/canonical line counts separate. Freeze a specimen-installed formatter; do not use raw layout or Python statement rules. Baseline/candidate use the same definition and packing controls. Java profile metrics do not silently change the legacy Language enum or comparator schema.
- No real Java specimen has yet been admitted. Select a licensed, locally buildable responsibility with mutable state, exception cleanup and an injectable output sink. Require baseline tests and a concrete deletion hypothesis before freezing a source revision; a synthetic example alone cannot qualify.
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
