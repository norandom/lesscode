# Task 58 handover: boundary fix verified; checkout preflight stops continuation

2026-11-17, Europe/Berlin. Reports:
[R127](../docs/research/2026-11-17/14-model-simplification-boundary-correction.md),
[R128](../docs/research/2026-11-17/15-model-simplification-checkout-preflight.md).
The owner authorized the next group and parallel subagents. Three agents handled
the narrow fix, evidence/structural audit and independent controls/P4 review.
Model trials remained serial as required by the frozen protocol.

## Completed and verified

- Commit `329adc9`: exact one-line CRLF header normalization. Nine boundary
  controls pass; four newline controls fail against old freeze 3; 21 nextest
  tests pass. Controls finished 18:09:53 inside the 18:04–18:19 bound.
- Fresh S/B original fact hashes match exactly. Earlier 12,414 P1 evidence files,
  5,514 older evidence files and 282 frozen inputs were independently verified.
- Freeze 4 SHA-256:
  `03EC884DAA1112234D597097B7D575CFA3186E87E3BAD08DAC2C06E63A279660`.
  Admission SHA-256:
  `DC2F48C4D6E6FDA33884F65260E913D41B1A0B81D1A6D6CC9BECA056B0668F68`.
- S2 returned an unedited setter extraction. Boundary accepted, but U rises
  82 to 85: independent structural review says NotMeasurable. Raw target W/S/D
  changes 4/3/1 to 2/1/1; this is not accepted useful simplification.

## Stop and correction needed

Native S2 check began 18:12:17; stopped 18:28:18.7547076 after the reviewer found
three candidate-worktree pins translated LF to CRLF. The controller cloned with
system autocrlf=true, then set false too late. Its 203-other-files check compared
the clone before/after reply installation, not against frozen expected bytes.
This is a controller preflight failure. No candidate source was hand-edited.

All actual native base/step pin bytes match the freeze, as do all 24 earlier
qualification-side pin hashes. Candidate committed blobs also match. This was
checkout conversion before execution, not demonstrated runtime mutation.
Earlier native observations remain evidence. S2 base completed 53 killed/11
survived of 64 and four closing passes; step stopped with 27/5 partial outcomes.
No complete candidate behavior verdict exists. S2 received not_judged and its
terminal trial record is incomplete with one request/reply, no repair.

P4 independent review requires global stop under the unchanged protocol. S1/S2
remain terminal; S3/B1/B2/B3 remain not attempted. Total 48/80 outer starts,
2/18 calls, 2/6 attempted and candidate-producing, zero behavior-accepted,
measurable, useful or adopted. Apparatus 1,358/1,400; fixtures 357/600. No refunds.
Qualification deadline 18:25:47 has passed; original candidate deadline remains
21:49:21.7470226. Candidate checker bound is 1,200 seconds (2,400 was P1-only).

## Evidence, storage and next decision

[Group receipt](../docs/experiments/model-simplification/group-58-result-receipt.json)
links the result, 2,929-file index and independent `p2-p3/p4-review-v4.json` beneath
`D:/BuildArtifacts/lesscode/model-simplification/ms1/`. Preserve candidates,
partial native runs, pin discrepancies, requests and prior freezes. All scoped
processes ended; four identified check containers were removed. Other projects
were untouched. Original specimen remains tracked-clean at `3a214ff13ee...`.

Attributed caches approximately 17.59 GiB; D free 212.31 GiB. Retained MS1 evidence
3.355 GiB, approximately 0.442 GiB growth. Shared debug was reused; no isolated
build directory was created. Retain shared build outputs for reuse and frozen
binaries/evidence for audit; other shared dependencies are not exclusively owned.
No cache cleanup, C: storage relocation, upstream writeback or push was performed.

Proposed next decision, not executed: authorize bounded checkout/preflight repair
and a disposition for the four remaining trials. Set autocrlf=false at clone,
verify every protected working file against the freeze or explicitly frozen case
declaration blob, and reject a deliberate CRLF pin mismatch before calls/tests.
Cover both cases and preserve all clocks/counters unless expressly amended.
Do not silently restart S1/S2 or generate production tasks. R128 states the exact
proposal and evidence limits. Clear conversation context before the next group.
