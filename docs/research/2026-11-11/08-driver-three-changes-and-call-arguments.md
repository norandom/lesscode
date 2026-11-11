# R81: Three changes after the validation: a stop handler, three false successes removed, call-argument operators measured

- Report date/time and timezone: 2026-11-11, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-11, 08:40 to 09:35 local (code changes, then five batches from 08:56 to 09:34;
  times from commits and command output)
- Status: completed
- Spec: [agnostic-annotation-driver](../../../.kiro/specs/agnostic-annotation-driver/design.md), design
  Amendment 4 (Requirements 3.2, 5.7, 7.1)
- Hypothesis ID and predecessor: follows [R80](06-agnostic-driver-final-validation.md) and the losses recorded
  in [R75](01-specimen-controls-rerun-through-driver.md)
- Type: implementation record with one confirmatory measurement

## Decision

The owner chose three changes on 2026-11-11. All three are in; one of them changes what a run costs and is
measured here.

1. The driver now stops cleanly when told to. Requirement 5.7, met in part at validation, is met for Ctrl-C,
   Ctrl-Break, a termination signal and a closing console. A driver killed outright still relies on the
   restore journal.
2. Three functions that reported success without running anything are removed. R75 named two; the third was
   found while removing the first.
3. Call arguments are a sixth site kind with three operators. On the five gate scopes they add 125 mutants to
   557. Of the 125, 88 are killed or survive and 37 do not build. Swapping the first two arguments is nearly
   useless in Java and Rust (2 of 17 build) and is the owner's to switch off.

Separate outcomes: behaviour of the tool. Nothing here is a statement about the equivalence of a rewrite.

## Hypothesis and falsification

Fixed in design Amendment 4 before the batch receipts were read:

| Prediction | Result | Verdict |
| --- | --- | --- |
| Every mutant of the five earlier kinds keeps the class it had in R73 | 556 of 557; one Rust mutant ended in `timeout` at the 120-second limit instead of `killed` | refuted by one mutant |
| At least half of the new mutants are killed or survive | 88 of 125, 70% | holds |

The rule attached to the second prediction (switch the swap off in typed languages if fewer than half say
something) is not triggered. The per-language figures below are reported so that the owner can decide it on
better grounds than the overall share.

## Frozen inputs and method

- Code: commits `8e0e2b9` (removal), `56abd58` (stop handler), `eea0109` (call arguments). Binary for the
  batches built from the working tree of `eea0109`.
- Batches: the five gate scopes with the configurations of R73
  (`docs/experiments/annotation-driver/results/native/*-config.json`), fresh work directories, receipts in
  `docs/experiments/annotation-driver/results/native-args/`. Reference: the receipts of R73, matched by the
  hash of the mutated text.
- All five batches ran at the same time on one machine, beside two model-call batches.

## Evidence and results

### Stop handler

- `adapters/driver/interrupt.rs`: the handler sets a flag; the loop that waits for a project command sees it,
  kills that command's process tree, the runner restores the planted file and the run ends as void with exit
  code 3. The interrupted mutant gets no class and is not stored.
- Two system functions are declared in that file instead of adding a dependency. It is the crate's first
  unsafe code: two calls that register a function which only stores to an atomic.
- Test `tests/driver_interrupt.rs` sends a real signal (SIGTERM on Unix, Ctrl-Break on Windows) to a run whose
  test command sleeps for a minute, and checks exit code, void reason, restored copy and cleared journal.
  Green on all five CI platforms (run for `56abd58`).

### Removed

| Function | What it did |
| --- | --- |
| `PureCoreVerifier::compare` on the Rust, Java and TypeScript adapters | returned `Verified` with the number of cases the domain would have |
| `NativePureCoreVerifier::compare` | the same, and reported a divergence only when the candidate's name contained "divergent" |
| `NativeMutantGenerator::generate` | reported every control as killed ("default mock execution") |

None was reachable from the command line. What remains is real: `verify_equivalence` compares two functions
the caller supplies over a finite domain, and `evaluate_controls` takes the caller's evaluator. Ten tests
that pinned the removed behaviour went with it.

### Call arguments

The site is the argument list of a call. From the pack: a literal argument is swapped for its partner
(`true` and `false`, `0` and `1`), a non-empty string argument becomes the empty string, and the first two
arguments change places. At most two literal changes and one swap per call. A pack may leave the kind out.

| Scope | Earlier kinds, class unchanged | New mutants | Killed | Survived | Invalid | Wall time now (R73) |
| --- | --- | --- | --- | --- | --- | --- |
| Python `ratelimit.py` | 197 of 197 | 49 | 29 | 17 | 3 | 300 s (118 s) |
| TypeScript `state.ts` | 71 of 71 | 35 | 17 | 8 | 10 | 307 s (107 s) |
| TypeScript `verification.ts` | 55 of 55 | 19 | 10 | 0 | 9 | 231 s (88 s) |
| Java `Lexer.java` | 170 of 170 | 12 | 2 | 3 | 7 | 1,909 s (814 s) |
| Rust `update.rs` | 63 of 64 | 10 | 2 | 0 | 8 | 2,075 s (about 1,380 s) |
| Total | 556 of 557 | 125 | 60 | 28 | 37 | |

By operator:

| Operator | Python | TypeScript (two files) | Java | Rust |
| --- | --- | --- | --- | --- |
| Literal changed: killed / survived / invalid | 9 / 1 / 0 | 18 / 4 / 5 | 2 / 3 / 0 | none |
| First two arguments swapped: killed / survived / invalid | 20 / 16 / 3 | 9 / 4 / 14 | 0 / 0 / 7 | 2 / 0 / 8 |

- In Python the swap finds 16 survivors in one file: calls whose argument order no test pins down.
- In Java and Rust 15 of 17 swaps do not build, because the two arguments have different types.
- Wall times are not a measure of the operators' cost: five batches and two model runs shared the machine,
  and every batch took two to three times as long as alone. The one changed class is a Rust mutant that
  reached the 120-second limit under that load.
- Of the seven controls R75 recorded as lost, one is now reached: the changed string literal in a call
  (`TS_ADV_1`) has a counterpart that empties the string. The Java control that put `true` where a variable
  stood is not reproduced; its call gets a swap that does not build.

## Limits and interpretation

- One run per scope under heavy load. The classes of the earlier kinds are stable against it except for one
  time limit; the classes of the new mutants were not rerun.
- Five files. How many call-argument mutants a file gets depends on its style: between 7% and 49% more.
- Model-written packs under the present guide do not contain the new kind. They stay admissible, and their
  mutants are now a subset of the shipped packs' mutants; the comparison "same mutants as the shipped pack"
  of the Slice B runs (R68 to R79) was made before this change and is not affected.

## Tool/spec consequence and next gate

- For the owner: switch `swap_first_two` off in the Java and Rust packs (15 of 17 such mutants only cost a
  build), or keep it for the two that told something.
- The parity tests compare with the prototype after leaving the new kind out, and print its count per scope.
- `docs/driver.md` describes the stop behaviour and the sixth kind.

## Addition, 2026-11-11, about 10:20: the swap is switched off for Java and Rust

Owner decision of 2026-11-11 on the open point above: `swap_first_two` is removed from the Java and Rust
packs. On the gate scopes that takes away 7 Java and 10 Rust mutants, of which 15 did not build and 2 Rust
mutants were killed; the Rust scope now has no call-argument mutant and the Java scope has 5 (literals). The
Python and TypeScript packs keep the swap. The table above shows the state before this change.

## COBESY check

Outcome first: three changes, one measured; one prediction holds and one is refuted by a single mutant, with
its cause named as load and not verified by a rerun. Replay: commits, configurations and receipts named.
Disproof: thresholds from the amendment applied as written; the per-language picture that the overall
threshold hides is shown.
