# R114: Existing checks can support the workflow; simplicity and its attribution still need qualification

- Report date/time and timezone: 2026-11-17, 11:41 Europe/Berlin (UTC+1), from the clock.
- Execution interval: experiments not run; static source inspection and spec initialization only.
- Status: completed (analysis); proposed measures and prototype remain unvalidated.
- Spec: [model-simplification](../../../.kiro/specs/model-simplification/requirements.md),
  [analysis](../../../.kiro/specs/model-simplification/research.md); no approved requirements/tasks.
- Hypothesis ID and predecessor: MS-H1 (below); [R94](../2026-11-12/01-first-python-step.md),
  [R95](../2026-11-12/02-second-python-step-frame-release.md),
  [R99](../2026-11-12/06-repair-gate-result.md),
  [R110](../2026-11-14/03-model-seam-indicator-on-real-models.md),
  [R113](../2026-11-16/02-model-seam-release-and-final-validation.md).
- Type: exploratory static analysis and measurement proposal, not a preregistered experiment.

## Decision

Initialize the spec and proceed to requirements review. Reuse pack/tree analysis,
step checks, model transport and bounded repair. Do not equate existing size gates
or model-reply acceptance with useful simplification.

The owner selected separate reporting of simplicity, preserving the existing
behavior verdict. The analysis proposes branch-condition and predicate counts,
a bounded relation between identifiable state targets and their writers, and
explicit unresolved effects. None is claimed to have demonstrated efficacy here.

## Hypothesis and falsification

MS-H1: within a frozen responsibility and supported measurement boundary, a model
can produce a rewrite that passes the declared behavioral checks and reduces a
predeclared useful branch/state measure without exporting the work or hiding it
in unsupported constructs.

The bounded hypothesis fails if no candidate meets those conditions within the
budget to be declared. Measurement qualification fails if unchanged code reports
different counts, if relocation alone earns improvement, or if unresolved effects
are counted as absent state. Failed behavioral adverse controls leave protection
inconclusive. A count corrected on unchanged code is an instrument correction,
not a successful rewrite.

Useful absolute/relative thresholds, trial counts, call limits and execution budgets
are not yet frozen. No model was called and no hypothesis was tested. Requirements
and a provisional design precede the bounded prototype gate.

## Frozen inputs and method

- Source inspected: lesscode `e877da216ece0b8eea766d48befc417965865408`; tracked tree
  clean at the start. Unrelated untracked files were preserved.
- Main paths: `native/src/adapters/driver/{analysis,sites,measure,pack}.rs`,
  `native/packs/python.json`, `native/src/step/{plan,functions,check,check_runs,verdict}.rs`,
  `native/src/{state_contract,state_frame,seam,cli_deconvolute}.rs`,
  and `model/src/{repair,measure,taskset}.rs`. Targeted inspection also located
  regenerator prompt/candidate/receipt helpers and current plan-test expectations.
- Method: exact source slices and `rg` symbol/call-site searches; compare implemented
  contracts with the discovery brief and prior trials. Summary reads were followed
  by exact slices for substantive claims.
- Future specimens: `x_likes_exporter/exporter.py` and `x_likes_mcp/tools.py` in
  the existing `x_likes_scraper` clone identified during discovery at `3a214ff`.
  No specimen was edited or executed. Frozen whole-responsibility manifests,
  runtime/lock identities and quantitative baselines remain prototype work.

## Evidence and results

The [analysis](../../../.kiro/specs/model-simplification/research.md) contains the
asset-to-gap table, metric candidates, attribution comparisons, adverse controls
and contract tensions, with source links.

Verified from source:

1. `step plan` ranks by unnoticed faults; its rows do not contain branch counts.
2. The Python pack includes local assignments and standalone calls as write sites.
   Analysis labels sites by file/line; it does not resolve state targets and aliases.
3. Step verdicts contain test/fault information and token/node sizes, not a simplicity result.
4. Density qualification uses token/node reduction; `DeconvolutionOutcome` uses
   statement/line targets. Those are different from the owner's desired measure.
5. Model measurement counts passes of a named check. Its built-in tasks use parsing
   and static checks, not an end-to-end real-project simplicity judgement.
6. Existing repair stops on check acceptance. Requesting further simplification
   after behavior passes needs an explicit search policy, not a false refusal.

Proposed attribution separates evaluator change, pins, candidate change, manual
assistance, repair recovery, ranking effectiveness and actual adoption. In particular,
compare M1(S1) with M1(S0), not M1(S1) with M0(S0). Repeated retries without feedback
are needed only if claiming a causal benefit from feedback rather than reporting
observed recovery. No new runtime results or measured improvements are reported.

## Limits and interpretation

This is a source-level gap analysis before approved requirements. State-target
resolution, predicate coverage, movement across files and arbitrary side effects
remain uncertain. A supplied before/after state-map checker does not imply automatic
observation of the full program. Historical semantic requirements cannot replace
fresh evidence on the actual step path.

The two owner-selected cases can demonstrate bounded usefulness. They cannot show
that a new ranking selects better candidates, or establish general model efficacy.
No builds, tests, model trials, benchmark runs or upstream adoption occurred here.

## Tool/spec consequence and next gate

Requirements should define the report's independent outcomes, provenance and
comparability rules. Provisional design should resolve pack/mutation compatibility,
supported state forms, function/target identity and report schema integration.
Then freeze the prototype's inputs, targets, budgets and adverse controls before
execution. Production task generation waits for empirical evidence and phase approval.

## COBESY check

The owner/reviewer should be able to distinguish “the check passed” from “the code
got simpler” using the report and retained evidence. The suspected assumption to
challenge is that fewer reported writes or an accepted model reply proves improvement.
Disproof is explicit; unchanged and inconclusive outcomes are valid findings.
This analysis supports the owner's next requirements decision, not a rollout claim.

## Amendment, 2026-11-17: requirements generated

At the owner's request, [requirements](../../../.kiro/specs/model-simplification/requirements.md)
were generated from this analysis: 11 areas, 83 acceptance criteria. The
[inline requirements review](../../../.kiro/specs/model-simplification/requirements-review.md)
passed after two bounded repair passes. The spec audit checks all 83 criteria with
0 errors and 0 warnings; terminology and contradiction automation is unavailable
without the repository vocabulary file. Requirements await owner approval.
This is a document update, not an experimental result or a change to MS-H1.
