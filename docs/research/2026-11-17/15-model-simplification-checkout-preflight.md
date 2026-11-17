# R128: Checkout byte verification stops S2 before a behavioral verdict

- Written and executed: 2026-11-17, Europe/Berlin.
- Execution: S2 call began 18:10:48; native check 18:12:17–18:28:18.
- Status: stopped; controller preflight failure, not model or runtime mutation failure.
- Predecessor: [R127](14-model-simplification-boundary-correction.md).
- Spec: [prototype gate](../../../.kiro/specs/model-simplification/prototype-gate.md).
- Type: frozen trial followed by exploratory diagnosis; no production approval.

## Decision and falsification

The independent reviewer found three candidate-checkout observation pins whose
bytes differed from the freeze. The foreground controller had compared the clone
before and after installing the reply, rather than against the frozen inventory.
That check could prove that the reply changed only one file, but could not prove
that the initial checkout was correct. This is the controller's error.

The native check was stopped immediately at 18:28:18.7547076, before its
18:32:17.4566615 deadline. S2 received `not_judged` and ended incomplete after
one proposal, with no repair. Under the existing global-stop rule, S3, B1, B2 and
B3 remain not attempted. P4 review approves this disposition, not model efficacy.

## Root cause and evidence limits

System Git configuration has `core.autocrlf=true`. Cloning before setting the
candidate's local value to false translated three LF pin files to CRLF. Their
timestamps precede checker start. Their normalized text and committed Git blobs
match the freeze; their candidate working-tree bytes do not. The retained
`native-start.json` claim of 203 verified other files means before/after-clone
equality only and must not be read as frozen-inventory verification.

The distinction matters: all three pins in each actual native base/step tree
match the frozen bytes. The reviewer also verified all 24 pin hashes across the
four earlier qualification checks and their two sides. Earlier native observations
remain evidence; no runtime test tampering or original-specimen mutation was found.
The interrupted candidate check nevertheless supplies no behavioral verdict.

## Candidate and observations

The unedited S2 reply extracts a private setter. Its boundary check passes with
revision four. Independent complete-envelope comparison records target W/S/D
4/3/1 to 2/1/1, envelope W 10 to 8 and S 3 to 1, with B/P/E/L/D unchanged.
U rises 82 to 85 because three helper calls lack an admitted effect contract.
The result is **NotMeasurable**, not useful simplification; raw write reduction
does not establish less maintained behavior or compensate for unresolved effects.

The fresh native base side completed 64 mutants: 53 killed, 11 survived and four
passing closing baselines. The candidate side retained 32 partial outcomes,
27 killed and five survived. No complete candidate population, closing baseline
set, accepted comparison or behavioral rejection is claimed.

The call completed in 41.90 seconds: 16,826 input, 5,460 output and 1,550 reported
reasoning tokens. Across S1/S2: 33,652 input and 10,536 output tokens, with 2,713
reported reasoning tokens; reasoning is not assumed additional to output.
Provider charges are unavailable. Neither proposal was manually edited.

## Accounting and retained evidence

Total: 48/80 outer checker starts, 2/18 calls, 2/6 trials attempted and producing
candidates; two incomplete; zero endpoint failures, behavior-accepted, measurable,
useful or adopted outcomes. Only state was attempted; branch efficacy is untested.
Apparatus remains 1,358/1,400 lines and fixtures 357/600. No clock or counter refund.

The [result receipt](../../experiments/model-simplification/group-58-result-receipt.json)
links the retained index and review beneath
`D:/BuildArtifacts/lesscode/model-simplification/ms1/`. Exact requests/replies,
candidate bytes, facts, comparison, private commit, pin identities, partial native
results, stop process tree and judge result remain available. The four containers
identified in that native process tree were removed; other projects were untouched.
Original specimen remains tracked-clean at its frozen revision.

## Concrete next decision

A future bounded continuation must first authorize the changed stop disposition.
Prepare checkouts with `git -c core.autocrlf=false clone` before materialization,
then verify every protected working file against the frozen input or the explicitly
frozen case-declaration blob, not against an unqualified checkout. Test that an
LF-to-CRLF pin change is rejected before any target or model invocation. Cover both
S and B, retain the checks, and audit all other frozen hashes.

This proposal changes checkout verification, not the evaluator, model prompt or
candidate source. It has not been executed. S1/S2 remain terminal; only the four
unattempted trials could be considered, without assuming new clocks or budgets.
The original qualification deadline has passed; candidate time still ends at
21:49:21.7470226. Any continuation decision must address that explicitly. No
automatic restart, fifth apparatus revision, production plan or adoption follows.
