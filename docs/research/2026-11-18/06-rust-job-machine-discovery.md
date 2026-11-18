# R140 · Rust recovers the scattered job machine

Given only the three modules of the scattered R130 package, the Rust research CLI recovered `job.Job.phase`, its initial value and both guarded transitions. It traced the two inherited methods to separate files. Given the four modules of the explicit package in a separate run, it traced both rules to `rules.TRANSITIONS`. The handlers still live in separate files in that package. [Scattered source report](D:/BuildArtifacts/lesscode/corrective-audit/task66/verified/extract-implicit.stdout) · [Table source report](D:/BuildArtifacts/lesscode/corrective-audit/task66/verified/extract-explicit.stdout)

| Event | Accepts from | Sets state to | Guard and write in the scattered package |
| --- | --- | --- | --- |
| `begin` | `idle` | `running` | `start.py` lines 3 and 5 |
| `finish` | `running` | `done` | `finish.py` lines 3 and 5 |

The initializer sets `idle` at `job.py` line 6. An accepted event returns true. A refused event returns false and preserves the current state. For example, `finish` is refused on a fresh job and accepted after `begin`. Each report cites the entity, base imports, method, guard, write, return statements, and the value references used to determine each transition. The replay checks each quoted source fragment against its file and line.

## Bounded behavior and counterexample

Z3 found no difference in the initial state, event acceptance or state after any prefix of the 31 clean histories. The planted change in a copy makes `finish` accept an idle job. The shortest solver witness was one event, `finish`. Rust predicted refusal in the scattered machine and acceptance in the planted table. Three fresh invocations of the frozen specimen test confirmed the returned Boolean. The scattered and clean table cases passed. The planted case failed with `assert True is False`. [Clean solver receipt](D:/BuildArtifacts/lesscode/corrective-audit/task66/verified/clean-query/result.json) · [Planted solver receipt](D:/BuildArtifacts/lesscode/corrective-audit/task66/verified/planted-query/result.json) · [Witness query](D:/BuildArtifacts/lesscode/corrective-audit/task66/verified/planted-query/shortest.smt2) · [Specimen failure](D:/BuildArtifacts/lesscode/corrective-audit/task66/verified/planted-witness.stdout)

| Exact history length | Histories | Clean model differences | Planted model differences |
| ---: | ---: | ---: | ---: |
| 0 | 1 | 0 | 0 |
| 1 | 2 | 0 | 1 |
| 2 | 4 | 0 | 3 |
| 3 | 8 | 0 | 7 |
| 4 | 16 | 0 | 15 |

For each length Rust generated an SMT-LIB comparison and cross-checked SAT or UNSAT by enumerating the two extracted models. The counts in the table come from those enumerations. The separately executed R130 oracle suite passed all 63 cases, with zero skips. It checks both real Python versions on all 31 histories and includes its own wrong-guard control. [All clean modeled histories](D:/BuildArtifacts/lesscode/corrective-audit/task66/verified/clean-query/histories.json) · [Specimen JUnit](D:/BuildArtifacts/lesscode/corrective-audit/task66/verified/suite.xml)

## Experiment and controls

Preregistered at 09:35 Europe/Berlin on 18 November 2026, against Lesscode `2bd419b`. The first calibration ran at 09:41. Inline review added the event method declaration to the evidence, a receiver-shadowing check, source-citation verification and a separate baseline witness invocation. The complete final replay ran from 09:44:29 to 09:44:36. The criteria and bounds stayed fixed. The hypothesis was that native syntax-tree analysis could connect the receiver’s initial assignment to its inherited guards and writes, a relationship the earlier per-file measurements missed. A missing edge, incorrect source location, false scattered classification on the table, or failure to expose the planted guard would reject the result. [Preregistration](../../../.kiro/specs/state-consolidation/prototype-gate.md) · [R130 specimen and previous measurement](../2026-11-17/17-hidden-state-machine-fitness.md) · [R139 solver predecessor](05-e5-rust-z3-check.md)

The tool receives one package path for discovery. It uses the native Python parser to resolve direct relative imports, mixin bases, scalar constants and literal two-slot table entries. It recognizes an initializer and straight-line handlers with a single refusal guard, field assignment and Boolean returns. The author knew the fixture shape. Entity names, states, field, event names and filenames come from the input. A renamed copy yielded `parcel.Parcel.stage` with `admit` and `close`. An extra side effect and a missing base module each produced a diagnostic and exit 1. No model call or new Python adapter was required. [Renamed report](D:/BuildArtifacts/lesscode/corrective-audit/task66/verified/renamed.stdout) · [Effect rejection](D:/BuildArtifacts/lesscode/corrective-audit/task66/verified/extra-effect.stderr) · [Import rejection](D:/BuildArtifacts/lesscode/corrective-audit/task66/verified/missing-base.stderr)

The trial used the frozen R130 sources and unmodified pytest oracle, Rust 1.98.1 and the pinned external Z3 5.3.3. All three tests in the research package passed under cargo-nextest, including the previous E5 model-parser regression. Rustfmt and Clippy passed. A further end-to-end replay finished at 09:48:14 after the PowerShell script was updated to restore its caller’s environment. The new apparatus contains 826 physical lines of Rust including tests, plus 136 lines of PowerShell for replay and independent checks. No specimen lines were removed. [Frozen input identities](D:/BuildArtifacts/lesscode/corrective-audit/task66/frozen-r130.json) · [Build identities](D:/BuildArtifacts/lesscode/corrective-audit/task66/source-sha256.json) · [Evidence identities](D:/BuildArtifacts/lesscode/corrective-audit/task66/evidence-sha256.json) · [Rust tests](D:/BuildArtifacts/lesscode/corrective-audit/task66/verified-nextest.log)

## Replay

From the Lesscode checkout on the Windows research workstation, run the following in PowerShell. The first command builds the research binary. Calling it with one package prints the discovery report. The replay script runs both packages, the controls, Z3 and the specimen tests into a new directory.

```powershell
$manifest = 'docs/experiments/state-consolidation/e5-bound/Cargo.toml'
cargo +1.98.1 build --offline --locked --manifest-path $manifest --bin discover-machine
if ($LASTEXITCODE) { throw 'Build failed' }
$exe = Join-Path $env:CARGO_TARGET_DIR 'debug/discover-machine.exe'
& $exe 'D:/BuildArtifacts/lesscode/fitness60-concept/implicit'
$out = Join-Path 'D:/BuildArtifacts/lesscode/corrective-audit/task66' ('replay-' + [guid]::NewGuid().ToString('N'))
& ./docs/experiments/state-consolidation/machine-discovery/replay.ps1 -Binary $exe -NewEvidence $out
```

## Decision

The synthetic discovery check passed. This recognizer handles a narrow Python pattern. The result supplies a calibration for the next experiment on Lesscode’s historical Rust stale-worker case. That experiment must derive a source-backed machine and distinguishing history from pre-fix code, then check them against independently recorded behavior. General discovery, a proposed consolidation, compression and adoption remain open. [State-consolidation brief](../../../.kiro/specs/state-consolidation/brief.md)
