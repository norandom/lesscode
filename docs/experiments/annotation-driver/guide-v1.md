# Annotation guide, version 1

You are given one source file with numbered lines. Mark the places where a
small change would alter what the code does for its callers. A separate tool
checks every mark against the syntax tree and makes the change itself; you
only point. Do not write or suggest replacement code.

## What to mark

Use exactly these kinds.

| Kind | Mark this | Anchor |
| --- | --- | --- |
| `branch_condition` | A condition that decides which behavior a caller observes: a guard, a validation, a state check, a loop exit | The whole condition expression, without the keyword and without enclosing parentheses that belong to the statement |
| `failure_state` | The place where the code signals failure: raising or throwing, returning an error, or passing an error on to the caller | The whole statement or expression that signals the failure |
| `state_write` | An assignment that changes state which is read later: a field, a counter, a flag, a stored value | The whole assignment, without a trailing statement terminator |
| `error_order` | A check that is directly followed by another check, where it matters which of the two is reported or acted on first | The whole first check, from its keyword to the end of its body |
| `pure_shell_boundary` | A function that only computes from its arguments, or one that only performs effects | The whole function |

## What makes a good mark

Prefer places where a mistake would matter and could go unnoticed:

- paths taken when something is wrong, missing, malformed, late or refused;
- checks that protect a rule the rest of the code relies on;
- the order of two checks when both can be true at once;
- writes whose effect is only visible later or elsewhere.

Skip places where nothing a caller can observe would change: logging and
diagnostics only, constants defined once at the top of a file, code that
merely forwards a value.

Do not mark tests, comments, text inside string literals, or generated code.
Mark each place once per kind. Mark as many places as genuinely qualify; do
not pad the list and do not stop at a fixed number.

## Anchor rules

The anchor must be copied exactly from the file: the complete text of one
expression or statement, character for character, including line breaks if
it spans lines. Never include a trailing statement terminator such
as a semicolon: for a statement, the anchor ends where its last expression
ends. The alternatives of a match or switch arm are patterns, not condition
expressions; do not mark them as `branch_condition`. Do not shorten it, do not join lines, do not add or remove
spaces. `start_line` is the line where the anchor begins and `end_line` the
line where it ends. A mark whose anchor does not match is discarded.

## Output

Return only a JSON array, with no text before or after it. Each element:

```json
{"file": "<path as given>", "start_line": 0, "end_line": 0, "kind": "<kind>", "anchor": "<exact text>", "reason": "<one short sentence: what a caller would see change>"}
```
