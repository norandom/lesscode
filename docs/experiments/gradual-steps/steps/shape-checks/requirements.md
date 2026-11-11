# Requirements: `isPersistedWorkflowState` checks one rule per statement

Generated 2026-11-11 (Europe/Berlin) as a step of lesscode's gradual refactoring (release preparation
1.2.0), on a copy of this project. The step is declared in [step.json](step.json).

## Introduction

`isPersistedWorkflowState` in `src/state.ts` decides whether a parsed state file has the expected shape. It
packs three rules into one condition and three more into one returned expression. The step writes one rule
per statement, so that a rule can be added, removed or reported on its own. The function becomes longer;
the step is taken for readability, not for size.

## Requirements

### Requirement 1: Same decisions, one rule per statement
**Objective:** As a maintainer, I want each shape rule of a state file on its own line, so that a rule can
be changed without touching the others.

#### Acceptance Criteria
1. The `isPersistedWorkflowState` function shall accept and reject the same values as before the step.
2. The step shall leave the tests of `tests/state.test.ts` passing.
3. The step shall not increase the number of tokens in `src/state.ts` by more than 30.
