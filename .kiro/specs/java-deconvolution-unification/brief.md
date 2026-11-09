# Specification Brief: Java Deconvolution Unification

## 1. Problem Statement
Java represents a massive ecosystem of enterprise software frequently plagued by bloated design patterns, redundant intermediate wrapper classes, and scattered mutable state. Currently, Java is excluded from Lesscode's active target languages in `.kiro/steering/product.md`.

To expand automated code deconvolution and structural consolidation to the Java ecosystem, Java must be formally admitted as a target language, integrated into `lesscode-core`'s parser stack, and brought to full deconvolution parity.

## 2. Proposed Approach
1. **Scope Amendment:** Formally amend `.kiro/steering/product.md` to add Java to authorized target languages.
2. **Crate Integration:** Enable `tree-sitter-java` on `ast-grep-language` in `native/Cargo.toml`.
3. **Java State-Boundary Analyzer:** Deterministic detection of field mutation (`this.field = val`, `static` modifications), leveraging Java's static type system and access modifiers (`final`, `@Immutable`).
4. **Java Pure Core Verifier:** Static purity auditing of static methods and pure helper classes, with bounded equivalence checks across Java primitive and object domains.
5. **Java Adverse Mutation Engine:** AST mutant injection targeting `catch` blocks, `PrintStream` buffer flushes, and state assignment sites.
6. **JVM Specimen Runner:** Execution driver using standard Maven/Gradle and JUnit test runners.

## 3. Scope & Boundaries
- **In Scope:**
  - `tree-sitter-java` integration into `lesscode-core`.
  - Java state-boundary, purity, and density metrics.
  - Java adverse mutation testing with 100% kill-rate enforcement.
  - JVM specimen test execution runner.
  - Bounded prototype trial on a representative Java real-code specimen.
- **Out of Scope:**
  - Unbounded formal proofs across arbitrary JVM bytecode.
  - Dynamic bytecode modification or non-standard JVM agents outside the test runner.

## 4. Constraints
- Requires formal owner scope amendment in `.kiro/steering/product.md`.
- Prototype feasibility gate required on a real Java specimen before production tasks.
- Must respect the 400-line per-file complexity ceiling.
