# R57: Four scopes outside the specimens admitted for the annotation-driver gate; the fixed-operator control yields 0, 0, 1 and 0 mutants on them

- Report date/time and timezone: 2026-11-10, Europe/Berlin (UTC+1)
- Execution interval: 2026-11-10, about 09:55 to 10:20 local, one session
- Status: completed (gate checklist step 1, screening and admission only)
- Spec: [.kiro/specs/agnostic-annotation-driver/prototype-gate.md](../../../.kiro/specs/agnostic-annotation-driver/prototype-gate.md), Amendment 1
- Hypothesis ID and predecessor: H-AD1, [R56](02-agnostic-annotation-driver-direction.md)
- Type: preregistered (setup stage; no hypothesis test in this entry)

## Decision

Setup only. Four real scopes, one per language, are admitted and their test
baselines are healthy in isolated clones. This permits the next checklist
steps: the validator controls (no model), then guide version 0. It says
nothing yet about extraction, fidelity, behavior under mutation, compression
or adoption. No model call was made and nothing was spent.

The control expectation in the gate was wrong for one scope and is corrected
by a dated pre-run amendment. An additional arm is proposed and is not in
force until the owner confirms it.

## Hypothesis and falsification

Unchanged from R56 and the gate. This entry tests no hypothesis. The only
prediction it could check was the gate's statement that the fixed operators
would yield no mutants on admitted scopes; that failed on the TypeScript
scope (1 mutant) and held on the other three.

## Frozen inputs and method

Isolated clones made with `git clone` from the local checkouts into
`D:\BuildArtifacts\lesscode\annotation-driver\scopes` at each repository's
committed `HEAD`. Uncommitted changes in the upstream working trees (`mmhmem`
18 paths, `mvibe` 11 paths) are therefore not part of any scope. Upstream
checkouts were only read.

| Language | Role | Repository and revision | Scope file | SHA-256 of scope file | License |
| --- | --- | --- | --- | --- | --- |
| Rust | development | `mmhmem` `f32596f8c3117aa05a0d4db924a4de18d80f42d1` | `src/setup/check.rs` | `36eff9a6aaa7c3644f72b1d8c6ddc5c2d1899c0cd79acbadc6c5167524783788` | owner's repository, no license file |
| Python | development | `mvibe` `b56d0e37a716e77c6237f9aa5fe230745f76742c` | `mvibe_proxy/ratelimit.py` | `027e304481efc745ec8c04b9784d8e3efa54fd8516d197058ea4c928ffab6828` | Beer-ware |
| TypeScript | held-out | `OpenUltraCode` `37a4e55f1b9397332acc6434783099a631a04918` | `src/state.ts` | `3c9afdb17729b713455f3ee768f05e54e2511f4dadb550c9a70a4455deac7ad9` | MIT |
| Java | held-out | `commons-csv` `2bd66bf181d44ffc92806f585d3955e915666f7f` | `src/main/java/org/apache/commons/csv/Lexer.java` | `8a57dc0759e26dc91f79942d8cc0a1eb6c3f53449fb31da4dedf4a6cb9ff0e0c` | Apache-2.0 |

Screening rule: tracked source files of 150 to 800 canonical lines (non-blank,
not starting with a comment marker) that have a test file or test binary of
their own, excluding the earlier specimens (`CSVPrinter.java`, `slugify.ts`,
`degradation.ts`, the Facdrone and Lesscode lifecycle responsibilities).
Selection among the candidates was by hand: the scope with the clearest
single responsibility and the most tests relative to its size. That choice is
a manual step.

Tools: Rust 1.98.1 with cargo-nextest; uv with CPython per `mvibe`'s lock;
Node with pnpm 11.4.0 and the repository's `node --import tsx --test`;
Maven 3.9 with Temurin 21 in a container through
[compose.yaml](../../experiments/annotation-driver/compose.yaml). Fixed
operators: `lesscode-core` 1.0.1 release build of this repository.

## Evidence and results

| Scope | Canonical lines | Baseline command (in the clone) | Result | Arm B mutants |
| --- | --- | --- | --- | --- |
| `mmhmem` `src/setup/check.rs` | 212 | `cargo nextest run --locked --lib --test setup_check -E 'test(setup::check) \| binary(setup_check)'` | 28 passed | 0 |
| `mvibe` `mvibe_proxy/ratelimit.py` | 478 | `uv run --no-sync python -m pytest tests/test_ratelimit.py -q` | 230 passed | 0 |
| `OpenUltraCode` `src/state.ts` | 188 | `node --import tsx --test tests/state.test.ts` | 8 passed | 1 |
| `commons-csv` `Lexer.java` | 319 | `mvn -Dtest=LexerTest test` with audit and report plugins skipped | 37 passed | 0 |

Failed attempts, kept on record:

- The first two Rust baseline runs built every test target of `mmhmem` and
  failed. At this revision the integration test `mcp_tools` does not compile
  (61 errors), and other cargo processes were running on the shared machine
  during the first run. Restricting the build to the library and the
  `setup_check` test target gave the passing run above. The scope's own tests
  are unaffected; the consequence is that mutation runs on this scope must use
  the same restricted target list.
- Three Rust candidates considered first (`config/local_dirs.rs`,
  `storage/audit_rotate.rs`, `hook/codex.rs`) were dropped because more than
  half of each file is an inline test module, leaving the production part
  under the 150-line floor.

Logs are under `D:\BuildArtifacts\lesscode\annotation-driver\baseline-logs`
(local, not committed). The counts above are transcribed from them.

## Limits and interpretation

- Each scope is one file plus its tests. Whether that is the complete owned
  implementation of the responsibility, including helpers elsewhere, was not
  traced. For mutation this matters less than for compression, but it is not
  the whole-responsibility closure the project uses elsewhere.
- Test strength differs widely: 230 tests for 478 lines in Python, 8 tests for
  188 lines in TypeScript. A low kill rate on the TypeScript scope may reflect
  its tests, not the mechanism; the review labels are there to separate the
  two.
- All four repositories are the owner's or a well-known library. Nothing here
  speaks to unfamiliar code styles.
- The one arm B mutant on TypeScript shows the fixed operators are not
  strictly specimen-only for that language; some of their patterns are
  generic enough to match elsewhere.

## Tool/spec consequence and next gate

- Gate Amendment 1 records the admitted scopes, corrects the yield row (arm A
  must land more mutants than arm B on the scope, in addition to the fixed
  minimum), and carries the proposed arm D.
- Proposed arm D (pending the owner): the generic operators as declarative
  syntax-tree rules applied without a model (D1), and rule packs for the
  held-out languages written by the model offline and admitted by checks
  (D2). If either meets the gate, inference leaves the run-time path, which
  serves the second priority directly.
- Next: owner confirmation of arm D, then checklist step 2, the validator and
  its controls. No model call before the guide, tables and validator rules
  are frozen in a further entry.

## COBESY check

Outcome first: setup done, nothing tested. Replay: revisions, file hashes,
commands and tool versions are listed. Disproof: the one checkable prediction
failed on one scope and is reported and corrected. Failed attempts are kept.
Facts are in the tables; the scope choice is marked as manual. No feasibility
claim is made.
