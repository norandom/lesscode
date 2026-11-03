# R05: requirements approved; feasibility remains unestablished

Date: 2026-11-02, Europe/Berlin. Status: design discovery and setup probe, not a
compression trial. Predecessor: [R04](04-prototype-first.md). Spec:
[provisional design](../../../.kiro/specs/constraint-guided-compression/design-proposal.md).

## Decision

The owner approved requirements and the prototype-first workflow. The proposal
uses CodeQL exports, recorded model revisions, bounded checks and native comparison.
No new graph store or orchestration framework is required for the first spike.
Final design/task approval still depends on real-code evidence.

## Actual evidence

Fresh local commands verified CodeQL 2.26.3 and Python/JavaScript extractors.
Only legacy-upgrades appears in the resolved query-pack list. Retained receipts:

- [Timestamp, commands, executable hash and exit codes](evidence/codeql-design-setup/receipt.json)
- [Version](evidence/codeql-design-setup/version.json)
- [Extractors](evidence/codeql-design-setup/languages.json)
- [Query packs](evidence/codeql-design-setup/qlpacks.json)

No extraction ran. A read-only explorer checked Lesscode's analyzer orchestration,
report persistence, evaluation and snapshots. No defensible untried target met
its screening values of 25% and 30 net production statements. Analyzer orchestration
remains a lead, with distinct error/artifact/state semantics to understand. This
is a bounded negative discovery result, not evidence of minimality or failure
of the complete modeling loop.

## Next gate

The proposal maps 56 requirements, but that is not feasibility evidence. Run
bounded query-pack/extraction probes and qualify a source-grounded real target
before candidate generation. The [gate](../../../.kiro/specs/constraint-guided-compression/prototype-gate.md)
remains not-run; no production tasks are created. Keep revise/stop available.

COBESY check: actual observations, proposed mechanism and unestablished savings
are separate. An optimistic implementation schedule does not replace a result.
