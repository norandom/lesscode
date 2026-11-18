# R137 · Results supplied by literals in seven historical gates and evaluators

Static audit on 18 November 2026, Europe/Berlin, following the 17 November overnight inspection. Covers [corrective task 1.5](../../../.kiro/corrective-tasks.md). The question was whether the number or verdict in a report came from the claimed specimen operation. The search included hidden and untracked files in both specified trees.

```powershell
rg --files --hidden --no-ignore .kiro/specs docs/experiments -g 'run_gate.py' -g '*_evaluator.py'
```

The inventory returned five gates and two evaluators. The table traces values to their reports. Thresholds, seeds and explicit test inputs are valid constants. A fixture run supplies evidence about that fixture.

| Source at `b08adf1` | Reported value and what produced it |
| --- | --- |
| [Rust gate](../../../.kiro/specs/rust-deconvolution-unification/run_gate.py), lines 101, 334, 343, 352, 361, 394–395, 410, 466 | Admission returns a fixed status after collecting paths and tool output. Four fixed kill flags feed the kill rate and the overall verdict. A successful process supplies 314 suite passes or 10 budget passes. A failing suite supplies exactly one failure. The ceiling field is always true. |
| [TypeScript gate](../../../.kiro/specs/typescript-deconvolution-adapter/run_gate.py), lines 128, 343–348, 435–454 | Admission returns true after collecting metadata. A true flag supplies the buffer-drop kill and feeds the control aggregate. Probe lines are counted, but a separate true flag supplies the whole-apparatus ceiling verdict. The budget count is a 10/0 projection of an exit code and a text match. The `enabled` value in the fourth probe is an input. |
| [Java gate](../../../.kiro/specs/java-deconvolution-unification/run_gate.py), lines 115, 333, 361, 369, 376, 441–454 | Admission returns a fixed status. The actual branch taken determines the kill flag, but a compilation failure or probe exception can produce a kill. Line 361 belongs to the comparison path. A 10/0 budget count follows an exit-code and text check. The ceiling flag is true after counting adapter lines, without a whole-apparatus comparison. |
| [Native closure gate](../../../.kiro/specs/native-deconvolution-closure/run_gate.py), lines 85–86, 98–121, 206–212, 301–303 | Two zero divergence totals accompany a test-name check. The bounded count is the expression `21 + 8` next to a comparison between functions defined in the gate. Four literal scope records supply E5/E6 compression outcomes, including write counts and purity values. The budget count is a 10/0 projection of a process and stdout predicate. |
| [State-boundary gate](../../../.kiro/specs/state-boundary-deconvolution/run_gate.py), lines 221–262 | Both measurement records are literals. They supply the function, edge, write, statement, line and purity inputs to the density verdict. The scoring function operates on those records. |
| [E5 evaluator](../../experiments/deconvolution/e5-state-boundary-lifecycle/e5_evaluator.py), lines 231–242 | Scope totals come from `measure_written`. A literal zero supplies the pure-function count. The adverse controls use local examples, as documented in R134. |
| [E6 evaluator](../../experiments/deconvolution/e6-report-envelope/e6_evaluator.py), lines 232–243, 358–376 | The same zero supplies the pure-function count beside measured scope totals. With no candidate root, a literal true bypasses the compression predicate, allowing the disposition expression to select PASS if the other checks pass. |

Use the actual source paths, commands and outputs when replacing these claims. The E5/E6 scope measurements have since been rerun in [R135](01-rust-scope-measurements.md). The present audit does not supply a mutation result for either trial. [Full inventory and literal search output](D:/BuildArtifacts/lesscode/corrective-audit/foreground-20261118) · [Earlier findings and their scope](../2026-11-17/21-audit-of-port-and-reduction-claims.md)
