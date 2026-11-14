# Implementation Plan: the model seam

Generated 2026-11-12. **Approved by the owner on 2026-11-13** ("we proceed with the model seam"), with
tasks 4.3 and 5.3 added the same day for the indicator the owner asked for (requirements Amendment 2).
Basis: [requirements](requirements.md) (approved 2026-11-12, with Amendment 1) and [design](design.md)
(approved 2026-11-12); prototype gate `proceed`
([R105](../../../docs/research/2026-11-12/12-model-seam-gate-p1.md),
[R106](../../../docs/research/2026-11-12/13-model-seam-gate-p2-p3.md),
[R107](../../../docs/research/2026-11-12/14-model-seam-gate-p4.md)).
Task-graph sanity review: inline (no subagent, by the owner's rule), verdict PASS.

Rules for every task group (owner rules of 2026-11-07, 2026-11-10 and 2026-11-12):

- Foreground execution; one commit per completed task on main; Rust tests with cargo-nextest; format check
  and lints before every push. No Python tooling.
- Endpoints, models and keys come from endpoint files outside the repository (and the repository's `.env`
  for the local endpoint), never from code. No key in a file of the repository, a record or a message.
- Live calls: no two runs against one upstream side by side. The Anthropic endpoint has one call left of
  the three the owner allowed; group 5 uses it. (Used on 2026-11-14 by task 5.1; the owner then approved up to three more for its check, one used.)
- Each group ends with the budgets check passing for every new file at the default ceilings (2,800 tokens,
  300 units, 560 tokens per function), a dated research entry where a result or a deviation exists, and
  `.kiro/task_N_handover.md`. A file over budget is split, not exempted.
- The two items the gate carried forward are tasks 5.2 and 7.3. Task 5.3 can fail as well (a task set
  that does not tell models apart). Either can fail; a failure is recorded and
  goes back to the design before the group is closed.
- Groups 1 to 5 change nothing in `native/`. Group 7 is the only one that changes the core.

## Tasks

- [x] 1. The seam crate and one call
- [x] 1.1 Crate, data of a call, and a call that works against a stub endpoint
  - `model/` as its own crate with its own lock file, the client library pinned; `request.rs` and
    `client.rs` as designed; the endpoint read by the core's `load_endpoint`.
  - `tests/stub_endpoint.rs`: a local stub service scripted per test (from the gate's prototype).
  - Only stated settings go on the wire; text is read from a string and from parts, without reasoning;
    stop reason, tokens and a cut-off reply are reported; the Anthropic style's address gets its version
    segment when absent; a local address works like any other.
  - Done when `tests/call.rs` passes for each of these against the stub and a scan of `model/src` finds no
    model name and no address.
  - _Boundary: CallData, Call_
  - _Requirements: 1.1, 1.2, 1.3, 1.6, 1.8, 2.1, 2.2, 2.3, 2.4, 2.5, 2.8_
- [x] 1.2 A reply in a stated structure
  - A call may ask for a structure; the reply says whether its text has that structure, with or without a
    code fence around it.
  - Done when tests show a met and an unmet structure against the stub.
  - _Boundary: Call_
  - _Requirements: 2.6_
- [x] 1.3 Failures with their reasons, and the key never shown
  - `error.rs`: missing file or entry, unknown style, unreachable, refused with the endpoint's shortened
    message, no answer in time; one line each. `notice.rs`: the terminal notice writes the reason.
  - Done when `tests/failures.rs` passes and a search of every output, record and error text of the test
    run finds no part of the test key.
  - _Boundary: Failures_
  - _Requirements: 1.4, 1.5, 1.7, 4.1, 4.2, 4.3, 4.5_
- [x] 1.4 A record of every call
  - `record.rs`: one line per call to the path the caller names, with reasoning and cached tokens where
    the endpoint reports them; nothing written without a path.
  - Done when tests show a line for a successful and for a failed call, no line without a path, and no key.
  - _Boundary: Record_
  - _Requirements: 9.1, 9.2, 9.3, 9.4_

- [x] 2. Waiting with an estimate
- [x] 2.1 Wait out a refusal, announce each wait, stop at the limit
  - `wait.rs`: which refusals are waited out; the stated wait from a header, else from the message, else
    the first step; the wait made is the larger of that and twice the wait that just failed; the limit of
    ten minutes for refused attempts and pauses, settable. Time is injected.
  - Done when `tests/waiting.rs` passes the five cases of R106 without sleeping, the notice names host and
    estimate before each wait, waits appear in the reply and the record, and the stub never sees two open
    requests of one call.
  - _Boundary: Waiting_
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.6, 3.7, 3.8_
- [x] 2.2 A slow answer: "still running" notices and the answer limit
  - While an accepted call runs longer than expected, a notice at intervals; the answer limit of sixty
    minutes, settable, ends a call with "did not answer in time".
  - Done when tests with injected time show the notices and the limit.
  - _Boundary: Waiting_
  - _Requirements: 3.5, 4.3_

- [x] 3. The repair loop around a caller's check
  - `repair.rs` as designed: a fresh call with the request, the rejected reply and the reasons; the
    request again when no reply arrived or it was cut off; at most two repairs unless the caller says
    otherwise; every attempt recorded; the loop writes no file.
  - Done when `tests/repair.rs` shows acceptance on the first, second and third call, rejection after two
    repairs with all reasons, and that an accepted outcome exists only after a passed verdict; a doc test
    shows the loop cannot be called without a check.
  - _Boundary: RepairLoop_
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7, 7.3, 7.4_

- [x] 4. The two commands of `lesscode-model`
- [x] 4.1 `call`: the process protocol
  - `protocol.rs`, `main.rs`: one request on standard input, one reply on standard output, notices and the
    failure reason on standard error; protocol number checked; exit codes 0, 3 and 2.
  - Done when a test through the binary against the stub shows an answered call, a failed call with its
    reason on standard error and exit code 3, and an unreadable request with exit code 2.
  - _Boundary: ProcessProtocol_
  - _Requirements: 4.4_
- [x] 4.2 `check`: is this endpoint usable
  - `check.rs`: the embedded task, judged by the core's parsing and static checks in a temporary
    directory, with at most two repairs; the report names model, time, tokens and waits.
  - Done when `tests/check_command.rs` shows usable, refused and rejected with exit codes 0, 3 and 4, at
    most three requests at the stub, and no file created in the working directory.
  - _Boundary: CheckCommand_
  - _Depends: 3_
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 7.5_

- [x] 4.3 `measure`: is this model good enough, direct or with the repair loop
  - `measure.rs`: trials over a task set through the repair loop, the counts, cost per accepted answer,
    trials not run; the verdict rule as one function with its two thresholds. `taskset.rs`: the form of a
    task set and its check, and the built-in set of four small modules under `model/tasks/`.
  - `main.rs measure`: the report (task set, check, model, trials, counts with rates, cost, verdict),
    `--quick`, `--trials`, a supplied task set, `--out`, exit codes; trials in a temporary directory.
  - Done when `tests/measure.rs` passes: the rule at and around each threshold, the quick outcomes, no
    verdict when more than a fifth of the trials did not run, an endpoint failure not counted against the
    model, the report and exit codes through the binary against the stub, a supplied set, and no file in
    the working directory.
  - _Boundary: Measure_
  - _Depends: 3, 4.2_
  - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5, 10.6, 10.7, 10.8, 10.9, 10.10, 10.11, 10.12, 10.13, 10.14, 10.15_

- [x] 5. Live: the first cases
- [x] 5.1 The six first cases through `call` and `check`
  - `tests/live.rs` and a run script; each case once through `call` with the turn of R102 and once through
    `check`. Endpoint files outside the repository.
  - Done when a research entry records all six with time, tokens and result, and any case that fails is
    named with the endpoint's words.
  - 2026-11-14: run and recorded in [R108](../../../docs/research/2026-11-14/01-model-seam-first-cases-live.md).
    `call` and `check` pass on all six. `check` on the Anthropic case ran after the owner approved
    further calls (one used; four calls to that endpoint in all).
  - _Requirements: 2.7, 1.6_
- [x] 5.2 One real refusal waited out (carried from the gate)
  - Provoke a rate limit on the local endpoint with a long run of large calls, alone on that upstream;
    one further call through `call` must announce its estimate, wait and be answered.
  - Done when the research entry shows the announced estimate, the wait made and the answer; if no refusal
    can be provoked in one hour of trying, that is recorded and the owner decides.
  - 2026-11-14: met, [R109](../../../docs/research/2026-11-14/02-model-seam-real-refusal-waited-out.md). Ten
    calls of the load were refused, announced a wait, waited and were answered; the small call beside
    them was answered before the limit was met.
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [x] 5.3 The indicator on real models, and whether the built-in set tells them apart
  - `measure` with twenty trials on GLM 5.3 (Mistral's API and the local endpoint), Mistral Large and
    `gpt-6.1-sol`; the quick mode once on each, to compare its outcome with the full verdict. The
    Anthropic endpoint is left out (no calls left for it).
  - Done when a research entry records counts, rates, cost per accepted answer and verdict for each, and
    states whether the set separates the models. If every model passes every first reply, the set does
    not measure anything: it is hardened and measured again before the group closes, and that is recorded.
  - 2026-11-14: measured, [R110](../../../docs/research/2026-11-14/03-model-seam-indicator-on-real-models.md).
    All four cases are "good enough direct" (first replies 19, 17, 20 and 19 of 20); the set does not
    separate the models. The owner decided the same day: the indicator is a floor, not a ranking, and
    the set is kept (design Amendment 9).
  - _Requirements: 10.2, 10.3, 10.9, 10.10_

- [ ] 6. The research tool on the seam
  - `tools/guide-optimizer`: `Live` calls the seam; its own request building, reply reading and wait are
    deleted; its repair turns use `repair.rs`.
  - Done when its tests pass, a dry run gives the same summary as before, and one live judgement of four
    trials on GLM 5.3 completes.
  - _Depends: 3_
  - _Requirements: 8.2_

- [ ] 7. The core on the seam
- [ ] 7.1 The launcher
  - `native/src/seam.rs` from the gate's prototype: find the executable (variable, beside the running
    executable, search path), write the request, read the reply, pass standard error through; a clear
    message when the executable is not installed.
  - Done when tests with a scripted executable show an answered call, a failed call, an unreadable reply
    and the missing executable, and the file has its budget entry.
  - _Boundary: Launcher_
  - _Requirements: 7.2, 8.1_
- [ ] 7.2 `regenerate` through the launcher; the hand-written request leaves the core
  - `default_call_fn` and `cmd_run` use the launcher; `openai_call`, the hand-written `call`, `ureq_post`
    and the `ureq` dependency are removed; one code fence around a reply is removed before parsing; a
    failed turn writes its reason to standard error as well as to the receipt.
  - The tests of the removed transport move to the launcher with a scripted executable.
  - Done when the core's suite passes, `native/Cargo.lock` has no `ureq`, and `regenerator.rs` is below
    its ceiling by at least the size of the removed code.
  - _Depends: 6_
  - _Requirements: 8.1, 8.5_
- [ ] 7.3 A receipt that states only what was sent (carried from the gate)
  - New turns: the call fact has `"mode": "seam-1"` and the settings the executable reported as sent; no
    effort or temperature unless sent. Earlier receipts keep their form.
  - Done when a test reads a new receipt and finds no setting that the scripted executable did not report,
    and the recorded three-turn replay and the admission of stored receipts still pass.
  - _Requirements: 8.3, 8.4, 8.6_
- [ ] 7.4 The core stays free of models, checked
  - CI: the step flow's and the driver's suites run with no `lesscode-model` on the machine; a scan of
    `native/Cargo.lock` for the client library; the scan of `model/src` for model names and addresses.
  - Done when the three checks run in the pipeline and each fails on a planted violation.
  - _Requirements: 7.1, 7.2, 1.8, 2.8_

- [ ] 8. Release and documentation
- [ ] 8.1 Build and test `model/` in the pipeline
  - Format, lints, tests against the stub and the budgets for `model/` on the pipeline's platforms.
  - Done when the pipeline is green with the new jobs.
  - _Requirements: 2.8_
- [ ] 8.2 Release `lesscode-model` for the five targets
  - The release workflow builds, packages and signs the second binary beside the first.
  - Done when a dry run of the workflow on a branch produces both archives for all five targets.
  - _Requirements: 7.2_
- [ ] 8.3 Documentation
  - `.env.sample`, `README.md` and a page under `docs/`: the endpoint file, the check command, waiting and
    its limits, the measuring command and what its three verdicts mean, installing the second binary, what
    `regenerate` says when it is missing. Humanizer pass
    before the push.
  - Done when a reader can configure an endpoint and run the check from the page alone; tried by following
    it on a clean directory.
  - _Requirements: 1.1, 5.1, 3.2_
- [ ] 8.4 Final validation
  - The six first cases through the released form of both binaries; `kiro-cli check`; every requirement
    with its evidence in a validation report; the gate branches deleted.
  - Done when the report and a research entry are committed and the handover file is written.
  - _Depends: 5.1, 7.3, 8.2_
  - _Requirements: 2.7, 8.1, 8.2_
