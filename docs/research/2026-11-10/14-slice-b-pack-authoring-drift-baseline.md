# R68: Slice B baseline, Opus arm: every pack is admitted on the first reply, but Rust packs differ from run to run and admission does not notice

- Report date/time and timezone: 2026-11-10, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-10, 20:32 to 20:34 local (104 seconds wall time, four conversations at a time)
  (Corrected 2026-11-10 from file and commit times; first written as "2026-11-10, about 21:35 to 21:37 local (104 seconds wall time, four conversations at a time)", which was an estimate.)
- Status: completed for the current endpoint; the second model arm is not run
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md), Slice B
- Hypothesis ID and predecessor: runs the protocol preregistered in
  [R66](12-slice-b-pack-authoring-drift-preregistration.md) (with Amendments 1 and 2)
- Type: confirmatory baseline

## Decision

Pack authoring does not need prompt optimisation for admission: 20 of 20 first replies were admitted. It does
need stronger admission. For Rust the model wrote four behaviourally different packs in five runs, none equal
to the shipped pack, and every one passed. By the rule fixed in R66 this is first a finding about the checks,
to be fixed there before any `MIPROv2` work. Python also failed the "same mutants" claim, but for a different
reason: the five model packs agree with each other and the hand-written reference is the one that is
incomplete.

Separate outcomes: this measures stability of pack authoring on one endpoint. It says nothing about the
annotation layer, about drift over time, or about another model. No compression was measured.

## Hypothesis and falsification

As preregistered, per language on this endpoint:

| Claim | Rust | Python | TypeScript | Java |
| --- | --- | --- | --- | --- |
| H1: at least 4 of 5 first replies admitted | 5 of 5 | 5 of 5 | 5 of 5 | 5 of 5 |
| H2: at least 4 of 5 admitted packs give the shipped pack's mutants | 0 of 5, refuted | 0 of 5, refuted | 5 of 5 | 5 of 5 |
| H3: rejected packs admitted after at most two repairs | not exercised | not exercised | not exercised | not exercised |

No reply was refused, cut off or empty. No repair turn was needed, so the repair path is still untested.

## Frozen inputs and method

As in R66 Amendment 2: `pack-guide-v3.md` (SHA-256 `dc49e14bb0d62d1560724b828e54e1c7d216ca5bd47cb37580b618b4c7d02bbb`), runner
`docs/experiments/annotation-driver/slice-b/author_baseline.py` at commit `e6cdc6b`, model `claude-opus-5-5`
(requested and returned), examples from `native/packs/`, admission by `lesscode-core deconvolute admit-pack`
with control replay on every gate scope of the language, mutants by `native/examples/pack_mutants.rs`, both
built from commit `e6cdc6b`. Results: `docs/experiments/annotation-driver/results/slice-b/opus/`
(`summary.json`, per run the request, response, reply, pack, admission receipts and `run.json`).

## Evidence and results

| Language | Distinct pack texts | Groups of runs with identical mutants | Mutants, shipped pack | Mutants, model packs |
| --- | --- | --- | --- | --- |
| Java | 3 | 1 (all five) | 170 | 170 |
| TypeScript, two scopes | 3 | 1 (all five) | 71 and 55 | 71 and 55 |
| Python | 2 | 1 (all five) | 196 | 197 |
| Rust | 5 | 4 | 64 | 62, 62, 62, 70, 70 |

Java and TypeScript: the wording of the packs varied, the behaviour did not.

Python: all five packs add the pairs `in`/`not in` and `is`/`is not` to the token swaps and to the negation
pairs, which yields one more mutant on the gate scope (`is` to `is not` at line 85). The shipped pack lists
only `==`/`!=`. The model packs also leave `unwrap_kinds` empty where the shipped pack names the parenthesis
wrapper; on this scope that made no difference. One run added `assert` as a failure signal.

Rust, where the runs disagree:

- None of the five packs has the rewrite of the `?` operator, so two mutants of the shipped pack are missing
  in every run.
- None recognises a call beginning with `Err(` as a failure signal.
- Two runs list every macro invocation as a failure signal and replace it, which adds eight mutants at
  macros; the two runs use different replacements (`()` and the empty statement).
- Two runs use `{}` instead of `;` as the statement that does nothing, which changes the text of 39 mutants.
- Four runs treat a `match` guard as a condition, which the shipped pack does not.
- All five name only the outer attribute kind for test regions; the shipped pack also names the inner one.

Cost: 68,065 input and 38,605 output tokens in 20 calls. At the list price for this model (4 and 20 USD per
million tokens, from the vendor's model table cached 2026-10-06) that is about 1.04 USD. Under the
conservative price assumed for the cap in R66 (15 and 75 USD) it is 3.91 USD, and recorded spend under that
assumption is now 16.01 of 20 USD.

## Limits and interpretation

- H2 measures agreement with a hand-written pack. The Python result shows that this reference is not ground
  truth: there the model was consistent and more complete. "Same as shipped" is the wrong yardstick on its
  own; agreement between runs has to be reported beside it, as the table above does.
- Five runs per language; one endpoint; one day. Rates are coarse.
- The Rust differences were found on one file of 311 lines. Whether the extra macro mutants are useful or
  noise was not judged; no tests were run against any of these packs.
- Admission passed all 20 packs. Its controls are three seed annotations per scope and the cases derived
  from them. They do not contain a `?` site, a macro that is not a failure, an `Err(` call or an inner
  attribute, so they cannot tell these packs apart.
- The examples shown to the model for Rust were the Python and Java packs, which have no test markers and no
  prefix rules; the guide describes those fields in one row each.
- The second arm of R66 (another model) is not run. The owner decided on 2026-11-10 that the API runs stay
  on this model; a Mistral arm waits for its endpoint configuration.

## Tool/spec consequence and next gate

By the preregistered rule ("H2 fails while H1 holds: the admission checks are too weak; fixed there before
any prompt optimisation"):

1. Extend the control sets per language so that admission distinguishes the behaviours found here: positive
   controls for each failure form and each condition form the shipped pack covers, and negative controls
   (a macro that does not signal failure, text inside an inner test region). This needs the owner's review
   of what the intended behaviour is, since the Python result shows the hand packs are not always right.
2. Decide the Python question on its merits: adopt the model's two extra operator pairs in the shipped pack
   or reject them, with a reason.
3. Rerun this baseline unchanged after 1 and 2. Only if packs that pass the stronger admission still differ
   between runs does bounded prompt optimisation (`MIPROv2`) of the guide become the next step.
4. The annotation layer (run-to-run agreement 0.60 to 0.94 at the gate) remains the larger known instability
   and is untouched.

## COBESY check

Outcome first: admission is stable, behaviour is not for one language, and the checks miss it. Replay:
guide, runner, binaries and results are in the repository at named commits. Disproof: the preregistered
thresholds are applied as written, including to the claim that failed for a reason other than drift, which is
reported as a flaw of the yardstick and not explained away. Cost is given at the list price and under the
earlier assumption.
