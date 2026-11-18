# Requirements: the batch runner keeps its progress in one value

Generated 2026-11-18 (Europe/Berlin). Not yet approved by the owner.
The spec of one step, declared in [step.json](step.json), registered beforehand in the
[prototype gate](../state-consolidation/prototype-gate.md) of state-consolidation.

## Introduction

`run_batch` in `native/src/adapters/driver/runner.rs` gives its workers three locked fields: the queue of
mutants, the results, and the reason the run stopped. A worker asks two of them in turn before each mutant.
The step puts the three into one value behind one lock. A worker asks it once for the next mutant and gets
none when the run has stopped. Nothing a caller can observe changes.

## Requirements

### Requirement 1: Same results, one shared value
**Objective:** As a maintainer, I want the progress of a batch in one place, so that "stopped" and "nothing
left" cannot be asked apart.

#### Acceptance Criteria
1. The batch runner shall give the same receipts as before the step for the cases of the suites
   `driver_run` and `driver_interrupt`.
2. The step shall leave the test suite of the native core passing, including the complexity budgets.
3. The step shall not grow `native/src/adapters/driver/runner.rs` by more than 80 tokens.
4. The workers shall share one locked value where they shared three.
