# R143 · The reviewer predicted which worker would miss the fault

Given the Rust source and two runs **before** a test edit, the reviewer correctly predicted what each historical driver would do **after** the edit. The old reopened worker would discard the stored result, rerun the mutation with the original test, and miss the fault. The fixed reopened worker would also discard the stored result, but would copy the edited test and catch the fault. Both fresh workers would catch it. All 12 predictions matched the withheld evidence. This qualifies a trial on another case. [First score](D:/BuildArtifacts/lesscode/corrective-audit/task69/score.json) · [Preregistered gate](../../../.kiro/specs/state-consolidation/prototype-gate.md)

| Driver and work directory | Test file predicted and observed | Stored mutation result reused? | Selected fault predicted and observed |
| --- | --- | --- | --- |
| Original driver, reopened | Original | No | Missed |
| Original driver, fresh | Edited | No | Caught |
| Fixed driver, reopened | Edited | No | Caught |
| Fixed driver, fresh | Edited | No | Caught |

These are three values for each of four scenarios on one fault. The historical driver receipt established whether the selected mutation ran again and whether it survived. Seven independent child observations per scenario established which file the commands saw. The two histories that distinguish the implementations are the reopened runs. [R142 traced executions](08-native-worker-byte-trace.md)

## What the reviewer knew

The Rust research command built one request per revision of Lesscode’s MIT-licensed mutation driver, `60f79f3` and `aa9d7d7`. Each request included the control flow for obtaining a worker, source expressions connecting test identity to the result store and process directory, and a summary of two observed runs. These summaries were identical across revisions. The first run created a worker and ran five mutations. The unchanged run reopened it, reused five stored results and ran two baseline checks. No package names identifying the revision, changed-run traces, child hashes, mutation outcomes or prior answers were supplied. Rust checked the selected functions against R142’s original source hashes. The input directory held the source-site inventory and only the first two event logs. [Packet builder](../../experiments/state-consolidation/worker-trace/forecast.rs) · [Fixed packet](D:/BuildArtifacts/lesscode/corrective-audit/task69/trial/X/request.json) · [Original packet](D:/BuildArtifacts/lesscode/corrective-audit/task69/trial/Y/request.json)

The question supplied the proposed edit, the test command's rule, and the effect of the selected mutation. The original test looked for `def step(`. The edited test looked for `if a > 3:`. The selected mutation retained the first string and removed the second. The disputed fact was which file the child would see when it ran in an existing worker. The fixed revision served as a control against simply predicting a stale file whenever a worker was reused.

## Sealing and scoring

The two request hashes were saved at 11:08:51 Europe/Berlin on 18 November 2026. The same released `lesscode-model call` sent each in a separate request to R141’s qualified endpoint. The model was `zai-glm-5-3` with high reasoning effort, 8,000 output tokens per request, a 300-second answer limit and up to 30 seconds of refusal waits. Each received one attempt and finished normally. The fixed answer used 2,065 input and 1,726 output tokens. The original used 1,766 input and 2,709 output tokens. The calls ran from 11:09:57 to 11:10:24. Both answers were sealed before the scorer opened the held-out files at scoring time. The first score was recorded at 11:11:03. [Frozen packet hashes](D:/BuildArtifacts/lesscode/corrective-audit/task69/trial/packets.json) · [Fixed answer seal](D:/BuildArtifacts/lesscode/corrective-audit/task69/trial/X/seal.json) · [Original answer seal](D:/BuildArtifacts/lesscode/corrective-audit/task69/trial/Y/seal.json) · [Protocol hashes before the calls](D:/BuildArtifacts/lesscode/corrective-audit/task69/protocol-before-calls.json)

Inline review checked the explanations as well as the fields. Both answers traced the edited test digest through `run_command::identity_with_tests` to a different outcome key and located the child’s current directory in `process::run`. The original answer correctly explained that `workspace::open` recovers a pending mutation and leaves the existing test file in place in this history. The fixed answer identified `sync::refresh` as the code that copies the edited file before the test. Both explained why creating a worker copies the edited project. [Original response](D:/BuildArtifacts/lesscode/corrective-audit/task69/trial/Y/answer.json) · [Fixed response](D:/BuildArtifacts/lesscode/corrective-audit/task69/trial/X/answer.json)

The grader rejected all twelve individual inversions of the reference fields. A separate check placed invalid files named for the withheld histories beside the allowed input. The resulting request was byte-identical. Changing a selected source file prevented request generation. After the first score, the scoring script was updated to report the counted controls and return a failing exit code for an incorrect answer. Rechecking the sealed pair reproduced all twelve matches. Changing one Boolean to the string `"false"` in a disposable answer caused exit 2. The original scorer, score and answers are retained. The research crate passed all four existing nextest tests, formatting and Clippy. The existing R142 prediction command also reproduced its original pre-fix report byte for byte. [Packet controls](D:/BuildArtifacts/lesscode/corrective-audit/task69/packet-controls/results.json) · [Repeated score](D:/BuildArtifacts/lesscode/corrective-audit/task69/recheck.json) · [Malformed-answer control](D:/BuildArtifacts/lesscode/corrective-audit/task69/scorer-control/score.json) · [Test log](D:/BuildArtifacts/lesscode/corrective-audit/task69/nextest.log)

## Replay without a model call

From the checkout in PowerShell, run the scorer on the two frozen answers. It checks both seals, verifies the reference files against R142’s evidence index and writes a fresh score. The explanations remain available for human review.

```powershell
$e = 'D:/BuildArtifacts/lesscode/corrective-audit/task69'
$score = Join-Path $e ("score-" + [guid]::NewGuid().ToString('N') + ".json")
& ./docs/experiments/state-consolidation/worker-trace/score-forecast.ps1 -Run "$e/trial" -NewScore $score
```

To repeat the live experiment, use `run-forecast.ps1 -Stage prepare -Tracer "$e/forecast.exe" -Run $newDirectory` with a fresh directory under `D:/BuildArtifacts`, inspect both requests, then run the same script with `-Stage call -Run $newDirectory` from the repository root. Each run permits one attempt per packet. [Experiment runner](../../experiments/state-consolidation/worker-trace/run-forecast.ps1)

## Decision and limits

R141’s seven-file source-only request yielded no visible forecast. R143 produced correct forecasts on shorter, investigator-selected packets. Several things changed together, including the excerpts, question, early observations and output allowance. This trial cannot isolate which change helped. The investigator already knew which functions mattered and supplied the fault and edit. The model saw neither withheld execution. Its success measures forecasting on one known mechanism. [R141 source-only trial](07-real-rust-worker-discovery.md)

The next gate is a separately selected real stateful case. Freeze the selection rule, input boundary, competing predictions and oracle before reading its behavioral answer. Test whether Rust can locate relevant evidence without a list of worker-specific functions. Z3 was used in R140 for bounded equivalence. This trial used the model and historical process observations. Production lines removed were zero. [R140 solver result](06-rust-job-machine-discovery.md) · [State-consolidation brief](../../../.kiro/specs/state-consolidation/brief.md)
