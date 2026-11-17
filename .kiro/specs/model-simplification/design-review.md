# Design review: model simplification

2026-11-17. Result: **PASS for provisional design authoring**. This is not a
production readiness verdict, design approval or empirical proceed decision.
Review mode: inline in the foreground; no independent reviewer or subagent ran.

## Inputs and authority

Reviewed approved requirements, brief, source-backed research, core/prototype-first
steering, adjacent step/model/pack contracts, budget rules, actual integration code
and the draft prototype protocol. Applied `kiro-spec-design` and its design-review
gate. The draft lived outside the repository under
`D:\BuildArtifacts\lesscode\spec-review\model-simplification-20261117\`
until this bounded review completed. Documents are English per spec.json.

## Mechanical checks

The local PowerShell audit extracted each criterion number under its requirement
heading and checked the explicit design traceability table: **83/83 present**.
Boundary, dependency, revalidation, file-plan and contradiction sections are populated;
all four components map to concrete files. The gate contains U1-U7, two named cases,
numerical targets, controls, budgets and stop conditions. No draft placeholders remain.

The canonical `tools\bin\kiro-cli.exe check --feature model-simplification` audit
checks requirements syntax; it does not establish architectural correctness or
empirical feasibility. Vocabulary-based term/contradiction checks remain unavailable
because this repository has no `.kiro/vocabulary.toml`. Inline contradiction review
therefore remains explicit rather than being attributed to that tool.

Final canonical audit: exit 0, 83 criteria checked, 0 errors, 0 warnings, 4
informational findings (missing vocabulary, roadmap format, undeclared topics and
intentionally absent tasks). The file-plan header was aligned with the audit's
`Files` column convention so its component/path check recognizes the concrete map.

## Bounded review and repairs

1. Repaired a terminal-checker-error gap: the existing repair callback returns only
   pass/reject. The design now adds a fallible entry point over one shared loop,
   retains the old API and explicitly ends incomplete trials. Corrected the flow
   diagram and file ownership. Separated inline authoring review from the independent
   evidence review still required before proceed.
2. Made the predicate accounting include assigned/table-held logic, defined how
   legacy grouped protection facts join qualified identities without false precision,
   specified a strict candidate-edit format, included module registration in the
   file plan and clarified simplicity outcomes for rejected candidates. Rechecked
   all criterion mappings and boundary/compatibility consequences.

Two repair passes; no unbounded design expansion. No new requirement or phase
approval was invented. The prototype must still show whether the proposed subset is
implementable within its budget and useful on real code.

## Judgment and remaining gates

- Measurement, proposal orchestration, step reporting and evaluation have explicit
  ownership and directional dependencies. The core remains model-free.
- A separate admitted metric companion avoids silently deleting mutation coverage.
  Opt-in plan v2/sidecars preserve legacy verdict, guard and ledger meanings.
- Private Git histories satisfy the existing checker without committing candidates
  into the source repository. Exact snapshots, input admission and raw-answer records
  support attribution.
- The prototype retains complete replacement accounting, numerical targets, known-bad
  state/output controls and fixed budgets. Success on one axis cannot qualify another.
- The source state/alias model is a bounded empirical risk with concrete falsifiers.
  Container/runner identity, observation pins, profile implementation and the complete
  execution freeze are still missing and explicitly required before candidate calls.
- The contradictions table addresses historical line targets, current host/runtime
  rules, ranking compatibility, mutation coverage and evidence-review authority.

Current result: provisional design may be saved. Requirements approved; design
generated but unapproved; prototype not-run; production tasks ungenerated and
implementation not ready. No product/specimen code, builds, tests or model calls
were part of this authoring review.
