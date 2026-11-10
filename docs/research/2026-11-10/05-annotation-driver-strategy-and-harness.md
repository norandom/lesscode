# R59: Strategy consolidated before any run: only horizontal arms remain, one engine with a data pack per language, and the Rust scope is replaced because it had 5 mutable sites

- Report date/time and timezone: 2026-11-10, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-10, about 11:05 to 11:30 local for the counts and baselines reported here; engine and harness work follows and is appended below when done
- Status: completed (setup: decisions, engine, harness and model-free checks; no arm batch and no model call)
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

## Addition, 2026-11-10 (about 11:35 to 12:15 local): engine, harness, checks and freeze

The engine, harness and their checks were written by a delegated Opus agent
at the highest effort setting from a written specification, then reviewed and
rerun by the session agent, which also made the changes listed under
"Decisions after review". No arm batch was run and no model was called.

Model-free checks, [receipt](../../experiments/annotation-driver/results/engine-checks-receipt.json), all passing:

| Check | Result |
| --- | --- |
| Pack node kinds valid for each grammar, packs at most 40 lines | 4 of 4 (22 to 25 lines) |
| Positions are character offsets; CRLF and non-ASCII text handled | 4 of 4 |
| Every control seed with an operator yields a real change confined to its statement | 12 of 12 seeds; also all 321 mutable sites, 0 outside their span |
| Site counts reproduce R59 with its method, and match the engine's normalised counts | 4 of 4 |
| Selection deterministic and kind-balanced | 4 of 4 |
| Harness smoke per scope: identical file invalid, unparseable file invalid, one real change classified, baselines pass before and after, file restored byte for byte | 4 of 4; the real change was killed in every scope |

Validator controls after the changes: 76 of 76
([receipt](../../experiments/annotation-driver/results/controls-receipt.json)).

Sites that can yield a mutant with these packs:

| Scope | Eligible | With operator | By kind | Without operator |
| --- | --- | --- | --- | --- |
| Rust | 33 | 16 | branch 11, failure 2, order 2, write 1 | 17 failure sites (16 `return`, 1 `Err(`) |
| Python | 200 | 161 | write 110, branch 40, failure 6, order 5 | 38 `return`, 1 chained assignment |
| TypeScript | 50 | 29 | branch 16, failure 9, write 3, and 1 see note | 20 `return`, 2 writes that are not statements |
| Java | 157 | 115 | branch 51, write 50, order 11, failure 3 | 40 `return`, 2 writes in `for` headers |

Note: the engine's Rust count is 33, not 38, because five `if let`
conditions cannot be negated and are excluded by the pack. The TypeScript
count is 50 after excluding one empty `for (;;)` condition.

Decisions after review (gate Amendment 4):

- Python tests run serially so that a failing test and a broken import have
  different exit codes. Found by the implementing agent; without it every
  kill in the Python scope would have been recorded as invalid.
- Arm D draws its 40 from sites that have an operator.
- The validator accepted a lone bracket as an anchor when it was unique on
  its line. It now accepts named syntax nodes only; a control covers it.
- The TypeScript pack excludes the empty `for` condition. That is an edit to
  a held-out language's pack made after seeing the held-out scope, and is
  recorded as a contamination of D1 for TypeScript.

Failed attempt, kept on record: the first run of the checks could not start
the Java scope's tests. The Docker API pipe of the Podman machine was missing
after an earlier machine restart in this session; restarting the machine
restored it and the Java smoke test then passed. No scope was modified.

Frozen inputs for the D1 batches (LF line endings):

| File | Lines | SHA-256 |
| --- | --- | --- |
| `validator.py` | 237 | `afa504eaf246240c7c1ed2b5920c8162c410181317c01acfe9e47829c7c3a046` |
| `engine.py` | 195 | `c584e0c119be4766a8bd02341c0bd6f7675de8babd7c334d816a65c0dc862bbd` |
| `harness.py` | 202 | `125e3d10b318264488859b61d9ae03f60125afa4f680c154ec345c6c854cd593` |
| `run_arm.py` | 126 | `0fb08f528aeb9b8e39caf544c067ba2c30c9e8bb4fd7f9779fcb62bc489fb28e` |
| `run_controls.py` | 139 | `4238c2c33aefa98fedb5533e22c7e3f9a62de6b76e1793f945575fda8bbbb38b` |
| `run_engine_checks.py` | 322 | `de6310f4ef1cc80d3ac0a85f6ab58e5cf397af5e78db0a4f335f0d12f3e70dbe` |
| `controls.json` | 53 | `99966c71510fb6ac0dd71d84f348e03dc9e5e5eb0f2e98e1ce54859fd54fc14b` |
| `scopes.json` | 39 | `229ff3fa7be807678cdaffaadbf4a0669c8c544d36e9fd05c010472d80072260` |
| `tables/rust.json` | 25 | `8be82132121987419773defe0e51511ea7e50028265da187da5f75a37e09b787` |
| `tables/python.json` | 22 | `7d30f778994b05517416b108f63316a72f1d4801b43fa47ddc61d540757732c5` |
| `tables/typescript.json` | 22 | `22ce24939e92aaf9f541723b43fd6701eaf9494408cdb320db1c8fe5acf84808` |
| `tables/java.json` | 22 | `9a7f65c126ba84c0a6c447913539bfc75035ceddc74122a15d4764b30d55746f` |

These supersede the hashes in R58 for the files that changed. The engine
(`engine.py`, 195 lines) and the harness contain no language or repository
name; the four packs total 91 lines.

Limits of this addition: the smoke test ran one real change per scope, which
shows the harness classifies, not how the operators perform. Rust has only
16 mutable sites, so its D1 batch will be 16 mutants, not 40.

## Addition 2, 2026-11-10 (about 12:10 local): freeze before the first model call

Guide version 0 and the model-call runner are frozen here, before any model
call. The runner was written by a delegated Opus agent from a written
specification and tested only against a local fake endpoint (17 tests); it
has made no real call.

| File | Lines | SHA-256 |
| --- | --- | --- |
| `guide-v0.md` | 51 | `4f011694ed11da621b4f6d9ff10c4f227d7aa562cf212ca79caf44bb74ee3ae1` |
| `annotate.py` | 272 | `b4a9d5d95aec50b0c9fe2bb0b7d073c1c32cf715b3f1e5d8b49fbe60c8609133` |
| `test_annotate.py` | 393 | `91b440bb66dc06beba4d221d550be4f28984ac7c365fa1ce41c48fe52a0e1b6a` |
| `scopes.json` (Java limit, Amendment 5) | 39 | `b63e3a5e454b11425526d76d9f6e0b8b93293a57850cc2f902b0289a6b89b113` |
| `compose.yaml` (Amendment 5) | 41 | `3d1e1bfeab3d574484df6d34c962acd7052986a7eca8bdae0bf4a900f0b909c6` |

Request shape: the guide is the system message; the user message is the
scope file with line numbers under a fixed one-sentence header; no sampling
parameters and no extended thinking are requested; 16,000 output tokens at
most; at most two attempts per call. Endpoint: Anthropic style,
`claude-opus-5-5`, from the owner's `model.env`.

Plan for the first calls, fixed now: one development call each on the Rust
and Python scopes with guide version 0. These are not counted runs. If
either scope has fewer than 80% valid annotations, the guide is revised
using those two scopes only and frozen again as version 1 under a new hash;
otherwise version 0 is the frozen guide. The counted runs (arm A twice per
scope on all four scopes, D2 for the two held-out languages) follow. Every
call, including development calls, counts toward the 20 USD cap and is
recorded in the ledger.

## COBESY check

Outcome first: decisions, no mechanism result. Replay: counts, revisions,
hash and commands given. Disproof: remaining rejection conditions listed, and
the loss of one way to be wrong is stated plainly. Negative observations
that weaken the hypothesis (returns without an operator, trivially broken
imports) are recorded before any run. Facts are in the tables; the choices
are marked as choices.
