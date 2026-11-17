# R126: First proposal exposes newline-sensitive boundary comparison

- Written and executed: 2026-11-17, Europe/Berlin.
- Predecessor: [R125](12-model-simplification-p1-qualified.md).
- Spec: [prototype gate](../../../.kiro/specs/model-simplification/prototype-gate.md).
- Status: stopped on a mandatory evaluator control; no candidate behavior accepted.

## Finding and falsification

The first state proposal returned complete, but the frozen boundary checker said
an original signature changed or disappeared. The function headers had not changed
semantically: the frozen original used CRLF and the reply used LF. The checker
stores and compares raw header strings, including their newline bytes.

Two diagnostic controls establish the cause without editing the candidate:

1. Copy the original source, changing only CRLF to LF. The frozen boundary checker
   rejects it with exit 2 and the same signature message.
2. Use that LF original as the diagnostic reference for the retained candidate.
   The frozen checker accepts the boundary. This is diagnostic evidence only;
   it does not admit the candidate or substitute a new original into the experiment.

The first control falsifies boundary-format invariance. R125's four complete native
checks remain valid observations, but full P1 admission is superseded by this newly
demonstrated defect. Measurement formatting controls did not cover this separate
raw-header comparison. No model repair was requested against the evaluator failure.

## Trial and evidence

The existing bridge recorded S1 as incomplete after a not_judged response naming
the boundary defect. No candidate repository or native behavioral check was run.
One provider request produced one complete source reply in 37.31 seconds:
16,826 input tokens, 5,076 output tokens and 1,163 reported reasoning tokens;
provider charges were unavailable. Reasoning-token accounting is retained as
reported, not added to output tokens as an assumed independent quantity.

The [receipt](../../experiments/model-simplification/candidate-stop-receipt.json)
links the result and evidence index under
`D:/BuildArtifacts/lesscode/model-simplification/ms1/`. Candidate bytes, requests,
reply, call receipt, raw facts, diagnostic sources and judge response are preserved.

The proposed code moves three writes through a new private setter and moves an
instance annotation to the class. Raw self.tweets W/S/D becomes 2/1/1 versus 4/3/1;
whole-file W becomes 8 versus 10, while U rises from 82 to 86. The three new calls
and class annotation remain unresolved; the new helper has no admitted effect
contract. This is NotMeasurable, not a useful state simplification. The annotation
also changes observable class metadata. No test-based behavior claim is made.

## Invocation and resource accounting

47/80 target checker/control starts remain consumed. Static boundary diagnostics
do not run specimen tests. Model calls: 1/18; scheduled trials: six; attempted: one;
candidate-producing: one; behavior-accepted/measurable/useful/adopted: zero.
S2, S3, B1, B2 and B3 are explicitly not attempted because of the global stop.

The candidate window started 17:49:21.7470226 and retains its 21:49:21.7470226
deadline. No elapsed time or consumed request is refunded. No fourth version has
been applied or executed. Apparatus remains 1,358/1,400 lines and fixtures 357/600.

Before the live request, the tool's interactive terminal creation was denied before
any process started. A piped PowerShell launch then revealed that the Node tool's
PATHEXT was only .CPL, causing document launch instead of native command execution.
The orphaned stub process was stopped. Setting PATHEXT=.EXE in that child only
allowed a successful stub judge handshake. The actual model identity and absence
of model/reasoning overrides were checked; the live call used the frozen request.
These are controller setup observations, not Podman failures or extra model calls.

## Prepared correction, not applied

The exact one-line patch is
`D:/BuildArtifacts/lesscode/model-simplification/ms1/p2-p3/proposed-boundary-correction.patch`.
In boundary.rs, replace raw header `.to_string()` with
`.replace("\r\n", "\n")`. Do not normalize or edit model source bytes. Preserve
all other signature and exterior checks.

Required regressions: original CRLF versus LF accepted in both directions;
genuine parameter/signature changes still rejected; allowed private helper still
accepted; B exterior edits still rejected. Then rebuild and freeze the changed
spike, verify all unaffected native-check inputs by hash, and amend admission.
Earlier native observations may be reused only under that verified identity.

Continuing requires owner authorization for a fourth apparatus version: the
approved revision cap is three. Proposed bounded next step is at most 15 minutes
for this correction and controls, keeping both existing deadlines and all counters.
S1 remains terminal/incomplete; only the remaining five scheduled trials may run.
No retry, extra trial, broader rewrite or production approval is implicit.

## Interpretation

This is a specific bug in our evaluator, not evidence that Podman is unavailable.
The failed control prevents a valid model-effectiveness conclusion. Preserve both
the successful native qualification and the later falsification; do not relabel
the false boundary rejection as a model defect or manufacture a useful result.
