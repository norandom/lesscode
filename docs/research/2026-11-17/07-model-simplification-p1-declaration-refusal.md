# R120: P1 baseline refused because the other declaration changed outside scope

- Report date/time: 2026-11-17, Europe/Berlin.
- Execution interval: qualification clock began 14:46:54.6075613; S1 ended 14:46:59.936629, before target tests.
- Status: inconclusive qualification; frozen attempt stopped; disposition **revise**.
- Spec: [H-MS1 prototype gate](../../../.kiro/specs/model-simplification/prototype-gate.md), P1.
- Hypothesis: H-MS1; predecessor [R119](06-model-simplification-p0-freeze.md).
- Type: execution of the preregistered first baseline; retrospective failure report. Proposed correction below has not executed.

## Decision

The first native baseline check refused the prepared repository. P1 is not
qualified, and no model trial may start. This is a declaration-history defect in
the experimental setup, not a model failure, failed specimen test or evidence
against useful simplification. Preserve freeze-1 and its failure evidence.

## Hypothesis and falsification

H-MS1 remains untested. The immediate assumption that the shared declaration
commit supplies a usable native baseline was falsified: changing both declarations
in one commit violates each responsibility's single-declaration scope boundary.
No measurement threshold, observation boundary or scope rule is relaxed.

## Frozen inputs and method

Used the committed R119 receipt and frozen S1 command without modification.
The frozen verifier checked 278 inputs and 5,514 indexed evidence files; the
completion supplement and its nine evidence hashes also matched. Original source
remained clean at `3a214ff13ee768535481f71258ac2b3529306870`; the private judged
repository remained clean at `4d7afda8adaf3cee407885b708c05e9962ea66fc`.
The frozen Podman image was available. No runtime test was started.

The controller cloned the retained bundle into the scheduled S1 directory,
removed its local origin, charged the start before execution and invoked the
frozen native checker with four workers and a 1,200-second deadline.

## Evidence and results

Artifact root: `D:/BuildArtifacts/lesscode/model-simplification/ms1/p1/`.

- `S1-start.json`: exact command and charged count 32.
- `S1-result.json`: native exit 1 at 14:46:59.936629.
- `S1-verdict.json`: declaration/revisions, pin separation, size and frame checks
  pass; changes-stay-in-scope refuses `.kiro/specs/ms1-branches/step.json`.
- `baseline-diff.txt`: both step declarations changed between `edc60f9` and `4d7afda`.
- `S1-stdout.log` and `S1-stderr.log`: retained process output.
- `result.json`: hash-linked evidence inventory, SHA-256
  `E86A570BDD020D7D437BD4312EAD342F46B6DA74777359F3E98E13F9EB9DD859`.

Native `step/check.rs::outside_scope` admits judged files and the current step's
own files. The other step declaration is neither. The verdict has no functions
run and no behavioral evidence file was produced. S2/B1/B2 were not started after
this mandatory baseline failure. This diagnosis uses retained diff/verdict and
source inspection; their predicted symmetric refusal was not separately executed.

Counts: 1 of 4 qualification checks started, 0 usable baselines, 0 target tests
started in this batch. Cumulative charged starts: 32/80. Model trials: 6 scheduled,
0 attempted, 0 endpoint failures, 0 candidate-producing, 0 behavior-accepted,
0 measurable, 0 useful, 0 adopted. Two responsibilities remain unqualified.

## Limits and interpretation

P0's passing tests cannot replace P1. Contracts S1/S2/U/P and the complete adverse
control matrix remain unadmitted; no numeric ceiling supplement exists. The first
refusal says nothing about their eventual outcome. No production source, specimen,
frozen evidence, tests, scope or model requests were edited.

## Tool/spec consequence and next gate

Proposed bounded correction: create separate private repositories/bundles for S
and B from the retained history. For each, use a common source/test baseline and
change only that case's own declaration in the judged revision. Verify identical
source, pins, runtime, packs and measurement bytes, and explicitly inventory the
different declaration histories. Do not add the other declaration to scope,
weaken the checker, or overwrite freeze-1. Preregister exact new refs, commands,
artifact paths and affected revalidation before creating the revised freeze.

One apparatus revision remains available but has not been executed. Any revision
must respect cumulative limits: apparatus 1,344/1,400 lines, pins 336/600, starts
32/80, calls 0/18. The preparation deadline remains 14:57 Berlin; the qualification
deadline is 16:16:54.6075613 Berlin, including session gaps. Neither clock resets.
If the correction cannot fit those limits, record stop rather than silently
extending them. Normal design approval and independent P4 review remain pending.

Inline completion verification: **NOT_VERIFIED**, claim type **TASK**, claim
"P1 qualification complete". Evidence is the fresh exit-1 verdict; three native
checks and contract/control/numeric admission remain missing. Failure recording
does not mark P1 complete or establish FEATURE_GO.

## COBESY check

Failure is visible first; replay inputs and raw refusal are retained. The setup
assumption is falsified explicitly. Unrun checks and untested model efficacy are
kept separate from the observed scope refusal.
