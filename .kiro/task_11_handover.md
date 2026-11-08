# Task group 11 handover: correctness diagnosis and adjusted concept

Written 2026-11-08. Group scope: the owner asked to identify defects using the
earlier prototypes, produce a corrective spec, explain what blocks compression,
and produce an adjusted concept with a practical verification/recovery approach.
All investigation and review ran in the foreground. No implementation subagents,
production fixes, model calls or adoption work were dispatched.

## Result

The correctness doubt is supported, not dismissed. Fresh probes against source
`a755712` reproduce six root-cause groups: oracle failure/timeout treated as
success; lost runtime feedback; best record paired with failing final files;
still-failing counterexample marked discharged; configurable threshold used as
validity; missing measurements reported as compression met. Five controller
defects also reproduce in the Python reference.

The earlier E3/R25 and E4/R26 records show independent limitations in observation
coverage (exception-side state, partial output, binding times). Their evidence
was read, not re-executed. The current replay/parity tests do not close those gaps.

## Deliverables

- [R39](../docs/research/2026-11-08/06-refinement-correctness.md): diagnosis,
  source anchors, exact limits and evidence links.
- [Corrective spec](specs/refinement-evidence-integrity/requirements.md): six
  requirements, 34 acceptance criteria; requirements-generated, approvals false.
- [Adjusted concept](specs/refinement-evidence-integrity/concept.md): independent
  verifier tests, state-transition properties, artifact identity, causal feedback,
  explicit observation coverage, versioned counterexamples and bounded recovery.
- [Verification proposal](specs/refinement-evidence-integrity/verification.md):
  concrete controls and staged qualification; not a production task list.
- [Probe runner](../docs/experiments/deconvolution/refinement-correctness/run.ps1)
  and retained raw logs/provenance in its `results/` directory.
- Research index and dated parent requirements/design/tasks/prototype-gate notices
  cross-link the correction and invalidate affected broad interpretations while
  preserving historical artifacts and approvals.

## Fresh evidence

- Existing regenerator suites: native 45/45, Python 59/59, both exit 0.
- New expected-behavior assertions: native 9 cases, one positive control pass
  and eight failures (nextest exit 100); Python five failures (pytest exit 1).
  These are red baseline evidence, not green feature validation.
- Recorded run: 2026-11-08T16:13:16.8608761Z to 16:13:24.7288819Z.
- Actual runtime: rustc 1.98.1, nextest 0.9.150, Python 3.13.16, pytest 9.0.3,
  xdist 3.8.0. The documented runner/test lock differs; the discrepancy is
  explicit and must be reconciled for later formal qualification.
- Source/probe/WIP hashes, raw logs and isolated resolved Cargo lock retained.
  Probe dependency packages match the native lock; only the probe root is added.
- Production source unchanged. R36/R37 frozen receipt hashes still match their
  sha256sum-format sidecars. New-doc links and six/34 requirement structure checked.
- No full repository suite, live efficacy run, mutation campaign, fix validation
  or cross-platform Dagger run is claimed. Prior abandoned background pytest
  output was not recovered and is not used.

## Current state and next steps

The corrective requirements and concept are reviewable; they are not implicitly
approved. Technical design and production tasks have not been generated. Start
the next group from this handover and the latest mmhmem checkpoint. After owner
review, create the smallest correction design, retain these regressions, and
qualify causal feedback plus exact artifact delivery before another compression
efficacy claim. The current user request does not authorize Slice B adoption.

One real responsibility with a concrete removal hypothesis, frozen observation
matrix, usefulness target and budget is the proposed later efficacy gate. A
repaired evaluator establishes truthful decisions, not useful compression by
itself. R38's no-growth cap 7,004 and U7 remain unchanged.

Owner WIP remains untouched and outside this commit: two round21 divergence JSONs,
rules/evaluate.py and test_rule_evaluation.py. Numerous untracked files existed
before this group; do not blanket-add or delete them. Use native PowerShell at
`D:/Source/lesscode`; the C: path is a junction. Keep the known clean-PATH protocol.
Do not run rust_u5_gate.py casually: it overwrites a frozen receipt.

Per owner rule, ask the user to clear context before the next task group.
