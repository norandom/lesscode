# R29: S9 surface-contract comparison (tree-only vs tree+recorder)
- Report date: 2026-11-07, Europe/Berlin.
- Execution interval: 2026-11-07, minutes; receipt `native-prototype/results/s9-surface-contracts.json` (+ `.sha256`).
- Status: evidence collected under the approved plan; NO ratification (OP-2 remains an owner decision).
- Spec: [prototype plan](../../../.kiro/specs/deconvolution-native-core/prototype-plan.md) section 7 (S9).
- Type: disposable analysis over existing apparatus; zero new maintained code charged to U7.

## Conclusion first
R26 amendment 1 ("surface = tree imports plus recorder imports") has favorable, cheap,
well-bounded evidence on both frozen scopes:

| Scope | A (tree-only) names | B additions | Additions |
| --- | --- | --- | --- |
| E3 (five decision_evidence modules) | 62 | 1 | `decision_evidence_publication._canonical` (private function the recorder binds) |
| E4 (seven lifecycle modules) | 37 | 3 | `lifecycle_projection.labeled_reason` (function), `lifecycle_evidence.LiveEvidence` (class), `lifecycle_store.<module>` (whole-module import) |

- `labeled_reason` is exactly the name whose absence from contract A let E4 turn 2 fail
  at arm collection; under B the regenerator program's rule 2 (preserve every surface
  name, same kind/signature) would have covered it before any arm ran. The same holds
  for `_canonical` (E3) and `LiveEvidence` (E4): recorder-bound names with no tree importer.
- All additions resolve to existing kinds in the frozen specimen (none were `unresolved`
  in A), so B adds no phantom requirements — it only extends protection to names the
  recorder already depends on.
- B does not adopt anything: ratification of the four R26 changes stays with the owner
  (OP-2). Historical protocols and recorded decisions are untouched; no surface file
  was rewritten (the analysis only reads the frozen `public-surface.json` files and
  scans the recorder sources with the same `scan()`/`kinds()` the derivers use).

## Method
`s9_surface_contracts.py` imports the existing `surface.py` (E3) and `surface4.py` (E4)
derivers, parses `e1b_oracle.py` / `e4_oracle.py` with the same AST scan, and diffs the
recorder-derived items against the frozen contract A per module/name/kind. Frozen
specimen root `.runs/e2-facdrone/source` (untracked; bytes present locally); failure to
read it fails closed.

## Amendment 1: contract ratified for S10 (owner decision, 2026-11-07)
The owner ratified contract B (tree + recorder imports) together with the four R26
changes (surface includes recorder imports; turn parameters preregistered with
thinking-only turns counted as failed; content hashes over bytes as stored;
case-distinct receipt labels) and R25's formatting and both-tree sensitivity rules as
written. S10 executes under this contract; historical protocols and recorded decisions
stay unchanged. OP-2 moves from pending to ratified-for-S10.

## Limits and interpretation
- Evidence covers the two frozen scopes only; it does not predict unseen scopes.
- The `<module>` item for `lifecycle_store` reflects the recorder's whole-module import;
  ratifying B means the module object identity itself becomes surface.
- S9 does not measure sensitivity effects on candidates (that is S10/U6 territory under
  whichever contract is ratified); it measures only the contract delta.

## Tool/spec consequence and next gate
Present this evidence with OP-2 (four R26 changes incl. this one) to the owner before
S10 ratification freezes the contract; S10 must run under a resolved, ratified contract
per prototype-gate.md. U7 ledger unchanged (774/1,400).

## COBESY check
Target behavior: agents measure the actual delta of a proposed contract change on frozen
evidence before anyone ratifies it, instead of arguing semantics in the abstract.
Challenge the assumption that "recorder imports" is a large or vague set: on both scopes
it is one to three concrete, resolvable names.
