# Requirements: `_escape_html` uses the standard library

Generated 2026-11-12 (Europe/Berlin) as a step of lesscode's gradual refactoring, on a copy of this
project. The step is declared in [step.json](step.json).

## Introduction

`HTMLFormatter._escape_html` in `x_likes_exporter/formatters.py` escapes the text of a tweet for the HTML
export with five chained `str.replace` calls. Python's `html.escape` does the same job. The step hands
the work to the standard library and keeps the output as it is today, character for character.

## Requirements

### Requirement 1: Same output, less own code
**Objective:** As a maintainer, I want the escaping done by the standard library, so that the project
keeps no escaping table of its own.

#### Acceptance Criteria
1. The `_escape_html` method shall return the same string as before the step for every input, including
   `&#39;` for an apostrophe.
2. The step shall leave the tests of `tests/test_formatters.py` passing.
3. The step shall not increase the number of tokens in `x_likes_exporter/formatters.py`.
