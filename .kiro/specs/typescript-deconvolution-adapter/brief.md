# Specification Brief: TypeScript Deconvolution Adapter

## 1. Problem Statement
While `lesscode-core` supports TypeScript and TSX at the static syntax and comparison level (`ast-grep` / `tree-sitter-typescript`), deconvolution and state-boundary verification (mutable frame analysis, bounded pure core verification, adverse mutation testing, and dynamic trace recording) currently only execute against Python specimens. 

To achieve multi-language deconvolution parity, TypeScript must be elevated from passive comparison to full dynamic deconvolution.

## 2. Proposed Approach
Implement the TypeScript deconvolution adapter adhering to the unified core traits established in `native-deconvolution-closure`:
1. **TypeScript StateFrame Analyzer:** AST detection of property mutations (`this.x = y`, `obj.prop = val`, module-level `let` assignments).
2. **TypeScript Pure Core Verifier:** Static purity enforcement (zero property/variable writes, zero external I/O) and bounded equivalence checking across TypeScript primitive and object domains.
3. **TypeScript Adverse Mutant Generator:** AST mutation operators injecting state-boundary, exception-handling, and stream-buffering faults into TypeScript source.
4. **TypeScript Specimen Runner:** Direct Node.js / Bun IPC execution runner supporting Vitest and Jest test batteries.

## 3. Scope & Boundaries
- **In Scope:**
  - TypeScript AST state frame analysis and purity checking.
  - TypeScript adverse mutant generation and kill-rate gating.
  - Node.js / Bun specimen execution runner.
  - Bounded prototype trial on a representative TypeScript specimen.
- **Out of Scope:**
  - Modifying the existing Python or native Rust core modules.
  - Relaxing verification criteria or line reduction targets ($\ge 20\%$ lines, $\ge 10\%$ statements).

## 4. Constraints
- Must implement the unified `StateFrameAnalyzer`, `PureCoreVerifier`, `AdverseMutantGenerator`, and `SpecimenTraceRunner` traits.
- Per-file complexity caps ($\le 400$ physical lines, $\le 300$ syntax units) strictly enforced.
