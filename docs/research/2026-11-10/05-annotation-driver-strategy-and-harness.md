# R59: Strategy consolidated before any run: only horizontal arms remain, one engine with a data pack per language, and the Rust scope is replaced because it had 5 mutable sites

- Report date/time and timezone: 2026-11-10, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-10, about 11:05 to 11:30 local for the counts and baselines reported here; engine and harness work follows and is appended below when done
- Status: running (decisions recorded; engine and harness not yet built)
- Spec: [.kiro/specs/agnostic-annotation-driver/prototype-gate.md](../../../.kiro/specs/agnostic-annotation-driver/prototype-gate.md), Amendment 3
- Hypothesis ID and predecessor: H-AD1, [R58](04-annotation-driver-validator-controls.md)
- Type: preregistered (pre-run amendment; no hypothesis test in this entry)

## Decision

The owner clarified that "hard-coded arms" means non-agnostic runs that do
not apply horizontally, and asked the agent to settle the strategy directly.
Gate Amendment 3 does that before any mutation run or model call:

1. Arms B (fixed operators) and C (a different existing tool per language)
   are removed. Neither applies horizontally. The outcome "stop in favour of
   existing tools" is deleted with arm C.
2. One engine and one set of generic operators serve every language. All
   language knowledge is a data pack: the node-kind table plus a few operator
   templates. The remaining arms differ only in how sites are chosen: every
   eligible site (D1 with hand-written packs, D2 with model-written packs for
   the held-out languages) or the sites a model annotates (A).
3. A scope needs at least 12 eligible sites in at least 3 kinds. The Rust
   scope admitted in R57 has 5 sites in 2 kinds and is replaced.
4. Arm D is capped at 40 mutants per scope by a seeded, kind-balanced
   selection. Classification rules are fixed (invalid, killed, survived,
   timeout).
5. Strategy is decided by the session agent; clearly specified
   implementation is delegated to Opus at the highest effort setting.

This permits building the engine and harness and running D1. No result about
the mechanism exists yet.

## Hypothesis and falsification

H-AD1 is unchanged. What changes is what it is compared with. With B and C
gone, the gate no longer asks "is this better than what exists" but "does a
horizontal mechanism meet absolute targets, and does per-scope model
annotation add anything over applying generic rules everywhere". The second
question is now sharply testable, because A and D1 share engine, operators
and packs and differ only in site selection.

Removing arm C removes a way for the hypothesis to lose. That is a real cost
of this amendment and is stated here: the gate can no longer conclude that an
off-the-shelf mutation tool would have been enough for a single language. The
owner's first priority makes that conclusion irrelevant to the decision, but
it is not disproved.

Rejection conditions that remain: the validity, yield, oracle,
meaningfulness, agnosticism and apparatus rows, on at least three of four
languages including one held-out language.

## Frozen inputs and method

Eligible sites were counted with the validator's own eligibility rules and
the tables frozen in R58, excluding comments, strings and test regions.

| Scope | Eligible sites | By kind |
| --- | --- | --- |
| Rust `mmhmem` `src/setup/check.rs` (R57) | 5 | branch 4, write 1 |
| Rust `mmhmem` `src/setup/ui/update.rs` (replacement) | 38 | failure 19, branch 16, order 2, write 1 |
| Python `mvibe` `mvibe_proxy/ratelimit.py` | 200 | write 111, failure 44, branch 40, order 5 |
| TypeScript `OpenUltraCode` `src/state.ts` | 51 | failure 29, branch 17, write 5 |
| Java `commons-csv` `Lexer.java` | 157 | write 52, branch 51, failure 43, order 11 |

Other Rust files screened at the same revision, all with passing tests:
`setup/adopt.rs` 2 sites, `services/stats.rs` 6, `services/worker.rs` 13,
`services/warmup.rs` 20. The replacement is the one with the most sites.

Replacement scope: `mmhmem` `f32596f8c3117aa05a0d4db924a4de18d80f42d1`,
`src/setup/ui/update.rs`, SHA-256
`804522e68de5d5059369418ab8190baf718eaed06c821ac007ab03661878381e`, 229
canonical lines. Baseline in the clone:
`cargo nextest run --locked --lib --test setup_update_spike -E 'test(setup::ui::update) | binary(setup_update_spike)'`,
17 tests passed.

The controls of R58 were rerun with the new Rust scope and three new
hand-made seeds for it: 72 of 72 behaved as expected.

## Evidence and results

Setup facts only, as tabulated above. Two observations bear on later rows:

- Many failure sites are `return` statements, for which version 0 has no
  generic operator. The number of sites that can actually yield a mutant is
  lower than the eligible count, especially in TypeScript and Java.
- In the Python scope 111 of 200 sites are assignments, many of them
  module-level constants. Dropping one breaks the import, which Amendment 3e
  classifies as invalid, not killed. Without that rule arm D would look
  better than it is.

## Limits and interpretation

- Choosing the Rust replacement by site count favours scopes that are easy
  for a syntactic mechanism. It was done before any run and by a stated rule,
  but it is still a selection, and the result will not show how the
  mechanism does on Rust code with little branching.
- The site counts depend on the hand-written tables. A different table would
  count differently; D2 measures how far a model-written table departs.
- The operators are deliberately few. A `failure_state` on a `return`, and a
  `state_write` that is not a statement of its own, produce nothing in
  version 0.

## Tool/spec consequence and next gate

- Gate Amendment 3 governs. `scopes.json` now carries the admission
  thresholds and the build and test command for each scope.
- Next: engine, operators, packs and harness with model-free checks; then
  D1 on all four scopes. The inputs are re-frozen by hash after that work,
  in an addition to this entry, before guide version 0 and the first model
  call.

## COBESY check

Outcome first: decisions, no mechanism result. Replay: counts, revisions,
hash and commands given. Disproof: remaining rejection conditions listed, and
the loss of one way to be wrong is stated plainly. Negative observations
that weaken the hypothesis (returns without an operator, trivially broken
imports) are recorded before any run. Facts are in the tables; the choices
are marked as choices.
