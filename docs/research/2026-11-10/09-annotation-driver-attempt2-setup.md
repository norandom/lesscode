# R63: Attempt 2 setup: pack format version 1, a second TypeScript scope and frozen inputs; no counted run yet

- Report date/time and timezone: 2026-11-10, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-10, about 15:40 to 17:00 local (implementation and model-free checks)
- Status: completed for setup; development calls and counted runs follow in a later entry
- Spec: [.kiro/specs/agnostic-annotation-driver/prototype-gate.md](../../../.kiro/specs/agnostic-annotation-driver/prototype-gate.md), Amendments 10 and 11
- Hypothesis ID and predecessor: H-AD1, attempt 2 of 2, after [R61](07-annotation-driver-gate-result.md) and [R62](08-annotation-driver-parallel-batches.md)
- Type: preregistered (inputs and rules frozen here before any attempt 2 model call or batch)

## Decision

Setup only. The engine, validator and packs implement gate Amendment 10a,
all model-free checks pass, and the second TypeScript scope is admitted by
the stated rule. This permits the development calls on Rust and Python and,
after the guides are frozen, the counted runs. It is no evidence about the
mechanism.

## Hypothesis and falsification

Unchanged: H-AD1 with the targets and `proceed` rule of the gate. What
attempt 2 changes is recorded in Amendment 10. Two predictions specific to
the revision can fail: that TypeScript now lands mutants in at least three
kinds on both of its scopes, and that the frozen guide reaches 80% valid
annotations there.

## Frozen inputs and method

Implementation by a delegated Opus agent at the highest effort setting from
a written specification; reviewed, extended for the fifth scope and rerun by
the session agent.

| File | Lines | SHA-256 |
| --- | --- | --- |
| `validator.py` | 290 | `05422d6e42352d4ace3bd33830ac9a0933c4172c653979a1a3564a28a73645df` |
| `engine.py` | 273 | `31052d0df9af356b01014560ef1d68e98a093ae5b78b30ee471512650e70c73e` |
| `harness.py` | 339 | `92429b593de0905f54633c963f4a160f49fbf1bc7aee43220bb8214676bd4694` |
| `pool.py` | 265 | `55f687ce2aa179d807c7dbc7a484f1d91139e756e4e90b8faabbb537d9d2f8a4` |
| `run_arm.py` | 159 | `c2ba072ccda1004661c8d23299c17ddd0b57fae37b370eb984e08f20e9151ca0` |
| `pack_check.py` | 390 | `935bbe460c76b558d821028cd6af363190cb306b9191b94e3a8763aabb5cae3b` |
| `run_controls.py` | 172 | `ace514ce472f7e58123c7f12b823c061db93266d0d63f69045fbb1aef0be65db` |
| `controls.json` | 65 | `3cc3c563ccbf375c9d77ade62c1c29a7288704dbfebbce9e2d9ca40ed492fce0` |
| `scopes.json` | 55 | `32dd0054dd85cd54391ddd6eab9ad0e264464db48a6a88c71468021817f7a921` |
| `tables/rust.json` | 33 | `d2d3dc291c90a36a5d2c3df2ca97940a32221e02ae2d422c0ce7afdd37233dd1` |
| `tables/python.json` | 26 | `ebcdba056290add19156f2d516345fa99e7cf70dd5432ffe097b5e8c72ab9a7a` |
| `tables/typescript.json` | 29 | `bc65de93b20564ec4f2b2d9b14da5e0ebe53d7e95b2215ce14ec6c532fb6dd6b` |
| `tables/java.json` | 27 | `5367d13bc4d2b6ddc44d189d215844093514b2749d4d679edf9be716c74447e3` |

Drafts, not yet frozen (they may be revised from the development calls on
Rust and Python only): `guide-v2.md`
(`d32113204c3f6239a59575f0e5626d420332ac034a1bea69e6df7627d57ab6b2`) and
`pack-guide-v2.md`
(`46ea8bcdd6286fd9e928c41960a7d1da1c8d4eb6bccbe98294e27ac867269949`).

Second TypeScript scope, by the rule of Amendment 10c: two files qualified
(`config.ts`, 220 lines, 59 sites; `verification.ts`, 183 lines, 64 sites,
both in 4 kinds). The one with more sites is admitted:
`OpenUltraCode` `src/verification.ts` at `37a4e55`, SHA-256
`9908a3ad63cba13f813ca75a7fb441ff09c377698afe9dee62a738cbdee3c8c0`, tests
`tests/verification.test.ts` (7 passing). It has no comment, so the
comment control does not apply to it.

## Evidence and results

Eligible sites and mutants under the version 0 and version 1 hand packs
(sites / mutants):

| Scope | Version 0 | Version 1 | Version 1 mutants by kind |
| --- | --- | --- | --- |
| Rust `update.rs` | 33 / 16 | 71 / 66 | write 39, condition 16, failure 9, order 2 |
| Python `ratelimit.py` | 200 / 161 | 211 / 199 | write 121, condition 64, failure 9, order 5 |
| TypeScript `state.ts` | 50 / 28 | 70 / 74 | condition 44, write 19, failure 11 |
| TypeScript `verification.ts` | not a scope | 64 sites | counted in the D1 batch |
| Java `Lexer.java` | 157 / 115 | 184 / 189 | condition 99, write 69, order 11, failure 10 |

Every scope is at or below 200 mutants, so arm D runs all of them and no
selection is made.

Model-free checks, all passing:

- Validator controls: 95 of 95 on five scopes, including the new control for
  an anchor completed by the statement terminator.
- Engine checks: packs valid (kinds and tokens exist in each grammar),
  positions, all 528 mutants of the four original scopes are real changes
  confined to their statement or token, counts, selection, harness smoke
  test on four scopes, and a regression check that the version 0 packs still
  give exactly the sites and operators of the committed D1 receipts.
- Tool tests: 17 and 23 passing. The two model-written version 0 packs are
  still admitted and produce byte-identical mutants.

Things found that limit version 1 before any run:

- The Rust and Python scope files contain no returned condition, so the
  `returned` rule adds sites only in TypeScript and Java.
- Java's `instanceof` and Python's `is not` / `not in` do not match the
  token rule as written (a separate node kind, and compound tokens). Neither
  occurs in the scopes. Left as is.
- Most new Rust sites are calls that print (`p.say(...)`), 39 write sites
  where version 0 had 1. Whether dropping them is caught by the tests is what
  the batch will show.

## Limits and interpretation

- The second TypeScript scope comes from the same repository and author as
  the first. It tests another file, not another code base.
- The hand-written TypeScript and Java packs were again extended by the
  designer with knowledge of those scopes' rejected annotations. D2, with
  packs the model writes from the Rust and Python packs, stays the clean
  test of agnosticism.
- The designer's survivor labels are reduced to a cross-check sample
  (Amendment 11); the verdicts use the independent labels.

## Tool/spec consequence and next gate

Gate Amendment 11 records the admitted scope and the label procedure. Next:
development calls for the two guides on Rust and Python, freeze, then the
counted D1, D2 and A runs on five scopes, independent labels, and the gate
decision.

## COBESY check

Outcome first: setup, no result. Replay: hashes, rule and counts given.
Disproof: the two predictions that can fail are named. Known weaknesses of
the revision are listed before any run. No claim about the mechanism.
