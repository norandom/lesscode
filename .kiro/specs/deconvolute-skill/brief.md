# Brief: deconvolute-skill

## Problem
Developers need a reusable Codex skill that removes unnecessary structure while preserving observed behavior.

## Current State
The product skill has not been implemented. The efficiency-baseline evaluator is complete; agent-quality-rules has a requirements draft and remains the next upstream dependency. Existing package AGENTS.md guidance and the self-evaluation are examples, not the product skill.

## Desired Outcome
Audit, selected rewrite and review modes with evidence-ranked candidates and measured benefit over plain Codex.

## Approach
Delete, inline, reuse, adopt native features, simplify, then introduce abstraction only when justified. Work in small reversible batches; report uncertainty and validation gaps.

## Scope
- **In**: skill instructions, conditional references, candidate evidence, conservative editing and review.
- **Out**: analyzer implementation, mandatory MCP and automatic pattern insertion.

## Boundary Candidates
Own the audit/refactoring workflow and candidate selection. Consume declared policy from agent-quality-rules and observed evidence from efficiency-baseline without changing their checks, thresholds or rule meanings.

## Out of Boundary
Do not claim passing tests prove equivalence or reduce safeguards to improve metrics.

## Upstream / Downstream
- **Upstream**: efficiency-baseline and agent-quality-rules.
- **Downstream**: transactional-rewrites and codex-integration.

## Existing Spec Touchpoints
- **Extends**: none.
- **Adjacent**: efficiency-baseline owns reporting semantics.

## Constraints
English. Discovery draft; phase approvals pending. Preserve public APIs and user changes. Evaluate skill-only trials before investing in MCP.
