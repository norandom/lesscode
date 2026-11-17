# R121: Revised baseline exceeds the checker bound; H-MS1 stops before model trials

- Report date/time: 2026-11-17, Europe/Berlin.
- Execution interval: revision prepared and frozen by 14:54:54.1496368; native check started 14:56:24.5626386 and was interrupted at its 1,200-second deadline. Descendant cleanup recorded at 15:16:26.5933528.
- Status: stopped bounded attempt; P1 incomplete.
- Spec: [prototype gate](../../../.kiro/specs/model-simplification/prototype-gate.md), P1-P4 disposition.
- Hypothesis: H-MS1; predecessor [R120](07-model-simplification-p1-declaration-refusal.md).
- Type: revision preregistered in retained JSON before execution; retrospective report of correction, controls and timeout.

## Decision

The complete bounded group stops here. The corrected state baseline passed the
scope guard and completed its base-side fault campaign, but the step-side campaign
did not finish within the frozen 1,200-second checker bound. There is no complete
native verdict or usable P1 baseline. S2/B1/B2 and all six model trials remain
unattempted. Both permitted apparatus versions have been used. No third revision,
longer checker deadline or reset of cumulative effort is inferred from this result.

This is incomplete experimental setup evidence. H-MS1's claim about useful model
simplification remains untested; the timeout is not a model failure, killed fault,
behavioral rejection or evidence that the proposed reductions are impossible.

## Hypothesis and falsification

The bounded revision tested whether separate declaration histories could remove
R120's scope refusal without changing the specimen, pins, measurement or native
checker. That narrow correction worked: the checker reached its test campaigns.
The further prerequisite that a usable native comparison fits the declared
20-minute bound failed in this execution. Neither result establishes compression.

The owner explicitly requested continuing the whole group in the current context.
This superseded the context-clear pause, not the protocol's limits or gates.

## Frozen inputs and method

All artifact paths below are relative to
`D:/BuildArtifacts/lesscode/model-simplification/ms1/`.

`revision-2/preregistration.json` precedes the changes. Two private `main` commits
each change only their own declaration's base to the common retained `4d7afda`:

- S: `ccb65f3f102c1d1cd8d4cb409e438fbd4450dab6`.
- B: `eeac790aa80b2801923a9462ebaa64f6e123c7d2`.

Separate bundles are in `freeze-2/revision/`. Freeze-2 contains 282 verified inputs;
SHA-256 `3A4B12DCB89D328E0096D50D0F446A0DEE3CA5F476059E187108182FE58ABF62`.
Of the 278 previous inputs, 277 are byte-identical; only execution-plan.json
changed, and four revision records/bundles were added. Source, pins, profile,
requests, runtime, packs, native binaries and apparatus code did not change.
The new plan retains references to the immutable freeze-1 runner/binaries/requests.
The separate case refs/bundles supersede the old shared declaration provenance.
Original freeze-1 and R120 failure evidence remain intact.

Preparation finished before its unchanged 14:57 deadline. P1's fixed global
deadline remained 16:16:54.6075613, with no refund for session gaps or failures.

## Evidence and results

The first revised S1 invocation, start 33, exited 3 before tests. Its bundle
contained `main` without a HEAD entry, so clone left an empty working tree;
removing origin also removed the remote main ref. An initial checkout failed.
The controller fetched `main:main` from the same retained bundle, checked out main,
and verified the exact frozen revision and own-declaration-only diff. This was an
additional manual checkout intervention, not a capability of the frozen clone
command. It changed no frozen bytes or intended revision. All failures remain in
the record. Start 34 retried the same native checker command.

| Evidence | Result |
| --- | --- |
| `p1-v2/S1-result.json`, `S1-clone.log`, `S1-checkout.json` | Empty-checkout failure and disclosed recovery |
| `qualification-work-v2/S1/base-run.json` | Complete base campaign: 64 faults, 53 killed, 11 survived; all four closing baselines passed |
| `p1-v2/S1-timeout.json` | Whole-check deadline reached; step-side report and final native verdict absent |
| `p1-v2/callback-order/results.txt` and log | Frozen pins reject the injected ordering fault: 9 failed, 87 passed, expected exit 1 |
| `p1-v2/partial-failure-state/results.txt` and log | Frozen pins reject prematurely replaced state: 7 failed, 89 passed, expected exit 1 |
| `p1-v2/structural-controls/results.json` | Dynamic field, generic setter, escaped alias, helper relocation and hidden table predicate retain explicit U; no useful verdict |
| `p1-v2/facts-S-{1,2}.json`, `facts-B-{1,2}.json` | Repeated extraction is byte-identical for each case |
| `p1-v2/conditional-baselines.json` | Raw counts and derived ceilings retained, explicitly not admitted |

The whole-check deadline stopped the native process. Cleanup identified its
descendants by parent process IDs and creation times and stopped its four named
test containers. It did not stop unrelated services or delete evidence. No test
ending because of that interruption is counted as a killed fault or valid verdict.

The [compact receipt](../../experiments/model-simplification/p1-result-receipt.json)
links `p1-v2-final.json`, SHA-256
`323F97D040FE50AA663778672532323DB416C7D8BDDF460DE730C027DF4941BF`.
Its index covers 2,850 retained evidence files, including 118 internal test-run
directories created during the revised native campaign and two direct controls.
Those internal runner executions are reported separately from the protocol's
outer checker/control invocation counter; they are not 118 independent native
checker invocations or 118 successful tests.

Outer charged starts are **36/80**: 31 before P1, the original scope refusal,
the revised empty-checkout failure, the timed-out native retry, and two behavioral
controls. Apparatus code remains 1,344/1,400 maintained lines and pins 336/600;
no production or apparatus source changed in this revision. Qualification used
two of two apparatus versions. Calls 0/18; trials 6 scheduled, 0 attempted,
0 endpoint-failed, 0 candidate-producing, 0 behavior-accepted, 0 measurable,
0 useful and 0 adopted. Both responsibilities remain unqualified.

## Limits and interpretation

The two extra faults test the existing pins; their expected failures do not
establish native baseline acceptance. The base-side 53/64 detection result is
partial evidence, not the final before/after comparison. Contracts S1/S2/U/P and
the full control/admission matrix remain incomplete. Conditional W4/S3/D1 and
B7/P17 imply ceilings W3/S2/D1 and B5/P15, but none is an admitted trial baseline.
No model endpoint was contacted, no candidate generated and no specimen adopted.

The timed-out command retains four native workers and two pytest workers per
test run. The two direct controls overlapped part of the native campaign; this
execution detail is disclosed rather than attributing the timeout to a particular
component or extrapolating it to other runs. Runtime bounds are operational
constraints, not performance benchmarks or grounds for weakening protection.

## Tool/spec consequence and next gate

Solution-unlock disposition is **stop** for this bounded attempt. P1-P4 remain
unchecked, production eligibility false, design/tasks approvals unchanged, and
independent P4 review not performed. Further execution needs a new explicit
protocol/budget decision; do not silently resume partial caches under a reset
checker deadline or treat the remaining global minutes as an override.

Inline completion verification: **NOT_VERIFIED**, claim type **TASK**, claim
"P1 qualification complete". Fresh evidence is the deadline interruption and
absence of a complete native verdict; remaining native checks and admission are
missing. This report closes the group's stopped attempt, not the qualification
task or feature. No FEATURE_GO is claimed.

## COBESY check

The stop and its narrow meaning come first. Both corrections and failures are
replayable; untested model efficacy is separate from setup findings. No test,
threshold, runtime bound or scope was weakened to obtain a positive outcome.
