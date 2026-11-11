# Requirements: `verdict()` collects its reasons in one pass and returns once

Generated 2026-11-11 (Europe/Berlin). Not yet approved by the owner.
This is the spec of one step of [gradual-refactoring-steps](../gradual-refactoring-steps/requirements.md)
(task 6.1). The step is declared in [step.json](step.json). The change was first made by hand in
[R89](../../../docs/research/2026-11-11/15-first-tracked-step.md) and refused there by the line ceiling then
in force; the patch is `docs/experiments/gradual-steps/step1/change-refused.patch`.

## Introduction

`verdict` in `native/src/recording.rs` decides the exit code of a recording run and lists the reasons. It
sets up its list of reasons inside a branch and has two returns. The step sets the list up once, lets the
three checks follow each other and returns once. Nothing a caller can observe changes.

## Requirements

### Requirement 1: Same results, no growth
**Objective:** As a maintainer, I want `verdict` to read top to bottom, so that the next check is added in
one obvious place.

#### Acceptance Criteria
1. The `verdict` function shall return the same exit code and the same reasons in the same order as before
   the step.
2. The step shall leave the test suite of the native core passing, including the complexity budgets.
3. The step shall not increase the number of tokens in `native/src/recording.rs`.
