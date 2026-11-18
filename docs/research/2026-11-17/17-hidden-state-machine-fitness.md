# R130 · Fitness of checked refactoring and cross-file machine discovery

**Release decision, 2026-11-17.** Keep the published checked-refactoring workflow. Defer the claim that Lesscode finds or simplifies hidden state machines. The owner defines *scattered state* as an implicit state machine spread across files. The setter result in [R129](16-scattered-state-release-decision.md) remains a useful counterexample to counting assignments. Its duplicated-rule criterion missed protocols in which every transition appears only once.

**Amendment, 2026-11-18.** The deferral above describes the state through 17 November: no tool returned a machine, and the write-site measures could not tell the scattered job machine from the table. On 18 November the picture changed in parts. The prepared job machine was recovered from source ([R140](../2026-11-18/06-rust-job-machine-discovery.md)). On the worker case the source-only model gave no prediction ([R141](../2026-11-18/07-real-rust-worker-discovery.md)), so the test under "What would change the decision" was not met as written. The reply-checking machine of the table below was put in one place and adopted after a bounded comparison over 125 histories ([R154](../2026-11-18/20-reply-machine-one-place.md), [R155](../2026-11-18/21-reply-rule-in-the-loop.md)). A finder returned that machine from the four files ([R156](../2026-11-18/22-find-step-reply-machine.md)), failed on another repository ([R157](../2026-11-18/23-finder-on-mmhmem.md)) and, revised, matches a hand census there as a fit ([R158](../2026-11-18/24-finder-follows-names.md)). The release claim stays deferred: the finder is a research binary, no released command finds a machine, and transfer to a third repository is not tested.

**Working definition.** Name the entity and an event. Show two histories in which the same event has different allowed outcomes. Trace where each relevant state is encoded, where the guard is read, who performs the transition, and what a caller observes. When these pieces are spread across files and a reader has to reconstruct the governing transition rules, the machine is implicit. A shared field can have many users without such a protocol. An explicit transition table can have handlers in many files. Making a machine explicit can be useful even when the number of transitions or writes stays the same.

### What happened in this repository

| Case | Distinguishing history or control | What we established |
| --- | --- | --- |
| Mutation-test worker | Run with a populated worker, change a test, then rerun in the same work directory. A fresh copy must see the new test. A stale copy can rerun under a new cache key with the old test bytes. | R93 recorded a wrong refusal that affected a developer. The outcome key and worker refresh are in different files. Both current regression tests passed: a changed test invalidates stored outcomes **and** reaches the reused copy. This is a historical, human-diagnosed machine with an independently recorded consequence. [Original failure](../2026-11-11/19-refused-pinned-accepted.md) · [Regression tests](../../../native/tests/driver_run.rs) |
| Reply checking | The guide adapter saves a stop reason after a model reply. The repair loop next calls the checker, which consumes that reason. A refusal ends the trial; a cutoff with budget remaining can retry. | The adapter and the loop jointly implement the order. The `Stop` and `Outcome` enums name events and outcomes. Displaying those outcomes in another file is a negative example: rendering does not decide the transition. The released analyzer reported local sites and gaps on four files. It missed both operations on the stop slot and emitted no link between the callbacks. [Adapter](../../../tools/guide-optimizer/src/adapter.rs) · [Loop](../../../model/src/repair.rs) · [Outcome display](../../../model/src/check.rs) |
| Checked step | Run the same guard for the real `one-exit` step at five Lesscode revisions. | Installed 2.0.0 reported an undeclared branch (exit 0), a missing verdict twice (1, 1), matching evidence (0), and a verdict invalidated by a changed test (1). All matched expectations written before execution. This exercised the guard on this repository's history. The machine description came from the reviewers. The earlier checked change reached v2.0.0 through merge `63d456f`. [Check, verdict and adoption context](../2026-11-11/18-three-real-steps.md) · [Guard](../../../native/src/step/guard.rs) |

### Could the proposed measure tell the difference?

The controlled pair uses one runtime job with three states. Starting an idle job enters running. Finishing succeeds only when running. In one version two files hard-code the edges. In the other a single table owns them; the handlers still live in different files. An independent oracle checked both versions on all 31 start/finish histories of length zero through four. A deliberately wrong guard was detected (63 pytest cases passed).

The frozen revision-four spike counted three lexical field targets, three writes and zero writer dispersion in **both** versions. They are three lexical owners of one runtime field. The explicit table added three module bindings to the overall write count. The current state measurements fail this discrimination test. They do not return a transition model. This is a synthetic counterexample to the metric, and the four-file Rust analysis above is a separate test of the released analyzer. Neither was a model-proposal trial.

### Method and evidence

Exploratory fitness review, written and run 2026-11-17, approximately 18:57–19:01 Europe/Berlin, on Lesscode `88d92d5`. Three agents covered measurement, real code and release fitness. The local regression tests used cargo-nextest. The first invocation lacked the repository's `toy_check` test executable; both tests passed after building that example. Eight other tests in that binary were outside the filter. The guard used an empty bare repository backed by Lesscode's Git objects so files in today's checkout could not supply a historical verdict. This replay checks the five recorded revisions, not all override and error paths.

Raw evidence is under `D:\BuildArtifacts\lesscode\`. SHA-256 identifies each record:

| File | SHA-256 |
| --- | --- |
| `fitness60-guard/result.json` (version, revisions, predictions and outputs) | `6EC26A93B7FF63D163B9AD643724CE0B48429BCCE94F01DD5BC8E9B8C5EF597D` |
| `fitness60-guard/worker-verified.log` | `708C5B5F32D19C7D0FC35D3B6CD6A5FF4B04773134015F41D9131A5EF9BA09B3` |
| `fitness60-local/manifest.json` (four source/receipt pairs) | `0E5414ACD173984459F3E167DBBD41CFC0A2FAD1D70C8DABD432CF6EBA004FAC` |
| `fitness60-concept/result.json` (freeze, control sources and replay script) | `52C9FE089D40AE6A6B60F9F53E8A90541A2F4EA1B64BAF22D1FFFD1CDB05604F` |

### What would change the decision?

Give a future prototype the ordinary source and tests from Lesscode `60f79f3`, the parent of the worker fix. Withhold the fix, later regression, Git history, research notes and expected transition map. Ask it to identify the logical entity, reconstruct the source-backed transitions across files and predict an observable difference between two histories. Check its answer against the recorded worker failure and exercise the predicted paths. An explicit machine with distributed callers must escape a false alarm. A miss, invented transition, or unaccounted effect keeps the discovery claim deferred.

If discovery succeeds, separately try one proposed simplification on this repository. A maintainer should be able to identify where to change a transition after the edit; the normal and failure histories must still pass the existing behavioral checks. A correctly reconstructed machine alone does not show that a rewrite helps. [Active tasks](../../../.kiro/specs/model-simplification/task-generation-readiness.md) · [Original stopped experiment](15-model-simplification-checkout-preflight.md)
