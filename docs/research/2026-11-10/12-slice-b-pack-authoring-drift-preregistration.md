# R66: Preregistration of the Slice B drift baseline for pack authoring (not run)

- Report date/time and timezone: 2026-11-10, Europe/Berlin (UTC+1)
- Execution interval: none. This entry fixes the protocol before any call is made.
- Status: preregistered, not run. Running it needs the owner's go.
- Spec: [constraint-guided-compression](../../../.kiro/specs/constraint-guided-compression/requirements.md)
  (Amendment 5, Slice B); written as task 2.4 of
  [agnostic-annotation-driver](../../../.kiro/specs/agnostic-annotation-driver/tasks.md) (Requirement 9.6)
- Hypothesis ID and predecessor: first Slice B step; follows [R64](10-annotation-driver-gate-attempt2.md)
  and [R65](11-annotation-driver-rust-sizing-spike.md)
- Type: confirmatory baseline (measures drift; does not optimise anything)

## Decision

Nothing is decided by this entry except the protocol. It exists so that Slice B, the owner's second priority,
has a concrete, dated first step that cannot be reinterpreted after the numbers are seen.

Correction to an earlier statement of mine: the design and two reports to the owner said this baseline "needs
the driver's first task group only". That is true for the admission rate, which `deconvolute admit-pack`
(task group 2, done) measures. The second measure, identity of mutants, needs the native engine of task
group 3. The baseline therefore starts after group 3.

## Hypothesis and falsification

The gate showed that a model wrote the TypeScript and Java packs once each, that both were admitted on the
first reply and that both gave the same mutants as the hand-written packs (R64). One run per language says
nothing about drift.

Claims to test, per endpoint:

- H1 (admission): with the frozen guide, a pack is admitted on the first reply in at least 4 of 5 runs for
  each of the four languages.
- H2 (same result): in at least 4 of 5 runs per language, the admitted pack yields exactly the mutants of
  the shipped pack on that language's gate scopes (same site, same operator, same hash of the mutated text).
- H3 (repair): every pack not admitted on the first reply is admitted after at most two repair replies that
  are given only the admission feedback.

Refuted when, for any language on an endpoint, fewer than 4 of 5 first replies are admitted (H1), fewer than
4 of 5 admitted packs reproduce the shipped mutants (H2), or a pack is still rejected after two repairs (H3).
A refusal by the endpoint is recorded as its own outcome and counts against H1; the request is not reworded.

What each outcome leads to:

- All three hold on both endpoints: pack authoring needs no optimisation. Slice B moves to the annotation
  layer, whose measured baseline is weaker (two identical runs agree on 0.60 to 0.94 of their sites, R64).
- H1 or H3 fails: bounded optimisation of the pack guide (`MIPROv2`, `GEPA` only if that stalls) with the
  admission result as the metric, on languages held apart from the ones used to judge it.
- H2 fails while H1 holds: the admission checks are too weak, since they let through packs that behave
  differently. That is a finding about the checks first, and is fixed there before any prompt optimisation.

## Frozen inputs and method

- Runs: 5 per language for Rust, Python, TypeScript and Java on each endpoint, 20 authoring conversations
  per endpoint. Each run is a fresh conversation.
- Endpoints: the current one (`claude-opus-5-5`, configuration in the owner's `model.env`) first; the Mistral
  endpoint second, once the owner supplies its configuration. Temperature and other sampling settings are
  the endpoint defaults and are recorded per call.
- Guide: `pack-guide-v2.md` (SHA-256 `46ea8bcdd6286fd9e928c41960a7d1da1c8d4eb6bccbe98294e27ac867269949`) with one sentence added for the pack key
  `test_attribute_kinds`, which the native pack format introduced after the gate. The amended guide is saved
  as `pack-guide-v3.md` and its hash recorded here by dated amendment before the first call. No other change.
- Examples shown to the model: two shipped packs of other languages. For TypeScript and Java: Rust and
  Python, as in the gate. For Rust: Python and Java. For Python: Rust and Java. The target language's own
  pack is never shown.
- Admission: `lesscode-core deconvolute admit-pack` at the commit recorded with the run, with control replay
  against the gate's controls once that is available (task 3.2). No Python admission code is used.
- Mutants: the native engine on the gate scopes at their frozen commits, compared with the shipped pack's
  mutants by site, operator and hash.
- Repair: at most two repair replies per run, each given the admission feedback only.
- The runner that makes the calls is experimental tooling outside the binary; the product still contains no
  model call.

Measures per language and endpoint: first-reply admission (n of 5), failed check names, repair replies
needed, admitted packs reproducing the shipped mutants (n of 5), number of distinct admitted pack texts,
refusals, tokens and cost.

## Cost bound

The two authoring calls of the gate used about 3,150 input and 1,500 to 1,800 output tokens each. At the
price assumed so far in this project (15 and 75 USD per million tokens, not verified against a price list)
that is about 0.18 USD per call: about 3.60 USD for the 20 first replies of the current endpoint and at most
about 10.80 USD if every run needed both repairs. Spend recorded so far is at most about 12.10 USD of the
20 USD cap, so the full worst case does not fit. Rule: the run stops before any call that would pass the cap,
and what was not run is reported as not run. The Mistral arm needs its own cap from the owner.

## Evidence and results

None. Nothing was run.

## Limits and interpretation

- Five runs per language give coarse rates: 4 of 5 and 5 of 5 cannot be told apart from a true rate of 0.8
  or 0.95. The baseline detects gross instability, not small drift.
- "Same mutants on the gate scopes" is tested on five files. A pack can agree there and differ elsewhere.
- The Rust and Python runs use different examples from the gate's TypeScript and Java runs, so they are not
  replications of the gate.
- Drift over time (the same request weeks apart) is not measured; the two endpoints stand in for a model
  change.
- The annotation layer, where the gate measured the larger instability, is deliberately second.

## Tool/spec consequence and next gate

- Start condition: task group 3 of the driver complete, the guide amendment hashed here, and the owner's go.
- Results go into a new dated entry that links back to this one; any change to this protocol before the run
  is a dated amendment below.

## Amendment 1, 2026-11-10 (before any call; after task group 3 of the driver, R67)

- The engine-dependent start condition is met: the native engine reproduces the prototype's mutants.
- The pack format gained a second key after the gate, `negation_swaps` (token pairs whose swap repeats the
  negation of a condition). `pack-guide-v3.md` must therefore add one sentence each for `test_attribute_kinds`
  and `negation_swaps`. A pack without `negation_swaps` yields more mutants than the shipped pack, which
  would fail H2 for a reason that is not drift.
- H2 is made precise: "the mutants of the shipped pack" are those of the shipped pack as embedded in the
  binary at the commit recorded with the run.
- Still open before the first call: the hash of `pack-guide-v3.md` in this entry, and the owner's go.

## Amendment 2, 2026-11-10 (before any call; the owner gave the go: "absolutly")

- Guide: `docs/experiments/annotation-driver/pack-guide-v3.md`, SHA-256
  `dc49e14bb0d62d1560724b828e54e1c7d216ca5bd47cb37580b618b4c7d02bbb`. It is version 2 with the title changed
  and two table rows added, for `test_attribute_kinds` and `operators.branch_condition.negation_swaps`.
- Runner: `docs/experiments/annotation-driver/slice-b/author_baseline.py`. Examples shown are the shipped
  packs under `native/packs/` (which now carry the two new keys), per the mapping fixed above. Admission is
  `deconvolute admit-pack` with control replay on every gate scope of the language (two for TypeScript); a
  pack counts as admitted only if it is admitted on all of them. Mutants are listed by the research example
  `native/examples/pack_mutants.rs`.
- Stop rule made operational: 0.25 USD is held back before each call; no call is made when recorded spend
  plus held-back amounts would pass 20 USD at the assumed price. Runs that cannot start are reported as not
  run.
- Scope of this run: the current endpoint only. The Mistral arm waits for the owner's endpoint configuration
  and its own cap.
- A reply cut off at the output limit counts as not admitted and may be repaired.

## Amendment 3, 2026-11-10 (after the first run R68, before the rerun; owner: "approved")

The owner approved the two consequences proposed in R68. Changes before the rerun, nothing else:

- Shipped Python pack: the operator pairs `in`/`not in` and `is`/`is not` are adopted (token swaps, negation
  pairs, and the yes-or-no tokens of returned expressions). The five Python packs of R68 then give exactly
  the shipped pack's mutants on the gate scope (checked for runs 1 and 3: 197 and 197, identical).
- Rust admission controls: a hand-written control set, `native/tests/fixtures/driver/controls/rust/`, with
  five cases on a small sample file: the `?` operator is a failure site and offers the rewrite; an `Err(`
  call is a failure site; a macro that builds a value is not; an aborting macro is; code under an inner test
  attribute is a test region. Control cases may now name the operators a site must offer. All five Rust
  packs of R68 are rejected by this set; the shipped pack is admitted.
- The runner adds every hand-written control set of the language to admission. Guide, examples mapping,
  thresholds, repair limit and model are unchanged. Results go to `results/slice-b/opus-rerun1/`.
- Not controlled, on purpose: `{}` against `;` as the statement that does nothing (both are no-ops, so the
  hash comparison overstates that difference), and `match` guards as conditions (an addition by the model
  that may be right; undecided).
- Cap accounting now uses the vendor's list price for the model (4 and 20 USD per million tokens for
  `claude-opus-5-5`); a model without a listed price keeps the conservative assumption. Recorded spend is
  4.27 USD under list prices. The cap stays 20 USD.
- Owner direction, 2026-11-10: "keep in mind that we also hardened against inference drift, mipro makes
  sense". The rule "prompt optimisation only if packs still differ" is therefore dropped. After the rerun,
  bounded `MIPROv2` optimisation of the pack guide is the next Slice B step in either case, with admission
  and run-to-run agreement as the metric; it gets its own preregistration.

## COBESY check

Outcome first: a protocol, nothing run, and one correction of an earlier claim about what the baseline needs.
Replay: inputs, counts, thresholds and stop rule are fixed. Disproof: three refuting conditions with the
action each leads to. Costs are stated with the unverified price assumption named.
