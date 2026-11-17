# R123: Infrastructure exit 125 was counted as a killed mutant; P1 stops

- Written: 2026-11-17, Europe/Berlin.
- Execution interval: extended P1 began 15:25:47.6266045; first check ended 15:48:12.9143105. Its fresh retry began 15:51:11.3548730 and was deliberately interrupted at 16:03:27 after the classification defect was found.
- Status: stopped; mandatory infrastructure classification is unqualified.
- Spec: [prototype gate](../../../.kiro/specs/model-simplification/prototype-gate.md), P1/U3/U6.
- Hypothesis: H-MS1; predecessor [R122](09-model-simplification-extended-qualification.md).
- Type: execution report and inline root-cause investigation; no fix applied.

## Decision

Longer runtime let the first extended native check finish, but it returned
not_judged after a Podman connection failure in a closing baseline. Inspection
then found a separate correctness problem: its step-side report classifies a
mutant with exit 125 as killed. The mandatory distinction between infrastructure
failure and detected behavioral fault therefore fails. P1 cannot admit these
checks, and no model call may follow.

The owner selected longer execution with frozen code/checks unchanged. Fixing the
runner/declaration configuration falls outside that amendment. An exact, unapplied
correction sketch and verification plan are retained for review; no frozen input,
production source, specimen or test was changed to obtain a passing result.

## Hypothesis and falsification

R122's longer-bound hypothesis was operational, not a claim of model efficacy.
The first check finished within 2,400 seconds, so its failure was not another
whole-check timeout. However, the assumption that the frozen configuration keeps
infrastructure failures out of the killed-fault population was falsified by a
retained native result. H-MS1 model simplification remains untested.

## Frozen inputs and method

Freeze-2, its two case refs, native binaries, profile, packs, source, pins and
individual test-run settings remained unchanged. The three-hour window started
at 15:25:47.6266045 and ends at 18:25:47.6266045 Berlin. Starts were charged before
execution; fresh repositories/work directories were used, without partial-cache
reuse. Four native workers and two pytest workers per target run were retained.

The first extended S1 check consumed start 37. After it returned not_judged,
read-only Podman health and image-availability checks passed. No service restart
occurred. One full retry consumed start 38 within the same clock. That retry was
stopped when the prior run's false classification was identified; it did not
independently time out or demonstrate a second SSH failure.

## Evidence and results

Artifact root: `D:/BuildArtifacts/lesscode/model-simplification/ms1/`.

| Evidence | Observed result |
| --- | --- |
| `p1-extended/S1-result.json` | Exit 2, not timed out, ended 15:48:12.9143105 |
| `qualification-extended/S1/.kiro/specs/ms1-state/verdict.json` | not_judged: step worker 3 closing baseline failed with Podman SSH connection reset and exit 125 |
| `qualification-work-extended/S1/step-run.json`, mutant 50 | `class: killed`, `exit_code: 125`, `reason: null` |
| `p1-extended/false-kill-125.json` | Extracted offending record, preserving site/operator/hash |
| `p1-extended/runtime-recovery.json` | Read-only health/image checks passed; no service restart or deadline reset |
| `p1-extended-retry/S1-result.json`, `safety-stop.json` | Controller interruption, exit -1, not timeout; descendants and four identified containers stopped |
| `infrastructure-classification/finding.json` | Root cause, source references, official documentation and unapplied proposal hash |

Mutant 50 changes `and` to `or` in the condition
`self.checkpoint and not stop_callback` at exporter.py:115. Its SHA-256 is
`9922c323d995e5f71068e1bc06d43e31b1be5981ee4611d772423f0877b7f598`.
The base-side record for the same mutant has exit 1. Both aggregate populations
say 53 killed/11 survived, but the step-side killed count includes the exit-125
record. Matching aggregates do not validate that classification or permit replacing
the step observation with the base observation.

The [compact receipt](../../experiments/model-simplification/extended-p1-result-receipt.json)
links `extended-p1-final.json`, SHA-256
`D323CBF83EB126C019A233A867CD79CB39DA24455A888736171FC6C066D178C0`.
The evidence index contains 4,792 files and 232 internal test-run directories from
the extended run and retry. Those nested runner executions are distinct from the
cumulative **38/80 outer checker/control starts**. Model calls 0/18; trials 6
scheduled, 0 attempted, 0 endpoint-failed, 0 candidate-producing, 0 behavior-accepted,
0 measurable, 0 useful, 0 adopted. S2/B1/B2 remain unrun.

## Root cause and smallest correction

Inline kiro-debug result:

- **ROOT_CAUSE:** the frozen declarations supply `runner_evidence_patterns` of
  `passed` and `failed`, no `error_output_patterns`, and invalid exit codes 2–5.
  The runner defaults to 125 on setup exceptions and propagates Podman exec errors.
  In [classes.rs](../../../native/src/adapters/driver/classes.rs), an unrecognized
  nonzero test exit with either generic output word reaches the killed result.
- **CATEGORY:** CONFIG_GAP.
- **CONFIDENCE:** HIGH for the false classification; the underlying SSH reset
  cause is not determined.
- **FIX_PLAN:** in a new harness revision, emit `MS1_INFRA_FAILURE` for setup
  exceptions and Podman execution failures 125/126/127; add that marker to both
  declarations' `run.error_output_patterns`. The existing core already supports
  this field through [declaration.rs](../../../native/src/step/declaration.rs).
  Keep source, pins, metric profile, packs and core unchanged. Publish a new
  immutable freeze and separate case declarations; never edit an old freeze.
- **VERIFICATION:** inject infrastructure failures at mutant execution and require
  error/incomplete, never killed; include a genuine assertion failure that must
  remain killed, plus passing opening/closing baselines. Requalify both cases.
- **NEXT_ACTION:** STOP_FOR_HUMAN, because the selected option explicitly required
  unchanged checks; the needed harness/configuration amendment is not authorized.

The exact sketch is `infrastructure-classification/proposed-correction.txt`,
SHA-256 `A14683C10CB43D696BFFDEBF15A50332A197E048CD071A2D8C3DC38E72625FA5`.
It is a reviewable sketch, not an applied patch or verified fix. Simply putting
125 in invalid_exit_codes is not the proposed correction: a machine failure must
make evidence incomplete, not merely exclude a mutant as invalid.

[Podman's official exec documentation](https://docs.podman.io/en/latest/markdown/podman-exec.1.html#exit-status)
identifies 125 as a Podman failure. The captured closing-baseline message includes
the SSH connection reset. The exact output of mutant 50 is not retained in its
record, so this report does not assert that its particular failure had the same
SSH cause. The frozen configuration and actual exit/class pair establish the
classification defect without that inference.

## Limits and next gate

Safe error classification would not itself fix transport reliability or guarantee
successful qualification. No transport tuning, service restart, production patch,
new pin or core change was attempted. Earlier frozen evidence and failed outcomes
remain intact. Two apparatus versions have executed; the draft is not a third
executed version. Maintained apparatus/pin counts remain 1,344/1,400 and 336/600.
The observed native run was not accepted: its separate closing-baseline guard
correctly returned not_judged despite the erroneous individual mutant label.

Proposed next authorization, not yet granted: one narrowly scoped harness revision
with up to 15 minutes for correction and deterministic controls, retaining the
current 18:25:47 P1 deadline and all consumed counts. No automatic clock reset or
broader implementation follows. If approved after that deadline, a new time
decision would also be needed.

P1 remains NOT_VERIFIED; P1-P4 stay unchecked, production eligibility false and
independent P4 review not performed. A passing transport health query or a matching
fault total cannot substitute for complete qualified evidence.

## COBESY check

The negative finding comes first, with the exact falsifying record. Actual failure,
inferred mechanism, unknown transport cause and unverified correction are separated.
No model result or compression claim is manufactured from setup progress.
