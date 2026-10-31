# Roadmap

## Overview
Develop Deconvolute as a measured system for improving AI-heavy codebases and preventing agents from adding convolution. Establish open-source measurement first, then configurable coding rules and regression gates, followed by deterministic rewriting and optional agent interfaces.

Status: efficiency-baseline requirements, corrected design and tasks are approved; implementation has completed 32 of 33 executable tasks, including reproducible installed pilots, adverse-path evidence and actual native Windows/Linux/macOS validation. Task 7.4 readiness review and final feature validation remain pending; no feature-level GO is recorded. The 2026-10-31 clarification adds prevention through measured agent rules as the next proposed feature. Later feature specifications remain discovery briefs, not approved implementation tasks.

## Approach Decision
- **Chosen direction**: open-source, local-first, benchmark-first delivery.
- **Why**: every developer can reproduce analysis without a commercial license; controlled comparisons reveal whether each layer adds value.
- **Alternatives considered**: immediate MCP composition is quick to demonstrate but obscures attribution; three specialized semantic backends add premature maintenance. Full plugin packaging follows proven skill value.

## Scope
- **In**: measurement harness, independent evaluator, configurable measured coding rules and change gates, optional skill, later reversible transformations and MCP.
- **Out**: SciTools dependencies, mandatory paid analysis, automatic GoF insertion, AI-origin detection, guaranteed equivalence and universal quality scores.

## Constraints
Core analysis tools must be open source and usable without a paid account. LLM costs are separate. Python is the first vertical slice; eventual JS/TS and Java support does not imply initial parity. Preserve behavior, APIs, security checks and user changes.

## Boundary Strategy
- **Why this split**: establish the evaluator before optimization; separate instruction policy, deterministic edits and delivery integration.
- **Shared seams to watch**: metric definitions, scope manifest, result schema, evaluator ownership and unavailable-check semantics.

## Specs (dependency order)
- [ ] efficiency-baseline -- Open-source measurement, fixtures and reproducible comparison reports. Dependencies: none
- [ ] agent-quality-rules -- User-configurable measured coding constraints, change ratchets and agent-facing violations to prevent new convolution. Dependencies: efficiency-baseline
- [ ] deconvolute-skill -- Audit and reversible simplification policy compared with plain Codex. Dependencies: efficiency-baseline
- [ ] transactional-rewrites -- Evidence-backed preview/apply/rollback with a small rule set. Dependencies: efficiency-baseline, deconvolute-skill
- [ ] codex-integration -- Optional local MCP facade and verified Codex packaging. Dependencies: deconvolute-skill, transactional-rewrites

## Decision checkpoints
1. efficiency-baseline requirements, corrected design and tasks approved. Next: implement the baseline through the Kiro implementation workflow.
2. Prove unchanged-source measurement stability and rejection of behavior-breaking reductions.
3. Specify measured agent rules and compare ungated versus rule-guided/gated trials; use the example simplifier skill only as a side reference. Evaluate skill-only trials as an optional comparison.
4. Compare skill-only with tool-assisted trials before expanding semantic backends.
