> Superseded on 2026-11-09 by [review-amendment.md](review-amendment.md).
> The PASS below records the earlier review only; it does not approve the revised
> requirements/design or certify prototype feasibility.
# Requirements Review

**Date:** 2026-11-09  
**Specification:** `native-deconvolution-closure`  
**Mode:** Inline review gate (Scientist Mode / COBESY alignment)  
**Result:** PASS

---

## 1. Mechanical Checks
- **Numeric IDs Present:** All requirement sections use numeric headings (`Requirement 1` through `Requirement 6`).
- **EARS Acceptance Criteria:** Every acceptance criterion (30 criteria across 6 requirements) follows strict EARS syntax (`When [event]`, `If [trigger]`, `While [precondition]`, `Where [condition]`, `The [system] shall`).
- **No Premature Implementation Language:** Avoids prescribing internal trait hierarchies or concrete internal types that belong in technical design. Focuses on system-observable contracts, boundary enforcement, density metrics, mutation kill rates, digest compatibility, and budget enforcement.

---

## 2. Scope & Coverage Review
- **Coverage of Python Apparatus Transfer ("All"):**
  - *Native State Frame & Boundary Enforcement (Requirement 1):* Addresses porting `src/lesscode_baseline/state_boundary/frame.py` into native analysis and verification.
  - *Native AST Density & Structural Consolidation Metrics (Requirement 2):* Addresses porting `density.py` into native Rust metrics using `ast-grep` / `tree-sitter`.
  - *Native Bounded Formal Equivalence (Requirement 3):* Addresses porting `pure_verifier.py` into native Rust property-based / domain-checking logic.
  - *Native Stream Monitoring & Adverse Mutation Gates (Requirement 4):* Addresses porting `stream_monitor.py` and `mutant_generator.py` into native Rust.
  - *Native Specimen Execution & Recording Engine (Requirement 5):* Addresses eliminating `native/shim/recording_shim.py` and establishing native execution and observation capture.
  - *Native Refine Loop Integration & Budget Governance (Requirement 6):* Addresses porting streaming / token guards in `regenerator.rs` and enforcing `complexity-budgets.json`.

---

## 3. Boundary & Adjacent Systems
- **Boundary Continuity:**
  - Brief: `Boundary Candidates` (frame conditions, density metrics, pure equivalence, adverse mutants, recording shim elimination, token/streaming refine loop).
  - Requirements: Explicit inclusion/exclusion and adjacent expectations.
  - Deferred to Design: File structure plan, crate dependency additions (if any), prototype gate accounting under `prototype-first.md`.
- **Adjacent System Expectations:**
  - Preserves CLI subcommands (`prepare`, `compare`, `admission`, `budgets`, `surface`, `regenerate`), exit codes, and frozen LF-body digests.
  - Preserves the 4 owner WIP files strictly untouched.
