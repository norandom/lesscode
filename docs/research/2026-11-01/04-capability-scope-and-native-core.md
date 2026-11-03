# D01-04: capability-level reductions justified a narrow native evaluator

Discussion date: 2026-11-01. Written retrospectively on 2026-11-02; no new run.
Status: positive scoped trials and implemented measurement workflow.
Spec: [capability-compression-core](../../../.kiro/specs/capability-compression-core/spec.json).

## Decision and evidence

The owner rejected per-function goals: splitting code into many atomic functions
must not improve the acceptance score. Count complete capability ownership and
the wider containing scope.

- [ATS01](../../experiments/ats-report/README.md): 384 to 202 physical lines and
  298 to 135 LLOC; 8,820 comparisons match. No upstream adoption.
- [NPU01](../../experiments/npu-backends/README.md): 171 to 101 capability lines
  and 68 to 47 LLOC; 3,168 comparisons and 339 tests per version. This is software
  behavior evidence, not a hardware/performance experiment.
- [Native self-application](../../experiments/native-self-apply/trial/record.md):
  adopted reduction of 32 lines and 29 statements across the owned scope.

These were selected cases, not a general discovery success rate. Neither ATS01
nor NPU01 needed SMT. Native readiness and replay evidence are linked in the
[readiness report](../../reports/native-core-readiness.md); preparation records
must not be mistaken for the later completed checks.

## Tool consequence

The owner preferred compiled tooling, chose Rust, and directed removal of broad
layers before extension. The native comparison core and skill stayed narrow;
generic transactions, MCP and solver integration were not prerequisites.
Self-application became a maturity check after bootstrap, not a blocker to it.
Use nextest for Rust and pytest-xdist for Python checks; operability and
compression efficacy remain separate claims.

[Next: local baselines](05-local-baselines.md) | [Day index](README.md)
