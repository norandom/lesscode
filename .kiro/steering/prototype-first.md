# Prototype-first design gate

## Native-core solution unlock (2026-11-07)
Apply [U1-U7](../specs/deconvolution-native-core/prototype-gate.md) and
spec.json.solution_unlock. Matching evidence and independent review, not status
alone, control production eligibility. No -y waiver. Provisional design and
authorized disposable experiments use existing tools/explicit inventories;
do not require the future checker or runner to discover feasibility.
Unlock never replaces ordinary phase approval.

Owner direction, 2026-11-02: research-bearing features must fail early through
bounded prototypes before a production implementation task plan is approved.
A plausible architecture or successful parser/database setup is insufficient.
This rule applies to new compression mechanisms and material changes to their
analysis/modeling assumptions, not routine fixes or documentation changes.

## Kiro sequence

1. Establish requirements and a provisional design proposal. Identify its riskiest
   assumptions, candidate real-world scope and expected net compression mechanism.
2. Write `prototype-gate.md` in the spec: frozen inputs, observable behaviors,
   absolute and relative useful-reduction targets, evidence requirements, budget,
   adverse controls and stop conditions. Explain why the expected savings are
   plausible by naming the implementation that could disappear and the support
   that must replace it. An estimate is not a measured result.
3. Run the smallest isolated research spike needed to challenge those assumptions.
   Prototype scripts, fixtures and a bounded experiment checklist may precede
   production tasks; this gate must not require a completed product to test it.
   Use the authorized research scope and preserve upstream specimen checkouts.
4. Record actual behavior, compression and failure evidence in the dated lab log.
   Assess the gate as `proceed`, `revise`, `stop` or `not-run`. A successful
   extraction or synthetic fixture alone cannot qualify as `proceed` for a
   real-code compression mechanism.
5. Only after `proceed`, revise the design around the demonstrated subset and
   seek normal design approval before generating production implementation tasks.
   Retain unsupported cases as exclusions and bounded follow-up experiments.

## Required evidence for proceed

- At least one representative real-code responsibility is exercised end to end
  by the proposed mechanism, including relevant cross-function/state behavior.
- A concrete candidate meets its predeclared scope-level usefulness target after
  accounting for helpers, tables, wrappers and changed callers.
- Frozen behavioral checks and adverse controls support the declared boundary;
  source-model gaps and any restricted proof claims are explicit.
- The record distinguishes manual interventions from capabilities exercised by
  the prototype. A human-authored shortcut cannot prove an unexercised automatic
  modeling loop. Earlier trials cannot be inherited as proof of a new mechanism.
- Remaining implementation work is bounded and supported by these findings.

These are feasibility conditions, not proof of general efficacy. Larger/held-out
evaluation still follows. If a mandatory behavioral boundary cannot be checked,
the gate remains inconclusive and its decision is `revise` or `stop`.

## Enforce before task execution

`kiro-spec-design` may produce a provisional design and prototype plan before
the gate passes, but must label it as such and leave design approval pending.
`kiro-spec-tasks` must check the applicable gate before creating production tasks.
`kiro-impl` must check it before executing dependent production tasks, even if a
stale task list exists. The `-y` flag can fast-track a human phase approval; it
cannot create missing empirical evidence or convert a failed gate to proceed.

On `revise` or `stop`, retain the results and change the hypothesis or scope
before another bounded attempt. Do not expand infrastructure, silently lower
targets or continue downstream tasks to make a failed experiment look complete.
Material source, model, dependency or semantic-boundary changes invalidate the
affected gate claims and require proportionate revalidation.

Every gate links to its dated evidence and the research index. Use COBESY's
conclusion-first, falsifiable framing. Record negative outcomes as valid research
progress. Do not mark a production feature complete merely because its hypothesis
was rejected.

## Cross-spec contradiction gate — 2026-11-09

Before design approval, surface conflicting scope, semantic claims, runtime
dependencies, contracts, budgets and approval states in a "Contradictions and
disposition" table. Record source authority, resolution, evidence and affected
consumers. Open contract contradictions block approval; empirical unknowns remain
explicit prototype blockers. A research checklist may precede production tasks,
but must not be mislabeled an executable production plan or a completed gate.
