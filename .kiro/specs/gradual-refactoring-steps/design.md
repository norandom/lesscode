# Design: gradual refactoring in checked steps

Generated 2026-11-11 (Europe/Berlin) from the approved [requirements](requirements.md). Status: **approved by the owner
on 2026-11-11**, with the decisions under "Owner decisions". Evidence so far: [R88](../../../docs/research/2026-11-11/14-gradual-steps-prototype-gate.md)
(prototype gate), [R89](../../../docs/research/2026-11-11/15-first-tracked-step.md) (first real step by hand),
[R90](../../../docs/research/2026-11-11/16-budgets-count-tokens.md) (budgets in tokens).

## Overview

`lesscode-core` gets a command group `step`. A step is one small change on one branch with one small spec.
The commands answer, by exit code, whether the tests still pass after the step and whether they still notice
planted faults in the changed code at least as often as before. A guard command, cheap enough for a hook,
accepts a branch only if a matching accepted verdict exists.

Nothing here writes a refactoring or a test. The tooling judges what a person or a model wrote.

### Goals

- One verdict per step, bound to revisions and to hashes of code, tests and declaration.
- Pins judged mechanically: a pin counts only if it passes on today's code and fails on a named fault.
- A guard that is an exit code, usable as a pre-push hook and in a pipeline, without any skill or model.
- A ledger that shows the series of steps and holds the best number reached per function.
- Runs short enough to use: only the functions a step touches and their callers in scope.

### Non-Goals

- Generating tests, pins or refactorings. No symbolic execution, no run-time detouring.
- Equivalence proofs. A differential check for pure units was discussed and is not in the requirements; it
  is left out (owner decision, below).
- Judging a spec's consistency or traceability; that is `kiro-cli`.
- Replacing the complexity budgets; they stay a separate check.

## Boundary Commitments

### This Spec Owns

- The step declaration, the commands `step plan | pin | check | guard | record | install-hook`.
- The verdict, the evidence file and the ledger, and their formats.
- Two changes in the driver: the outcome store depends on the test files (Requirement 9), and a run can be
  limited to named functions.

### Out of Boundary

- The driver's packs, operators, classes and safety rules; they are used as they are.
- openidea-kiro and `kiro-cli`; the evidence format is proposed to them and marked provisional until agreed.
- Branch handling. The tooling never creates, switches or deletes a branch and never commits, pushes or
  merges (10.2, 10.5).

### Allowed Dependencies

- The `git` executable, read-only: `rev-parse`, `archive`, `log`, `diff --name-only`, `show`. Already
  required by `deconvolute install-hook`.
- The driver's library functions (`analysis`, `sites`, `runner`, `store`, `measure`, `workspace`).
- No new crate. No network, no model (1.4).

### Revalidation Triggers

- A change to the driver's run receipt, classes or store key.
- A change to what the pack calls a function, a call or a state write.
- Agreement on the evidence format with openidea-kiro (replaces the provisional one).

## Architecture

### Existing Architecture Analysis

`deconvolute run` takes a run configuration with a source root, files, test paths and commands; it copies
the root, mutates in the copies and classes each mutant. Its receipt lists mutants with their site (file and
position) but not their function; `deconvolute analyze` lists function boundaries. The prototype
(`docs/experiments/gradual-steps/step_verdict.py`) joined the two and compared two runs per function. That
join and comparison move into the product. The store keys outcomes by original text, mutated text and
configuration identity, not by the tests: after a pin is added a stored "survived" is reused. This is the
hazard written down in `docs/driver.md` and it is why Requirement 9 exists.

### Architecture Pattern & Boundary Map

```
 .kiro/specs/<step>/step.json          declaration (in the repository, read only)
            |
            v
   step commands (native/src/step/)  -- git archive -->  copies outside the repository
            |                                              base/      step/      pin/
            |  uses                                          |          |          |
            v                                                v          v          v
   driver: analysis, sites, runner, store, measure   <--  tests and fault runs in the copies
            |
            v
   verdict.json (+ .sha256)   evidence.json   ledger.jsonl      the only writes in the repository
            |
            v
   step guard  (hook, pipeline): recomputes revisions and hashes, compares, exits
```

The step code sits beside the driver, not inside `adapters/driver/`, and like it names no language (1.5; the
existing name scan is extended to `native/src/step/`).

### Technology Stack

Rust, the crates already locked. JSON via `serde_json`, receipts via `receipt::write_receipt` with a
SHA-256 sidecar.

## File Structure Plan

### Directory Structure

```
native/src/step/
  mod.rs          command dispatch, exit codes
  declaration.rs  step.json: read and validate
  revisions.rs    git: resolve revisions, list commits and changed files, archive into copies
  functions.rs    functions per file from the pack; which changed; which call them
  plan.rs         step plan
  pin.rs          step pin
  check.rs        step check: the checks in order, the verdict
  verdict.rs      verdict and evidence: write, read, hashes
  guard.rs        step guard, step install-hook
  ledger.rs       append, chain, ratchet, release listing
native/tests/step_*.rs
docs/steps.md     user guide, pipeline example
```

Every file is listed in `native/complexity-budgets.json` with the default ceilings (2,800 tokens, 300 units,
560 tokens per function).

### Modified Files

- `native/src/adapters/driver/store.rs`, `runner.rs`, `config.rs`: test-file identity in the store key;
  optional list of functions to run.
- `native/src/cli.rs` or `main.rs`: the `step` group.
- `docs/driver.md`: the hazard note is replaced by the new behaviour.
- `README.md`: one section; `.agents/skills/`: a skill that walks the phases and calls the commands (soft
  layer, last).

## System Flows

### A step from start to merge

1. `step plan --files F...` on the base: per function, noticed and unnoticed faults, writes outside the
   frame, tokens. Ranked by unnoticed faults; functions with no noticed fault are marked (2.1 to 2.5).
2. The developer writes the step's spec and `step.json`, creates the branch.
3. Pins: write a test, `step pin --fault N`. Admitted only if it passes on the base code and fails on that
   fault (3.1, 3.2). Commit the pins by themselves.
4. The change: commit separately (3.4).
5. `step check`: writes the verdict. Exit 0 accepted, 1 refused, 2 or 3 not judged.
6. `step record`: appends the ledger entry for an accepted verdict.
7. Push. The hook runs `step guard`; the pipeline runs it again.

### step check, in order

Each check appears in the verdict with its result; the first group stops the check as "not judged".

| # | Check | Result when it fails |
| --- | --- | --- |
| 1 | Declaration valid; base and step revisions resolve; step revision is a commit, the scope has no uncommitted change | not judged (exit 3) |
| 2 | Files changed between base and step are inside the scope or are test paths or the step's spec | refused (1.2) |
| 3 | No commit between base and step changes both a test path and a scope file | refused (3.4) |
| 4 | Tests on the base copy and on the step copy: run not void, tests pass | void: not judged (exit 2); tests fail: refused (5.1, 5.9) |
| 5 | Functions to run: changed ones and their callers in scope; listed in the verdict | (5.2, 5.10) |
| 6 | Fault runs on base and step, limited to those functions | void: not judged |
| 7 | Per function, unnoticed after is not higher than before, nor higher than the ledger's last accepted number | refused, naming function and faults (5.3, 7.5) |
| 8 | Functions without a counterpart: totals of the file compared, and said so | refused if the total rises (5.4, 5.11) |
| 9 | Tokens and nodes of the scope before and after; growth within `max_token_growth` | refused (5.5, 5.12) |
| 10 | Where a frame specification is declared: no state write outside it in the step | refused (5.6) |
| 11 | Evidence: the commands tied to criteria, run in the step copy | recorded; a failing tie refuses |

### step guard

Reads the declaration for the branch being pushed, recomputes the base and step revisions and the hashes of
the scope files, the test files and the declaration, and compares them with the verdict. No copy, no run.

- Verdict present, hashes equal, result accepted: exit 0.
- No verdict, or any hash or revision differs: stale, exit 1 (6.1, 6.2).
- A ledger entry "overridden" for exactly this revision with a reason: exit 0, printed as overridden (6.4).
- A branch for which no declaration names it is not a step branch; the guard says so and exits 0. Whether
  the main branch accepts only step branches is a branch-protection setting, not something a local tool can
  hold.

## Requirements Traceability

| Requirement | Where |
| --- | --- |
| 1.1 | `declaration.rs` |
| 1.2 | `check.rs` check 2, `revisions.rs` |
| 1.3, 1.5 | packs via the driver registry; name scan over `native/src/step/` |
| 1.4 | no network or model code; test that the step crate paths import none |
| 2.1 to 2.5 | `plan.rs`, `functions.rs`; runs in copies only |
| 3.1, 3.2, 3.7 | `pin.rs` |
| 3.3, 3.6 | `pin.rs` report of remaining unnoticed faults; no threshold |
| 3.4 | `check.rs` check 3 |
| 3.5 | no code path writes a test file; covered by the protection test (10.3) |
| 4.1 | `functions.rs` (calls and state writes that leave a function) in `plan.rs` output |
| 4.2, 4.4 | `declaration.rs` `out_of_reach`; `check.rs` reports "unprotected" |
| 4.3 | nothing intercepts calls; stated in `docs/steps.md` |
| 5.1 to 5.12 | `check.rs`, `verdict.rs` |
| 6.1, 6.2, 6.5, 6.7 | `guard.rs` |
| 6.3 | `guard.rs` `install-hook` |
| 6.4 | `ledger.rs` override entry, `guard.rs` |
| 6.6 | `docs/steps.md` |
| 7.1 to 7.5 | `ledger.rs`, `check.rs` check 7 |
| 8.1, 8.2 | `declaration.rs` `criteria`, `verdict.rs` evidence |
| 8.3 | evidence schema name `step-evidence-provisional-v0` |
| 8.4 | no spec parsing beyond the criterion identifiers as strings |
| 9.1, 9.2 | driver `store.rs`, `runner.rs` |
| 10.1, 10.4 | `revisions.rs` (`git archive` into the work directory) |
| 10.2, 10.5 | only the read-only git commands above; test that lists the git arguments used |
| 10.3 | writes limited to the three declared paths; test compares the repository before and after |
| 11.1 | done: R89 |
| 11.2 to 11.5 | final tasks: two foreign projects, research entries |

## Components and Interfaces

| Component | Domain/Layer | Intent | Req Coverage | Files |
| --- | --- | --- | --- | --- |
| Declaration | step | Read and validate `step.json` | 1.1, 4.2, 8.1 | native/src/step/declaration.rs |
| Revisions | step | Resolve revisions, list commits, copies from `git archive` | 1.2, 10.1, 10.2, 10.4, 10.5 | native/src/step/revisions.rs |
| Functions | step | Changed functions, callers, faults per function, seams | 4.1, 5.2, 5.10 | native/src/step/functions.rs |
| DriverChanges | driver | Store key with test files; function filter | 9.1, 9.2, 5.2 | native/src/adapters/driver/store.rs, native/src/adapters/driver/runner.rs, native/src/adapters/driver/config.rs |
| Plan | step | `step plan` | 2.1 to 2.5 | native/src/step/plan.rs |
| Pin | step | `step pin` | 3.1 to 3.7 | native/src/step/pin.rs |
| Check | step | `step check` | 5.1 to 5.12, 1.2, 3.4, 4.4, 7.5 | native/src/step/check.rs |
| Verdict | step | Verdict and evidence | 5.7, 5.8, 8.2, 8.3 | native/src/step/verdict.rs |
| Ledger | step | Append, chain, ratchet, release, override | 7.1 to 7.5, 6.4 | native/src/step/ledger.rs |
| Guard | step | `step guard`, hook, pipeline guide | 6.1 to 6.7 | native/src/step/guard.rs, docs/steps.md |
| CommandLine | step | Dispatch and exit codes | 5.8 | native/src/step/mod.rs |

### Declaration

`.kiro/specs/<step>/step.json`:

```json
{
  "step": "verdict-one-exit",
  "spec": "verdict-one-exit",
  "branch": "step/verdict-one-exit",
  "base": "ad75c0f",
  "language": "rust",
  "root": "native",
  "scope": ["src/recording.rs"],
  "test_paths": ["tests/"],
  "test": ["cargo", "nextest", "run", "--locked", "--no-fail-fast"],
  "env": {"CARGO_TARGET_DIR": "target"},
  "run": {"timeout_floor_s": 180, "workers": 2},
  "max_token_growth": 0,
  "frame_spec": null,
  "out_of_reach": [{"function": "probe", "reason": "starts a process; no seam yet"}],
  "criteria": [{"id": "1.1", "command": ["cargo", "nextest", "run", "--test", "recording", "verdict_"]}],
  "outputs": {"verdict": ".kiro/specs/verdict-one-exit/verdict.json",
              "evidence": ".kiro/specs/verdict-one-exit/evidence.json",
              "ledger": ".kiro/steps/ledger.jsonl"}
}
```

`run` carries the driver's run-configuration fields unchanged (patterns, timeouts, exclusions). `base` must
be an ancestor of the step revision. Unknown fields are an error.

### Revisions and copies

`git archive <revision> <root>` is unpacked into `<work>/base` and `<work>/step`; `--work` is required and
must lie outside the repository (checked on resolved paths; a drive root is rejected). The step revision is
`HEAD` of the declared branch as `git rev-parse` gives it; the tooling does not check it out. Because the
copies come from commits, an uncommitted change in scope would be judged wrongly, hence check 1.

### Functions

From the pack's function boundaries (as `deconvolute analyze` reports them) per file and revision:

- *changed*: a function whose token sequence differs between base and step, or that exists on one side only;
- *callers in scope*: functions in the scope files that contain a call site whose callee name equals a
  changed function's name. This is a match by name, not a call graph; it can include a function too many and
  cannot see calls through values. The verdict lists what was run so that the limit is visible (5.10).

A fault belongs to the function whose boundaries contain its site; faults outside any function are counted
under the file.

### Driver changes

- **Store key (9.1, 9.2):** the configuration identity gains a hash over the files under `test_paths`
  (relative path and content, sorted). A changed test file gives new keys, so affected mutants run again
  without `--no-reuse`. Stores written before this change simply no longer match.
- **Function filter:** the run configuration accepts `functions` per file; sites outside them are not
  mutated and are counted as "not selected" in the receipt. Without the field nothing changes.

### Pin

`step pin --declaration D --fault N --work W`: `N` is the mutant number in the base fault run of the plan or
of a named receipt. The pin copy is the base revision of the scope plus the test paths of the step revision.
Tests run twice, on that copy and on it with fault `N` applied. Results: admitted; rejected because it fails
on the unchanged code; rejected because it passes on the fault (3.2). The output also lists the unnoticed
faults of the scope that remain (3.3).

### Verdict and evidence

`step-verdict-v1`: step, base and step revisions, `hashes` (scope files, test files, declaration), `checks`
(name, result, detail), `functions_run`, per function `before` and `after` (noticed, unnoticed, other
classes), `unmatched_functions`, `size` (tokens, nodes; before, after), `result`
(`accepted | refused | not_judged`), `reasons`. Written with a SHA-256 sidecar.

`step-evidence-provisional-v0`: per tied criterion the command, exit code, pass or fail, revisions and
hashes. The schema name carries "provisional" until openidea-kiro agrees a format (8.3).

### Ledger

`.kiro/steps/ledger.jsonl`, one JSON object per line, appended only. Each entry: step, date, base and
revision, result (`accepted | overridden | release`), numbers before and after per function, the verdict's
hash, and `previous`: the SHA-256 of the line before. `step guard` and `step record` verify the chain; a
rewritten earlier entry breaks it and is reported (7.4). The ratchet reads, per file and function, the
`after.unnoticed` of the last accepted entry (7.2). `step record --release NAME` appends a release entry and
prints the steps since the previous one (7.3). `step record --override --reason TEXT` appends an overridden
entry for the current revision; an empty reason is an error (6.4).

The two entries in `docs/experiments/gradual-steps/ledger.jsonl` stay where they are as research records;
the product ledger starts empty.

### Guard and hook

`step install-hook` adds a call to `lesscode-core step guard --pushed` to the pre-push hook, beside the
budgets call that `deconvolute install-hook` writes; it does not replace it. If `tools/bin/kiro-cli` exists
the hook also runs `kiro-cli check --feature <spec>`; the guard itself never calls it (6.5, 8.4).
`docs/steps.md` shows the same two commands as a pipeline job.

### Command line and exit codes

`lesscode-core step <plan|pin|check|guard|record|install-hook>`. Exit codes follow the driver: 0 accepted or
done, 1 refused or rejected, 2 not judged because a run is incomplete or void, 3 bad input.

## Data Models

Three JSON documents (declaration, verdict, evidence) and one JSON-lines file (ledger), as above. All read
with unknown fields rejected. Hashes are SHA-256 over file bytes; a set of files is hashed as sorted lines
of "path, hash".

## Error Handling

- A void run is never read as a refusal or an acceptance; the verdict says `not_judged` with the driver's
  void reason.
- Git failing, a revision that does not resolve, a work directory inside the repository: exit 3, nothing
  written.
- A verdict is written for accepted, refused and not judged alike, so that a refusal is on record.
- An interrupted check leaves no verdict; the driver's stop handler restores the copies.

## Testing Strategy

- Unit: declaration validation; function change detection; caller match; ledger chain and ratchet; hashes.
- Integration, on small fixture repositories created in a temporary directory by the tests: accepted step;
  refusal by each of checks 2, 3, 4, 7, 8, 9, 10; pin admitted and both rejections; guard accepts, goes
  stale on a change to code, to a test and to the declaration; override; release listing.
- Protection: the repository tree and its refs before and after every command differ only in the three
  declared outputs; the list of git arguments used contains no writing command.
- Driver: a changed test file leads to a rerun; the function filter mutates only inside the named functions.
- Real steps (11.2): one in a foreign project in each of two languages, recorded with their verdicts.

## Performance & Scalability

A whole-file fault run on `recording.rs` took about 20 minutes (R88). The function filter is what makes a
step check usable: `verdict()` has 11 faults against 191 in the file. The check runs the tests four times at
least (base, step, and the two fault runs); the store carries base results from the plan to the check as
long as code, tests and configuration are unchanged.

## Migration Strategy

Nothing existing changes behaviour except the store key; old stores are ignored, not deleted. The prototype
script stays under `docs/experiments/` as the record of R88.

## Prototype gate

- Status: `proceed` for the demonstrated subset. R88: a step judged from two fault runs per function; pins
  turn a refusal into an acceptance; shown on Rust, with verdicts on Java and TypeScript. R89: one real step
  by hand, pins accepted, change refused by a hard boundary.
- Not yet shown by any run, and therefore the first tasks, each small and able to fail:
  1. the function filter gives the same classes for the selected faults as a whole-file run;
  2. copies from `git archive` build and test like a working tree (line endings, ignored files, submodules);
  3. single-fault pin admission through the product path.
  If one fails, the design is revised before the remaining tasks.

## Owner decisions, 2026-11-11

1. **Callers:** changed functions plus their callers in scope, matched by name.
2. **Override:** kept as required (6.4); it is the one way past the guard and leaves a ledger entry.
3. **Differential check for pure units:** left out for now.
4. **The change refused in R89:** kept for the finished tooling, as its first step (task 6.1).
5. No release for the token budgets was asked for.

## Amendments

### Amendment 1, 2026-11-11 (task group 1, [R91](../../../docs/research/2026-11-11/17-gradual-steps-three-assumptions.md))

- **Whole repository.** The copies of base and step hold the whole repository, not a `root` directory: a
  copy of `native/` alone failed 5 of 363 tests that read files elsewhere. The declaration has no `root`;
  its paths are relative to the repository root and its commands run there.
- **Hashes of committed content.** The verdict and the guard hash the blobs of the revisions, not files on
  disk, because working trees can hold converted line endings.
- **No `git archive`.** Trees are read with `git ls-tree` and `git cat-file --batch`; the allowed git
  commands are `rev-parse`, `merge-base`, `ls-tree`, `cat-file` and `log`.
- **Driver options as built:** `functions` in the run configuration and `deconvolute run --only N`; the
  receipt gains `sites_not_selected` and `mutants_not_selected` when they are used.
- **Prototype gate:** the three points listed above as not yet shown are shown; status `proceed`.

### Amendment 2, 2026-11-11 (task group 3): as built

- **Declaration.** No `root` (Amendment 1). Fields as in the example, plus `setup` and `build`; a key of
  `run` that the declaration sets itself is an error.
- **Function names.** A function takes the `name` field of its node or of the node it is bound to; one
  without either is counted with the function around it.
- **Callers.** A call is a name token directly before an opening parenthesis (design: "callee name"). No
  pack field is needed for it.
- **`step plan`** takes a run configuration (`--config`), not a declaration, because a plan is made before
  a step is declared. Receipt `step-plan-v1`.
- **Binary.** `main.rs` was at its ceiling; its path helpers moved to `cli_paths.rs` and its ceilings were
  lowered to the measured size.

### Amendment 3, 2026-11-11 (task group 4): as built

- **`focus`.** New declaration field: per file, functions the step is about. A step that only adds pins
  changes no function; without `focus` no fault would be run for it. `step pin` runs the faults of `focus`.
- **Order of checks.** The checks without a run (scope, commits, size, frame) come first, and the runs are
  skipped when one of them refuses. A refusal is cheap that way; the verdict then lacks the run checks.
- **Void runs.** Requirement 5.9 says a void run refuses the step; the verdict says `not_judged` (exit 2),
  as Requirement 5.8 and the design's table have it. For the branch the effect is the same: the guard accepts
  only `accepted`.
- **Tests fail or command cannot start.** The driver now reports a command that cannot be started with its
  own reason. Failing tests refuse the step; a command that cannot start is `not_judged`.
- **Hashes.** The verdict holds the hashes git has for the committed scope files and declaration, and one
  SHA-256 over path and git hash of the files under the test paths.
- **No sidecar.** Requirement 10.3 allows three files in the repository; a `.sha256` beside the verdict
  would be a fourth. The ledger will carry the verdict's hash.
- **Uncommitted work** in the declaration, the scope or the tests is bad input (exit 3), nothing is written.
- **Ledger number (7.5)** is compared in task 5.1, when the ledger exists.

### Amendment 4, 2026-11-11 (task group 5): as built

- **Hook.** The hook is one line, `exec lesscode-core step pre-push`. The shell version in this design read
  the pushed refs with a subshell, which failed on this Windows machine. `step pre-push` runs the budgets
  check, the guard and, where `tools/bin/kiro-cli` exists, `kiro-cli check --feature <spec>`. The guard
  itself still calls nothing.
- **Committed first.** The guard reads verdict and ledger from the commit it judges and falls back to the
  files on disk only when the commit has none. An override therefore counts once the ledger is committed.
- **Moved branch.** The guard accepts a later commit of the branch when the hashes of scope, tests and
  declaration are those of the verdict; committing verdict, evidence and ledger does not make it stale.
- **Base.** A verdict is written for one base commit. If the declared base is a branch that moved, the
  guard refuses; declarations should name a commit or a tag.
- **Release.** `step record --release NAME --ledger PATH` needs no declaration.
- **Ledger entry.** `result` is `accepted`, `overridden` or `release`; an accepted entry holds the
  verdict's SHA-256, the hashes, and per function the numbers before and after.

### Amendment 5, 2026-11-11 (owner decision): the evidence format aims at openidea-kiro's `RunRecord`

- openidea-kiro is the owner's co-project. Its spec `verification-evidence` (design approved, not
  implemented) defines the execution record `RunRecord` and `breadth accept`. lesscode proposes no format
  of its own; `RunRecord` is the target and `step-evidence-provisional-v0` stays until `breadth accept`
  exists to test against.
- The gap that matters: `RunRecord` names every test with its result, and exit code 0 alone is not
  acceptance there. lesscode records the command and its exit code. Closing it needs per-test results from
  the project's runner, through the pack or the run configuration; not designed here.
- Comparison field by field: [evidence-alignment.md](evidence-alignment.md).

## Contradictions and disposition

| Conflict | Authority | Disposition |
| --- | --- | --- |
| Driver design says the store key has three parts; this spec adds the tests | Requirement 9, approved | Amendment to the driver design when the task is done; `docs/driver.md` updated |
| R89 says the change is refused; R90 says it would be within budget | Both are true at their time | R89 amended with a dated note; change not applied |
| Requirement 6 wants an unbypassable guard; a local hook can be skipped with a flag | Git | The pipeline job is the hard boundary; the hook is the early one. Stated in `docs/steps.md` |
| Evidence format is not agreed with openidea-kiro | Requirement 8.3 | Provisional name; no change to openidea-kiro. Target is its `RunRecord` (Amendment 5) |
