# R139 · Rust and Z3 check a changed E5 function

Rust checked E5’s `signal_standing` against its baseline. Both returned identical reports on all 81 inputs in the declared domain. With one incorrect row lookup planted in the candidate, Z3 found a counterexample. Replaying that input against the real functions displayed a stale reading only in the planted version. This completes [corrective task 2.3](../../../.kiro/corrective-tasks.md).

Preregistered on 18 November 2026 at 09:13 Europe/Berlin. The first solver run finished at 09:18:07. The final controller ran from 09:21:43 to 09:21:47. Written the same morning. The hypothesis follows the finding in R134 that the historical checks ran evaluator-local examples. R138 tested a different property, the sensitivity of the E5 specimen’s tests to selected faults. [Frozen gate](../../../.kiro/specs/state-consolidation/prototype-gate.md) · [R134](../2026-11-17/21-audit-of-port-and-reduction-claims.md) · [R138](04-e5-e6-specimen-faults.md)

## What was checked

E5 folded the baseline helper `_latest_basis` into `signal_standing`. The baseline comprehension keeps a row when its digest and revision match the newest row. The candidate loop skips a row when that pair differs. Rust parsed the actual Python syntax trees, checked the baseline helper call and translated the inclusion filter at baseline line 70 and the exclusion filter at candidate line 77. The planted copy changes a single receiver in the exclusion filter, reading the revision from `newest` instead of `r`.

Every input has two ordered readings, one for the traded-horizon monitor on 5 September and one for the survival monitor on 8 September. Each digest ranges from `d0` to `d2` and each revision from `r0` to `r2`, independently. Each reading is within its boundary. The standing is unrecorded, the baseline and flag are absent, the clock is noon UTC on 17 September 2026 and the overdue interval is 14 days. The check would fail on a clean report difference, a mismatch between a modeled window and the real output, a missed planted difference, or a witness that failed fresh replay.

Rust emitted QF_LIA queries asking whether the two predicates for keeping the older window could differ. It called the external Z3 executable before invoking any specimen function. After SAT, Rust reran the same query with `get-value` and passed the returned assignment to three new CPython processes. Separately, three batch executions evaluated all 81 assignments, checking both window membership and the complete JSON return values. Python only constructs the target types, calls the real function and serializes its observations and import identities. The Rust controller verifies the module, kernel, runtime and solver hashes. [Rust and counted adapter](../../experiments/state-consolidation/e5-bound/) · [All inputs](D:/BuildArtifacts/lesscode/corrective-audit/task65/verified/all-inputs.json)

| Comparison | Z3 verdict on older-window membership | Complete reports that differed |
| --- | --- | ---: |
| Baseline vs. E5 candidate | UNSAT | 0 / 81 |
| Baseline vs. E5 with the planted revision error | SAT | 18 / 81 |

For witness 9 the older row had `d0, r1` and the newest had `d0, r0`. Both clean reports contained only the survival reading of 8 September. The planted report also displayed the traded-horizon reading of 5 September as “Within its boundary.” Its revision had already been superseded. The 18 differences comprise every case in which the digest matched and the revision differed. All 243 observed window lists agreed with the corresponding modeled membership. [Clean query](D:/BuildArtifacts/lesscode/corrective-audit/task65/verified/clean.smt2) · [Planted query](D:/BuildArtifacts/lesscode/corrective-audit/task65/verified/fault.smt2) · [Solver model](D:/BuildArtifacts/lesscode/corrective-audit/task65/verified/model.txt) · [Replayed baseline](D:/BuildArtifacts/lesscode/corrective-audit/task65/verified/fresh-baseline.json) · [Replayed candidate](D:/BuildArtifacts/lesscode/corrective-audit/task65/verified/fresh-candidate.json) · [Replayed fault](D:/BuildArtifacts/lesscode/corrective-audit/task65/verified/fresh-planted.json)

## Checks and replay

The controller refused an unmodeled row field. Its cargo-nextest regression test rejected duplicate, missing and out-of-domain model bindings and an unknown verdict. Rustfmt and Clippy passed. A separate receipt check reconstructed all 81 distinct inputs, the 18 report differences and the three fresh outputs. The specimen’s existing lifecycle projection tests passed on each clean tree, eight per arm, with zero skips. The first concurrent pytest attempt exhausted Windows paging capacity before yielding a valid pair of results. The successful retry ran one suite at a time with two xdist workers and a single BLAS thread. The initial Rust build also required correcting an array-index expression passed to the JSON macro. No experiment bounds changed. [Rust test log](D:/BuildArtifacts/lesscode/corrective-audit/task65/nextest.log) · [Receipt check](D:/BuildArtifacts/lesscode/corrective-audit/task65/verification.txt) · [Baseline JUnit](D:/BuildArtifacts/lesscode/corrective-audit/task65/retry-baseline.xml) · [Candidate JUnit](D:/BuildArtifacts/lesscode/corrective-audit/task65/retry-candidate.xml)

The research crate uses the parser, bounded process runner and SHA-256 implementation from native Lesscode at `a6b5e22`. Its lock uses the same dependency versions as the native lock. Rust 1.98.1 drove Z3 5.3.3 (build `430d19e697e2ec9bad79ff210d41b4a7a5fcfa3e`) and the specimen’s CPython 3.13.16. The Rust apparatus has 603 physical lines including its model-parser test. The invocation adapter has 41 physical lines. The module copies and full observations are kept on the workstation with the frozen inputs. The specimen archive has no top-level license file. [Build inputs](D:/BuildArtifacts/lesscode/corrective-audit/task65/source-sha256.json) · [Evidence hashes](D:/BuildArtifacts/lesscode/corrective-audit/task65/evidence-sha256.json) · [Verified receipt](D:/BuildArtifacts/lesscode/corrective-audit/task65/verified/result.json)

From the Lesscode repository in PowerShell, use the existing E2/E5 specimen trees and the pinned tools named in the gate. Each run requires a new output directory under the existing evidence parent.

```powershell
$manifest = 'docs/experiments/state-consolidation/e5-bound/Cargo.toml'
$evidence = 'D:/BuildArtifacts/lesscode/corrective-audit/task65'
$out = Join-Path $evidence ('replay-' + [guid]::NewGuid().ToString('N'))
cargo +1.98.1 run --offline --locked --manifest-path $manifest -- . $out
if ($LASTEXITCODE) { throw 'E5 bounded check failed' }
Get-Content (Join-Path $out 'result.json')
```

## Spec consequence

The approved prototype runtime worked for this manually selected filter. Z3 checked row retention symbolically. Exhaustive specimen execution established whole-report agreement on the stated domain. Missing row fields, other dispositions, repeated monitors, other sequence lengths and other standing or flag values are outside that domain. The apparatus consumed an existing rewrite. It discovered no state machine and generated no compression measurement. The next feature gate is whether the tool can locate the prepared scattered job machine from source before proposing a consolidation. [State-consolidation brief](../../../.kiro/specs/state-consolidation/brief.md)

Amendment, 18 November 2026. The general part of this check is ported into `lesscode-core` as `deconvolute bound-check`. The experiment crate gained two modes, `--target` and `--windows`; the original run of this entry is unchanged. [R152](18-bound-check-in-core.md)
