# Prototype gate: agnostic annotation-guided driver

Status: **revise**, 2026-11-10, attempt 1 of 2 ([R61](../../../docs/research/2026-11-10/07-annotation-driver-gate-result.md)); attempt 2 authorized by the owner and specified in Amendment 10, not yet run. Rust, Python and Java meet every gated row in at least one arm; TypeScript fails in every arm, so the agnosticism row is not met. Amendments 3 to 8 govern where they differ from the original text. A second attempt needs the owner's go. Research preparation only. No production
tasks exist or are authorized. Execution needs the owner's go, because it
selects external scopes and spends model tokens. See the [brief](brief.md) and
[R56](../../../docs/research/2026-11-10/02-agnostic-annotation-driver-direction.md).

## Hypothesis

H-AD1: given one frozen annotation guide and a scope the guide was not written
for, a model returns located annotations that a deterministic validator
accepts, and generic operators applied at those sites produce mutants that
land and that the scope's own tests classify. The same guide works, unchanged,
on languages it was not developed on.

What could disappear if this holds: the specimen-bound operator bodies in
`native/src/adapters/{rust,java,typescript}/mutation.rs` (about 450 lines) and
the need to write such operators for each new language and codebase. What must
replace them: an annotation contract and validator, a generic operator set,
one node-kind table per language, and the guide. The saving is an estimate
until measured.

## Arms

| Arm | What it does | Role |
| --- | --- | --- |
| A | Annotation-guided: model annotates, validator checks, generic operators mutate | Mechanism under test |
| B | Existing fixed operators (`lesscode-core deconvolute mutate`) | Frozen control; expected to yield no mutants on these scopes |
| C | One existing mutation tool per language (cargo-mutants, cosmic-ray or mutmut, StrykerJS, PIT), restricted to the same files | Comparison; shows what a non-model tool already gives |

Arm A uses the existing endpoint seam (`LESSCODE_MODEL_ENV`, Opus). The model
identity, request parameters and raw responses are retained. No prompt
optimisation, no multi-turn repair and no human edit of annotations is
allowed inside a counted run.

## Scopes and admission

Four scopes, one each in Rust, Python, TypeScript and Java. Each must:

- be outside the experiment specimens (not the files or responsibilities used
  in R49 to R53, and not containing the substrings the fixed operators match);
- be a complete responsibility of 150 to 800 canonical lines with its own
  tests that pass in an isolated copy;
- have a recorded revision, license and dirty-tree hash.

Candidates to screen (not yet admitted): a module of `mmhmem` (Rust), a
Facdrone or `mvibe` subsystem other than the R53 specimen (Python), an
`OpenUltraCode` module other than the R53 specimen (TypeScript), a
`commons-csv` class other than `CSVPrinter` (Java). If a language has no
admissible scope, that language is reported as not-run; it is not replaced by
a synthetic fixture.

Development and held-out split, frozen before any model call: the guide is
written and adjusted using the Rust and Python scopes only. It is then frozen
by hash and run unchanged on the TypeScript and Java scopes. Only the
node-kind table may differ per language.

## Annotation contract, version 0

One JSON object per annotation: `file`, `start_line`, `end_line`, `kind`,
`anchor` (the exact source text of the annotated node), `reason`. Kinds:
`failure_state`, `branch_condition`, `error_order`, `state_write`,
`pure_shell_boundary`.

The validator accepts an annotation only if all of these hold, and records
the rejection reason otherwise:

1. the file is inside the frozen scope and is not a test or generated file;
2. `anchor` occurs at the stated lines, byte for byte;
3. the smallest syntax node covering the anchor has a node kind that the
   language's table allows for the annotation kind;
4. the site is not inside a comment or string literal;
5. it is not a duplicate of an accepted annotation.

## Research checklist

1. Screen and admit scopes; freeze inputs, the split, the taxonomy, the
   validator rules, the node-kind tables and the targets below. Record them in
   a dated research entry before the first model call.
2. Validator controls first, without a model: feed hand-made wrong
   annotations (wrong line, altered anchor, site in a comment, site in a test
   file, kind and node mismatch, duplicate) and a no-op mutant. All must be
   rejected or classified invalid.
3. Establish a healthy baseline: the scope's tests pass in the isolated copy
   used for mutation. Run arm B and arm C.
4. Write guide version 0 against the Rust and Python scopes. Freeze it.
5. Run arm A twice per scope with identical settings on all four scopes.
6. Review every surviving mutant by hand and label it equivalent, real test
   gap or invalid. This review is a manual intervention and is reported as
   such.
7. Count the prototype's lines by part (contract and validator, operators,
   per-language tables, guide, glue) and estimate what would enter
   `native/src`.

## Targets, falsifiers and decision

All targets are fixed here before execution. Changing one needs a dated
amendment made before the run it applies to.

| Dimension | Proceed condition | Otherwise |
| --- | --- | --- |
| Validator integrity | 100% of the hand-made wrong annotations rejected; the no-op mutant classified invalid | stop until fixed; no model run counts |
| Validity | On each scope, at least 80% of returned annotations pass validation, and no rejected annotation is ever applied | revise the guide (development scopes only) or the contract |
| Yield where the control has none | On each scope, at least 4 landed mutants covering at least 3 kinds, while arm B produces 0 | revise or stop |
| Oracle | Every landed mutant is classified by the real tests against a healthy baseline in the same copy; harness errors and timeouts are not kills | gaps block |
| Meaningfulness | At least 70% of landed mutants are killed or are labelled a real test gap in review | revise; equivalent-mutant rate reported |
| Agnosticism | The frozen guide meets the validity and yield targets on both held-out languages; no scope-specific string in guide, operators or tables; each language table is at most 40 lines | revise; report which language failed and why |
| Comparison | Arm C results reported for the same files; if arm C alone gives equal or better meaningful mutants at lower cost on all four scopes, the recommendation is to adopt the existing tools and stop this mechanism | stop is a valid outcome |
| Apparatus | Estimated production size fits the R41 ceiling of 7,173 canonical lines together with the removal of the fixed operator bodies | owner-reviewed amendment before production, not an automatic pass |

Reported, not gated (baseline for the second priority): agreement between the
two runs of arm A per scope, as the Jaccard index over validated annotation
sites and over kinds; token use and cost per scope.

Outcome is `proceed`, `revise`, `stop` or `not-run`. `proceed` needs every
gated row on at least three of the four languages, including at least one
held-out language, and no failing validator control. Only `proceed` followed
by ordinary design approval unlocks production task generation.

## Bounds

At most two attempts. Per attempt: two runs of arm A per scope, at most two
model calls per run, 64k output tokens per call, a total spend cap of 20 USD
(a dated pre-run amendment is needed to raise it). Per-stage limits: 1,800
seconds for a build or test run, 900 seconds for a mutation batch. Work in
isolated copies under `D:\BuildArtifacts\lesscode\annotation-driver`; keep
prototype scripts and retained results under
`docs/experiments/annotation-driver/`. Linux-only tools run through a Compose
file in that directory. Nothing is adopted into `native/src` during the gate,
and the released commands are not changed.

## Amendment 1, 2026-11-10 (pre-run; after scope screening, [R57](../../../docs/research/2026-11-10/03-annotation-driver-scope-admission.md))

Made before any model call and before any arm A, C or D run.

**1a. Admitted scopes** (owner go "proceed", 2026-11-10; isolated clones under
`D:\BuildArtifacts\lesscode\annotation-driver\scopes`, baselines healthy):

| Language | Role | Scope | Canonical lines | Baseline tests |
| --- | --- | --- | --- | --- |
| Rust | development | `mmhmem` `src/setup/check.rs` @ `f32596f` | 212 | 28 passed (lib `setup::check` and `tests/setup_check.rs`) |
| Python | development | `mvibe` `mvibe_proxy/ratelimit.py` @ `b56d0e3` | 478 | 230 passed (`tests/test_ratelimit.py`) |
| TypeScript | held-out | `OpenUltraCode` `src/state.ts` @ `37a4e55` | 188 | 8 passed (`tests/state.test.ts`) |
| Java | held-out | `commons-csv` `Lexer.java` @ `2bd66bf` | 319 | 37 passed (`LexerTest`, Maven in a container) |

**1b. Correction of the control expectation.** Arm B was expected to produce
no mutants on these scopes. Measured: 0 on the Rust, Python and Java scopes
and 1 on the TypeScript scope. The yield row therefore reads: arm A lands at
least 4 mutants covering at least 3 kinds on each scope **and lands more than
arm B on that scope**. Nothing else in that row changes.

**1c. Arm D, proposed by the agent, pending the owner's confirmation.** Not
part of the gate until confirmed; recorded here so it cannot be added after
results are known.

| Arm | What it does | Role |
| --- | --- | --- |
| D1 | Rules as data: the generic operators are written once as declarative syntax-tree rewrite rules (ast-grep rule files, one small pack per language) and applied at every matching site in the scope. No model at run time. | Tests whether a deterministic, data-only driver is already agnostic enough |
| D2 | The rule packs for the two held-out languages are written by the model, offline, from the frozen guide and the two development packs, and admitted only if they pass the validator controls and fixture checks. No model at run time. | Tests the model as an offline rule author instead of a per-scope annotator |

Arm D uses the same scopes, validator, oracle, review and targets as arm A.
Decision rule if confirmed: when D1 or D2 meets the gated rows on the same
languages as arm A, the recommendation is the D form, with per-scope
annotation kept only as an optional ranking of sites. The reason is the
second priority: a driver with no inference at run time has no run-time
inference drift, and the remaining model output is a small, reviewable,
hash-pinned artifact. If arm D fails where arm A passes, arm A stands.

## Amendment 2, 2026-11-10 (pre-run; [R58](../../../docs/research/2026-11-10/04-annotation-driver-validator-controls.md))

Made before any model call and before any arm A, C or D run.

**2a. Arm D is in force.** The owner approved the proposal of Amendment 1c on
2026-11-10. Arms D1 and D2 run on the same scopes and are judged by the same
rows as arm A, with the decision rule stated there.

**2b. The hard-coded operators are to be removed completely** (owner,
2026-11-10: "we need to fully remove hardcoded arms"). Consequences:

- Arm B exists in this gate only as a measurement, taken with the released
  1.0.1 binary. It is not a component of any outcome.
- A `proceed` design must delete the operator bodies in
  `native/src/adapters/{rust,java,typescript}/mutation.rs` and every
  specimen-specific string. They are not kept as a control set; this
  supersedes the corresponding condition in
  [constraint-guided-compression](../constraint-guided-compression/requirements.md)
  Amendment 5. A model-independent reference for drift comes from the
  hash-pinned rule packs instead.
- The agnosticism row also covers rule packs: no scope-specific string in the
  guide, the operators, the tables or the rule packs.
- If the gate ends in `revise` or `stop`, the removal requirement remains;
  what the released command does in the meantime is the owner's decision.

**2c. Validator rules as implemented and frozen** (contract version 0; the
five rules above, made precise):

- Rule 2: the anchor must be exactly the text of one syntax node that starts
  and ends on the stated lines. Text that is only part of a node, or that
  matches nodes at more than one position in those lines, is rejected
  (`anchor_mismatch`, `anchor_ambiguous`).
- Rule 3: the kind check uses the node, any wrapper or wrapped node covering
  the same text, and a parenthesis wrapper where the language table names
  one, so an anchor on the expression inside `if (...)` counts as the
  condition. `error_order` also needs a following sibling of the same kind.
- Rule 1 also rejects sites under a test attribute named in the language
  table (`test_region`), and annotations with missing or inconsistent fields
  are rejected as `malformed`.
- A mutant is `changed` only if it parses without error nodes and its token
  sequence, ignoring comments and whitespace, differs from the original.

Frozen by hash in R58: `validator.py`, the four tables (14 to 17 lines each),
`scopes.json`, `controls.json`, `run_controls.py`. Changing any of them after
the first model call needs a dated amendment.

**2d. Validator integrity row: met.** 71 of 71 controls behaved as expected
on the four scopes (R58). This validates the checker, not the mechanism.

## Amendment 3, 2026-11-10 (pre-run; strategy consolidation, [R59](../../../docs/research/2026-11-10/05-annotation-driver-strategy-and-harness.md))

Made before any model call and before any mutation run. The owner clarified
that "hard-coded arms" means non-agnostic runs, those that do not apply
horizontally, and asked the agent to settle the strategy directly. Where this
amendment conflicts with the text above or with Amendments 1 and 2, this
amendment governs.

**3a. Only horizontally applicable arms remain.**

- Arm B (fixed operators) is removed as an arm. Its four counts in R57 stay
  on record as history; it is not run again and no row refers to it. The
  yield row is the absolute floor only: at least 4 landed mutants covering at
  least 3 kinds per scope.
- Arm C (one existing mutation tool per language) is removed. A different
  tool, setup and operator set per language cannot apply horizontally, so it
  cannot be an outcome under the first priority. The Comparison row and the
  "stop in favour of existing tools" outcome are deleted. Whether an existing
  tool is useful inside one language is not examined here.
- The product consequence of Amendment 2b is unchanged: nothing
  non-horizontal remains after a `proceed` design.

**3b. One mechanism, two ways of choosing sites.** All language knowledge is
one data pack per language: the node-kind table of Amendment 2c plus operator
templates. The engine is the same for every language and reads only the pack.

| Kind | Generic operator | Language data needed |
| --- | --- | --- |
| `branch_condition` | replace the condition by its negation | one negation template |
| `failure_state` | remove the failure: drop a raise or throw statement, or stop propagating an error | per node kind, a replacement template; node kinds without one yield no mutant |
| `state_write` | drop the write when it is a statement of its own | one empty-statement template and the statement wrapper kinds |
| `error_order` | swap the check with the following check | none |
| `pure_shell_boundary` | none (kept for the analysis consumer) | none |

The arms differ only in how sites are chosen:

| Arm | Sites | Model |
| --- | --- | --- |
| D1 | every site the pack makes eligible, with hand-written packs for all four languages | none |
| D2 | every eligible site, with packs for TypeScript and Java written by the model offline from the guide, the Rust and Python packs and the grammar's node kinds; a model-written pack is admitted only if the scope's hand-made controls pass with it | once per language, offline |
| A | the sites the model annotates under the frozen guide, validated as in Amendment 2c, hand-written packs | per scope and run |

Because the engine and operators are shared, a difference between A and D1
measures exactly what per-scope annotation adds. The hand-written TypeScript
and Java packs are the reference for D2; agreement between each model-written
pack and its reference is reported.

**3c. Admission also requires enough sites.** A scope is admitted only if its
pack makes at least 12 sites eligible across at least 3 kinds. Counted with
the frozen tables: Python 200 sites, Java 157, TypeScript 51, and the Rust
scope of Amendment 1a only 5 sites in 2 kinds. That scope could not meet the
yield row whatever the mechanism did, so it is replaced, before any run, by
`mmhmem` `src/setup/ui/update.rs` at the same revision (229 canonical lines,
38 sites in 4 kinds, 17 tests passing in the clone). It had the most eligible
sites among the six Rust files screened with passing tests.

**3d. Bounded runs.** Arm D runs at most 40 mutants per scope, chosen by a
seeded shuffle (seed 20261110) taken round-robin across kinds, so every kind
present is represented. Arm A runs every validated annotation that has an
operator.

**3e. Classification.** A mutant is `invalid` if it is a no-op or does not
parse (Amendment 2c), if the scope's build step fails, or if the test runner
reports that it could not load or collect the tests. It is `killed` only if
tests ran and at least one failed, `survived` if all passed, and `timeout` if
the time limit was hit; a timeout is neither a kill nor a survival. The
unmutated scope must pass before and after every batch, and the scope file
must be restored byte for byte, or the batch is void.

**3f. Roles.** Strategy and gate decisions are made by the session agent.
Clearly specified implementation tasks are delegated to Opus at the highest
effort setting, on the owner's instruction. This concerns who writes the
prototype; the model under test in arms A and D2 remains the Opus endpoint
configured through `LESSCODE_MODEL_ENV`.

**3g. Revised checklist.** Steps 1 and 2 are done. 3: engine, operators,
packs and harness, with model-free checks, then D1 on all four scopes. 4:
guide version 0 from the Rust and Python scopes; freeze guide and packs by
hash. 5: arm A twice per scope and D2 on the two held-out languages. 6:
review of survivors. 7: size accounting. The decision rule of Amendment 1c
stands: if D1 or D2 meets the gated rows on the same languages as A, the
recommendation is the D form.

## Amendment 4, 2026-11-10 (pre-run; after the engine and harness were built, [R59](../../../docs/research/2026-11-10/05-annotation-driver-strategy-and-harness.md) addition)

Made before any arm batch and before any model call. Each point comes from
building and checking the engine; none comes from a mutation result.

- **4a. Python test command.** The scope's own pytest settings run parallel
  workers, under which an ordinary test failure and a collection error are
  not distinguishable by exit code. The command now runs serially (`-n 0`):
  a failing test exits 1 (killed), a collection error exits 2 (invalid).
  Baseline unchanged, 230 passed.
- **4b. The cap of 3d applies to mutants.** Arm D selects among the eligible
  sites that have an operator, so the 40 are 40 mutants. Sites without an
  operator are reported, not drawn.
- **4c. Validator rule 2 counts named syntax nodes only.** A lone bracket
  that happened to be unique on its line was accepted as an anchor. It is now
  rejected, and a control for it is added (76 controls, all as expected).
- **4d. TypeScript pack.** The empty condition of `for (;;)` is excluded
  (`empty_statement`); negating it cannot parse. This edit to a held-out
  language's hand-written pack was made after seeing that scope, so D1 on
  TypeScript is not a clean held-out result. D2, whose pack the model writes
  without seeing the scope, remains the agnosticism test.
- **4e. Sites that can yield a mutant**, with these packs: Rust 16 of 33
  eligible, Python 161 of 200, TypeScript 28 of 50, Java 115 of 157. Most of
  the rest are `return` statements labelled as failure sites.

## Amendment 5, 2026-11-10 (operational; during D1, before any Java result)

The first Java D1 batch was stopped by hand at its seventh mutant, which
looped forever; the limit of 900 s per run would have cost 15 minutes for
every such mutant, and the test container kept running after the harness
gave up. No Java mutant result from that batch is used. The Java limit is
now 180 s per run (the unmutated run takes 10 to 75 s) and the container
ends itself after 240 s. Classification is unchanged: a run that hits the
limit is `timeout`, neither killed nor survived. The clone was restored from
its commit and its file hash matches R57. D1 for Rust, Python and TypeScript
had already finished and is not affected.

## Amendment 6, 2026-11-10 (before any D2 call): how model-written packs are made and admitted

Fixed while the counted arm A mutation batches were running and before any
pack-authoring call. It makes Amendment 3b's D2 precise.

- **Input to the model.** A pack authoring guide (`pack-guide-v0.md`,
  SHA-256 `2a6eab36fb8a4ebbb20c7369e832bef22306648feccbacf15a9fdde7990abe44` at the time of
  writing) as the system message; the example packs and the name of the
  target language as the user message. The model is not given the grammar's
  node kinds, the scope, or the hand-written pack of the target language.
- **Developing the guide without the held-out languages.** The guide may be
  revised only from leave-one-out trials on the development languages: the
  model writes the Python pack from the Rust pack alone, and the Rust pack
  from the Python pack alone, and each is checked as below against its own
  scope. The guide is then frozen by hash before any TypeScript or Java
  call.
- **Admission, all deterministic.** A model-written pack is admitted only if
  it is one JSON object of at most 40 lines and 2,500 characters with the
  expected fields, every node kind it names exists in the grammar, its
  templates use only the known placeholders, the scope's hand-made controls
  all behave as expected with it, and every control seed that has an
  operator yields a real change.
- **One repair.** If the first pack is not admitted, the model gets the list
  of failed checks (names of invalid kinds, labels of failing controls; no
  source text) and one more attempt. A pack not admitted after that is a D2
  failure for the language.
- **D2 run.** For TypeScript and Java the examples are the Rust and Python
  packs. An admitted pack is run exactly like D1 (same seed and cap) and
  judged by the same rows. Agreement with the hand-written pack is reported
  as the share of eligible sites the two packs have in common.
- **Counting.** Authoring calls count toward the 20 USD cap. The two
  development trials are not counted runs.

## Amendment 7, 2026-11-10 (before any TypeScript or Java pack call): pack guide frozen as version 1

Leave-one-out trials on the development languages, as Amendment 6 allows:

| Guide | Trial | Result |
| --- | --- | --- |
| version 0 | Python pack from the Rust pack | admitted on the first attempt |
| version 0 | Rust pack from the Python pack | no pack: the endpoint refused both attempts (`stop_reason: refusal`, category `cyber`) and returned no text |
| version 1 | Rust pack from the Python pack | admitted on the first attempt |
| version 1 | Python pack from the Rust pack | admitted on the first attempt |

Version 1 differs from version 0 only by an opening paragraph that says what
the work is: mutation testing of a project's own test suite, in isolated
copies that are discarded. That is a true description added for clarity; it
hides nothing and no wording was removed. The refusal is recorded as a
property of the endpoint: a request about rewriting code to remove failure
handling can be declined when its purpose is not stated. A refusal is a
failed authoring attempt, never a pack.

Frozen for the TypeScript and Java calls: `pack-guide-v1.md`, SHA-256
`6fd47bf4246dc90089b58b13a73a86a539ef1e108898c98c3809c1de9343abc4`. If the endpoint refuses a held-out language with this guide, that is
a D2 failure for the language; the guide is not reworded again.

## Amendment 8, 2026-11-10 (operational; owner: the runs are too slow)

The harness now keeps the outcome of each mutant, keyed by the original
file's hash, the mutated text's hash and the scope's build and test
settings, and reuses it when the same mutant comes up again in another arm
or run. A reused outcome is marked `cached` in the receipt and names the
batch that first ran it. `AD_RESULT_CACHE=off` disables reuse.

Why this does not change results: a mutant's outcome depends only on those
three things. The store was first filled from the batches already run; 165
mutants had been run more than once across D1, arm A and TypeScript D2, and
every repeat had the same class as the first run. The one way this can
mislead is a run that hit the time limit for reasons of load, not of the
mutant; timeouts are reported with that caveat.

Applies to Java arm A run 2 and Java D2, which are rerun or run with it.
`harness.py` SHA-256 `337b47e75921d3e94cca7aa5d272ec57f9708eda5447c26bca0af84fdbfd13c6`.

## Amendment 9, 2026-11-10 (operational; after attempt 1, owner: the serial runs make no sense)

How batches run, from now on ([R62](../../../docs/research/2026-11-10/08-annotation-driver-parallel-batches.md)):

- Mutants of a scope run concurrently in independent worker copies that
  pull from one queue (`run_arm.py --workers N`). Rust is limited to one
  worker: concurrent builds of that scope exhaust memory.
- A run may take five times what the unmutated run needed on that worker,
  with a floor per scope, in place of the fixed limits of Amendment 5.
- Java runs in one long-lived container per worker, compiling only the
  scope file and running the one test class.
- One outcome store is shared by all workers and the serial path
  (Amendment 8).
- New class `error`: a test command that did not start, or a build that ran
  out of memory, is not a result about the mutant. It is retried once, never
  stored, and reported. A failing exit counts as killed only when the test
  runner's own output shows that it ran. `error` mutants are outside every
  row's numerator and denominator, and a batch with any left is incomplete.

Verified before use: with reuse off, every D1 mutant of the four scopes gets
the same class as in the serial receipts of R60. None of the 240 kills of
attempt 1 was a launch failure. Classification of real test runs is
unchanged.

## Amendment 10, 2026-11-10 (before attempt 2; owner go: "Full revision")

The owner chose the full revision for the second and last attempt, an
independent agent for the survivor labels, and separately that the released
commands which pass without doing the work must fail loudly now (a native
change outside this gate). Everything below is fixed before any attempt 2
run. Targets and the `proceed` rule are unchanged from attempt 1.

**10a. Pack format version 1** (data only; the engine stays free of
language and project names). Additions to each language pack:

| Addition | What it does |
| --- | --- |
| `kinds.state_write.call_statements` | A statement that consists of a call (or an awaited call) whose value is not used is a write site; the existing operator drops the statement. Covers state changed through calls. |
| `kinds.branch_condition.returned` | A returned expression is a condition site when its node kind and one of its own operator tokens are in the pack's lists. Covers predicate functions. |
| `operators.branch_condition.token_swaps` | Pairs of operator tokens. Each such token inside a condition site yields a mutant with its partner (at most two per site, in source order), in addition to the negation. These keep the types intact. |
| `operators.failure_state.returned_literal_swaps` | A return that signals failure by a literal in the pack's list yields a mutant returning the partner literal. First operator for `return`-based failure sites. |
| `statement_terminator` | An anchor that omits the statement terminator is accepted when the anchor plus the terminator is exactly one node. |

A site may now yield several mutants. Packs stay within 40 lines.

**10b. Guides.** The annotation guide and the pack authoring guide are
revised to describe 10a, using the Rust and Python scopes and leave-one-out
pack trials only, then frozen by hash before any TypeScript or Java call.

**10c. Scopes.** The four scopes of attempt 1 stay. A second TypeScript
scope is added by rule: among `OpenUltraCode` source files of 150 to 800
canonical lines with their own passing test file, other than `state.ts`,
`slugify.ts` and `degradation.ts`, the one with the most eligible sites
under the version 1 hand pack, provided it has at least 12 sites in at
least 3 kinds. A language meets a row only if all of its scopes do.

**10d. No sampling where it is affordable.** Arm D runs every mutant of
every eligible site when a scope has at most 200 mutants; above that, 200
chosen by the seeded, kind-balanced selection. Batches run in the worker
pool of Amendment 9.

**10e. Arms.** D1 with hand-written version 1 packs on all scopes; D2 with
packs the model writes for TypeScript and Java from the Rust and Python
version 1 packs (admission as in Amendment 6, one repair); A with the frozen
guide, two runs per scope. The decision rule of Amendment 1c stands.

**10f. Labels.** Survivors are labelled by an independent agent that sees
only the code, the tests and the change, not the designer's labels. The
meaningfulness row uses the independent labels. The designer's labels are
reported next to them with the disagreements. The same check is applied to
the 36 survivors of attempt 1.

**10g. Bounds.** This is the second of two attempts. Model spend for both
attempts together stays within the 20 USD cap. `error` mutants (Amendment 9)
must be zero in every counted batch or the batch is rerun.

## Amendment 11, 2026-11-10 (before any attempt 2 call or batch; [R63](../../../docs/research/2026-11-10/09-annotation-driver-attempt2-setup.md))

- **Second TypeScript scope admitted** by the rule of 10c:
  `OpenUltraCode` `src/verification.ts` (183 canonical lines, 64 eligible
  sites in 4 kinds, 7 tests passing). The other qualifying file,
  `config.ts`, had 59 sites.
- **Designer labels become a cross-check.** In attempt 1 the independent
  labels and the designer's agreed on 32 of 36 survivors with no opposite
  verdicts. In attempt 2 the independent agent labels every survivor and
  those labels decide; the designer labels a seeded sample of 30 survivors
  (seed 20261110) and the agreement is reported. This replaces the last
  sentence but one of 10f.
- **Inputs frozen** by hash in R63: engine, validator, harness, pool, packs
  (version 1), scopes and controls. The two guides are drafts until the
  development calls on Rust and Python are done.

## Result of attempt 1, 2026-11-10

Outcome `revise`; full tables and limits in R61. No design approval and no
production tasks follow from it. The proposal for attempt 2 in R61 is not in
force until the owner accepts or changes it and it is recorded here as a
dated amendment before any run.

## Evidence record

Screening and admission are recorded in R57 (Rust scope replaced in R59), the validator controls in R58, the engine checks in the R59 addition, arm D1 in R60, arms A and D2 and the gate result in R61.
No arm A, C or D result exists yet. Execution begins with a dated research entry that
freezes the inputs, and ends with one that reports each row above, the raw
annotations, validator decisions, mutants, test logs and the review labels.
A validator control validates the checker, not the mechanism.
