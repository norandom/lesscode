# Brief: gradual refactoring, one checked step at a time

Written 2026-11-11 (Europe/Berlin) as a discovery brief. Nothing here is approved; requirements, design and
tasks follow the usual phases, and a prototype gate comes before production tasks.

## The owner's question (2026-11-11)

"What I need to answer is: is the code healthy and working afterwards. Can we support a gradual refactoring
somehow, step by step, minor release by minor release?"

This changes what the project is asked to show. Until today the claim was size: substantial reduction with
behaviour preserved. Measured by syntax the reductions are small (R87). The question now is not how much a
rewrite removes but whether a codebase can be changed in small steps, each of which is shown to leave it
working and no worse.

## What "working" and "healthy" can mean, mechanically

| Word | Check for one step | Exists today |
| --- | --- | --- |
| Working | The project's own tests pass on the step's branch | yes, any project |
| Working, where tests are thin | Planted faults in the changed files: are they noticed as often as before? A step must not lower the share of killed faults in the code it touches, and must not add unnoticed ones without saying so | `deconvolute run`, per file; no comparison of two runs yet |
| Same outside | The public surface of the changed scope is unchanged | `surface` command, Python and TypeScript only |
| Not bigger | Tokens and nodes of the changed files do not grow, or the growth is declared | `deconvolute density` |
| Not harder to change | Limits on routines and files, duplicate blocks, scattered state: no new breach | [openidea-kiro](https://github.com/norandom/openidea-kiro) `kiro-cli code check`, with a ratchet |
| Still inside its frame | State is written only where declared | `deconvolute analyze --frame-spec` |

Each row is a check that can fail. None of them proves equivalence; together they say "nothing we can
measure got worse, and the tests that exist still pass and still bite".

## What is missing

1. **A step verdict.** One command that takes the state before a step and after it (two Git revisions) and
   answers per changed file: tests pass; fault sensitivity not lower; size not higher; frame respected. Today
   each check is a separate command with a separate receipt, and nothing compares two runs.
2. **A ledger.** A small file in the repository that records, per step and per release, what was measured,
   so that "minor release by minor release" is a visible series: release, steps taken, verdict of each,
   numbers before and after. A step that makes something worse can still be taken, but it is written down.
3. **A ratchet.** The measured values of the last accepted step are the floor for the next one, per file.
4. **A way to choose the next step.** Where are survivors concentrated, where is the most state written
   outside a frame, which file breaches a limit: a ranked list, so that a step is small and aimed.

## What it would not do

- Decide what to refactor or write the refactoring. A person or a model does that; this judges the step.
- Replace the project's tests. Where they are thin, the fault check says so; it does not supply behaviour.
- Promise size. A step may remove nothing and still be worth taking.

## Prototype gate (before any production task)

Take two real steps on this repository and one on a foreign project, by hand with today's commands, and see
whether the four checks can be compared across a step and whether the comparison says something a reviewer
did not already know.

- Step A, this repository: replace `verdict()` in `native/src/recording.rs` by the candidate's version
  (R86: compiles, tests pass). Expected: tests pass, killed faults 10 of 11 before and 10 of 12 after, fewer
  tokens.
- Step B, this repository: a step that should be refused, for example one that removes a test or drops a
  check, to see that the verdict notices.
- Step C: one small step in one of the gate scopes of another project.
- The gate passes if all three verdicts can be produced from receipts without judgement calls, and Step B
  is refused for the right reason. It fails if comparing fault runs across a step needs manual matching.

## Relation to other work

- `agnostic-annotation-driver` (complete): supplies `run`, `analyze`, `density`, `merge-annotations`.
- openidea-kiro: supplies the "not harder to change" check and its ratchet; this spec would call it or read
  its report, not rebuild it.
- Slice B (closed): a model may propose a step; the mechanical checks decide.
