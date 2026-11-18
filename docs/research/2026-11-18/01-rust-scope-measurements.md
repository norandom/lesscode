# R135 · Rust reproduces the E5 and E6 scope measurements

Written 18 November 2026, Europe/Berlin. Foreground prototype begun at 07:41. Completed for corrective 2.2 on the four frozen trial arms and two controls. The experiment covers the measurement portion of [requirement 2](../../../.kiro/specs/native-deconvolution-closure/requirements.md). It follows [R134](../2026-11-17/21-audit-of-port-and-reduction-claims.md).

## Decision

Reuse the existing Rust `measures::measure_written` for whole Python scopes. Both historical evaluators called this function’s Python predecessor. The overnight draft targeted a different measurement that counts repeated matching calls within individual files. The foreground prototype exposes both definitions through the Rust CLI.

| Trial and arm | Functions | Distinct scope edges | Write sites | Per-file call occurrences, summed |
| --- | ---: | ---: | ---: | ---: |
| E5 baseline | 69 | 89 | 12 | 62 |
| E5 candidate | 50 | 61 | 11 | 36 |
| E6 baseline | 102 | 145 | 4 | 90 |
| E6 candidate | 63 | 101 | 4 | 66 |

## Hypothesis and controls

The native output must match the historical Python output on the same named files. Compare all edge tuples, all write-site records, all per-file rows and totals. Independently compare the per-file call-occurrence count with the historical density function. Freeze file hashes and expected values before editing Rust.

On two copies of E5's candidate, repeat a matching call in `pod_lifecycle_standing`, then separately add a call to a different declared function. Python and Rust both returned 61 distinct edges and 37 call occurrences for the repeat, and 62 edges and 37 occurrences for the new pair. The unmodified candidate had 61 and 36. These controls exercise source measurement and do not execute the added calls.

## Method and evidence

The detached prototype uses native source at `b08adf1`. The comparison reads the seven E5 files and six E6 files named by the evaluators on both sides. Python reference modules came from the parent of `4f0aed4`. Whole-scope function counts are distinct bare names per file. Edges are distinct caller-file/name and callee-file/name tuples. The frozen write vocabulary matches SQLAlchemy-style text. The ordinary analyzer’s pack-defined write sites and function boundaries are recorded separately.

From the original repository, set `$run` to the directory linked below. These commands reproduce the comparisons.

```powershell
& .runs/e2-facdrone/venv/Scripts/python.exe "$run/replay.py" reference
& .runs/e2-facdrone/venv/Scripts/python.exe "$run/replay.py" native "$run/lesscode-core.exe"
```

The harness invokes `deconvolute measure-scope --root ROOT --file FILE` with a repeated file argument for the full scope, and `deconvolute analyze --lang python --source FILE` for each file. It checks input hashes before native comparisons. The released binary returned exit 3 for the new command. The prototype matched both references across all four trial arms. The first native suite run failed three budget checks. Removing duplicated measurement logic brought the module within its frozen limits and the full Windows suite passed all 402 tests.

The run directory contains the exact commands in `cli.jsonl`, inputs in `reference.json`, comparison results in `native.json`, the source patch, replay harness, frozen Windows executable, and test logs. [Run directory](D:/BuildArtifacts/lesscode/corrective-audit/foreground-20261118) · [Patch](D:/BuildArtifacts/lesscode/corrective-audit/foreground-20261118/2.2-prototype.patch)

## Limits and next decision

The ordinary language-pack analyzer counted 323 → 317 state-write sites for E5 and 302 → 271 for E6. Its function boundaries were 69 → 50 and 103 → 64 respectively. For E6, two definitions share a bare name in one file. The scope measurer deduplicates those names.

Verification used Rust 1.98.1. The pinned Windows nextest 0.9.143 run passed 402 tests. Linux Dagger passed 401 native tests and exported overall exit 0. The cross-file regression ran on both platforms. [Windows result](D:/BuildArtifacts/lesscode/corrective-audit/foreground-20261118/pinned-nextest.log) · [Linux results](D:/BuildArtifacts/lesscode/corrective-audit/foreground-20261118/dagger-evidence)

The prototype establishes structural measurement on these specimens. Static name matching can bind multiple definitions across files. A recorded edge does not establish a runtime call. Production adoption remains an owner decision. The bounded solver, runtime state observation and mutation-sensitivity requirements need independent specimen evidence.

Amendment, 18 November 2026. The call counts are ported into `lesscode-core` and are driven by the language packs for all four languages. The changes this prototype made to `measures.rs`, and its command that printed the `scope-measures-v1` report, were not taken over. [R151](17-pack-driven-call-counts.md)
