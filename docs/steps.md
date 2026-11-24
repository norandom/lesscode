# Refactoring in checked steps

Status: part of `lesscode-core` from release 1.2.0 (spec
[gradual-refactoring-steps](../.kiro/specs/gradual-refactoring-steps/design.md)). All commands
below exist, and they work alike on Python, TypeScript, Java and Rust: the declaration names the language
and your project's own test command. Six real steps have been taken with them from start to finish, in
Python, TypeScript, Java and Rust projects ([examples](#examples)).

Why it works this way, for readers new to the idea: [testability.md](testability.md).

A step is one small change on one branch with one small spec. The tooling answers by exit code whether the
project's tests still pass after the step and whether they still notice planted faults in the changed code
at least as often as before. It writes no test and no refactoring; it judges what a person or a model wrote.

Exit codes of every `step` command: 0 accepted or done, 1 refused or rejected, 2 not judged because a run
is incomplete, 3 bad input or a void run.

## The path in three parts

1. See what is protected. `step plan` runs your tests against planted faults and says, per function,
   how many they notice ([section 1](#1-see-where-a-step-would-help-step-plan)).
2. Protect what you will change. For the functions you are going to make smaller, the brownfield test
   stage adds tests until nothing in them waits. It changes no code
   ([`step protect`](#protect-what-you-will-change-step-protect)).
3. Make it less. Each reduction is a step of its own: declared, pinned where needed, judged and
   recorded ([sections 2 to 6](#2-declare-the-step)).

Skip the middle part where the plan shows the functions protected already. Nothing refuses a step because
the stage was skipped: the stage advises, and the step is judged as any other.

## 1. See where a step would help: `step plan`

```sh
lesscode-core step plan --config run.json --work ../work --out plan.json [--frame-spec frame.json]
```

`run.json` is a run configuration as for [`deconvolute run`](driver.md): the files, the test paths and the
project's own test command. The faults run in copies under `--work`, which must lie outside the source
root; the source tree is only read.

The plan lists every function of the configured files, ranked by unnoticed faults:

| Column | Meaning |
| --- | --- |
| noticed | planted faults the tests caught |
| unnoticed | planted faults the tests let through; each is a behaviour a change could alter unseen |
| tokens | size of the function, outside comments |
| writes | state writes in it; with `--frame-spec`, the receipt also counts those the specification does not allow |
| "no fault noticed" | faults were planted and the tests caught none: nothing in this function is protected |
| "no fault planted" | the pack offers no change for this function; the plan says nothing about it |

For a function with no noticed fault the receipt (`step-plan-v1`) lists what it reaches out to: the names
it calls and its state writes. That is where a test needs a fake, or where the function stays out of the
step.

What to do with a plan: name the functions you are going to change. Where they have unnoticed faults,
write tests that turn them into noticed ones, and only then change the code. For one or two faults a pin
of the step does it ([section 3](#3-pin-before-you-cut-step-pin)); for more, or for several functions,
the [brownfield test stage](#protect-what-you-will-change-step-protect).

### Watch the run in a browser: `--page` and `--open`

A plan of a subsystem can take an hour, because every planted fault costs a build and a test run. Two
options of `step plan` and `step check` show what is happening.

| Option | What it does |
| --- | --- |
| `--open` | Starts your default browser on a page that follows the run while it goes on. The page is served from this machine only (`127.0.0.1`). At the end the report is written as with `--page`. |
| `--page [place]` | Writes the report of the run as one HTML file when the run ends and prints where. Without a place it goes beside the other output. Nothing is served. |
| `--page-source` | Shows the line of code of the example fault on the page. Without it the page has names and no source text. |

```sh
lesscode-core step plan --config run.json --work ../work --out plan.json --open
```

The page answers one question per command. For `step plan`: which of your functions do your tests
protect? For `step check`: does this change keep what your tests protect? It says first that no file of
your project is changed, then shows the steps of the run in one row and where the run is.

| Word on the page | Meaning |
| --- | --- |
| planted fault | one small mistake made on purpose in a copy of your code, such as a `<` turned into a `<=` |
| noticed | a test failed on the fault |
| unnoticed | every test passed although the fault was there |
| unprotected | a function in which no fault was noticed |

An unprotected function is not wrong. It has no test that would tell you if it broke. Leave it alone if
you do not plan to change it. If you do, protect it first: that is the
[brownfield test stage](#protect-what-you-will-change-step-protect), a small spec whose tasks are the
tests for it. If it talks to a device, a window or a network, a test cannot
come first; separate what it decides from what it does and test the part that decides.

Every run of your commands in a copy is a build, and builds side by side can take more memory than the
machine has. The copies therefore take turns where memory asks for it. The first build of a copy and
the first build with a planted fault each run alone once, and the tool notes how much memory they took.
After that a run starts beside others only when the memory that was free with nothing running holds
all of them, with a quarter to spare. The tool says so when a copy waits. On a system that does not
tell how much memory is free (macOS), nothing is held back.

If the first run of a copy does not get through, the tool says which of two things happened: the
commands ended before any test ran (a build failed or was ended by the system), or a test did not
pass. It tells them apart by `runner_evidence_patterns` in the configuration, the fragments of output
that show your test runner ran; without them it cannot. The whole output is kept under the work
directory as `first-run-w<copy>-<attempt>.log`. The copy tries once more when no other copy is in its
first run. If that fails too, the copy stops, the page says so, and the run goes on with the others.
When no copy gets through, the run cannot judge anything; where no test ran at all, `step check`
answers "not judged" and does not say that your tests fail.

The report is a single file and can be kept or sent. It holds the names of your files and functions; see
[`step reports`](#several-reports-on-one-page-step-reports) for a copy without them.

## Protect what you will change: `step protect`

Status: in `lesscode-core` from release 2.2.0 (spec
[brownfield-test-stage](../.kiro/specs/brownfield-test-stage/design.md)). The stage was walked once by
hand, on a renderer ([R184](research/2026-11-24/05-brownfield-stage-prototype.md)). It has not yet been
walked on a real project with these two commands.

The brownfield test stage comes between a plan and a reduction. You name the functions of the plan you
are going to make smaller. The stage adds tests until each of them is protected enough, and it changes no
line of the code. Two commands begin and end it; in between are the commands of a step, `step pin` and
`step check`. An agent walks it with the skill
[protect-before-reduce](../.agents/skills/protect-before-reduce/SKILL.md), which says who does what and
where it waits for you.

| Word | Meaning |
| --- | --- |
| written off | the owner of the code approved a reason why no test is to notice a fault |
| waiting | unnoticed, and not written off |
| protected enough | nothing in the function is waiting |

### Begin: `step protect begin`

```sh
lesscode-core step protect begin --plan plan.json --config run.json \
    --function <file>:<name> [--function <file>:<name> ...] --spec <name> \
    [--specs-dir .kiro/specs] [--branch protect/<name>]
```

Run it on the commit the plan was made from, with nothing uncommitted. It runs nothing. It reads the plan,
the receipt of the plan's run, the run configuration and the code, and writes two files into
`<specs-dir>/<name>/` and nowhere else:

- `step.json`, a declaration as in [section 2](#2-declare-the-step). Its scope is the files of the chosen
  functions, its `focus` the functions, and `max_token_growth` is 0. The stage's `step pin`,
  `step check` and `step protect result` take it.
- `stage.json`: what is to be protected. Per chosen function, every fault the plan planted in it, by an
  id that stays the same between runs (`negate@68:11`: operator, line and column in the file).

Before it ends it prints:

- the chosen functions, each with its unnoticed faults and the line they are in;
- the unprotected functions you did not name ("left alone"), and any named function that lies wholly in
  code not built on this system ("left out");
- how many faults will be run again, and about how long each of the two runs of them takes, from the
  seconds they took in the plan;
- what one copy of the plan takes on disk, measured on the plan's first copy. Run the command before you
  remove the plan's copies; afterwards it says that the size is not known.

| Exit | Meaning |
| --- | --- |
| 0 | the stage is begun; the two files are written |
| 3 | refused, and nothing is written: a function the plan does not have (the ones it has are listed), a plan without a result, a file that changed since the plan, a stage already begun there, a test path without a file, a specs directory outside the repository |

Then make the branch the command named. All the stage's work goes there.

Where the tests of a file live inside the file of the code (Rust unit tests of private items), the stage
needs a test file of its own and one line in the code file that joins it, such as
`#[cfg(test)] mod protect;`. That line changes the code. Commit it, with the empty test file, before the
plan, so that it is part of the base.

### Between: one test per task, then the check

The stage spec beside the two files says, per chosen function, what could change unseen there, and has one
task per test. The skill has templates for both. Then, for each task:

1. Write the test, in a test file only, and commit it by itself.
2. Ask `step pin` for each fault the task names. Exit 0: the test passes on the unchanged code and fails on
   the fault.

The first `step pin` of a stage runs the faults of the chosen functions on the unchanged code: the base
run. It is the long run, and `step protect begin` said how long it takes. Run it once before any test
exists, with `--fault 1`. Without a test the answer is exit 1, and that is expected there.

A fault that no test should chase is proposed for writing off, in `write-offs.json` beside the
declaration. There are two kinds: `nothing_a_caller_sees`, and `not_testable_as_it_stands`, a test cannot
reach it without a change of the code. A proposal has a reason a person can check in the code, and
`approved_by` and `approved_at` set to `null`. Only the owner of the code fills those two fields, and
commits the file.

Last, `step check` runs the faults of the chosen functions again with the new tests in place. Commit the
verdict it writes.

### End: `step protect result`

```sh
lesscode-core step protect result --declaration .kiro/specs/<name>/step.json [--out <file>]
```

It runs nothing. It reads the verdict of `step check`, `stage.json` and `write-offs.json` **as they are
committed on the branch, and no other copy**. A verdict or an approval that is written and not committed
does not count. A test file committed after the check makes the verdict one for other content: run the
check again.

| Exit | Meaning |
| --- | --- |
| 0 | every chosen function is protected enough |
| 1 | a fault waits, or the check refused because unnoticed faults rose; each is named |
| 2 | not judged: no committed verdict, a verdict for other content, a code file the branch changed, or a check that did not judge. Never a pass |
| 3 | bad input, such as a write-off for a fault the stage does not have, or of another kind |

Per chosen function it prints four numbers: noticed before, noticed now, written off, waiting. It names
every write-off with its reason, who approved it and when. For a fault written off as not testable as it
stands, it says what the function reaches out to: a reduction has to keep the behaviour there by other
means. On exit 0 it names the next step: declare the reduction of a chosen function as a step
([section 2](#2-declare-the-step)), on a branch of its own.

On exit 0 and 1 it writes `protection.json` beside the declaration (`step-protection-v1`): the four
numbers, the write-offs, the faults that wait, the base, the revision and the hash of the verdict. Commit
it with the stage. A later reader finds there what was protected, what was written off and why, and what
still waits, without the run.

### What the stage costs

- Time: one run of your project's commands, a build and the tests, for every planted fault that is run
  again. The faults of the chosen functions run twice, once in the base run and once in the check, and
  each pin runs the tests once on the unchanged code and once on its fault. `step protect begin` prints how many faults and about how long. On the
  renderer of R184 the base run took 17 min 50 s, each later pin between 90 and 143 s, and the whole
  stage 1 h 15 min.
- Disk: every copy is a full build. `step protect begin` prints what one copy takes; it leaves out
  what a project builds outside the copy. On the renderer eight copies took 34.5 GiB. Remove the copies
  as the stage goes.
- A person is needed three times: you name the functions, the owner of the code reviews once, and the owner
  merges the tests. Nothing else waits for a person.

### In a pipeline

Both commands open no page, start no listener and ask nothing; everything they need is in arguments and
files. Leave `--open` away from `step plan` and `step check`. Make `step protect result` the job that
gates: exit 0 passes, exit 1 means a fault waits, exit 2 is never a pass. Because the result reads
committed files only, a pipeline and a person at a checkout get the same answer for the same commit, and
an approval reaches the pipeline only by the owner's commit. A pipeline cannot name the functions, review
or merge. A run without the owner ends at exit 1 with the proposals named as waiting.

## 2. Declare the step

One file in the repository, committed on the step's branch, for example
`.kiro/specs/shape-checks/step.json`. This one is from a real step in a TypeScript project:

```json
{
  "step": "shape-checks", "spec": "shape-checks", "branch": "step/shape-checks",
  "base": "37a4e55f1b9397332acc6434783099a631a04918",
  "language": "typescript",
  "scope": ["src/state.ts"],
  "test_paths": ["tests/"],
  "setup": [["pnpm", "install", "--frozen-lockfile"]],
  "build": ["pnpm", "exec", "tsc", "--noEmit"],
  "test": ["node", "--import", "tsx", "--test", "tests/state.test.ts"],
  "run": {"timeout_floor_s": 30, "timeout_ceiling_s": 300, "exclusions": ["node_modules", "dist"],
          "runner_evidence_patterns": ["tests ", "pass ", "fail "]},
  "focus": {"src/state.ts": ["isPersistedWorkflowState"]},
  "max_token_growth": 30,
  "outputs": {"verdict": ".kiro/specs/shape-checks/verdict.json",
              "evidence": ".kiro/specs/shape-checks/evidence.json",
              "ledger": ".kiro/steps/ledger.jsonl"}
}
```

Only a few fields depend on the language. These are the ones the real steps used.

| `language` | `test` | Also needed |
| --- | --- | --- |
| `python` | `["uv", "run", "--no-sync", "python", "-m", "pytest", "tests/test_ratelimit.py", "-q", "-x"]` | `setup` `[["uv", "sync", "--quiet"]]`; in `run`: `"invalid_exit_codes": [2, 3, 4, 5]`, `"exclusions": [".venv", "__pycache__"]` |
| `typescript` | `["node", "--import", "tsx", "--test", "tests/state.test.ts"]` | `setup` and `build` as above; in `run`: `"exclusions": ["node_modules", "dist"]` |
| `java` | Maven, offline, one test class: `[..., "-o", "-B", "-Dtest=CSVPrinterTest", "test"]` | `setup` that compiles the tests once; in `run`: `"exclusions": ["target"]`, `"invalid_output_patterns": ["COMPILATION ERROR", ": error:"]` |
| `rust` | `["cargo", "nextest", "run", "--locked", "--manifest-path", "native/Cargo.toml", "--test", "recording"]` | `"env": {"CARGO_TARGET_DIR": "target"}`; in `run`: `"exclusions": ["target"]`; with a `--test` filter Cargo builds only the named test programs, so add `"build": ["cargo", "build", "--locked", "--manifest-path", "native/Cargo.toml", "--examples"]` when the tests start an example or another binary |

Commands are argument vectors and are not passed through a shell. On Windows that rules out `mvn`, which
is a script; the Java step called Maven's launcher class with `java` instead. The full declarations of
the six steps are in `docs/experiments/gradual-steps/steps/`.

Paths are relative to the repository root and commands run there, in copies of the whole repository.
`focus` names the functions the step is about; their faults are run even when the step does not change
them. `run` takes the other fields of a run configuration. Optional: `frame_spec`, `out_of_reach`
(functions left unprotected on purpose, each with a reason), `criteria` (commands tied to acceptance
criteria of the step's spec), `setup`, `build`.

## 3. Pin before you cut: `step pin`

```sh
lesscode-core step pin --declaration .kiro/specs/shape-checks/step.json --fault 4 --work ../work
```

Write a test, commit it on the branch by itself, and ask whether it pins an unnoticed fault of the base.
The tooling builds a copy with the code of the base and the test files of the branch, and runs the tests
with and without the fault.

| Exit | Result |
| --- | --- |
| 0 | admitted: the tests pass on the unchanged code and fail on the fault |
| 1 | rejected, with which happened: they fail on the unchanged code, or they pass on the fault |
| 2 | not judged |

A fault number that is not an unnoticed fault of the base is an error that lists the ones that are. The
output also lists the faults that remain unnoticed; none is required to be gone.

## 4. Judge the step: `step check`

```sh
lesscode-core step check --declaration .kiro/specs/shape-checks/step.json --work ../work
```

Checks that need no run come first; the runs follow only if those pass.

| Check | Refused when |
| --- | --- |
| changes stay in scope | a changed file is neither in scope, nor under a test path, nor the step's own (its spec folder, its outputs) |
| pins apart from the change | one commit changes a test file and a scope file together |
| size | the scope grows by more tokens than `max_token_growth` |
| state frame | a state write of the step is outside the declared frame specification |
| tests and fault runs | the project's tests do not pass on the base or on the step |
| planted faults | in a function, more faults go unnoticed after the step than before; the faults are named |
| evidence for criteria | a command tied to a criterion does not pass |

Faults run in the functions the step changes, in the functions of the scope that call them, and in the
functions in `focus`. A function that exists on one side only cannot be matched by name; the totals of the
file are then compared and the verdict says so.

Exit 0 accepted, 1 refused, 2 not judged (a run is void or incomplete, for example because a command could
not be started). The verdict (`step-verdict-v1`) is written to the declared path in all three cases. It
names the base and step commits and the hashes git has for the committed scope files, test files and
declaration. Work on disk that is not committed is an error (exit 3) and nothing is written: the judgement
would be about another state than the one you see.

The evidence file (`step-evidence-provisional-v0`) lists per criterion the command, its exit code and the
same hashes. Its format is provisional until it is agreed with openidea-kiro.

## 5. Put it on record: `step record`

```sh
lesscode-core step record --declaration .kiro/specs/shape-checks/step.json
lesscode-core step record --declaration .kiro/specs/shape-checks/step.json --override --reason "hotfix; pin follows in step 12"
lesscode-core step record --release 1.2.0 --ledger .kiro/steps/ledger.jsonl
```

The ledger is a file of JSON lines in the repository, only ever appended. Each entry carries the hash of
the line before it; if an earlier entry is edited or removed, every later use of the ledger stops with
"the ledger was rewritten".

- Without options, the accepted verdict is recorded with its numbers per function. A verdict that is not
  accepted, or that no longer matches the committed code, tests and declaration, is not recorded (exit 1).
- `--override` records that the step goes ahead without an accepted verdict. It needs a reason and shows
  up everywhere as "overridden", never as "accepted". It holds for exactly the committed content it was
  made for.
- `--release` records a release and lists the steps since the one before.

From then on `step check` holds each function to the best number an accepted step left on the ledger: a
later step may not leave more faults unnoticed than that, even if its own base got worse in between.

Commit the verdict, the evidence and the ledger on the branch. That moves the branch, not what was judged.

## 6. The boundary: `step guard` and the hook

```sh
lesscode-core step guard --branch step/shape-checks
lesscode-core step install-hook
```

The guard runs nothing. It reads the commit: is there a verdict that says "accepted" for exactly the scope
files, test files and declaration this commit has, or an override on the ledger for exactly that content?
If not, it exits 1 and says what changed. It judges what is committed; a verdict or an override that is
only on disk counts once it is committed. A branch that no declaration names is not a step branch and
passes.

`step install-hook` writes a pre-push hook that runs `lesscode-core step pre-push`: the complexity
budgets, the guard for every pushed branch, and, where `tools/bin/kiro-cli` is installed, the audit of the
step's spec. Any of them stops the push by exit code.

A local hook can be skipped with `git push --no-verify`. The boundary that cannot be skipped is the same
command in your pipeline, as a required check on the main branch:

```sh
lesscode-core step guard --branch "$BRANCH" --revision "$COMMIT"
```

Use a commit or a tag as `base` in the declaration. A branch name such as `main` moves, and a verdict is
written for one base commit.

## Several reports on one page: `step reports`

```sh
lesscode-core step reports plan.html check.html --out ../reports [--without-names-of my-project]
```

Takes reports that runs wrote with `--page` and writes a copy of each and `index.html`, one page that
lists them, into a new or empty directory. Each report is one block of the list, built the same way
whether the run was accepted or refused.

`--without-names-of <project>` writes numbers where that project's names stand, in the copies and in
the list. A project is named as its repository's directory is, for example `my-project` for
`D:\Source\my-project`. Left out are files, functions and types, the shapes of the state, the project's
commands, the files it wrote and the source line of the example fault. Kept are the branch, the
commits and the directory of the copies. Repeat the option for several projects.

For `step check`, `--measures <file>` puts numbers of the state and local-complexity lenses on the
page that were measured elsewhere.

### Reading the change itself

The report says whether a change kept what the tests protect. It does not help you read the diff. For
that we recommend [diffcore](https://github.com/jamesaphoenix/diff-core), a separate project (MIT): it
groups the changed files of a diff into ranked flows, from syntax trees and call graphs, so that a
large change is read in an order that makes sense. It has a command line, a desktop app and a VS Code
extension.

## Examples

Three steps taken on 2026-11-11, all accepted, recorded and passed by the guard
([R92](research/2026-11-11/18-three-real-steps.md); declarations, verdicts, evidence and ledgers in
`docs/experiments/gradual-steps/steps/`). The numbers are noticed / unnoticed faults in the function.

| Step | Change | Function before | after | Tokens of the file | Time |
| --- | --- | --- | --- | --- | --- |
| `slug-separator`, OpenUltraCode, TypeScript | `slugify` drops a condition that is always true | 9 / 1 | 8 / 0 | 1,283 to 1,277 | 4 min |
| `comment-start`, commons-csv, Java | `printComment` starts every comment line through one private method | 19 / 2 | 17 / 2, new method 2 / 0 | 1,420 to 1,415 | 25 min |
| `one-exit`, this repository, Rust | `verdict()` collects its reasons in one pass and returns once | 11 / 0 | 12 / 0 | 7,307 to 7,306 | 10 min |

The three ran at the same time on one machine. In the Java step the new method has no counterpart on the
base, so the totals of the file were compared as well. In the TypeScript step the fault that went unnoticed
before sat in the condition that could not be false; it is gone with the condition.

A fourth step was refused first ([R93](research/2026-11-11/19-refused-pinned-accepted.md)). In OpenUltraCode,
`isPersistedWorkflowState` checked six rules in two conditions; the step `shape-checks` writes one rule per
statement.

```text
$ lesscode-core step check --declaration .kiro/specs/shape-checks/step.json --work ../work
step shape-checks: refused
  src/state.ts: isPersistedWorkflowState: unnoticed faults rise from 1 to 7: #15 line 224 literal:false->true, ...
```

The tests had one case for a corrupt state file, and it was not valid JSON. No test gave the function a
file that parsed and had the wrong shape, so after the split six more single rules could be broken unseen.
Nine short tests, one misshapen file each, went into a commit of their own:

```text
$ lesscode-core step pin --declaration .kiro/specs/shape-checks/step.json --fault 4 --work ../work
#4 line 224 literal:false->true: admitted (the tests with the pin pass on the unchanged code and fail on the fault)
$ lesscode-core step check --declaration .kiro/specs/shape-checks/step.json --work ../work
step shape-checks: accepted
```

Noticed / unnoticed in the function: 3 / 1 before, 11 / 1 after. The one that remains swaps two checks that
can stand in either order. The file grew by 25 tokens, inside the 30 the step had declared.

A fifth step, in a Python project, shows why the pins come first
([R94](research/2026-11-12/01-first-python-step.md)). In x_likes_scraper, `_escape_html` replaced five
characters with five chained `str.replace` calls; the step `escape-html` calls `html.escape` instead. The
plan listed the function as "no fault noticed": no test called it with any of the five characters.

```text
$ lesscode-core step check --declaration .kiro/specs/escape-html/step.json --work ../work
step escape-html: accepted
```

That acceptance was wrong for the user. `html.escape` writes `&#x27;` for an apostrophe and the old code
wrote `&#39;`, so the export changed, and nothing noticed. With two tests that fix what the method returns,
committed first and admitted by `step pin`, the same change was refused ("the project's tests do not pass
on the step"). The change that keeps `&#39;` was accepted: 0 noticed / 15 unnoticed before, 3 / 0 after,
seven lines removed and two added. Where the plan says "no fault noticed", an acceptance tells you nothing
until the function is pinned.

A sixth step in the same project, `checkpoint-clear`, declared a frame specification and was accepted
([R95](research/2026-11-12/02-second-python-step-frame-release.md)). It was accepted before its pins as
well: the change adds the argument `missing_ok=True`, and the Python pack plants no fault in a keyword
argument. The test for a directory without files exists because a person thought of the case.

Two plans. For `src/state.ts` of OpenUltraCode, a TypeScript project, with the tests of
`tests/state.test.ts` (2026-11-11; receipt in
`docs/experiments/gradual-steps/steps/shape-checks/plan-state.json`): 17 entries, 45 faults noticed, 26
not. `update` leads with 4 unnoticed of 5, and `mutate` and `retryTransient` have no noticed fault at all.
`isPersistedWorkflowState` had 3 noticed and 1 unnoticed, few enough to pin, and became the step
`shape-checks` above.

For `native/src/recording.rs` of this repository, a Rust file, with the tests of `tests/recording.rs` only
(2026-11-11, 191 faults, 13.8 minutes with three copies; receipt in
`docs/experiments/gradual-steps/gate2/plan-recording.json`): 91 faults noticed, 100 not. Twelve functions have
no noticed fault, led by `replay_to_l2` (20 unnoticed) and `assemble_meta` (17); `verdict` has 11 noticed and
none unnoticed, which is why it was the first step. A function can look unprotected because the configured
test command does not run the tests that cover it; configure the tests you rely on.

## What no command checks

These could not be made mechanical. They stay with the person who takes or reviews the step; the
[agent skill](../.agents/skills/refactor-in-steps/SKILL.md) names them and does not decide them.

- An accepted step is not shown to behave the same. It passed the tests and lost no fault sensitivity in
  the functions that were run. Where a function is pure and its inputs are few, compare old and new on all
  of them yourself; the `verdict()` change was compared on 480 inputs in R89.
- A pin is admitted when it passes on the code and fails on the fault. Whether the behaviour it fixes is
  the one you want is your call.
- Nothing checks that a step is worth taking, small enough, or easier to read afterwards.
- The reason given for "out of reach" is recorded and never verified. The same goes for the reason of an
  override: it has to be there, and somebody has to read it.
- `step check` runs the commands a declaration ties to criteria. It does not check that the criteria say
  what the step is for, or that a criterion without a command is met. `kiro-cli check` audits the form of
  the spec, in the hook, where it is installed.
- Only unnoticed faults refuse a step. A function in which fewer faults are planted afterwards passes, so
  read the numbers in the verdict.
- `git push --no-verify` skips the local hook. The pipeline job is the boundary that holds.
- `step protect result` sees a name and a date in `approved_by` and `approved_at`. It cannot know that
  the owner of the code wrote them, so it prints every approver and date where a false one is seen. That
  the reason for a write-off is true is the owner's to judge.
- A stage's test is admitted when it fails on the fault. Whether the behaviour it fixes is one worth
  keeping is the review's, as for a pin.

## Limits

- Functions are matched by name. Two methods of one name in a file are counted together.
- A call is a name directly before an opening parenthesis. Calls through values are not seen.
- A whole file of a compiled language can take many minutes; the store under `--work/outcomes` makes a
  second plan with unchanged code, tests and configuration immediate.
