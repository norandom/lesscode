# D01-05: reproducible baselines were necessary but were not compression trials

Discussion date: 2026-11-01. Written retrospectively on 2026-11-02; no new run.
Status: baselines and rejected shortcut. Hypothesis H6 in the [log](../README.md).
Spec: [capability-compression-core](../../../.kiro/specs/capability-compression-core/spec.json).

## Decision and evidence

The owner redirected work from platform validation to empirical measurement on
mmhmem and mvibe, requested Cargo installation for Rust tools, and required exact
versions and provenance. Main-only linear development replaced branch/PR work.

The [local-adoption record](../../experiments/local-adoption/README.md) reports
4,418 production / 3,805 test lines for the declared mmhmem Rust scope, and
5,218 production / 14,044 test lines for mvibe's declared Python/TypeScript scope.
These are distinct populations, not comparable feature-efficiency scores.
A Rust parser check rejected misleading measurements, and a proposed standard-
library proxy shortcut failed a behavioral probe.

## Hypothesis revision

The owner challenged the absence of savings and requested a real trial, naming
Facdrone as a useful specimen. Baseline counts do not show that a repository
cannot compress; they contain no replacement. The next step must produce an
isolated before/after candidate and verify its complete behavior boundary.

[Next: real replacements](06-real-replacements.md) | [Day index](README.md)
