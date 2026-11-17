# Foreground controller for the frozen H-MS1 prototype

P0 prepares this procedure; P1 must qualify the instrument, observations and native
step baselines before any `drive ... live` command. A process exit of zero from
`drive` means a record was written, not that behavior or simplicity was accepted.
The protocol, execution plan and frozen hashes control every action below.

## P1 admission

1. Verify `freeze.json` against the digest in the committed freeze receipt, then
   verify every input and retained evidence hash. Check the original specimen
   revision/cleanliness, private baseline refs and image availability. Missing or
   changed inputs stop the run; never repair against an infrastructure failure.
2. Use the four predeclared qualification copies and commands in the plan. Clone
   the retained bundle into each new directory, remove its local `origin`, and
   run the unchanged native step checker with four isolated fault copies and
   two pytest workers per copy. Model trials themselves remain serial.
3. Keep a global qualification clock (90 minutes) and a 1,200-second deadline
   for each checker invocation. Interrupt at the bound and retain an incomplete
   record. Count every started target-test/checker invocation before starting it.
4. Review the S1/S2/U/P contracts against both real responsibilities and the
   complete adverse-control matrix. Raw relations are conditional, not proof of
   alias resolution. A boundary that the contracts do not cover is NotMeasurable.
   Do not narrow the approved responsibility to make qualification pass.
5. Run each original twice with matching normalized observations and usable step
   baselines. Reuse P0 evidence only when every relevant frozen byte matches;
   it does not substitute for the named P1 native step checks.
6. Retain a hash-linked supplement containing admitted numeric baseline facts,
   exact integer ceilings, comparison assumptions, control results and the gate
   decision. Inspect the complete manifest/supplement before the first live call.
   No second owner approval is required within the already-approved protocol.

## P2/P3 execution, only after admission

Use the six scheduled paths/commands in `execution-plan.json`. Confirm the
nonsecret endpoint identity and absence of environment overrides. Never copy
credentials into the freeze, requests, candidate repositories or containers.

`drive` writes each request and raw reply, then prints `JUDGE` for a complete
reply. At that point the foreground controller:

1. Saves the exact reply source in that attempt's retained artifacts. Creates a
   new candidate repository from the frozen bundle at the scheduled path, with
   only `main` and no remote. Does not edit the reply or take paths from it.
2. Runs `simplicity-spike boundary PROFILE ORIGINAL REPLY SCOPE`. All four
   arguments are the frozen paths or the one pending reply path from the plan.
   A syntax/scope rejection is retained as a pre-execution candidate rejection;
   no native behavioral verdict is invented for a checker that did not run.
3. Copies the reply to the single fixed case file, commits it in the private
   repository, and verifies every other source/test/declaration byte against the
   frozen source inventory. A source/test/profile/declaration mismatch stops
   execution. An unchanged source may use an empty private commit.
4. Runs the scheduled native `step check` under its deadline. Retains its full
   verdict, logs, revision, hashes and exit code. A missing, interrupted, stale or
   not-judged result is infrastructure failure, never a repairable model failure.
5. Emits exactly one JSON line to the waiting bridge, with `result` equal to
   `accepted`, `refused` or `not_judged` and a `reasons` array. The line is copied
   into the attempt record beside the native verdict or scope-rejection record.
   `accepted` requires the actual unchanged step flow to accept behavior.
6. Extracts the structural facts with the frozen profile, then performs the
   declared complete-responsibility comparison. Retains the facts and manual
   comparison justification even for rejected candidates. Structural usefulness
   cannot override behavior or cause more calls after accepted behavior.

The existing repair loop stops on accepted behavior, refusal, terminal failure
or the two-repair limit. A terminal judge failure never makes another endpoint
call. Replies marked cutoff/refused cannot be accepted from partial text.
Use a global four-hour experiment deadline and the 18-seam-call ceiling; stop
unscheduled/remaining trials explicitly when a global bound is reached. Record
provider request counts and charges when available, otherwise mark unavailable.
Do not confuse the 8,192-token output ceiling with actual usage or an input cap.

No specimen writeback, upstream push, production change, independent P4 review
or design approval is implied. This is an explicitly manual prototype controller,
with the existing seam and step code providing the calls and behavioral checks.
