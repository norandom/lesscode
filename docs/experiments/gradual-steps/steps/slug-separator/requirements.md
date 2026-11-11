# Requirements: `slugify` drops a condition that is always true

Generated 2026-11-11 (Europe/Berlin) as a step of lesscode's gradual-refactoring-steps (task 6.2), on a copy
of this project. The step is declared in [step.json](step.json).

## Introduction

`slugify` in `src/degradation.ts` writes a separator when `needsSeparator && slug.length > 0`.
`needsSeparator` is only ever set to `slug.length > 0` or to `false`, and `slug` never gets shorter, so the
second half of the condition is always true when the first is. A planted fault in that half cannot be
noticed by any test. The step removes it.

## Requirements

### Requirement 1: Same slugs, less code
**Objective:** As a maintainer, I want `slugify` to hold no condition that cannot be false, so that every
condition in it can be tested.

#### Acceptance Criteria
1. The `slugify` function shall return the same slug as before the step for every input.
2. The step shall leave the tests of `tests/degradation.test.ts` passing.
3. The step shall not increase the number of tokens in `src/degradation.ts`.
