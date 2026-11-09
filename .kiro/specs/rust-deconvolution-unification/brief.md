# Brief: Rust Deconvolution Adapter

Amended 2026-11-09. Qualify a bounded Rust adapter over the shared native
contracts, preserving language semantics through actual runtime execution.


## Boundary candidates

Analysis and runtime observation of state effects; finite-domain comparison;
targeted adverse controls; whole-replacement accounting. Initial runtime:
pinned Rust toolchain and cargo-nextest, including relevant build features and target. Specimen: an isolated copy of one complete Lesscode native responsibility, with the evaluator and expected outcomes frozen outside the candidate.

Reuse candidate: Reuse syn/proc-macro2 for syntax; semantic resolution or instrumentation must be separately demonstrated. Syntax discovery does not establish purity.
Unsupported effects return unverified; harness failures never count as behavioral
success or mutation kills.

## Constraints and next phase

Depend on native closure's qualified common contract, not another adapter.
The [provisional design](design.md) and [prototype gate](prototype-gate.md) are generated following the owner request. No production task plan exists or is authorized.
Respect R41 whole-apparatus accounting, applicable measured registry ceilings and
new-file/function defaults. Count maintained runtime adapters wherever stored.
Self-application uses an isolated candidate and frozen external evaluator. Registry membership does not prohibit reductions; exceeding a registered ceiling or changing accounting requires an explicit amendment. No adoption into native/src occurs during prototype qualification.

See [requirements](requirements.md) and [review amendment](review-amendment.md).
