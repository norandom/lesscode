# R115: Isolated proposals need a separate measurement profile and observable state boundary

- Report date/time: 2026-11-17, 12:41 Europe/Berlin (11:41 UTC).
- Execution interval: not run; source inspection and specification authoring only.
- Status: planned.
- Spec: [model-simplification design](../../../.kiro/specs/model-simplification/design.md),
  [prototype gate](../../../.kiro/specs/model-simplification/prototype-gate.md), requirements 1-11.
- Hypothesis: H-MS1; predecessor [R114](01-model-simplification-measurement-attribution.md).
- Type: preregistered protocol proposal; execution manifest still pending.

## Decision

The provisional design and a bounded two-case protocol are ready for preparation.
Production tasks remain gated. No structural baseline, behavioral result, model
candidate, useful simplification or adoption was established during this work.

Use an admitted structural companion to the existing mutation pack, and private Git
histories for proposal checks. Require explicit alias/callback/failure observations
for the exporter case. These choices resolve integration problems found in source;
they remain hypotheses about executable feasibility.

## Hypothesis and falsification

H-MS1 proposes that the existing seam and step checker can support at least one
unedited model candidate with a useful branch or repeated-write reduction over a
complete responsibility. Both exporter state and tools branching cases are attempted.

For the `self.tweets` relation, the proposed target removes at least one write and
25% of its write sites, plus at least one repeated site and 25% of its repeated-site
measure. For the complete validation family, remove at least two branches/20% and
one predicate/10%. Companion measures and complete-scope accounting must not worsen.
These are predeclared targets, not measured estimates or claims about the whole
module's percentage improvement. The [gate](../../../.kiro/specs/model-simplification/prototype-gate.md)
defines exact boundaries, controls, budgets and decisions.

Falsifiers include undetected changed state/outputs, a score improved by relocation
or expression folding, unresolvable relevant effects, or no useful candidate inside
the fixed budget. A failure is retained; neither targets nor observation contracts
may be weakened to obtain proceed.

## Frozen inputs and method

Source inspected: Lesscode `7260fa86f7c9d09fa771f71d1a5fdb0bd49ce82d` and MIT specimen
`x_likes_scraper` at `3a214ff13ee768535481f71258ac2b3529306870` under
`D:\BuildArtifacts\lesscode\steps-foreign\x_likes_scraper`.
Exact inspected source/test/lock hashes are retained in the gate. The source clone
was not edited. Authoring drafts and mechanical audit are under
`D:\BuildArtifacts\lesscode\spec-review\model-simplification-20261117\`.

This is an integration-focused design investigation, using existing pinned code and
no new external library selection. P0 must still produce the complete caller/test
inventory, executable observations, profile, runtime overlay/container digest and
reviewable execution manifest. No candidate call is permitted before that freeze
and successful instrument/protection controls. Exact baseline quantities are derived
under the fixed rules before proposals, without adjusting the thresholds.

## Evidence and results

| Source finding | Design consequence |
| --- | --- |
| `native/src/step/context.rs` requires committed judged inputs and a declaration tied to revisions | Construct private synthetic histories; preserve original bytes/refs and record mappings |
| `native/src/adapters/driver/pack.rs` is strict; existing state-write rules feed fault/frame behavior | Add an admitted measurement companion rather than silently changing the legacy mutation population |
| `model/src/repair.rs` checks each answer but its callback cannot report terminal judge failure | Propose one additive fallible-check entry point around the existing loop; keep its infallible API and transport behavior |
| `repair::Attempt` lacks raw response contents | Capture requests/raw answers at the call boundary, linked to actual candidate/verdict bytes |
| Exporter restores `self.tweets` before fetch callbacks and later mutates a resumed list in place | Preserve callback observations, aliases, ordering and failure-state behavior, not just final tweet values |
| Filter helpers encode type, bounds, regex and dependency errors in an order | Qualify full error precedence and public caller behavior; a shorter expression alone is not a useful result |
| Specimen dev manifest/lock lacks pytest-xdist | Freeze a separate runner overlay for containerized specimen tests; preserve upstream locks |

The manual source census found four direct `self.tweets` assignment/mutation sites
and seven `if` sites in the filter validator family. These observations are not an
admitted measurement baseline. No runtime behavior was observed here.

Authoring validation maps all 83 criteria to concrete components/contracts. Review
is inline, not independent empirical review; see the [review record](../../../.kiro/specs/model-simplification/design-review.md).

## Limits and interpretation

The proposed state identities describe bounded lexical field families, not a general
heap model. Unresolved effects remain explicit. Existing tests and planted faults do
not prove equivalence, and a passing synthetic fixture does not qualify a real case.
The exporter may have no useful change that preserves its alias/failure behavior.
The validation family may already need all its decisions. Those are valid outcomes.

Historical native-core U1-U7 supplies evidence disciplines, but its E3/E4 replay APIs
and line-compression U5 apply to that experiment. This feature's approved branch/state
objective supplies the new U5 targets. Keep the 1,400-line/two-apparatus-version effort
ceiling and independent evidence-review prerequisite; no old success is inherited.
The owner requires foreground work, so this authoring review dispatched no agents.

## Tool/spec consequence and next gate

Requirements remain approved. Design is generated but provisional and unapproved;
prototype status is not-run; tasks are ungenerated. Next prepare the small disposable
spike and complete frozen execution manifest, then perform the bounded research run
under its required authorization. Production tasks follow qualified evidence and
normal design approval, restricted to the demonstrated subset.

No product code, specimen code, dependency lock, mutation profile, guard or ledger
was changed. No tests, builds, model calls or prototype runs were performed; the
specification audit is document validation only.

## COBESY check

The decision is visible first; inspected facts and proposed outcomes are separate.
Targets, possible failure and stop conditions are explicit. Negative outcomes do
not threaten the workflow, and setup or passing replies cannot stand in for useful
simplification. Evidence and the [research index](../README.md) link back to the spec.
