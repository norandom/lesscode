# Task 56 handover: extended P1 stopped on infrastructure false-kill classification

2026-11-17, Europe/Berlin. Read [R123](../docs/research/2026-11-17/10-model-simplification-infrastructure-classification.md)
and its [receipt](../docs/experiments/model-simplification/extended-p1-result-receipt.json).

Owner selected option 1: unchanged checks, 40-minute whole checks, fresh three-hour
P1 window. R122 preregistration commit 85e888f. Clock began 15:25:47.6266045;
deadline remains 18:25:47.6266045 Berlin. No counters were refunded.

Extended S1 start 37 ended 15:48:12, exit 2/not_judged: a step closing baseline
failed on Podman SSH connection reset. Health/image checks subsequently passed,
without restarting any service. One fresh full retry started at 15:51:11 (start 38).

While it ran, inspection found extended S1 step mutant 50 (and-to-or at exporter.py
line 115) classified killed with exit 125. Config uses generic passed/failed runner
evidence, omits error patterns and does not classify 125 specially. Frozen runner
uses 125 for infrastructure failure; official Podman docs corroborate that code.
Both aggregate populations say 53/11, hiding the erroneous class. Exact mutant-50
stderr is unavailable; do not claim its SSH cause is proven from the closing error.

Retry was deliberately stopped 16:03:27; native exit -1, not a timeout or second
transport failure. Its process and four identified containers are stopped. No
checks remain running. S2/B1/B2 and all model trials remain unrun.

Unapplied correction sketch:
`D:/BuildArtifacts/lesscode/model-simplification/ms1/infrastructure-classification/proposed-correction.txt`.
Emit a specific infrastructure marker for setup exceptions/Podman exits 125–127;
map it through both declarations' existing error_output_patterns. No core change
is needed. Verify injected infrastructure failures become error/incomplete and a
genuine assertion failure remains killed before requalifying. The sketch is not
an applied or verified fix and does not solve the unknown SSH-reset cause.

Await explicit authorization for this harness revision: option 1 required unchanged
checks. Proposed bound is 15 minutes for correction/controls while retaining the
18:25:47 deadline; no new clock is implicit. If that deadline has elapsed, ask for
a time decision as well. Keep original freezes/evidence immutable and use a new
freeze for any approved revision. No subagent authorization exists.

Counters: 38/80 outer starts; 0/18 calls; 0/6 trials. Two apparatus versions executed,
1,344/1,400 maintained apparatus lines, 336/600 pin lines. The draft is unexecuted.
P1 NOT_VERIFIED; P1-P4 unchecked; solution-unlock stop; production approvals false;
no independent P4 review or model-efficacy claim.

Final result `extended-p1-final.json` SHA-256
D323CBF83EB126C019A233A867CD79CB39DA24455A888736171FC6C066D178C0.
Index covers 4,792 files and 232 nested test-run directories (distinct from outer
invocation counts). All artifacts are under the MS1 root. Preserve unrelated
untracked files; no specimen writeback, production changes or push occurred.
