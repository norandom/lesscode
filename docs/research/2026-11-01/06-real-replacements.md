# D01-06: marginal deletion exposed a weakness in the compression method

Discussion date: 2026-11-01. Written retrospectively on 2026-11-02; no new run.
Status: one modest candidate, one marginal rejection; neither adopted upstream.
Spec context: [follow-up](../../../.kiro/specs/constraint-guided-compression/brief.md).

## Actual trial results

| Trial | Complete changed module | Behavioral evidence | Decision |
| --- | --- | --- | --- |
| [mvibe P02](../../experiments/proxy-routing-trial/README.md) | 334 to 311 lines; 190 to 174 AST statements | 1,172 tests pass on each copy; 118,588 differential observations match | Retain reviewable modest candidate |
| [Facdrone FCSV01](../../experiments/facdrone-csv-trial/README.md) | 1,056 to 1,053 lines | 54 rendered CSV artifacts match; selected suite has 33 passes / 13 pre-existing failures on both copies | Reject as marginal |

The Facdrone failures involve Windows publication behavior; the record does not
claim a passing full-project gate. The final saving is three lines, not an earlier
provisional four. Original specimen checkouts were preserved. These checks used
regression/differential execution, not SMT; the user explicitly asked about that.

## Owner correction and consequence

The owner rejected presenting a few saved lines in a 100k+ repository as the
desired outcome. The interpretation is a weakness in our method, not evidence
that Facdrone, mvibe or mmhmem lack convolution or compression opportunities.

The assistant initially emphasized better candidate discovery. The owner then
corrected that framing: the central mechanism must be constraint modeling driven
across the codebase by an LLM modeler. Preserve this correction; a better ranking
system alone is not the agreed follow-up.

[Next: constraint modeler](07-llm-constraint-modeler.md) | [Day index](README.md)
