# R141 · The historical Rust worker failure reproduces

The previous trial showed that Rust could recover a small Python machine and use Z3 to find a planted error. On the real Rust driver, the configured model returned no state machine or prediction. A separate replay reproduced the historical bug and confirmed its fix. The replay used the evaluator’s known history. It supplies a reference for a future discovery method. [R140 and its plain-language interpretation](06-rust-job-machine-discovery.md) · [State-consolidation gate](../../../.kiro/specs/state-consolidation/prototype-gate.md)

## What happened in the driver

The mutation driver makes a working copy of a project, introduces a small fault, and runs the project’s tests on that copy. Suppose an old test misses the fault and a developer strengthens it. The pre-fix driver noticed the changed test and discarded the old result. It then ran the test again in a working copy that still contained the old test file. A fresh working copy ran the strengthened test and caught the very same fault.

We compiled the real CLI from the pre-fix revision `60f79f3` and its fix `aa9d7d7`. Each ran a toy project through four histories with one worker. Five mutations were offered per run. The selected mutation inverted the condition on line 2. The original test checked only for the function declaration. The stronger test checked for the original condition. The project source and driver configuration stayed constant within each arm.

| History | Did the driver reuse the selected result? | Before the fix | After the fix |
| --- | --- | --- | --- |
| First run with the original test | No | Fault missed | Fault missed |
| Same work directory and unchanged test | Yes | Fault missed | Fault missed |
| Same work directory after strengthening the test | No | **Fault missed** | **Fault caught** |
| Fresh work directory with the same strengthened test | No | **Fault caught** | **Fault caught** |

The old and new project test files had SHA-256 prefixes `bf9b2727b746` and `d770a6d43ce3`. Before the fix the reused worker still contained the first hash after the edit. After the fix it contained the second hash, just like a fresh worker. In both revisions the receipt’s test identity changed and the selected mutation ran again. The fix also passed the two historical regression tests selected by `test(a_changed_test_file)` under cargo-nextest. [Pre-fix observations](D:/BuildArtifacts/lesscode/corrective-audit/task67/pre-history/history.json) · [Corrected observations](D:/BuildArtifacts/lesscode/corrective-audit/task67/fixed-verified-history/history.json) · [Regression log](D:/BuildArtifacts/lesscode/corrective-audit/task67/fix-original-regression.log) · [Original R93 developer impact](../2026-11-11/19-refused-pinned-accepted.md)

## Could the tool discover that history?

The hypothesis was that a model reading the complete pre-fix subsystem could identify the persisted entity, trace the guards and execution across files, and predict an observable difference for two histories. Success required a source-backed prediction before comparison with the recorded failure. The evaluator knew the case and selected the same seven pre-fix source and test files used in R131. The requests contained neither the expected answer nor the later fix, regression, research records or Git history. The separate control consisted of the four explicit-table Python modules from R130. Each input went through the same neutral Rust request builder and the released `lesscode-model call`. [Previous attempt R131](../2026-11-17/18-blind-worker-protocol.md) · [Frozen file identities](D:/BuildArtifacts/lesscode/corrective-audit/task67/frozen-inputs.json) · [Research CLI](../../experiments/state-consolidation/protocol-review/main.rs)

| Run | Input tokens | Output and result |
| --- | ---: | --- |
| Unrelated catalogue qualification | 44,934 | Retrieved all three requested lot numbers in order and finished normally. |
| Seven Rust files, preregistered attempt | 18,415 | All 4,000 output tokens went to reasoning. Zero visible characters, no prediction. |
| Explicit-table Python control | 384 | Correctly located the rules in one table and described how `finish` depends on prior `begin`. Its response failed the required report format. |
| Rust output-capacity diagnostic | 18,415 | Same input and endpoint with the cap raised to 16,000. Again all output tokens went to reasoning, with zero visible characters. |

The control’s ten quotations each matched a unique source excerpt. Both history objects omitted the required evidence IDs, and the answer omitted several requested top-level fields. Its prose also called two table values inline literals where the source uses named constants. The strict checker rejected the response. The reviewer recorded those findings before opening the historical fix. [Frozen first responses and scoring](D:/BuildArtifacts/lesscode/corrective-audit/task67/score-before-reference.json) · [Control citation audit](D:/BuildArtifacts/lesscode/corrective-audit/task67/control-citations.json) · [Rust first receipt](D:/BuildArtifacts/lesscode/corrective-audit/task67/freeze-A.json) · [Capacity diagnostic](D:/BuildArtifacts/lesscode/corrective-audit/task67/freeze-capacity.json)

## Timing and reproducibility

Preregistered on 18 November 2026 at 09:55 Europe/Berlin on Lesscode `82c8cdd`. The qualification ran at 09:55:59. The two first responses were frozen by 09:58:14. At 10:00 a separately registered diagnostic changed only the real request’s output cap. It ended at 10:01:45. Scoring was frozen at 10:02:47, before examining the later fix or running the behavioral comparison. The checked replay ran from 10:08 to 10:15. The test project emulates the historical mechanism using the actual Rust driver and its existing `toy_check` executable. The R93 run on OpenUltraCode is the independent historical reference.

The model was `zai-glm-5-3` at the repository’s configured endpoint with high reasoning effort. The model CLI SHA-256 starts with `bc08c4ad2499`. Each call allowed 300 seconds for the answer and 30 seconds of refusal waits. The Rust research package passed four cargo-nextest tests, Rustfmt and Clippy. No production code or specimen rewrite was involved. [Qualification receipt](D:/BuildArtifacts/lesscode/corrective-audit/task67/qualification-repository.status.json) · [Rust tests](D:/BuildArtifacts/lesscode/corrective-audit/task67/nextest.log) · [Artifact hashes](D:/BuildArtifacts/lesscode/corrective-audit/task67/evidence-sha256.json)

Both historical revisions declare the same Cargo package version. On the first fixed build Cargo reused the pre-fix binary from the shared target. The identical binary hash and failing fixed-worker assertion exposed the mismatch. That attempt is excluded. Refreshing timestamps on the extracted fixed sources forced a recompile with the same locked dependencies. The verified executable hashes start with `c9a614d79790` before and `f04fc2719cb3` after. The current checkout’s shared debug CLI was relinked and verified as 2.0.0. [Excluded build](D:/BuildArtifacts/lesscode/corrective-audit/task67/fixed-build.log) · [Verified rebuild](D:/BuildArtifacts/lesscode/corrective-audit/task67/fixed-verified-build.log)

To repeat the behavioral comparison without making a model call, run from the Lesscode checkout in PowerShell. Each invocation requires a new evidence directory.

```powershell
$e = 'D:/BuildArtifacts/lesscode/corrective-audit/task67'
$replay = './docs/experiments/state-consolidation/protocol-review/replay-history.ps1'
$id = [guid]::NewGuid().ToString('N')
& $replay -Driver "$e/pre-driver.exe" -ToyCheck "$e/toy-check.exe" -Revision before -NewEvidence "$e/before-$id"
& $replay -Driver "$e/fixed-verified-driver.exe" -ToyCheck "$e/toy-check.exe" -Revision after -NewEvidence "$e/after-$id"
```

## Decision

**Revise the discovery method.** R140’s bounded Z3 result stands. This trial produced no Rust state model to pass to Z3. Increasing the model’s output allowance fourfold still yielded no visible prediction on the real subsystem. The next proposed experiment is for Rust to extract and cite where inputs are read, where a work tree is created or reopened, and which tree reaches the test process. Check those facts before asking a model to infer a history. Because the evaluator now knows this mechanism, broader discovery will also require a separate case. [State-consolidation brief](../../../.kiro/specs/state-consolidation/brief.md)
