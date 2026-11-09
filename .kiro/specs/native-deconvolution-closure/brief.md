# Brief: Native Deconvolution Closure

Amended 2026-11-09 after cross-spec review.

## Problem and boundary candidates

Recording and verification span Rust control code and Python runtime support.
The objective is a smaller reusable native controller with explicit target-runtime
adapters, preserving behavior and evidence contracts. A compiled controller does
not eliminate the runtime needed to execute a Python specimen.

Candidates: reuse existing state/metric contracts; move orchestration and evidence
comparison to Rust; retain counted runtime hooks for function execution, database
listeners, reset/seed/cleanup and streams; preserve provider token counting and
streaming. Evaluate complete alternatives rather than porting every module.

## Scope and constraints

Own the common versioned evidence contracts and Python adapter. TypeScript, Java
and Rust are sibling downstream adapters with independent prototypes.
Keep frozen checks/goldens and owner WIP unchanged. Count all maintained
instrumentation regardless of file location. Respect R41's 7,173 canonical-line
no-growth ceiling and registered per-file/function budgets.

[Requirements](requirements.md), [provisional design](design.md) and
[prototype gate](prototype-gate.md) replace the previous unconditional
shim-removal plan. Production tasks are withdrawn pending evidence and phase review.
See [review amendment](review-amendment.md).
