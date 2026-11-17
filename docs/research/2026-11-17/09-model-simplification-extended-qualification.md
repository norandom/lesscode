# R122: Preregister the owner-approved longer P1 qualification run

- Written: 2026-11-17, Europe/Berlin, before the extended checks.
- Status: authorized, execution pending; no qualification success claimed.
- Spec: [prototype gate](../../../.kiro/specs/model-simplification/prototype-gate.md).
- Hypothesis: H-MS1; predecessor [R121](08-model-simplification-p1-timeout.md).
- Type: preregistration of an operational-budget amendment.

## Decision and authorization

After R121 stopped, the owner selected option 1: keep frozen code and checks
unchanged, raise the whole-check limit from 20 to 40 minutes, and grant P1 a fresh
three-hour budget. Model trials may follow only if every P1 gate passes. This
explicit decision supersedes R121's prohibition on an automatic timeout reset.
It does not approve production work, a third measurement implementation or weaker
tests. Earlier failures and consumed invocation counts remain retained.

## Hypothesis and falsification

The operational hypothesis is that the unchanged native comparisons complete
within 2,400 seconds each and all four qualification checks plus remaining
admission fit the fresh 180-minute window. An incomplete or refused comparison,
failed mandatory control, changed frozen input or expired global window prevents
model calls. No inference about model simplification follows from setup alone.

## Frozen inputs and execution

Use freeze-2, SHA-256
`3A4B12DCB89D328E0096D50D0F446A0DEE3CA5F476059E187108182FE58ABF62`,
and its immutable freeze-1 runner/binary/request references. The two case refs
remain S `ccb65f3f102c1d1cd8d4cb409e438fbd4450dab6` and B
`eeac790aa80b2801923a9462ebaa64f6e123c7d2`.

Run S1, S2, B1 and B2 serially in fresh `qualification-extended/` repositories
and `qualification-work-extended/` output directories under the existing MS1
artifact root. Clone the matching bundle with explicit `--branch main`, remove
origin and verify the expected ref. This preserves the disclosed main-selection
intervention from R121 while avoiding the known empty checkout. Do not reuse the
timed-out work directories or partial caches.

Keep the native command, four fault workers, two pytest workers, observation pins,
scope, packs, runtime and declarations unchanged. The enclosing checker watchdog
allows 2,400 seconds; individual target-run limits remain as frozen. Stop children
and their identified containers on deadline; retain incomplete evidence.

Write the fresh start and three-hour deadline to `p1-extended/clock.json` before
execution, and each exact command, start count and result beside its logs. Begin
at cumulative count 36/80 and charge every new outer invocation before it starts.
Model calls remain 0/18 and trials 0/6; the existing candidate-phase four-hour
budget and repair bounds are unchanged. No apparatus code or pin changes are
authorized by this amendment.

## Required admission and interpretation

All four native baselines must be usable. Review matching observations, frozen
control evidence, S1/S2/U/P contracts and complete responsibility/envelope facts;
retain a hash-linked admitted baseline and ceilings before any live call. R121's
conditional counts are not automatically admitted by a passing process exit.
Keep the two responsibilities and six predeclared trials. Independent P4 review
is still required before a proceed decision, and normal design approval remains
separate. No subagent execution is authorized.

## COBESY check

The changed bound, unchanged protections and explicit falsifier precede execution.
The earlier timeout remains evidence and consumed work, not a result to overwrite.

## Execution amendment, 2026-11-17, 15:50 Berlin

The first extended S1 check (start 37) ended at 15:48:12.9143105 with exit 2,
not_judged. Both fault populations returned 53 killed/11 survived, but one
step-side closing baseline failed on a forcibly closed Podman SSH connection.
That makes the entire result void; matching fault counts do not rescue it.

Read-only Podman health and frozen-image checks then succeeded without a service
restart. One fresh full retry is recorded under `p1-extended-retry/`, with new
repository/work directories, unchanged checks and no cache reuse. It consumes
start 38 and the existing clock, whose deadline is 18:25:47.6266045 Berlin.
The remaining S2/B1/B2 checks follow only on success. A repeated infrastructure
failure stops this batch; no repeated automatic recovery or budget reset is
authorized by this operational retry. Raw failed logs and runtime-recovery.json
remain under `p1-extended/`. This is not model repair or acceptance of void evidence.

## Final disposition amendment

[R123](10-model-simplification-infrastructure-classification.md) supersedes the
running status. Inspection found an actual infrastructure exit 125 counted as
killed in the first extended check. The retry was deliberately stopped at 16:03:27
on this mandatory correctness failure, not on a second timeout or SSH failure.
No model calls ran; correction of the frozen checks requires explicit authorization.
