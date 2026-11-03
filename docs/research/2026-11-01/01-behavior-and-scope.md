# D01-01: preserve features and reduce their implementation

Discussion date: 2026-11-01. Written retrospectively on 2026-11-02; no new run.
Status: scope correction. Spec context:
[capability compression](../../../.kiro/specs/capability-compression-core/spec.json).

## Decision

The owner clarified that Lesscode studies code, not requirements reduction or
application-domain redesign. Facdrone is an example of an agent-grown repository,
not an upstream refactoring deliverable. Trials may use isolated copies, without
changing or committing the specimen. Preserve tests as behavioral boundaries.

## Why the direction changed

Modularization helps agents navigate large codebases but does not by itself
remove implementation. Lower routine complexity can be achieved by adding helpers.
The research must count the whole responsibility, including wrappers and callers,
and look for existing language/library capabilities or simpler representations.

The [Facdrone inspection](../../experiments/facdrone/README.md) records a concrete
modularization example and source census. It is read-only inspection and proposed
research, not evidence of successful deletion. Application-specific redesign
proposals were corrected as outside Lesscode's agenda.

## Consequence

Separate production, tests, fixtures and apparatus. Do not infer maintenance
hours from lines, and do not optimize hardware/performance or byte counts.
Falsify a replacement if it cuts behavior or merely exports complexity.

[Next: synthesis and local trials](02-synthesis-and-local-trials.md) | [Day index](README.md)
