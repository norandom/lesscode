# Prototype gate: model simplification

Written 2026-11-17. Decision: **not-run**. Protocol proposed; execution manifest
not yet frozen. No model call, specimen test, measurement run or useful result is
claimed. This is a bounded research checklist, not production tasks or design approval.

Owner approval, 2026-11-17: "approved" after the design/protocol delivery at
09959f6. Preparation and bounded execution are authorized under this protocol.
The complete freeze and mandatory controls remain prerequisites; approval does
not establish results or authorize production implementation. The older approval
wording below describes the protocol before this owner decision; no repeat approval
is needed for work that remains within these approved bounds.

Spec: [design](design.md), [requirements](requirements.md), [R115](../../../docs/research/2026-11-17/02-model-simplification-design-gate.md),
[research index](../../../docs/research/README.md).

## Decision and falsifiable hypothesis

H-MS1: the existing model seam and step checker, with a small deterministic
measurement spike, can produce at least one unedited model candidate that removes
real decision or repeated state-write work over a complete responsibility while
preserving its observed behavior. Both named cases must be attempted. Preparation,
parser success, corrected metrics, extra pins and accepted replies are separate
outcomes and do not establish useful simplification.

Reject this hypothesis for the attempted boundary if observations cannot be checked,
the measures reward the controls below, or no candidate meets its predeclared target
within the budget. Keep negative results. A narrower proceed decision may cover only
the demonstrated axis; it cannot qualify failed state semantics from branch success.

## Known source identities

Repository: `D:\BuildArtifacts\lesscode\steps-foreign\x_likes_scraper`.
Revision: `3a214ff13ee768535481f71258ac2b3529306870`. Tracked tree was clean on inspection.
License declared in pyproject: MIT; preserve its license and notices in snapshots.
Lesscode inspected at `7260fa86f7c9d09fa771f71d1a5fdb0bd49ce82d`; freeze the eventual
experiment commit and any disposable prototype patch separately. These identities
describe inspected bytes, not a passing runtime baseline.

| File | Raw-byte SHA-256 |
| --- | --- |
| `x_likes_exporter/exporter.py` | `603A213173490CDB67312EA3636D3B110A438D35FD3B5B55494A3DA4C68BE917` |
| `x_likes_mcp/tools.py` | `627D7E7ED065A2360C7C1FA9B919CCD9F9BD0658A56F3E31191F85CBF6054093` |
| `pyproject.toml` | `412AEC33BF823BA51C2C53DCAE4D974DAE10306F66B0A075C2C6B1D6D3C79152` |
| `uv.lock` | `48FD5673AC2C4358CC14AE5EFD8DEEB4BF03D005C988373C8CBE484E8D65E1D6` |
| `tests/test_exporter_resume.py` | `330DEAC7B567E6454A4396DE56D06984B35C4868779C4F50E79E737111BDE9CC` |
| `tests/mcp/test_tools.py` | `CCCF1407281BEF2F12F3BD16F6E4EE0E09E360888DFB7DBBB0935C1D1BA021E8` |
| Lesscode `native/packs/python.json` | `090E24807F179A0D54A9C9DC5BBAECDF3CD7A9DE954BF5514F321BD962B8085A` |
| Lesscode `native/Cargo.lock` | `9045EA8F6E505AD3DBC3797727E89CD22A16E2FEBEA9384A4BD1B739D709606A` |
| Lesscode `model/Cargo.lock` | `0485B3401E0DA8FBF0005B6C57AFAEFE87514A1AA6EB7643CF9BE6EC954E293D` |

P0 expands this inventory to every source/test/support file actually used, including
license, imported error helpers, test fixtures and runner configuration. A hash listed
here never substitutes for retained, retrievable artifact bytes.

## Responsibilities, observations and plausible replacement

### S: retained tweet state

Responsibility: `TweetExporter`'s retained tweet lifecycle across construction,
resume, fetching, checkpoint callbacks and download. Measure the entire
`x_likes_exporter/exporter.py`, all replacement support and any changed caller.
The target relation is the lexical `self.tweets` field family. Report other field
and effect relations too. Public methods/signatures and all call sites remain fixed.
Only this file may change; new internal helpers are counted in full. New files,
dependency/config changes and changes to other components are rejected in this arm.

Inspection found four direct `self.tweets` write/mutation sites: initialization,
resume restoration, in-place extension and fresh-result assignment. This is a manual
census, not a qualified W baseline. Eliminating a redundant assignment/merge path
could reduce repeated writes without introducing a second state representation.
Pure preparation may need local intermediate values; those are reported as L.
Moving identical writes to a setter or merging functions is not a qualifying gain.

The existing in-place merge makes this a risky hypothesis. Preserve:

- Ordered tweet contents, duplicates, cursor/start cursor, return value and object
  identity where observers retain aliases to the resumed or previously held list.
- Same-user versus changed-user checkpoints, resume disabled, empty and nonempty
  batches, progress/stop callback order and values, and checkpoint save/clear calls.
- Values visible in callbacks and after failures at resume, fetch, checkpoint and
  downloader boundaries; exact successful and partial-state behavior of the baseline.
- Public output, exceptions and relevant stdout; downloader inputs and options.

Pin observations on unchanged source with fake client/checkpoint/downloader objects;
never use credentials, cookies or a live social-network service. Inventory all
consumers of the affected state. If a required alias/effect cannot be observed or
bounded by a supported contract, state qualification fails rather than assuming purity.

### B: structured-filter validation

Responsibility: `_validate_filter`, `_validate_year`, `_validate_month_field` and
`_validate_filter_dependencies` in `x_likes_mcp/tools.py`, including their shared
constants and any replacement support. `_validate_filter` callers, imported error
factory and the rest of the module are read-only boundary witnesses. Collect them
in P0. Only those four function bodies and new private helpers in the same file may
change; all other declarations, public signatures, dependencies and files stay fixed.

The complete responsibility includes the four original functions and every helper,
table or expression used to replace them, counted once. Also report the whole module
as an accounting envelope: no branch/state work may migrate outside the target set.
The target set is frozen before calls; it cannot be narrowed after seeing a candidate.

Inspection found seven explicit `if` sites across the three validators. Repeated
optional-value checks and dependency predicates might share a simpler formulation.
Any table, loop, lookup or helper needed for that formulation is charged. No saving
is presumed: the error-order contract may make these decisions irreducible.

Preserve acceptance/rejection, field/reason strings and first-error precedence for
None, bool, integers at/around bounds, nonintegers, month types/pattern boundaries,
and combinations of year/month dependencies. Include the regex's actual handling
of trailing newlines and custom/truthy inputs admitted at the public boundary;
do not silently tighten validation. Check the public `search_likes` path with its
resolver fake as well as direct validators. Preserve existing error-factory behavior.

## Fixed measures and usefulness thresholds

Use the definitions B/P/E/L/W/U/D/S and qualified identities in the provisional
design. Profile, selector semantics and effect contracts are fixed in the execution
manifest. Record raw facts beside totals; unknown is never zero. A bounded claim
requires explicit unchanged external-effect assumptions and controls supporting them.

| Case | Mandatory absolute and relative target | Companion controls |
| --- | --- | --- |
| S | For `self.tweets`, reduce W by at least 1 site and at least 25%; reduce S by at least 1 and at least 25% | D for that target must not rise; whole-file-plus-support W, S, D, B, P and E must not rise; no new relevant U, escaping alias or secondary state representation |
| B | Across the full validation responsibility, reduce B by at least 2 and at least 20%; reduce P by at least 1 and at least 10% | E, W, S and D must not rise; whole-module-plus-support totals must not rise; no new relevant U, opaque predicate logic or lost condition coverage |

Relative change always uses the positive measured baseline for that exact target.
A zero baseline makes the target unsatisfied, not automatically passed. Freeze actual
baseline facts and derived integer ceilings before model calls. The manual counts
above do not authorize changing a target if the measured counts differ. Report full
responsibility and envelope totals alongside targeted deltas; “25% less state” is
not an allowed description of a 25% reduction for one field.

Behavior must be accepted by the unchanged step flow. Existing token growth/frame
limits still apply, but source-size reduction is not a usefulness criterion. A
behavior-accepted unchanged, regressed or mixed answer ends that trial. Additional
trials are scheduled in advance, not created to hide that outcome.

## Comparison arms and attribution

1. **Instrument**: unchanged source under legacy measurement versus the admitted
   structural profile. Report local assignments reclassified and unresolved effects;
   keep the legacy mutation pack byte-identical. Compare eligible mutant sets on
   the same unchanged source. Any lost/gained mutant requires a separate pack arm
   and revision of this gate; a structural companion alone must not change the set.
2. **Protection**: unchanged source with existing tests, then admitted additional
   pins. Record known fault detection gained, with no source simplification credit.
3. **Candidate**: pinned baseline versus each unedited model candidate under the
   same tests, packs, profile, runtime and declaration. Retain first-answer and
   repaired outcomes separately. Human selection and test authoring are disclosed.

No feedback-causality, automatic-selection or model-ranking claim is made. Such a
claim would need its own matched control arm under requirements 8.4/8.5. No human
candidate edits or rewrite-specific hints qualify as unedited model success. Preserve
them separately if supplied. Unknown assistance never counts as unassisted.

## Adverse controls and expected outcomes

| Control | Required result |
| --- | --- |
| Same bytes twice; comments/format/local rename only | Deterministic facts; no structural gain; qualified identity unaffected by local spelling |
| Local rebinding versus global/nonlocal/attribute/subscript writes; alias mutation | Correct L/W distinction or explicit U; unresolved target cannot support state improvement |
| Fold nested branches into Boolean expressions | Retained P/short-circuit work visible; fails useful target when only B changes |
| Move work to helper/file/table; merge writer functions | All support counted; mere relocation or a smaller writer count cannot pass |
| Generic setter, dynamic attribute, escaped alias or unknown call | Effect unresolved unless admitted contract supports it; no false zero or useful state claim |
| Wrong return/validation reason or error precedence | Frozen behavioral checks reject the relevant candidate |
| Wrong callback state/order, lost in-place aliasing or changed partial failure state | Frozen behavioral checks reject; if undetected, repair pins on baseline in a new freeze before proposals |
| Edit test/declaration/profile; out-of-scope helper or path escape | Candidate rejected before execution; original bytes/refs unchanged |
| Stale hash, missing/partial record, interrupted checker, changed runtime | Incomplete/incompatible, never accepted/useful; no repair against infrastructure |
| Stubbed accepted unchanged answer, rejected then repaired, refusal, cutoff, terminal judge error | Correct stop/repair behavior and retained raw attempts; accepted unchanged never triggers optimization |

All mandatory controls must pass before candidate generation. A failing control is
instrument/protection evidence, not a model failure. Preserve its record when revising.

## P0: prepare and freeze, without model calls

This preparatory checklist is allowed before production tasks. It does not approve
experiments. Write an isolated Rust spike and PowerShell orchestration using current
core/seam functions. A disposable patch to the repair loop may test the proposed
fallible-check interface; do not modify production sources in the working repository.
No parser, transport, generalized recorder or replacement test runner is added.

Use `D:\BuildArtifacts\lesscode\model-simplification\ms1\` for snapshots, caches,
logs and binaries. Resolve source/work/output roots and reject roots or overlap with
the specimen. Run host tools from their repository directories through PowerShell.
Existing Rust 1.98.1, lockfiles and nextest apply; native verification uses existing
Dagger through the native Windows CLI and Podman. Never execute in WSL.

Target Python tests run in a Linux container, preferably a small Compose invocation,
with frozen source/test mounts, no model credentials and no external network during
tests. The source lock remains unchanged. Its dev dependencies do not include xdist:
prepare an explicitly pinned runner overlay, using uv for setup, and freeze its full
resolution. Record image digest, Python/uv/pytest/xdist versions and executable hashes.
Do not guess them or call setup complete from a container's existence.

Freeze `freeze.json` plus retained files and SHA-256 sidecars containing:

- Full source/test/support inventories, caller/observation boundary, source/ref
  before-images, license, dependency/runtime identities and normalized paths.
- Rust spike and any disposable patch; measurement profile/effect contracts;
  tool binaries, locks, test/pin bytes, fault controls and exact expected outcomes.
- Private Git declaration/pin revisions, fixed scope slots, step settings and
  exact setup/test/check commands. Tests use pytest-xdist with two workers; model
  trials are serial. Freeze collection and any predeclared exclusions with reasons.
- Request template, included context files, model/endpoint identity, sent options,
  temperature if supported, output limit, waiting policy and intervention log.
  Keep keys out of retained material; identify secret configuration without its bytes.
- Targets above, budgets below, observation normalization (only nondeterministic
  metadata, never domain content/order/identity), expected control decisions and
  retrieval locations for every artifact. Any item absent means not ready.

Inspect the resulting complete manifest before authorizing the bounded real-code
run, as required by the inherited solution-unlock policy. The current authoring
request is not recorded as approval of an as-yet-unwritten execution manifest.

## P1-P4: bounded execution after the freeze

- [x] **P1, qualify the instrument and baseline.** Run both originals twice under
  the frozen observations, require matching normalized outcomes and usable step
  baselines. Run all measurement, behavioral and lifecycle controls. Record old/new
  measurement and pin effects separately. Retain numeric baselines and derived
  ceilings in a hash-linked supplement before the first model call.
- [ ] **P2, attempt state.** Run three predeclared trials of S against the same pinned
  baseline, each with at most two repairs. Save requests, raw replies, candidate
  bytes/diffs, verdicts, comparable facts or explicit gaps and resource use.
- [ ] **P3, attempt branches.** Run three predeclared trials of B under the same
  shared evaluator/profile. No shared implementation or rules may change between
  P2 and P3. A required change stops this attempt; revise and requalify both cases.
- [ ] **P4, assess evidence.** Verify artifact bytes/digests, original refs/contents,
  all outcomes and complete-replacement accounting; obtain independent evidence
  review before a proceed decision. Owner-designated human review is possible;
  agent review requires explicit owner authorization under the foreground rule.

Do not proceed from broken setup to model calls. Do not omit the second case after
success or keep calling after a case's three scheduled trials. If a global stop
prevents a scheduled trial, retain it as not attempted with the reason.

## Effort, call and runtime bounds

- One initial apparatus version and at most one revised version, each preregistered
  before its own execution. Material changes require a dated hypothesis amendment.
  A revision does not erase earlier failures or refund consumed budgets.
- At most 1,400 added/modified canonical maintained lines of Rust/PowerShell
  apparatus, cumulative union at each file's largest size; count copied glue too.
  Existing production files modified in a disposable patch are charged in full,
  except at most 20 registration-only lines in an existing entry point. New specimen
  tests/fixtures are reported separately and capped at 600 added maintained lines.
  Existing repository file/function ceilings still apply; no budget is raised here.
- Six scheduled trials, at most 18 seam calls total across both apparatus versions.
  Each allows one proposal plus two repairs; output limit 8,192 tokens per call.
  Freeze the existing seam waiting/retry settings. Count provider requests, returned
  tokens, timeouts and charges when available; do not mistake an output ceiling for
  actual usage or an overall input-token/cost cap. No parallel endpoint trials.
- Setup/preparation execution: at most 120 minutes; qualification: at most 90 minutes;
  candidate experiment: at most four hours, with 20 minutes per checker invocation.
  At most 80 checker invocations in total, including original/control/pin/candidate
  invocations; record spent checks before starting more. These are operational limits,
  not performance benchmarks. Timeout is incomplete evidence, never a killed fault.

Owner amendment, 2026-11-17: explicitly selected "Add 60 minutes (recommended)"
during P0. Preparation increases from 60 to 120 minutes total. The recorded
12:57 Europe/Berlin start therefore gives a 14:57 deadline, including session gaps.
No consumed effort is refunded. Apparatus/pin caps, version limits, qualification
time, checker limits and all model-call limits remain unchanged.
- Stop immediately on original-tree mutation, frozen-input mismatch, escaping scope,
  unavailable required observation, budget exhaustion or an unqualified mandatory
  control. Stop the affected trial on refusal, terminal call/judge failure or accepted
  behavior. No optimization continuation after accepted behavior.

## Evidence and gate decision

Retain source, candidates, diffs, raw calls, all check logs, profile facts, normalizers,
control outcomes, intervention records, tool identities, resource counts and manifests.
Commit compact research summaries and artifact sidecars; bulky files stay under the
artifact root with verified retrieval paths. A missing local artifact is unavailable
evidence even when its old sidecar remains in Git.

Report scheduled/attempted/endpoint-failed/candidate-producing/behavior-accepted/
measurable/useful/incomplete/adopted counts, explicit denominators and two unique
responsibilities. A trial is attempted once its first call begins. Retain overlapping
failure categories and distinguish first-answer from repair recovery. Adoption is
not established by these isolated tests; do not write back to the specimen.

| Gate | Required evidence; all currently not-run |
| --- | --- |
| U1 boundary | Complete frozen observations/inventories, hashes and reviewed execution plan |
| U2 real behavior | Both cases attempted; unchanged observations reproducible; each claimed candidate judged on the frozen tests/pins and relevant cross-function state |
| U3 adverse paths | Every applicable control above has its expected outcome; no missing evidence becomes pass |
| U4 common mechanism | One frozen evaluator/profile across both cases; per-case data distinguished from human logic changes |
| U5 useful replacement | At least one unedited model candidate meets its case's absolute/relative structural targets and all companion/behavior controls; failed axes excluded |
| U6 compatibility | Original step codes/guard/ledger unchanged, legacy mutation population preserved, source/refs unchanged, supported and unsupported facts explicit |
| U7 bounded and reviewed | Within cumulative budgets, durable verified evidence and independent evidence review; normal design approval still pending |

These are this feature's explicit applications of the inherited U1-U7 evidence
principles. E3/E4 recording APIs, replay-specific observations and old line-reduction
targets belong to the historical native-core experiment. The approved branch/state
objective replaces that historical U5 metric here; its old results are not inherited.
The 1,400-line/two-version effort bound and independent evidence review are retained.

- **proceed**: all applicable gates pass, both cases have attempts and at least one
  complete responsibility is useful; revise design to that demonstrated subset and
  seek normal design approval before production tasks.
- **revise**: a concrete bounded correction remains within cumulative budgets;
  preserve the failed freeze and preregister the changed hypothesis before rerunning.
- **stop**: budgets exhausted, required observations cannot be preserved, or no useful
  result remains plausible without expanding the mechanism or lowering targets.
- **not-run**: no qualifying execution evidence. This is the current decision.

## Missing evidence at protocol authoring

No spike, runtime overlay/container digest, executable observation pins, complete
freeze manifest, admitted measurement companion, controls, candidate or behavioral
result exists for H-MS1 yet. P0 makes those prerequisites concrete. No production
task list or empirical success follows from this protocol document.

## P0 completion amendment, 2026-11-17

[R119](../../../docs/research/2026-11-17/06-model-simplification-p0-freeze.md)
and the [freeze receipt](../../../docs/experiments/model-simplification/freeze-receipt.json)
supersede the preparation status above. P0 is complete: the apparatus, observation
pins, runtime, private declarations, exact requests/commands and evidence inventory
are frozen and verified. The actual specimen class is `XLikesExporter`; the
earlier `TweetExporter` wording refers to that same retained-state responsibility.

P1-P4 remain unchecked. Raw facts are conditional, and native step baseline
qualification and the complete adverse-control admission are still required.
No candidate call or useful-result claim is made. H-MS1 remains **not-run** and
production eligibility remains false. The preparation extension changes time
alone; all other approved limits and stop conditions remain in force.

## P1 failure amendment, 2026-11-17

[R120](../../../docs/research/2026-11-17/07-model-simplification-p1-declaration-refusal.md)
records the first frozen native baseline refusal: the shared declaration commit
changes the other case's declaration outside scope. No target tests or model calls
ran. Remaining qualification checks stopped; P1-P4 stay unchecked. Disposition is
**revise**, not proceed; H-MS1 efficacy remains untested. Freeze-1 remains immutable.
The proposed correction uses separate case declaration histories, with new refs
and a reviewed freeze before requalification; it is not executed or qualified.
Starts are 32/80, calls 0/18. Preparation still ends 14:57 Berlin; the fixed P1
deadline is 16:16:54.6075613 Berlin. Session gaps and failed checks are not refunded.

## P1 final disposition amendment, 2026-11-17

[R121](../../../docs/research/2026-11-17/08-model-simplification-p1-timeout.md)
and its [receipt](../../../docs/experiments/model-simplification/p1-result-receipt.json)
record the permitted second version and its incomplete native baseline.
Separate declaration histories removed the scope refusal; an additional manual
checkout was required after cloning the bundles. The native S1 comparison then
reached its 1,200-second bound before the step-side campaign finished. Base-side
53/64 fault detection and passing closing baselines do not complete that comparison.

Disposition: **stop** for this bounded attempt. Two versions used, 36/80 outer
starts charged, 0/18 calls, 0/6 trials attempted. P1-P4 remain unchecked and model
efficacy untested. Conditional numeric facts are not admitted. No deadline reset,
third revision, production approval or independent-review claim follows. Preserve
both freezes, failures and partial evidence; further execution requires a new
explicit protocol/budget decision.

## Owner-approved extended P1 amendment, 2026-11-17

After R121, the owner selected option 1: unchanged frozen code/checks, a
2,400-second whole-check bound and a fresh 180-minute qualification window.
[R122](../../../docs/research/2026-11-17/09-model-simplification-extended-qualification.md)
preregisters the fresh directories, exact clock/counter records and retained
gates. Earlier stops remain historical evidence. Starts begin at 36/80, calls
0/18, trials 0/6; no other cap, protection or phase approval changes. This is an
explicit operational-budget amendment, not a third measurement implementation.

## Extended P1 classification failure, 2026-11-17

[R123](../../../docs/research/2026-11-17/10-model-simplification-infrastructure-classification.md)
records a completed-but-void extended check and an actual mutant with exit 125
classified as killed. The mandatory infrastructure control fails; the retry was
deliberately stopped. Disposition remains **stop**, with 38/80 outer starts and
zero model calls. No complete baseline is admitted and P1-P4 remain unchecked.

The owner-approved window still ends 18:25:47.6266045 Berlin, but remaining time
does not waive this correctness gate. The proposed runner/error-pattern correction
requires a new explicit harness-revision decision because option 1 kept checks
unchanged. No frozen code was patched, no third apparatus version executed and no
production or independent-review approval is implied.

## 2026-11-17 amendment: approved classification correction verified

[R124](../../../docs/research/2026-11-17/11-model-simplification-classification-correction.md)
records owner approval and five passing controls for frozen revision 3. Correction
controls completed within the original 15-minute bound. P1 retains its 18:25:47.6266045
Berlin deadline; 43/80 starts are consumed. Apparatus including control support is
1,358/1,400 lines; pins plus synthetic fixtures are 357/600. Full fresh qualification
remains pending. Earlier failed evidence and phase approval states remain intact.

## 2026-11-17 amendment: P1 qualified

[R125](../../../docs/research/2026-11-17/12-model-simplification-p1-qualified.md)
and its hash-linked admission receipt qualify the corrected evaluator: four
accepted checks, matching per-mutation outcomes and 32 passing closing baselines.
47/80 starts are charged; model calls remain zero. The numeric ceilings and
bounded effect contracts are admitted before proposals. P2–P4 and production
approval remain pending. Earlier failed qualification evidence is preserved.

## 2026-11-17 amendment: boundary newline control invalidates full admission

[R126](../../../docs/research/2026-11-17/13-model-simplification-boundary-newlines.md)
retains the four successful native checks but supersedes full P1 admission:
raw CRLF/LF header comparison falsely rejects the original after newline-only
conversion. S1 ended incomplete after one call; five trials remain unattempted.
The one-line patch is prepared, not applied. A fourth version exceeds the current
three-version authorization. All original deadlines/counters remain in force.

## 2026-11-17 amendment: owner-authorized fourth boundary correction

[R127](../../../docs/research/2026-11-17/14-model-simplification-boundary-correction.md)
records the owner's next-group authorization, including parallel agents for
independent controls/review. Revision four changes only raw-header CRLF comparison.
Nine boundary controls and 21 spike tests pass; the frozen prior evidence is
verified. No clock/counter reset, S1 retry, extra trial or production approval is
authorized. Admission requires unchanged original facts and the linked supplement.
Endpoint trials remain serial. Candidate checker limit remains 1,200 seconds.
The hash-linked v4 admission supplement restores P1 after exact fresh baseline
fact matches; earlier invalidated admissions and S1's incomplete outcome remain.
