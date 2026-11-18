# R142 · Rust traced which test the worker ran

The historical mutation driver noticed a test edit and queued all five mutations for another run. The old worker still ran the old test. The Rust prototype identified the last copy of the file in the worker directory and correctly reconstructed the test bytes seen at **69 of 69 test launches**. The fixed driver copied the edited test before launching it. Preventing that one refresh from running brought back the stale file and the missed fault. [Verified results for all twelve histories](D:/BuildArtifacts/lesscode/corrective-audit/task68/trial/verified.json)

Recorded 18 November 2026, Europe/Berlin. The trial ran from 10:44:31 to 10:47:46. This completed the preregistered extraction experiment on the known failure from R141. The feature remains in discovery. [Preregistration](../../../.kiro/specs/state-consolidation/prototype-gate.md) · [R141 and the historical reproduction](07-real-rust-worker-discovery.md)

## What each worker saw

The test originally checked that the function declaration existed. The edited test checked its branch condition and could detect an inverted condition on line 2. Each cell gives the number of test launches that saw the indicated file and whether the selected fault was caught. “Reused” means the driver returned the stored mutation result. Even with all five results reused, the driver still launched two baseline checks.

| Test history | Mutation results reused | Old driver | Fixed driver | Fixed driver with refresh disabled |
| --- | ---: | --- | --- | --- |
| First run, original test | 0/5 | 7 original, missed | 7 original, missed | 7 original, missed |
| Reopen worker, same test | 5/5 | 2 original, missed (reused) | 2 original, missed (reused) | 2 original, missed (reused) |
| Reopen worker after edit | 0/5 | **7 original, missed** | **7 edited, caught** | **7 original, missed** |
| Create fresh worker after edit | 0/5 | 7 edited, caught | 7 edited, caught | 7 edited, caught |

Before the fix, the last copy into the reopened worker came from the **first run**, at `workspace.rs:97`. Its SHA-256 began with `bf9b2727b746`. With the fix, the last copy came from the **edited run**, at `sync.rs:85`, with SHA-256 `d770a6d43ce3`. Every arm computed the same new test identity after the edit. The difference arose between invalidating the stored result and preparing the directory in which the test ran. [Old trace](D:/BuildArtifacts/lesscode/corrective-audit/task68/trial/before/prediction.json) · [Fixed trace](D:/BuildArtifacts/lesscode/corrective-audit/task68/trial/fixed/prediction.json) · [Disabled-refresh trace](D:/BuildArtifacts/lesscode/corrective-audit/task68/trial/disabled/prediction.json)

## How the result was checked

The experiment used Lesscode’s MIT-licensed mutation driver from `60f79f3` and its historical fix `aa9d7d7`. The control was a disposable copy of the fix with a false guard around the refresh call. The helper and its copy expression remained in the compiled source. All arms used R141’s Python source fixture, genuine toy test executable, single worker and four test histories. The Rust syntax-tree reader scanned the 24 top-level driver modules before the fix and 25 after it. The investigator selected the input, cache, workspace and process operations to observe. The reader located their calls and the filesystem copy calls, recording the original file hashes, byte ranges and quotations before instrumentation. [Frozen inputs and control construction](../../experiments/state-consolidation/worker-trace/replay.ps1) · [Pre-fix source locations](D:/BuildArtifacts/lesscode/corrective-audit/task68/trial/before/overlay/sites.json) · [Fixed source locations](D:/BuildArtifacts/lesscode/corrective-audit/task68/trial/fixed/overlay/sites.json)

| Observation under `native/src/adapters/driver/` | Before the fix | After the fix |
| --- | --- | --- |
| Project test read, aggregate test identity, configuration identity | `source.rs:75`, `run_command.rs:197,198` | Same locations |
| Mutation key and stored outcome lookup | `run_command.rs:139`, `runner.rs:333` | `run_command.rs:139`, `runner.rs:342` |
| Reopen or create the worker | `runner.rs:134,138` | `runner.rs:136,145` |
| Copy the test into a new worker | `workspace.rs:97` | `workspace.rs:102` |
| Refresh an edited test in an existing worker | No copy observed | `sync.rs:85` |
| Pass the mutation work tree to the test runner, then set the process directory | `runner.rs:175`, `process.rs:94` | `runner.rs:184`, `process.rs:94` |

The predictor read only the source inventory and ordered driver events. It recomputed the project test identity, checked the configuration identity used for the mutation keys, and followed successful copies into the directory given to the test command. A separate child process hashed the file relative to its actual working directory immediately before invoking the original test executable. The verifier compared every predicted path and hash with that independent observation. Separate runs of the two frozen, uninstrumented drivers reproduced the same mutation hashes, classifications and reuse decisions across all four histories. [Driver predictor](../../experiments/state-consolidation/worker-trace/predict.rs) · [Child observations after the edit](D:/BuildArtifacts/lesscode/corrective-audit/task68/trial/fixed/children/changed.jsonl) · [Comparison and checks](../../experiments/state-consolidation/worker-trace/verify.ps1)

Two damaged traces tested what happens when evidence is missing. Deleting the initial copy produced a sequence gap. Renumbering the remaining events removed the gap, but left no observed copy into the child-visible path. In both cases the predictor exited with an error and produced no prediction. A second verification of the frozen evidence reproduced the same results file byte for byte. [Sequence diagnostic](D:/BuildArtifacts/lesscode/corrective-audit/task68/trial/missing-event/diagnostic.txt) · [Missing-copy diagnostic](D:/BuildArtifacts/lesscode/corrective-audit/task68/trial/missing-origin/diagnostic.txt) · [Verification repeat](D:/BuildArtifacts/lesscode/corrective-audit/task68/recheck/verified.json)

All four existing research-crate tests passed under cargo-nextest. Rustfmt and Clippy with warnings denied also passed. The 69-launch replay and damaged traces exercised the new command directly on Windows. [Test log](D:/BuildArtifacts/lesscode/corrective-audit/task68/nextest.log) · [Clippy log](D:/BuildArtifacts/lesscode/corrective-audit/task68/clippy.log) · [Format log](D:/BuildArtifacts/lesscode/corrective-audit/task68/fmt.log)

## Replay

From the checkout in PowerShell, with Rust 1.98.1, the existing Cargo target and R141’s frozen inputs, this command rebuilds and instruments the historical drivers, executes all five sets of histories and verifies the predictions. The script checks the hashes of both source archives, both reference drivers and the toy executable. The three instrumented builds have distinct Cargo package names.

```powershell
$manifest = 'docs/experiments/state-consolidation/e5-bound/Cargo.toml'
cargo +1.98.1 build --manifest-path $manifest --locked --offline --bin worker-trace
if ($LASTEXITCODE) { throw 'Could not build worker-trace' }
$tracer = Join-Path $env:CARGO_TARGET_DIR 'debug/worker-trace.exe'
$run = Join-Path 'D:/BuildArtifacts/lesscode/corrective-audit/task68' ([guid]::NewGuid().ToString('N'))
& ./docs/experiments/state-consolidation/worker-trace/replay.ps1 -Tracer $tracer -NewEvidence $run
```

## What this permits next

Rust can now explain which test file a known worker executed using source locations and runtime events. The observer assumes one worker, one tracked file, and no concurrent writes between reading the source, copying it and starting the test. The experiment used a case the investigator had already diagnosed. It called neither a model nor Z3 and measured zero production lines removed. R140 remains the separate evidence for the solver. [R140 bounded solver check](06-rust-job-machine-discovery.md)

The research apparatus adds 633 physical lines of Rust across the extractor, event probes, predictor and command wrapper, plus 180 lines of PowerShell for replay and verification. It reuses the existing Rust parser and SHA-256 helpers. [Experiment sources](../../experiments/state-consolidation/worker-trace/)

The next test should withhold the edited-worker run and its outcome. Give a reviewer the source facts and observations available before the edit, and freeze its prediction for a reopened worker and a fresh worker before revealing the results. A correct forecast on this familiar case would justify a trial on a separately selected case. Independent discovery and production consolidation remain open. [State-consolidation brief](../../../.kiro/specs/state-consolidation/brief.md)
