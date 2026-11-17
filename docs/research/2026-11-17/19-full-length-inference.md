# R132 · Full-length inference on an unrelated catalogue

**Decision.** The local server failed qualification. It processed part of the catalogue and returned no answer. Keep the next held-out discovery case unused.

## Protocol

Written and executed November 17, 2026, Europe/Berlin. [R131](18-blind-worker-protocol.md) left a runtime question open after two incomplete full-packet attempts. The [model simplification spec](../../../.kiro/specs/model-simplification/task-generation-readiness.md) keeps discovery deferred.

The qualification requires a complete local chat response to an unrelated prompt at least as long as the 18,313-token R131 input. The request must finish within 300 seconds, report a normal stop, and return the correct three lot numbers in order. A runtime failure, timeout, truncated response, out-of-range token count or wrong value fails this qualification.

The generator used Python's random seed 62 for six-digit lot numbers and numbered the shelves consecutively. It used the model tokenizer to select 1,661 rows and requested the values for shelves 0001, 0830 and 1659. The question appeared before and after the catalogue. These lookups sample the beginning, middle and end of the prompt. The script allowed 900 output tokens and required the reported prompt length to leave that much room in the context.

The run used llama.cpp server build 12199 (`372dc5ca1`) on loopback, one slot, CPU inference with 12 threads, a 20,480-token context, q8_0 key and value caches, flash attention, batch size 256, seed 61 and temperature zero. The server used the same Llama 3.1 8B weight blob as R131, verified by SHA-256 `667b0c1932bc6ffc593ed1d03f895bf2dc8dc6df21db3042284a6f4416b06a29`. The retained request JSON has SHA-256 `57a16a3f77bace5e6e9460a6f92b92c6a4b7fe781b1829b398706081cb42c16f`.
## Observation and limits

The run started at 20:34:19 CET. Inference began at 20:34:27 and the client recorded Windows connection reset 10054 at 20:35:43, about 76 seconds later. The last server progress line showed 5,376 tokens processed (29%). The expected values were 281431, 775291 and 768503. There was no response to score, no server-reported final token usage and no finish reason. The process poll had no exit code immediately before harness cleanup. The cause of the reset remains unknown.

The retained files under `D:/BuildArtifacts/lesscode/fitness62/` are the generator and runner (`run.py`), frozen request, raw server log and result JSON. Replaying the runner requires the same installed weights and llama-server. Use a copy in a fresh evidence directory to preserve this run's output. A subsequent runtime change can be tested against this same catalogue and acceptance criteria. This trial supplies no evidence about cross-file discovery, simplification or release readiness.
