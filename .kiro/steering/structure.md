# Project Structure

The repository contains Kiro authoring workflows, specifications and the Python baseline package. Foundation, evidence, snapshots, bounded execution, measurement, comparison, reporting, integrated CLI, adverse paths and native cross-platform validation are implemented and independently reviewed (33 of 33 tasks). The final feature gate returned GO. The baseline spec's `validation.md`, `readiness.md` and `requirement-coverage.md` record evidence and limits; `docs/evidence/` retains compact durable audit summaries.

- `.kiro/steering/`: project policy and roadmap.
- `.kiro/specs/<feature>/brief.md`: proposed feature boundary.
- `.kiro/specs/efficiency-baseline/evaluation-protocol.md`: measurement proposal.

Approved baseline implementation locations are detailed in its design File Structure Plan:

- `skills/deconvolute/`: planned optional product skill, distinct from Kiro authoring skills; not yet implemented.
- `src/lesscode_baseline/`: local CLI, typed contracts, snapshots, bounded execution, measurements, decisions, reporting and experiment aggregation.
- `tests/fixtures/`: synthetic and adversarial behavior fixtures.
- `benchmarks/`: licensed corpus manifests and fixed evaluator definitions.

Keep generated run artifacts separate from measured product source. The measurement feature owns versioned result schemas and scope definitions; downstream features consume them. Record source revision and license for external fixtures.
