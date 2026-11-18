# Task 75 handover · Two ports into core and the first real state-machine case

Written 18 November 2026, 15:00 Europe/Berlin. The port group is half done: 2.2 and 2.3 are in `lesscode-core`. On the owner's direction of the afternoon the work then turned to hidden state machines on real code. State-machine discovery (R140) and the Java lowering (R146, R147) are not ported.

## Done

**2.2, call counts** (`fec7d8d`, record `17a4d44`). Each language pack has a `calls` entry. `deconvolute analyze` reports `call_occurrences`; the new `deconvolute measure-scope` gives the call edges of a scope, for all four languages. Python equals R135's frozen reference on the four E5 and E6 arms and both controls, per file and as edge lists. [R151](../docs/research/2026-11-18/17-pack-driven-call-counts.md) · [Evidence](D:/BuildArtifacts/lesscode/corrective-audit/task75/2.2)

**2.3, bounded solver check** (`c1c76e8`, record `1eb106f`). `deconvolute bound-check --target T --out D` takes symbols with ranges and, per version, an SMT-LIB expression and a command. On E5 it repeats R139: unsat and 0 of 81 for the candidate, sat and the same 18 of 81 for the planted fault, witness 9 reproduced. A model that misses the fault is reported `inconsistent`. The E5 filter stays in the experiment crate. [R152](../docs/research/2026-11-18/18-bound-check-in-core.md) · [Evidence](D:/BuildArtifacts/lesscode/corrective-audit/task75/2.3)

**The reply-checking machine, found by people, measured by tools** (`45aff76`). The guide adapter keeps a reply's stop reason in a cell that one closure writes and another consumes; the repair loop calls them in order. `analyze` and `measure-scope` report two of its six parts. Causes: closures are not functions for the driver, match arms are not conditions, a method call on a cell is not a write. [R153](../docs/research/2026-11-18/19-reply-machine-tool-findings.md)

**The same machine in one place** (`f38dca5`). A version in which the check receives the whole reply gave the same trial record as today's code on all 125 histories of three replies. A planted fault was found and reproduced. The project's tests pass on it. Only the adapter refuses a cut-off or refused reply; with a text-only check, as `check.rs` and `measure.rs` use, the loop accepts one in 62 of 125 histories. [R154](../docs/research/2026-11-18/20-reply-machine-one-place.md) · [Patch, not applied](../docs/experiments/state-consolidation/reply-machine/one-place.patch) · [Evidence](D:/BuildArtifacts/lesscode/state-consolidation/r154)

Verification of the two ports: Windows fmt and clippy clean, nextest 409 of 409 at four threads, doctests, budgets 76 files with none exceeded; Linux through Dagger at `c1c76e8` overall exit 0, 408 of 408. No suite test calls a solver; that path ran on E5 and on the reply machine only.

## Decisions the owner may want to take or reverse

- **Apply the one-place patch?** It changes the signature of `with_repairs` in `lesscode-model` (the check gets the reply) and removes the cell from the adapter.
- **Should the loop itself refuse a reply that is not whole?** That gives all three callers the rule and changes what `lesscode-model check` and `measure` accept.
- **`measures.rs` was left as it is.** The prototype's fix for a callee in parentheses was not taken, so two Python edge rules exist. The frozen `scope-measures-v1` report is not a command, because it carries the SQLAlchemy vocabulary.
- **`bound-check` runs the target file's commands as written** and copies no tree. The help and the docs say so.
- **The prototype copy** `D:\BuildArtifacts\lesscode\corrective-audit\foreground-20261118\prototype` (a worktree at `b08adf1`) is no longer needed for 2.2 and was left in place.

## Things that cost time

- New files under `native/src/adapters/driver` must be flat files: `the_engine_names_no_language_or_project` reads each entry as a file.
- Budgets: 2,800 tokens a file, 560 a function; a new file needs an entry in `native/complexity-budgets.json`, which has CRLF line endings.
- Packs are limited to 40 lines and 2,500 characters. The Rust pack is at 40 and 2,480, so the three causes of R153 cannot be met by pack rules without making room or changing the limit.
- Do not pipe `cargo` into `Select-Object -First`; it ends the run. Use four nextest threads: with eight, `the_installed_hook_stops_a_push_without_a_matching_verdict` failed once and passed alone.

## Storage

D had 190.82 GiB free at entry and 179.58 GiB at close. Each of the two Dagger evidence runs took about 5 GiB of D that this project cannot prune (the Podman machine's disk). The shared Cargo target measured 47.56 GiB after 2.2 and was not measured again; lesscode-owned caches are about 28 GiB, under the 40 GiB limit. The isolated target of R154 (4.54 GiB) and its worktree are removed. Retained evidence: 39 MiB under `corrective-audit\task75`, 30 MiB under `state-consolidation\r154`, under 1 MiB under `state-consolidation\r153`.

## Next

1. The owner's two decisions on the reply machine above.
2. The find step, which is the open gate of `state-consolidation`: return the six parts of R153 from the four files alone. It needs closures as functions, match arms as conditions and cell writes as writes, in a way that fits the pack limit.
3. The remaining ports: R140 as a history form of `bound-check` (R154 encoded histories by hand as three integer symbols), then the Java lowering.
4. Then, as before: 3.1 with 5.2 and 3.4 (`pure_verifier.rs` can go now), 3.2 and 3.3, group 4, the rest of 5. The Python-only `check_syntax` in `surface.rs` is open; `tree::parse_clean` exists for it.

Local commits stay unpushed (`origin/main` at `a5df503`); a humanizer pass on `README.md` and `docs/driver.md` is due before any push. 6.2 and 6.3 are open. Slice B drift hardening (MIPROv2/GEPA) is unchanged, measured through R79, and awaits the owner's choice. Next research ID: R155. [Corrective tasks](corrective-tasks.md)

## Amendment, 15:25

The owner said to proceed with both decisions on the reply machine. The one-place patch is applied (`48010b5`). The repair loop itself now rejects a reply that was cut off or refused, for every caller (`3db7caf`): a text-only check accepted one in 62 of 125 histories before and in none after; the adapter's records differ in one reworded sentence only. Three tests that stated the old behaviour were changed and one was added. [R155](../docs/research/2026-11-18/21-reply-rule-in-the-loop.md) · [Evidence](D:/BuildArtifacts/lesscode/state-consolidation/r155)

For the next release notes: `lesscode-model` no longer accepts a reply that was cut off or refused, and the check passed to `with_repairs` takes the reply. D had 175.89 GiB free before the third Dagger run of the day; the builds of the model crate and the guide tool added about 3.7 GiB to the shared Cargo target, so lesscode-owned caches are near 32 GiB and should be measured before the next heavy build. Next research ID: R156.
