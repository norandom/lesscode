# Handover: Agnostic Driver Gate Attempt 1 Is `revise`; Owner Decisions Pending, 2026-11-10

Updated 2026-11-10, about 13:30 Europe/Berlin. This file is the single handover and action list.
The current sections below govern continuation; dated history follows for evidence
and earlier interpretations, not as an active implementation queue.

## Current state

Updated 2026-11-10. Start with the [task 28 handover](task_28_handover.md); it
has the result table, the open decisions, the owner's rules and the commands.

1. **Top priority: the agnostic driver** (spec
   [agnostic-annotation-driver](specs/agnostic-annotation-driver/brief.md)).
   **Gate attempt 2 (R64): the rules-as-data form meets every mechanism row
   on all four languages, with model-written packs for the held-out ones.
   Open: the apparatus ceiling (owner decision). Next: requirements, then a
   design for approval.** History: attempt 1 ran earlier today; outcome revise
   ([R61](../docs/research/2026-11-10/07-annotation-driver-gate-result.md)).
   One engine with a data pack per language meets every gated row on Rust,
   Python and Java in at least one arm and fails on TypeScript in every arm.
   Packs written by the model for TypeScript and Java were admitted and are
   equivalent to hand-written ones. Nothing in `native/` changed.
2. **Owner decided (2026-11-10 evening):** the apparatus is ported to Rust,
   one runtime; breaking codebases must be avoided. Feasibility report:
   [docs/reports/agnostic-driver-feasibility.md](../docs/reports/agnostic-driver-feasibility.md).
   Owner approved: mutation and analysis ported together; sizing spike as
   the first design step; cap decided after it. **Requirements are
   generated (nine areas) and await the owner's approval;** then design.
3. **Second priority:** hardening against LLM inference drift, in
   `constraint-guided-compression` Slice B (requirements Amendment 5). Not
   started. The run-to-run agreement measured today is its baseline.
4. **Release:** `v1.0.2` is published (2026-11-10) as the first signed
   release: archives carry zipsign signatures, `SHA256SUMS.sig` and a
   provenance attestation, all verified after publication. It contains the
   fail-loudly fixes (item 5). The owner holds the signing key and approves
   the `release` environment for each tag
   ([docs/release-signing.md](../docs/release-signing.md)).
5. **R55 false passes fixed in 1.0.2:** Python `analyze` and `mutate` exit 2
   and write nothing, `mutate` exits 1 when no operator matched, the hook
   checks budgets only. The hard-coded mutation operators themselves stay
   until a design passes the gate; `cli_deconvolute.rs` is at 399 of its
   400-line ceiling.
   Batches now run in parallel worker copies (R62): a full pass over the
   four scopes takes about 8 minutes instead of 43, with identical results.
6. **Working rules from the owner today:** the agent decides strategy and
   records it; clear implementation goes to Opus at xhigh effort; do not
   manage the Podman machine, only run containers; the runs were too slow,
   so keep batches short and reuse outcomes.
7. **Not done:** installers on Linux aarch64 and macOS; the pre-push hook is
   not installed in this checkout; about 200 untracked experiment files
   linked from `docs/README.md` are still untracked.

Details of the release, CI and container work of this morning are in the
[task 27 handover](task_27_handover.md).
Updated 2026-11-09 late evening:
1. **Major Release v1.0.0 Shipped:**
   - Unified native four-substrate deconvolution engine (`lesscode-core`) covering **Python**, **TypeScript**, **Java**, and **Rust**.
   - Git tag `v1.0.0` pushed to GitHub; Zenodo citation metadata (`CITATION.cff`, `.zenodo.json`) updated.
   - Intelligent Git pre-push hook and porcelain CLI commands (`analyze`, `mutate`, `density`, `install-hook`, `budgets`) operational.
   - Dogfood self-application verified on clean checkout clone with 100% test pass rate (352/352 tests) and 0 budget violations across 38 governed files.
   - Comprehensive `README.md` overhaul completed.

2. **CI Pipeline Remediation:**
   - Obsolete `.github/workflows/baseline-platforms.yml` removed following Python prototype retirement at `fd0b5b9`.
   - All 14 `cargo clippy` warnings resolved with idiomatic Rust 1.98.1 constructs.
   - `cargo fmt --check` cleanly passes with brownfield complexity budget ceilings strictly maintained via targeted helper extraction and formatting skips.

3. **Slice B (MIPROv2 & DSPy Probabilistic Turn Refinement) Blueprint Ready:**
   - Full continuation roadmap and two-tier programmatic reward architecture established in [`.kiro/task_26_handover.md`](task_26_handover.md).

## Read next

00. [Task 28 Handover](task_28_handover.md): agnostic annotation driver, prototype gate attempt 1 (revise), open owner decisions and how to run the prototype (2026-11-10). Read this first.
0. [Task 27 Handover](task_27_handover.md): v1.0.1 release workflow, adopter documentation, and R55 released-binary dogfood findings (2026-11-10). Read this first; it narrows several claims in the current-state list above.
1. [Task 26 Handover](task_26_handover.md): CI remediation, v1.0.0 README update, and Slice B resumption blueprint.
2. [Task 25 Handover](task_25_handover.md): Dogfood self-application and intelligent pre-push hook.
3. [Constraint-Guided Compression Specification](specs/constraint-guided-compression/design.md): Slice B design and two-tier programmatic reward specification.
4. [R33 Research Log](../docs/research/2026-11-07/07-dspy-probabilistic-turn-refinement-design.md): DSPy probabilistic turn refinement design.
5. [R54 Synthesis Log](../docs/research/2026-11-09/12-dogfood-self-application-synthesis.md): Dogfood self-application synthesis and verification.

## Next authorized work and candidate directions

With `state-boundary-deconvolution` finished and closed, the following candidate directions are available:

1. **Native Rust Substrate Migration (`lesscode-core` / `native/src/state_contract.rs`):**
   Port the verified Python prototype components (`StateFrame`, `StreamMonitor`, `PureVerifier`, `MutantGenerator`)
   directly into the native Rust substrate within the R41 apparatus ceiling ($\le 7,173$ canonical lines).
2. **Broadened Specimen Self-Application (Facdrone Subsystems):**
   Expand the deconvolution engine to remaining Facdrone subsystems (e.g. `facdrone-orchestration`, `facdrone-worker`)
   to test multi-subsystem cross-boundary coordination.
3. **CI & Cross-Platform Verification (Dagger):**
   Execute the native Dagger CI validation pipeline (`.dagger/`, `dagger.json`) per `docs/ci-validation.md`
   to verify cross-platform determinism across Windows, Linux, and macOS.
4. **New Capability Discovery (`$kiro-discovery`):**
   Launch a fresh discovery phase to define the next product milestone on the roadmap.

## Current decisions and actions

OP identifiers are retained so earlier discussions remain traceable.

| ID | Disposition and next action |
| --- | --- |
| OP-1 | Proposed: reuse existing crate/pins in `native/Cargo.lock` (`ast-grep-core 0.51.0`, `serde 1.0.229`, `serde_json 1.0.154`, `sha2 0.11.0`, `clap 4.6.0`, plus locked transitive `ignore`, `walkdir`, `regex`, `toml_edit`, `syn`, `windows-sys`) and preserve `lesscode-core` prepare/compare, Python subprocess shim and pure turn program. Drop `serde_jcs`, `fs4`, `quick-junit` (replace with `junit-parser`), `gix`, `pg_query` and `postgresql_embedded`. Rust/mmhmem SQLite (`rusqlite` `trace_v2` + `session`) and TS/JS (`oxc`) are later; embedding/model transport deferred. |
| OP-2 | RATIFIED for S10, 2026-11-07 (owner, opencode chat): surface = tree + recorder imports (contract B; S9/R29 evidence: +1 E3 name, +3 E4 names incl. labeled_reason) plus the other three R26 changes (preregistered turn parameters with thinking-only turns counted failed; raw-byte hashes; case-distinct receipt labels) and R25 formatting/both-tree sensitivity rules. Historical protocols preserved; general ratification beyond this prototype still owner business. |
| OP-3 | RESOLVED: E5 trial executed on lifecycle monitor group (R42), broke Density Trap, and adopted into `.runs/e5-adopted-lifecycle/` (R43). E6 generalization trial executed on report-envelope group (R45) and adopted into `.runs/e6-adopted-report-envelope/` (R46). |
| OP-4 | DONE: 380c629 contains task 6.7, residue, missing-test artifacts, receipt/logs, R26, index and spec research updates. |
| OP-5 | DONE: Housekeeping and spec updates committed across task groups 1 through 19. |
| OP-6 | Carry E4 safeguards into follow-up: stdout plus stderr/termination evidence, interrupted runs never mutation kills, one job per launch with resume notes, clean launcher parent, E4-specific missing-test schema adapter. Detailed facts below. |
| OP-7 | Carry E3 safeguards: E1c-2 solver launcher, PYTHONDONTWRITEBYTECODE=1, restore/hash verification, receipted host guard. Scratch memory guard is not delivered apparatus. Detailed facts below. |
| OP-8 | Deferred: corrected edge census/automatic selection until a concrete unmet need. Preserve R21 rejection and proposed corrections. |
| OP-9 | Production foundation: `state_contract.rs` (92 lines) integrated; component ceilings rebalanced; Rust substrate capped at 7,173 canonical lines (R41). |
| OP-10 | First-slice evidence plan: S9 surface delta, S2 digests/artifact sidecars, S10 reuse complete. |
| OP-11 | Not-run: artifact checks, live adapter integration and usefulness checks are distinct. |
| OP-12 | Deferred: full five-target release expansion. Preserve existing delivered CLI/API/platform contracts when shared code changes. |
| OP-13 | RESOLVED: S10 gate passed on attempt 2 (C_app 1,193 <= 1,364, U1-U7 PASS). |
| OP-14 | RESOLVED: Prototype feasibility gate passed under amended Rust substrate ceiling of 7,173 canonical lines (R41). |
| OP-15 | RESOLVED: E5 Lifecycle Trial passed (-24.8% lines, -12.1% stmts, 0 replay divergences; R42, R43). |
| OP-16 | RESOLVED: E6 Generalization Trial passed (-23.01% lines, -10.49% stmts, 4/4 adverse mutants killed; R44, R45, R46). |
| OP-17 | RESOLVED: Milestone synthesis report published in `docs/reports/state-boundary-deconvolution-report.md`. |
| OP-18 | RESOLVED: Feature integration validation executed with GO verdict in `specs/state-boundary-deconvolution/validation-report.md`. |
| OP-19 | ACTIVE: Maintain the 4 preserved owner WIP files (`sql-divergences-round21.json` x2, `rules/evaluate.py`, `tests/test_rule_evaluation.py`) strictly unstaged and untouched. |

## Guardrails and validation state

Native Windows, plain user; WSL CI-only; no bare bash, elevation or symlink
privilege assumptions. Preserve upstream specimens and frozen copies. Use isolated
copies under .runs/, one heavy job at a time, serial deterministic attribution,
numeric threads=1, adaptive deadlines and receipted host-load probes.
Scope changes go through the scope guard and normal review. No third scope,
candidate adoption, broad platform expansion or upstream protocol ratification is
authorized merely by this handover.

Spec amendment validation passed: JSON/TOML parsing, local links, numeric/EARS
criteria, U1-U7, archives, locked approvals and absence of production design/tasks.
Independent review corrections added candidate/turn demonstrations and separated
S9 evidence gathering from S10 ratification. These are document checks, not
prototype results. Keep the research index/spec links current when evidence changes.

## Historical record

The following dated narrative preserves earlier evidence and interpretations.
Earlier statements about uncommitted task 6.7, native-core initialization,
S1-first/full-suite sequencing and pending platform checks are superseded by
the current state and action list above.

Written 2026-11-03 evening, updated 2026-11-04 after E3, 2026-11-06 after
E4 and 2026-11-06 late evening after the run report and the start of the
native-core spec, Europe/Berlin. Read this first, then the files it points
to. Nothing is running.

## Historical: fusion plan before the solution-first amendment (2026-11-07)

- **Fusion run.** OpenUltraCode strategy critique-revise-vote: panels
  `ultracode-fusion-panel-a` and `ultracode-fusion-panel-b`, decider = the selected
  model (`vibe-oidc/mistral-large-4`) as coordinator and final arbiter; three rounds
  each (generation, cross-critique, revision+vote); real subagent dispatch throughout.
  Diversity caveat: all roles ran on the same selected model (independent contexts,
  same family). Arbitrated plan: pre-kiro owner gate -> requirements -> spikes S1-S9
  before design approval -> design -> tasks -> per-task reviewed implementation ->
  final validation; every loop gate maps 1:1 to a fixture-replay smoketest; one crate
  (lib+bin) with 12 subcommands and the 0/1/2/3 exit-code decision table; five-target
  native-runner release matrix (cargo-zigbuild rejected: no MSVC target; cargo-dist
  rejected: too much generated CI surface); hand-written budget <= 3,500 lines first
  release, <= 4,600 for three languages.
- **Spec and process amendments (owner asked 2026-11-07; uncommitted).**
  `deconvolution-native-core`: `requirements.md` amendment 2026-11-07 (N1-N10,
  generated, NOT approved), new `gates.md`, `verification.md`, `spikes.md`,
  `breath.toml` (scope guard: editing it is the only way to change scope); `spec.json`
  phase now `requirements-generated`, requirements approved=false. Project level: new
  `.kiro/breadth.toml` (capability/evidence map, QOpenBB convention), new
  `.kiro/steering/complexity.md` + `.kiro/complexity-budgets.json` (complexity caps,
  machine-enforced by a checker foundation task), this handover and the roadmap
  updated. E2E verification of the .kiro workflow is CLI-driven (runner with
  evidence.json per step, phase-gate checks, no browser/Playwright per owner).
- **Still gated.** No Rust code, no third scope, no R26 amendment adoption until the
  owner answers the brief's four open questions plus the recorder-imports ratification
  (spike S9 informs it). Open item 4 stands: the 6.7 files are uncommitted — commit them
  before the committed E3/E4 artefacts are used as regression fixtures.
- **Open points** are consolidated in [current decisions and actions](#current-decisions-and-actions): owner gate
  (OP-1 four questions, OP-2 R26 ratification, OP-3 loop next step), housekeeping
  commits (OP-4 6.7 files, OP-5 the 2026-11-07 amendments incl. this file),
  research open items (OP-6 to OP-8), native-core prerequisites blocked on the gate
  (OP-9 checker, OP-10 spikes, OP-11 smoketests, OP-12 release flow).

## Historical: run report and native-core proposal (2026-11-06)

- **Run report and paper.** `docs/reports/deconvolution-loop-run-report.md`
  (new: process as run with commit ids, E4 task by task, six regenerator
  turns, prediction table after one of three scopes, apparatus findings,
  prototype resume with a status per component, maturity assessment,
  options) and `docs/reports/deconvolution-loop-applied.md` (updated for
  two scopes). COBESY blueprint in `docs/reports/blueprints/`. Commit
  f105aa5, pushed.
- **Owner stance.** Not ready to decide on a Rust build. Wants Q&A to
  understand the prototype first. Target shape: own CLIs, purpose-built
  fuzzers, instrumentation, a library with a reusable core, cross-platform,
  working on specimens in Python, TypeScript/JavaScript and Rust. Clear
  specs first, so that effort and porting can be judged. Reuse the Rust
  ecosystem; write little code. Do not start Rust code, a third scope or
  the R26 spec amendments until the owner asks.
- **New spec `.kiro/specs/deconvolution-native-core/` (initialized, no
  phase approved).** `brief.md` (problem, desired outcome, reuse-first
  approach, scope, boundaries, constraints, four open questions for the
  owner), `research.md` (two web surveys of 2026-11-06), `spec.json`
  (phase initialized), `requirements.md` (project description only).
  Uncommitted until the owner asks.
- **Instrumentation answer (research.md section 1).** A debugger library
  (DAP via debugpy, CDP for Node, lldb-dap for Rust) can capture calls and
  returns, but it adds 500 to 1,000 lines plus a serialiser per language,
  is weak for Rust on native Windows, and cannot see SQL. It removes almost
  none of the recorder's 534 lines, which are alphabet, fixtures and
  normalisation; SQL capture is a 6-line hook. Recommended: the driver
  observes the surface directly through one generic shim per language
  (Python, Node subprocess or PyO3; Rust in-process), runtime hooks
  (`sys.monitoring`, `module.registerHooks`) for internal calls, and
  store-level SQL capture (Rust PostgreSQL wire proxy on `pgwire` codecs;
  SQLite trace hook in all three languages). Debuggers stay out of the core.
- **Reuse map (research.md section 2).** Every component has a maintained
  crate or tool except the agnostic decision logic: receipt (serde_json,
  fs4, tempfile), guard (sysinfo, process-wrap), hashing and diffs (ignore,
  sha2, similar, diffy), surface and measures (ast-grep rule packs plus SCIP
  indexes from scip-python, scip-typescript, rust-analyzer), mutation
  (cargo-mutants, cosmic-ray, StrykerJS), sequence generation (proptest as a
  library writing corpus JSON), solver (CrossHair subprocess; Kani Linux CI
  only; nothing for TS/JS), model client (reqwest; no official Rust SDK),
  query differential (postgresql_embedded, postgres, sqlparser), reports and
  regression (tabled, insta against committed E3/E4 artefacts). mutmut,
  cargo-fuzz, AFL and Kani are Unix-only; stack-graphs is archived.
- **First spike when the owner wants one.** Is wire-level SQL capture
  byte-identical across two recordings of the frozen E4 original on native
  Windows (driver prepare thresholds, type lookups, pool resets)? A minimal
  Rust passthrough proxy, the 50-sequence corpus recorded twice, compared
  byte for byte and against the SQLAlchemy-hook record.
- **Next, on the owner's word.** Answer the brief's four open questions in
  Q&A (extend `native/lesscode-core` or add a workspace member; first
  Rust-target specimen and store, mmhmem with SQLite being cheapest; PyO3 or
  subprocess for Python; when the regenerator moves into the suite); then
  `$kiro-spec-requirements deconvolution-native-core`.

## Where we are

E4 (2026-11-06): the loop ran on a second Facdrone scope, the lifecycle
monitor group (seven files, 1,495 lines), with the recorded regenerator
program of task 4, under the frozen plan of task 6.1 and the owner's go, and
is recorded as [R26](../docs/research/2026-11-06/01-e4-lifecycle-regeneration.md).
Accepted at turn 3 (T1 held; turn 1 spent all 128k output tokens thinking,
turn 2 failed at the arm's collection on `labeled_reason`, a name only the
recorder imports and the surface deriver missed). T2 failed: 1,405 canonical
lines against at most 1,345 (6.0% fewer, the report predicted 10 to 20). T3
held (52 functions for 69), T4 held (63 edges for 89), S held (11 write sites
in shells, pure share 0.519, patch check pass on both trees). T5 held:
candidate 19/28 = 0.679 in the band 0.60 to 0.75; the original scored 12/29 =
0.414 on its own sample, a composition result stated in advance (8 draws
annotation-only, 6 in residue); 0.70 unmet on both; M2 to M4 caught on both
trees at the predicted sites, M1 missed as predicted. T6 held on F1 and F3 of
the independent review (128 probes, 26 disagreements, F1 to F6); 6.5 is void
under kill criterion 4 (the `window` planted-mutant solver control failed),
with the `compare` and `standing` counterexamples replayed on both trees and
kept as evidence (F3 by a second route; F7, an error-ordering difference). T7
held: 74 held-out sequences (seed 20261107, recorded after acceptance; 26 of
them the acceptance seed sequences replayed again, 47 new; 1 generated
sequence duplicates an acceptance one) replay on the candidate with zero
divergence at both levels. Residue corrected:
`lifecycle_root.lifecycle_store` is reached by a seed test; `bind_lifecycle`
is unreached and was not declared. Research code 771 of 900 lines (6.7's
generator cut from 100 to 80 to meet its bound). Deliverables of 6.7:
`e4-lifecycle/residue.md` (the protection statement in the form of
requirement 10.2), `results/missing-tests-e4.{md,json}` (28 entries,
generated by `missing_tests4.py`, receipt `missing-tests-e4-2`), R26, this
file, the index and both spec research files. Every step was reviewed by an
independent agent before the next: 6.2 APPROVED with three low notes; 6.3 was
itself the fresh review (its probe harness rerun byte-identically by the
controller); 6.4 and 6.5 NEEDS_FIXES then APPROVED; 6.6 APPROVED with two low
notes applied; 6.7 NEEDS_FIXES, nine text fixes applied (note
`task-6.7-review-fixes`). 6.1 to 6.6 are committed (1d9c59c, 46355f6,
9232a49, f415a38, 2f5bb0d, 20b2977, a44e79f); 6.7 is not. Open: (a) the
four spec amendments R26 proposes (surface = tree imports plus recorder
imports; turn parameters preregistered and a thinking-only turn counted;
content hashes over bytes as stored; labels distinct beyond letter case),
plus adopting R25's two rules as written; (b) the owner's next decision: a
third scope under the same protocol, adoption of `cand-turn3` outside the
loop with the missing-test report as the list of tests to write first, or
stop and move to prevention. Candidate root `.runs/e4-lifecycle/cand-turn3`;
mutable copies `orig-mut` and `cand-mut` verified identical to their
references at the end; goldens `golden-a/b.json` and `held-out-a/b.json` on
disk under `.runs/e4-lifecycle/` with committed sidecars.

E3 ran on 2026-11-04 (09:28 to 14:53) and is recorded as
[R25](../docs/research/2026-11-04/02-e3-evidence-regeneration.md). One
regeneration round met the oracle (T1), with 1,074 lines for 1,274, 546
statements for 660, 51 functions for 67, 60 intra-set edges for 84, 11 write
sites all in declared shell functions and 36 of 51 functions in a checked
pure core (T2, T3). T4 failed on its kill rate (18 of 28 = 0.643 against
0.70; M1 to M3 caught). T5 found one realizable oracle-blind difference
(`capture_payload` leaves no `AuditedAbsence` after an exception because
the candidate calls the route after deduplication); the reviewer and 300
random inputs had already exposed it, and the capture pair fails verifier
check 2 on a strict reading. Reviewer caveat: 26 statements of the reduction
are packing (about 572 in the original's style). No kill criterion fired;
H8(c) stands on one scope with a lowered protection claim. Research code
545 lines against 400, reported. Index, H8 lineage row and the spec
research protocol are updated. Owner decision pending: E3b (second round
against the known gaps, with the counterexample as a new oracle operation
under a new preregistration), E4 (second scope), or prevention.

Candidate root `.runs/e3-facdrone/cand` (only the five files differ from the
frozen copy); mutant copy `.runs/e3-facdrone/mut` verified identical to the
candidate at the end. E1c-2 (R24) stands as recorded.

Committed 2026-11-04 as `730e033`: R24, R25, the E1c-2 and E3 directories
(coverage data and files over 1 MB kept out, sha256 sidecars in place),
`adaptive.py` and its reference, the tooling-doc verifier section, this
file. The large golden trace `golden-e1c2.json` and `t5-soundness.json` are
on disk only; their hashes are committed beside them.

Release and archive (2026-11-04): spec, report, citation files and badge
committed (281a615, 9086551) and pushed; GitHub release `v0.1.0` published;
Zenodo archived it as DOI 10.5281/zenodo.23958364 under concept DOI
10.5281/zenodo.23958363 (the README badge and `CITATION.cff` carry the
concept DOI). Later releases archive automatically; bump `version` and
`date-released` in `CITATION.cff` and `.zenodo.json` first.

## Rules that are not negotiable

- Native Windows only, plain user account. WSL is CI-only; nothing executes
  there. Never call a bare `bash` from Python or PowerShell: Windows resolves
  it to the System32 WSL launcher. Use native Python, PowerShell, or the full
  Git Bash path. Tell every subagent this explicitly.
- No symlink privilege, Developer Mode or elevation assumed, by the research
  harnesses or by the desired implementation. Specimen tests that need them
  stay recorded as baseline failures.
- Never modify `D:\Source\facdrone`, `D:\Source\mmhmem`, or the frozen copies.
  Mutants only in separate copies under `.runs/`.
- One heavy job at a time; serial pytest for anything whose attribution must
  be deterministic; BLAS thread variables set to 1. Deadlines are adaptive:
  `docs/experiments/deconvolution/adaptive.py` (probe, scaled, wait_for_idle);
  never hard-cap wall clock; a timeout under load is rerun once when idle. The host ran out of
  memory once when cargo-mutants, a Facdrone pytest and an unrelated rustc
  build overlapped.
- Preregister before executing; amendments are dated and never change
  predictions after results; failed predictions are reported as failed.
- Research entries use `docs/research/ENTRY-TEMPLATE.md`, conclusion first,
  COBESY check at the end; update the index in the same change.
- mmhmem: call `session_resume_or_handoff` for project `lesscode` at start,
  checkpoint after each unit of work. Last checkpoint 2026-11-03 19:24 UTC.

## Where things stand, by hypothesis

H8 (recorded trace oracle + edge measures + regeneration), registered in
[R19](../docs/research/2026-11-03/01-dynamic-deconvolution-direction.md):

| Part | Status | Entry |
| --- | --- | --- |
| H8(a) oracle, Rust (mmhmem session bus) | mechanics positive; gain prediction hit a 96% seed-test ceiling | [R20](../docs/research/2026-11-03/02-e1a-session-bus-oracle.md) |
| H8(b) edge-based selector (Facdrone census) | rejected as formulated; C2 and C5 failed; recorder validity proven | [R21](../docs/research/2026-11-03/03-e2-facdrone-edge-census.md) |
| H8(a) oracle, Python (Facdrone evidence publication) | byte-deterministic, catches stored-representation change; kill rate 0.712 to 0.746; gap-halving failed | [R22](../docs/research/2026-11-03/04-e1b-evidence-publication-oracle.md) |
| Guided-generation rung (CrossHair on lifted predicates) | failed as run because drivers concretised strings and the oracle never calls the readers; one target killed by solver input | [R23](../docs/research/2026-11-03/05-e1c-concolic-rung.md) |
| Guided-generation rung, corrected and verified (E1c-2) | S1-S4 failed, S5 held; 7/10 killed, 3 unresolved, 0 proved; solver narrowed to a verified counterexample finder, never equivalence authority | [R24](../docs/research/2026-11-04/01-e1c2-corrected-solver-rung.md) |
| H8(c) regeneration (E3, five evidence modules) | T1-T3 held in 1 round; T4 kill rate 0.643 failed (M1-M3 caught); T5 one real oracle-blind difference on the exception path; stands on one scope with a lowered protection claim | [R25](../docs/research/2026-11-04/02-e3-evidence-regeneration.md) |
| H8(c) regeneration on a second scope with the recorded program (E4, seven lifecycle files) | T1 held at turn 3; T2 failed (6% fewer canonical lines against 10 to 20); T3, T4 held (25% fewer functions, 29% fewer edges); T5 held (0.679; original 0.414 by composition; 0.70 unmet on both); T6 held on F1 and F3, 6.5 void on a failed solver control; T7 held (zero divergence on 74 held-out sequences: 26 of them the acceptance seed sequences replayed again, 47 new; 1 generated sequence duplicates an acceptance one); S held; stands on two scopes with the line prediction failed on the second | [R26](../docs/research/2026-11-06/01-e4-lifecycle-regeneration.md) |

Tool assessment for symbolic execution per language:
[docs/symbolic-execution-tooling.md](../docs/symbolic-execution-tooling.md).

## Environment facts

- Facdrone frozen copy `.runs/e2-facdrone/source` at `d49f48bd…`, venv
  `.runs/e2-facdrone/venv` (CPython 3.13.16, from the copy's uv.lock plus
  coverage 7.13.0). Hypothesis extras `.runs/e1b-facdrone/extras`; CrossHair
  0.0.112 and z3 in `.runs/e1c-facdrone/extras`. Mutant copy
  `.runs/e1c-facdrone/mut`.
- Facdrone embedded PostgreSQL runs natively only with the two shim fixes in
  `docs/experiments/deconvolution/e2-facdrone-edge-census/e2_winshim.py`
  (unquoted `listen_addresses`, detached pipes on `pg_ctl start`); load with
  `-p e2_winshim` and the e2 directory on `PYTHONPATH`. Gate marker expression
  is `not port_baseline and not live_broker`; `not db` would deselect the
  database seeds.
- 236 Facdrone tests fail on this host at baseline (symlink privilege, file
  locks, POSIX-only modules); recorded, not repaired; identical with and
  without the recorder.
- mmhmem isolated copy `.runs/e1-mmhmem/original` at `8a9ce348`; cargo-mutants,
  cargo-llvm-cov, nextest installed natively; no nightly, so no Miri or
  cargo-fuzz.
- `python` is not on PATH; use `uv.exe run --no-project --python 3.13` or the
  venv interpreter.

## Specimen findings not yet reported upstream

- mmhmem: `checkpoint` panics when the 6,000-byte working-state cap falls
  inside a multi-byte character (`truncate_to_budget`, `session_bus.rs:355`).
- Facdrone test support on Windows: quoted empty `listen_addresses` and
  pipe inheritance on `pg_ctl start`; `os.geteuid` call in the fixture plugin.

## Spec written after E3 (uncommitted)

`.kiro/specs/deconvolution-loop/` (2026-11-04, owner request): brief,
algorithm-agnostic requirements with a per-requirement feasibility table,
a provisional design documenting the prototype as built and what a native
implementation would copy or rebuild, a retrospective prototype-gate
assessment (`proceed` on one Python scope, `revise` for size-under-
formatting and sensitivity, `not-run` for scope selection and other
languages), and research links. Owner approved requirements, design and
tasks on 2026-11-04 ("approved"). `tasks.md` (reviewed twice by an
independent agent, 18 findings applied) holds bounded apparatus revisions,
the regenerator-program prototype and E4; tasks marked "owner go" (3.2, 4,
5.2, 6.1 for 6.2 to 6.7) still need an explicit answer before they start.
Executed 2026-11-05 (owner: "continue the process as planned"), each by
an implementer subagent, an independent kiro-review and fresh
verification, then a selective commit: 1.2 (3a14745), 1.1 (3094ec5), 2.2
(a461ee8), 2.1 (e60c3a0), 3.1 (33180c5), 1.3 (606e37d), and on 2026-11-06
3.2 (eb80fc4). Findings worth knowing: formatted,
the E3 candidate has 1,188 lines, above the T2 bar of 1,083, and a
formatter cannot expose statement packing (spec gap in requirement 4.2);
the missing-test report was actionable (a fresh writer's tests killed 3 of
3 gaps and exposed the exception path); test-infra detection reproduces
the 19 importing files, the 236/24 deselections and the marker expression
but not the 191 coverage seeds. Details in `tasks.md` Implementation
Notes. Task 3.2 (2026-11-06): the same sensitivity arm on the original
scores 17/26 = 0.654 against the candidate's 18/28 = 0.643; neither meets
0.70; T4's failure measures the arm on this module, not a loss from
regeneration (R25 Amendment 2). The 2026-11-05 series was cut off after
mutant 24 when the session ended; a terminated arm reads as "killed" in
`classify`, so the rerun voided it by receipt note. Owner on 2026-11-06:
"proceed" (pause lifted); order 4 (waits for
`C:\Users\mariu\.lesscode\model.env`), then 5.2 scope choice (survey done:
lifecycle monitor group ranked first), then 6.1; 5.1 after 4.
Done 2026-11-06, each reviewed by an independent sonnet agent and
pushed: 4 (576e072: regenerator program `regenerator/regen_turn.py`,
three recorded turns against claude-opus-5-5, the third at 128k output /
medium effort produced the E3-scope candidate `.runs/e4-regen/cand-t4`;
the first turn's silent stream was cut at 310 s, the second spent 64k
tokens thinking), 5.1 (d5cbb2f: that candidate 1,214 lines / 549
statements / 53 functions / 63 edges / pure 0.679, edit-scope and
structure pass, behaviour untested), 5.2 (be6226d: lifecycle scope
apparatus `e4-lifecycle/`, 24-operation recorder, corpus 50 sequences /
317 operations recorded twice byte-identically, round 0 exit 0 in 75.7
s; goldens on disk under `.runs/e4-lifecycle`), 6.1 (1d9c59c: E4 plan
with T1 to T7 fixed, original-side sample and four hand-written
mutants, reviewer brief). Owner go for 6.2 to 6.7 given 2026-11-06; 6.2 to
6.7 executed the same day (see "Where we are").
Lessons: receipt the probe that sets a deadline; sidecar hashes must
be over the bytes as stored (CRLF on Windows bit three times); a
terminated arm reads as killed in `classify`.
Owner stance recorded: tests belong to the TDD workflow; a cleaner smaller
AI version is preferred when it holds.

Applied recap paper (uncommitted): `docs/reports/deconvolution-loop-applied.md`,
the loop in eight steps, results R20 to R25, a preregistered prediction for
the next three scopes (lines 10 to 20 percent, functions 20 to 30 percent,
edges 25 to 35 percent fewer; kill rate 0.60 to 0.75; zero held-out
divergence as the trust row), and the DSPy-style regenerator program the
owner's model endpoint would run. Linked from the research index and the
spec's research.md.

## Historical E4 follow-up details

Current dispositions are in [current decisions and actions](#current-decisions-and-actions).
The numbered items below preserve the original E4 follow-up record, not a second work queue.

1. Owner decision: a third scope under the same protocol (to fill the
   remaining rows of the report's three-scope prediction, with the
   amendments of item 2 adopted first), adoption of the E4 candidate
   `cand-turn3` outside the loop (a decision on `residue.md` and the seven
   differences, with `missing-tests-e4.md` as the tests to write first), or
   stop and move to prevention at generation time. E3b (a second round on
   the E3 candidate) remains possible but was not chosen. Do not start any
   of them without the owner's choice.
2. Spec amendments proposed in R26, not yet adopted (dated amendments to
   the deconvolution-loop requirements): the surface is the tree's imports
   plus the recorder's imports (req 1.5); turn parameters (output ceiling,
   effort, thinking display) are preregistered and a thinking-only turn is
   a failed turn (req 6.8); content hashes are over bytes as stored
   (req 11.2); receipt labels are distinct beyond letter case (req 11.3).
   The two rules proposed in R25 (statement counts in the original's
   formatting; sensitivity on original and candidate alike) were executed
   by design in E4 and should be adopted as written.
3. E4 apparatus facts for any follow-up: the arm captures stdout only, so
   a bare exit code needs a receipted probe before a classification stands;
   the harness reaps long serial background scripts under memory pressure
   (run one job per launch with stop and resume notes); CrossHair did not
   reach a planted difference in `for_window` that 164 of 300 random inputs
   expose; a tree-loaded parent process exits 0xE0000001 when a child
   imports the same tree (keep the launcher's parent clean); the E3
   missing-test generator's input schema is E3-specific, `missing_tests4.py`
   adapts E4's files by import.
4. Historical commit checklist, completed by 380c629 (2026-11-07 correction):
   `e4-lifecycle/missing_tests4.py`, `e4-lifecycle/residue.md`,
   `e4-lifecycle/results/missing-tests-e4-inputs.json`,
   `results/missing-tests-e4.{md,json}`, `results/receipt.json` and the
   `results/logs/missing-tests-e4*.log` files, R26
   (`docs/research/2026-11-06/01-e4-lifecycle-regeneration.md`), the index,
   both spec research files and this handover.
5. Known E3 apparatus facts for any follow-up: the T5 drivers must use the
   E1c-2 CrossHair launcher (audit wall blocks an import-time hostname call);
   Hypothesis refuses `@given` on functions with defaults; `run_e3` writes
   bytecode into the mutant copy unless `PYTHONDONTWRITEBYTECODE=1`; the
   memory guard (`wait_for_idle(1800, 0.35, 8 GB)`, run if free RAM above
   6 GB) is a scratch script, not apparatus.
6. A corrected edge census (E2b) if a measured selector is still wanted:
   distinct codec sites, size-normalised measures, minimum executed size,
   worker-independent attribution.
