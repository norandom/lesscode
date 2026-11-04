# Regenerate behind the surface: a measured loop for shrinking AI-generated code

Report on the Lesscode deconvolution research, November 2026.

- Project: Lesscode, a research effort to make AI-generated codebases
  smaller and less entangled without changing their behaviour.
- Period covered: 1 to 4 November 2026. Six experiments on two real
  codebases, recorded as dated entries R19 to R25 in the project's
  research log.
- Audience: a reader who knows software verification and SMT solving but
  has not followed this project. Every project-specific term is defined on
  first use; a glossary is at the end.
- Status: research results on one scope. Nothing here has been adopted
  into any product.

## 1. Executive summary

Coding agents produce working code that scatters one responsibility across
many small functions and files. Rewriting such code smaller is easy for the
same agents. Knowing that the rewrite still behaves the same is the hard
part, and tests alone do not settle it: they rarely assert on what bytes
reach the database or on what happens on an error path.

We built a loop that lets a language model rewrite one bounded part of an
existing codebase behind that part's real public interface, and that
measures how much of the old behaviour the rewrite demonstrably keeps. The
loop records the original's behaviour under an effect-capturing sandbox,
scores that recording by fault injection before any rewrite exists, lets
the model regenerate the code under a structural contract, accepts only on
zero divergence from the recording, measures size and internal coupling on
both versions, has a second agent review the diff, then searches the
rewrite for differences the recording cannot see.

On its first full run, on five modules (1,274 lines) of a Python trading
application with a PostgreSQL store, the loop accepted a rewrite in the
first of four allowed rounds. The rewrite had 16 percent fewer lines,
24 percent fewer functions and 29 percent fewer internal call edges, with
zero divergence on 64 recorded behaviour sequences and every existing test
passing. The same run measured the limits of that claim: the recording
catches 64 percent of injected faults, not the 70 percent we predicted,
and one real behavioural difference on an exception path was invisible to
the recording and was found by review and random testing instead.

Our conclusion is that the loop is a usable research instrument and worth
building into a spec-driven development workflow, as long as its output is
read as "measured protection with a declared residue" and never as
"equivalent". Automatic selection of what to rewrite, behaviour over
several rounds, and languages other than Python remain unproven.

## 2. The problem

### 2.1 What "convoluted" means here

The codebases in question work. Their tests pass. Their problem is
structural: a single responsibility, such as "record the evidence behind a
trading decision and store it", is spread over a dozen functions in four or
five files, each forwarding to the next, each validating a little, each
holding part of the state. Maintaining such code means reading all of it
to change any of it. We call this convolution, and the goal of the project
is deconvolution: fewer lines, fewer functions, fewer internal edges, the
same behaviour at the boundary that the rest of the system uses.

### 2.2 Why the obvious approaches failed on real code

Before the work reported here, the project tried three approaches on the
same kind of code. Each is recorded in the research log with its failure.

- Static state models. We extracted call and data-flow facts with a code
  query engine and built a model of the responsibility's state. On real
  code the model could not be shown to match the source: known aliasing
  and overwrite flows were missed, and adding evidence did not close the
  gap (entries R8 to R10).
- Model-checked models. We had a language model write a formal model of
  the scope and checked it with an SMT solver. The model could be checked
  for consistency but never for fidelity to the program, so a solver
  verdict about the model said nothing certain about the code. The
  approach was withdrawn as an equivalence authority (R19).
- Instructions without measurement. A skill with careful rewriting
  guidance, applied by an agent to real scopes, produced candidates that
  were sometimes larger and sometimes wrong (R11 to R18).

The common lesson was that any claim of "same behaviour" has to be checked
against the running original, not against a model of it. The research
direction changed accordingly on 3 November 2026.

## 3. The setting

### 3.1 Specimen codebases

Two real codebases served as specimens. Both are the owner's own projects,
used under isolated frozen copies that were never modified.

- Facdrone, a Python trading-research application: about 2,800 Python
  files, around 1,500 tests, an embedded PostgreSQL database started by the
  test fixtures, and a database schema managed by migrations. The scope
  used for the main experiments is its evidence-publication
  responsibility: five modules that capture which inputs a trading policy
  consumed, which signals and route it took, seal that evidence into the
  database with content digests, and read it back. Original size: 1,274
  lines, 660 statements, 67 functions, 13 database write sites, 84 call
  edges among the five files.
- mmhmem, a Rust server that gives coding agents shared memory. Its
  session bus (checkpoint and resume of working state) served as the Rust
  specimen for the recording oracle.

### 3.2 Host and constraints

All runs were native Windows 11 under a plain user account, with no
elevated privileges, no symbolic-link privilege and no Linux subsystem.
This matters because 236 of Facdrone's tests fail on such a host for
environmental reasons (file locks, POSIX-only modules). They were recorded
once as baseline failures and deselected identically for the original and
every candidate, never repaired.

The host is shared and sometimes busy. Every deadline in the loop is
adaptive: a probe measures other processes' CPU use, free memory and a
short benchmark, derives a factor between 1 and 4, and scales the
preregistered base deadline by it. A step that times out under load is
rerun once when the host is idle. One run was killed by the operating
system when memory ran out; a memory guard was added before every heavy
step and the affected measurement was rerun from scratch.

### 3.3 Experimental discipline

Each experiment was preregistered: predictions, kill criteria, frozen
input identities and budgets were written down before the run, and
predictions were never changed afterwards. Decisions the plan left open
were recorded as dated amendments. Every command ran through a receipt
writer that records arguments, environment, timing, exit status, the
host-load probe and the content hashes of apparatus and result files.
Failed attempts stay in the receipt. Research entries lead with the
conclusion and report failed predictions as failed.

## 4. The loop

The loop takes one declared scope (a set of source files) and returns a
candidate rewrite plus a protection statement. It has eight steps.

```mermaid
flowchart LR
  A[1 Derive the public surface] --> B[2 Record behaviour from the original]
  B --> C[3 Score the recording by fault injection]
  C --> D[4 Regenerate behind the surface]
  D --> E{5 Accept: zero divergence?}
  E -- no, fewer than 4 rounds --> D
  E -- yes --> F[6 Measure size and structure]
  F --> G[7 Independent review]
  G --> H[8 Sensitivity, differential search, residue]
```

### Step 1. Derive the public surface

The public surface of a scope is the set of names defined inside it that
anything outside it references: production code, tests, and test code that
patches or overrides a name. Names conventionally marked private count if
something outside uses them. The derivation is mechanical, by parsing
every file in the repository.

Why it matters: on the Facdrone scope the derivation found 62 surface
names where a careful hand-written list had 47. Two of the extras were
private module globals that tests replace at runtime. A rewrite that
renamed or inlined them would have passed its own checks and broken those
tests.

### Step 2. Record behaviour from the original

A recorded corpus is a set of sequences. Each sequence starts from a
declared database state, issues a list of operations through the public
surface, and stores every observation needed to compare outcomes. The
recorder runs the original under a sandbox that captures effects: results,
raised errors, dumps of every affected table after each operation, files
produced, and the text and parameters of every SQL statement issued.

Observations are split into two levels. Level 1 is what the caller and the
persistent state can see: results, errors, table contents, files. Level 2
is how the code talked to the store: query text and parameters. A rewrite
may legitimately phrase a query differently, so Level 2 divergences are not
failures by themselves; they are arbitrated by a query differential (step
5). Timestamps are normalised to first-seen ordinals; the normalisation is
declared.

Two independent recordings of the same corpus must be byte-identical
without any masking, and a replay of the recording against the original
must show zero divergence. Both held (R22). The corpus contains sequences
derived from the existing tests and sequences generated from the surface's
operations with a recorded seed; the generated ones found two behaviours
no test exercised, both on failure paths.

This is differential testing in McKeeman's sense, with a recorded oracle
instead of a second implementation.

### Step 3. Score the recording by fault injection

Mutation testing, in the sense of DeMillo, Lipton and Sayward: inject a
small syntactic fault into the original, run the acceptance check, and
record whether the fault was caught (killed) or not (survived). Faults
that break compilation are incompetent and excluded. The kill rate is
killed divided by killed plus survived. Mutants judged equivalent by
inspection stay in the denominator; a rate that excludes them may be
reported only as a labelled post hoc figure.

This step turns "the tests pass" into a number before any rewrite exists.
On the Facdrone scope, the existing tests alone killed 42 of 59 sampled
mutants (0.712) and tests plus replay killed 44 (0.746). The gain was
small but of the right kind: the replay caught a change to the compression
level of stored evidence, which no test asserts on. Three hand-written
semantic faults (a ledger adopted before commit, a legacy path that skips
reference validation, that compression change) are kept as fixed controls
for every later run.

### Step 4. Regenerate behind the surface

The regenerator is a language-model agent. It receives the scope files,
the derived surface, the definitions of the measures, the size and
structure targets, a structural contract, and the exact acceptance
command. It receives nothing else and may edit nothing but the scope files.

The structural contract requires every function in the candidate to be
declared either pure core (no connection, session or engine parameter; no
call that executes, commits or flushes against the store) or effect shell.
A static check enforces both rules and requires every write site to lie
inside a declared shell function. The purpose is twofold: the rewrite
becomes easier to reason about, and step 8 has pure functions to search.

Rounds are capped at four. After each round the candidate's changes,
measures and acceptance outcome are recorded. Every file outside the scope
is hashed against the frozen original; any difference, including a touched
test, corpus or apparatus file, voids the candidate.

### Step 5. Accept on zero divergence

A candidate is accepted when all recorded sequences replay with zero
Level 1 divergence, every existing test selected as a seed passes with the
same baseline deselections as the original, and a fixed observation
harness from an earlier trial (128 printed observations of the scope's
outputs) produces identical output. If any Level 2 pair differs, the
query differential loads at least one hundred perturbed database states,
each seeded with a boundary row that sits exactly on the predicate the
query tests, executes both queries, and requires identical result sets.
The arm fails closed: if the replay did not write a fresh result file
covering every sequence, the round failed.

An acceptance pass is a statement about the recorded sequences. Every
report states how many sequences and operations the pass covers.

### Step 6. Measure size and structure

On both trees, with the same script: physical lines, statements, functions,
write sites, distinct intra-scope call edges (caller function to callee
function, both in the scope), and file-to-file hops. Accounting is
whole-replacement: a helper the rewrite moves outside the scope still
counts. Tests, fixtures and research apparatus are reported separately and
never counted as reduction.

### Step 7. Independent review

A second agent, with the plan, the diff and both trees and without the
regenerator's reasoning, verifies the edit-scope rule, the surface
signatures, the apparatus hashes and the measures; states hypotheses about
behavioural differences; and probes each hypothesis concretely on both
trees. It also reports formatting effects on the measures. In the run
reported here the reviewer found that part of the statement reduction was
denser formatting and named the exception-path difference that step 8
later confirmed.

### Step 8. Sensitivity, differential search, residue

Sensitivity repeats step 3 on the candidate: a seeded sample of injected
faults, each applied in an isolated copy, run through the acceptance arm,
classified, and the copy restored and hash-verified before the next.

The differential search takes pairs of pure functions, original and
candidate, with the same signature, and runs a solver-backed differential
(CrossHair's `diffbehavior`, a concolic engine over Python with Z3
underneath) to look for an input on which they differ. Concolic execution
runs the code on concrete values while tracking symbolic constraints on
the path, then asks the solver for values that flip a branch; this is the
mechanism SAGE established for whitebox fuzzing. Before any solver verdict
counts, four verifier gates must pass per pair: more than one explored path
on each side (the drivers are actually symbolic); agreement of the two
drivers on a generated input corpus (the encoding is sound); a planted
fault in the candidate yields a counterexample and an equivalent renaming
exhausts its paths (the pipeline can detect and can finish); and every
counterexample reproduces on concrete replay (it is real). A verdict of
"no differences found" without path exhaustion is inconclusive and is
never reported as equivalence.

The residue is declared last: functions the corpus never reaches,
surviving mutants not judged equivalent, unresolved differential pairs,
differences the corpus alphabet cannot express, and input classes the
corpus never carried. The protection statement is the acceptance corpus
plus the measured sensitivity plus the differential outcome, with that
residue, and nothing stronger.

## 5. Results

### 5.1 The six experiments

| Entry | Question | Specimen | Outcome |
| --- | --- | --- | --- |
| R20 | Can a recorded oracle be built for Rust and does it beat the tests? | mmhmem session bus | Oracle built, deterministic, caught every injected fault; the predicted gain over tests failed because tests already reached 96 percent of the scope. Generated inputs found a real panic (a byte cap falling inside a multi-byte character). |
| R21 | Do structural measures (edges, write sites) pick the scattered scopes automatically? | Facdrone, 2,275 responsibilities | Rejected. Two of five predictions failed; the measures did not rank the known scattered scopes first. Scope selection remains manual. |
| R22 | Can the oracle be built for Python with a database, and does replay halve the gap the tests leave? | Facdrone evidence publication | Oracle byte-deterministic. Kill rate 0.712 with tests, 0.746 with replay; the halving prediction failed. Replay caught a stored-representation fault tests miss. |
| R23 | Does solver-guided input generation resolve the faults the oracle missed? | Same scope, 10 surviving faults | Failed as run: the drivers concretised strings, and the oracle never called the scope's reader functions at all. |
| R24 | Same question, with the drivers corrected and a verifier in front of the solver | Same | Verifier controls passed. 7 of 10 faults killed: 4 by the extended oracle alone, 3 only by solver-found inputs. 0 proved equivalent. The preregistered threshold was missed, so the solver is admitted as a counterexample finder, not as an equivalence authority. |
| R25 | Does a model regenerate the scope smaller and less coupled within the acceptance rule? | Five modules, 1,274 lines | Accepted in round 1. See 5.2. |

### 5.2 The regeneration run in detail

Predictions were fixed before the run. Five held or partly held; one
failed.

| Prediction | Measured | Held |
| --- | --- | --- |
| Zero Level 1 divergence on all 64 sequences, identical harness output, all seed tests pass, within 4 rounds | Round 1: 64 sequences, 311 operations, 0 Level 1 divergences, 0 Level 2 pairs; 1,403 + 39 + 92 tests passed across three test groups | Yes |
| Statements at most 561, lines at most 1,083, functions at most 67 | 546 statements, 1,074 lines, 51 functions | Yes, with a caveat below |
| Call edges at most 67; write sites at most 13, all in shell functions; at least half the functions pure, static check passing | 60 edges; 11 write sites, all in the 15 declared shell functions; 36 of 51 functions pure; check passed | Yes |
| Kill rate on 30 injected faults at least 0.70; the three hand-written controls caught | 18 killed, 10 survived, 2 incompetent: 0.643; all three controls caught (two by tests, the compression change by replay only) | No on the rate |
| At least one realisable difference the recording cannot see | One: after an exception inside the capture function, the original leaves an audit note that the rewrite does not, because the rewrite calls the routing step after deduplication instead of before. Reproduced on concrete replay. Two other pairs inconclusive. | Yes, with a caveat below |

What the rewrite did: six record-validation helpers became one validator;
two sealing paths share one private function; a three-function resolver
walk became one function; several one-call helpers were inlined; two
caches were replaced by one resolver object. Only the five scope files
differ from the original; 4,851 other files are hash-identical.

Caveats, both from the independent review and both reported next to the
successes:

- Of the 114 statements removed, 26 are denser formatting. Reformatted
  into the original's style the candidate has about 572 statements, above
  the 561 target. The reductions in functions, edges and write sites do
  not depend on formatting.
- The exception-path difference was named by the reviewer reading the
  diff, then shown on 151 of 300 random inputs by the differential's own
  soundness corpus, and only then confirmed by the solver. The solver
  found nothing in the two other pairs. On this scope it added no
  difference that cheaper means had not already exposed.

Of the ten surviving injected faults, three change only type annotations
(never evaluated at runtime), two are equivalent by inspection, two differ
only on malformed input of mismatched length, and three are real gaps:
a malformed reference string, a ledger record without a currency field,
and lax validation of stored JSON. All three are inputs the recorded
corpus never carried.

Budget: research code written for this run was 545 lines against a
preregistered 400; the overrun is reported, nothing was compacted. Machine
time was about five and a half hours including the fault-injection and
solver phases, on a shared host at load factors between 1.0 and 3.6.

## 6. What we learned

**The set of recorded operations decides what is visible.** Four of the
seven faults killed in R24 died the moment the oracle began calling reader
functions it had never called. The exception-path difference in R25 is
invisible to the oracle for the same reason: no recorded operation calls
that function directly. Extending the oracle's alphabet is a new corpus
version and a re-validated baseline, not a tweak.

**Existing tests carry most of the protection.** Replay added four points
of kill rate on the original and one class of fault the tests cannot see.
In a workflow where a test-driven process owns the tests, the loop's job
is to measure what they miss and hand that back as a list of missing
tests, not to replace them.

**Strictness at Level 2 is a design choice.** Because the oracle compares
query text, a semantically identical query with a different row limit is
a divergence. That is right for fault detection and wrong for a rewrite
that is allowed to rephrase queries; the query differential exists to
arbitrate, and on this run it was never needed because the rewrite issued
identical queries.

**Random inputs reached the real difference before the solver.** This
matches the lesson of SAGE: constraint-guided generation pays off where
random generation cannot reach the branch. Here the branch was reachable
with 300 random inputs. The solver's place is the last gate, for the pure
functions where random generation stalls, and only behind the four
verifier checks.

**Statement counts are gameable; edges and functions are not.** Future
size claims are reported after canonical formatting of both trees.

**Tests that reach into internals block consolidation.** Two privately
named globals had to survive because tests patch them. Test suites that
exercise a scope through its surface leave the rewrite more room.

## 7. Limitations

- One scope, five files, one regenerator, one round, one language, one
  host. These are feasibility results, not efficacy results.
- Zero divergence is a statement about 64 sequences. The measured
  sensitivity (0.643) and the declared residue are the honest size of the
  claim.
- Every real gap found so far is on malformed or failing inputs, where
  recorded traces are thinnest.
- The fault-injection rate on the candidate is not comparable with the
  earlier rate on the original: different fault population, different test
  selection. Future runs sample both trees the same way.
- Performance, memory and other non-functional behaviour were not
  measured. The rewrite dropped a cache and repeats some hashing.
- The apparatus is Python-specific and was assembled by hand for this
  scope: the effect recorder, the measures, the fault injector and the
  solver drivers all depend on the language and the store.

## 8. A prediction for the next scopes

The owner asked what the loop would save on further code. One scope and
one round are thin ground, so this is a preregistration for the next three
scopes, not a result. Class: Python scopes of 800 to 2,500 lines with one
scattered responsibility, a relational store behind them, and an existing
test suite that executes most of the scope.

| Quantity | Predicted per scope | The prediction is wrong if |
| --- | --- | --- |
| Lines, after canonical formatting of both trees | 10 to 20 percent fewer | median under 10 on three scopes |
| Functions | 20 to 30 percent fewer | median under 15 |
| Intra-scope call edges | 25 to 35 percent fewer | median under 20 |
| Rounds to acceptance | 1 on two of three scopes, at most 3 on the third | any scope exhausts 4 rounds |
| Kill rate of the recording on the candidate | 0.60 to 0.75 | below 0.55, or above 0.85 with no change in method |
| Differences invisible to the recording | at least one per scope, on an error or malformed-input path | none found by review, random inputs and differential together |
| Divergence on a corpus recorded after acceptance | zero | any |

The last row is the one that decides trust. If an accepted rewrite
diverges on sequences recorded later, the acceptance alphabet is too weak
and must be strengthened before anything else is tuned.

## 9. Where the loop belongs in spec-driven development

The owner develops with a spec-driven workflow in the Kiro style
(requirements, design, tasks, implementation, each gated by review), with
tests written by the implementation phase. The loop fits after
implementation as a deconvolution task with its own gate. It consumes what
that workflow already produces and returns artefacts the workflow can act
on.

- Inputs from the workflow: the spec's public contract, which the surface
  derivation checks itself against; the tests, which become the seeds;
  the host's baseline test failures, recorded as deselections.
- Outputs to the workflow: a missing-test report (surviving faults and
  reproduced differences, each with the input class that would
  distinguish it) as a task list for the next test iteration; the residue
  for the design's limitations section; the protection statement, with
  its numbers, for the validation report.
- The loop's own discipline (preregistered predictions, kill criteria,
  budgets, dated amendments) is the research-side counterpart of the
  workflow's phase approvals.

The regeneration step is where a model API endpoint enters. In the run
reported here the regenerator was an interactive agent, run once. For a
repeatable loop it becomes a program in the sense of DSPy: a signature
with fixed inputs (scope files, surface, contract, targets, and from the
previous round the first divergence, the measure gaps and the surviving
faults) and fixed outputs (candidate files, structure declaration, change
log); a hard metric (acceptance) and a soft one (measures against targets);
and a bounded optimisation loop of at most four turns per scope, where
each turn is fed only the first divergence and the measure deltas. Two
things this enables have not been done: bootstrapping the regenerator's
examples from accepted candidates across scopes, and comparing two
regenerator programs on the same frozen scope under the same oracle. The
second is the experiment that would show whether the loop or the model is
doing the work.

Detecting a repository's test infrastructure automatically is an
implementation concern, listed below. For research runs it is enough to
hand the loop the test files and deselections explicitly.

## 10. Open work, in order of how much it blocks the next step

1. Scope selection. Rejected once (R21); every scope so far was chosen by
   hand. Without it the loop serves a developer who already knows where
   the mess is.
2. Multi-round behaviour. Never observed; the one run accepted in round 1.
3. Canonical-format measures and the missing-test report. Both cheap,
   both unbuilt, both needed before the prediction in section 8 can be
   scored.
4. Test-infrastructure detection: runner, markers, fixtures, store setup.
5. Other languages. A Rust oracle exists (R20); no regeneration has run
   outside Python. The per-language tool stack is assessed in the
   project's tooling note: bounded model checking and cargo-mutants for
   Rust, CrossHair and cosmic-ray for Python, property tests only for
   JavaScript and TypeScript, where no mature concolic engine exists.
6. Cost and scaling. One scope took a working day of machine time.

## 11. Glossary

- Scope: the set of source files chosen for one run of the loop.
- Public surface: every name defined in the scope that anything outside
  the scope references, including tests and private names.
- Recorded corpus: sequences of surface operations recorded from the
  original with all observations; replayed against candidates.
- Level 1 / Level 2: observations of outcome (results, errors, stored
  state, files) versus observations of store interaction (query text and
  parameters).
- Kill rate: injected faults caught divided by faults caught plus faults
  not caught, incompetent faults excluded, equivalent faults kept.
- Pure core / effect shell: the declared split of a candidate's functions
  into those with no store access and those holding the connections and
  the write sites.
- Intra-scope edge: a call from one function in the scope to another.
- Concolic execution: running code on concrete inputs while collecting
  symbolic path constraints, then solving for inputs that flip branches.
- Verifier gates: the four checks that must pass before a solver verdict
  counts (symbolic-ness, encoding soundness, planted-fault and
  equivalent-renaming controls, concrete realisability).
- Residue: everything a run declares unprotected.
- Adaptive deadline: a preregistered base time scaled by a measured
  host-load factor.

## 12. References

Project records (research log entries, each with plan, receipts and logs):

- R19, Dynamic deconvolution direction. `docs/research/2026-11-03/01-dynamic-deconvolution-direction.md`
- R20, Session-bus trace oracle (Rust). `docs/research/2026-11-03/02-e1a-session-bus-oracle.md`
- R21, Facdrone edge census. `docs/research/2026-11-03/03-e2-facdrone-edge-census.md`
- R22, Evidence-publication oracle (Python). `docs/research/2026-11-03/04-e1b-evidence-publication-oracle.md`
- R23, Concolic rung as run. `docs/research/2026-11-03/05-e1c-concolic-rung.md`
- R24, Corrected solver rung with verifier. `docs/research/2026-11-04/01-e1c2-corrected-solver-rung.md`
- R25, E3 evidence regeneration. `docs/research/2026-11-04/02-e3-evidence-regeneration.md`
- Retrospective of the earlier static track, R1 to R18. `docs/research/2026-11-02/01-retrospective.md`
- Symbolic-execution tooling per language, including the KLEE assessment and the verifier rules. `docs/symbolic-execution-tooling.md`
- Deconvolution-loop specification: algorithm-agnostic requirements with per-requirement feasibility status, and a design documenting the prototype. `.kiro/specs/deconvolution-loop/`

External works used or assessed in this project:

- P. Godefroid, M. Y. Levin, D. Molnar. Automated Whitebox Fuzz Testing. NDSS 2008. And: SAGE: Whitebox Fuzzing for Security Testing. Communications of the ACM 55(3), 2012. The reference point for constraint-guided input generation over blind fuzzing.
- C. Cadar, D. Dunbar, D. Engler. KLEE: Unassisted and Automatic Generation of High-Coverage Tests for Complex Systems Programs. OSDI 2008. Assessed and not adopted: the targets are interpreted languages with database effects.
- L. de Moura, N. Bjørner. Z3: An Efficient SMT Solver. TACAS 2008. The solver under CrossHair.
- P. Schanely. CrossHair: a concolic analysis tool for Python, with `diffbehavior`. https://github.com/pschanely/CrossHair. Version 0.0.112 used.
- D. R. MacIver, Z. Hatfield-Dodds, et al. Hypothesis: A new approach to property-based testing. Journal of Open Source Software 4(43), 2019. The random-generation rung and the differential's soundness corpus.
- cosmic-ray, mutation testing for Python (version 8.7.0); cargo-mutants, mutation testing for Rust.
- O. Khattab et al. DSPy: Compiling Declarative Language Model Calls into Self-Improving Pipelines. arXiv:2310.03714, 2023. The frame for the regenerator as a program with a metric and a bounded optimisation loop.
- cc-sdd, Kiro-style spec-driven development for coding agents, as installed in this repository under `.agents/skills/kiro-*`.

Standard references for two techniques summarised above:

- W. M. McKeeman. Differential Testing for Software. Digital Technical Journal 10(1), 1998.
- R. A. DeMillo, R. J. Lipton, F. G. Sayward. Hints on Test Data Selection: Help for the Practicing Programmer. IEEE Computer 11(4), 1978.
