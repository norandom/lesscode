# Approved selection contract amendment

Status: approved by the user on 2026-10-31. The correction is incorporated into design.md and tasks.md; task 2.2 implementation may resume.

## Problem established during task 2.2

The approved `select_observations(evidence, rule) -> tuple[EntityObservation, ...]` interface cannot distinguish a confirmed zero-function population from unavailable evidence or an out-of-run scope. It also has no per-rule, per-side diagnostic channel.

`EntityObservation` requires a real function location and name for function CCN. An unavailable function inventory cannot honestly be represented by a fabricated observation. `EvidenceInventory` forbids entities when unavailable; making a whole bundle unavailable would discard independent usable scalar evidence. Its global identities also support later entity-treatment validation.

A fresh debug investigation returned `SPEC_CONFLICT`, `STOP_FOR_HUMAN`, high confidence. No task 2.2 production changes were made. Requirements remain valid; the service contract and task boundary need correction.

## Proposed interface

Replace the tuple return with one versioned per-rule result:

```python
@dataclass(frozen=True)
class RuleSelection(Record):
    schema_version: int
    rule_id: str
    baseline: EvidenceInventory | None
    candidate: EvidenceInventory
    comparison_status: Literal["not_required", "comparable", "unavailable"]
    comparison_reasons: tuple[str, ...]

def select_observations(evidence: EvidenceBundle, rule: NumericRule) -> RuleSelection:
    ...
```

Reuse `EvidenceInventory` for each rule's selected population. Its source identity, explicit availability, reason and observations already express the required side-local information. Do not change its existing invariants or baseline formats.

- A complete empty candidate inventory means a confirmed zero-function population, supported by complete coverage of a matching nonempty product scope. Consumers explicitly disclose that there are no applicable functions.
- Missing inventory, omitted nonempty files, ambiguous empty scope or aggregate narrowing produce an unavailable selected inventory with the reason. They never become complete empty selections.
- Keep available candidate observations when reference evidence or comparison is unavailable.
- `comparison_status` concerns measurement definition, producer, unit, scope and population compatibility. It does not establish entity continuity or behavior equivalence.
- Use `not_required` for absolute-only rules with no change budget. These may omit baseline selection; absent reference metrics must not block candidate-only evaluation.
- Reference-dependent rules retain a baseline inventory, including its explicit unavailable reason where applicable. Missing or incompatible comparison evidence uses `unavailable` and nonempty comparison reasons.
- Comparable selections require complete side inventories and no comparison-failure reasons. Entity matching remains task 2.3 and may independently withhold a ratchet pass.
- Each result identifies the rule it represents. Canonical parsing rejects unknown versions, malformed identities and inconsistent status/reason combinations.

Keep `EvidenceBundle.report` unchanged. Bundle inventories remain the stable source of entity addresses for treatment validation. Per-rule selections filter or interpret that evidence without overwriting global availability or changing entity IDs according to rule order.

## Proposed task changes after approval

Task 2.2 will additionally own `RuleSelection` in `rules/contracts.py` and its round-trip/negative tests in `tests/test_rule_contracts.py`. Its boundary becomes **Evidence reader, Policy contracts**. Implement and review the contract before integrating selection, within this same task. No new dependency or baseline schema is required.

Task 3.1 will explicitly consume `RuleSelection`: unavailable blocking selections become insufficient findings, advisory selections retain unavailable reasons, confirmed empty populations are disclosed, and absolute-only candidate evaluation does not require reference availability. Task 2.3 retains ownership of entity continuity and treatment IDs.

No product requirements, analyzer behavior, policy limits, enforcement levels or approval metadata change automatically with this proposal.

## Required acceptance cases

1. Confirmed zero functions differs from absent inventory and outside-scope selection.
2. Missing complexity evidence does not erase a separately usable scalar selection.
3. An absolute-only candidate selection works without baseline metrics.
4. Unavailable reference evidence retains available candidate values and explicit comparison reasons.
5. Narrowing an aggregate or changing its definition, producer, version, unit or population cannot yield a supported comparison.
6. Short-file omissions stay unavailable; duplication retains pair-span denominators and ratios above one.
7. Selection order does not change bundle inventories or treatment entity IDs.
8. Serialization preserves statuses, reasons, observations and source identities; contradictory combinations fail closed.

After approval, update the design and task boundary, implement with failing-first tests, obtain independent review, and run the canonical regression before marking task 2.2 complete.

## Verified implementation checkpoint

Tasks 1.1–1.5 and 2.1 are complete and independently reviewed. Commit `7052b5d` is pushed to `origin/main`. Task 2.1's current-source regression passed 439 tests in 813.105 seconds, exit 0, with 11 established platform skips. This checkpoint does not claim feature completion or skill effectiveness.
