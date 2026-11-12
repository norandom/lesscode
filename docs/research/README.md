# Research log

> **Milestone Synthesis Report:** For the comprehensive cross-subsystem synthesis synthesizing Experiments E1 through E6, breaking the Density Trap, and certifying the dual-specimen maturity gates under COBESY principles, see [`docs/reports/state-boundary-deconvolution-report.md`](../reports/state-boundary-deconvolution-report.md).

## Planned: R96 Why the second model writes fewer admissible packs (preregistered 2026-11-12)

[R96](2026-11-12/03-model-diagnosis-preregistration.md) reopens the Slice B question for cost: the loops must work on a model cheaper than Opus.
- **Arms:** Mistral Large 4 at its default reasoning level with 16,000 and 64,000 output tokens, at "high" with 64,000, and GLM 5.3 at "high"; 20 trials each with the Rust judge. Opus from R69.
- **Asked:** is the gap the reasoning level (never set in R72), the output limit, or knowledge of the grammar.

## Latest: R95 A second Python step with a frame specification; a wrong prediction; the test suite fills the temp folder (2026-11-12)

[R95](2026-11-12/02-second-python-step-frame-release.md) continues the open work from handover 43.
- **The step:** in x_likes_scraper, `Checkpoint.clear` uses `unlink(missing_ok=True)`. Accepted with a frame specification declared: 4 noticed / 3 unnoticed before, 4 / 0 after.
- **Wrong prediction:** a refusal was expected before the pins; the change was accepted. The Python pack plants no fault in a keyword argument.
- **Shown on throwaway changes:** a frame refusal (5 s; reports five sites for one new write, because sites are line numbers) and a release entry. An out-of-reach mark was not needed and is still not shown.
- **In this repository:** hook message fixed, guard job added to the pipeline (not yet run on the host), adopter check retired.
- **Found:** the test suite had left 26,285 directories in the temp folder since 2026-11-09. The regenerator tests are fixed; 21 other test files are not.

## Prior: R94 The first step in a Python project; an unpinned change that alters output is accepted (2026-11-12)

[R94](2026-11-12/01-first-python-step.md) is open work from handover 43, taken up before the onboarding spec.
- **The step:** in x_likes_scraper, `_escape_html` hands its work to `html.escape`. Accepted, recorded, passed by the guard: 0 noticed / 15 unnoticed before, 3 / 0 after.
- **The finding:** without a pin, the naive change was accepted although it writes `&#x27;` where the old code wrote `&#39;`. The plan had marked the function "no fault noticed". With two pinning tests the same change was refused.
- **Also shown:** a push through the installed hook (passed; refused in 0.6 s for a commit after the verdict) and an override, on a throwaway commit.
- **Still not shown on a real step:** release entry, frame specification, out-of-reach mark.

## Prior: R93 A real step is refused, pinned and accepted; a reused working copy kept its old tests (2026-11-11)

[R93](2026-11-11/19-refused-pinned-accepted.md) closes what R92 left open, as preparation for release 1.2.0.
- **The step:** in OpenUltraCode, `isPersistedWorkflowState` rewritten to one rule per statement. Refused: unnoticed faults in the function rise from 1 to 7. Nine tests admitted by `step pin`. Accepted: 3 noticed / 1 unnoticed before, 11 / 1 after.
- **The defect:** between refusal and acceptance stood two wrong refusals. A working copy was reused as it was, so the mutants ran again against the tests without the pins. Releases 1.1.0 and 1.1.1 have the same fault in `deconvolute run`.
- **The fix:** `aa9d7d7`; a copy follows its source before it is used again. A test that fails without the fix is in the suite.
- **Still not shown on a real step:** override, release entry, frame specification, out-of-reach mark, push through the installed hook.

## Prior: R92 Three real steps through the step tooling, in Rust, Java and TypeScript: all accepted (2026-11-11)

[R92](2026-11-11/18-three-real-steps.md) is tasks 6.1 and 6.2 of `gradual-refactoring-steps`.
- **This repository, Rust:** the `verdict()` change refused in R89 is accepted; 11 noticed / 0 unnoticed faults before, 12 / 0 after; 7,307 to 7,306 tokens. Merged into main as `63d456f` by owner decision the same day.
- **commons-csv, Java:** `printComment` starts comment lines through one method; 19 / 2 to 17 / 2 plus 2 / 0 in the new method.
- **OpenUltraCode, TypeScript:** `slugify` drops a condition that is always true; 9 / 1 to 8 / 0.
- **Not exercised:** no step was refused and none needed a pin; those paths rest on the fixture tests and R91.
- **Next:** a real step that needs a pin and one that is refused, in a foreign project (owner decision).

## Prior: R91 The three assumptions of the step design hold; two details of the design change (2026-11-11)

[R91](2026-11-11/17-gradual-steps-three-assumptions.md) is the gate of task group 1 of `gradual-refactoring-steps`: proceed.
- **Function filter:** the 11 faults in `verdict()` have the same classes as in the whole-file run, in 2.1 minutes instead of about 17.
- **Copies from a commit:** same test results as the working tree on this repository, a Java and a TypeScript project, if the whole repository is copied; a copy of `native/` alone fails 5 of 363 tests.
- **Pin against one fault:** admitted, rejected as useless and rejected as failing, all three through `deconvolute run --only`.
- **Design changes:** copy the whole repository; hash committed content, not files on disk; no `git archive`.

## Prior: R90 Complexity budgets count tokens; the change refused in R89 would now be within budget (2026-11-11)

[R90](2026-11-11/16-budgets-count-tokens.md) changes the `budgets` check by owner decision.
- **Metric:** tokens outside comments and test code per file and per function; defaults 2,800 and 560 (seven tokens a line, measured 6.96). Line ceilings in older budgets files still apply.
- **Migration:** every ceiling of this repository carried over with the headroom it had; one file and two functions frozen because they are over the new defaults. Nothing became looser.
- **R89's change:** 7,306 of 7,307 tokens, within budget. Whether it goes in is the owner's decision.

## Prior: R89 The first tracked step: the pins are accepted, the change is refused by a guard this repository already had (2026-11-11)

[R89](2026-11-11/15-first-tracked-step.md) takes the first real step by hand on a clone, for `gradual-refactoring-steps` (requirements generated, not approved).
- **Pins, accepted:** one test fixes the order of the reasons `verdict()` returns; planted faults noticed in the function 10 of 11 to 11 of 11. Branch `step/verdict-one-exit`, pushed for review, not merged.
- **Change, refused:** identical results on 480 inputs, format and lints clean, but the file grows from 969 to 972 lines against a frozen ceiling of 969; and it removes one token. Kept as a patch.
- **Point:** the refusal came from a hard boundary (the complexity budget), not from a skill or a reviewer.
- **Releases:** 1.1.0 and 1.1.1 are published and verified (signatures, checksums, attestation, install script, Zenodo).

## Prior: R88 Prototype gate for gradual refactoring: a step can be judged from two fault runs, and pinning first makes it pass (2026-11-11)

[R88](2026-11-11/14-gradual-steps-prototype-gate.md) is the prototype gate of the brief `.kiro/specs/gradual-refactoring-steps/brief.md`, after the owner asked whether code is healthy and working after a change and whether refactoring can be supported step by step.
- **Method:** `deconvolute run` before and after a step, compared function by function; refused when unnoticed faults rise.
- **Results:** a step that removes two tests is refused (noticed faults 10 to 0 in the function) although everything passes; a simpler `verdict()` is refused because a second order of checks is unpinned; one added test is accepted; two foreign steps are accepted from earlier receipts.
- **Pin first:** two assertions that pass on the code and fail on the two planted faults turn the refusal into an acceptance.
- **For requirements:** totals when functions are renamed, run only what a step touches, stored outcomes must depend on the tests, isolation for functions nothing tests, evidence keyed by spec criteria for openidea-kiro.

## Prior: R87 Measured by syntax, the four specimen candidates are 6% to 19% smaller, not 24% to 44% (2026-11-11)

[R87](2026-11-11/13-density-by-syntax-specimens-restated.md) re-measures the specimens of R50 to R53 with `deconvolute density` after it was changed to count syntax (design Amendment 6, thresholds fixed beforehand).
- **Tokens fewer:** Java 5.6%, Rust 7.0%, Python 7.5% and 9.8% (two sets), TypeScript 18.8%. Reported before, in lines: 38.5%, 40.3%, 23.8%, 36.8%. None meets the threshold for a substantial reduction.
- **Why:** the candidates put more on each line; the earlier command counted lines and, as "statements", keywords and semicolons per line.
- **Unchanged:** the candidates pass the tests there are (R50, R51, R86).
- **Open for the owner:** whether regeneration can produce reductions that are substantial by syntax is open again.

## Prior: R86 The Rust specimen candidate, built for the first time: it works, and most of its 40% is formatting (2026-11-11)

[R86](2026-11-11/12-rust-candidate-built-and-measured.md) closes the open point of R75.
- **Works:** in place of the six original functions the candidate compiles, lints clean and passes all 375 tests; under the driver 18 of its 36 mutants are killed (original 20 of 38). Three of the six functions are not reached by the tests that ran.
- **Reduction restated:** 248 to 148 lines (40.3%, "pass") as committed; 248 to 219 (11.7%) and 83 to 76 statements (8.4%), "marginal", with both files formatted the same way.
- **Tool weakness:** `deconvolute density` counts lines and depends on layout; the other specimens' figures were measured the same way and not re-measured.

## Prior: R85 Three annotation runs select more stably than one, but not stably enough to call the selection fixed (2026-11-11)

[R85](2026-11-11/11-slice-b-annotation-hardening.md) runs the preregistration [R84](2026-11-11/10-slice-b-annotation-hardening-preregistration.md): six fresh Opus runs per scope, two rules over three runs.
- **Majority of three:** agrees more than single runs in four of five files (0.74 to 0.96 against 0.62 to 0.86) and keeps as few sites as one run; the target of 0.80 in four files is missed (three).
- **Union of three:** more stable in four of five files, keeps 10% to 60% more sites.
- **Recommendation:** if annotations are used, the majority of three, described as what two of three runs agreed on. The default of every eligible site remains the only selection that does not vary.
- **Cost:** 3.25 USD; recorded spend about 32.9 of 40 USD.

## Prior: R83 Annotation layer: Opus runs agree on 0.69 to 0.92 of their sites; three of five Mistral runs give no annotations (2026-11-11)

[R83](2026-11-11/09-slice-b-annotation-baseline.md) runs the preregistration [R82](2026-11-11/07-slice-b-annotation-baseline-preregistration.md): five annotation runs per scope on two models, checked by the driver's validator.
- **Opus:** every run usable; median agreement between runs 0.69 to 0.92 per scope; sites named in every run are 44% to 85% of those named in any. Two runs fall below 80% accepted (claim A1 refuted).
- **Mistral Large 4:** 15 of 25 calls end at the 32,000-token output limit without annotations; where a reply comes it is as valid as Opus's. Agreement with Opus runs 0.28 to 0.72, lower than among Opus runs in every scope.
- **Reading:** rejected annotations never become mutants, so this drift changes what is tried, not whether a run is sound. The default (every eligible site) does not drift.
- **Deviation:** the caller could not read the second model's reply format; seventeen runs were finished from their saved responses without a new call.
- **Cost:** 4.20 USD; recorded spend about 29.7 of 40 USD.

## Prior: R81 Three changes after the validation: a stop handler, three false successes removed, call-argument operators measured (2026-11-11)

[R81](2026-11-11/08-driver-three-changes-and-call-arguments.md) records the owner's three choices after R80.
- **Stop handler:** Requirement 5.7 is met for Ctrl-C, Ctrl-Break, termination and a closing console; tested with a real signal on five platforms.
- **Removed:** three functions that reported success without running anything (R75 had named two).
- **Call arguments:** a sixth, optional site kind. On the five gate scopes 125 new mutants beside 557; 88 are killed or survive. 556 of 557 earlier mutants keep their class (one time limit under load). Swapping the first two arguments rarely builds in Java and Rust (2 of 17).

## Prior: R80 Final validation of the agnostic driver: implemented as specified, one criterion in part (2026-11-11)

[R80](2026-11-11/06-agnostic-driver-final-validation.md) is task 8.3 of `agnostic-annotation-driver`; the report is in the spec directory.
- **Result:** 382 tests, lints, budgets and smoke runs pass; CI is green on five platforms for the commit with every code change. Every requirement has evidence; 5.7 is met in part (a driver killed from outside can leave a project command running; no signal handler).
- **Not independent:** done inline by the implementer under the owner's foreground rule.
- **Open for the owner:** a release, a signal handler, two library functions that report success without running anything (R75), a CI job for the guide optimiser.

## Prior: R79 A MIPROv2-style search does not fix Rust admission on Opus; the GEPA run did (2026-11-11)

[R79](2026-11-11/05-slice-b-miprov2-opus.md) runs the preregistration [R77](2026-11-11/03-slice-b-miprov2-preregistration.md) with a second engine in the guide optimiser (TPE sampler copied from `dsrust`; MIPROv2's proposer wording and trial loop, without demonstrations and minibatches, which this task has no use for).
- **Result:** same model, starting guide, rules and 60 authoring calls as the GEPA run. The chosen guide scores 0.50 on the scoring set (claim: at least 0.75, refuted); in the final judgement Rust is admitted 0 of 5 times, against 5 of 5 for GEPA's guide (R71). Nothing is adopted.
- **Reading:** GEPA writes each new guide from the judged replies; MIPROv2 proposes from a task description and tips before any failure is seen. Here the failures carried the information. One run per engine; no general ranking.
- **Cost:** 5.82 USD; recorded spend about 25.5 of 40 USD (stop raised by the owner on 2026-11-11).
- **Slice B state:** drift measured between runs, between models and under two optimisers; the annotation layer and drift over time are not measured.

## Prior: R78 The node kind list removes invalid node kinds but does not raise admission on Mistral Large 4 (2026-11-11)

[R78](2026-11-11/04-slice-b-node-kinds-result.md) runs [R76](2026-11-11/02-slice-b-node-kinds-preregistration.md): the grammar's node kinds as extra input for the pack author, with a same-day control arm on each model.
- **Mistral Large 4:** first replies naming a node kind the grammar does not have fall from 10 to 1 of 20; first replies admitted stay at 5 and 6 of 20, because more replies end at the output limit and the rest fail control cases. Two of the four claims are refuted.
- **Opus:** 15 of 20 in both arms; the list changes nothing.
- **Not adopted** as part of the task's input. With precise repair feedback Mistral ends at 15 of 20 admitted without the list.
- **Cost:** 4.46 USD for the four arms.

## Prior: R75 The specimen gates' adverse controls, looked up and rerun through the driver (2026-11-11)

[R75](2026-11-11/01-specimen-controls-rerun-through-driver.md) is tasks 7.1 and 7.2 of `agnostic-annotation-driver`; dated amendments are appended to R49 to R53.
- **What the old figures were:** R52 (Rust): the four kills and the 100-case parity were written into the gate script; the candidate was never built. R50 (TypeScript): three probe comparisons and one kill written into the script. R51 (Java): four probe comparisons, one of them a mutant that did not compile. No mutant was run against a project's own tests.
- **Rerun:** five of the twelve controls have a driver mutant; with the projects' tests four are killed and one survives (a dropped `lock.unlock()`). Seven are recorded as not reproducible.
- **Measured instead:** killed of killed plus survived, whole file: TypeScript candidate 5 of 5 and original 15 of 16; Java candidate 78 of 89 and original 78 of 90; Rust original 75 of 133 (the six specimen functions 20 of 38).
- **A void run:** copies that share a build directory through the environment (a global `CARGO_TARGET_DIR`) gave 16 of 133 wrong classes; the closing run caught it. Documented, and the void reason now quotes the failing output.
- **Deleted on the same day:** the six text-matching adapter files (869 lines) and their tests.

## Prior: R74 Guide optimisation on Mistral Large 4: no improvement found (2026-11-10)

[R74](2026-11-10/20-slice-b-guide-optimisation-mistral.md) runs R70 on the second model with the parameters of the Opus run.
- **Result:** after 62 metric calls the starting guide and the one kept candidate both score zero on the scoring set; nothing is adopted and no final judgement was run. The candidate got 4 of 17 Java packs admitted on minibatches, Rust never.
- **Barrier:** ten of eleven proposals discarded, eight for length and two for changing the statement of purpose.
- **Reading, with R71 and R72:** optimising the instruction removed instability for a model that knows the grammars and did nothing for one that does not. Giving the author the grammar's node kinds is the idea to decide next.
- **Cost:** 1.99 USD; recorded spend about 15.2 of 20 USD.

## Prior: R73 Native driver runs the five gate scopes end to end; 554 of 556 mutants classed as by the prototype (2026-11-10)

[R73](2026-11-10/19-native-driver-batches-on-gate-scopes.md) is task 8.1 of `agnostic-annotation-driver`.
- **Result:** `deconvolute run` on the gate's Python, TypeScript (two files), Rust and Java scopes with the projects' own tests: every mutant both sides ran has the same class (556 of 556). A first Java run with a fixed time limit had two timeouts the prototype did not; with the limit scaled from the baseline they agree. Mutants only the prototype has are the 38 dropped repeats.
- **Safety:** no scope file changed; no container left behind.
- **Speed:** Python 118 s and TypeScript about 100 s with six copies; Java 814 s with four containers; Rust about 23 minutes, serial.
- **Open:** final validation and the removal of the old adapters (task groups 7 and 8.3) need the owner.

## Prior: R72 Slice B baseline on Mistral Large 4: 5 of 20 first replies admitted against 15 of 20 on Opus (2026-11-10)

[R72](2026-11-10/18-slice-b-baseline-mistral.md) is the second model arm of R66, with the same guide and checks.
- **First replies admitted:** Rust 0, Python 1, TypeScript 4, Java 0 of 5. The commonest failure is a node kind that does not exist in the grammar; admission names it.
- **Repair:** Java 5 of 5 admitted after one repair turn and identical to the shipped pack's mutants; Rust never admitted; six runs used the whole output limit for reasoning and wrote no pack.
- **Consequence:** changing the model is the largest drift measured so far. The barrier held: nothing inadmissible got through.
- **Deviations:** output limit 16,000 tokens instead of 4,000, because the model counts its reasoning against it; a first attempt crashed on the multi-part reply.
- **Cost:** 0.73 USD.

## Prior: R71 Guide optimisation on Opus: admission fixed for all four languages, Rust still differs from the hand pack, not adopted (2026-11-10)

[R71](2026-11-10/17-slice-b-guide-optimisation-opus.md) runs R70 on `claude-opus-5-5` with the copied GEPA engine.
- **Final judgement, five fresh trials per language:** first replies admitted 5 of 5 for Rust, Java, Python and TypeScript (Rust was 0 of 5 under guide version 3). Java, Python and TypeScript give the shipped pack's mutants in every trial.
- **Not adopted:** the Rust packs are admitted and consistent with each other, but differ from the shipped pack in the same four points every time (the empty statement, how `?` is rewritten, aborting macros, `match` guards), so the preregistered rule fails for Rust.
- **Barrier:** eight proposals were discarded, all for length.
- **Cost:** 6.49 USD; the estimate printed before the run was too low, and the budget stop interrupted the final judgement, which was completed in a second invocation. The tool now holds that budget back.
- **For the owner:** decide the four Rust points on their merits.

## Prior: R70 Slice B guide optimisation preregistered, not run; optimiser is the `dsrust` MIPROv2 (2026-11-10)

[R70](2026-11-10/16-slice-b-guide-optimisation-preregistration.md) fixes metric, split, verification and acceptance for optimising the pack guide.
- **Optimiser:** by the owner's direction a Rust port of DSPy (`dsrust`, alpha) supplies `MIPROv2`, in a separate research package; no Python. Its parameters and soundness on this task are not yet checked.
- **Acceptance:** on fresh runs, at least 4 of 5 for Rust and Java on admission and on equal mutants, and no regression for Python and TypeScript.
- **Budget:** user-set parameters with defaults; the 20 USD project stop remains.
- **Second endpoint:** Mistral (`mistral-large-4`) is configured locally for the open model arm of R66.

## Prior: R69 Slice B rerun: stronger admission stops every first Rust pack; repair works twice in five; Java differs too (2026-11-10)

[R69](2026-11-10/15-slice-b-baseline-rerun-stronger-admission.md) reruns the baseline of R68 with the hand-written Rust controls and the extended Python pack.
- **Rust:** 0 of 5 first replies admitted; 2 admitted after two repairs, 2 still rejected, 1 repair request refused by the endpoint. The repair feedback did not name the expected operator; that defect is fixed.
- **Java:** admitted 5 of 5, but 3 of 5 packs lack the operator for `throw` and so differ from the shipped pack. An example pack in the request had changed, so this is not sampling alone.
- **Python and TypeScript:** 5 of 5 admitted and identical to the shipped pack's mutants.
- **Consequence:** checks alone do not harden authoring. Next: a preregistered, bounded `MIPROv2` optimisation of the pack guide, as the owner directed.
- **Cost:** 1.62 USD at the list price; 5.89 of 20 USD recorded.

## Prior: R68 Slice B baseline: packs always admitted, Rust packs differ between runs and admission misses it (2026-11-10)

[R68](2026-11-10/14-slice-b-pack-authoring-drift-baseline.md) runs the protocol of R66 on the current endpoint (20 conversations, `claude-opus-5-5`).
- **Admission:** 20 of 20 first replies admitted; no refusal, no repair needed, so the repair path is still untested.
- **Same behaviour:** Java and TypeScript packs give exactly the shipped pack's mutants in all five runs. Python packs agree with each other in all five runs but give one mutant more than the shipped pack, because the model adds two operator pairs the hand pack lacks. Rust packs fall into four behaviours in five runs, none equal to the shipped pack.
- **Consequence, as preregistered:** the admission checks are too weak; strengthen the controls before any `MIPROv2` work, decide the Python pairs on their merits, then rerun.
- **Cost:** about 1.04 USD at the list price; 3.91 USD under the conservative assumption used for the cap.

## Prior: R67 Native engine reproduces the prototype; 38 of 594 mutants dropped on purpose as repeats (2026-11-10)

[R67](2026-11-10/13-native-driver-engine-parity.md) closes task group 3 of `agnostic-annotation-driver`.
- **Parity:** the engine in `lesscode-core` gives exactly the prototype's sites and, under the gate packs, exactly its mutants (by hash) on the five gate scopes; 98 of 98 controls give the recorded outcome. The scope files are now test fixtures and the test needs no Python.
- **Intended difference:** with the shipped packs a swap of a condition's own equality operator is dropped when the site is also negated: 38 of 594 mutants (2, 3, 3, 11 and 19 per scope). Gate kill rates included these repeats.
- **Consequence for Slice B:** the engine-dependent start condition of R66 is met; the pack guide must mention the two new pack keys before the baseline runs.

## Prior: R66 Slice B preregistered: drift baseline for pack authoring, not run (2026-11-10)

[R66](2026-11-10/12-slice-b-pack-authoring-drift-preregistration.md) fixes the protocol of the first Slice B
step before any call is made.
- **Protocol:** 5 authoring runs per language for four languages on the current endpoint, then on Mistral; frozen guide; admission by `deconvolute admit-pack`; at most two repairs.
- **Claims:** at least 4 of 5 first replies admitted, at least 4 of 5 admitted packs reproduce the shipped pack's mutants, every rejected pack admitted after at most two repairs. Each failure has a named consequence.
- **Start condition:** driver task group 3 (the mutant comparison needs the native engine), the amended guide hashed, and the owner's go.
- **Cost:** about 3.60 USD for the first replies at the assumed price; the worst case with repairs does not fit the remaining cap, so the run stops at the cap.

## Prior: R65 Rust sizing spike for the driver: exact parity, measured size (2026-11-10)

[R65](2026-11-10/11-annotation-driver-rust-sizing-spike.md) is the first step of the
design phase for `agnostic-annotation-driver`.
- **Parity:** a Rust port of the model-free core on the crate's pinned parser (0.51.0) gives exactly the prototype's sites and mutants on all five scopes, the same outcome on 98 controls, and the same pack admission and classification results. No grammar difference; no pack changed.
- **Size:** core 898 canonical lines (1,035 physical), pack admission 413 as ported; the adapters it makes removable are 753 canonical lines. Two grammar crates are added (MIT); the binary grows by about 1.5 MB.
- **Corrections:** the earlier estimate (750 to 1,000) omitted pack admission; and the earlier statement that the port must fit under the 7,173-line cap was unchecked: the cap's counted set (R41) contains neither the adapters nor the deconvolution command code.
- **Open:** how the driver is accounted against the cap (owner), and the batch runner, which was not ported or sized.
- **Queue:** Slice B (drift hardening) stays second and starts when the driver's design is approved.

## Prior: R64 Annotation-driver gate, attempt 2: mechanism rows met for the rules-as-data form (2026-11-10)

[R64](2026-11-10/10-annotation-driver-gate-attempt2.md) reports the second and last
gate attempt.
- **Outcome:** arm D (generic operators driven by a data pack per language, no model at run time) meets yield, oracle, meaningfulness and agnosticism on Rust, Python, TypeScript (two scopes) and Java. Meaningful share of landed mutants: 98%, 98%, 98%, 90% and 89%, with independent survivor labels.
- **Model-written packs:** the TypeScript and Java packs were admitted on the first reply and yield exactly the mutants of the hand-written packs.
- **Arm A (model annotates):** validity 81% to 100% on every scope and run; all rows met on Rust, Python and Java; one of four TypeScript runs landed two kinds instead of three. Every arm A mutant is also an arm D mutant, so annotation only selects.
- **Open row:** the apparatus ceiling. The counted native apparatus is at its cap of 7,173 lines and no Rust port was sized; this is an owner decision before production.
- **Limits:** 24% to 35% of TypeScript mutants do not compile; kind names are loose; five scopes, one model; this is about mutation, not compression.
- **Spend, both attempts:** about 233k input and 115k output tokens, at most 12.10 USD at an assumed price, cap 20 USD.
- **Next:** requirements and a design for the owner's approval. Owner decision the same evening: Rust port, single runtime, no breakage of codebases; see the [feasibility report](../reports/agnostic-driver-feasibility.md).

## Prior: R63 Annotation-driver attempt 2 setup (2026-11-10)

[R63](2026-11-10/09-annotation-driver-attempt2-setup.md) freezes the inputs of the
second and last gate attempt. Setup only, no counted run.
- **Pack format version 1** implemented as data: call statements as write sites, returned conditions, type-preserving token swaps, a literal swap for `return`-based failures, anchors completed by the statement terminator. Mutants per scope rise from 16, 161, 28 and 115 to 66, 199, 74 and 189; all at or below 200, so arm D runs every mutant.
- **Second TypeScript scope** admitted by rule: `OpenUltraCode` `src/verification.ts`.
- **Checks:** 95 of 95 controls on five scopes; engine checks pass, including a regression check that the version 0 packs still reproduce the committed D1 receipts.
- **Also today:** the independent survivor labels for attempt 1 (R61 amendment: 32 of 36 agree, no verdict changes) and release 1.0.2 with the fail-loudly fixes (R55 amendment).

## Prior: R62 Parallel batches for the annotation-driver prototype (2026-11-10)

[R62](2026-11-10/08-annotation-driver-parallel-batches.md) replaces the serial
batching of attempt 1. Apparatus only; the gate outcome of R61 stands.
- **Times, same 40-mutant batches, nothing reused:** Java 1,875 s to 178 s, TypeScript 412 s to 70 s, Python 97 s to 32 s; Rust unchanged at 207 s and kept serial because concurrent builds run out of memory.
- **Same results:** every mutant gets the class it has in the serial receipts of R60, on all four scopes.
- **How:** worker copies pulling from one queue, time limits scaled from the unmutated run, a long-lived Java container that compiles one file and runs one test class, one shared outcome store.
- **Defects found and fixed:** a test command that failed to start was counted as killed, and an out-of-memory build as an invalid mutant; both are now class `error`. None of the 240 kills of attempt 1 was affected.
- **For attempt 2:** the rejected TypeScript annotations are mostly state-writing calls and returned boolean expressions the taxonomy does not cover.

## Prior: R61 Annotation-driver gate, attempt 1: revise (2026-11-10)

[R61](2026-11-10/07-annotation-driver-gate-result.md) reports the counted runs
and the gate outcome.
- **Outcome:** `revise`. Rust, Python and Java meet every gated row in at least one arm; TypeScript, a held-out language, fails in every arm (kind coverage everywhere, annotation validity 71% and 76% in arm A), so the agnosticism row is not met.
- **Arm A (model annotates, two runs per scope):** all rows met on Rust, Python and Java. Run-to-run agreement on sites: 0.93, 0.78, 0.95 (TypeScript), 0.64.
- **Arm D2 (model writes the pack):** TypeScript and Java packs admitted on the first reply and equivalent to the hand-written ones (identical sites for TypeScript, 97% for Java). Java D2 meets all rows at 78% meaningful; Java D1, a different draw, was 67%.
- **What annotation adds:** a higher share of killed mutants and fewer invalid ones, most on Java; not needed to obtain mutants.
- **Also recorded:** one pack request was refused by the endpoint until its purpose was stated; survivor labels are the designer's own and provisional; 21 model calls, about 125k input and 56k output tokens.
- **Next:** owner decision on a second attempt (type-preserving operators, an operator for `return` failure sites, a second TypeScript scope, independent label check).

## Prior: R60 Arm D1 results: two languages pass, two miss one row each (2026-11-10)

[R60](2026-11-10/06-annotation-driver-d1-results.md) reports the first measured
arm: generic operators at every eligible site, no model, hand-written packs.
- **Counts (run / killed / survived / timeout / invalid):** Rust 16 / 11 / 5 / 0 / 0; Python 40 / 35 / 5 / 0 / 0; TypeScript 28 / 11 / 6 / 1 / 10; Java 40 / 17 / 14 / 5 / 4. Baselines passed before and after; every scope restored byte for byte.
- **Gated rows:** Rust and Python meet yield and meaningfulness (94% and 98%). TypeScript misses yield: mutants landed in 2 kinds, not 3, because 10 of 28 failed the type check. Java misses meaningfulness on provisional labels: 24 of 36 landed (67%, target 70%).
- **Survivor review (30, by the mechanism's designer, provisional):** 21 real test gaps, 5 equivalent, 4 undetermined. Among the gaps, three safety checks in the TypeScript scope can be removed with all tests passing.
- **Also recorded in R59 additions:** engine, harness and checks; guide version 0 reached 53% valid annotations on Rust, version 1 (two added sentences) 81% and 98% on Python, and is frozen. About 38k input and 18k output tokens spent on four development calls.
- **Next:** counted arm A runs and model-written packs (D2), then the gate decision.

## Prior: R59 Annotation-driver strategy consolidated before any run (2026-11-10)

[R59](2026-11-10/05-annotation-driver-strategy-and-harness.md) records gate
Amendment 3, made before any mutation run or model call.
- **Only horizontal arms remain.** The fixed operators (arm B) and the per-language existing tools (arm C) are removed as non-agnostic; the "stop in favour of existing tools" outcome goes with arm C, and the entry states that this removes one way for the hypothesis to lose.
- **One engine, one data pack per language** (node-kind table plus operator templates). Arms differ only in site selection: every eligible site (D1 hand-written packs, D2 model-written packs for held-out languages) or model-annotated sites (A), so A against D1 measures what annotation adds.
- **Rust scope replaced** before any run: the R57 scope had 5 eligible sites in 2 kinds and could not meet the yield row; `mmhmem` `src/setup/ui/update.rs` has 38 in 4 kinds, 17 tests passing. New admission rule: at least 12 sites in 3 kinds.
- **Bounds and classification fixed:** 40 mutants per scope for arm D by a seeded, kind-balanced selection; a broken build or import is invalid, not killed.
- **Next:** engine and harness with model-free checks, then D1.

## Prior: R58 Annotation validator controls pass; arm D in force; hard-coded operators to be removed (2026-11-10)

[R58](2026-11-10/04-annotation-driver-validator-controls.md) completes checklist
step 2 of the gate, without a model.
- **Result:** 71 of 71 controls behaved as expected on the four scopes: 11 hand-made correct annotations accepted, 40 wrong ones rejected with the expected reason, 20 mutant classifications correct. This validates the checker, not the mechanism.
- **Owner decisions (gate Amendment 2):** arm D (rules as data; the model as offline rule author for held-out languages) is approved; the hard-coded mutation operators must be removed completely and are not kept as a control set.
- **Frozen by hash:** validator, four language tables (14 to 17 lines each), scope manifest, controls.
- **Early warning:** the Rust scope has few branch sites, so the yield row may be hard to meet there.
- **Next:** comparison arm and mutation baseline (model-free), then generic operators, rule packs and guide version 0.

## Prior: R57 Scopes admitted for the annotation-driver gate (2026-11-10)

[R57](2026-11-10/03-annotation-driver-scope-admission.md) completes checklist
step 1 of the gate. Setup only; no model call, no hypothesis test.
- **Admitted scopes, baselines healthy in isolated clones:** Rust `mmhmem` `src/setup/check.rs` (212 lines, 28 tests) and Python `mvibe` `mvibe_proxy/ratelimit.py` (478 lines, 230 tests) for guide development; TypeScript `OpenUltraCode` `src/state.ts` (188 lines, 8 tests) and Java `commons-csv` `Lexer.java` (319 lines, 37 tests) held out.
- **Control:** the fixed operators yield 0, 0, 1 and 0 mutants on these scopes. The gate expected 0 everywhere; Amendment 1 corrects the yield row before any run.
- **Proposed, pending the owner:** arm D, the generic operators as declarative syntax-tree rules with no model at run time, and rule packs for the held-out languages written by the model offline and admitted by checks.
- **Next:** owner confirmation of arm D, then the validator and its controls.

## Prior: R56 Direction: agnostic annotation-guided driver, gate before design (2026-11-10)

[R56](2026-11-10/02-agnostic-annotation-driver-direction.md) records the owner's
priorities after R55 and the plan that follows. Nothing has been run.
- **Priorities:** (1) a driver that is agnostic to code and language, so the process applies horizontally; (2) hardening against LLM inference drift. Both rest on an annotation-guidance process, later optimised with bounded and verified `MIPROv2` (`GEPA` if needed). Opus endpoint now, Mistral later.
- **Hypothesis H-AD1:** the model returns located annotations from a fixed taxonomy and writes no code; a deterministic validator and generic operators produce mutants; real tests classify them; one frozen guide works on languages it was not developed on.
- **Spec:** new [`agnostic-annotation-driver`](../../.kiro/specs/agnostic-annotation-driver/brief.md) at phase `initialized`, with a [prototype gate](../../.kiro/specs/agnostic-annotation-driver/prototype-gate.md) (status not-run): three arms (annotation-guided, fixed operators as control, an existing mutation tool per language), four scopes outside the specimens, guide developed on Rust and Python and frozen for TypeScript and Java, targets fixed before execution, stop in favour of existing tools as a valid outcome.
- **Next gate:** the owner's go to screen scopes and run it.

## Prior: R55 The released binary used as an adopter would use it (2026-11-10)

[R55](2026-11-10/01-released-binary-adopter-dogfood.md) runs the 1.0.1 binary's
documented commands against this repository from the outside:
- **Disposition:** completed, exploratory; the hypothesis that every documented command gives a meaningful receipt on ordinary code was rejected.
- **Works:** `budgets` (38 files, 0 violations with `--root native/src`), `deconvolute density` (reproduces R52: 248 to 148 lines, 83 to 70 statements), `deconvolute analyze` for Rust, Java and TypeScript, and the pre-push hook's budget gate.
- **Does not work for an adopter:** `analyze --lang python` returns an empty receipt with `"passed": true` without analysis; `mutate` produced no mutants for 36 of 38 Rust files and 0 of 40 Python files because its operators replace specimen-specific substrings; the hook's `refactor/*` handling prints a note and checks nothing.
- **Breaks:** with the hook installed and the binary not on `PATH`, every push fails. Three README commands failed as written (`--file`, missing `--out`, wrong budgets file).
- **Consequence:** README, docs index and agent skill corrected; R54's tool consequence is narrowed to budget enforcement. No code changed. Owner decision open: implement, fail loudly, or drop the three non-working paths.

## Prior: R54 Self-application dogfooding, clean-clone validation & intelligent pre-push hook (2026-11-09)

> Amended 2026-11-10 by [R55](2026-11-10/01-released-binary-adopter-dogfood.md): the hook enforces complexity budgets only. The state-boundary check on `refactor/*` and `deconvolute/*` branches described below is not implemented.

[R54](2026-11-09/12-dogfood-self-application-synthesis.md) records the execution,
clean-clone verification, and tooling integration for the `dogfood-self-application`
milestone:
- **Disposition:** COMPLETED (all 7 spec tasks certified GO).
- **Clean-Clone Test Pass Rate:** 352/352 native tests (100%) passed via `cargo nextest` across 33 binaries on a pristine repository clone (`..\lesscode-dogfood-copy`) with zero manual interventions or warm caches.
- **Fixture Integrity & CRLF Normalization:** Tracked missing regenerator turn fixtures and historical `.runs/` fixtures with `text eol=lf` in `.gitattributes`, permanently eliminating byte-level hash divergences in `historical_s2_bindings_drift_reproduces_counts`.
- **Porcelain Commands:** Implemented `analyze`, `mutate`, and `density` subcommands in `native/src/cli_deconvolute.rs` with isolated specimen build targets (`target/isolated-specimen`).
- **Intelligent Git Pre-Push Hook:** Implemented `pre-push` and `install-hook` in `native/src/cli_hook.rs`, enforcing complexity budgets and AST state-boundary cleanliness on `refactor/*` and `deconvolute/*` branches while cleanly bypassing branch deletions and `LESSCODE_SKIP_HOOK=1`.
- **Complexity Budgets:** 10/10 budget checks passed across 38 governed files; 0 exceeded, 0 unlisted, 0 missing.

## Prior: R53 Multi-language deconvolution unification synthesis and four-substrate certification (2026-11-09)

[R53](2026-11-09/11-multi-language-deconvolution-synthesis.md) records the comprehensive
retrospective synthesis and full certification of the unified multi-language deconvolution
architecture across all four production language profiles (**Python**, **TypeScript**, **Java**, and **Rust**):
- **Disposition:** UNIFIED & COMPLETED (all 4 specs certified GO).
- **Substrate Unification:** Hosted natively under `lesscode-core` with 4 normative carrier contracts (`StateFrameAnalyzer`, `PureCoreVerifier`, `AdverseMutantGenerator`, `SpecimenTraceRunner`).
- **Cross-Language Compression:** Verified substantial whole-responsibility compression across 4 distinct production specimens:
  - Python (Facdrone): −23.80% lines, −11.20% stmts (Breaking Density Trap, 100 cases, 0 divergences).
  - TypeScript (`OpenUltraCode`): −43.78% lines, −40.38% stmts (100 cases, 0 divergences).
  - Java (`apache/commons-csv`): −38.46% lines, −27.68% stmts (100 cases, 0 divergences).
  - Rust (`lesscode-core` lifecycle): −40.32% lines, −15.66% stmts (100 cases, 0 divergences).
- **Adverse Sensitivity:** 100% kill rate across 20/20 mandatory adverse mutation controls (Python 8/8, TypeScript 4/4, Java 4/4, Rust 4/4).
- **Apparatus Budget:** 7,173 canonical lines held (0 lines overage against R41 ceiling); 36/36 modules within complexity budgets.
- **Substrate Health:** 338/338 `cargo-nextest` tests passed across 33 binaries; 32/32 `pytest` tests passed in parallel.
- **Reference Published:** Architecture and extension guide published in `docs/multi-language-deconvolution.md`.

## Prior: R52 Rust deconvolution adapter prototype feasibility gate (2026-11-09)

[R52](2026-11-09/10-rust-adapter-prototype-gate.md) records the execution and
certification of the bounded prototype feasibility gate for `rust-deconvolution-unification`:
- **Disposition:** PROCEED (receipt and SHA-256 sidecar verified: `bcd823bc7edc79141d4dcbe5951eb8b2ec39773be5471fe7964d31a5248ff4a6`).
- **Candidate Admission:** Admitted native process lifecycle supervisor responsibility (`native/src/recording.rs`, lines 712–969) in isolated copies under `native/adapters/rust/baseline_lifecycle.rs` and `native/adapters/rust/candidate_lifecycle.rs` with frozen Rust 1.98.1 and cargo-nextest 0.9.143.
- **Compression:** 248 -> 148 canonical lines (−40.32%, 100 lines removed) and 83 -> 70 statements (−15.66%, 13 statements removed), meeting both the 20%/10% relative and 50/10 absolute thresholds.
- **Protocol & Codec:** Verified line-delimited JSON IPC event frames (`rust-nextest-v1`) and IEEE-754 value transports.
- **Finite-Domain Parity:** 100 Cartesian cases evaluated across process lifecycle, exit codes, and environment configs with 0 divergences.
- **Adverse Controls:** 100% kill rate (4/4 killed) on mandatory adverse controls (deadline kill omission, inverted verdict condition, inverted error classification, dropped environment isolation).
- **Specimen Regression:** Full Nextest execution across 28 binaries passed with 314 tests run, 314 passed, 0 failures.
- **Apparatus Budget:** Complexity budgets passed 10/10; R41 7,173 ceiling held.
- **Outcome:** Technical design approved and Phase 2 production tasks unblocked for foreground execution.

## Prior: R51 Java deconvolution adapter prototype feasibility gate (2026-11-09)

[R51](2026-11-09/09-java-adapter-prototype-gate.md) records the execution and
certification of the bounded prototype feasibility gate for `java-deconvolution-unification`:
- **Disposition:** PROCEED (receipt and SHA-256 sidecar verified: `207765c042ef02e263c6d19648c19532675dc9c52b95e6044a179d409e2b3410`).
- **Candidate Admission:** Admitted real licensed specimen `apache/commons-csv` at commit `2bd66bf181d44ffc92806f585d3955e915666f7f` (Apache License 2.0) with Microsoft OpenJDK 21.0.12.1 LTS, Apache Maven 3.9.9, and JUnit 5.
- **Compression:** 221 -> 136 canonical lines (−38.46%, 85 lines removed) and 112 -> 81 statements (−27.68%, 31 statements removed), substantially exceeding both the 20%/10% relative and 50/10 absolute thresholds.
- **Protocol & Codec:** Verified line-delimited JSON IPC event frames (`java-junit-v1`) and buffered stream flushing via `ProbeMain.java`.
- **Finite-Domain Parity:** 100 Cartesian cases evaluated across 5 CSV formats and 10 field data structures with 0 divergences.
- **Adverse Controls:** 100% kill rate (4/4 killed) on mandatory adverse controls (state mutation omission, lock cleanup omission, delimiter sequencing inversion, dropped stream flush).
- **Specimen Regression:** Full Maven surefire execution of `CSVPrinterTest` passed with 168 tests run, 0 failures, 0 errors, 7 skipped.
- **Apparatus Budget:** Complexity budgets passed 10/10; R41 7,173 ceiling held; adapter probe bounded at 212 canonical Java lines.
- **Outcome:** Technical design approved and Phase 2 production tasks unblocked for foreground execution.

## Prior: R50 TypeScript deconvolution adapter prototype feasibility gate (2026-11-09)

[R50](2026-11-09/08-typescript-adapter-prototype-gate.md) records the execution and
certification of the bounded prototype feasibility gate for `typescript-deconvolution-adapter`:
- **Disposition:** PROCEED (receipt and SHA-256 sidecar verified).
- **Candidate Admission:** Admitted real licensed specimen `OpenUltraCode` with pinned Node.js 24.22.1, TypeScript 6.0.3, and tsx 4.22.4.
- **Compression:** 201 -> 113 canonical lines (−43.78%, 88 lines removed) and 52 -> 31 statements (−40.38%, 21 statements removed), exceeding both the 20%/10% relative and 50/10 absolute thresholds.
- **Protocol & Codec:** Verified line-delimited JSON IPC event frames (`ts-node-v1`) and IEEE-754 bit representations for floats via `runner.mjs`.
- **Finite-Domain Parity:** 100 Cartesian cases evaluated across notice factories and degradation summary with 0 divergences.
- **Adverse Controls:** 100% kill rate (4/4 killed) on mandatory adverse controls (severity inversion, slugify drop, stream buffer drop, summary error order).
- **Specimen Regression:** `pnpm exec tsc --noEmit` passed exit 0; `tests/degradation.test.ts` passed 3/3.
- **Apparatus Budget:** Complexity budgets passed 10/10; R41 7,173 ceiling held; adapter probe bounded at 175 JavaScript lines.
- **Outcome:** Technical design approved and Phase 2 production tasks unblocked for foreground execution.

## Prior: R49 Native deconvolution closure prototype feasibility gate (2026-11-09)

[R49](2026-11-09/07-native-closure-prototype-gate.md) records the execution and
certification of the bounded prototype feasibility gate for `native-deconvolution-closure`:
- **Disposition:** PROCEED (receipt and SHA-256 sidecar verified).
- **Parity:** Exact match on frozen E3/E4 replay digests with 0 Level 1 / Level 2 divergences.
- **Fidelity:** 0 counterexamples across bounded Cartesian domain `[-10, 10]` + edge cases.
- **Adverse Controls:** 100% kill rate across mandatory controls (E3 AuditedAbsence, E4 F1, F3, F7).
- **Compression:** Reproduced E5 Lifecycle (−24.82% lines, −12.17% stmts) and E6 Report Envelope (−23.01% lines, −10.49% stmts).
- **Apparatus Accounting:** 7,173 canonical lines held (0 overage against R41 ceiling); 10/10 budget tests passed.
- **Outcome:** Technical design and foreground TDD implementation tasks formally unblocked and approved.

## Prior: R48 aligned provisional adapter designs (2026-11-09)

[R48](2026-11-09/06-aligned-language-adapter-designs.md) records the shared native
contract and TypeScript/Java/Rust provisional designs, per-language research and
bounded prototype gates. Requirements approved by the owner's design request;
designs unapproved, gates not-run, production tasks absent.

## Prior: R47 specification review and hypothesis amendment (2026-11-09)

[R47](2026-11-09/05-deconvolution-spec-review-amendment.md) revises native closure
and the TypeScript/Java/Rust requirements after cross-spec review. Java is an
authorized target; runtime adapters remain counted; bounded claims and unknown
outcomes are explicit; native production planning awaits prototype evidence.
Documentation review only: no new experiment or runtime qualification.

## Prior: R46 Report envelope adoption & generalization maturity gate certification (2026-11-09)

[R46](2026-11-09/04-e6-report-envelope-adoption-maturity-gate.md) records the formal adoption of the
deconvoluted **Report Envelope & Snapshot Store** group (6 files) into `.runs/e6-adopted-report-envelope/`
and certifies the dual-specimen **Generalization Adoption Maturity Gate**:
- **Dual-Specimen Replication:** Confirmed cross-subsystem deconvolution across two distinct Facdrone
  domains: Signal Lifecycle Monitors (E5, 7 files, −24.82% lines, −12.12% statements) and Report Envelope
  & Snapshot Store (E6, 6 files, −23.01% lines, −10.49% statements).
- **Combined Impact:** 13 files, 3,433 baseline lines compressed to 2,616 adopted lines (−23.80%),
  with 11.20% statement reduction, 33.92% function reduction, and 30.93% call edge reduction.
- **State Confinement:** 15 total write sites confined strictly to designated imperative shells (11 in
  lifecycle store, 4 in reporting request store).
- **Behavioral Soundness:** 100% adverse mutant kill rate, 0 bounded pure-core counterexamples, 0 replay
  divergences, and 100% test pass on all unit and acceptance suites.
- **Rust Apparatus Volume:** Counted Rust substrate held at 7,173 canonical lines (0 overage against R41 ceiling).
- Full adoption artifacts verified: `e6-adopted-report-envelope.diff` and `e6-adoption-report.json`.

## Prior: R45 E6 generalization trial qualification — report envelope certified (2026-11-09)

[R45](2026-11-09/03-e6-generalization-trial-qualification.md) records the empirical qualification
of **Experiment E6: State-Boundary Deconvolution Generalization Trial** on an unseen
Facdrone subsystem (**Report Envelope & Snapshot Store**, 6 files, 1,938 baseline lines):
- **Canonical lines reduced from 1,938 to 1,492** (−23.01% reduction vs $\ge 20\%$ target).
- **AST statements reduced from 925 to 828** (−10.49% reduction vs $\ge 10\%$ target; Density Trap defeated).
- **Functions de-fragmented from 102 to 63** (−38.24% reduction vs $\ge 20\%$ target).
- **Call edges de-fragmented from 145 to 101** (−30.34% reduction vs $\ge 25\%$ target).
- **Write sites confined strictly to 4 imperative shell locations** in `reporting_request_store.py`.
- **Pure module share held at 5/6 (83.3%)** (target $\ge 50\%$).
- **Adverse mutation sensitivity:** 4/4 adverse controls killed (100% kill rate, E6-C1 to E6-C4).
- **Pure-core bounded equivalence:** 0 counterexamples across bounded domain $[-10, 10]$ and edge cases.
- **Rust apparatus stability:** 7,173 canonical lines held (0 overage against R41 ceiling).
- **Acceptance test battery:** 46/46 passed with zero regressions across `facdrone-reporting`, `facdrone-kernel`, and root suites.
- Certified as a generalizable, subsystem-agnostic architectural compression loop.

## Prior: R44 E6 state-boundary deconvolution generalization trial setup (2026-11-09)

[R44](2026-11-09/02-e6-generalization-trial-setup.md) records the preregistration and baseline
census for **Experiment E6: State-Boundary Deconvolution Generalization Trial** on an unseen
Facdrone subsystem (**Report Envelope & Snapshot Store**, 6 files, 1,938 canonical lines):
- Scope: 6 files across `facdrone-reporting` and `facdrone-kernel` (`reporting_request.py`,
  `reporting_request_store.py`, `snapshot_store.py`, `evidence_validation.py`,
  `composition_errors.py`, `canonical.py`).
- Baseline metrics: **1,938 canonical lines**, **925 AST statements**, **102 functions**,
  **145 intra-scope call edges**, and **4 write sites** (confined to `reporting_request_store.py`).
- Preregistered target bars: $\le 1,550$ canonical lines ($\ge 20\%$ reduction), $\le 832$ statements
  ($\ge 10\%$ reduction; Density Trap defense), $\le 81$ functions, $\le 108$ call edges.
- Adverse controls battery qualified: 4/4 passed (missing failure state, buffer truncation,
  out-of-domain purity, error order inversion).
- Rust substrate apparatus volume held at 7,173 canonical lines (0 overage against R41 ceiling).
- Acceptance test battery verified passing across reporting and root suites.

## Prior: R43 lifecycle deconvolution adoption & self-application maturity gate (2026-11-09)

[R43](2026-11-09/01-e5-lifecycle-adoption-maturity-gate.md) records the formal adoption of the
deconvoluted Facdrone `lifecycle` monitor group (7 files) into `.runs/e5-adopted-lifecycle/` and
the certification of the **Self-Application Maturity Gate**:
- **Canonical lines reduced from 1,495 to 1,124** (−24.8% reduction, −371 lines).
- **AST statements reduced from 759 to 667** (−12.1% reduction, −92 statements).
- **Functions de-fragmented from 69 to 50** (−27.5% reduction, −19 functions).
- **Call edges de-fragmented from 89 to 61** (−31.5% reduction, −28 edges).
- **State write sites confined to 11 imperative shell locations** (target $\le 12$).
- Verified with exact pre/post SHA-256 hashes and 1,504-line unified diff.
- Full regression passed: specimen 108/108 tests, replay oracle 50/50 sequences with 0 Level 2 divergences.
- Verification suites green: Rust 241/241, Python state-boundary 32/32, apparatus volume 7,173 lines (0 overage).

## Prior: R42 E5 state-boundary deconvolution trial — breaking the density trap on lifecycle (2026-11-08)

[R42](2026-11-08/09-e5-state-boundary-deconvolution-lifecycle.md) records the empirical
execution and successful qualification of **Experiment E5: State-Boundary Deconvolution Trial**
on the Facdrone `lifecycle` monitor group (7 files, 1,495 baseline lines). The trial formally
**broke the Density Trap** observed in E4 without oracle blindness:
- **Canonical lines reduced by 24.8%** (1,124 lines vs $\le 1,196$ target / 1,495 baseline).
- **AST statements reduced by 12.1%** (667 stmts vs $\le 683$ target / 759 baseline).
- **Functions de-fragmented by 27.5%** (50 funcs vs $\le 55$ target / 69 baseline).
- **Call edges de-fragmented by 31.5%** (61 edges vs $\le 66$ target / 89 baseline).
- **100% kill rate on adverse controls** (E3 `AuditedAbsence`, E4 F1, F3, F7).
- **0 counterexamples on bounded pure-core equivalence** over $[-10, 10]$ and edge cases.
- **Rust apparatus volume held at 7,173 lines** (0 overage against R41 amended ceiling).
- Replay oracle green across 50/50 sequences and 317 operations with 0 Level 2 divergences.
- Unit suites fully passing: Rust 241/241, Python state-boundary 32/32, specimen 108/108.

## Prior: R41 state-boundary apparatus ceiling amendment — owner chose amend (2026-11-08)

[R41](2026-11-08/08-state-boundary-ceiling-amendment.md) records the owner disposition
on the prototype feasibility gate overage (option (a) of the preregistered outcome rule;
instruction: *"a) it is"*): **the Rust-substrate apparatus ceiling is amended from 7,004 to
7,173 canonical lines** — exactly the measured total on the extended counted set (7,079 on
R37 counted set + 94 for `native/src/state_contract.rs`), effective as a **no-growth cap**.
All 4 adverse controls (E3 `AuditedAbsence`, E4 F1 `Silent_None`, E4 F3 `Buffer_Drop`,
E4 F7 `Error_Order`) achieved 100% kill rates, the Facdrone `lifecycle` density trap was
defended, and the feasibility gate is marked **PASS** under the amended ceiling. Spec
`state-boundary-deconvolution` fully qualified.

## Prior: R40 structural consolidation without line reduction — the scattered state and density boundary (2026-11-08)

[R40](2026-11-08/07-structural-deconvolution-and-state-boundaries.md) synthesizes
the empirical results on structural consolidation across E1–E4, R11, R25, R26,
and R34–R39: an LLM can de-fragment scattered responsibilities (−24.6% functions,
−29.2% call edges, write sites confined to 11 shell functions, 51.9% pure core),
but canonical lines dropped only 6.0% (missing the 10–20% target; density replaced
fragmentation), while trace oracles missed exception-side state and partial buffer
divergences. Proposes a new specification (`state-boundary-deconvolution`) using
software verification techniques (frame conditions, bounded SMT pure-core equivalence,
adverse mutant killing) to soundly govern state-boundary deconvolution.

## Prior: R39 confirms refinement correctness defects; repaired and verified (2026-11-08)

[R39](2026-11-08/06-refinement-correctness.md) reproduced wrong-candidate
delivery, lost runtime feedback, erroneous obligation discharge, false oracle
passes after failure/timeout, and unsupported validity/compression outcomes.
Repaired and verified under specification
[refinement-evidence-integrity](../../.kiro/specs/refinement-evidence-integrity/spec.json)
(Task group 12; commit `37f21c4`, handover `f7c254f`). Adversarial probe battery
passes (Rust 9/9, Python 5/5) with regenerator suites green (Rust 50/50, Python 59/59).

## Prior: R38 U5 ceiling amendment — owner chose amend (2026-11-08)
[R38](2026-11-08/05-u5-ceiling-amendment.md) records the owner disposition on
the R37 negative (option (a) of the preregistered outcome rule; instruction:
"proceed with the amended ceiling"): **the Rust-substrate U5 ceiling is
amended from 2,270 to 7,004 canonical lines** — the measured R37 total on the
R37 counted set, effective as a **no-growth cap** (growth or baseline change
needs a further dated decision). The 20% whole-replacement reduction claim is
**retired as falsified**; the Python 3,783 and Rust 7,049/7,004 negatives
stay on record (`Req 7.8`) and the R36/R37 gate records remain FAIL under
the frozen 2,270. **Slice B adoption planning unlocked; production adoption
execution still gated** by `Req 7.9`/`7.11` and unscheduled until the owner
requests a plan. Task 10.1; no code, budget or frozen record changed.

## Prior: R37 slim-pass gate FAIL as measured — levers exhausted, disposition resolved by R38 (2026-11-08)
[R37](2026-11-08/04-rust-slim-pass-remeasure.md) executed under task 9.6
(gate record
[`rust-u5-gate-r37.json`](../experiments/deconvolution/probabilistic-refine/results/rust-u5-gate-r37.json)):
G1-R items 1/2/3/5 **PASS** through the slimmed substrate (six Tier-1
controls zero-runtime, E4 three-turn replay at the recorded scores with
frozen digests, `k=2` exhaustion retains best, receipts
admission-verified), and **U5 = 7,004 canonical lines vs the unchanged
2,270 ceiling — FAIL by 4,734**. The preregistered levers delivered −45
net of 7,049 (defect-7 split: receipt 178 → 124 + jsonval 130; shared
kernels `j_get`/serde-bridge/escape consolidated into `jsonval.rs`;
writer merges abandoned on semantic grounds under the byte-parity
freezes): the bounded-consolidation hypothesis is falsified — the overage
is port expression volume, not consolidation overhead. Negative retained,
no ceiling or baseline change; Slice B stays withheld (`Req 7.10`); owner
chooses amend (dated decision) or close — *resolved 2026-11-08 by
[R38](2026-11-08/05-u5-ceiling-amendment.md): amend*. Task group 9 closed —
handover at
[.kiro/task_9_handover.md](../../.kiro/task_9_handover.md).

## Prior: Rust-apparatus slim pass preregistered — task group 9 opened (2026-11-08)
[R37](2026-11-08/04-rust-slim-pass-remeasure.md) freezes the slim-pass
accounting before any slim code exists, recording the owner dispositions on
the R36 negative: slim the Rust apparatus (`regenerator.rs` 2,798 the
largest lever; shared derivable structure across `surface`/`measures`/
`recording`), split `receipt.rs` for defect 7 without amending R30, and
investigate the g1r `FROZEN_RUST_CANONICAL = 1,333` provenance. The counted
set extends to every new module the split/slim adds (`Req 6.1`); the
canonical method, behavioral freezes and the unchanged 2,270 ceiling carry
over from R36; PASS requires the counted total ≤ 2,270 with budgets exit 0
(at least −4,779 of 7,049, −68%); fail returns to the owner with
amend/close options.

## Prior: Rust-port U5 re-measurement FAIL as measured — owner disposition open (2026-11-08)
[R36](2026-11-08/03-rust-port-u5-remeasure.md) executed under task 8.6
(commit `7657969`; gate record
[`rust-u5-gate.json`](../experiments/deconvolution/probabilistic-refine/results/rust-u5-gate.json)):
G1-R items 1/2/3/5 re-discharged **PASS** through the Rust substrate (six
Tier-1 controls zero-runtime; E4 three-turn replay at the recorded scores
with frozen digests reproduced; k=2 exhaustion retains best; receipts
admission-verified), and **U5 = 7,049 canonical lines vs the frozen 2,270
ceiling — FAIL by 4,779** (new modules 6,382 + re-measured base 452 + shim
178 + hooks 16 + 21). The smaller-replaceable-representation hypothesis is
falsified on the target substrate (port ≈ 2.9× the Python stage-1 canonical);
no target weakened, negative retained; Slice B stays withheld (`Req 7.10`);
slim/amend returns to the owner. Task group 8 closed — handover at
[.kiro/task_8_handover.md](../../.kiro/task_8_handover.md).

## Prior: Rust-port U5 re-measurement preregistered — task group 8 opened (2026-11-08)
[R36](2026-11-08/03-rust-port-u5-remeasure.md) freezes the re-measurement
accounting before any port code exists: counted set = ported
`native/src/{surface,measures,recording,regenerator}.rs` + re-measured
`admission.rs`/`receipt.rs` + enumerated apparatus hooks; unchanged
rustfmt/ruff canonical method, unchanged 2,270 ceiling against
$B_{\text{app}}^{\text{expanded}} = 2{,}838$; Python modules stay as
superseded stage-1 reference; deterministic G1-R checklist items 1/2/3/5 must
re-discharge through the Rust substrate; pinned `ureq` recorded as a Req 3.6
dependency decision; pass unlocks Slice B adoption planning, fail returns to
the owner with slim/amend options.

## Prior: U5 disposition — Python stage-1 negative accepted; Rust re-measure gate before Slice B (2026-11-08)
[R35](2026-11-08/02-u5-disposition.md) records the owner disposition for the
failed whole-replacement apparatus ceiling: U5 stays failed on the Python
stage-1 prototype (3,783 vs 2,270 canonical lines; frozen ceiling unchanged,
`Req 7.8` negative retained). Slice B adoption remains withheld until a
Rust-port re-measurement of U5 under the same accounting passes the frozen
ceiling. The logged concise-output directive dispatch defect (emitted by
`next_rollout_params` but never sent to the model) is fixed in
`regenerator.py` with dated comments and a regression test.

## Prior: G1-R discharged — live weaker-model recovery reaches R=1.0; U5 fails as measured (2026-11-08)
[R34](2026-11-08/01-g1r-gate-and-live-recovery.md) executes the frozen G1-R
checklist for the DSPy-style refinement slice: all deterministic items green
(six Tier-1 controls fail closed with zero runtime, E4 replay sub-second,
k=2 exhaustion, every receipt admission-verified), and live run B-2 shows
`claude-haiku-4-5` recovering from a Tier-1 out-of-scope failure to full
acceptance R=1.0 inside the frozen budgets, with cost and negatives retained.
The live run also surfaced and fixed five apparatus defects (temperature and
effort dispatch, structure-grammar prompt bug, structure diagnostics, key
normalization). U5 is recorded as failed (3,783 vs 2,270 canonical lines);
owner disposition open, Slice B adoption still withheld. Python remains the
stage-1 prototype substrate; Rust is the target. Verification: pytest -n 4
(648 passed; only documented pre-existing failures), cargo nextest 123/123,
lesscode-core budgets exit 0, full receipt/sidecar hash sweep.

## Prior: constraint-guided-compression design reconciled with H8/H9 + DSPy-style probabilistic turn refinement (2026-11-07)
Owner reviewed the feasibility audit of `constraint-guided-compression` and directed
reconciling the spec with the completed detection/recording cores plus DSPy-style
probabilistic multi-turn prompt refinement (`dspy.Refine`/`BestOfN` with two-tier
static-assertion and runtime-oracle reward feedback) so weaker or non-deterministic
models recover from thinking-budget, syntax, surface or divergence failures across
bounded turns ([R33](2026-11-07/07-dspy-probabilistic-turn-refinement-design.md),
[design](../../.kiro/specs/constraint-guided-compression/design.md)). Demonstrated
detection/loop apparatus (`surface`, `measures`, `editscope`, `testinfra`, single-turn
`regen_turn`) is separated from the provisional multi-rollout refinement slice (`H10`,
gated by spike `G1-R` in [prototype-gate.md](../../.kiro/specs/constraint-guided-compression/prototype-gate.md));
design approval remains owner-gated.

## Prior same-day milestone: attempt-2 descope passes the full gate — verdict proceed (2026-11-07)
Owner descope decision executed as attempt 2 of 2, preregistered before any code
change ([R31](2026-11-07/05-descoped-stratum-rerun.md), plan §9): the Rust digest
twin left the prototype (Rust keeps self-contained sidecar admission), Python owns
digests. Both scopes re-produce the frozen identities byte-identically (E3
`92a48f77…` ×2 + three goldens zero divergence; E4 `0b2e383d…` ×2 at exactly
5,479,067 bytes + golden-a zero divergence), both candidates accepted as
preregistered (level2_ops=0), 9/9 controls fail closed, and every frozen target
passes: **C_app = 1,193 ≤ 1,364 (ratio 0.6997), saved 512 ≥ 200, C_total 2,691 ≤
3,003, per-scope smaller, U7 1,193 ≤ 1,400** (attempt-1's 1,808 retained; like-
for-like sensitivity 272 ≥ 200). Evidence review approved inline with recorded
residual risks (`results/s10-evidence-review-2.json`); **gate verdict proceed** —
only the demonstrated slice may seek normal production design approval, which
stays owner-gated. Attempt-1 history: [R30](2026-11-07/04-s10-shared-core.md)
(reuse proven mechanically, U5/U7 failed as frozen, C_app 1,808); S9 in
[R29](2026-11-07/03-s9-surface-contracts.md) (contract B ratified); S2 in
[R28](2026-11-07/02-s2-artifact-admission-digest-parity.md) (admission + Rust
digest parity, U7 ceiling raised to 1,400 by owner mid-run).

## Proposal: solution-first native slice (2026-11-07)
[R27](2026-11-07/01-solution-first-native-slice.md): owner-requested spec amendment,
U1-U7 before production expansion, updated 2026-11-07 with empirical canonical baseline
audit (`2,222` written -> `3,963` canonical lines across 14 files; 3-stratum `U5`
accounting; `U7` brownfield/verbatim-extraction rules) and Rust library audit (`Cargo.lock`
pins kept; `serde_jcs`, `fs4`, `quick-junit`, `gix`, `pg_query` dropped/replaced; `syn`,
`toml_edit`, `windows-sys`, `ruff_python_parser`, `rusqlite`, `oxc`, `junit-parser` added).
Prototype plan frozen and owner-approved 2026-11-07; execution started (see R28).
Historical H8 evidence below remains unchanged.

Current direction (2026-11-06): H7's static state models and family-duplication
selection did not verify or compress real scattered code. H8 proposes a recorded,
mutation-scored trace oracle, edge-based convolution measures and regeneration
behind the public API. Gates run so far (R20 to R24): oracle mechanics positive
on Rust and Python, gain predictions failed, edge selector rejected, solver rung
rejected as equivalence authority but useful as a verified counterexample
finder. Regeneration has run on two scopes. E3 (R25, 2026-11-04): one round met
the oracle with 16% fewer lines and 29% fewer edges, mutation sensitivity missed
its bar (0.643; the original scores 0.654 on the same arm) and one oracle-blind
difference on an exception path is real. E4 (R26, 2026-11-06), the first of the
three scopes the report's section 8 preregistered, run with the recorded
regenerator program on the lifecycle monitor group: accepted at turn 3, 25%
fewer functions and 29% fewer edges inside the predicted bands, the line
prediction failed (6% fewer canonical lines against 10 to 20), sensitivity 0.679
inside its band with the original at 0.414 on its own sample, seven oracle-blind
differences on error paths, zero divergence on 74 held-out sequences (26 of
them the acceptance seed sequences replayed again, 47 new; 1 generated sequence
duplicates an acceptance one), the differential void on a failed solver
control. H8(c) stands on two scopes with a lowered protection claim and one
failed prediction; the owner decides between a
third scope, adoption of the E4 candidate outside the loop, or prevention. Four
protocol amendments are proposed from E4 (the surface includes the recorder's
imports; turn parameters preregistered; hashes over bytes as stored; labels
distinct beyond letter case). The loop as prototyped is documented
algorithm-agnostically in the
[deconvolution-loop spec](../../.kiro/specs/deconvolution-loop/requirements.md)
with per-requirement feasibility status. The earlier CodeQL direction is history.

## Dated lab reports

Report for third parties (2026-11-04, no new evidence):
[Regenerate behind the surface](../reports/deconvolution-loop-applied.md),
self-contained: setting, the loop in eight steps with definitions, results
R20 to R25, lessons, limitations, a falsifiable prediction for the next
three scopes, the loop's place in spec-driven development, glossary and
references.

Latest proposal: [R19: dynamic deconvolution direction](2026-11-03/01-dynamic-deconvolution-direction.md).
Owner review of 2026-11-03 recorded; static dataflow and LLM-authored SMT models
withdrawn as equivalence authority; H8 registered with kill criteria per stage.

Latest replacement: [R18: provider representation](2026-11-02/18-provider-representation-trial.md).
Rejected: 30 fewer named-argument bindings but 12 more lines and 10 more statements;
70 selected tests pass on each version and 32 provider observations match.

Latest screen: [R17: provider orchestration](2026-11-02/17-provider-orchestration-screen.md).
Common algorithms are already shared; the narrow orchestration rewrite fails
admission. Repeated constructor fields remain an unresolved representation lead.

Latest trial: [R16: evidence-publication binding](2026-11-02/16-evidence-publication-trial.md).
Rejected: 1,256 → 1,259 AST statements across the complete accounting scope.
128 differential observations matched; lifecycle integration remains untested.

Latest discovery: [R15: CodeQL cross-module cases in Facdrone](2026-11-02/15-codeql-cross-module-cases.md).
781 extracted files; three connected responsibility hypotheses, with state,
caller and behavior boundaries. Discovery evidence, not measured savings.

Latest modeling result: [R14: AALpy learns real snapshot behavior](2026-11-02/14-aalpy-state-learning.md).
Four states under copy-only observations, six with source reads; 2,144 bounded
sequences matched and six wrong merges were rejected. No compression claimed.

Latest deployment correction: [R13: Windows client, Cairn HTTP server](2026-11-02/13-windows-client-cairn-api.md).
Real cross-environment HTTP coordination passed; Windows remains the agent runtime.

Latest: [R12: direct Cairn adoption prototype](2026-11-02/12-cairn-direct-prototype.md).
Upstream API/storage and local execution tests pass; LLM search and compression
remain untested. R11 is a search limitation, not proof of uncompressible code.

Folders group the discussion or research day in Europe/Berlin. Each entry records
its actual writing date separately when backfilled. Execution dates, when known,
remain separate; file modification times are not treated as execution evidence.

| Discussion / record day | Record | Status |
| --- | --- | --- |
| 2026-11-07 | [33: H8/H9 + DSPy-style probabilistic turn refinement design](2026-11-07/07-dspy-probabilistic-turn-refinement-design.md) | planned: `constraint-guided-compression` reconciled around completed `capability-compression-core` + `deconvolution-native-core` + proven loop apparatus (`surface` Contract B, 3-stratum `measures`, `editscope`, `testinfra`, `regen_turn`) and extended with H10 DSPy-style `Refine`/`BestOfN` two-tier reward/assertion iteration for weaker or non-deterministic models; H7 static-CodeQL/authored-SMT pipeline withdrawn from design; demonstrated slice (`proceed`) separated from H10 multi-rollout slice (`revise`, gated by `G1-R`); `design.md` generated, owner approval pending |
| 2026-11-07 | [32: Rust specimen detection boundary](2026-11-07/06-rust-specimen-testinfra-detection.md) | completed: deconvolution-loop task 2.3 (optional, outside the gate's Python scope, owner go) — `testinfra.py` unmodified on the E1 frozen mmhmem copy: exit 0, `runner: unknown`, 0 test files, 0 seeds, 0 parse errors; known runner is cargo test (ci.yml) with nextest via `.config/nextest.toml`; corpus 22 `tests/*.rs` + 18 inline modules + 128 `#[test]`; the nested `ci/` uv project (pytest ≥8.0.2, 9 `ci/scripts/test_*.py`) is invisible to root-config detection; one true positive: the `docker-compose.yml` store file fact; no Rust regeneration claim; last open loop task closed |
| 2026-11-07 | [31: Descoped replacement stratum — attempt 2](2026-11-07/05-descoped-stratum-rerun.md) | completed, **verdict proceed**: owner descope preregistered then executed (attempt 2 of 2) — Rust digest twin deleted, admission self-contained (415 canonical; it had imported pyjson); both scopes reproduce frozen identities (`92a48f77…`/`0b2e383d…` 5,479,067 bytes), candidates accepted as preregistered, 9/9 controls; C_app 1,193 ≤ 1,364 (0.6997), saved 512 (like-for-like 272), C_total 2,691 ≤ 3,003, U7 1,193 ≤ 1,400 (attempt-1 1,808 retained); U1-U7 pass; inline evidence review approved with residual risks; production still owner-gated |
| 2026-11-07 | [30: S10 shared core](2026-11-07/04-s10-shared-core.md) | completed with verdict **revise**: shared engine reproduced both frozen recorders byte-identically (E3 `92a48f77…`, E4 `0b2e383d…` 5,479,067 bytes), zero-divergence replays, both candidates accepted as preregistered, 9/9 adverse controls fail-closed, U4 freeze re-proven; U5/U7 FAILED as frozen (C_app 1,808 > 1,364, saved −103, U7 1,808 > 1,400); cost driver the Rust byte-parity core (1,046); owner decision required (descope ~1,072 / stop) |
| 2026-11-07 | [27: Solution-first native slice](2026-11-07/01-solution-first-native-slice.md) | planned (`solution_unlock=not-run`); spec amendment for U1-U7 before production expansion; Amendment 2 (2026-11-07): read-only canonical baseline audit (`2,222` written -> `3,963` canonical lines across 14 files; `e1b_oracle.py` 406->659, `e4_oracle.py` 534->1,258), 3-stratum `U5` accounting ($B_{\text{app}}, B_{\text{ops}}, B_{\text{seeds}}$), `U7` brownfield/verbatim-extraction rules, and Rust crate audit (`serde_jcs`, `fs4`, `quick-junit`, `gix`, `pg_query` dropped/replaced; `syn`, `toml_edit`, `windows-sys`, `ruff_python_parser`, `rusqlite`, `oxc`, `junit-parser` added) |
| 2026-11-06 | [26: E4 lifecycle regeneration](2026-11-06/01-e4-lifecycle-regeneration.md) | second scope (seven lifecycle monitor files, 1,495 lines) with the recorded regenerator program; T1 held, accepted at turn 3 (turn 1 spent 128k tokens thinking, turn 2 failed on a surface gap: a name only the recorder imports); T2 failed (1,405 canonical lines against at most 1,345, 6.0% fewer); T3 held (52 functions for 69) and T4 held (63 edges for 89); T5 held (candidate 19/28 = 0.679 in the band 0.60 to 0.75; original 12/29 = 0.414 by its sample's composition, stated in advance; 0.70 unmet on both; M2 to M4 caught on both trees, M1 missed as predicted); T6 held on F1 and F3 from 128 review probes (26 disagreements, F1 to F6), 6.5 void under criterion 4 (the window solver control failed), compare and standing counterexamples kept as evidence (F7); T7 held, zero divergence on 74 held-out sequences (26 of them the acceptance seed sequences replayed again, 47 new; 1 generated sequence duplicates an acceptance one); S held (11 writes in shells, pure 0.519, patch check pass); residue corrected (`lifecycle_root.lifecycle_store` is reached, `bind_lifecycle` unreached and undeclared); missing-test report 28 entries; 771/900 lines; about $6.96 of model time (estimate); owner decision third scope, adopt or stop |
| 2026-11-04 | [25: E3 evidence regeneration](2026-11-04/02-e3-evidence-regeneration.md) | T1-T3 held in 1 round (1,274→1,074 lines, 660→546 statements, 84→60 edges, 11 writes all in shell, 36/51 pure); T4 failed on kill rate 18/28 = 0.643 (M1-M3 caught); T5 found one realizable oracle-blind difference (`capture_payload` absences after an exception) that reading and 300 random inputs had already exposed; reviewer caveat: 26 statements are packing; no kill criterion fired; 545/400 lines; owner decision E3b, E4 or stop. Amendment 2 (2026-11-06, task 3.2): the same arm on the original scores 17/26 = 0.654 (post hoc 0.773), one kill apart from the candidate, so T4 measures the arm's sensitivity on this module, not a loss from regeneration; five real corpus gaps shared by both trees |
| 2026-11-04 | [24: E1c-2 corrected solver rung](2026-11-04/01-e1c2-corrected-solver-rung.md) | S1-S4 failed, S5 held; kill criterion fired on a verified pipeline (4 verifier checks passed); 7 of 10 targets killed (4 by the reader-inclusive alphabet alone, 3 only by realised solver inputs), 3 unresolved, 0 proved equivalent; guided generation rejected as equivalence authority for DB-entangled predicates; owner decision E3 or stop |
| 2026-11-03 | [23: E1c concolic rung](2026-11-03/05-e1c-concolic-rung.md) | R1-R4 failed, R5 held; 1 of 10 targets solved, 1 of 8 killed; cause: driver encoding concretised strings and the oracle never calls the reader functions; corrected rerun pending owner decision |
| 2026-11-03 | [22: E1b Facdrone evidence-publication oracle](2026-11-03/04-e1b-evidence-publication-oracle.md) | Oracle byte-deterministic, catches M1-M3 incl. stored-representation change tests miss; kill rate 0.712 to 0.746 on 60 mutants, Q1 gap-halving failed; H8(a) rejected as formulated; random generation found 2 new behaviours; owner decision pending |
| 2026-11-03 | [21: E2 Facdrone edge census](2026-11-03/03-e2-facdrone-edge-census.md) | 2,275 responsibilities, 94.7% reached; C1/C3/C4 held, C2 and C5 failed; edge measures rejected as the E3 selector; recorder validity proven by diff; two Windows defects in specimen test support recorded |
| 2026-11-03 | [20: E1a session-bus trace oracle](2026-11-03/02-e1a-session-bus-oracle.md) | Oracle built, deterministic, catches all mutants; P2 gain prediction hit a 96% seed-test ceiling; Arm B partial; generated inputs found a real panic; owner decision pending |
| 2026-11-03 | [19: Dynamic deconvolution direction](2026-11-03/01-dynamic-deconvolution-direction.md) | Proposal after owner review; H8 registered; E1 oracle feasibility and E2 edge census preregistered; not run |
| 2026-11-02 | [18: Provider representation trial](2026-11-02/18-provider-representation-trial.md) | Concrete replacement rejected for expansion; scoped behavior checks pass; no adopted savings |
| 2026-11-02 | [17: Provider-orchestration admission screen](2026-11-02/17-provider-orchestration-screen.md) | No rewrite admitted; existing shared helpers; constructor alternative deferred, metric limitation explicit |
| 2026-11-02 | [16: Evidence-publication binding trial](2026-11-02/16-evidence-publication-trial.md) | Candidate rejected: more statements, no adopted savings; full lifecycle not validated |
| 2026-11-01 | [Yesterday's seven experiment and decision entries](2026-11-01/README.md) | Retrospectively written 2026-11-02; original evidence linked |
| 2026-11-02 | [01: Prior experiments and hypothesis revisions](2026-11-02/01-retrospective.md) | Retrospective catalog of retained evidence; no new execution |
| 2026-11-02 | [02: CodeQL selection and setup](2026-11-02/02-codeql-direction.md) | Backend selected; CLI verified; extraction pending |
| 2026-11-02 | [03: Requirements and continuity anchor](2026-11-02/03-requirements.md) | Requirements generated for review; no new trial |
| 2026-11-02 | [04: Prototype-first Kiro gate](2026-11-02/04-prototype-first.md) | Real-code feasibility required before production tasks; gate not run |
| 2026-11-02 | [05: Provisional design and setup evidence](2026-11-02/05-provisional-design.md) | Requirements approved; query packs and real-code feasibility pending |
| 2026-11-02 | [06: CodeQL fixture extraction and replay](2026-11-02/06-codeql-extraction.md) | Revise: Python 13/13, TypeScript 12/13; callback omitted; no compression claim |
| 2026-11-02 | [07: Complete call census and state-flow checks](2026-11-02/07-codeql-call-census.md) | G0b: 9/9 checks each; unnamed-target omission fixed; full feasibility remains revise |
| 2026-11-02 | [08: Alias, overwrite and reachable exception boundary](2026-11-02/08-codeql-state-boundary.md) | G0c: runtime controls pass; both queries miss known alias flow; revise completeness assumption |
| 2026-11-02 | [09: Dependency checkpoints and controlled contrasts](2026-11-02/09-codeql-dependency-diagnostic.md) | Direct receiver restores flow but retains overwritten input; graph evidence is not an exact state model |
| 2026-11-02 | [10: Source-backed state model and adverse controls](2026-11-02/10-source-backed-state-model.md) | 16/16 observations per language; five wrong models rejected; real-code compression remains untested |
| 2026-11-02 | [11: Real-case admission screen](2026-11-02/11-real-case-admission-screen.md) | Fifteen files inventoried, thirteen overlapping leads; no qualifying replacement admitted or executed |

Older source, patches, metrics, charts and checks remain under `docs/experiments/`
and linked reports. Their paths and historical conclusions are preserved. New
narrative lab reports belong in `docs/research/YYYY-MM-DD/NN-topic.md`; place new
durable evidence beside the report or link an existing experiment directory.
Ignored `.runs/` output alone is not durable evidence.

## Hypothesis lineage

These IDs are retrospective navigation labels, not original preregistration IDs.
They do not replace trial identifiers in the original records.

| ID | Proposition | Current evidence / revision |
| --- | --- | --- |
| H1 | A shared synthesized kernel removes divergent flow | Works on verbose restricted fixtures; expands the compact control; no general source-equivalence claim |
| H2 | Local metrics and family factoring identify useful removal | Mixed and often marginal; evaluate the whole replacement |
| H3 | SMT feedback and complexity caps drive further compression | A bad proposal was caught; revised outcome classifier had no additional line/LLOC reduction |
| H4 | Shared framework knowledge and connected graphs improve proposals | Pilot benefit unestablished; graph rewrite increased LLOC; retain negative results |
| H5 | Complete capability scope exposes substantial redundancy | ATS/NPU positive manually selected cases; not automatic discovery efficacy |
| H6 | Native measurement plus agent discovery yields useful general compression | Native workflow exercised; local baselines and marginal trials do not establish this broader claim |
| H7 | Query-grounded, multi-pass constraint modeling drives substantial compression | Never ran end to end in 18 entries; static state models rejected on G0c/G0d; superseded by H8 as the equivalence approach |
| H8 | A recorded, mutation-scored trace oracle plus edge measures lets an LLM regenerate scattered responsibilities with fewer edges and equal behaviour | H8(a) oracle mechanics positive on mmhmem (R20) and Facdrone (R22) but the gain predictions failed on both, so H8(a) is rejected as formulated; H8(b) edge selector rejected as formulated (R21); H8(c) regeneration untested and not admitted under the preregistered gates; guided-generation rung failed as run (R23) and, corrected and verified, rejected for DB-entangled predicates while killing 7 of 10 residue targets (R24); H8(c) run as E3 (R25): one regeneration round met the oracle with 16% fewer lines and 29% fewer edges and a checked pure core, but mutation sensitivity missed its bar (0.643) and one oracle-blind difference is real, so H8(c) stands on one scope with a lowered protection claim; H8(c) run on a second scope as E4 (R26) with the recorded regenerator program: accepted at turn 3, 25% fewer functions and 29% fewer edges inside the predicted bands, canonical lines only 6% fewer (T2 failed), sensitivity 0.679 inside its band with the original at 0.414 on its own sample, seven oracle-blind differences on error paths, zero divergence on 74 held-out sequences (26 of them the acceptance seed sequences replayed again, 47 new; 1 generated sequence duplicates an acceptance one), the differential void on a failed solver control; H8(c) stands on two scopes with the line prediction failed on the second; owner decision pending: third scope, adoption of the E4 candidate outside the loop, or prevention |
| H9 | Shared recording, replay, acceptance and turn-input apparatus plus thin scope adapters reduces maintained apparatus across E3 and E4 while preserving observations | Registered in [R27](2026-11-07/01-solution-first-native-slice.md); Attempt 1 (`R30`) failed `U5`/`U7` (`C_app = 1,808`); Attempt 2 descope (`R31`) passed all `U1–U7` (`C_app = 1,193 <= 1,364`, saved 512) and production migration completed (`C_app = 1,203 <= 1,364`, 19/19 tests, 9/9 adverse controls fail closed) |
| H10 | A DSPy-style `Refine`/`BestOfN` probabilistic turn controller with a two-tier programmatic reward (fast static AST/surface/structure assertions before runtime trace replay) recovers weaker or non-deterministic model rollouts without weakening the frozen oracle | Registered in [R33](2026-11-07/07-dspy-probabilistic-turn-refinement-design.md) and designed in [`constraint-guided-compression/design.md`](../../.kiro/specs/constraint-guided-compression/design.md); demonstrated single-turn/3-turn baseline apparatus (`proceed` from R25/R26/R29/R31/R32) separated from multi-rollout recovery (`revise`, gated by bounded spike `G1-R`) |

H7 extends scope-wide reasoning and reintroduces solver research for a concrete
purpose. It does not erase H1-H6 or restore previously dropped infrastructure.
Use the [retrospective](2026-11-02/01-retrospective.md) for evidence links and the
[current Kiro brief](../../.kiro/specs/constraint-guided-compression/brief.md)
for the proposed feature boundary.

## Record each expansion before calling it progress

1. Name the hypothesis, its predecessor and a disproof condition.
2. Freeze scope, source identity, evaluator, dependencies and usefulness target.
3. State which missing capability the proposed tool change addresses.
4. Run a bounded prototype; retain failures, counterexamples and intermediate models.
5. Report whole-replacement results and evidence limits. Distinguish candidate,
   verified candidate and adopted code; do not sum overlapping trials.
6. Link the result to the spec decision it informs. Approval of a backend or a
   license does not automatically approve requirements, design and tasks.

Use the [entry template](ENTRY-TEMPLATE.md). Each applicable spec links back here
and each lab entry links to its spec. Update this index in the same change as the
entry. New interpretations get a dated amendment; do not silently overwrite old
results or relabel an exploratory check as preregistered.

## COBESY handoff

Audience and anchor: the owner and agents continuing the research. Target action:
inspect the latest decision and its evidence before extending the tool. The
assumption to challenge is that more infrastructure, passes or lower complexity
must mean less maintained code. The owner welcomes early disproof; rejected
hypotheses are useful outcomes, not contributor failures.

Composition: outcome first, then hypothesis, method, evidence, limitations and
next decision. Keep findings and proposals separate. The date index supplies
chronology; individual reports lead with the decision. A future agent should be
able to reproduce the result without inheriting the conversation.

Validation: no pooled success rates across unlike scopes, no unsupported proof
claims, no bytes-as-benefit or inferred maintenance-hours savings. Quantitative
charts require actual data. Preserve test and apparatus accounting separately.
