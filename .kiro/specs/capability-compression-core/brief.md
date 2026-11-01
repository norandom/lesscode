# Brief: capability-compression-core

## Problem
Function-level simplification can distribute convolution into more helpers.
ATS01 and NPU01 instead reduced complete capabilities under frozen checks.
Turn that demonstrated procedure into a reusable tool without porting the entire
Python measurement platform or claiming automatic discovery is already proven.

## Desired Outcome
A local native tool and skill workflow compare complete capability replacements,
preserve software behavior and expose net implementation changes and uncertainty.

## Boundary Candidates
- Own capability scope, whole-replacement accounting and evidence-bound comparison.
- Consume existing project tests, type checks and differential checks.
- Let the skill select responsibilities and propose simpler implementations using
  existing repository capabilities; do not equate AST similarity with equivalence.
- Exclude hardware/performance, feature removal, Java, automatic synthesis and
  MCP/plugin integration. Keep source specimens unchanged.
- Drop broad policy engines, generic transaction/codemod infrastructure, solver
  integration and multiple rewrite backends from this delivery scope. Do not
  introduce extension interfaces for them ahead of demonstrated need.

## Evidence and proposal
See [native direction](../../../docs/reports/native-compression-direction.md).
Rust is the recommended implementation; Python remains the research reference.
This is a new, narrower successor path, not an approval to replace the existing CLI.
The owner approved requirements, design and tasks with task-generation `-y`.
Independent task-graph review passed; 22 executable tasks are ready for implementation.
All 22 implementation tasks are reviewed and implemented. The scoped Windows
workflow and Python self-application are verified; final validation remains
MANUAL_VERIFY_REQUIRED pending current Linux/macOS integrated evidence.

## Maturity gate
After the first working slice passes its fixtures, use Lesscode on a complete
capability in this repository. Freeze the evaluator and checks; preserve the
baseline unless a safe benefit is demonstrated. Record both accepted and rejected
results. This gates scoped maturity claims, not bootstrap. Carry requirement 5
into design, tasks and final validation without adding a new platform layer.
