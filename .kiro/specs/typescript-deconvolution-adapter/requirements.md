# Requirements Document: TypeScript Deconvolution Adapter

## Introduction

Define the requirements for extending the state-boundary deconvolution engine to full dynamic parity on TypeScript and JavaScript codebases.
Building upon the unified traits established in `native-deconvolution-closure`, this specification delivers native AST write-site detection, bounded pure-core equivalence, adverse mutation testing, and dynamic specimen execution for TypeScript.

## Boundary Context

- **In scope:**
  - TypeScript state frame extraction and write-site detection (`tree-sitter-typescript`).
  - Static purity verification and bounded domain property checking for TypeScript functions.
  - TypeScript adverse mutant synthesis targeting promise/exception rejection and buffer flushes.
  - Node.js / Bun specimen runner executing Vitest/Jest test suites over standard IPC.
- **Out of scope:**
  - Changes to the Python or Rust language adapters.
  - Modifications to the core refine loop or comparator algorithms.

## Requirements

### Requirement 1: TypeScript State Frame and Write-Site Analysis
1. When TypeScript source is analyzed, the TypeScript adapter shall extract mutable frame conditions identifying instance properties, module-level variables, and global storage writes.
2. When a TypeScript candidate is evaluated, the TypeScript adapter shall verify that all write sites reside strictly within declared imperative shell functions.
3. If an unauthorized property mutation or undeclared state leak is detected, the TypeScript adapter shall emit a line-anchored state violation receipt.

### Requirement 2: TypeScript Pure Core Verification
1. When functions are designated as pure, the TypeScript adapter shall statically confirm zero property writes, zero external method invocations, and zero ambient I/O calls.
2. When evaluating pure candidate functions against baseline logic, the TypeScript adapter shall verify bounded domain equivalence across numbers, strings, arrays, and schema edge cases.
3. If output divergence occurs, the TypeScript adapter shall emit a minimal reproducible counterexample witness.

### Requirement 3: TypeScript Adverse Mutation Testing
1. When adverse mutation testing is conducted, the TypeScript adapter shall inject targeted mutants into promise rejections, catch blocks, and stream/buffer flushes.
2. When candidate test suites execute against adverse mutants, the TypeScript adapter shall require a 100% kill rate before granting qualification.

### Requirement 4: Node.js Specimen Execution Runner
1. When recording or replaying execution sequences, the TypeScript runner shall drive Node.js / Bun processes over standard I/O pipes.
2. The TypeScript runner shall capture pre-abort console and stream outputs, enforcing buffer preservation and digest parity.
