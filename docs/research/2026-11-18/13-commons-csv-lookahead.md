# R147 · Rust lowered the whole CSV look-ahead and Z3 found each changed version's shortest exposing history

Rust lowered the full body of the Commons CSV iterator's `hasNext()` from source: the closure guard, the fill of the pending record and the returned condition. It also lowered the closure guard of `next()`, as in R146. Z3 compared three edited copies with the original at every history length up to four. For a rewrite with swapped operands and extra parentheses it found no difference at any depth. For an unconditional fill it returned `hasNext, hasNext`. For an inverted fill test it returned `hasNext`. Fresh JVM processes matched the four sealed witness traces, and censuses of all 121 histories agreed with every per-depth verdict for both changed versions. [Sealed witnesses](D:/BuildArtifacts/lesscode/corrective-audit/task73/verified/seal.json) · [Census result](D:/BuildArtifacts/lesscode/corrective-audit/task73/verified/result.json)

## Hypothesis and registration

This preregistered follow-up to R146 was written and executed on 18 November 2026, Europe/Berlin. The gate was registered at 12:47:43, before any code for it existed. It would fail on a difference for the clean control, disagreement between the solver and the JVM at any depth, a witness trace mismatch or an accepted refusal control. Before execution the researcher expected `hasNext` for the inverted arm and `hasNext, hasNext` for the unconditional arm. This expectation was not a gate criterion, and both witnesses matched it. [Preregistration](../../../.kiro/specs/state-consolidation/prototype-gate.md) · [Registration hash](D:/BuildArtifacts/lesscode/corrective-audit/task73/registration.json)

## What came from the source and what came from the researcher

Rust accepts the body of `hasNext()` only as three statements in order. The first is a closure guard whose block only returns `false`. The second fills `current` from `getNextRecord()`, either unconditionally or inside one `if` with no else. The third returns a condition. Conditions may use the unqualified `isClosed()`, comparisons of `current` with `null` in either order using `==` or `!=`, parentheses and `&&`. For each version Rust produces four predicates over `closed` and `pending`. [Look-ahead lowering](../../experiments/state-consolidation/iterator-discovery/lookahead_trial.rs)

| Arm | Fill test in `hasNext()` | Return | Lowered fill | Lowered return |
| --- | --- | --- | --- | --- |
| Original (`CSVParser.java:263-271`) | `current == null` | `current != null` | `(not pending)` | `pending` |
| Clean | `(null == current)` | `null != current` | `(not pending)` | `pending` |
| Unconditional | none | `current != null` | `true` | `pending` |
| Inverted | `current != null` | `current != null` | `pending` | `pending` |

The closure guards lowered to `closed` in every arm. The remaining semantics still come from the hand-written R144 model: what `getNextRecord()` reads, consumption in `next()`, the two-record input, the read counter, `close` and the encoding of return values. Rust masks the two lowered regions and requires the remainder of the parser to hash to the pinned original.

| Source control applied to the original | Expected outcome | Observed |
| --- | --- | --- |
| Fill from `CSVParser.this.nextRecord()` | Refuse a different reader | `unsupported fill` |
| `recordCount++;` before the return | Refuse an extra statement | `unsupported hasNext body` |
| `return current != null \|\| isClosed();` | Refuse an unsupported return | `unsupported return: unsupported guard operator or operands` |
| Changed exception message in `next()` | Refuse a change outside the lowered regions | `source outside the modeled regions changed` |

The fourth control was added during implementation, after registration. It checks the hash pin directly. No query was written for any control.

## Solver and JVM results

Each comparison sent the pinned Z3 one QF_LIA query per depth over `hasNext`, `close` and `next`. Steps differ when a public return, an exception class or the parser record count differs. Rust sealed both witnesses with their predicted traces at 12:51:53. The JVM census began afterwards. Each of the four Java builds compiled a disposable source tree with JDK 21.0.12.1 and pinned Commons IO 2.22.0. The R146 observer was reused unchanged, with a new parser over `oak\nelm\n` for every history. [Trial script](../../experiments/state-consolidation/iterator-discovery/lookahead-trial.ps1) · [JVM observer](../../experiments/state-consolidation/iterator-discovery/GuardReplay.java)

| Length | Histories | Clean (solver) | Unconditional (solver / JVM differing) | Inverted (solver / JVM differing) |
| ---: | ---: | --- | --- | --- |
| 0 | 1 | unsat | unsat / 0 | unsat / 0 |
| 1 | 3 | unsat | unsat / 0 | sat / 1 |
| 2 | 9 | unsat | sat / 1 | sat / 4 |
| 3 | 27 | unsat | sat / 4 | sat / 12 |
| 4 | 81 | unsat | sat / 13 | sat / 36 |

The original and clean builds produced byte-identical censuses of 547 rows. In the inverted build a fresh iterator never reads, so the first `hasNext` returns `false` at count zero where the original returns `true` at count one. The unconditional build reads again on the second `hasNext`, which advances the count to two and overwrites the unread `oak`. The fresh replays reproduced these rows exactly.

The refactor of the R146 guard code was checked by replaying R146 with the new selector. Its per-depth verdicts, witness and the eight differing histories matched R146, and its length-three query was byte-identical. [R146 regression](D:/BuildArtifacts/lesscode/corrective-audit/task73/r146-regression/result.json)

## Replay and decision

From the Lesscode checkout in PowerShell, run the complete experiment with the frozen selector. The script requires a new output directory under `D:\BuildArtifacts`.

```powershell
$e = 'D:/BuildArtifacts/lesscode/corrective-audit/task73'
$run = Join-Path $e ([guid]::NewGuid().ToString('N'))
& ./docs/experiments/state-consolidation/iterator-discovery/lookahead-trial.ps1 -Selector "$e/selector/iterator-discovery.exe" -NewEvidence $run
```

The gate passed on this specimen. The selector was built with Cargo 1.98.1 from the research manifest using `--locked --offline`. The research crate passed five nextest tests, including a new test of the comparison lowering, along with Rustfmt and Clippy over all targets with warnings denied. No model was called and production reduction was zero. [Nextest](D:/BuildArtifacts/lesscode/corrective-audit/task73/nextest.log) · [Clippy](D:/BuildArtifacts/lesscode/corrective-audit/task73/clippy.log) · [Evidence hashes](D:/BuildArtifacts/lesscode/corrective-audit/task73/evidence-sha256.json)

Two of the iterator's three operations are now partly derived from source: all of `hasNext()`'s control flow and the guard of `next()`. What the fill reads is still matched by exact text and supplied by the model, as are the rest of `next()` and the record reader. The open steps are to lower the consumption in `next()` and to apply the method to a source that was not chosen for this protocol. General discovery and production adoption remain open.

The research tooling grew by 415 net physical lines: 244 lines of Rust, including the lowering, the shared depth runner and dispatch, and 171 lines of PowerShell for the trial. The Java observer was reused unchanged.
