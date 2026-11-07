# Brief: deconvolute-skill

## Problem
Developers need a reusable Codex skill that removes unnecessary structure while preserving observed behavior.

## Current State
The experimental `.agents/skills/lesscode-discovery/` workflow exists. Product packaging and measured skill efficacy remain unestablished. The baseline is complete; agent-quality-rules has an approved historical plan with 9/25 tasks marked complete and remaining work retired from active delivery. ATS01 and NPU01 motivate a capability-scoped workflow rather than per-function minimization.

## Desired Outcome
Audit, selected rewrite and review modes with evidence-ranked candidates and measured benefit over plain Codex.

## Approach
Delete, inline, reuse, adopt native features, simplify, then introduce abstraction only when justified. Work in small reversible batches; report uncertainty and validation gaps.

## Scope
- **In**: skill instructions, conditional references, candidate evidence, conservative editing and review.
- **Out**: analyzer implementation, mandatory MCP and automatic pattern insertion.

## Boundary Candidates
Own capability selection, repository-wide installed-capability knowledge and concrete replacement proposals. Freeze whole-replacement scope, use existing capabilities before new dependencies, and consume comparison evidence without weakening its checks or scope. Evolve the experimental skill alongside the narrow native core; the broad policy engine is removed from active delivery. Hardware/performance and byte-size optimization are excluded.

## Out of Boundary
Do not claim passing tests prove equivalence or reduce safeguards to improve metrics.

## Upstream / Downstream
- **Upstream**: efficiency-baseline reference evidence and proposed capability-compression-core.
- **Downstream**: none scheduled. Broad policy, transaction and integration layers are removed from this delivery scope.

## Existing Spec Touchpoints
- **Extends**: none.
- **Adjacent**: efficiency-baseline owns reporting semantics.

## Constraints
English. Discovery draft; phase approvals pending. Preserve public APIs and user changes. Use existing agent editing and isolated copies; do not build a generic rewrite engine, MCP interface or separate pattern adviser for this slice.
