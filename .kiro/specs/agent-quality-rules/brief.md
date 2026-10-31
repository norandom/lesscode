# Brief: agent-quality-rules

## Problem
Developers need to constrain coding agents before they add convoluted code, using reproducible measurements rather than relying solely on a simplification prompt.

## Current State
The efficiency-baseline evaluator is implemented and validated. Its self-evaluation includes a bounded simplification, meaningful CCN/LLOC charts and a documented whole-package duplication-coverage limitation. Package-scoped AGENTS.md instructions already guide names, idiom and evidence preservation; they are not a configurable prevention engine. This spec has approved requirements with 65 criteria across nine areas and a generated design whose quality gate passed. Human design approval, tasks and implementation remain pending.

## Desired Outcome
Developers can declare measurable coding rules, check proposed changes against them and receive actionable evidence of violations. Existing debt and newly introduced regressions are reported separately. Coding agents can use the same checks while making changes.

## Approach
Build on baseline evidence and profiles. Candidate rule families include per-function complexity and nesting where supported, duplication, source-growth budgets, public API/invariant preservation and configured architecture boundaries. Specify supported metrics and limits explicitly; unsupported analysis remains unavailable. Do not claim a prompt alone enforces a gate.

## Scope
- **In**: configurable absolute limits and change budgets, scoped non-regression ratchets, machine-readable violations with locations/evidence, contextual review criteria, safe requested AGENTS.md guidance updates, explicit rule exceptions and evaluator integrity.
- **Out**: implementing replacement analyzers, fixed universal quality scores, automatic rewrites, mandatory MCP and reducing behavior/security checks to meet structural metrics.

## Boundary Candidates
- Baseline owns measurement definitions and observed evidence.
- This feature owns preventive policy interpretation and the agent-facing check workflow; exact enforcement placement is a design decision.
- Optional skills consume policy and findings rather than replace deterministic checks.

## Upstream / Downstream
- **Upstream**: efficiency-baseline.
- **Downstream**: agent guidance/skills, transactional rewrites and optional integration.

## Constraints
English. Open-source analysis and local execution. No arbitrary global reduction target. Rules and exceptions must be visible and reproducible; a rewriting agent cannot silently relax its own gate. Users choose policies appropriate to their projects. Requirements are approved and design is generated; human design approval, tasks and implementation remain pending. Unsupported evidence and unverified required semantic review must not become passes.

## Evaluation
Compare identical tasks and starting snapshots with and without policy guidance/enforcement. Report behavior, violations, accepted changes, complexity/duplication/size deltas and observed agent cost separately. Adversarial trials must expose metric gaming and attempts to weaken the rules. The supplied Anthropic simplifier is an optional comparator and wording reference, not the acceptance oracle.

Side reference reviewed: [Anthropic code-simplifier](https://github.com/anthropics/claude-plugins-official/blob/main/plugins/code-simplifier/agents/code-simplifier.md). Useful policy ideas include preserving behavior, removing redundant layers, preferring readable code over compressed expressions and focusing changes on the requested scope. Its JavaScript/React conventions are project-specific examples. These instructions supply guidance; this feature still needs executable criteria, recorded measurements and independently controlled gates.
