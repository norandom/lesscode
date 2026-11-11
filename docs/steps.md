# Refactoring in checked steps

Status: in development after release 1.1.1 (spec
[gradual-refactoring-steps](../.kiro/specs/gradual-refactoring-steps/design.md)). All commands
below exist. Three real steps have been taken with them from start to finish, one each in Rust, Java and
TypeScript ([examples](#examples)).

Why it works this way, for readers new to the idea: [testability.md](testability.md).

A step is one small change on one branch with one small spec. The tooling answers by exit code whether the
project's tests still pass after the step and whether they still notice planted faults in the changed code
at least as often as before. It writes no test and no refactoring; it judges what a person or a model wrote.

Exit codes of every `step` command: 0 accepted or done, 1 refused or rejected, 2 not judged because a run
is incomplete, 3 bad input or a void run.

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

What to do with a plan: pick a function with few unnoticed faults, write tests that turn them into noticed
ones (pins), and only then change the code.

## 2. Declare the step

One file in the repository, committed on the step's branch, for example `.kiro/specs/one-exit/step.json`:

```json
{
  "step": "one-exit", "spec": "one-exit", "branch": "step/one-exit", "base": "main",
  "language": "rust",
  "scope": ["native/src/recording.rs"],
  "test_paths": ["native/tests/"],
  "test": ["cargo", "nextest", "run", "--locked", "--manifest-path", "native/Cargo.toml"],
  "env": {"CARGO_TARGET_DIR": "target"},
  "run": {"timeout_floor_s": 180, "exclusions": ["target"]},
  "focus": {"native/src/recording.rs": ["verdict"]},
  "max_token_growth": 0,
  "outputs": {"verdict": ".kiro/specs/one-exit/verdict.json",
              "evidence": ".kiro/specs/one-exit/evidence.json",
              "ledger": ".kiro/steps/ledger.jsonl"}
}
```

Paths are relative to the repository root and commands run there, in copies of the whole repository.
`focus` names the functions the step is about; their faults are run even when the step does not change
them. `run` takes the other fields of a run configuration. Optional: `frame_spec`, `out_of_reach`
(functions left unprotected on purpose, each with a reason), `criteria` (commands tied to acceptance
criteria of the step's spec), `setup`, `build`.

## 3. Pin before you cut: `step pin`

```sh
lesscode-core step pin --declaration .kiro/specs/one-exit/step.json --fault 2 --work ../work
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
lesscode-core step check --declaration .kiro/specs/one-exit/step.json --work ../work
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
lesscode-core step record --declaration .kiro/specs/one-exit/step.json
lesscode-core step record --declaration .kiro/specs/one-exit/step.json --override --reason "hotfix; pin follows in step 12"
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
lesscode-core step guard --branch step/one-exit
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

## Examples

Three steps taken on 2026-11-11, all accepted, recorded and passed by the guard
([R92](research/2026-11-11/18-three-real-steps.md); declarations, verdicts, evidence and ledgers in
`docs/experiments/gradual-steps/steps/`). The numbers are noticed / unnoticed faults in the function.

| Step | Change | Function before | after | Tokens of the file | Time |
| --- | --- | --- | --- | --- | --- |
| `one-exit`, this repository, Rust | `verdict()` collects its reasons in one pass and returns once | 11 / 0 | 12 / 0 | 7,307 to 7,306 | 10 min |
| `comment-start`, commons-csv, Java | `printComment` starts every comment line through one private method | 19 / 2 | 17 / 2, new method 2 / 0 | 1,420 to 1,415 | 25 min |
| `slug-separator`, OpenUltraCode, TypeScript | `slugify` drops a condition that is always true | 9 / 1 | 8 / 0 | 1,283 to 1,277 | 4 min |

The three ran at the same time on one machine. In the Java step the new method has no counterpart on the
base, so the totals of the file were compared as well. In the TypeScript step the fault that went unnoticed
before sat in the condition that could not be false; it is gone with the condition.

A plan for `native/src/recording.rs` of this repository, with the tests of `tests/recording.rs` only
(2026-11-11, 191 faults, 13.8 minutes with three copies; receipt in
`docs/experiments/gradual-steps/gate2/plan-recording.json`): 91 faults noticed, 100 not. Twelve functions have
no noticed fault, led by `replay_to_l2` (20 unnoticed) and `assemble_meta` (17); `verdict` has 11 noticed and
none unnoticed, which is why it was the first step. A function can look unprotected because the configured
test command does not run the tests that cover it; configure the tests you rely on.

## What no command checks

These could not be made mechanical. They stay with the person who takes or reviews the step; the
[agent skill](../.agents/skills/refactor-in-steps/SKILL.md) names them and does not decide them.

- **Same behaviour.** An accepted step passed the tests and lost no fault sensitivity in the functions that
  were run. That is not a proof that it behaves the same. Where a function is pure and its inputs are few,
  compare old and new on all of them yourself (the `verdict()` change was compared on 480 inputs in R89).
- **What a pin asserts.** A pin is admitted when it passes on the code and fails on the fault. Whether the
  behaviour it fixes is the wanted one is not checked.
- **Whether the step is a good one.** Small enough, worth taking, easier to read: no check.
- **The reason for "out of reach".** It is recorded, not verified.
- **The reason for an override.** It must be there; nobody checks that it is a good one.
- **The spec.** `step check` runs the commands tied to criteria; whether the criteria say what the step is
  for, and whether a criterion without a command is met, is not checked. `kiro-cli check` audits the form of
  the spec, in the hook, where it is installed.
- **Fewer noticed faults.** Only unnoticed faults refuse a step. A function in which fewer faults are planted
  after the step passes; read the numbers in the verdict.
- **A skipped local hook.** `git push --no-verify` skips it. Only the pipeline job is a boundary.

## Limits

- Functions are matched by name. Two methods of one name in a file are counted together.
- A call is a name directly before an opening parenthesis. Calls through values are not seen.
- A whole file of a compiled language can take many minutes; the store under `--work/outcomes` makes a
  second plan with unchanged code, tests and configuration immediate.
