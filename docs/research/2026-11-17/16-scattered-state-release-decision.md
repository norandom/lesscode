# R129 · What would count as simplifying scattered state?

Amendment, 2026-11-17: the owner means implicit state machines spread across files. Coordinating a duplicated rule, as considered here, is only one possible symptom. [R130](17-hidden-state-machine-fitness.md) tests the clarified meaning and records the release decision.

Amendment, 2026-11-18: one day later a hidden machine of this repository was consolidated and a finder returned it from source. The count of write sites played no part in either. [R155](../2026-11-18/21-reply-rule-in-the-loop.md) · [R156](../2026-11-18/22-find-step-reply-machine.md) · [R158](../2026-11-18/24-finder-follows-names.md)

Written 2026-11-17, Europe/Berlin. Exploratory source review of Lesscode at `91e176d` and the completed MS1 records. The two model calls belong to [R126](13-model-simplification-boundary-newlines.md) and [R128](15-model-simplification-checkout-preflight.md). This review ran no experiment. [Spec and next task](../../../.kiro/specs/model-simplification/task-generation-readiness.md)

**Decision.** Keep state simplification open. The prototype can locate some writes and flag unknown effects. The outstanding product question is whether it can show a maintainer which state-management decision they can remove. Validate on Lesscode itself once we can distinguish a real removal from delegation on supported code.

### What the setter tells us

The exporter initializes a list, restores it from a checkpoint, extends it in place, or replaces it after a fresh fetch. Those actions serve different lifecycle events. Both proposals routed three assignments through a setter. A change to *when* the list is restored or replaced would still require editing the corresponding callers.

For this one nonempty field, the spike calculates repeated sites as the number of write sites minus one. A change from four sites to two therefore also changes three repeated sites to one. These two numbers count the same change. Writer dispersion stayed at one (two writer functions) and the new helper calls had unresolved effects. Both proposals were classified as unmeasurable. The state gate only requires dispersion to avoid increasing. A lower site count alone cannot tell a developer that fewer places need to agree. [Calculation](../../../docs/experiments/model-simplification/spike/src/relations.rs) · [Frozen targets](../../../.kiro/specs/model-simplification/prototype-gate.md) · [Observed outcome](15-model-simplification-checkout-preflight.md)

The frozen observations explain why treating all four sites as duplicates would be a mistake. On a nonempty resume, the result must be the same list object that the fetch callback saw. After an empty resume the fresh list becomes the result. On a fetch failure the restored list remains visible. These are concrete reasons to preserve distinct transitions. [Original behavior checks](../../experiments/model-simplification/pins/test_state_observations.py)

### A maintenance question in Lesscode itself

In the recording shim, registration and removal each decide whether a SQL listener belongs to the engine instance or its class. Both the record and replay commands also arrange removal after success or an exception. If the target-selection rule changed, registration and removal would have to agree. A plausible replacement would choose the target once and pair registration with cleanup for both callers. It would still need to register and remove the listener, preserve event and callback identity, and clean up on failure. [Recording shim](../../../native/shim/recording_shim.py)

This is a candidate to reason about. The admitted state profile recognizes particular field writes and one contracted list mutation. The listener registry is an external effect. The source supports a hypothesis about duplicated lifecycle decisions, but the current profile cannot judge that example. Adding support for SQLAlchemy solely to score this case would make a poor release test. [Admitted effect contracts](../../experiments/model-simplification/measurement-contracts.md)

### How to decide

For the next comparison, write down the state, one concrete change to its update rule, every owner that must coordinate today, and the observer that would notice a mistake. Look for a real pair within the admitted scope where a duplicate decision or synchronization obligation disappears. Count any obligation moved to a helper or caller. Include a setter-only version as a negative control. A comparison that rewards the setter, hides an effect, or misses the genuine removal leaves the measure open. This interpretation guides the next research decision. The original MS1 targets and dispositions remain historical evidence.

If the distinction holds, run the tool on one maintained Lesscode responsibility and record the before/after diff, behavior evidence, tool contribution and maintainer's keep-or-decline decision. If it cannot make the distinction within its supported scope, defer the state feature. The checkout fix and further filter trials do not resolve this question. [Release roadmap](../../../.kiro/steering/roadmap.md)
