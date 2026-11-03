# D01-02: solver feedback caught defects but did not guarantee compression

Discussion date: 2026-11-01. Written retrospectively on 2026-11-02; no new run.
Status: mixed experimental evidence. Hypotheses H1-H3 in the
[log](../README.md). Follow-up spec:
[constraint-guided compression](../../../.kiro/specs/constraint-guided-compression/brief.md).

## Decision and evidence

The owner proposed parameterized kernels and CEGIS as a mechanism for compressing
divergent functions. Keep the mechanism, but restrict every proof claim to its
encoded semantics and account for the full emitted replacement.

- [Synthetic CEGIS](../../experiments/cegis/README.md): verbose 66-line family
  becomes 35 shared or 22 inline lines; compact control expands. This establishes
  a mechanism on restricted fixtures, not real-project generality.
- [Self-application](../../experiments/self-apply/README.md) and
  [leaf records](../../experiments/leaf-contracts/README.md): narrow applicability
  and marginal/unsupported replacements do not justify a runtime migration.
- [Reader family](../../experiments/reader-family/README.md): 63 to 60 lines,
  including helper; insufficient benefit for the new indirection.
- [Outcome hotspot](../../experiments/outcome-hotspot/README.md): 72 to 60 lines,
  62 to 41 Radon LLOC; SMT covered only the extracted final-decision boundary.
- [LLM follow-up](../../experiments/outcome-llm/README.md): a proposal changed six
  outcomes; revision restored checks but remained at 60 lines / 41 LLOC. CCN
  improved to 43, without meeting the proposed cap of 36.

## Hypothesis revision

The owner rejected tiny savings as the desired outcome and asked for SMT-augmented
LLM iteration. Counterexamples helped reject unsafe code, but the corrected
candidate did not establish additional line reduction. Do not relabel this as a
successful general compressor or full-function proof.

Next question: does repository-wide framework knowledge reveal replacements
that local arithmetic and representation changes miss?

[Next: framework and graph trials](03-framework-and-graph-trials.md) | [Day index](README.md)
