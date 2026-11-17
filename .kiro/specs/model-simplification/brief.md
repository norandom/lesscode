# Brief: model-simplification

Written 2026-11-17, 11:28 Europe/Berlin (clock: 10:28 UTC). Discovery only;
requirements, design, prototype gate and production tasks are not yet approved.
Language: English, following project steering; spec.json is created at initialization.

## Problem

A developer can use lesscode to check a small refactoring, but still has to find and
write the change. The next feature lets a configured model propose a simplification
where branch conditions and state concentrate, then judges the candidate through
the existing step flow. Success means fewer branch conditions or less scattered
state with the declared behavior checks intact, not fewer lines or tokens.

## Current State

- The model seam supplies endpoint handling, bounded repairs and receipts through
  `lesscode-model`. It does not own the task asked of a model or its acceptance checks.
- `native/src/step/plan.rs` ranks functions by unnoticed faults, then noticed faults.
  It lists tokens and state-write sites but does not rank by branch conditions or
  measure how state is distributed.
- `native/src/step/verdict.rs` reports behavior checks and before/after tokens and
  nodes. There is no separate simplicity result.
- `native/packs/python.json` identifies every assignment and augmented assignment,
  plus standalone calls, as state writes. R95 found local assignments counted as
  writes in a function that writes no state. A write-site count alone is not a
  measure of scattered state.
- R94 and R95 exercised manually chosen Python steps, not model-generated
  simplification. Both exposed missing behavioral protection. R94 accepted an
  output change before pins; R95 found a keyword-argument change outside the
  mutation operators' coverage. Passing checks is not an equivalence proof.

## Desired Outcome

For a declared responsibility, the developer can see branch and state candidates,
request a model proposal, inspect the candidate and its checks, and see whether
simplicity improved. The report keeps two answers distinct: what the behavioral
checks established, and what changed in the simplicity measures. An accepted step
does not automatically count as a successful simplification.

Owner decision in this discovery, 2026-11-17: **report simplicity separately;
preserve the existing behavior verdict.** A lack of simplicity improvement does
not introduce a new reason for that verdict to refuse a step.

## Approach

Use one new spec for the bounded proposal-and-check workflow, extending the existing
pack-driven measurements and step reports. Reuse the model seam for calls and
repairs; keep the step flow and driver deterministic and usable without a model.
Keep the original code, tests and declared boundary fixed while evaluating a
candidate in isolation. Carry actual check reasons into a bounded repair attempt.

This follows the owner's handover direction and R99's propose/check/repair shape.
R99 is evidence for pack repair only, not proof that repair simplifies real code.
The CLI entry point and orchestration location remain design decisions; this brief
does not authorize adding network calls to `step` or the driver.

Alternatives considered:

| Approach | Benefit | Limitation | Scope |
| --- | --- | --- | --- |
| Extend the existing checks with bounded model proposals (selected direction) | Reuses the seam, step declarations and verdicts; supports the requested end-to-end outcome | Needs evidence for state measurement and candidate isolation | Medium, subject to prototype |
| Add measurements only; leave proposals to the developer | Small deterministic change | Does not deliver the requested model-driven simplification | Small, possible prototype stage |
| Build an autonomous repository-wide refactoring agent | Could attempt many changes | Adds orchestration and adoption decisions before one useful step is demonstrated | Large, excluded |

## Scope

- **In**: branch/state ranking in `step plan`, retaining protection information;
  correcting Python state classification through pack data and, only if needed,
  generic rule support; model proposals and bounded repair through the seam;
  existing step checks; a separate before/after simplicity report; real-code
  prototype evidence on the owner's `x_likes_scraper`.
- **Out**: onboarding levels and plugin delivery; new model clients or optimizers;
  regenerating packs per project; model rankings; arbitrary tool-using agents;
  automatic upstream adoption, commits or pushes; equivalence proofs; line/token
  reduction as the definition of effectiveness.

## Boundary Candidates

1. **Pack-driven measurement** identifies branch conditions and state facts, with
   explicit limits. Rules remain generic across the admitted language packs.
2. **Proposal orchestration** supplies bounded context and receives candidates
   through the seam. It cannot edit the frozen tests, declaration or judge to pass.
3. **Step integration and reporting** preserves the behavioral verdict and reports
   simplicity independently over the whole declared responsibility, including
   new helpers and changed callers.

These are internal responsibilities of one feature, not three independent specs.

## Out of Boundary

The model seam continues to own endpoint configuration, transport, waits and call
receipts. The existing step flow owns pins, behavioral checks, the ledger and guard.
Onboarding owns its broader user journey and protection policy; this workflow must
still expose unsupported or unprotected behavior in its prototype and reports.
The installer notice collision and intermittent guard test are separate follow-ups.

## Upstream / Downstream

- **Upstream**: `model-seam`, `gradual-refactoring-steps`, and the admitted packs
  and generic driver from `agnostic-annotation-driver`.
- **Downstream**: the onboarding spec, which consumes a demonstrated proposal and
  checking workflow. No onboarding implementation is included here.

## Existing Spec Touchpoints

- **Extends**: gradual-refactoring-steps Requirement 2 (ranking) and the reporting
  portion of Requirement 5; pack classification owned by agnostic-annotation-driver.
  Requirements/design must record these contract changes explicitly rather than
  silently replacing the old ranking or changing fault populations.
- **Adjacent**: model-seam Requirements 6 and 7 (repair and a model-free judge),
  and Requirement 10 (task-set measurement). A real simplification task set may
  consume its existing measurement facility; no new model benchmark is assumed.
- **Adjacent**: state-boundary-deconvolution and the current frame checks. Reuse
  applicable contracts; do not introduce a second semantic authority for behavior.

## Constraints and Prototype Questions

- Keep behavior, public contracts and safeguards. Existing growth limits remain
  safeguards unless explicitly revised; tokens are not a simplicity reward.
- Define branch counting and state distribution before experiments. Local rebinding,
  attribute/subscript mutation, global/nonlocal writes, aliases and unknown calls
  need explicit treatment. Unknown effects must not be presented as known purity.
- Freeze the accounting boundary so moving branches or writes into helpers cannot
  manufacture improvement. Define how incomparable or unsupported cases are reported.
- A pack correction can change mutation sites as well as measurements. Compare both
  revisions under the same frozen pack and retain evidence of coverage changes;
  fewer planted faults alone must not establish better protection or simplicity.
- Freeze useful absolute/relative targets, behavioral checks, adverse controls,
  call/run budgets and stop conditions in a prototype gate before execution. No
  numerical efficacy threshold or state metric is claimed as established here.
- Start with `x_likes_exporter/exporter.py` for state and `x_likes_mcp/tools.py`
  for branching. The existing clone at
  `D:\BuildArtifacts\lesscode\steps-foreign\x_likes_scraper` is at `3a214ff`
  with no tracked working changes at discovery. Preserve it; freeze an explicit
  experimental copy and its dependencies before any run.
- Include the missing-test and mutation-coverage lessons of R94/R95. Keep pins
  separately established before rewrites; identify human interventions and limits.
- Gate results distinguish measurement fidelity, behavioral evidence, actual
  simplification and adoption. Both real responsibilities are planned trials;
  production scope must follow demonstrated results, not assumed generality.
- Work in the foreground. Rust product/prototype tooling, PowerShell on Windows,
  no execution in WSL. Disposable output belongs under `D:\BuildArtifacts`.
  The target Python suite is a separate specimen-runtime concern to resolve in
  the prototype plan against the current execution rules before running it.
- No new dependencies are selected in discovery. Inline inspection found reusable
  pack analysis, function attribution, seam and step-report boundaries; runtime
  viability is not yet demonstrated. No subagents or model calls ran.

## Evidence and Continuity

- [Owner direction and next work](../../handover.md), including the superseding
  release decision; the simplification prototype no longer blocks the 2.0.0 tag.
- [R94: first Python step](../../../docs/research/2026-11-12/01-first-python-step.md).
- [R95: second Python step and local assignments counted as state](../../../docs/research/2026-11-12/02-second-python-step-frame-release.md).
- [R99: bounded repair result](../../../docs/research/2026-11-12/06-repair-gate-result.md).
- [R113: seam validation](../../../docs/research/2026-11-16/02-model-seam-release-and-final-validation.md).
- [Prototype-first policy](../../steering/prototype-first.md).

This is a discovery brief, not a new experimental result. The discovery sequence
was initialization, requirements review, then provisional design and a bounded
real-code prototype gate. Production task generation waits for the gate and phase approvals.

## Initialization update, 2026-11-17

The owner requested initialization with an analysis of existing capabilities and
attributable measurable improvements. [spec.json](spec.json) now records phase
`initialized`; [requirements.md](requirements.md) contains the project description,
with acceptance criteria still pending. [research.md](research.md) compares current
assets with desired measures and proposes attribution controls. The dated record is
[R114](../../../docs/research/2026-11-17/01-model-simplification-measurement-attribution.md).
No experiment ran and no later phase was approved. Next: generate requirements for review.
