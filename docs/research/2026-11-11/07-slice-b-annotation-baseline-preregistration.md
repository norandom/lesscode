# R82: Preregistration: drift of the annotation layer between runs and between models

- Report date/time and timezone: 2026-11-11, Europe/Berlin (UTC+1)
- Execution interval: not yet run; written before any call of this baseline
- Status: preregistered
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md), Slice B
- Hypothesis ID and predecessor: the second artefact named in the roadmap's Slice B inheritance (the optional
  annotation layer); baseline to compare with: two runs per scope in
  [R64](../2026-11-10/10-annotation-driver-gate-attempt2.md)
- Type: preregistration of a confirmatory baseline (two models, five runs per scope)

## Decision

Owner decision, 2026-11-11: measure the annotation layer next. In it a model reads a source file and names
the sites worth mutating; the driver validates each annotation and mutates only the accepted ones. Pack
authoring is done once per language; annotation would be done per file, so its drift matters more often.

## Hypothesis and falsification

Five fresh annotation runs per scope on each model, with the frozen guide. Every annotation is checked by
the driver's own validator. Agreement between two runs is the number of sites both name, divided by the
number either names (accepted annotations only).

| Claim | Statement | Refuted when |
| --- | --- | --- |
| A1 | On `claude-opus-5-5`, at least 80% of a run's annotations are accepted, in every run of every scope | any run below 80% |
| A2 | On `claude-opus-5-5`, the median agreement between runs of one scope is at least 0.50 in every scope | any scope below 0.50 |
| A3 | Agreement between an Opus run and a Mistral run of the same scope is lower than agreement among Opus runs, in at least four of five scopes (medians) | fewer than four scopes |
| A4 | On `mistral-large-4`, at least 80% of annotations are accepted in at least three of five scopes (median over runs) | fewer than three scopes |

Where the thresholds come from: the two Opus runs per scope of the gate's second attempt, re-checked today
with the native validator and this baseline's tooling, give validity 0.81 to 1.00 and agreement 0.55 to 0.94
(R64 reported 0.60 to 0.94 with the prototype's measure). A1 and A2 say that five runs will not look worse
than those two did.

Descriptive: the sites named in every run of a scope and in any run, against the eligible sites; reasons for
rejection; replies cut off at the output limit; refusals; cost.

What the result will not show: whether the selected sites are the important ones. That was the subject of
the gate's arm A (R61, R64) and is not re-measured. No project's tests are run.

By construction, not as a claim: an annotation the validator rejects never becomes a mutant. Drift in this
layer therefore changes which faults are tried, not whether a run is sound.

## Frozen inputs and method

- Guide: `docs/experiments/annotation-driver/guide-v2.md` (SHA-256 `d3211320...`), as in the gate's second
  attempt. One call per scope and run, no repair turn.
- Scopes: the five gate scopes at the gate revisions, from the isolated clones.
- Caller: `annotate.py`, unchanged, driven by `slice-b/annotate_baseline.py`. Validator:
  `native/examples/annotation_sites.rs` built from commit `eea0109`, with the shipped packs. Call-argument
  sites, added today, are not among the four kinds the guide knows and are left out of the eligible count.
- Output limits: 16,000 tokens for `claude-opus-5-5`; 32,000 for `mistral-large-4`, which counts its
  reasoning against the limit (R72).
- Budget: expected about 4 USD on Opus and 1.5 USD on Mistral. Recorded before: about 25.5 of 40 USD.
- A refusal is an outcome and is not reworded. A run without a usable reply counts as a run without
  annotations and is reported.

## Evidence and results

None yet. Results go into their own entry.

## COBESY check

Outcome first: nothing measured; four claims with thresholds. Replay: guide, scopes, caller, validator and
limits named. Disproof: each claim can fail. The thresholds' origin is stated, including that they were set
after re-checking earlier runs with the new tooling.
