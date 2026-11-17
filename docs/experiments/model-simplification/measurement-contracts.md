# H-MS1 bounded measurement contracts

Written 2026-11-17 during P0. This is the initial qualification proposal, not
an admitted profile or a positive experiment result. The earlier syntax-census
drafts are retained exploratory preparation, not successful qualification runs.

## Scope and evidence boundary

Use the frozen source and observation population in `freeze.json`. The whole
exporter file is the S accounting envelope. B owns the four named validation
functions, their shared constants and all replacement support; its envelope is
the whole tools module. Every new helper/table/caller is charged. Tests protect
behavior and are not a reduction target. Model source is never hand-edited.

The spike emits facts, source locations, conditional write relations and per-owner
B/P/E/L/W/U totals. It does not issue a behavioral verdict or declare useful code.
The existing step checker retains that authority. Qualification of this bounded
profile and calculation of baseline ceilings occur in P1 before any model call.

## Semantic contracts requiring qualification

- **S1, field family:** `self` is the first unshadowed parameter of an ordinary
  instance method in the frozen class. Direct field assignments refer to a lexical
  instance-field family, not an individual runtime object. The class has no custom
  attribute dispatch or descriptors for those fields. New decorators, receiver
  rebinding, descriptor machinery or dynamic field access invalidate this contract.
- **S2, list mutation:** the one admitted mutation selector is
  `XLikesExporter` / `self.tweets.extend`. In the observation boundary the retained
  tweets are ordinary lists. Its write target is the same field family as direct
  assignments. This is an explicit runtime contract, not inference from a method
  name or annotation. Unknown receiver types/dispatch remain outside a qualified
  state comparison. Alias identity, duplicates, order and partial failures are
  observed; allocation in place of extension must fail those pins.
- **U, external effects:** all other calls, unresolved bindings and alias mutations
  remain visible. No unknown is counted as zero. A candidate must preserve the
  multiset of unresolved effect expressions and their qualified owners, and must
  not introduce a new relevant effect, escape, state representation or opaque
  predicate. Equality of that multiset is necessary, not sufficient: review the
  arguments, order and reachability against the frozen observations. If their
  non-interference cannot be justified, report NotMeasurable. Do not assume purity.
- **P, decisions:** comparisons, atomic condition leaves and short-circuit work
  count at each physical occurrence, including assigned/returned expressions.
  Iteration sites contribute B. Logic hidden in calls, dispatch or data that the
  profile cannot account for makes the affected comparison NotMeasurable.
  `isinstance`, regex matching and error factories retain their existing contracts;
  candidate changes to those operations require accounted logic and frozen pins.

Straight-line alias resolution, arbitrary Python dispatch and a general effect
analysis are not implemented. An unsupported candidate is a recorded outcome,
not a reason to extend the profile during a frozen trial. No claim covers arbitrary
list subclasses, dynamically modified classes or unobserved external effects.
This restriction must be explicitly reviewed at P1; if it does not satisfy the
approved responsibility boundary, state qualification fails, not silently narrows.

## Comparison procedure

1. Verify all frozen hashes, source/ref identities, single-file scope and unchanged
   public signatures. For B, reject changes outside the four bodies and new private
   helpers; shared constants and all other declarations remain unchanged.
2. Run the same frozen spike on both files. Sum owner totals across the complete
   responsibility and the module envelope. Retain all raw facts. Declaration
   renames/reordering require explicit correspondence; do not merge owners by name.
3. For each resolved target retain sites and writers; W is site count,
   S is `max(0, sites - 1)`, D is `max(0, writers - 1)`. Sum S/D over the full envelope.
   Report L and U separately, and check all non-worsening companions in the gate.
4. S requires target W/S reductions of at least one and 25%; B requires B reduction
   of at least two and 20%, P reduction of at least one and 10%. Derive integer
   ceilings from qualified baselines before calls. Zero cannot satisfy a relative
   target. Mere relocation, branch folding or merging writer functions cannot pass.
5. A model answer is useful only after the original step flow accepts behavior and
   every structural/observation prerequisite passes. Accepted behavior ends a trial
   even for unchanged, mixed, regressed or NotMeasurable structure.

Comparison review and manifest inspection are foreground/manual prototype work,
disclosed separately from deterministic extraction and the existing step/seam APIs.
No automatic whole-program equivalence or automatic selection claim is made.
