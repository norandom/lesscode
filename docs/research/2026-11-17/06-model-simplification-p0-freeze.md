# R119: P0 apparatus and evidence frozen; qualification remains pending

- Report date/time and timezone: 2026-11-17, Europe/Berlin; finalized with the P0 commit.
- Execution interval: preparation clock began 12:57; final freeze created 14:29:54.992; verification and documentation followed in this session. Precise build/test timestamps are retained in the artifact tree. This interval includes preceding R116-R118 work and session gaps.
- Status: completed preparation; H-MS1 qualification not-run.
- Spec: [prototype gate](../../../.kiro/specs/model-simplification/prototype-gate.md), P0; [preparation](../../../.kiro/specs/model-simplification/prototype-preparation.md).
- Hypothesis and predecessor: H-MS1; [R118](05-model-simplification-predicates-bindings.md).
- Type: retrospective preparation report under the approved bounded protocol; not a candidate experiment.

## Decision

P0 is complete. The frozen apparatus can now enter P1 qualification. It has not
established a measurable or useful candidate, admitted the effect contracts, or
passed the native step baselines. No model endpoint was called. Production design
approval, task generation and implementation remain blocked by the research gate.

## Hypothesis and falsification

H-MS1 is unchanged: the existing seam/checker plus a small companion may yield an
unedited, behavior-preserving reduction in repeated writes or decision work. P0
makes that test executable; it does not confirm it. P1 must reject unsupported
aliases/effects or failed controls rather than narrow the approved responsibility.

The owner explicitly approved adding 60 preparation minutes: 120 total, ending
14:57 Berlin from the recorded 12:57 start. Code, pin, model-call and checker caps
were not raised. There are 1,344 canonical maintained Rust/PowerShell apparatus
lines, including existing preparation glue, and 336 pin lines; caps are 1,400/600.
The frozen plan conservatively charges 31 of 80 target-test/collection/checker
starts before P1. Six trials and at most 18 seam calls remain scheduled, with none
attempted. P1 has its separate 90-minute limit; no effort is refunded.

## Frozen inputs and method

The [compact receipt](../../experiments/model-simplification/freeze-receipt.json)
identifies `D:/BuildArtifacts/lesscode/model-simplification/ms1/freeze-1/freeze.json`,
SHA-256 `196BA0BA948224DD370BEDD98153C1C14B58239DEE8936766A0527052E524CDC`.
It contains 278 frozen inputs and an index of 5,514 separately retained evidence
files. All hashes were verified. These local bytes are required for replay; the
receipt is not a substitute if they disappear.

The upstream specimen is unchanged at `3a214ff13ee768535481f71258ac2b3529306870`.
The isolated judged repository has pin baseline
`edc60f91c3daf86bdc7d4078fec7c0c3beed55e4` and declaration revision
`4d7afda8adaf3cee407885b708c05e9962ea66fc`, retained in a Git bundle. Its original
private root is `4c8ff0349699774220a557e0dd23393863d5e621`. No upstream remote or
specimen writeback is used. MIT is declared in the retained pyproject; the source
has no tracked standalone LICENSE/NOTICE file. Full tracked source/test/support
and caller inventories are retained, including callers outside model context.

The image is `c2d586c24d16ae0e9d614d53be8bdb50fab61ad83a0a4af40abf63c510473a64`:
Python 3.12.12, pytest 9.1.1, xdist 3.8.0, execnet 2.1.2, ruff 0.17.2 and uv 0.9.7.
Tests run with network disabled, UID/GID 65534, root-owned read-only source, two
pytest workers and temporary storage inside the container. The 47 original
selected tests plus 49 new pins give 96 tests; unrelated suites are explicitly
excluded in the plan. Dependency locks, runtime archive, executable hashes and
installed versions are retained. Native production and legacy pack bytes are
unchanged. The spike reuses native parsing and the existing model repair loop.

Exact commands, four qualification copies, six trial paths, requests, context,
effect contracts and intervention policy are in frozen `execution-plan.json` and
`apparatus/controller-procedure.md`. Endpoint/model configuration is recorded as
Anthropic `https://api.anthropic.com`, `claude-opus-5-5`, without a probe; temperature
and reasoning are unset. Output ceiling is 8,192 tokens; existing wait/answer limits
are 600/3,600 seconds. Credentials are referenced, never copied or hashed.

## Evidence and results

All artifact paths below are relative to `D:/BuildArtifacts/lesscode/model-simplification/ms1/`.

| Evidence | Observed result |
| --- | --- |
| `p0/final-nextest.log`, `p0/final-build.log` | 21 nextest tests passed; locked offline build succeeded |
| `p0-controls-sealed/baseline-one.log`, `baseline-two.log` | 96 passed twice, 10.31s and 6.59s |
| `p0-controls-sealed/results.txt` and individual logs | Alias allocation, callback-state, wrong reason, precedence and wrong return faults all exited 1 as expected |
| `p0/facts-exporter-1.json`, `facts-exporter-2.json`, corresponding tools files | Repeated companion reports match; facts remain conditional |
| `p0/mutants-*-before.json`, `p0/mutants-*-after.json`, `p0-mutants/` | Complete legacy populations match after output-path normalization: 142 exporter, 239 tools |
| `p0/stub-*` | Accepted unchanged stops after one request; rejection then repair uses two; refusal uses one; cutoff exhausts three; terminal judge error stops after one |
| `p0-postfreeze/boundary-*.log`, `drive-path-escape.log` | Unchanged source accepted; exterior constant/signature changes and escaping output path rejected |
| `freeze-1-verification.log`, `p0-postfreeze/guard-*.log` | Complete freeze verified; positive fixture accepted; tampered/missing input, stale manifest digest and escaping input rejected |

The raw target relation `<module>::XLikesExporter#1::self.tweets` has W=4, S=3,
D=1 (two writers) under the explicit lexical/direct-field contracts. The four
validators have B=7 and P=17. These are not admitted numeric baselines; P1 must
establish their coverage and freeze integer ceilings before proposals. Unknown
calls/aliases remain U. The legacy tools report retains its dynamic-attribute gap
at line 567 outside the four validators; no pack alteration hides it.

Preparation failures are retained. An initial container copy lacked `/case`; a
precedence mutation did not match CRLF input; an initial dependency resolution
expanded newer versions and was interrupted before reusing the existing seam lock.
The first read-only assertion run failed because extracted Windows tar permissions
remained writable (one failure, 95 passes). Explicit root ownership and removal of
write bits fixed the final runner; all seven sealed control runs then behaved as
expected. Earlier logs/bundles remain evidence, not successes or model failures.

## Limits and interpretation

The profile is a bounded lexical analysis with explicit direct-field and list
mutation contracts, not general alias or effect inference. Contract admission is
pending. It must not turn unknown effects into zeros or use smaller writer counts
as proof of useful reduction. The controller manually supplies a judgment only
after the real native step result, and manually compares structural facts; these
interventions are disclosed. Stub acceptance is not behavioral acceptance.

P0 checks preparation only. The four native step qualification runs, complete
adverse-control admission and numeric supplement remain P1 work. There is no
Dagger/cross-platform validation claim, independent P4 review, candidate success
or production readiness. Model trials: scheduled 6, attempted 0, endpoint failures
0, candidate-producing 0, behavior-accepted 0, measurable 0, useful 0, adopted 0.

## Tool/spec consequence and next gate

Inline completion verification: **VERIFIED**, claim type **TASK**, limited to P0
preparation and freeze. Evidence is the input/evidence hash verification, final
build/nextest output, sealed behavioral controls and retained protocol checks.
There is no scope-equivalent evidence for a FEATURE_GO claim. H-MS1 remains
**not-run** and P1-P4 remain unchecked. No production approval was inferred.

Resume from [task 53 handover](../../../.kiro/task_53_handover.md), verify the
committed receipt and unchanged original, then execute P1 within its bounds.
Missing evidence or an unqualified mandatory control stops live generation.

## COBESY check

The visible outcome is a replayable preparation freeze. Expected disproof and
failed preparations are retained. Facts, conditional interpretation, qualification,
candidate usefulness and adoption are separate; extra pins and parser success
receive no simplification credit.
