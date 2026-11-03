# R01: useful examples did not yet establish a general compression mechanism

Amendment, 2026-11-02: [separate entries for the 2026-11-01 discussion](../2026-11-01/README.md)
now explain each experiment/decision turn. They index the same retained evidence;
this catalog remains a cross-trial overview, not an additional run.

Report date: 2026-11-02, Europe/Berlin. Retrospective catalog, not a new trial.
The referenced records are authoritative for execution timestamps and source
versions. Several explicitly date their work to 2026-11-01; no precise execution
order is inferred for records lacking timestamps. The sequence below follows
hypothesis development rather than claiming a clock-ordered run ledger.

## Decision

Retain positive and negative evidence. Capability-scoped replacement can remove
substantial code, but neither local rewrites nor graphs alone demonstrated a
reusable driver. The follow-up is an LLM constraint modeler using queryable facts,
with counterexample-driven revision and explicit source-model fidelity checks.

## Evidence catalog

| Track | Retained evidence | Outcome and effect on the hypothesis |
| --- | --- | --- |
| Facdrone inspection | [Roadmap and census](../../experiments/facdrone/README.md) | Modularization can improve routine scores while adding code; specimen is not an upstream refactoring deliverable |
| Synthetic CEGIS | [Sources, solver results and charts](../../experiments/cegis/README.md) | Verbose 66-line fixture becomes 35 shared or 22 inline; compact control grows; restricted synthesis, not arbitrary Python proof |
| Lesscode self-applicability | [Self-application](../../experiments/self-apply/README.md) | Narrow applicability and rejected replacements; solver support does not measure compressibility |
| Declarative leaf records | [Leaf trial](../../experiments/leaf-contracts/README.md) | 65 to 61 lines, incomplete compatibility boundary; no runtime migration. Historical byte measurements are not current benefit criteria |
| Record-reader family | [Reader trial](../../experiments/reader-family/README.md) | 63 to 60 lines; indirection/typing costs outweigh a marginal result |
| Outcome hotspot | [Stage 1](../../experiments/outcome-hotspot/README.md) | 72 to 60 lines, 62 to 41 Radon LLOC; SMT only covers the extracted final-decision boundary |
| LLM with SMT feedback | [Stage 2](../../experiments/outcome-llm/README.md) | Unsafe proposal detected; corrected proposal reduces CCN further but remains 60 lines / 41 LLOC |
| Global framework inventory | [Controlled pilot](../../experiments/capability-guided/run-01/README.md) | Both arms find the same 12-line reuse; no demonstrated incremental compression from inventory |
| Connected report graph | [Graph trial](../../experiments/report-graph/README.md) | Joint candidate increases LLOC 69 to 73; rejected despite passing behavioral checks |
| Feature-path representations | [Falsification probes](../../experiments/feature-path/README.md) | Representations carry distinct information; deletion premises rejected, not proof of minimality |
| ATS capability | [ATS01](../../experiments/ats-report/README.md) | 384 to 202 physical lines, 298 to 135 LLOC; 8,820 comparisons; isolated candidate, no upstream adoption |
| NPU capability | [NPU01](../../experiments/npu-backends/README.md) | Positive shared-backend replacement with 3,168 comparisons and 339 tests per version; preserve provider safeguards |
| Native external evaluation | [Readiness report](../../reports/native-core-readiness.md), [replay preparation](../../experiments/native-replay/README.md) | Later readiness evidence supersedes the preparation page's pending wording; evaluator maturity differs from discovery efficacy |
| Native self-application | [Adoption record](../../experiments/native-self-apply/trial/record.md) | Adopted 32-line / 29-statement reduction across the owned scope; includes wrappers and shared implementation |
| Local adoption baselines | [mmhmem/mvibe](../../experiments/local-adoption/README.md) | Baselines and rejected shortcut; no compression claim |
| mvibe proxy routing | [P02](../../experiments/proxy-routing-trial/README.md) | 334 to 311 lines; 1,172 tests each side and differential checks; modest isolated candidate, no SMT |
| Facdrone CSV | [FCSV01](../../experiments/facdrone-csv-trial/README.md) | 1,056 to 1,053 lines; rejected as marginal. 54 rendered artifacts match; baseline Windows publication failures persist; no SMT |

These populations overlap and use different metrics and proof boundaries.
Do not sum reductions, tests or comparisons into a cumulative efficacy score.
The final Facdrone saving is three lines, not an earlier provisional four.

## Hypothesis revisions

The [earlier synthesis report](../../reports/compression-trial-progress.md)
records the transition from restricted kernels to SMT-assisted local proposals.
The [capability report](../../reports/capability-compression.md) records why whole
responsibilities matter. Neither establishes automated discovery or measured
maintenance effort. Later marginal trials exposed the missing mechanism: a
model of cross-function behavior and state that actively drives replacement.

The [new brief](../../../.kiro/specs/constraint-guided-compression/brief.md)
captures that correction. The [next dated record](02-codeql-direction.md)
selects the evidence backend without claiming that the mechanism already works.

## Specs and continuation

- [Efficiency baseline](../../../.kiro/specs/efficiency-baseline/spec.json): historical measurement foundation.
- [Capability compression core](../../../.kiro/specs/capability-compression-core/spec.json): implemented scope-level evaluator; see its readiness evidence for exact claims.
- [Constraint-guided compression](../../../.kiro/specs/constraint-guided-compression/spec.json): initialized follow-up; backend selected, formal phases pending.

COBESY check: outcome first; each result has an evidence link; rejection is
visible; chronology is not invented; the next decision is distinguishable from
past results. [Research log](../README.md).
