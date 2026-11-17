# R118: Predicate work and local rebinding can be separated conservatively

- Report: 2026-11-17, 13:19 Europe/Berlin.
- Execution interval: approximately 13:13-13:19 Europe/Berlin.
- Status: completed for this instrument increment; P0 remains incomplete.
- Spec: [prototype gate](../../../.kiro/specs/model-simplification/prototype-gate.md).
- Hypothesis: H-MS1 preparation; predecessor [R117](04-model-simplification-census-pins.md).
- Type: exploratory instrument development, not a candidate trial.

## Decision

Retain the predicate extractor, lexical owner identities and conservative binding
classifier. Do not qualify simplicity comparisons yet: aliases, target dispersion,
opaque calls and implicit decisions remain unresolved. No model call was made.

## Hypothesis and falsification

Folding conditions must retain decision work; moving Boolean expressions into
assignments/returns must not erase them. Local rebinding must differ from explicit
module writes without hiding calls or augmented-assignment mutation. Formatting and
local renaming must not change owner identities. The targeted checks test these
claims; they do not establish completeness for arbitrary Python.

## Inputs and method

The research crate and existing dependencies are unchanged in scope. Profile rules
remain data in [profile.json](../../experiments/model-simplification/spike/profile.json).
`syntax-census-draft-3` adds assignment/condition selectors. Nested selector grammar
kinds and condition selector exclusivity are validated before source extraction.
Owner paths use lexical declarations and same-name ordinals; source offsets locate
facts but no longer define owner identity. Reordered/renamed declarations still need
an explicit correspondence before comparison.

## Evidence and results

Twelve nextest checks pass; locked offline build passes. New checks cover folding,
assigned/returned predicates and negation, comparison chains, deduplication across
selectors, stable lexical ownership, explicit globals, local RHS calls, nonlocal
uncertainty and augmented assignments that may mutate aliased values.

Final real-source reports are retained under
`D:\BuildArtifacts\lesscode\model-simplification\ms1\census-v3-final\`, including
the exact profile copy. Both still declare `not_measurable`. Exporter emits 51
predicate observations, 28 L and 92 U facts; tools emits 83 predicate observations,
51 L, 11 W and 103 U facts. These are draft instrument observations, not qualified
simplicity baselines or improvements. Earlier census directories remain intact.

## Limits and interpretation

Attribute/subscript writes, nonlocal bindings and alias effects are unresolved.
Augmented assignment remains U because rebinding syntax can invoke in-place mutation.
Implicit iteration decisions, opaque call logic and Boolean type inference are not
complete. Synthetic positive checks cannot qualify these omitted semantics. All
reports retain the unmeasurable status; there is no D/S claim or useful candidate.

## Tool/spec consequence and next gate

P0 still needs the remaining semantic profile/controls, broader observation pins,
private step histories and a complete execution manifest. Current cumulative
maintained apparatus is 542 lines: main 225, predicates 126, bindings 122, identity
29 and PowerShell 40. This stays below 1,400 without refunding moved/deleted code;
pins remain 131 physical lines pending canonical formatting. No production files,
source specimen or mutation pack changed. Approval remains valid within the protocol.

## COBESY check

Instrument corrections are separated from code simplification. Unsupported semantics
remain visible and can prevent a claim. Evidence supports only the named bounded checks.
