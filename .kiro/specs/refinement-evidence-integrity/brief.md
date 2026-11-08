# Correct refinement decisions and recovery

The Lesscode maintainer needs to trust that a compression candidate reported as
valid is the exact candidate checked, that failures remain visible, and that
counterexamples reach subsequent attempts. The current Rust controller and its
Python reference violate those conditions despite their existing suites passing.
The owner requested diagnosis, a corrective spec and an adjusted concept on
2026-11-08. [R39](../../../docs/research/2026-11-08/06-refinement-correctness.md)
contains fresh reproductions; [concept.md](concept.md) describes the proposed
verification and recovery approach.

## Boundary Candidates

- Correct the existing refinement controller's candidate selection, feedback,
  obligation tracking and reported outcomes, and its runtime-oracle adapter.
- Make the tests challenge the evaluator's own failure modes and the correspondence
  between a declared result and delivered code.
- Requalify any affected compression claim with explicit observation coverage.
- Preserve the original implementation as a usable fallback whenever a candidate
  is rejected, evidence is incomplete, or the bounded search ends.

No new solver, graph platform, DSPy integration, language support, automatic
adoption or global rewrite engine is part of this correction. A new framework is
not needed to reproduce or fix these defects.

## Upstream / Downstream

The upstream contracts are constraint-guided-compression requirements 4.2-4.6,
5.5-5.8, 6.3-6.7 and 7.11. The earlier E3/E4 prototypes supply adversarial
examples, not proof that the current loop is correct. Python parity is secondary
evidence because both implementations share defects. The R38 7,004-line cap and
ordinary phase approvals remain in force. Downstream adoption and efficacy claims
depend on correction and proportionate revalidation; adoption planning is still
unscheduled.
