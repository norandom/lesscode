# Requirements Document

## Introduction

The Deconvolution Loop takes one declared scope of an existing codebase and
produces a smaller candidate with fewer internal edges whose behaviour at
the scope's public surface is unchanged on everything that was recorded,
with the protection level of that claim measured and declared rather than
assumed. It is the algorithm-agnostic form of the loop prototyped in
R22 to R25. The subject of every criterion is "the Loop". Criterion
identifiers use section.item notation, for example 4.2.

Generated 2026-11-04 by the agent at the owner's request as documentation.
Not yet approved. The feasibility status of each requirement is listed in
section 13; a requirement marked unproven is a specification of intent, not
a demonstrated capability.

## Boundary Context

- **In scope**: deriving the public surface of a declared scope; recording
  and replaying behaviour with declared observation levels; measuring
  size and edges with whole-replacement accounting; a structural contract
  separating pure logic from effects; bounded regeneration rounds with an
  acceptance decision and kill criteria; independent review; sensitivity
  scoring by mutation; differential counterexample finding on the pure
  core; declaring the residue; provenance and budget discipline.
- **Out of scope**: choosing which scope to regenerate (unproven, R21);
  prevention at generation time; proving equivalence; merging a candidate
  into the specimen; writing or weakening tests; reducing features;
  performance, memory or byte-size targets.
- **Adjacent expectations**: a test-driven workflow owns the test suite and
  consumes the Loop's missing-test report; the Loop uses existing tests as
  part of acceptance but never edits them. Project runtimes and test
  infrastructure must be present and are detected, not provided. Measures
  for code outside the scope come from the existing comparison core.

## Requirements

### Requirement 1: Scope and public surface

**Objective:** As a researcher, I want the scope's public surface derived
mechanically, so that regeneration is constrained by what is actually used
rather than by a hand-written list.

#### Acceptance Criteria

1. When a scope is declared as a set of source files, the Loop shall derive
   the public surface as every name defined in the scope that is referenced
   from any file outside the scope, including test files and including
   names conventionally marked private.
2. The Loop shall record, for each surface name, the referencing files and
   the kind of reference (import, attribute read, patch or override).
3. When a reference cannot be resolved to a definition, the Loop shall list
   it as unresolved rather than omit it.
4. The Loop shall list references among files inside the scope separately
   and shall not count them as surface.
5. If a hand-written surface list is supplied, then the Loop shall report
   every name the derivation adds or drops relative to it, and the derived
   surface shall be authoritative.

### Requirement 2: Behaviour recording with observation levels

**Objective:** As a researcher, I want behaviour recorded once from the
original and replayed against candidates, so that acceptance rests on
observed effects rather than on reasoning.

#### Acceptance Criteria

1. The Loop shall record a corpus of sequences, each a list of operations
   over the scope's surface starting from a declared initial state, with
   every observation needed to compare outcomes.
2. The Loop shall separate observations into Level 1 (results, raised
   errors, persistent state after the operation, produced files) and
   Level 2 (the text and parameters of queries issued to external stores).
3. When the same corpus is recorded twice from the original, the Loop shall
   produce byte-identical recordings without masking, and it shall refuse
   to proceed if they differ.
4. When a recording is replayed against the original, the Loop shall report
   zero divergence at both levels, and it shall stop the experiment if it
   does not.
5. The Loop shall apply a declared normalisation (for example, timestamps
   become first-seen ordinals) and shall document every normalised field.
6. The Loop shall include sequences generated from the surface's operations
   beyond those derived from tests, with the generator seed recorded.
7. When a new operation is added to the corpus alphabet, the Loop shall
   record a new corpus version with its own identity and shall not alter
   an existing one.

### Requirement 3: Acceptance decision

**Objective:** As a researcher, I want a single, fail-closed acceptance rule,
so that a candidate is either accepted on evidence or rejected with the
first divergence named.

#### Acceptance Criteria

1. The Loop shall accept a candidate only when every corpus sequence
   replays with zero Level 1 divergence, every existing test selected as a
   seed passes with the same baseline deselections as the original, and
   every reference observation harness output is identical.
2. When a Level 2 divergence occurs, the Loop shall accept it only if a
   query differential over at least one hundred perturbed states, each
   seeded with a boundary row, returns identical result sets for the
   original and the candidate query.
3. If the acceptance replay does not produce a fresh result file covering
   every sequence, then the Loop shall treat the round as failed.
4. The Loop shall report the first divergence by sequence, operation and
   observation path.
5. The Loop shall never treat an acceptance pass as proof of equivalence,
   and every report shall state the number of sequences and operations
   the pass covers.

### Requirement 4: Measures and whole-replacement accounting

**Objective:** As a researcher, I want size and structure measured the same
way on original and candidate, so that a reduction claim survives
formatting tricks and relocated code.

#### Acceptance Criteria

1. The Loop shall measure, per file and for the scope: physical lines,
   statements, functions, write sites to external stores, distinct
   intra-scope call edges, and distinct file-to-file hops.
2. The Loop shall report statements both as written and after a declared
   canonical formatting of both trees, and a reduction claim shall cite the
   canonical figure.
   Amendment 2026-11-05 (owner decision after task 1.1): a formatter never
   changes the statement count, so the Loop shall also report a third
   reading, the separated-form statement count, in which every multi-name
   import, tuple assignment, chained comparison and similar packed
   statement is counted as the number of single-name statements it
   replaces, under a declared and versioned rule; a reduction claim shall
   cite the separated-form figure alongside the canonical one.
3. The Loop shall count every new helper, wrapper, table, configuration or
   changed caller that the candidate introduces, inside or outside the
   original files, as part of the candidate.
4. When the candidate relocates code outside the scope, the Loop shall
   include the destination in the measurement and report the relocation.
5. The Loop shall report tests, fixtures and apparatus separately from
   production implementation and shall not count them as reduction.
6. The Loop shall reproduce measures text-identically on repeated runs over
   unchanged trees.

### Requirement 5: Structural contract

**Objective:** As a researcher, I want pure logic separated from effects and
checked, so that the regenerated code is simpler to reason about and the
differential step has something to work on.

#### Acceptance Criteria

1. When a candidate is produced, the Loop shall require a declaration that
   assigns every function in the scope to either the pure core or the
   effect shell.
2. The Loop shall check that no declared pure function receives a
   connection, session or engine parameter and that none calls an
   effectful operation on an external store.
3. The Loop shall check that every write site lies inside a declared shell
   function.
4. The Loop shall report undeclared functions, the pure share by function
   count and by statement count, and every failed check.
5. While the declared pure share is below the preregistered minimum or any
   check fails, the Loop shall not count the round as meeting the
   structural prediction.

### Requirement 6: Regeneration rounds

**Objective:** As a researcher, I want regeneration bounded and auditable,
so that a failed experiment is a result and not a loop that never ends.

#### Acceptance Criteria

1. The Loop shall give the regenerator only the scope files, the derived
   surface, the definitions, the measure targets, the structural contract
   and the acceptance command, and shall record the exact inputs.
2. The Loop shall limit regeneration to a preregistered number of rounds.
3. When a round ends, the Loop shall record the changes made, the measures,
   the acceptance outcome and the first divergence, if any, before the
   next round starts.
4. If the candidate changes any file outside the scope, any test, any
   recorded corpus or any apparatus file, then the Loop shall declare the
   candidate void.
5. The Loop shall verify by content hash that every file outside the scope
   is identical between the original and the candidate trees.
6. The Loop shall preserve surface names that tests patch or override as
   the names the candidate's own functions look up.
7. When a round follows a failed round, the Loop shall give the
   regenerator the unchanged inputs of 6.1 plus only the previous round's
   first divergence and its measure gaps against the targets, and nothing
   else from the previous round. When a round follows an accepted round
   whose candidate has been sensitivity-scored, the Loop shall add that
   candidate's surviving faults and realised counterexamples.
8. Where the regenerator is a model endpoint rather than an interactive
   agent, the Loop shall take the endpoint configuration from the
   environment, shall never store credentials in any artefact, and shall
   record every prompt, response, model identifier and seed in the receipt
   by content hash.

### Requirement 7: Independent review

**Objective:** As a researcher, I want a reviewer that did not write the
candidate to look for differences the oracle cannot see, so that reasoning
and probes supplement recording.

#### Acceptance Criteria

1. When a candidate passes acceptance, the Loop shall submit it to a
   reviewer with the plan, the diff and the two trees only, and without the
   regenerator's rationale.
2. The Loop shall require the reviewer to verify the edit-scope rule, the
   surface signatures, the apparatus hashes and the measures independently.
3. The Loop shall require the reviewer to state behavioural hypotheses,
   probe them concretely on both trees, and report every probe's agreement
   or disagreement.
4. The Loop shall require the reviewer to report formatting effects on the
   measures, including statement packing and maximum line length.
5. The Loop shall carry every reviewer hypothesis forward as a named input
   to the sensitivity and differential steps.

### Requirement 8: Sensitivity scoring

**Objective:** As a researcher, I want the protection of the acceptance
corpus measured, so that "zero divergence" carries a number.

#### Acceptance Criteria

1. The Loop shall draw a sample of mutants from the candidate with a
   recorded seed and population size, apply each in an isolated copy, run
   the acceptance arm and classify the mutant as killed, survived or
   incompetent.
2. The Loop shall report the kill rate as killed over killed plus survived,
   with incompetent mutants excluded and survivors kept in the
   denominator.
3. Where a reading that excludes mutants judged equivalent by inspection is
   given, the Loop shall label it post hoc and shall not use it as the
   result.
4. The Loop shall run every hand-written semantic mutant inherited from the
   original's oracle experiments, ported to the candidate where the
   mutated code moved, and shall report each as caught or missed with the
   catching evidence.
5. The Loop shall restore the isolated copy after every mutant and verify
   it by content hash before the next.
6. When sensitivity is compared with an earlier figure, the Loop shall
   state whether the populations are comparable, and shall draw future
   samples from the original and the candidate alike.
7. The Loop shall emit the survivors and realised counterexamples as a
   missing-test report naming the function, line, mutated condition and
   the input class that would distinguish it.

### Requirement 9: Differential counterexample finding

**Objective:** As a researcher, I want pure-core pairs searched for inputs
where original and candidate differ, so that oracle-blind differences are
found before anyone trusts the acceptance pass.

#### Acceptance Criteria

1. The Loop shall select pairs of original and candidate pure functions
   with the same signature, at most a preregistered number, and shall
   record why each pair was chosen and which reviewer hypothesis it tests.
2. Before any solver run, the Loop shall pass four verifier checks per
   pair: more than one explored path on each side; agreement of the two
   drivers on a recorded or generated input corpus; a planted mutant that
   yields a counterexample and an equivalent renaming that exhausts; and
   concrete replay of every counterexample.
3. When the corpus check of a pair finds disagreement, the Loop shall
   report that disagreement as a behavioural difference found by the
   corpus, mark the pair's solver verdict as not gated, and still replay
   the difference concretely.
4. The Loop shall report each pair as counterexample, inconclusive or
   exhausted, and shall never report inconclusive as equivalent.
5. If either control fails, then the Loop shall declare the differential
   step void and leave the other steps standing.
6. When a counterexample is realisable, the Loop shall replay it as a new
   corpus sequence if the corpus alphabet can express it, and otherwise
   shall report that the alphabet cannot express it.

### Requirement 10: Residue and protection declaration

**Objective:** As a researcher, I want every report to say what is not
protected, so that a reader can judge the claim.

#### Acceptance Criteria

1. The Loop shall declare the residue of every run: unreached functions,
   surviving mutants not judged equivalent, unresolved differential
   targets, differences the corpus alphabet cannot express, and input
   classes the corpus never carried.
2. The Loop shall state the protection level as the acceptance corpus
   plus the measured sensitivity plus the differential outcome, with the
   residue, and shall not state it in any stronger form.
3. The Loop shall report every failed prediction as failed, next to the
   predictions that held.

### Requirement 11: Preregistration, provenance and budgets

**Objective:** As the owner, I want every run falsifiable and reproducible,
so that progress is empirical rather than persuasive.

#### Acceptance Criteria

1. Before any heavy step, the Loop shall record predictions, kill criteria,
   frozen input identities and budgets, and shall not change predictions
   afterwards; open decisions are recorded as dated amendments.
2. The Loop shall write a receipt for every command with its arguments,
   environment, start and end time, exit status, host load probe and the
   content hashes of apparatus, corpus and result files.
3. The Loop shall keep failed gate iterations in the receipt and shall not
   delete or relabel them.
4. The Loop shall scale every deadline by a measured host-load factor
   within declared bounds, shall never impose a fixed wall-clock cap, and
   shall rerun once when idle a step that timed out under load.
5. Before every heavy step, the Loop shall wait for the host to be idle
   within a bound and shall refuse to start when free memory is below a
   declared floor.
6. The Loop shall count research code written for a run against a
   preregistered line budget and shall report overruns rather than compact
   frozen files.
7. The Loop shall never modify the original specimen, the frozen copy or
   a recorded corpus during a run, and shall operate on mutants only in
   separate isolated copies.

### Requirement 12: Host and isolation constraints

**Objective:** As the owner, I want the Loop to run on an ordinary developer
machine, so that it is usable where the code is written.

#### Acceptance Criteria

1. The Loop shall run natively on the developer's operating system under a
   plain user account without elevated privileges, symbolic-link
   privileges or a secondary execution environment.
2. If specimen tests fail on the host for environmental reasons, then the
   Loop shall record them as baseline deselections, apply the same
   deselections to original and candidate, and shall not repair them.
3. The Loop shall run heavy jobs one at a time and shall pin numeric
   library thread counts to one for runs whose attribution must be
   deterministic.
4. The Loop shall detect the specimen's test runner, markers, fixtures and
   external-store setup and shall report what it detected before the first
   run.

## 13. Feasibility evidence by requirement

Status as of 2026-11-04. "Proven" means exercised end to end on real code
with recorded evidence; it is a feasibility statement on one scope, not
general efficacy.

| Requirement | Status | Evidence |
| --- | --- | --- |
| 1 Surface | Proven (Python) | R25 step 0: 62 names derived, 15 beyond the hand list |
| 2 Recording | Proven (Python with external store; Rust in-process) | R22 Q5 byte-identical recordings; R20 Rust oracle; 2.6 generated sequences found 2 new behaviours (R22 Q4 failed its count) |
| 3 Acceptance | Proven | R25 T1: 64 sequences, zero divergence; 3.2 query differential built and controlled, not yet exercised on a real divergence |
| 4 Measures | Partial | R25 T2/T3 measured; 4.2 canonical-formatting figure is a reviewer estimate (about 572), not yet a tool |
| 5 Structural contract | Proven | R25 T3: 36 of 51 pure, 11 writes in shell, check passes; one known false positive in the effectful-call rule |
| 6 Rounds | Proven for one round; 6.7 and 6.8 unproven | R25: accepted in round 1 of 4 by an interactive agent; the turn protocol (6.7) was never exercised and no endpoint program (6.8) exists |
| 7 Review | Proven | R25 step 2: 76 probes, H4 later confirmed by T5 |
| 8 Sensitivity | Proven as a measurement, failed as a target | R22 0.712 to 0.746; R25 0.643 against 0.70; 8.7 missing-test report not built |
| 9 Differential | Proven as counterexample finder only | R24 verifier controls; R25 one counterexample, two inconclusive; 9.3 corpus check found the difference first |
| 10 Residue | Proven | R24 and R25 declare residue |
| 11 Provenance | Proven | receipts, adaptive budgets, amendments in E1c-2 and E3 |
| 12 Host | Proven on Windows | 236 baseline deselections; 12.4 detection is manual today |
| Scope selection | Rejected once, out of scope | R21 edge census failed C2 and C5 |
| Other languages | Unproven | R20 Rust oracle only; no regeneration |
