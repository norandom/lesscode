# Requirements Document: Rust Deconvolution Unification

## Introduction

Define the requirements for achieving complete state-boundary deconvolution parity on Rust codebases.
Extending the unified trait model, this specification provides Rust AST write-site analysis, pure functional core verification, adverse mutation sensitivity gates, and dynamic specimen execution via `cargo-nextest`.

## Boundary Context

- **In scope:**
  - Rust state frame extraction and write-site detection via `syn`.
  - Static purity verification and bounded domain equivalence checking for Rust functions.
  - Rust adverse mutant synthesis targeting error-handling matches, stream flushes, and state mutations.
  - `cargo-nextest` dynamic specimen runner capturing test events and execution digests.
- **Out of scope:**
  - Unbounded formal verification of arbitrary `unsafe` blocks.
  - Changes to other language adapters.

## Requirements

### Requirement 1: Rust State Frame and Write-Site Analysis
1. When Rust source files are analyzed, the Rust adapter shall extract mutable frame conditions identifying `mut` bindings, `&mut` parameters, and interior mutability wrappers (`RefCell`, `Mutex`).
2. When a deconvolution candidate executes, the Rust adapter shall verify that all state mutation sites reside strictly within declared imperative shell functions.
3. If an unauthorized state mutation or undeclared state leak is detected, the Rust adapter shall emit a structured violation receipt.

### Requirement 2: Rust Pure Core Verification
1. When Rust functions are designated as pure, the Rust adapter shall statically verify that they accept no `&mut` parameters, access no static mutable items, and perform no I/O operations.
2. When evaluating pure candidate functions against baseline logic, the Rust adapter shall verify bounded equivalence across primitive integers, floats, options, results, and composite structs.
3. If output divergence occurs, the Rust adapter shall emit a minimal reproducible counterexample witness.

### Requirement 3: Rust Adverse Mutation Testing and Specimen Runner
1. When adverse mutation testing is conducted, the Rust adapter shall inject targeted mutants into `match` arm error patterns, buffer flushes, and state assignment expressions.
2. When candidate test suites execute against adverse mutants, the Rust adapter shall require a 100% kill rate before granting qualification.
3. When executing test batteries, the Rust runner shall drive `cargo-nextest` over standard I/O pipes, capturing pre-abort streams and test execution digests.
