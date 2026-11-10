# R69: Slice B baseline rerun: stronger admission stops every first Rust pack, repair succeeds twice in five, and Java now differs too

- Report date/time and timezone: 2026-11-10, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-10, 20:44 to 20:47 local (153 seconds wall time, four conversations at a time)
  (Corrected 2026-11-10 from file and commit times; first written as "2026-11-10, about 22:05 to 22:08 local (153 seconds wall time, four conversations at a time)", which was an estimate.)
- Status: completed for the current endpoint
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md), Slice B
- Hypothesis ID and predecessor: rerun of [R68](14-slice-b-pack-authoring-drift-baseline.md) under
  [R66](12-slice-b-pack-authoring-drift-preregistration.md) Amendment 3
- Type: confirmatory baseline

## Decision

Deterministic checks alone do not harden pack authoring. With the stronger Rust controls no Rust pack was
admitted on the first reply, the repair turns fixed two of five, two stayed rejected and one repair request
was refused by the endpoint. Java, stable in the first run, gave packs that differ from the shipped pack in
three of five runs. This supports the owner's direction of 2026-11-10: bounded optimisation of the guide
(`MIPROv2`) is the next step, beside the checks and not instead of them.

One defect of the tooling contributed and is fixed: the repair feedback for a missing operator said only
"operators_differ" without naming the control or the expected operator, so the model could not know what to
change. The two Rust runs that stayed rejected failed on exactly that check.

Separate outcomes: stability of pack authoring on one endpoint, on one evening. No tests of any project were
run against these packs; no compression was measured.

## Hypothesis and falsification

As preregistered in R66, thresholds unchanged:

| Claim | Rust | Python | TypeScript | Java |
| --- | --- | --- | --- | --- |
| H1: at least 4 of 5 first replies admitted | 0 of 5, refuted | 5 of 5 | 5 of 5 | 5 of 5 |
| H2: at least 4 of 5 admitted packs give the shipped pack's mutants | 0 of 2, refuted | 5 of 5 | 5 of 5 | 2 of 5, refuted |
| H3: rejected packs admitted after at most two repairs | 2 of 5, refuted | not exercised | not exercised | not exercised |

## Frozen inputs and method

R66 with Amendments 1 to 3: guide `pack-guide-v3.md` unchanged, model `claude-opus-5-5`, runner and binaries
at commit `66f1d4d`, the hand-written Rust control set added to admission, the Python pack with two more
operator pairs. Results: `docs/experiments/annotation-driver/results/slice-b/opus-rerun1/`.

One input changed for a language other than the one it was meant for: the Python pack is one of the two
examples shown when the Java and TypeScript packs are requested, so those requests differ from the first run
by the two added operator pairs in an example.

## Evidence and results

| Language | First reply admitted | Admitted in the end | Repair replies | Same mutants as shipped | Distinct admitted packs |
| --- | --- | --- | --- | --- | --- |
| Java | 5 of 5 | 5 | 0 | 2 of 5 | 5 |
| Python | 5 of 5 | 5 | 0 | 5 of 5 | 2 |
| TypeScript | 5 of 5 | 5 | 0 | 5 of 5 | 4 |
| Rust | 0 of 5 | 2 | 10 | 0 of 2 | 2 |

Rust:

- Every first reply failed the controls for the `Err(` call and the inner test attribute; two also treated
  a value-building macro as a failure signal. These are the behaviours R68 found, now caught.
- After two repair turns, two runs were admitted. Both remove the `?` instead of replacing it by an unwrap
  (their operator has the expected label, so the control passes, but the resulting text is unlikely to
  compile), keep `match` guards as conditions and drop aborting macros. Their mutants differ from the
  shipped pack's by two on the gate scope.
- Two runs were still rejected after two repairs, both only on the missing `?` rewrite, with the
  uninformative feedback described above.
- One run's second repair request was refused (`stop_reason: refusal`, category `cyber`). As preregistered
  it was not reworded. The first two requests of the same conversation were answered.

Java: three packs have no operator for `throw` statements, which removes three mutants on the gate scope; two
handle `instanceof` through the node kind instead of the token, with the same mutants as the shipped pack.
In the first run all five Java packs had the `throw` operator.

Cost: 115,798 input and 57,948 output tokens in 30 calls, 1.62 USD at the list price. Recorded spend is
5.89 of 20 USD at list prices.

## Limits and interpretation

- The Java change between the two runs cannot be attributed to sampling alone, because an example pack in
  the request changed. It still shows that an admitted Java pack can lack an operator the shipped pack has,
  and that no control notices.
- Five runs per language, one endpoint, one evening.
- The controls check that a site offers an operator with a given label. They do not check what the operator
  writes, so a rewrite that cannot compile passes. Whether a mutant compiles is only known when a project's
  build runs, which is the batch runner's job.
- The hand-written controls encode the shipped packs' behaviour as intended behaviour. The Python result of
  R68 showed that the hand packs can be the incomplete side; the same may hold for `match` guards.
- The refusal happened on a repair turn whose content is admission feedback about failure sites and test
  regions. One occurrence; no rate can be given.

## Tool/spec consequence and next gate

1. Fixed in the same change: the `seed_mutants` detail now names the failing control, the node kind and the
   expected and found operators. This changes the repair feedback, so later runs are not comparable with
   this one on H3.
2. Next Slice B step, by the owner's direction: a preregistered, bounded `MIPROv2` optimisation of the pack
   guide. Metric: first-reply admission and run-to-run agreement of mutants, measured on languages held
   apart from those used for optimisation; budget and stop rule fixed beforehand; the refusal behaviour
   counted as an outcome.
3. Controls still missing: an operator for `throw` in Java; what a `?` rewrite must produce.
4. The annotation layer remains untouched.

## COBESY check

Outcome first: three claims refuted where they were tested, with the numbers. Replay: guide, runner, binaries
and results at named commits. Disproof: thresholds applied as preregistered. A change of input that weakens
one comparison and a tooling defect that contributed to one refuted claim are both stated.
