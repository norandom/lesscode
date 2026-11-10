# Requirements Document

## Introduction

Lesscode needs to measure how well a project's tests detect small faults, and to list where code writes state,
on any code in its target languages. Today `lesscode-core` does this only for the files its operators were
written for: the mutation operators replace fixed substrings, the analyzers match lines of text, and Python has
neither ([R55](../../../docs/research/2026-11-10/01-released-binary-adopter-dogfood.md)). Since release 1.0.2
the affected commands say so instead of reporting success.

A prototype showed a replacement that does not depend on the code in front of it: one engine driven by a small
data pack per language, with packs that a model can write offline and deterministic checks admit
([R64](../../../docs/research/2026-11-10/10-annotation-driver-gate-attempt2.md),
[feasibility report](../../../docs/reports/agnostic-driver-feasibility.md)). The owner decided on 2026-11-10
that this is built into `lesscode-core` in Rust as the only runtime, that it covers mutation and analysis
together, and that it must never damage a codebase.

These requirements describe what a maintainer, an adopter and a continuing researcher can observe of that
driver. Criterion identifiers use section.item notation, for example 3.2.

## Boundary Context

- **In scope**: language packs and their admission; discovery of sites and the analysis receipt for Rust,
  Python, TypeScript and Java; generation of mutants; running mutants against a project's own tests and
  classifying the outcome; protection of the examined code; validation of externally supplied site
  annotations; removal of the specimen-bound operators and line-based analyzers; evidence that the native
  driver reproduces the prototype's gate results.
- **Out of scope**: calling a model from the binary; writing packs inside the binary; prompt optimisation and
  hardening against model drift (second priority, in
  [constraint-guided-compression](../constraint-guided-compression/requirements.md) Slice B); regenerating or
  compressing code; equivalence checking; density measures; the pre-push hook; languages other than the four
  named, beyond admitting a pack for them.
- **Adjacent expectations**: the project under examination supplies its own test command and a working build;
  the driver does not install dependencies. The repository's complexity budgets and the apparatus cap of 7,173
  canonical lines apply; the prototype-first rule requires the sizing result of Requirement 8 before production
  tasks are generated. The existing `prepare`, `compare`, `admission`, `surface`, `regenerate`, `density` and
  `budgets` commands keep their behaviour.

## Requirements

### Requirement 1: Language packs
**Objective:** As a maintainer, I want everything language-specific to be a small data file that is checked
before use, so that adding a language is not a code change and a wrong pack cannot produce wrong results
silently.

#### Acceptance Criteria
1. The driver shall take every language-specific rule it applies (which syntax elements are conditions,
   failure signals, state writes, adjacent checks and function boundaries; comment and string kinds; operator
   templates) from the pack of the language being examined.
2. The driver shall ship admitted packs for Rust, Python, TypeScript and Java.
3. When a pack is submitted for admission, the driver shall verify that it is well formed, that it stays within
   the size limit of 40 lines and 2,500 characters, that every syntax element it names exists in the grammar
   the driver uses for that language, and that its templates use only the defined placeholders.
4. When a pack is submitted for admission together with a scope and control annotations for that scope, the
   driver shall verify that every control behaves as expected with the pack and that every control site with an
   operator yields a parseable change.
5. If any admission check fails, then the driver shall reject the pack, name each failed check with the
   offending names or control labels, and shall not use the pack for discovery, analysis or mutation.
6. If a command names a language for which no admitted pack is available, then the driver shall report that
   the language is not supported, produce no receipt, and exit with a code that is distinct from success and
   from "no finding".
7. The driver shall contain no file name, project name or source fragment of any particular codebase in its
   rules or packs.

### Requirement 2: Site discovery and analysis
**Objective:** As a reviewer, I want a located list of the places where a file decides, fails, writes state
and changes behaviour boundaries, so that I can see what a change could affect, in all four languages.

#### Acceptance Criteria
1. When a source file and language are given, the driver shall report each eligible site with its file, its
   start and end position, its kind and the text of the syntax element.
2. The driver shall exclude sites inside comments, inside string literals and inside regions the pack marks as
   test-only.
3. When analysis is requested, the driver shall write a receipt that lists the state-write sites, the function
   boundaries and the constructs the pack declares it cannot follow, for Rust, Python, TypeScript and Java.
4. When a frame specification with permitted write sites is supplied, the driver shall report every state-write
   site outside it as a violation and shall exit with the "finding" code if any exists.
5. If the source file does not parse under the language's grammar, then the driver shall report where parsing
   failed and shall not report sites for that file.
6. The driver shall report the same sites, in the same order, for the same file, pack and driver version on
   every run and on every supported platform.

### Requirement 3: Mutant generation
**Objective:** As a maintainer, I want mutants for arbitrary code produced by generic operators, so that test
strength can be measured on code the tool has never seen.

#### Acceptance Criteria
1. When mutants are requested for a file, the driver shall apply each operator the pack defines for a site's
   kind and syntax element and shall produce one mutant per applicable operator, each differing from the
   original only inside that site's statement or token.
2. The driver shall offer at least these operators where the pack supplies their data: negate a condition,
   swap paired operator tokens inside a condition, remove a failure signal, replace a returned literal by its
   paired literal, drop a state-writing statement, and swap two adjacent checks.
3. If a generated change does not parse, or differs from the original only in comments or whitespace, then the
   driver shall classify it as invalid and shall not run it.
4. If negating a condition and swapping its own equality operator would express the same condition at one
   site, then the driver shall emit only one of the two mutants.
5. When mutants are generated, the driver shall write a receipt naming for each mutant its site, its operator
   and a hash of the mutated text, and listing the sites for which the pack offers no operator.
6. If no mutant can be generated for a file, then the driver shall say so, write the receipt with a count of
   zero, and exit with the "finding" code.
7. Where a limit on the number of mutants is given, the driver shall choose mutants by a seeded procedure that
   is reproducible and represents every site kind present.

### Requirement 4: Running mutants against the project's tests
**Objective:** As a maintainer, I want each mutant run against my own tests and classified, so that I learn
which faults my tests would miss.

#### Acceptance Criteria
1. When a mutation run is requested with a test command, the driver shall first run that command on the
   unmutated code and shall proceed only if it passes.
2. When a mutant has been applied, the driver shall run the optional build command and the test command and
   shall classify the mutant as killed when the tests ran and at least one failed, as survived when all passed,
   as invalid when the build failed or the test runner could not load the tests, and as timed out when the time
   limit was reached.
3. The driver shall allow each mutant a time limit derived from the duration of the unmutated run, with a
   configurable floor and ceiling.
4. If a build or test command could not be started, or ended in a way the configuration declares a resource
   failure, then the driver shall classify the mutant as an error, retry it once, and shall never record an
   error as killed, survived or invalid.
5. If a failing test exit carries no evidence that the test runner executed, then the driver shall classify the
   mutant as an error.
6. When all mutants have run, the driver shall run the unmutated tests again and shall declare the run void if
   they do not pass.
7. When a run ends, the driver shall write a receipt with the classification of every mutant, counts by class
   and by site kind, the time limit used, the identities of the source, pack and driver, and whether any error
   remains; a run with a remaining error shall be reported as incomplete.
8. Where several workers are requested, the driver shall run mutants concurrently in independent copies and
   shall produce the same classification for every mutant as a serial run.
9. When the same mutant of the same original under the same commands has already been classified, the driver
   shall be able to reuse that outcome and shall mark it as reused in the receipt; errors and outcomes of void
   runs shall never be reused.

### Requirement 5: Protection of the examined code
**Objective:** As an adopter, I want certainty that running the driver cannot change or damage my code, so
that I can use it on a working tree I care about.

#### Acceptance Criteria
1. The driver shall never write a mutant into the source tree it was pointed at.
2. When a mutation run starts, the driver shall work in a copy of the examined code that it creates or that
   the caller names as disposable, and shall refuse to run if that location is the examined source tree.
3. When a mutant has been run, the driver shall restore the original bytes of every changed file in the copy
   and verify them against the hash recorded before the run.
4. If the driver is interrupted or a child process is killed during a run, then on its next start for the same
   copy the driver shall detect an unrestored file, restore it or discard the copy, and report what it did.
5. If restoration cannot be verified, then the driver shall stop the run, report the file, and exit with a
   failure code.
6. The driver shall write receipts, logs and stored outcomes outside the examined source tree unless the
   caller names a location inside it explicitly.
7. When a run ends or is stopped, the driver shall leave no build or test process of its own running.

### Requirement 6: Externally supplied annotations
**Objective:** As a researcher, I want to hand the driver a list of sites that a person or a model selected,
so that a selection can be tried without the driver depending on a model.

#### Acceptance Criteria
1. When a list of annotations (file, lines, kind, exact anchor text) is supplied, the driver shall accept an
   annotation only if the file is in scope and not a test, the anchor is exactly one syntax element at the
   stated lines, that element is allowed for the kind by the pack, the site is not in a comment, string or
   test-only region, and it does not duplicate an accepted annotation.
2. When an anchor omits or includes the statement terminator, the driver shall resolve it to the same site and
   record that it did.
3. If an annotation is rejected, then the driver shall record the reason for that annotation and shall not
   generate a mutant from it.
4. When annotations are accepted, the driver shall generate and run mutants for exactly those sites and report
   the share of supplied annotations that were valid.
5. The driver shall not contact a model or any network service to obtain or check annotations.

### Requirement 7: Replacement of the specimen-bound operators and analyzers
**Objective:** As the owner, I want the hard-coded, non-agnostic code paths gone, so that no result depends on
whether a file resembles an experiment specimen.

#### Acceptance Criteria
1. When the driver is delivered, `lesscode-core` shall contain no mutation operator or analysis rule that
   matches a substring or line pattern taken from a particular specimen.
2. When the driver is delivered, `deconvolute analyze` and `deconvolute mutate` shall produce their results
   through the driver for Rust, Python, TypeScript and Java.
3. The driver shall keep the documented exit-code meanings of `lesscode-core`: success, finding, incomplete,
   and invalid input or failure.
4. When an existing specimen experiment is re-run through the driver, the workflow shall record the driver's
   result beside the earlier one without altering the earlier record.
5. If a behaviour of the removed operators cannot be reproduced by a pack-driven operator, then the workflow
   shall record that loss in the dated research log and shall not reintroduce a specimen-specific rule.

### Requirement 8: Single runtime, size and platforms
**Objective:** As the owner, I want one binary within the agreed size limits on every supported platform, so
that adopters install one thing and the apparatus does not grow unnoticed.

#### Acceptance Criteria
1. The driver shall be part of the `lesscode-core` executable and shall require no other language runtime,
   interpreter or service to discover sites, analyze, generate mutants, validate annotations or admit packs.
2. When a design is proposed for approval, the workflow shall present a measured size of a native
   implementation of the kind check and at least one operator, the size of the code it makes removable, and
   the resulting position against the apparatus cap and the per-file budgets.
3. If the measured design exceeds the apparatus cap, then the workflow shall withhold production task
   generation until the owner records a dated decision on the cap.
4. When a grammar the driver uses differs in version from the one a pack was written against, the workflow
   shall re-admit the pack against the driver's grammar before relying on it.
5. The driver shall pass the repository's correctness checks on Linux x86_64 and aarch64, Windows x86_64, and
   macOS on Intel and Apple Silicon.
6. When dependencies are added for the driver, the workflow shall record them in the dependency and license
   inventory and keep the locked build reproducible.

### Requirement 9: Evidence that the native driver matches the prototype
**Objective:** As a continuing researcher, I want the native driver held to the results the prototype was
judged by, so that the port cannot quietly change what was shown.

#### Acceptance Criteria
1. When the native driver is run on the five scopes of the prototype gate with the admitted packs, the
   workflow shall compare its sites and mutants with the retained prototype receipts and report every
   difference with its cause.
2. When the native driver runs those mutants against the scopes' tests, the workflow shall compare each
   classification with the retained receipts and treat any unexplained difference as a defect.
3. When the validator controls of the prototype are replayed through the native driver, every control shall
   behave as recorded.
4. When a result of this work is recorded, the workflow shall add a dated research entry, update the research
   index, link the entry and this spec to each other, and keep setup, behavioural evidence and adoption as
   separate outcomes.
5. If a comparison cannot be made because a retained receipt or scope revision is unavailable, then the
   workflow shall report the comparison as not run and shall not report it as passed.
