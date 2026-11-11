# The agnostic driver (not yet in a release)

State on 2026-11-11: built on `main`, covered by the test suite on five platforms, not part of the released
binary 1.0.2. The older text-matching analysers and operators were deleted on 2026-11-11 (commit `964921c`).
Spec: `.kiro/specs/agnostic-annotation-driver/`. Evidence: research entries R65, R67 and R73 under
`docs/research/2026-11-10/` and R75 under `docs/research/2026-11-11/`.

The driver finds the places in a source file where behaviour is decided, plants small faults there
(mutants), and can run a project's own tests to see which faults they catch. Everything it knows about a
language is in a data pack; the same engine serves Rust, Python, TypeScript and Java.

## Commands

All are subcommands of `lesscode-core deconvolute`. Exit codes: 0 success, 1 finding, 2 incomplete, 3 bad
input, unsupported language or a void run.

| Command | What it does | Writes to the source tree |
| --- | --- | --- |
| `analyze --lang L --source F [--frame-spec S] [--out R]` | Lists the sites of a file by kind, the state-write sites, the function boundaries and the constructs the pack declares as gaps. Exit 1 on a gap or on a write outside the frame specification | no |
| `mutate --lang L --source F --out-dir D [--out R] [--limit N --seed S]` | Writes every valid mutant of a file into `D`, with a receipt naming site, operator and hash. Exit 1 when there is none | no |
| `run --config C --work W [--workers N] [--annotations A] [--no-reuse] --out R` | Mutates the configured files in copies under `W`, runs the project's commands against each mutant and classes it. Exit 2 when errors remain, 3 when the run is void | no; only the copies change, and each is restored and verified |
| `admit-pack --pack P --lang L [--controls K --source-root S] [--out R]` | Checks a language pack before it is used. Exit 1 when it is rejected | no |

`run` is the one command that executes a project's commands. It refuses a work directory that is the source
root, lies inside it or contains it.

## Site kinds and operators

| Kind | What it is | Operators |
| --- | --- | --- |
| `branch_condition` | A condition that decides control flow, or a returned yes-or-no expression | negate; swap paired operator tokens |
| `failure_state` | A place that signals failure to the caller | remove or rewrite the signal; swap a returned literal |
| `state_write` | An assignment, an in-place update, or a call that stands alone as a statement | drop the statement |
| `error_order` | A check that is directly followed by another check | swap the two |
| `pure_shell_boundary` | A function or method definition | none; reported by `analyze` |

A mutant that does not parse, or differs from the original only in comments or white space, is not written
and not run. A swap that says the same as the negation at the same site is left out.

## Classes of a mutant in a run

| Class | Meaning |
| --- | --- |
| `killed` | The tests failed |
| `survived` | The tests passed: a gap in the tests, or a change that makes no difference |
| `invalid` | The mutant did not build, or the tests could not be loaded |
| `timeout` | A step reached its time limit |
| `error` | The command could not start, the machine ran out of a resource, or the test runner left no sign that it ran. This says nothing about the mutant; it is retried once and never stored |

A run is void, and nothing in it may be relied on, when the unmutated tests do not pass before or after the
mutants, when a mutant could not be run by any copy, when a restoration could not be verified, or when the
run was stopped from outside.

Stopping a run: on Ctrl-C, Ctrl-Break, a termination signal or a closing console the driver kills the
project command it is waiting for with everything that command started, writes the original file back into
the copy, writes the receipt as void and exits with code 3. Mutants classed before the stop stay in the
outcome store and are reused by the next run. A driver that is killed outright (`kill -9`, the task manager)
cannot do this; the file it had planted is then restored when the copy is next opened.

## Run configuration

A JSON file. Commands are argument vectors; nothing is passed through a shell.

```json
{
  "language": "python",
  "source_root": "../my-project",
  "files": ["pkg/ratelimit.py"],
  "test_paths": ["tests/"],
  "setup": [["uv", "sync", "--quiet"]],
  "build": null,
  "test": ["uv", "run", "--no-sync", "python", "-m", "pytest", "tests/test_ratelimit.py", "-q", "-x"],
  "env": {},
  "exclusions": [".venv", "__pycache__"],
  "timeout_floor_s": 20,
  "timeout_ceiling_s": 300,
  "timeout_factor": 5,
  "invalid_exit_codes": [2, 3, 4, 5],
  "invalid_output_patterns": [],
  "error_output_patterns": [],
  "runner_evidence_patterns": [" passed", " failed", " error"],
  "timeout_exit_codes": [],
  "workers": 6,
  "serial_only": false
}
```

- `source_root` is relative to the configuration file. `setup` runs once in each fresh copy.
- The time limit of a step is `timeout_factor` times the slowest unmutated run of that step, at least the
  floor and at most the ceiling. An argument containing `{limit_s}` receives that limit in seconds; with
  `timeout_exit_codes` this lets a command that works inside a container stop itself and be classed as a
  timeout.
- Patterns are plain text fragments, not regular expressions.
- Each copy must build into its own directory. The driver copies the source tree; it cannot see a build
  cache that your tools share through the environment. Cargo with a global `CARGO_TARGET_DIR` is such a case:
  two copies then share compiled units, a copy can run tests against another copy's mutant, and classes come
  out wrong (16 of 133 in one measured run, R75). Set `"env": {"CARGO_TARGET_DIR": "target"}` so that each
  copy builds inside itself, or run one copy with `serial_only`. The driver notices the damage only through
  the closing run of the unmutated tests, which then fails and voids the run; the receipt names the copy and
  shows the end of the failing output.
- Outcomes are stored under `W/outcomes` by the hash of the original file, the mutated text and the
  configuration, and reused by later runs unless `--no-reuse` is given.

The configurations used for the five gate scopes are in
`docs/experiments/annotation-driver/results/native/`.

## Language packs

The shipped packs are `native/packs/{rust,python,typescript,java}.json`, at most 40 lines each. A pack names
the grammar's node kinds for each site kind and gives the operator templates. `admit-pack` checks size,
shape, that every node kind exists in the grammar, the template placeholders, and, with a control file,
that recorded annotations get their recorded outcome and that accepted ones yield a real change.

A control file is `{"files": [...], "test_paths": [...], "cases": [{"label", "annotations": [...],
"expected": [...], "operators": [...]}]}`; `operators` is optional and names the operators a site must
offer. Hand-written control sets are in `native/tests/fixtures/driver/controls/`.

## What is not there yet

- A release that contains the driver.
- Operators for a changed argument, a changed literal inside a call, the order of a list, or what a process
  prints. Seven of the twelve hand-made controls of the earlier specimen gates were of these kinds and have
  no driver mutant (R75).
- A fifth language: it needs a grammar compiled into the binary and an admitted pack.
- Faster Rust runs: each mutant rebuilds the crate.
