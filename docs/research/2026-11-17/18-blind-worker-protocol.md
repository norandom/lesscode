# R131 · Blind discovery trial on the mutation-test driver

**Decision.** Keep cross-file discovery deferred. Lesscode 2.0.0 produced per-file receipts on this pre-fix specimen. The local model produced no complete prediction on the full subsystem. The historical failure remains the reference for scoring.

## Question and scoring

Can a model given ordinary pre-fix source predict that the same changed tests yield different results in a reused and a fresh work directory? [R130](17-hidden-state-machine-fitness.md) identified this as the next discovery check. The independent reference is the [recorded failure and fresh-directory comparison](../2026-11-11/19-refused-pinned-accepted.md). The model saw neither report.

The real arm requires a source-backed explanation of both identities. The changed test hash makes the previous stored outcome ineligible. Opening an existing worker can nevertheless execute old test bytes. The model must predict an observable difference between histories. For the control, it must locate the explicit transition table and recognize that separate callers do not scatter the rules. A consequential invented transition or a missed prediction fails the gate. Only a passing map warrants a behavioral replay and a separate simplification trial. [Spec and active tasks](../../../.kiro/specs/model-simplification/task-generation-readiness.md)

## Inputs and method

Executed and written November 17, 2026, Europe/Berlin. The evaluator knew the failure and selected the subsystem. Discovery here means finding the protocol within that selection. Revision `60f79f34880065134c198039923f0fdfd21ea3fd` (MIT) supplied the complete `run_command`, `runner`, `workspace`, `store`, `config` and `process` Rust modules under `native/src/adapters/driver`, plus `native/tests/driver_run.rs`. All seven came directly from Git blobs. The control packet supplied the four Python files of R130's explicit Job machine using basenames only. Both prompts asked the same question and contained no history, fix, later tests, expected answer or other arm's reply. Each model invocation had one turn and no tools.

The first protocol used local Llama 3.1 8B, seed 61, temperature zero, a 32,768-token context and a 900-token output limit. Ollama returned HTTP 500 on both arms. A standalone llama.cpp retry of each prompt exited 1 while allocating the 4 GiB KV cache. Both arms of this first trial were inconclusive. The two frontends used the same weights. Only the standalone logs identify the failed allocation.

Before receiving any prediction, a second protocol kept the two prompts, weights and scoring fixed and changed the runtime. It released the idle Ollama model and used llama.cpp build 12199 (`372dc5ca1`), a 20,480-token context, q8_0 key and value caches, batch size 256 and the same 300-second limit per arm. The original server had counted 18,313 input tokens on the real packet and 283 on the control. Each arm started a new process. Model weight blob SHA-256: `667b0c1932bc6ffc593ed1d03f895bf2dc8dc6df21db3042284a6f4416b06a29`.


## Full-packet result and released analyzer

The second trial's real arm ran from 19:10:20 to 19:12:19 and exited with Windows code 3758096385. Its output and diagnostic files were empty. The control ran from 19:12:19 to 19:12:35, named the policy explicit and located it in `rules.py`, then exited with the same code mid-answer. The cause of these exits is unknown. The short control observation supports the classification alone. There was no complete paired result to score.

For a separate product baseline, the installed Lesscode 2.0.0 binary analyzed the exact seven real files and four control files individually through `deconvolute analyze`. All eleven emitted receipts. The runner reported three gaps and the process module one, both with exit 1. The other nine files returned 0. Receipts describe local sites, writes, functions, gaps and violations. None supplied a cross-file transition map or a prediction for the two work-directory histories. A clean per-file receipt does not establish that worker and store agree.

## Narrower diagnostic

The evaluator selected existing pre-fix functions for a separate diagnostic after the full-packet attempt. The packet contains lines 99–160 and 186–245 of `run_command.rs`, 131–281 and 322–362 of `runner.rs` (clipped at EOF), 116–159 of `workspace.rs`, and the last integration test starting at line 478 of `driver_run.rs`. No expected answer was added. This made locating the interaction easier and changed the claim to interpreting selected evidence. The question and generation budget stayed fixed. The context was 8,192 tokens.

The excerpt run finished at 19:16:12 with exit 0. Its proposed discrepancy was that a `mutant_executed` event fails to update `Planned`. In the supplied source, `run_run` destructures `Planned` before invoking `run_batch`; `one` operates on a job and a workspace. The purported update does not occur in that lifecycle. The reply missed the changed test hash, stale worker bytes and fresh-directory comparison. This is a scored miss on assisted excerpts. The full-packet attempt remains inconclusive. No model-predicted worker history qualified for behavioral replay, and no simplification or release was attempted.

## Replay evidence

Retained under `D:/BuildArtifacts/lesscode/fitness61/` are both protocols, request builders, raw inputs, process logs, the CLI replies and eleven analyzer receipts. The exact UTF-8 full prompt hashes are `D95763D144BB1FA7FF2CA11E9E17A87E3743213CCE9209B53079BB6A49059DFF` and `AC45C30EE8360C60A8505F4F113E36320EC011FF852E5768DD25E3ECF86A5041`. Source payloads matched all seven Git blobs and all four control modules. The excerpt prompt and reply hashes are `3ADA764389052C2CC8A6C4067C67C53FB055CB8C541AF43EB5CF8CDBDD2AF008` and `2FFDB18C7E477C55DA08DEADDFBD80D070970ABE779BD50F4D7A7268FB78F0EA`. The released baseline summary hash is `10DA3E0470D50C591DE957A691882435456C628CC57DF1D05BF0E3E4FD8A5D6D`.

The full discovery question remains open. A future attempt would need a runtime demonstrated to return a complete answer at this input length before spending another held-out case. The scored miss applies to this model, prompt and specimen. The existing checked-refactoring release decision stands.
