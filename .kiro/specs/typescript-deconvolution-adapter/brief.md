# Brief: TypeScript and JavaScript Deconvolution Adapter

Amended 2026-11-09. Qualify a bounded TypeScript and JavaScript adapter over the shared native
contracts, preserving language semantics through actual runtime execution.


## Boundary candidates

Analysis and runtime observation of state effects; finite-domain comparison;
targeted adverse controls; whole-replacement accounting. Initial runtime:
Node.js and one specimen-native test runner; Bun and other runners require separate qualification. Specimen: one real TypeScript/JavaScript responsibility selected and frozen before prototype execution.

Reuse candidate: Existing ast-grep/tree-sitter; compare Oxc scope/symbol analysis before adding custom machinery. Syntax discovery does not establish purity.
Unsupported effects return unverified; harness failures never count as behavioral
success or mutation kills.

## Constraints and next phase

Depend on native closure's qualified common contract, not another adapter.
The [provisional design](design.md) and [prototype gate](prototype-gate.md) are generated following the owner request. No production task plan exists or is authorized.
Respect R41 whole-apparatus accounting, applicable measured registry ceilings and
new-file/function defaults. Count maintained runtime adapters wherever stored.


See [requirements](requirements.md) and [review amendment](review-amendment.md).
