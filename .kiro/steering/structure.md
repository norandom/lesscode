# Project Structure

## Research records

Owner instruction, 2026-11-02: organize narrative lab reports by date under
`docs/research/YYYY-MM-DD/NN-topic.md`, indexed in `docs/research/README.md`.
Use the entry template and COBESY's conclusion-first, falsifiable framing.
Record hypothesis revisions, failed attempts, actual execution provenance and
evidence limits. Cross-link each report with its Kiro spec; add the index entry
in the same change. Preserve existing experiment artifacts at their stable paths.
Backfilled records must say retrospective and must not invent execution dates.
Date later amendments instead of overwriting earlier interpretations. Expand
the tool in bounded, evidence-led stages; a successful setup or accepted backend
does not establish modeling fidelity, compression or approval of all spec phases.

The repository contains Kiro authoring workflows, specifications and the Python baseline package. Foundation, evidence, snapshots, bounded execution, measurement, comparison, reporting, integrated CLI, adverse paths and native cross-platform validation are implemented and independently reviewed (33 of 33 tasks). The final feature gate returned GO. The baseline spec's `validation.md`, `readiness.md` and `requirement-coverage.md` record evidence and limits; `docs/evidence/` retains compact durable audit summaries.

- `.kiro/steering/`: project policy and roadmap.
- `.kiro/specs/<feature>/brief.md`: proposed feature boundary.
- `.kiro/specs/efficiency-baseline/evaluation-protocol.md`: measurement proposal.

Approved baseline implementation locations are detailed in its design File Structure Plan:

- `skills/deconvolute/`: planned optional product skill, distinct from Kiro authoring skills; not yet implemented.
- `src/lesscode_baseline/`: local CLI, typed contracts, snapshots, bounded execution, measurements, decisions, reporting and experiment aggregation.
- `src/lesscode_baseline/rules/`: historical policy implementation. The task list marks 9/25 tasks complete; remaining work is retired from active delivery. Preserve uncommitted work and do not treat unchecked tasks as reviewed completion.
- `src/lesscode_baseline/AGENTS.md`: package-scoped guidance for idiomatic code, role-revealing names, justified abstractions and preserved evidence contracts; instruction guidance rather than an automatic rule engine.
- `tests/fixtures/`: synthetic and adversarial behavior fixtures.
- `benchmarks/`: licensed corpus manifests and fixed evaluator definitions.

Keep generated run artifacts separate from measured product source. The measurement feature owns versioned result schemas and scope definitions; downstream features consume them. Record source revision and license for external fixtures.
The implemented `.kiro/specs/capability-compression-core/` uses one native
library/CLI package under `native/`, with Scope, Metrics and Compare modules and
external CLI/integration tests. `.agents/skills/lesscode-discovery/` contains the
verified recipe. Native tests use cargo-nextest; Python reference tests use
pytest-xdist. `docs/experiments/native-replay/` and `native-self-apply/` retain
fresh empirical evidence; the latter includes the adopted policy-helper reuse.
Native Windows/Linux/macOS checks passed in Actions run 41333477507; Linux uses
Dagger, while Windows/macOS remain native host jobs. Exact revisions and artifact
identities are retained in `docs/evidence/native-core-41333477507.json`.
Frozen `docs/experiments/` records remain
reference evidence. Broad policy, transaction, MCP/plugin and solver-integration
layers are removed from active delivery, not queued dependencies.
