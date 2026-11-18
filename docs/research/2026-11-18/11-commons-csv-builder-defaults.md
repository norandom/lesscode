# R145 · Rust recovered the defaults that govern the CSV iterator

Rust traced the parser counter and row limit from their constructor assignments to two different nested Java classes, both named `Builder`. Using the recovered values and the iterator behavior already recorded in R144, the research harness forecast **108 of 108 values** across the original parser and two independently changed copies. Raising the counter default moved every observed count by six. Setting the row limit to one prevented the second record from being returned. [Forecasts sealed before execution](D:/BuildArtifacts/lesscode/corrective-audit/task71/verified/seal.json) · [Fresh JVM results](D:/BuildArtifacts/lesscode/corrective-audit/task71/verified/results.json)

## What Rust found in the source

This preregistered follow-up to R144 was completed and written up on 18 November 2026 at 12:22 CET. The extraction hypothesis would fail if either independent edit left the recovered value unchanged, a field from the other builder supplied the value, or the selector assigned a number to unsupported arithmetic. The citation check would fail if it chose between two matching locations on a wrong-line claim. [Registration record](D:/BuildArtifacts/lesscode/corrective-audit/task71/protocol.json)

R144 supplied the constructor assignments but missed the declarations that explained the initial values. The revised selector uses the declared parameter type and enclosing class to locate each field. It includes the assignment, type binding and declaration in the numbered packet. For the selected default format, it also includes the zero-argument factory called in the static field initializer. The original 367 lines are still present. The 19 additional lines bring the packet to 386 verified lines across five files. [R144](10-commons-csv-iterator-discovery.md) · [Source inventory](D:/BuildArtifacts/lesscode/corrective-audit/task71/verified/baseline/packet/selection.json) · [Line and delta checks](D:/BuildArtifacts/lesscode/corrective-audit/task71/verification.json)

| Value | Source of the default | Use in the public parser run |
| --- | --- | --- |
| Parser record counter | `CSVParser.Builder.recordNumber = 1` at `CSVParser.java:161` | The constructor at line 531 subtracts one, so the initial observable count is zero. |
| Format row limit | `CSVFormat.Builder.maxRows` at `CSVFormat.java:278` is a primitive `long` with no explicit initializer. Its Java default is zero. | The constructor copies it at line 1666. With zero, both records are accessible. |
| Default format | `CSVFormat.DEFAULT` at `CSVFormat.java:1052` constructs the format from `Builder.create()`. | Rust resolved the zero-argument factory at line 214. The packet contains its body and the parser's reference to the default format. |

The two numeric results describe the constructor assignments when the builder fields still have their declared defaults. A caller can override either value. Rust recognizes direct field copies and subtraction of a plain decimal literal from the qualified field. The changed-default experiment used this restricted subset.

## What happened when the defaults changed

The input was the Apache-2.0 Commons CSV source at `2bd66bf181d44ffc92806f585d3955e915666f7f`. Each disposable source tree was compiled separately with the R144 observer, JDK 21 and pinned Commons IO 2.22.0. Each of three histories used a new parser over `oak\nelm\n` and two iterator handles. Each arm produced 18 pairs of values, covering handle identity or operation result alongside the parser's current count. The research harness used the known R144 iterator transitions and the values extracted from each source tree. It froze all three forecasts before invoking Java. [Preregistration](../../../.kiro/specs/state-consolidation/prototype-gate.md) · [Research harness](../../experiments/state-consolidation/iterator-discovery/defaults-trial.ps1)

| Source arm | Initial count | Rows accessible | Observed difference from baseline | Matches |
| --- | ---: | ---: | --- | ---: |
| Baseline | 0 | 2 | Reproduced the original R144 recording byte for byte | 36/36 |
| Counter default 1 → 7 | 6 | 2 | All 18 counts rose by six. Every operation returned the same result. | 36/36 |
| Row limit 0 → 1 | 0 | 1 | Four results and seven counts changed. In the first history B returned `oak`, then A's attempt to consume the second record threw `NoSuchElementException`. | 36/36 |

The group ran on 18 November 2026, Europe/Berlin. The first three-arm run finished at 12:15:18. After tightening the accepted source syntax, the repeat froze its forecasts at 12:16:36 and completed the JVM observations at 12:16:48. Both runs produced identical observations in every arm. Adding identically named fields with values 99 and 42 to the wrong nested builders left the extracted defaults unchanged. Changing the initializer to `1 + 6` and a direct copy to `builder.maxRows + 1` left the corresponding numeric results unresolved. [First observations](D:/BuildArtifacts/lesscode/corrective-audit/task71/trial/results.json) · [Source controls](D:/BuildArtifacts/lesscode/corrective-audit/task71/verified/controls.json)

## Resolving the earlier quotations

Against the exact packet originally supplied to the reviewer, Rust confirmed 20 claimed lines and uniquely located the two misplaced quotations. The iterator return was at line 905, claimed as 904. The closed-parser exception was at 276, claimed as 275. Each audit row retains the claim, matching lines and resolved location. On a deliberately wrong line, `if (isClosed()) {` matched both 264 and 275, so Rust returned an ambiguous result. Absent text, text present only in another file, and an unknown filename also received no location. The command returned a failing exit code on these controls. R144's original exact-location grade remains 20/22. [Resolution of the sealed answer](D:/BuildArtifacts/lesscode/corrective-audit/task71/verified/citations.json) · [Unresolved controls](D:/BuildArtifacts/lesscode/corrective-audit/task71/verified/unresolved.json)

## Replay and decision

From the Lesscode checkout in PowerShell, run the complete experiment using the frozen R145 binary and R144 evidence. The script requires a new output directory.

```powershell
$e = 'D:/BuildArtifacts/lesscode/corrective-audit/task71'
$run = Join-Path $e ([guid]::NewGuid().ToString('N'))
& ./docs/experiments/state-consolidation/iterator-discovery/defaults-trial.ps1 -Selector "$e/defaults.exe" -NewEvidence $run
```

The two initialization gaps and the quotation-resolution check passed on this specimen. The research crate passed all four existing nextest tests, Rustfmt and Clippy. This trial made no new model or solver call and removed zero production lines. The next gate is to derive a transition from the Java source itself, ask Z3 for a short history that distinguishes a changed guard, and replay that history on the JVM. This would test the remaining hand-written part of the forecast. [Nextest](D:/BuildArtifacts/lesscode/corrective-audit/task71/nextest.log) · [Clippy](D:/BuildArtifacts/lesscode/corrective-audit/task71/clippy.log) · [Spec brief](../../../.kiro/specs/state-consolidation/brief.md)

The research tooling grew by 499 net physical lines. The Rust selector added 333 net lines including citation resolution, and the PowerShell replay added 166. The Java observer was reused as written. [Selector](../../experiments/state-consolidation/iterator-discovery/main.rs) · [Replay](../../experiments/state-consolidation/iterator-discovery/defaults-trial.ps1)
