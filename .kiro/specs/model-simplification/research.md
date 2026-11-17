# What exists, what is missing, and how to attribute improvement

2026-11-17, 11:41 Europe/Berlin. Static analysis at `e877da216ece0b8eea766d48befc417965865408`.
Inputs: [brief](brief.md), current source, and the prior research below. Formal
requirements are not generated or approved. This analysis informs them; it is not
a requirements-validation result. No experiment or new baseline measurement ran.
Dated record: [R114](../../../docs/research/2026-11-17/01-model-simplification-measurement-attribution.md).

## Finding

Most execution machinery exists. What is missing is a defensible link between a
candidate rewrite and the claim that it made the responsibility simpler. Adding
model calls is not the principal uncertainty. Measuring scattered state without
mistaking local variables, unknown effects or moved code for improvement is.

Keep four outcomes separate: measurement fidelity, behavioral protection,
simplification, and adoption. Preserve the owner's decision: the simplicity result
does not change the existing step verdict or its exit code.

## What we have versus what we want

The links name exact source files; symbols identify the inspected contracts.

| Capability | Existing asset and behavior | Desired contribution | Gap |
| --- | --- | --- | --- |
| Branch discovery | [driver analysis](../../../native/src/adapters/driver/analysis.rs), `analyse`, returns pack-defined sites including `BranchCondition` | Comparable counts across the whole responsibility, with locations and supported constructs | **Missing:** aggregation/reporting in `step`; **Unknown:** adequacy of pack coverage as a simplicity measure |
| State discovery | [Python pack](../../../native/packs/python.json), [site rules](../../../native/src/adapters/driver/sites.rs), `allowed`, and `Analysis.write_sites` identify assignment/call sites and `file:line` labels | Separate local rebinding, identifiable nonlocal writes and unresolved effects; associate writes with state targets | **Missing:** target identities and writer relations; **Constraint:** these rules also select mutation sites |
| Candidate planning | [step plan](../../../native/src/step/plan.rs), `rows_of` and `run_plan`, report faults, tokens, writes and outside-frame counts; sort by unnoticed then noticed faults | Rank branch/state concentrations while keeping protection gaps visible | **Missing:** branch/state ranking policy and a test of its usefulness; a larger number is only a selection signal |
| Responsibility scope | [function attribution](../../../native/src/step/functions.rs), `FileFunctions`, matches names and scans name-plus-parenthesis calls | Include owned helpers, moved code, changed callers and outside-function sites on both sides | **Constraint:** same-name functions are grouped; indirect calls are not resolved. Existing caller scan is not a complete dependency boundary |
| Behavioral checks | [step runs](../../../native/src/step/check_runs.rs), `compare` and `judge_by_runs`, compare tests and faults against base and ledger | Reuse these checks and show their coverage limits beside simplicity | **Constraint:** fewer surviving faults can result from fewer mutation sites; acceptance is not equivalence |
| State frames | [frame check](../../../native/src/step/check.rs) and `outside_frame` use declared source-site labels | Preserve frame checks and distinguish them from state-distribution measures | **Constraint:** line-based identities move when lines are inserted; a permitted site is not a resolved state object |
| Structural metrics | [syntax size](../../../native/src/adapters/driver/measure.rs), [density command](../../../native/src/cli_deconvolute.rs), and [ScopeMetrics](../../../native/src/state_contract.rs) expose sizes or supplied aggregate counts | Report branches and state dispersion with support/uncertainty information | **Constraint:** density passes on token/node targets; `DeconvolutionOutcome` uses statement/line targets. Neither is the new simplicity judge |
| State semantics helpers | [NativeStateFrameAnalyzer](../../../native/src/state_frame.rs), `verify_dynamic_state`, compares supplied before/after maps | Reuse if a prototype supplies relevant observations | **Unknown:** observation coverage. This helper does not extract aliases or automatically observe all program state |
| Model proposals | [regenerator](../../../native/src/regenerator.rs), `build_prompt`, candidate preparation and receipts; [seam launcher](../../../native/src/seam.rs) calls the second binary | Supply branch/state context and preserve each proposed patch's provenance | **Missing:** task-specific proposal orchestration tied to step results; current launcher purpose is `regenerate` |
| Repair | [model repair](../../../model/src/repair.rs), `with_repairs`, invokes a caller's check and stops when it passes | Reuse bounded repair and actual failure reasons | **Unknown:** orchestration interface. A behavior-passing but unchanged candidate must not be relabeled a behavior failure just to obtain another repair |
| Repeated measurement | [model measure](../../../model/src/measure.rs), `Counts` and `Measured`, report first/within-repair acceptance and call/output-token totals; [task sets](../../../model/src/taskset.rs) use parsing/static checks | Measure behavior acceptance and useful simplification as distinct rates for real tasks | **Missing:** outcome vector and real step-check adapter. Existing “good enough” thresholds assess a named check, not simplicity |
| Reports and history | [step verdict](../../../native/src/step/verdict.rs), `Verdict`, binds revisions/hashes, checks, fault rows and size | Attach a versioned simplicity comparison and attribution record | **Missing:** simplicity facts and provenance link; **Constraint:** guard/ledger compatibility and old receipts |

The code has reusable primitives, not an already-qualified simplification pipeline.
No new library or general-purpose analysis framework is justified by this inventory.

## Proposed measures

These are candidates to challenge in the prototype, not accepted metric definitions.
Use a vector of facts rather than a weighted score that can hide regressions.
Every result names the source revisions, metric version, pack digest and scope.

| Question | Proposed observable | Interpretation and limits |
| --- | --- | --- |
| Are there fewer branch conditions? | `B`: unique supported condition sites over the complete responsibility, including helpers and module-level code; retain per-function locations | Existing pack sites can seed this count. Document whether conditionals, loops, match arms, comprehensions and returned predicates are covered; missing coverage is not zero complexity |
| Were branches merely folded into an expression? | `P`: supported atomic predicates and short-circuit decision points, with the exact counting rule frozen | A companion control, not a claim of full logical complexity. Rewriting two nested `if`s into one compound condition must not automatically earn simplification credit |
| Is writable state less scattered? | Relation `(state target, writer function, file)`; proposed `D = sum over targets of max(0, distinct_writer_functions - 1)` | Measures dispersion of identified writes, not all state coupling. Report the relation, target count, write-site count and files per target, not just D. Reads and effect ordering still matter to behavior |
| Is state knowledge sufficient? | `W`: identifiable nonlocal write sites; `L`: local rebindings; `U`: sites/effects whose target or effect is unresolved; explicit coverage gaps | A fall in W caused by a rise in U is not improvement. Calls are not assumed pure. Receiver spelling alone does not resolve identity; aliases and changed representations can make D incomparable |
| Did checks still hold? | Existing accepted/refused/not-judged result; base/candidate test results; noticed, unnoticed and other faults; named unprotected functions and adverse-control outcomes | Report exactly what ran. Invalid/timed-out/unsupported runs do not establish preserved behavior. Keep these separate from B/P/D |
| Did the model deliver useful candidates? | Behavior-accepted rate and useful-simplification rate, first reply and within the declared attempt budget | A useful candidate needs behavior acceptance, comparable measurement and the predeclared useful structural target. This is an experiment outcome, not a new guard condition |

For each comparable count, show before, after, signed change `after - before`, and
reduction `(before - after) / before` when before is positive. A zero baseline has
no percentage reduction; a missing measurement is unknown, never zero. Do not
compare raw counts between languages as though their packs had identical semantics.

For target matching, use owner/type or declared scope context where supported;
same-spelled fields on unrelated objects must not be merged. Preserve an explicit
mapping when representation changes. Renaming targets cannot itself lower D.
The existing name-based function grouping cannot be assumed sufficient here.

Suggested report states are `improved`, `unchanged`, `mixed`, `regressed`, and
`not measurable`. A proposed improvement needs a predeclared target-axis reduction,
no worsening of the declared companion controls, and comparable coverage. Mixed
trade-offs stay visible. An unknown state measure can coexist with a measured
branch reduction, but cannot support an overall claim of less scattered state.

Exact absolute/relative usefulness thresholds belong in the prototype gate after
a trustworthy baseline and before asking the model for candidates. One removed
condition establishes a count change, not automatically a useful simplification.
The specimen values, sample sizes, budgets and thresholds are currently **not measured
or frozen**. Initialization does not fill those blanks with invented numbers.

## Attribution: separate what changed

### 1. Measurement correction versus code improvement

Let S0 be unchanged source, S1 the candidate, M0 the current evaluator and M1 the
proposed evaluator. Preserve a four-cell comparison for any shared metric:

| | Unchanged source S0 | Candidate S1 |
| --- | --- | --- |
| Current evaluator M0 | M0(S0) | M0(S1) |
| Proposed evaluator M1 | M1(S0) | M1(S1) |

`M1(S0) - M0(S0)` is a measurement change on unchanged code. It is not model credit.
`M1(S1) - M1(S0)` is the candidate's measured change under one evaluator. Never
report `M1(S1) - M0(S0)` as the rewrite's benefit. Where M0 has no matching metric,
mark the cell unavailable and establish M1's baseline first. Do not force unlike
definitions into a numeric comparison.

Example, illustrative only: changing a reported write count from 12 to 4 on the
same file after excluding locals fixes the instrument; it removes no program state.

Correct classification must also retain mutation evidence. Freeze one pack for
both source revisions and separately list sites/operators lost or gained by the
pack change. A changed mutation population invalidates naive comparisons against
old ledgers. Designing compatibility or re-baselining is required; silently making
historical protection look better is not acceptable.

### 2. Better tests versus a better implementation

Keep original source with original tests as the historical reference. Establish
pins on original source, admit them against named faults, then freeze that test
suite before the rewrite. Compare original and candidate under the same pinned
suite and pack. Record newly detected bad behavior as a test-protection gain,
not a structural gain caused by the model. Retain known-bad candidates to challenge
the checker; do not expose any declared held-out controls in repair prompts.

### 3. Model contribution versus human assistance and selection

Retain the raw answer, extracted patch, checked patch, hashes and every manual edit.
Record who selected the responsibility, wrote pins, supplied a concrete rewrite
hint, repaired code or changed the boundary. Human-edited candidates belong in an
assisted category, not an unassisted model-success numerator. Even an unedited
model answer can be human-guided; retain that distinction in its context record.

The first two files are owner-selected case studies. They can show a useful step,
not that the ranking found it or that arbitrary repositories will improve.

### 4. Repair contribution versus another chance

Compare the first answer with the final candidate and report how many initially
unsuccessful trials were recovered. That is observed recovery within a budget.
To claim the feedback itself caused improvement, add a paired fresh-retry arm
without failure feedback, using the same task set, model/settings and attempt
budget; vary arm order and preserve all outcomes. This stronger attribution is
optional unless such a causal claim is made. No optimizer comparison is needed.

The existing repair loop stops on check acceptance. Under the owner's reporting
decision, a behavior-passing non-improving answer may end as “accepted, unchanged.”
Any further search for simplicity needs a separately specified bounded policy;
do not manufacture a behavior refusal to reuse repair.

### 5. Ranking contribution versus case-study choice

Compare current fault-based ranking and proposed branch/state ranking over the same
eligible population. Freeze k, tie handling, the proposal method and budgets; reuse
results for overlapping candidates and retain each ranker's selected set. Compare
useful-candidate yield per attempted responsibility, with all failed and unmeasurable
cases visible. A ranking's correlation with its own branch/state score proves
nothing about its ability to find useful changes. This can follow the first two
case studies; it is necessary before claiming better selection.

## Minimum outcome record

Reuse existing receipts/verdicts and link them rather than duplicating their contents.
A trial record needs: responsibility ID and scope manifest; original/candidate hashes;
evaluator and pack identity; test/pin identity; model, endpoint and sent settings;
prompt/context hashes; raw answer and candidate path; attempt/rejection reasons;
behavior result; simplicity vector and gaps; intervention category; adoption state.
Pin runtime, dependency locks and tool revisions in the run manifest. Keep secrets out.

Report a flow of counts: scheduled, attempted, endpoint-failed, candidate-produced,
behavior-accepted, measurable, useful and adopted. Report useful/scheduled and
useful/attempted, plus behavior-accepted/attempted; name the denominator every time.
Missing outcomes remain visible. Keep the seam's existing not-run convention intact
in its own report; do not silently reinterpret its acceptance rate as useful yield.
If trials repeat a responsibility, report repetitions and unique responsibilities
separately. Give per-case results before aggregates; a larger file must not hide a
failed state case. Record calls/tokens as resource usage, not code-quality rewards.

Adoption means a candidate was deliberately applied and recorded in the intended
project. A patch accepted in a disposable clone is not upstream adoption.

## Controls that could refute the measures

| Control | Required interpretation |
| --- | --- |
| Same source, same metric/pack/scope | Same measurements; nondeterminism prevents comparison |
| Whitespace/comment-only rewrite | No B/P/D improvement |
| Purely local assignments added or removed | No nonlocal-state improvement; mutation coverage changes remain visible |
| Branches moved to a new helper or file | Count the complete replacement; no credit for escaping the accounting boundary |
| Nested conditions folded into a compound expression | P and semantics must expose retained decision work; a smaller B alone is insufficient |
| Writes routed through a generic setter | Inspect indirect write responsibility; unresolved targets prevent a claimed D gain |
| Functions merged without removing decision/state work | Fewer writer functions alone cannot establish useful simplicity; companion metrics and responsibility review must expose the trade-off |
| Alias, dynamic attribute or unresolved call introduced | Report uncertainty; do not label it pure or reward fewer known writes |
| Required output, side effect or failure behavior removed | Behavioral controls must detect the change; otherwise protection is inconclusive even if structure shrinks |
| A genuine simpler candidate is larger in tokens | Size safeguards still apply as declared, but extra tokens do not negate a measured structural reduction |

Adverse controls qualify the measurement. Real `exporter.py` and `tools.py` steps
qualify usefulness. Synthetic controls alone cannot pass the real-code prototype gate.

## Implementation options and risk

| Option | Integration | Trade-off |
| --- | --- | --- |
| Extend existing structures only | Add fields to pack analysis, plan, verdict and current regenerator task context | Lowest duplication, but risks conflating mutation-site semantics with state metrics and enlarging existing modules |
| Separate measurement/proposal component | Consume source and existing step outputs; emit a linked analysis report | Easier to isolate early experiments, but risks a second parser/checker and inconsistent identities |
| Small shared measurement layer plus existing execution flow | Reuse pack/tree primitives and report facts to both plan and verdict; place bounded proposal orchestration outside the model-free step/driver | Recommended for prototyping; exact boundaries depend on demonstrated state fidelity. Avoid a second behavior judge |

Provisional effort **L** (the gap-analysis scale's 1-2 weeks), with **high research
risk**, not a delivery estimate or authorization: parsing/reporting are familiar,
but generic state identity and sound attribution are unproven. Narrow to supported
state forms if the gate demands it, labeling exclusions rather than fabricating
precision. No dependency change is proposed.

Compatibility review must cover `step-plan-v1`, step verdict/ledger consumers,
guard behavior, pack admission and mutant caches. Relevant existing tests include
`native/tests/step_plan.rs` (current ranking and standalone-call write counts),
`step_check.rs`, `step_functions.rs`, `step_guard.rs`, and model repair/measure tests.
These were inspected where relevant, not executed in this documentation task.

## Contract tensions and next decisions

| Existing commitment | Required disposition in requirements/design |
| --- | --- |
| Gradual steps Requirement 2 ranks by unnoticed faults | Explicitly define the new ranking or selectable mode; retain protection evidence |
| Model seam keeps step flow and driver free of model calls | Keep proposal orchestration outside these deterministic components; reuse seam transport |
| Existing density/compression gates use token, statement or line targets | Preserve their historical meaning; do not inherit them as this feature's simplicity target |
| Pack `StateWrite` feeds both analysis and mutations | Decide whether corrected metric facts and mutation eligibility need separate declarations; validate lost/gained faults |
| Old state-boundary specs contain stronger semantic claims | Do not import them as proof that the current step flow observes all state or proves equivalence |
| Simplicity is reporting only | No changed step acceptance/guard policy; experimental useful-yield classification remains separate |

Before prototype execution, resolve supported branch constructs and state identities,
matching across revisions, stable responsibility boundaries, report compatibility,
and treatment of accepted-but-unchanged candidates. Define baselines, targets, budgets
and stop conditions. Also resolve the specimen Python test-runtime route against the
current PowerShell/no-WSL execution rules. No production tasks follow from this analysis.

## Prior evidence

- [R94](../../../docs/research/2026-11-12/01-first-python-step.md): the output-changing
  candidate passed without pins and failed with them. A passing step alone is insufficient.
- [R95](../../../docs/research/2026-11-12/02-second-python-step-frame-release.md): local
  assignments inflate writes, keyword-argument mutation is absent, and frames move with lines.
- [R99](../../../docs/research/2026-11-12/06-repair-gate-result.md): bounded repair
  improved pack admission; this is not evidence of real-code simplification.
- [R110](../../../docs/research/2026-11-14/03-model-seam-indicator-on-real-models.md):
  the built-in static-check indicator is a floor, not a model ranking or a simplicity result.
- [R113](../../../docs/research/2026-11-16/02-model-seam-release-and-final-validation.md):
  the seam's release-form integration was exercised. It supplies execution evidence,
  not the missing simplification evidence.

The next deliverable is reviewed requirements derived from these gaps, followed by
a provisional design and [prototype gate](../../steering/prototype-first.md).
