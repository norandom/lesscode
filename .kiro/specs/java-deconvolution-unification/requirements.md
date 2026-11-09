# Requirements Document: Java Deconvolution Unification

## Introduction

Define the requirements for admitting Java as an authorized target language and establishing full state-boundary deconvolution parity for Java applications.
Following the unified trait model, this specification establishes native AST metrics, state frame enforcement, bounded pure-core equivalence, adverse mutation gates, and JVM execution runners for Java.

## Boundary Context

- **In scope:**
  - `tree-sitter-java` grammar integration into `lesscode-core`.
  - Java state frame extraction and field write-site analysis.
  - Java static purity auditing and bounded domain equivalence checking.
  - Java adverse mutant generation targeting exception handlers, buffer flushes, and state updates.
  - JVM execution runner driving JUnit test batteries.
- **Out of scope:**
  - Modifying existing Python, TypeScript, or Rust adapters.
  - Unbounded JVM bytecode symbolic execution.

## Requirements

### Requirement 1: Java Grammar and AST Metrics Integration
1. When Java source files are provided, the native metrics engine shall parse the source using `tree-sitter-java`.
2. When evaluating Java code, the native metrics engine shall calculate physical lines, canonical statements, decisions, and call depth.
3. The native metrics engine shall enforce the Density Trap gate requiring $\ge 20\%$ line reduction and $\ge 10\%$ statement reduction.

### Requirement 2: Java State Frame and Field Write-Site Analysis
1. When Java classes are analyzed, the Java adapter shall extract mutable state frames identifying private, protected, and static mutable fields.
2. When a deconvolution candidate executes, the Java adapter shall verify that all write sites are confined to declared imperative shell methods.
3. If an unauthorized field write or state leak is detected, the Java adapter shall emit a structured violation receipt.

### Requirement 3: Java Pure Core Verification
1. When Java methods are designated as pure, the Java adapter shall statically verify that they access no mutable fields, perform no I/O, and invoke no non-pure methods.
2. When evaluating pure candidate methods against baseline logic, the Java adapter shall verify bounded equivalence across primitive ranges, boxed numbers, and object contracts.
3. If output divergence occurs, the Java adapter shall emit a minimal reproducible counterexample witness.

### Requirement 4: Java Adverse Mutation Testing and JVM Runner
1. When adverse mutation testing is conducted, the Java adapter shall inject targeted mutants into `catch` blocks, output stream flushes, and state assignment statements.
2. When candidate test suites execute against adverse mutants, the Java adapter shall require a 100% kill rate before granting qualification.
3. When executing test batteries, the JVM runner shall drive JUnit over standard I/O pipes, capturing pre-abort streams and execution digests.
