# R127: Header newline normalization restores the boundary controls

- Written and executed: 2026-11-17, Europe/Berlin; correction began 18:04.
- Predecessor: [R126](13-model-simplification-boundary-newlines.md).
- Spec: [prototype gate](../../../.kiro/specs/model-simplification/prototype-gate.md).
- Type: preregistered correction; no production approval.

## Hypothesis and owner decision

The owner requested the next group as a whole and explicitly requested parallel
subagents after the recap identified fourth-version authorization as the next
step. This authorizes the proposed one-line correction and independent review.
It does not reset clocks, retry S1, increase call/check limits or approve production.
The correction/control window is 18:04–18:19; P1 still ends 18:25:47.6266045.

Raw compared function headers now normalize CRLF to LF. Candidate source bytes
remain untouched. The hypothesis fails if newline-only originals still fail or
if genuine signature/exterior edits become admissible. The patch changes one
line; no parser, measurement rule, runner, model request or dependency changes.

## Evidence and interpretation

Nine fresh CLI controls have their expected outcomes: unchanged S; CRLF/LF in
both directions for both cases; two genuine signature changes; B exterior edit;
and allowed private helper. The four newline cases all fail against freeze 3.
An independent agent reran these controls and inspected each fixture difference.
The existing spike nextest suite passes all 21 tests with none skipped.

The independent evidence audit verified 282 old frozen inputs, all 12,414 P1
evidence files and 5,514 earlier evidence files. Original specimen revision and
tracked cleanliness, private case bundle refs and admission digests match.
Freeze 4 contains 283 inputs and verifies. Only boundary.rs, the spike binary,
the plan and new preregistration differ; all other frozen inputs match.

Artifacts are retained beneath
`D:/BuildArtifacts/lesscode/model-simplification/ms1/`: `boundary-controls-v4/`,
`boundary-revision-4/` and `freeze-4/`. Freeze SHA-256:
`03EC884DAA1112234D597097B7D575CFA3186E87E3BAD08DAC2C06E63A279660`.
The [admission receipt](../../experiments/model-simplification/p1-v4-admission-receipt.json)
links the supplement: both fresh original census hashes exactly match their
admitted baselines. Correction controls completed at 18:09:53.2980360, inside
the bound. P1 is restored before its deadline; remaining trials may proceed.

## Limits and next gate

Native qualification observations retain their original execution dates; this
correction does not rerun specimen behavior or imply candidate acceptance.
S1 remains terminal/incomplete. Remaining trials are S2, S3, B1, B2 and B3,
serial at the endpoint. The retained candidate clock uses 1,200 seconds per
checker and ends 21:49:21.7470226; the P1-only extension is not applied to it.
Counters start at 47/80 checks and 1/18 calls. Apparatus remains 1,358/1,400
lines and fixtures 357/600; control source copies are retained diagnostic inputs.

Storage gate: attributed disposable caches 16.66 GiB; D free 214.98 GiB.
MS1 evidence measured 2.913 GiB before this group. Shared dependencies are not
exclusively attributable. Reuse the shared target; do not clean another active
project's output. No new isolated build directory was created.
