# R146 · Rust lowered a real Java guard and Z3 chose the history that exposes it

Rust parsed the closure guard of the Commons CSV iterator's `next()` from each source version and lowered it to an SMT predicate. Z3 then searched all event histories up to length four. For a redundant-parentheses rewrite it found no difference at any depth. For `isClosed() && current == null` it returned `hasNext, close, next` as the shortest distinguishing history. The fresh JVM replay matched the sealed prediction on both versions: the original throws `NoSuchElementException`, the changed parser returns `oak`. A full JVM census of all 121 histories agreed with every per-depth solver verdict. [Sealed witness](D:/BuildArtifacts/lesscode/corrective-audit/task72/verified/seal.json) · [Census result](D:/BuildArtifacts/lesscode/corrective-audit/task72/verified/result.json)

## Hypothesis and registration

This preregistered follow-up to R145 was written and executed on 18 November 2026, Europe/Berlin. The gate was registered at 12:26:13 before any solver or JVM run. It would fail on a difference for the clean control, a JVM difference shorter than the solver's witness, a witness trace mismatch, or an unsupported guard accepted as modeled. [Preregistration](../../../.kiro/specs/state-consolidation/prototype-gate.md) · [Registration hash](D:/BuildArtifacts/lesscode/corrective-audit/task72/registration.json)

## What came from the source and what came from the researcher

Rust locates the single public zero-argument `next()` declared in `CSVParser.CSVRecordIterator`. It requires a leading `if` whose only consequence is the original `throw`, with no `else`. It lowers the condition from an accepted subset: the unqualified `isClosed()` call, `current == null`, parentheses and `&&`. The original lowers to `closed`. The changed copy at `CSVParser.java:275` lowers to `(and closed (not pending))`. [Lowered predicates](D:/BuildArtifacts/lesscode/corrective-audit/task72/verified/changed-query/result.json) · [Guard lowering](../../experiments/state-consolidation/iterator-discovery/guard_trial.rs)

These two predicates are the only transitions derived from Java in this experiment. The rest of each step is the hand-written R144 model: look-ahead and consumption, the pending record, the two-record input, the read counter and the encoding of return values. To keep that model from being applied to code it does not describe, Rust masks the guard condition and requires the remaining parser source to hash to the pinned original. Any edit outside the condition is refused.

| Source control | Expected outcome | Observed |
| --- | --- | --- |
| `CSVParser.this.isClosed()` | Refuse a changed receiver | `unsupported guard expression`, no query written |
| `isClosed() \|\| current == null` | Refuse a disjunction | `unsupported guard operator or operands`, no query written |
| `current = null;` before the throw | Refuse an extra effect | `guard must only throw`, no query written |

## Solver and JVM results

The pinned Z3 binary received one QF_LIA query per depth over the three events `hasNext`, `close` and `next`. Each query asserts that some step's public return or exception class, or the parser's record count, differs between the two versions. Rust sealed both predicted traces at 12:38:44. The JVM census began at 12:38:48. Each of the three Java builds compiled the disposable source tree with JDK 21.0.12.1 and pinned Commons IO 2.22.0. Each history used a newly constructed parser over `oak\nelm\n`. [Trial script](../../experiments/state-consolidation/iterator-discovery/guard-trial.ps1) · [JVM observer](../../experiments/state-consolidation/iterator-discovery/GuardReplay.java)

| Length | Histories | Clean control (solver) | Changed guard (solver) | Changed guard (JVM histories that differ) |
| ---: | ---: | --- | --- | ---: |
| 0 | 1 | unsat | unsat | 0 |
| 1 | 3 | unsat | unsat | 0 |
| 2 | 9 | unsat | unsat | 0 |
| 3 | 27 | unsat | sat | 1 |
| 4 | 81 | unsat | sat | 7 |

The original and parentheses builds produced byte-identical censuses of 547 rows. In each of the eight differing histories, a `hasNext` fills the pending record, the parser is closed, and a later `next` arrives with no consuming `next` in between. In `next, hasNext, close, next` the changed parser returns `elm`. The change is visible only when a look-ahead has filled the pending record before the parser closes. Fresh processes on the original and changed builds reproduced the solver's four witness rows, counts included.

## Replay and decision

From the Lesscode checkout in PowerShell, run the complete experiment with the frozen selector. The script requires a new output directory under `D:\BuildArtifacts`.

```powershell
$e = 'D:/BuildArtifacts/lesscode/corrective-audit/task72'
$run = Join-Path $e ([guid]::NewGuid().ToString('N'))
& ./docs/experiments/state-consolidation/iterator-discovery/guard-trial.ps1 -Selector "$e/selector/iterator-discovery.exe" -NewEvidence $run
```

The gate passed on this specimen. The selector was built with Cargo 1.98.1 from the research manifest using `--locked --offline`. The research crate passed its four existing nextest tests, Rustfmt and Clippy with warnings denied. No model was called and production reduction was zero. [Nextest](D:/BuildArtifacts/lesscode/corrective-audit/task72/nextest.log) · [Clippy](D:/BuildArtifacts/lesscode/corrective-audit/task72/clippy.log) · [Evidence hashes](D:/BuildArtifacts/lesscode/corrective-audit/task72/evidence-sha256.json)

The result is narrow. One guard expression comes from source. Its surrounding transition system is still supplied by the researcher and pinned by hash. The open step is to derive more of that system from the Java body itself, for example the look-ahead that fills `current` in `hasNext()`, and to try the method on a source that was not chosen for this protocol. General discovery and production adoption remain open.

The research tooling grew by 469 physical lines: 270 lines of Rust for guard lowering and solver calls, 6 lines of dispatch in the selector, 140 lines of PowerShell for the trial and 53 lines of Java for the observer.
