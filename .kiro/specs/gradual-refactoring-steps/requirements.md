# Requirements: gradual refactoring in checked steps

Generated 2026-11-11 (Europe/Berlin) from the [brief](brief.md) after its prototype gate passed
([R88](../../../docs/research/2026-11-11/14-gradual-steps-prototype-gate.md)). Approved by the owner on 2026-11-11.

## Introduction

A developer with a grown codebase can rarely afford one large refactoring. What they can do is change one
small piece at a time, on a branch, and release in between. This feature lets `lesscode-core` judge each
such step: after it, do the project's tests still pass, and do they still notice faults in the changed code
at least as often as before. It adds the two things a step needs before it is taken, tests that pin current
behaviour and a view of what stands in the way of testing a unit in isolation, and it produces a guard that
a hook or a pipeline enforces. Agent skills may guide the work; they are not the boundary. The boundary is
an exit code.

Terms: a *step* is one small change with its own spec, on its own branch. The *base* is the revision the
step starts from. A *fault* is a mutant of the driver. *Noticed* means killed by the project's tests,
*unnoticed* means survived. A *pin* is a test added to make a fault noticed without changing the code under
test.

## How it is driven (owner's question, 2026-11-11)

| Layer | What | Binding |
| --- | --- | --- |
| Commands | `lesscode-core step ...`: plan, pin, check, record | each exits non-zero when its check fails |
| Guard | a Git pre-push hook and a pipeline job that run `step check` for a step branch | hard: the push or the merge is refused |
| Spec | one cc-sdd spec per step under `.kiro/specs/<step>/`, audited by `kiro-cli` | hard where `kiro-cli` is in the same guard |
| Skills | agent skills that walk through the phases and call the commands | soft: they cannot accept a step |

## Requirements

### Requirement 1: A step and its scope
**Objective:** As a developer, I want a step to name what it touches, so that its checks are small and aimed.

#### Acceptance Criteria
1. The step tooling shall read a step declaration that names the base revision, the files in scope, the
   project's build and test commands, and the identifier of the step's spec.
2. When a step changes a file outside its declared scope, the step check shall report that file and refuse
   the step.
3. The step tooling shall work on any language that has an admitted pack and shall contain no rule for a
   particular language or project.
4. The step tooling shall contact no model and no network service.

### Requirement 2: Choosing the next step
**Objective:** As a developer, I want to see where a small step would help most, so that I do not have to
guess.

#### Acceptance Criteria
1. When a plan is requested for a set of files, the step tooling shall list, per function, the number of
   noticed and unnoticed faults, the state writes outside the declared frame, and the size in tokens.
2. The plan shall rank functions by unnoticed faults and shall mark functions in which no fault is noticed
   at all.
3. The plan shall be produced from fault runs in copies and shall not change the source tree.

### Requirement 3: Pin before you cut
**Objective:** As a developer, I want tests that fix what the code does today, so that a later change cannot
alter it unseen.

#### Acceptance Criteria
1. When a pin is offered for a named unnoticed fault, the step tooling shall admit it only if the project's
   tests, with the pin, pass on the unchanged code and fail on that fault.
2. If a pin passes on the fault or fails on the unchanged code, then the step tooling shall reject it and say
   which of the two happened.
3. When pins are admitted, the step tooling shall report which unnoticed faults of the scope remain and
   shall not require that none remains.
4. The step check shall refuse a step whose pins and whose change to the code under test are in the same
   commit, so that pins are proven against the code as it was.
5. The step tooling shall not write pins itself; it shall judge pins written by a person or a model.

### Requirement 4: Seams
**Objective:** As a developer, I want to know why a function cannot be tested in isolation, so that I can
fake what stands in the way or leave the function out of the step.

#### Acceptance Criteria
1. When a function of the scope has no noticed fault, the step tooling shall list the calls and state writes
   in it that reach outside the function, as the pack identifies them.
2. The step declaration shall allow a function to be marked as out of reach with a reason, and the step
   check shall then report that function as unprotected instead of refusing the step for it.
3. The step tooling shall not rewrite or intercept calls at run time; isolation is done with the means of
   the project's language and test framework.

### Requirement 5: The step verdict
**Objective:** As a developer, I want one answer per step, so that I know whether the code is still working
and still protected.

#### Acceptance Criteria
1. When a step check is requested, the step tooling shall run the project's tests on the base and on the
   step, each in copies, and shall refuse the step if either run is void or the tests do not pass.
2. The step check shall run faults in the functions the step changes and in the functions of the scope that
   call them, and shall say which functions it ran.
3. If in any function the number of unnoticed faults is higher after the step than before, then the step
   check shall refuse the step and name the function and the faults.
4. Where a function of the base has no counterpart of the same name after the step, the step check shall
   compare the totals of the file and shall say that functions could not be matched.
5. The step check shall report the size of the scope in tokens and nodes before and after, and shall refuse
   a step that grows beyond what its declaration allows.
6. Where a frame specification is declared, the step check shall refuse a step that writes state outside it.
7. The step check shall write a verdict that names the base and step revisions, the hashes of the files in
   scope and of the test files, every check with its result, and the reasons for a refusal.
8. The step check shall exit with a code that distinguishes accepted, refused, and not judged (void run or
   bad input).

### Requirement 6: The guard
**Objective:** As a maintainer, I want a boundary that an agent or a hurried person cannot talk their way
past, so that an unchecked step does not reach the main branch.

#### Acceptance Criteria
1. The step tooling shall provide a guard command that accepts a step branch only if a verdict exists whose
   revisions and file hashes match the branch as it is, and whose result is accepted.
2. When the code, the tests or the declaration change after a verdict was written, the guard shall treat the
   verdict as stale and refuse.
3. The step tooling shall install the guard as a Git pre-push hook on request and shall document its use in
   a pipeline; both shall refuse by exit code.
4. Where an override is used, the guard shall require a written reason, record it in the ledger, and report
   the step as overridden, never as accepted.
5. The guard shall not depend on any agent skill, prompt or model.

### Requirement 7: Ledger and ratchet
**Objective:** As a maintainer, I want the series of steps to be visible release by release, so that
progress and regressions are on record.

#### Acceptance Criteria
1. When a step is accepted or overridden, the step tooling shall append an entry to a ledger file in the
   repository with the step, its revisions, its verdict and its numbers before and after.
2. The ledger shall keep, per function touched, the last accepted number of unnoticed faults; a later step
   shall be compared with that number as well as with its own base.
3. When a release is recorded, the step tooling shall list the steps since the previous release from the
   ledger.
4. The step tooling shall never rewrite an earlier ledger entry.

### Requirement 8: Evidence for the specification tools
**Objective:** As a user of cc-sdd and openidea-kiro, I want the verdict to count as executed evidence for
the step's spec, so that the two tools cover the suite together.

#### Acceptance Criteria
1. The step declaration shall allow tests to be tied to acceptance criteria of the step's spec.
2. When a verdict is written, the step tooling shall also write, per tied criterion, which tests ran, whether
   they passed, and on which revisions and file hashes.
3. The format of that evidence shall be agreed with openidea-kiro before it is fixed; until then it is
   marked as provisional in the output.
4. The step tooling shall not judge the consistency or traceability of a spec; that remains with `kiro-cli`.

### Requirement 9: Stored outcomes and tests
**Objective:** As a developer, I want a rerun after a test change to be a real rerun, so that a pin is never
judged by a stale result.

#### Acceptance Criteria
1. The driver shall make the identity of a stored outcome depend on the test files named in the run
   configuration, as well as on the source file, the mutated text and the configuration.
2. When test files change, the driver shall run the affected mutants again without being asked to.

### Requirement 10: Protection of the examined code
**Objective:** As the owner, I want the rule of the driver to hold here too: breaking codebases must be
avoided.

#### Acceptance Criteria
1. The step tooling shall read the base and the step from the repository and shall run everything in copies
   outside it.
2. The step tooling shall not create, switch or delete branches, and shall not commit, push or merge.
3. The only files the step tooling writes inside the repository shall be the verdict, the evidence and the
   ledger, at the places the declaration names.

### Requirement 11: Evidence that it works
**Objective:** As the owner, I want the feature shown on real steps before it is relied on.

#### Acceptance Criteria
1. Before production tasks, the workflow shall take one real step on a copy of this repository by hand (pins
   first, then the change), check what it breaks, and record the result in a dated research entry.
2. When the feature is delivered, the workflow shall take at least one step in a foreign project in each of
   two languages and record the verdicts, including any refusal.
3. When a result is recorded, the workflow shall add a dated research entry and update the research index.
4. If a check of this feature cannot be made mechanical, then the workflow shall record that and shall not
   replace it by a judgement in a skill.

## Not in scope

- Writing the refactoring or the pins. A person or a model does that.
- A symbolic executor or a run-time detouring framework.
- Equivalence proofs. The verdict says what the tests and the planted faults show.
- Changes to openidea-kiro; the evidence format is proposed to it, not imposed.
