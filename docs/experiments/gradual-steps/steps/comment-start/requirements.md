# Requirements: `printComment` starts every comment line in one place

Generated 2026-11-11 (Europe/Berlin) as a step of lesscode's gradual-refactoring-steps (task 6.2), on a copy
of this project. The step is declared in [step.json](step.json).

## Introduction

`printComment` in `CSVPrinter.java` writes the comment marker and a space at two places: before the first
line of a comment and after every line break inside it. The step moves the two statements into one private
method, `printCommentStart`, beside `printCommentSeparator`, which already ends a comment line.

## Requirements

### Requirement 1: Same output, one place
**Objective:** As a maintainer, I want the start of a comment line written in one place, so that a change
to it cannot be made at one place and forgotten at the other.

#### Acceptance Criteria
1. The `printComment` method shall write the same characters as before the step for every comment.
2. The step shall leave the tests of `CSVPrinterTest` passing.
3. The step shall not increase the number of tokens in `CSVPrinter.java`.
