# Brief: efficiency-baseline

## Problem
Developers need evidence that simplifying AI-generated code improves maintainability without damaging behavior.

## Current State
Initial discovery snapshot (2026-10-30): greenfield project; no product, measurements or benchmark results existed. This brief preserves discovery context. See [implementation status](implementation-status.md) and [readiness](readiness.md) for the current implemented evaluator and validation evidence.

## Desired Outcome
A local CLI produces reproducible baseline and candidate reports with open-source tools and independent behavior checks.

## Approach
Start with the approved Python fixture slice; evaluate Lizard complexity, jscpd duplication and native tests. Follow the adjacent evaluation protocol. Validate tools before selecting pinned versions.

## Scope
- **In**: analyzer adapters, corpus manifests, run artifacts, before/after comparison and evaluator integrity.
- **Out**: source mutation, skill policy and MCP packaging.

## Boundary Candidates
Own metric definitions, result schema, fixed scope and experiment protocol. Consume native test-runner results without replacing project tests.

## Out of Boundary
Do not implement transformation engines or infer AI provenance from code style.

## Upstream / Downstream
- **Upstream**: licensed fixtures and open-source analyzer CLIs.
- **Downstream**: deconvolute-skill and transactional-rewrites.

## Existing Spec Touchpoints
- **Extends**: none.
- **Adjacent**: downstream features consume this feature's measurement contracts.

## Constraints
At initial discovery: English; discovery direction approved and specification initialized, with requirements, design and tasks approvals pending. Those three approvals have since been recorded in `spec.json`. The continuing constraints are no proprietary analysis dependency, unavailable missing checks, evaluator independence and preservation of user changes.
