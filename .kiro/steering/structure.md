# Project Structure

The repository contains Kiro authoring workflows, active specifications and the Python baseline package. Foundation, evidence, snapshot, execution, measurement, comparison, reporting, integrated CLI, adverse-path hardening and actual native cross-platform validation are implemented and task-reviewed (32 of 33 tasks). Task 7.4 readiness review and final feature validation remain pending. The baseline spec's `readiness.md` and `requirement-coverage.md` record evidence and limits; `docs/evidence/` retains compact durable audit summaries.

- `.kiro/steering/`: project policy and roadmap.
- `.kiro/specs/<feature>/brief.md`: proposed feature boundary.
- `.kiro/specs/efficiency-baseline/evaluation-protocol.md`: measurement proposal.

Approved baseline implementation locations are detailed in its design File Structure Plan:

- `skills/deconvolute/`: planned optional product skill, distinct from Kiro authoring skills; not yet implemented.
- `src/lesscode_baseline/`: local CLI, typed contracts, snapshots, bounded execution, measurements, decisions, reporting and experiment aggregation.
- `tests/fixtures/`: synthetic and adversarial behavior fixtures.
- `benchmarks/`: licensed corpus manifests and fixed evaluator definitions.

Keep generated run artifacts separate from measured product source. The measurement feature owns versioned result schemas and scope definitions; downstream features consume them. Record source revision and license for external fixtures.
