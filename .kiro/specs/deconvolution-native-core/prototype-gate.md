# Solution-unlock gate

Written 2026-11-07; amended 2026-11-07 after empirical baseline complexity audit.
Status: **not-run**. No execution or phase approval.

## Key and authority
spec.json.solution_unlock is an evidence pointer, not a pass switch.
Production design approval/tasks/code require status=proceed,
evidence_review=approved, U1-U7 passed on one frozen attempt, available durable
artifacts with matching identities, plus normal owner phase approvals.
Missing/stale evidence, absent fields, not-run/revise/stop mean locked.
Neither -y, a build nor editing the key creates evidence.
Until a checker exists, reviewers apply this checklist explicitly; do not claim
automated enforcement. Later automation must enforce these same conditions.

Provisional design and bounded planning may precede unlock. Execution requires
owner approval of the frozen plan and permits only that disposable prototype.
No third regeneration scope, adoption or production implementation is implied.

## Preregister before execution
For S10, resolve and freeze the ratified observation/surface contract before its
execution. S9 instead may compare both explicitly frozen alternative surface
contracts under its own separately approved bounded plan; S9 neither adopts an
amendment nor requires prior ratification of the change it is investigating.
Freeze E3/E4 revisions and dirty hashes, equivalent baseline responsibilities and
complete file/caller inventories; protocol/owner decisions including surface
ratification; observations, normalizations, alphabets, seeds, baseline deselections,
controls; tools/locks/runtime/evaluator identities; commands/host rules; artifact
bytes/digests/retrieval locations (including explicit `--artifact-root` bindings to
local `.runs/` golden traces verified against committed `.sha256` sidecars);
numerical accounting and attempt budgets.
A sidecar without verified artifact bytes is unavailable evidence.

## Measured baseline anatomy (ruff 0.17.3 canonical formatting, line-length 100)
Empirical pre-audit of the E3/E4 baseline apparatus shows that dense one-liners
expand by +78% to +136% under canonical formatting (`2,222` as-written -> `3,963`
canonical lines across 14 files):
- `e1b_oracle.py`: `406` written -> `659` canonical (`145` shared `Norm`/helpers,
  `175` E3 `Env` ops/fixtures, `174` E3 `SEEDS`+generator, `159` generic runner).
- `e4_oracle.py`: `534` written -> `1,258` canonical (`160` E4 fixtures/helpers,
  `472` E4 `Env` ops, `470` E4 `SEEDS`+generator, `150` generic runner).
- Arm & replay scripts (`run_arm.py` 221/381, `run_e3.py` 44/63, `test_e3_replay.py`
  33/58, `run_e4.py` 77/146, `test_e4_replay.py` 32/62): `407` written -> `710` canonical.
- Shared/sentinel checks (`receipt_atomic.py` 153/192, `editscope.py` 65/111,
  `patchcheck.py` 60/203, `scope.py` 78/105): `356` written -> `611` canonical.
- `regen_turn.py`: `337` written -> `502` canonical (`~240` pure prompt/feedback/parse/write;
  `~262` HTTP streaming client).

Because `1,498` canonical lines in `e1b_oracle.py` and `e4_oracle.py` are scope-specific
domain operation handlers (`op_*`, fixtures: `854` canonical lines) and transcribed test
corpora (`SEEDS` + `generated`: `644` canonical lines) that must never be shortened or
weakened to manufacture compression, baseline $B$ and replacement $C$ are partitioned
into three explicit strata:
1. $B_{\text{app}}, C_{\text{app}}$ (**Replaceable apparatus & per-scope wiring**):
   generic recorder/replayer runner, `Norm`/diff helpers, arm/pytest group runner,
   replay hooks, receipt/editscope/patch checks, pure turn-input transformation and
   per-scope adapter wiring (`~1,705` canonical lines in baseline).
2. $B_{\text{ops}}, C_{\text{ops}}$ (**Specimen domain operations**): per-scope `op_*`
   methods and domain fixture constants (`~854` canonical lines in baseline).
3. $B_{\text{seeds}}, C_{\text{seeds}}$ (**Corpus seed sequences & generators**):
   `SEEDS` and `generated(n)` (`~644` canonical lines in baseline; immutable unless a
   new corpus version is registered).

## Proposed usefulness targets
Count shared code once on each side. Require ALL of the following under identical
canonical formatting (`rustfmt` for Rust; `ruff==0.17.3 format --line-length 100`
for Python):
1. **Apparatus & wiring compression**: $C_{\text{app}} \le 0.80 \cdot B_{\text{app}}$
   AND $B_{\text{app}} - C_{\text{app}} \ge 200$ canonical physical lines.
2. **Whole-replacement net reduction**: $C_{\text{total}} \le B_{\text{total}} - 200$
   canonical physical lines (where $\text{total} = \text{app} + \text{ops} + \text{seeds}$),
   with $C_{\text{seeds}} = B_{\text{seeds}}$ (zero seed sequences or generator branches dropped).
3. **Per-scope onboarding savings**: strictly fewer scope-specific maintained lines on
   EACH scope (E3 and E4) when comparing the scope's complete adapter + wiring against
   its baseline counterpart.
4. Report separated-form statements and as-written lines alongside canonical physical
   lines. Report dependency/integration burden; relocating custom logic is not deletion.
   Tests/fixtures are separate and cannot hide implementation.
Independently review responsibility parity and freeze numeric $B_{\text{app}}, B_{\text{total}}$
and derived ceilings before execution. Never lower targets post hoc.

## Mandatory evidence
| ID | Pass condition | Required evidence |
| --- | --- | --- |
| U1 | Complete boundary, observations and 3-stratum accounting frozen; artifact bytes verified against committed sidecars | Approved plan, inventories, raw-byte hashes |
| U2 | Live recording/replay on BOTH scopes | Two recordings per original identical after frozen normalization; original replay zero L1/L2 divergence; unchanged seeds/harnesses; actual replay of frozen E3/E4 candidates with preregistered expected acceptance decisions under the applicable contract, including known differences/residue |
| U3 | Adverse controls fail closed | Planted L1 difference detected; stale/missing/partial results refused; protected edit void; L2 gap without query evidence refused; interrupted arm not a kill |
| U4 | E4 uses frozen shared core and descriptor semantics | Shared code frozen after E3; only declared E4 scope support changes; diffs/hashes |
| U5 | Useful complete replacement | $C_{\text{app}} \le 0.80 \cdot B_{\text{app}}$, $B_{\text{app}} - C_{\text{app}} \ge 200$, $C_{\text{total}} \le B_{\text{total}} - 200$, each scope's support smaller, zero seed/observation loss; 3-stratum physical/separated-form reports |
| U6 | Compatibility and truthful protection | Existing prepare/compare executable/arguments/results/exits checked; differences/residue faithfully reported; pure turn-input transformation checked for accepted and failed turns against frozen protocol examples, including exact permitted feedback and retained failed attempts |
| U7 | Bounded effort and reproducible evidence | <=600 added/modified canonical maintained lines of shared + adapter-wiring code cumulatively, <=2 attempts; receipts and independent review |

## U7 effort-charging rules (fail-closed, no brownfield deadlock)
1. **New and modified apparatus/wiring files charged in full**: Charge the union of all
   added or modified shared Rust files, shared Python runner/shim files and new/modified
   adapter wiring across both attempts at each file's largest observed canonical size.
   Deletions do not refund budget; restarts consume attempts.
2. **Pre-existing brownfield entrypoints (`native/src/lib.rs` at 618 lines, `main.rs` at 225 lines)**:
   Prefer a dedicated binary or submodule file so existing oversized files remain untouched.
   If `lib.rs` or `main.rs` is touched solely to register a module or subcommand dispatch
   ($\le 20$ changed lines, zero domain logic added, verified by diff), charge the changed lines
   rather than the pre-existing 618/225 lines; any broader edit charges the file in full.
3. **Verbatim-migrated `op_*` and `SEEDS` blocks**: When extracting existing E3/E4
   `op_*` methods, domain constants and `SEEDS`/`generated` blocks out of `e1b_oracle.py` /
   `e4_oracle.py`, verify verbatim AST/diff parity against the frozen originals. Unchanged
   migrated `op_*`/`SEEDS` bodies are counted in $C_{\text{ops}}$ and $C_{\text{seeds}}$
   for `U5`, while any new or modified adapter glue/wrapper lines around them are charged to `U7`.

A shared-core change during E4 onboarding fails U4. A second attempt requires a
dated hypothesis revision/new preregistration and replay of both scopes. E4 is
then known: claim bounded known-scope reuse, never unseen-scope generalization.

## Verdict
proceed: all U1-U7 plus independent review pass; only demonstrated slice may seek
normal production design approval.
revise: failed/missing criterion with a concrete bounded revision.
stop: effort/attempt limit exhausted or preserving observations defeats usefulness.
not-run: initial state, no execution evidence.

Retain failures, controls and overruns. Material input/contract/dependency/evaluator/
accounting changes invalidate affected claims. Prior regeneration results do not
satisfy this new gate. Record attempts in dated research entries with execution
dates and reciprocal spec/index links. Current evidence fields remain empty.
