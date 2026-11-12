# Requirements: `Checkpoint.clear` removes its files without asking first

Generated 2026-11-12 (Europe/Berlin) as a step of lesscode's gradual refactoring, on a copy of this
project. The step is declared in [step.json](step.json).

## Introduction

`Checkpoint.clear` in `x_likes_exporter/checkpoint.py` asks for each of its two files whether it exists
and then removes it. `Path.unlink(missing_ok=True)` does both in one call. The step uses it. The frame
specification [frame.json](frame.json) lists the places where the file writes state today; the step adds
none.

## Requirements

### Requirement 1: Same effect, two statements fewer
**Objective:** As a maintainer, I want `clear` to state what it removes and nothing else, so that the
method has no branch of its own.

#### Acceptance Criteria
1. The `clear` method shall remove both checkpoint files when they exist, shall not fail when one or both
   are missing, and shall print the same line as before the step.
2. The step shall leave the tests of `tests/test_checkpoint.py` passing.
3. The step shall not increase the number of tokens in `x_likes_exporter/checkpoint.py`.
4. The step shall add no state write outside the declared frame.
